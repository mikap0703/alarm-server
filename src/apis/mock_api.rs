use async_trait::async_trait;
use crate::alarm::Alarm;
use crate::apis::Api;
use log::debug;

pub struct MockApi {
    pub name: String,
    pub api_key: String,
}

#[async_trait]
impl Api for MockApi {
    async fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Result<(), String> {
        debug!("Mock API: trigger alarm");
        debug!("{:?}", alarm);
        Ok(())
    }

    async fn update_alarm<'a>(&'a self, alarm: &'a Alarm) -> Result<(), String> {
        debug!("Mock API: Updating alarm");
        debug!("{:?}", alarm);
        Ok(())
    }
}