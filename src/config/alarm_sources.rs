use std::collections::HashMap;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct AlarmSources {
    pub mail_sources: Vec<MailConfig>,
    serial_sources: Vec<SerialConfig>,
}

#[derive(Deserialize, Clone)]
pub struct MailConfig {
    pub name: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub max_age: u64,
    pub alarm_sender: String,
    pub alarm_subject: String,
    pub alarm_template_keywords: HashMap<String, String>,
    pub mail_schema: String,
    pub stichwoerter: HashMap<String, String>,
    pub ignore_units: Vec<String>,
    pub polling: bool,
    pub idle: bool,
}

#[derive(Deserialize)]
struct SerialConfig {
    pub name: String,
    port: String,
    delimiter: String,
    baudrate: u32,
    alarm_list: Vec<String>,
    rics: HashMap<String, String>,
}