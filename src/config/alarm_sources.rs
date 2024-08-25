use std::collections::HashMap;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct AlarmSources {
    mail_sources: Vec<MailConfig>,
    serial_sources: Vec<SerialConfig>,
}

#[derive(Deserialize)]
struct MailConfig {
    user: String,
    password: String,
    host: String,
    port: u16,
    tls: bool,
    max_age: u64,
    alarm_sender: String,
    alarm_subject: String,
    alarm_template_keywords: HashMap<String, String>,
    mail_schema: String,
    stichwoerter: HashMap<String, String>,
    ignore_units: Vec<String>,
}

#[derive(Deserialize)]
struct SerialConfig {
    port: String,
    delimiter: String,
    baudrate: u32,
    alarm_list: Vec<String>,
    rics: HashMap<String, String>,
}