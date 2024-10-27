use imap::types::UnsolicitedResponse;
use mailparse::{parse_mail, MailHeaderMap};
use regex::Regex;
use crate::alarm::Alarm;
use crate::config::alarm_sources::MailConfig;
use crate::mail_parser::{MailParser};
use crate::mail_parser::sl_secur_cad::SecurCadParser;
use crate::mail_parser::mock_parser::MockParser;

pub struct MailHandler {
    config: MailConfig,
    debug: bool,
    mailparser: Box<dyn MailParser>,
}

impl MailHandler {
    pub fn new(config: MailConfig, debug: bool) -> MailHandler {
        let mailparser: Box<dyn MailParser> = match config.mail_schema.as_str() {
            "SL-securCAD" => Box::new(SecurCadParser),
            _ => Box::new(MockParser),
        };
        Self { config, debug, mailparser }
    }

    pub fn start(&self) {
        let client = imap::ClientBuilder::new(self.config.host.as_str(), self.config.port)
            .connect()
            .expect("Could not connect to imap server");

        let mut imap = client
            .login(self.config.user.as_str(), self.config.password.as_str())
            .expect("Could not login to imap server");

        imap.select("INBOX").expect("Could not select mailbox");

        'idle_loop: loop {
            // todo: maybe mails are ignored (multiple mails, same time)
            println!("Warten auf neue Mails...");
            let idle_result = imap.idle().wait_while(|response| match response {
                UnsolicitedResponse::Recent(_) => {
                    println!("New mail received");
                    true
                }
                _ => false,
            });

            match idle_result {
                Ok(_) => {
                    // Get mail
                    let messages = imap.fetch("*", "BODY[]").unwrap();
                    let message = if let Some(m) = messages.iter().next() {
                        m
                    } else {
                        continue 'idle_loop;
                    };

                    let body = message.body().expect("Message did not have a body!");

                    let parsed_mail = match parse_mail(body) {
                        Ok(mail) => mail,
                        Err(e) => {
                            println!("Could not parse mail: {:?}", e);
                            continue 'idle_loop;
                        }
                    };

                    // Get the subject and sender
                    let subject = match parsed_mail.headers.get_first_value("Subject") {
                        Ok(Some(s)) => s.trim().to_string(),
                        _ => {
                            println!("Mail did not have a subject");
                            continue 'idle_loop;
                        }
                    };

                    // let subject = subject.trim();

                    let from = match parsed_mail.headers.get_first_value("From") {
                        Ok(Some(from)) => from,
                        _ => {
                            println!("Mail did not have a sender");
                            continue 'idle_loop;
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

                    println!("Subject: {}", subject);
                    println!("From: {}", sender);
                    println!("Text body: {}", text_body);
                    println!("HTML body: {}", html_body);


                    println!("{}", subject);
                    println!("that was the subject");
                    println!("{}", subject.trim()=="Test");
                    // validate the mail
                    // check the mails subject
                    println!("Config alarm_subject: {}", self.config.alarm_subject);

                    let config_subject = self.config.alarm_subject.to_string();

                    println!("{}", config_subject == subject);

                    if (self.config.alarm_subject != "*") && (subject != config_subject) {
                        println!("'{}' - '{}'", subject, self.config.alarm_subject);
                        println!("Der Betreff ({}) stimmt nicht mit dem Betreff der Config überein...Mail wird ignoriert", subject);
                        continue 'idle_loop;
                    }


                    // check the mails sender
                    if (self.config.alarm_sender != "*") && (sender != self.config.alarm_sender) {
                        println!("Der Absender ({}) stimmt nicht mit dem Absender der Config überein...Mail wird ignoriert", from);
                        continue 'idle_loop;
                    }

                    // Parse the mail
                    println!("{}: <{}> - {}", self.config.name, sender, subject);

                    let mut alarm = Alarm::new();

                    match self.mailparser.parse(&text_body, &html_body, &mut alarm) {
                        Ok(parsed) => println!("Parsed: {}", parsed),
                        Err(e) => println!("Could not parse mail: {}", e),
                    };
                }
                Err(e) => println!("IDLE finished with error {:?}", e),
            }
        }
    }

    fn extract_bodies(parsed_mail: &mailparse::ParsedMail) -> (String, String) {
        let mut text_body = String::new();
        let mut html_body = String::new();

        fn traverse_parts(mail: &mailparse::ParsedMail, text_body: &mut String, html_body: &mut String) {
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
