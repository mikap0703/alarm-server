pub mod divera_v2;
pub mod mock_api;
pub mod telegram;

use crate::alarm::Alarm;

pub trait Api: Send {
    fn trigger_alarm(&self, alarm: &Alarm) -> Result<(), String>;
    fn update_alarm(&self, alarm: &Alarm) -> Result<(), String>;
}