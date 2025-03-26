use std::cmp::PartialEq;
use std::collections::HashMap;
use std::sync::{Arc};
use std::thread;
use std::time::Duration;
use tokio::sync::Mutex;
use crate::alarm::{Alarm};
use crate::apis::Api;
use crate::apis::divera_v2::DiveraV2;
use crate::apis::mock_api::MockApi;
use crate::apis::telegram::Telegram;
use crate::config::alarm_templates::AlarmTemplates;
use crate::config::general::{ApiConfig, ApiType, GeneralConfig};
use log::{debug, error, info, warn};
use crate::config::Configs;

pub struct AlarmHandler {
    // channel to send and receive alarms
    recv_alarms: flume::Receiver<Alarm>,
    apis: Arc<Mutex<HashMap<String, Box<dyn Api>>>>,
    alarm_templates: AlarmTemplates,
    last_alarms: Arc<Mutex<Vec<Alarm>>>, // Change to Arc<Mutex<>> for shared mutable access
    config: GeneralConfig
}

#[derive(PartialEq, Debug)]
enum AlarmType {
    FirstAlarm,
    UpdateAlarm,
    DropAlarm
}

impl AlarmHandler {
    pub fn new(recv_alarms: flume::Receiver<Alarm>, config: GeneralConfig, alarm_templates: AlarmTemplates) -> Self {
        let mut apis_map = HashMap::new();
        for api_config in config.clone().apis {
            let name = api_config.name.clone();
            let api_key = api_config.api_key.clone();
            let api: Box<dyn Api> = match api_config.api {
                ApiType::Divera => Box::new(DiveraV2 { name, api_key }),
                ApiType::Alamos => Box::new(MockApi { name, api_key }),
                ApiType::Telegram => Box::new(Telegram { name, bot_token: api_key }),
                // _ => Box::new(MockApi { api_key: api_config.api_key }),
            };
            apis_map.insert(api_config.name.clone(), api);
        }

        let apis = Arc::new(Mutex::new(apis_map));

        Self {
            recv_alarms,
            apis,
            alarm_templates,
            last_alarms: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    pub fn start(&self) {
        let recv_alarms = self.recv_alarms.clone();
        let apis = self.apis.clone();
        let alarm_templates = self.alarm_templates.clone();
        let last_alarms = self.last_alarms.clone();
        let config = self.config.clone();

        // Use tokio::spawn to create an async task
        tokio::spawn(async move {
            loop {
                match recv_alarms.recv() {
                    Ok(mut alarm) => {
                        println!("{:?}", alarm);
                        info!("AlarmHandler received alarm: {}", alarm.title);

                        // apply default template
                        match alarm_templates.templates.get("default") {
                            Some(template) => {
                                for (api_name, receiver) in template.apis.clone() {
                                    debug!("Applying default template for {}", api_name);
                                    alarm.apply_template(api_name.clone(), receiver);
                                }
                            },
                            None => {
                                error!("Default template not found");
                                break;
                            }
                        };

                        // apply remaining templates from alarm
                        for template_name in alarm.template_names.clone() {
                            match alarm_templates.templates.get(&template_name) {
                                Some(template) => {
                                    for (api_name, receiver) in template.apis.clone() {
                                        debug!("Applying template {} for {}", template_name, api_name);
                                        alarm.apply_template(api_name.clone(), receiver);
                                    }
                                },
                                None => {
                                    warn!("Template {} not found", template_name);
                                    continue; // Changed break to continue to prevent exiting the loop
                                }
                            };
                        }

                        let alarm_type = {
                            let last_alarms_lock = last_alarms.lock().await;
                            if let Some(last_alarm) = last_alarms_lock.last() {
                                compare_alarms(&alarm, last_alarm, &config)
                            } else {
                                AlarmType::FirstAlarm
                            }
                        };

                        match alarm_type {
                            AlarmType::FirstAlarm => {
                                info!("Alarmierung ist ein Erstalarm");
                            },
                            AlarmType::UpdateAlarm => {
                                info!("Alarmierung ist ein Update");
                            },
                            AlarmType::DropAlarm => {
                                info!("Alarmierung ist irrelevant");
                                continue;
                            }
                        }

                        // Trigger alarm for each API
                        {
                            let apis_lock = apis.lock().await;

                            for (api_name, _) in alarm.receiver.clone() {
                                match apis_lock.get(&api_name) {
                                    Some(api) => {
                                        let result = match alarm_type {
                                            AlarmType::FirstAlarm => api.trigger_alarm(&alarm).await,
                                            AlarmType::UpdateAlarm => api.update_alarm(&alarm).await,
                                            AlarmType::DropAlarm => continue, // Skip this API
                                        };

                                        if let Err(e) = result {
                                            error!("Error triggering/updating alarm for API {}: {:?}", api_name, e);
                                        }
                                    }
                                    None => {
                                        error!("API {} not found", api_name);
                                        continue;
                                    }
                                };
                            }
                        }

                        // Call webhooks
                        for webhook in alarm.webhooks.clone() {
                            info!("Calling webhook: {}", webhook);
                            thread::spawn(move || {
                                let client = reqwest::Client::new();
                                let _ = client.get(webhook.as_str())
                                    .send();
                            });
                        }

                        // Update last_alarms after processing
                        let mut last_alarms_lock = last_alarms.lock().await;
                        last_alarms_lock.push(alarm);
                    },
                    Err(e) => {
                        error!("Error receiving alarm: {}", e);
                        break;
                    }
                }
            }
        });
    }
}

// Move compare_alarms to a standalone function
fn compare_alarms(new_alarm: &Alarm, old_alarm: &Alarm, config: &GeneralConfig) -> AlarmType {
    let time_diff = new_alarm.time - old_alarm.time;
    if time_diff < Duration::from_secs(config.timeout) {
        // update but maybe irrelevant
        let new_source = &new_alarm.origin;
        let new_source_key = config.source_priority.iter().position(|n| n == new_source);

        let old_source = &old_alarm.origin;
        let old_source_key = config.source_priority.iter().position(|n| n == old_source);

        println!("{:?}", config.source_priority);
        println!("new: {:?} - old: {:?}", new_source, old_source);
        println!("new: {:?} - old: {:?}", new_source_key, old_source_key);

        if let Some(new_source_key) = new_source_key {
            if let Some(old_source_key) = old_source_key {
                if new_source_key < old_source_key {
                    // new alarm is more important
                    return AlarmType::UpdateAlarm;
                } else if new_source_key > old_source_key {
                    // old alarm is more important
                    return AlarmType::DropAlarm;
                }
            }
        }

        AlarmType::UpdateAlarm
    } else {
        // last alarm was too long ago - new alarm
        AlarmType::FirstAlarm
    }
}