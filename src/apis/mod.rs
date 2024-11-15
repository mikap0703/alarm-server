pub mod divera_v2;
pub mod mock_api;
pub mod telegram;
use crate::alarm::Alarm;
use std::future::Future;
use async_trait::async_trait;

#[async_trait]
pub trait Api: Send {
    async fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Result<(), String>;
    async fn update_alarm<'a>(&'a self, _alarm: &'a Alarm) -> Result<(), String>;
}
