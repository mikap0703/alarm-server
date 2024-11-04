use crate::alarm::Alarm;
use crate::apis::Api;

pub struct MockApi {
    pub name: String,
    pub api_key: String,
}

impl Api for MockApi {
    fn trigger_alarm(&self, _alarm: &Alarm) -> Result<(), String> {
        println!("Divera API: Triggering alarm with key: {}", self.api_key);
        Ok(())
    }

    fn update_alarm(&self, _alarm: &Alarm) -> Result<(), String> {
        println!("Divera API: Updating alarm with key: {}", self.api_key);
        Ok(())
    }
}