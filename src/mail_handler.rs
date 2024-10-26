use imap::types::UnsolicitedResponse;
use mailparse::{parse_mail, MailHeaderMap};
use crate::config::alarm_sources::MailConfig;

pub(crate) struct MailHandler {
    config: MailConfig,
    debug: bool,
}

impl MailHandler {
    pub fn new(config: MailConfig, debug: bool) -> Self {
        Self { config, debug }
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
            println!("Starting IDLE");
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
                        Ok(Some(subject)) => subject,
                        _ => {
                            println!("Mail did not have a subject");
                            continue 'idle_loop;
                        }
                    };
                    let from = match parsed_mail.headers.get_first_value("From") {
                        Ok(Some(from)) => from,
                        _ => {
                            println!("Mail did not have a sender");
                            continue 'idle_loop;
                        }
                    };

                    let (text_body, html_body) = Self::extract_bodies(&parsed_mail);

                    println!("Subject: {}", subject);
                    println!("From: {}", from);
                    println!("Text body: {}", text_body);
                    println!("HTML body: {}", html_body);
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
