pub mod sl_secur_cad;
pub mod mock_parser;

use crate::alarm::Alarm;


pub trait MailParser {
    fn parse(&self, text_body: &str, html_body: &str, alarm: &mut Alarm) -> Result<String, String>;
}