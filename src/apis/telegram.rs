use crate::alarm::Alarm;
use crate::apis::Api;
use reqwest::Client;
use std::future::Future;
use urlencoding::encode;

pub struct Telegram {
    pub name: String,
    pub bot_token: String,
}

impl Api for Telegram {
    fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Box<dyn Future<Output = Result<(), String>> + Send + 'a> {
        Box::new(async move {
            println!("Telegram API: Trigger");
            let receivers = alarm.receiver.get(self.name.as_str());

            if receivers.is_none() {
                return Err(format!("No receivers found for Telegram API: {}", self.name));
            }
            let receivers = receivers.unwrap();

            let client = Client::new();

            for receiver in receivers.members.clone() {
                println!("Sending message to: {}", receiver);

                let url = format!(
                    "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
                    self.bot_token, receiver, encode(&*alarm.text)
                );

                let res = client.get(&url).send().await;

                if let Err(err) = res {
                    eprintln!("Error sending message: {}", err);
                }
            }

            Ok(())
        })
    }

    fn update_alarm<'a>(&'a self, _alarm: &'a Alarm) -> Box<dyn Future<Output = Result<(), String>> + Send + 'a> {
        Box::new(async move {
            println!("Telegram API: Updating alarm with key: {}", self.bot_token);
            Ok(())
        })
    }
}
