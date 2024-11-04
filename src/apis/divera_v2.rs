use crate::alarm::Alarm;
use crate::apis::Api;

pub struct DiveraV2 {
    pub name: String,
    pub api_key: String,
}

impl Api for DiveraV2 {
    fn trigger_alarm(&self, _alarm: &Alarm) -> Result<(), String> {
        println!("Divera API: Triggering alarm with key: {}", self.api_key);
        Ok(())
    }

    fn update_alarm(&self, _alarm: &Alarm) -> Result<(), String> {
        println!("Divera API: Updating alarm with key: {}", self.api_key);
        Ok(())
    }
}