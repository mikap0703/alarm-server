use async_trait::async_trait;
use crate::alarm::Alarm;
use crate::apis::Api;
use log::{debug, error, info, warn};

pub struct MockApi {
    pub name: String,
    pub api_key: String,
}

#[async_trait]
impl Api for MockApi {
    async fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Result<(), String> {
        info!("Mock API: trigger alarm");
        Ok(())
    }

    async fn update_alarm<'a>(&'a self, _alarm: &'a Alarm) -> Result<(), String> {
        info!("Mock API: Updating alarm");
        Ok(())
    }
}