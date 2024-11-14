use std::time::Duration;
use flume::Sender;
use imap::types::{Fetch, Fetches, UnsolicitedResponse};
use mailparse::{parse_mail, MailHeaderMap, ParsedMail};
use regex::Regex;
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
}

struct MailData {
    subject: String,
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
        let client = imap::ClientBuilder::new(self.config.host.as_str(), self.config.port)
            .connect()
            .expect("Could not connect to imap server");

        let mut imap = client
            .login(self.config.user.as_str(), self.config.password.as_str())
            .expect("Could not login to imap server");

        imap.select("INBOX").expect("Could not select mailbox");

        // imap.debug = self.debug;

        'debug: {
            if self.debug {
                let messages: Fetches = match imap.fetch("*", "RFC822") {
                    Ok(messages) => messages,
                    Err(e) => {
                        println!("Could not fetch mail: {:?}", e);
                        break 'debug;
                    }
                };
                let message: &Fetch = if let Some(m) = messages.iter().next() {
                    m
                } else {
                    break 'debug;
                };

                self.handle_mail(message);

                return;
            }
        }

        'idle_loop: loop {
            // todo: maybe mails are ignored (multiple mails, same time)
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
                Some(id) => {
                    id
                },
                None => {
                    continue 'idle_loop;
                }
            };

            match idle_result {
                Ok(_) => {
                    // Get mail
                    let messages: Fetches = match imap.fetch(new_mail_id.to_string(), "RFC822") {
                        Ok(messages) => messages,
                        Err(e) => {
                            println!("Could not fetch mail: {:?}", e);
                            continue 'idle_loop;
                        }
                    };
                    let message: &Fetch = if let Some(m) = messages.iter().next() {
                        m
                    } else {
                        continue 'idle_loop;
                    };


                    if self.handle_mail(message) { continue 'idle_loop; }
                }
                Err(e) => println!("IDLE finished with error {:?}", e),
            }
        }
    }

    fn handle_mail(&self, message: &Fetch) -> bool {
        let body = match message.body() {
            Some(body) => body,
            None => {
                println!("Could not get mail body");
                return true;
            }
        };

        let parsed_mail = match parse_mail(body) {
            Ok(mail) => mail,
            Err(e) => {
                println!("Could not parse mail: {:?}", e);
                return true;
            }
        };

        // Get the subject and sender
        let subject = match parsed_mail.headers.get_first_value("Subject") {
            Some(subject) => subject,
            _ => {
                println!("Mail did not have a subject");
                return true;
            }
        };

        // let subject = subject.trim();

        let from = match parsed_mail.headers.get_first_value("From") {
            Some(from) => from,
            _ => {
                println!("Mail did not have a sender");
                return true;
            }
        };

        // from = "name <mail>"

        let mail_regex = Regex::new(r"<(.*?)>").unwrap();
        let sender = if let Some(captures) = mail_regex.captures(&*from) {
            // Get the first captured group
            captures.get(1).unwrap().as_str()
        } else {
            println!("No email address found.");
            "*"
        };

        let (text_body, html_body) = Self::extract_bodies(&parsed_mail);
        println!("{}", text_body);
        println!("{}", html_body);
        // validate the mail
        // check the mails subject
        let config_subject = self.config.alarm_subject.to_string();

        println!("{}", config_subject == subject);

        if (self.config.alarm_subject != "*") && (subject != config_subject) {
            println!("'{}' - '{}'", subject, self.config.alarm_subject);
            println!("Der Betreff ({}) stimmt nicht mit dem Betreff der Config überein...Mail wird ignoriert", subject);
            return true;
        }


        // check the mails sender
        if (self.config.alarm_sender != "*") && (sender != self.config.alarm_sender) {
            println!("Der Absender ({}) stimmt nicht mit dem Absender der Config überein...Mail wird ignoriert", from);
            return true;
        }

        // Parse the mail
        println!("{}: <{}> - {}", self.config.name, sender, subject);

        let mut alarm = Alarm::new();

        alarm.alarm_source(self.config.name.clone());

        match self.mailparser.parse(&text_body, &html_body, &mut alarm, self.config.clone()) {
            Ok(parsed) => println!("Parsed: {}", parsed),
            Err(e) => println!("Could not parse mail: {}", e),
        };

        self.send_alarms.send(alarm).unwrap();
        false
    }

    fn extract_bodies(parsed_mail: &ParsedMail) -> (String, String) {
        let mut text_body = String::new();
        let mut html_body = String::new();

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
