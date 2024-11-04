use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::alarm::{Alarm, AlarmReceiver};
use crate::apis::Api;
use crate::apis::divera_v2::DiveraV2;
use crate::apis::mock_api::MockApi;
use crate::apis::telegram::Telegram;
use crate::config::alarm_templates::AlarmTemplates;
use crate::config::general::{ApiConfig, ApiType};

pub struct AlarmHandler {
    // channel to send and receive alarms
    recv_alarms: flume::Receiver<Alarm>,
    apis: Arc<Mutex<HashMap<String, Box<dyn Api>>>>,
    alarm_templates: AlarmTemplates,
    last_alarms: Vec<Alarm>,
}

impl AlarmHandler {
    pub fn new(recv_alarms: flume::Receiver<Alarm>, api_configs: Vec<ApiConfig>, alarm_templates: AlarmTemplates) -> Self {
        let mut apis_map = HashMap::new();
        for api_config in api_configs {
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
            last_alarms: Vec::new(),
        }
    }

    pub fn start(&self) {
        let recv_alarms = self.recv_alarms.clone();
        let apis = self.apis.clone();
        let alarm_templates = self.alarm_templates.clone();
        thread::spawn( move || {
            loop {
                match recv_alarms.recv() {
                    Ok(mut alarm) => {
                        println!("AlarmHandler received alarm: {}", alarm.title);

                        let default_template = alarm_templates.default.clone();

                        for (name, receivers) in default_template {
                            alarm.receiver.insert(name, AlarmReceiver {
                                groups: receivers.groups.unwrap_or_else(|| Vec::new()),
                                vehicles: receivers.vehicles.unwrap_or_else(|| Vec::new()),
                                members: receivers.members.unwrap_or_else(|| Vec::new()),
                            });
                        }

                        let apis = apis.lock().unwrap();

                        for (api, receiver) in alarm.receiver.clone() {
                            let api = match apis.get(&api) {
                                Some(api) => api,
                                None => {
                                    eprintln!("API {} not found", api);
                                    continue;
                                }
                            };

                            api.trigger_alarm(&alarm).unwrap();
                        }

                    }
                    Err(e) => {
                        eprintln!("Error receiving alarm: {}", e);
                        break;
                    }
                }
            }
        });
    }
}