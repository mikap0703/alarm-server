use std::sync::Arc;
use std::thread;
use std::time::Duration;
use flume::Sender;
use imap::{Connection, Session};
use imap::types::{Fetch, Fetches, UnsolicitedResponse};
use mailparse::{parse_mail, MailHeaderMap, ParsedMail};
use regex::Regex;
use tokio::sync::Mutex;
use crate::alarm::Alarm;
use crate::config::alarm_sources::MailConfig;
use crate::mail_parser::{MailParser};
use crate::mail_parser::sl_secur_cad::SecurCadParser;
use crate::mail_parser::mock_parser::MockParser;
use crate::mail_parser::plaintext_parser::PlaintextParser;

pub struct MailHandler {
    config: MailConfig,
    send_alarms: Sender<Alarm>,
    debug: bool,
    mailparser: Box<dyn MailParser>,
    seen_mails: Arc<Mutex<Vec<u32>>>
}

#[derive(Debug, Clone)]
struct MailData {
    subject: String,
    sender: String,
    text_body: String,
    html_body: String,
}

impl MailHandler {
    pub fn new(config: MailConfig, send_alarms: Sender<Alarm>, debug: bool) -> MailHandler {
        let mailparser: Box<dyn MailParser> = match config.mail_schema.as_str() {
            "SL-securCAD" => Box::new(SecurCadParser),
            "Plaintext" => Box::new(PlaintextParser),
            _ => Box::new(MockParser),
        };
        let seen_mails = Arc::new(Mutex::new(Vec::new()));

        Self { config, send_alarms, debug, mailparser, seen_mails }
    }

    pub fn start(&self) {
        let (send_mails, recv_mails) = flume::unbounded();

        // Start a thread for the idle loop
        {
            let send_mails = send_mails.clone();
            let client = imap::ClientBuilder::new(self.config.host.as_str(), self.config.port)
                .connect()
                .expect("Could not connect to imap server");

            let mut imap = client
                .login(self.config.user.as_str(), self.config.password.as_str())
                .expect("Could not login to imap server");

            imap.select("INBOX").expect("Could not select mailbox");

            // imap.debug = self.debug;
            imap.debug = true;
            thread::spawn(move || {
                // Start the idle loop and mail checking loop
                MailHandler::idle_loop(&mut imap, send_mails);
            });
        }

        // Start a thread for the mail checking loop
        {
            let send_mails = send_mails.clone();
            let client = imap::ClientBuilder::new(self.config.host.as_str(), self.config.port)
                .connect()
                .expect("Could not connect to imap server");

            let mut imap = client
                .login(self.config.user.as_str(), self.config.password.as_str())
                .expect("Could not login to imap server");

            imap.select("INBOX").expect("Could not select mailbox");

            // imap.debug = self.debug;
            imap.debug = true;
            thread::spawn(move || {
                // Start the idle loop and mail checking loop
                MailHandler::check_for_new_mail(&mut imap, send_mails);
            });
        }

        // Start a thread for the mail sending loop
        loop {
            match recv_mails.recv() {
                Ok(mail) => {
                    if self.handle_mail(mail) {
                        println!("Mail was handled");
                    } else {
                        println!("Mail was not handled");
                    }
                },
                Err(e) => {
                    println!("Could not receive mail: {:?}", e);
                }
            }
        }
    }

