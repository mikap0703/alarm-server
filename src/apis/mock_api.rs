use std::future::Future;
use crate::alarm::Alarm;
use crate::apis::Api;

pub struct MockApi {
    pub name: String,
    pub api_key: String,
}

impl Api for MockApi {
    fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Box<dyn Future<Output = Result<(), String>> + Send + 'a> {
        Box::new(async move {
            println!("Divera API: Triggering alarm with key: {}", self.api_key);
            Ok(())
        })
    }

    fn update_alarm<'a>(&'a self, _alarm: &'a Alarm) -> Box<dyn Future<Output = Result<(), String>> + Send + 'a> {
        Box::new(async move {
            println!("Divera API: Updating alarm with key: {}", self.api_key);
            Ok(())
        })
    }
}