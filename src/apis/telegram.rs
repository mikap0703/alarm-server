use async_trait::async_trait;
use crate::alarm::Alarm;
use crate::apis::Api;
use reqwest::Client;
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
        info!("Telegram API: Triggering alarm via POST");

        let receivers = alarm.get_receivers(self.name.as_str());
        let client = Client::new();

        for receiver in receivers.members.clone() {
            // 1. Prepare and escape individual data fields
            let title = escape_markdown_v2(&alarm.title);
            let body_text = escape_markdown_v2(&alarm.text);

            // 2. Build the message
            let mut text = format!("*{}*\n{}", title, body_text);

            if !alarm.address.object.is_empty() {
                text.push_str(&format!("\n{}", escape_markdown_v2(&alarm.address.object)));
            }

            if !alarm.address.info.is_empty() {
                // FIX: Escape the literal parentheses used for formatting
                text.push_str(&format!(" \\({}\\)", escape_markdown_v2(&alarm.address.info)));
            }

            if let (Some(lat), Some(lng)) = (alarm.address.coords.lat, alarm.address.coords.lon) {
                let lat_s = escape_markdown_v2(&lat.to_string());
                let lng_s = escape_markdown_v2(&lng.to_string());
                text.push_str(&format!("\n\n*Koordinaten:* {}, {}", lat_s, lng_s));

                // FIX: Only escape ')' and '\' inside the URL part of a Markdown link
                let raw_url = format!("https://maps.apple.com/?q={},{}", lat, lng);
                let link_url = raw_url.replace('\\', "\\\\").replace(')', "\\)");

                text.push_str(&format!("\n\n[Apple Maps]({})\n", link_url));
            }

            let alarm_einheiten = alarm.units.iter().map(|unit| escape_markdown_v2(unit)).collect::<Vec<String>>().join("\n");

            text.push_str(&alarm_einheiten);

            // 3. Construct the JSON payload
            let endpoint = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
            let payload = serde_json::json!({
                "chat_id": receiver,
                "text": text,
                "parse_mode": "MarkdownV2"
            });

            // 4. Send the POST request
            match client.post(&endpoint).json(&payload).send().await {
                Ok(res) => {
                    let status = res.status();
                    let response_body = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());

                    if status.is_success() {
                        info!("Message successfully sent to: {}", receiver);
                    } else {
                        error!("Telegram API Error ({}): {}", status, response_body);
                        // If it still fails, the response_body will tell us EXACTLY which character failed.
                    }
                }
                Err(err) => error!("Network error while contacting Telegram: {}", err),
            }
        }

        Ok(())
    }

    async fn update_alarm<'a>(&'a self, _alarm: &'a Alarm) -> Result<(), String> {
        info!("Telegram API: Updating alarm with key: {}", self.bot_token);
        Ok(())
    }
}
