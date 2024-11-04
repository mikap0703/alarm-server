use reqwest::Client;
use crate::alarm::Alarm;
use crate::apis::Api;

pub struct Telegram {
    pub name: String,
    pub bot_token: String,
}

impl Api for Telegram {
    fn trigger_alarm(&self, alarm: &Alarm) -> Result<(), String> {
        println!("Telegram API: Trigger");
        let receivers = alarm.receiver.get(self.name.as_str());

        if receivers.is_none() {
            return Err(format!("No receivers found for Telegram API: {}", self.name));
        }
        let receivers = receivers.unwrap();

        // Create a new HTTP client        let client = Client::new();

        for receiver in receivers.members.clone() {
            println!("Sending message to: {}", receiver);

            let _res = reqwest::blocking::get(&format!("https://api.telegram.org/bot{}/sendmessage?chat_id={}&text=Hello%20World", self.bot_token, receiver));
        }

        Ok(())
    }

    fn update_alarm(&self, _alarm: &Alarm) -> Result<(), String> {
        println!("Divera API: Updating alarm with key: {}", self.bot_token);
        Ok(())
    }
}