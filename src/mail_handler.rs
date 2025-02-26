use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::thread;
use std::time::Duration;
use colored::Colorize;
use flume::Sender;
use imap::{Connection, Session};
use imap::types::{Fetches, UnsolicitedResponse};
use mailparse::{parse_mail, MailHeaderMap, ParsedMail};
use regex::Regex;
use twox_hash::XxHash64;
use crate::alarm::Alarm;
use crate::config::alarm_sources::MailConfig;
use crate::mail_parser::{MailParser};
use crate::mail_parser::sl_secur_cad::SecurCadParser;
use crate::mail_parser::mock_parser::MockParser;
use crate::mail_parser::plaintext_parser::PlaintextParser;
use log::{debug, error, info, warn};


pub struct MailHandler {
    config: MailConfig,
    send_alarms: Sender<Alarm>,
    debug: bool,
    mailparser: Box<dyn MailParser>,
}

#[derive(Debug, Clone)]
struct MailData {
    subject: String,
    sender: String,
    text_body: String,
    html_body: String,
}

impl MailData {
    /// Generate a hash of all the fields combined.
    fn calculate_hash(&self) -> u64 {
        let mut hasher = XxHash64::default();
        self.subject.hash(&mut hasher);
        self.sender.hash(&mut hasher);
        self.text_body.hash(&mut hasher);
        self.html_body.hash(&mut hasher);
        hasher.finish()
    }
}

impl MailHandler {
    pub fn new(config: MailConfig, send_alarms: Sender<Alarm>, debug: bool) -> MailHandler {
        let mailparser: Box<dyn MailParser> = match config.mail_schema.as_str() {
            "SL-securCAD" => Box::new(SecurCadParser),
            "Plaintext" => Box::new(PlaintextParser),
            _ => Box::new(MockParser),
        };

        Self { config, send_alarms, debug, mailparser }
    }

    pub fn start(&self) {
        let (send_mails, recv_mails) = flume::unbounded();

        let inbox_name = self.config.name.purple();

        info!("{} MailHandler wird gestartet", inbox_name);

        // Start a thread for the idle loop
        if self.config.idle {
            let send_mails = send_mails.clone();
            let client = imap::ClientBuilder::new(self.config.host.as_str(), self.config.port)
                .connect()
                .expect("Could not connect to imap server");

            let mut imap = client
                .login(self.config.user.as_str(), self.config.password.as_str())
                .expect("Could not login to imap server");

            imap.select("INBOX").expect("Could not select mailbox");

            info!("{} Idle loop wird gestartet", inbox_name);

            imap.debug = self.debug;

            thread::spawn(move || {
                // Start the idle loop and mail checking loop
                MailHandler::idle_loop(&mut imap, send_mails);
            });
            info!("Idle loop started");
        }

        // Start a thread for the mail checking loop
        if self.config.polling {
            let send_mails = send_mails.clone();
            let client = imap::ClientBuilder::new(self.config.host.as_str(), self.config.port)
                .connect()
                .expect("Could not connect to imap server");

            let mut imap = client
                .login(self.config.user.as_str(), self.config.password.as_str())
                .expect("Could not login to imap server");

            imap.select("INBOX").expect("Could not select mailbox");

            info!("{} Polling loop wird gestartet", inbox_name);

            let interval = Duration::from_secs(self.config.polling_interval);

            imap.debug = self.debug;

            thread::spawn(move || {
                // Start the idle loop and mail checking loop
                MailHandler::polling_loop(&mut imap, send_mails, interval);
            });
        }

        // Start a thread for the mail sending loop
        let mut seen_mails = VecDeque::with_capacity(3);
        loop {
            match recv_mails.recv() {
                Ok(mail) => {
                    let hash = mail.calculate_hash();
                    if seen_mails.contains(&hash) {
                        info!("Mail already seen - skipping");
                        continue;
                    } else {
                        seen_mails.push_back(hash);
                        if seen_mails.len() > 3 {
                            seen_mails.pop_front();
                        }
                    }

                    if self.handle_mail(mail) {
                        info!("Mail was handled successfully");
                    } else {
                        error!("Mail was not handled or an error occurred");
                    }
                },
                Err(e) => {
                    error!("Could not receive mail: {:?}", e);
                }
            }
        }
    }

