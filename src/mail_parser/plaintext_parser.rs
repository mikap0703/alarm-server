use crate::alarm::Alarm;
use crate::config::alarm_sources::MailConfig;
use crate::mail_parser::MailParser;
use log::debug;

pub struct PlaintextParser;

impl MailParser for PlaintextParser {
    fn parse(&self, text_body: &str, html_body: &str, alarm: &mut Alarm, config: MailConfig) -> Result<String, String> {
        debug!("text: {}", text_body);
        debug!("html: {}", html_body);

        alarm.add_to_text(text_body.parse().unwrap());

        Ok(format!("Plaintext Parser done: {}", text_body))
    }
}
