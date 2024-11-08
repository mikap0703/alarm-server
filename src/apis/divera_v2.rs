use std::future::Future;
use crate::alarm::Alarm;
use crate::apis::Api;

pub struct DiveraV2 {
    pub name: String,
    pub api_key: String,
}

impl Api for DiveraV2 {
    fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Box<dyn Future<Output = Result<(), String>> + Send + 'a> {
        Box::new(async move {
            println!("Divera API: Trigger");
            Ok(())
        })
    }

    fn update_alarm<'a>(&'a self, _alarm: &'a Alarm) -> Box<dyn Future<Output = Result<(), String>> + Send + 'a> {
        Box::new(async move {
            println!("Divera API: Updating alarm");
            Ok(())
        })
    }
}