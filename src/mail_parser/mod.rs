pub mod sl_secur_cad;
pub mod mock_parser;
mod helpers;

use crate::alarm::Alarm;
use crate::config::alarm_sources::MailConfig;

pub trait MailParser {
    fn parse(&self, text_body: &str, html_body: &str, alarm: &mut Alarm, config: MailConfig) -> Result<String, String>;
}