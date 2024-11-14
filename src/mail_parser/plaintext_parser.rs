use crate::alarm::Alarm;
use crate::config::alarm_sources::MailConfig;
use crate::mail_parser::MailParser;

pub struct PlaintextParser;

impl MailParser for PlaintextParser {
    fn parse(&self, text_body: &str, html_body: &str, alarm: &mut Alarm, config: MailConfig) -> Result<String, String> {
        alarm.add_to_text(text_body.parse().unwrap());

        Ok(format!("Plaintext Parser done: {}", text_body))
    }
}
