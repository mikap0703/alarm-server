use async_trait::async_trait;
use crate::alarm::Alarm;
use crate::apis::Api;
use log::{debug, info};
use reqwest::Client;
use serde_json::json;
use serde_json::Value;

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
            text.push_str(&format!("\n{}", alarm.address.object));
        }

        if alarm.address.info != "" {
            text.push_str(&format!(" ({})", alarm.address.info));
        }

        if alarm.address.object_id != "" {
            text.push_str(&format!("Objekt-ID: {}", alarm.address.object_id));
        }

        // Add UTM if available
        if alarm.address.utm != "" {
            text.push_str(&format!("\n\nUTM: {}", alarm.address.utm));
        }

        if let (Some(lat), Some(lng)) = (alarm.address.coords.lat, alarm.address.coords.lon) {
            // Always start with coordinates-related text
            text.push_str(&format!("\n\nKoordinaten: {}, {}", lat, lng));

            // Add Apple Maps link
            text.push_str(&format!("\n\nhttps://maps.apple.com/?q={},{}", lat, lng));
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
                debug!("{:?}", response);
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

    async fn check_connection(&self) -> Result<String, String> {
        let client = Client::new();
        let url = format!(
            "https://app.divera247.com/api/v2/pull/all?accesskey={}",
            self.api_key
        );
        let res = client
            .get(url)
            .send()
            .await
            .map_err(|err| format!("Request error: {}", err))?;

        let status = res.status();
        let body = res.text().await.map_err(|err| format!("Failed to read response: {}", err))?;

        if !status.is_success() {
            return Err(format!("HTTP {} - {}", status, body));
        }

        let value: Value = serde_json::from_str(&body).map_err(|err| format!("Invalid JSON: {}", err))?;
        let divera_name = value
            .get("data")
            .and_then(|v| v.get("cluster"))
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str());

        match divera_name {
            Some(name) => Ok(format!("{}", name)),
            None => Ok("Verbunden, jedoch Name unbekannt".to_string()),
        }
    }
}