    fn idle_loop(imap: &mut Session<Connection>, send_mails: Sender<MailData>) {
        'idle_loop: loop {
            info!("Warten auf neue Mails...");
            let mut new_mail_id = None;
            let idle_result = imap.idle().timeout(Duration::new(120, 0)).keepalive(true).wait_while(|response| match response {
                UnsolicitedResponse::Exists(mail_id) => {
                    new_mail_id = Some(mail_id);
                    false
                }
                _ => {
                    debug!("No new mail received");
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
                            error!("Could not fetch mail: {:?}", e);
                            continue 'idle_loop;
                        }
                    };
                    MailHandler::parse_forward_mail(send_mails.clone(), messages);
                }
                Err(e) => error!("IDLE finished with error {:?}", e),
            }
        }
    }

    fn polling_loop(imap: &mut Session<Connection>, send_mails: Sender<MailData>, interval: Duration) {
        let last_seen_message_id = 0;
        loop {
            debug!("Periodically checking for new mail...");

            // Search for unseen messages
            let message_ids = match imap.search("UNSEEN") {
                Ok(ids) => ids,
                Err(e) => {
                    error!("Could not search for new mail: {:?}", e);
                    continue;
                }
            };

            // Proceed only if there are any unseen messages
            if let Some(&last_message_id) = message_ids.iter().max() {
                if last_message_id == last_seen_message_id {
                    warn!("No new mail found");
                    continue;
                }
                let messages: Fetches = match imap.fetch(last_message_id.to_string(), "RFC822") {
                    Ok(messages) => messages,
                    Err(e) => {
                        error!("Could not fetch mail: {:?}", e);
                        continue;
                    }
                };

                info!("New mail found...forwarding");
                MailHandler::parse_forward_mail(send_mails.clone(), messages);
            }

            // Sleep for selected interval
            thread::sleep(interval);
        }
    }


    fn parse_forward_mail(send_mails: Sender<MailData>, messages: Fetches) {
        let message = match messages.iter().next() {
            Some(message) => message,
            None => {
                error!("Could not get mail");
                return;
            }
        };

        let body = match message.body() {
            Some(body) => body,
            None => {
                error!("Could not get mail body");
                return;
            }
        };

        let parsed_mail = match parse_mail(body) {
            Ok(mail) => mail,
            Err(e) => {
                error!("Could not parse mail: {:?}", e);
                return;
            }
        };

        // Get the subject and sender
        let subject = match parsed_mail.headers.get_first_value("Subject") {
            Some(subject) => subject,
            _ => {
                "No subject".to_string()
            }
        };

        let from = match parsed_mail.headers.get_first_value("From") {
            Some(from) => from,
            _ => {
                "No sender".to_string()
            }
        };

        // from = "name <mail>"
        let mail_regex = Regex::new(r"<(.*?)>").unwrap();
        let sender = if let Some(captures) = mail_regex.captures(&*from) {
            // Get the first captured group
            captures.get(1).unwrap().as_str().to_string()
        } else {
            from.as_str().to_string()
        };

        let (text_body, html_body) = MailHandler::extract_bodies(&parsed_mail);

        let mail_data = MailData {
            subject,
            sender,
            text_body,
            html_body,
        };

        let _ = send_mails.send(mail_data);
    }

    fn handle_mail(&self, mail_data: MailData) -> bool {
        info!("Handling mail: {}: <{}>", mail_data.subject, mail_data.sender);

        // Validate mail
        if (self.config.alarm_subject != "*") && (mail_data.subject != self.config.alarm_subject) {
            warn!(
                "Subject mismatch: '{}' != '{}'",
                mail_data.subject, self.config.alarm_subject
            );
            return false;
        }
        if (self.config.alarm_sender != "*") && (mail_data.sender != self.config.alarm_sender) {
            warn!(
                "Sender mismatch: '{}' != '{}'",
                mail_data.sender, self.config.alarm_sender
            );
            return false;
        }

        let mut alarm = Alarm::new();
        alarm.alarm_source(self.config.name.clone());
        alarm.origin = self.config.name.clone();

        match self.mailparser.parse(&mail_data.text_body, &mail_data.html_body, &mut alarm, self.config.clone()) {
            Ok(_) => {
                self.send_alarms.send(alarm).unwrap();
                true
            }
            Err(e) => {
                error!("Could not parse mail: {}", e);
                false
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
