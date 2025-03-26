use async_trait::async_trait;
use flume::Receiver;
use crate::alarm::{Alarm, AlarmReceiver};
use crate::apis::Api;
use log::info;
use reqwest::Client;
use serde_json::json;

pub struct DiveraV2 {
    pub name: String,
    pub api_key: String,
}

#[async_trait]
impl Api for DiveraV2 {
    async fn trigger_alarm<'a>(&'a self, alarm: &'a Alarm) -> Result<(), String> {
        info!("Divera API: trigger alarm");
        info!("{:?}", alarm);

        let receivers = alarm.get_receivers(self.name.as_str());

        let mut text = alarm.text.clone();

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

        let client = Client::new();
        let req_body = json!({
            "accesskey": self.api_key,
            "Alarm": {
                "foreign_id": alarm.id,
                "priority": true,
                "title": alarm.title,
                "text": text,
                "address": alarm.address.street,
                "lat": 0,
                "lng": 0,
                "private_mode": true,
                "notification_type": 3,
                "notification_filter_access": true,
                "send_push": true,
                "group": receivers.groups,
                "vehicle": receivers.vehicles,
            },
            "instructions": {
                "group": {
                    "mapping": "name"
                },
                "vehicle": {
                    "mapping": "name"
                },
            }
        });

        let res = client.post("https://app.divera247.com/api/v2/alarms")
            .json(&req_body)
            .send()
            .await;

        match res {
            Ok(response) => {
                println!("{:?}", response);
                if response.status().is_success() {
                    info!("Alarm triggered successfully");
                    Ok(())
                } else {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    Err(format!("Failed to trigger alarm: {} - {}", status, text))
                }
            }
            Err(err) => Err(format!("Request error: {}", err)),
        }
    }

    async fn update_alarm<'a>(&'a self, alarm: &'a Alarm) -> Result<(), String> {
        info!("Divera API: Updating alarm");
        info!("{:?}", alarm);
        Ok(())
    }
}