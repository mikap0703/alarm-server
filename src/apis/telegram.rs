use async_trait::async_trait;
use crate::alarm::{Alarm, AlarmReceiver};
use crate::apis::Api;
use reqwest::Client;
use urlencoding::encode;
use log::{error, info};

pub struct Telegram {
    pub name: String,
    pub bot_token: String,
}

fn escape_markdown_v2(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('~', "\\~")
        .replace('`', "\\`")
        .replace('>', "\\>")
        .replace('#', "\\#")
        .replace('+', "\\+")
        .replace('-', "\\-")
        .replace('=', "\\=")
        .replace('|', "\\|")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('.', "\\.")
        .replace('!', "\\!")
}


#[async_trait]
impl Api for Telegram {
    async fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Result<(), String> {
        info!("Telegram API: Trigger");

        let receivers = alarm.get_receivers(self.name.as_str());

        let client = Client::new();

        for receiver in receivers.members.clone() {
            info!("Sending message to: {}", receiver);

            let mut text = format!("*{}*\n{}", escape_markdown_v2(&alarm.title), &alarm.text);

            if alarm.address.object != "" {
                text.push_str(format!("\n{}", alarm.address.object).as_str());
            }

            if alarm.address.info != "" {
                text.push_str(format!(" ({})", alarm.address.info).as_str());
            }

            if alarm.address.object_id != "" {
                text.push_str(format!("Objekt-ID: {}", alarm.address.object_id).as_str());
            }

            // Add UTM if available
            if alarm.address.utm != "" {
                text.push_str(format!("\n\nUTM: {}", alarm.address.utm).as_str());
            }

            if let (Some(lat), Some(lng)) = (alarm.address.coords.lat, alarm.address.coords.lon) {
                // Always start with coordinates-related text
                text.push_str(format!("\n\nKoordinaten: {}, {}", lat, lng).as_str());

                // Add Apple Maps link
                text.push_str(format!("\n\nhttps://maps.apple.com/?q={},{}", lat, lng).as_str());
            }

            text = escape_markdown_v2(&text);

            let url = format!(
                "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}&parse_mode=MarkdownV2",
                self.bot_token, receiver, encode(&*text),
            );

            println!("{}", url);

            match client.get(&url).send().await {
                Ok(res) => if res.status().is_success() {
                        info!("Message sent to: {}", receiver);
                    } else {
                        error!("Error sending message to {}: {}", receiver, res.status());
                        println!("{:?}", res.text().await);
                    },
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
