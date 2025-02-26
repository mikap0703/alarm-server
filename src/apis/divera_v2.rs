use async_trait::async_trait;
use crate::alarm::Alarm;
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

        let client = Client::new();
        let req_body = json!({
            "accesskey": self.api_key,
            "Alarm": {
                "foreign_id": alarm.id,
                "priority": true,
                "title": alarm.title,
                "text": alarm.text,
                "address": alarm.address.street,
                "lat": 0,
                "lng": 0,
                "private_mode": true,
                "notification_type": 3,
                "notification_filter_access": true,
                "send_push": true,
                "group": alarm.groups,
                "vehicle": alarm.vehicles,
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