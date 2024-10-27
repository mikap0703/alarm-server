use crate::alarm::Alarm;
use crate::mail_parser::MailParser;

pub struct SecurCadParser;

impl MailParser for SecurCadParser {
    fn parse(&self, text_body: &str, html_body: &str, alarm: &mut Alarm) -> Result<String, String> {
        Ok(format!("Parsed SecurCad: {}", text_body))
    }
}
