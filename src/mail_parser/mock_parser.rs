use crate::alarm::Alarm;
use crate::config::alarm_sources::MailConfig;
use crate::mail_parser::MailParser;

pub struct MockParser;

impl MailParser for MockParser {
    fn parse(&self, text_body: &str, _html_body: &str, _alarm: &mut Alarm, _config: MailConfig) -> Result<String, String> {
        Ok(format!("Parsed SecurCad: {}", text_body))
    }
}
