use crate::alarm::Alarm;
use crate::config::alarm_sources::MailConfig;
use crate::mail_parser::MailParser;

pub struct MockParser;

impl MailParser for MockParser {
    fn parse(&self, text_body: &str, html_body: &str, alarm: &mut Alarm, config: MailConfig) -> Result<String, String> {
        Ok(format!("Parsed SecurCad: {}", text_body))
    }
}
