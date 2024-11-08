pub mod divera_v2;
pub mod mock_api;
pub mod telegram;
use crate::alarm::Alarm;
use std::future::Future;

pub trait Api: Send {
    fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Box<dyn Future<Output = Result<(), String>> + Send + 'a>;
    fn update_alarm<'a>(&'a self, alarm: &'a Alarm) -> Box<dyn Future<Output = Result<(), String>> + Send + 'a>;
}
