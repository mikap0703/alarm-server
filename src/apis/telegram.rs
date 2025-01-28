use async_trait::async_trait;
use crate::alarm::Alarm;
use crate::apis::Api;
use reqwest::Client;
use urlencoding::encode;
use log::{debug, error, info, warn};

pub struct Telegram {
    pub name: String,
    pub bot_token: String,
}

#[async_trait]
impl Api for Telegram {
    async fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Result<(), String> {
        info!("Telegram API: Trigger");
        let receivers = alarm.receiver.get(self.name.as_str());

        if receivers.is_none() {
            return Err(format!("No receivers found for Telegram API: {}", self.name));
        }
        let receivers = receivers.unwrap();

        let client = Client::new();

        for receiver in receivers.members.clone() {
            info!("Sending message to: {}", receiver);

            let url = format!(
                "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
                self.bot_token, receiver, encode(&*alarm.text)
            );

            match client.get(&url).send().await {
                Ok(_) => (),
                Err(err) => eprintln!("Error sending message: {}", err),
            }
        }

        Ok(())
    }

    async fn update_alarm<'a>(&'a self, _alarm: &'a Alarm) -> Result<(), String> {
        info!("Telegram API: Updating alarm with key: {}", self.bot_token);
        Ok(())
    }
}