    fn idle_loop(imap: &mut Session<Connection>, send_mails: Sender<MailData>) {
        'idle_loop: loop {
            println!("Warten auf neue Mails...");
            let mut new_mail_id = None;
            let idle_result = imap.idle().timeout(Duration::new(120, 0)).keepalive(true).wait_while(|response| match response {
                UnsolicitedResponse::Exists(mail_id) => {
                    new_mail_id = Some(mail_id);
                    false
                }
                _ => {
                    println!("No new mail received");
                    true
                },
            });

            let new_mail_id = match new_mail_id {
                Some(id) => id,
                None => continue 'idle_loop,
            };

            match idle_result {
                Ok(_) => {
                    // Fetch the mail
                    let messages = match imap.fetch(new_mail_id.to_string(), "RFC822") {
                        Ok(messages) => messages,
                        Err(e) => {
                            println!("Could not fetch mail: {:?}", e);
                            continue 'idle_loop;
                        }
                    };
                    MailHandler::parse_forward_mail(send_mails.clone(), messages);
                }
                Err(e) => println!("IDLE finished with error {:?}", e),
            }
        }
    }

    async fn check_for_new_mail(imap: &mut Session<Connection>, send_mails: Sender<MailData>) {
        loop {
            println!("Checking for new mail...");
            let message_ids = match imap.search("UNSEEN") {
                Ok(ids) => ids,
                Err(e) => {
                    println!("Could not search for new mail: {:?}", e);
                    continue;
                }
            };

            for message_id in message_ids.iter() {
                let messages: Fetches = match imap.fetch(message_id.to_string(), "RFC822") {
                    Ok(messages) => messages,
                    Err(e) => {
                        println!("Could not fetch mail: {:?}", e);
                        continue;
                    }
                };
                MailHandler::parse_forward_mail(send_mails.clone(), messages);
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    }

    fn parse_forward_mail(send_mails: Sender<MailData>, messages: Fetches) {
        if let Some(message) = messages.iter().next() {
            if let Some(body) = message.body() {
                match parse_mail(body) {
                    Ok(parsed_mail) => {
                        let (text_body, html_body) = MailHandler::extract_bodies(&parsed_mail);
                        let subject = parsed_mail.headers.get_first_value("Subject").unwrap_or_default();
                        let sender = parsed_mail.headers.get_first_value("From").unwrap_or_default();

                        let mail_data = MailData {
                            subject,
                            sender,
                            text_body,
                            html_body,
                        };

                        let _ = send_mails.send(mail_data);
                    }
                    Err(e) => println!("Could not parse mail: {:?}", e),
                }
            }
        }
    }

    fn handle_mail(&self, mail_data: MailData) -> bool {
        println!("Handling mail: {}: <{}>", mail_data.subject, mail_data.sender);

        // Validate mail
        if (self.config.alarm_subject != "*") && (mail_data.subject != self.config.alarm_subject) {
            println!(
                "Subject mismatch: '{}' != '{}'",
                mail_data.subject, self.config.alarm_subject
            );
            return true;
        }
        if (self.config.alarm_sender != "*") && (mail_data.sender != self.config.alarm_sender) {
            println!(
                "Sender mismatch: '{}' != '{}'",
                mail_data.sender, self.config.alarm_sender
            );
            return true;
        }

        let mut alarm = Alarm::new();
        alarm.alarm_source(self.config.name.clone());

        match self.mailparser.parse(&mail_data.text_body, &mail_data.html_body, &mut alarm, self.config.clone()) {
            Ok(_) => {
                self.send_alarms.send(alarm).unwrap();
                false
            }
            Err(e) => {
                println!("Could not parse mail: {}", e);
                true
            }
        }
    }


    fn extract_bodies(parsed_mail: &ParsedMail) -> (String, String) {
        let mut text_body = String::new();
        let mut html_body = String::new();

        let body = &parsed_mail.get_body();
        match body {
            Ok(body_str) => {
                match parsed_mail.ctype.mimetype.as_str() {
                    "text/plain" => text_body = body_str.to_string(),
                    "text/html" => html_body = body_str.to_string(),
                    _ => {}
                }
            }
            Err(_) => {}
        }

        fn traverse_parts(mail: &ParsedMail, text_body: &mut String, html_body: &mut String) {
            for part in &mail.subparts {
                let content_type = &part.ctype.mimetype;
                let body = match part.get_body() {
                    Ok(body) => body,
                    Err(_) => continue,
                };

                if content_type == "text/plain" {
                    *text_body += &body;
                } else if content_type == "text/html" {
                    *html_body += &body;
                }

                // Recurse into nested parts
                traverse_parts(part, text_body, html_body);
            }
        }

        traverse_parts(parsed_mail, &mut text_body, &mut html_body);
        (text_body, html_body)
    }
}
