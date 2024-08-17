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
    alarm_template_keywords: std::collections::HashMap<String, String>,
    mail_schema: String,
    stichwoerter: std::collections::HashMap<String, String>,
    ignore_units: Vec<String>,
}

#[derive(Deserialize)]
struct SerialConfig {
    port: String,
    delimiter: String,
    baudrate: u32,
    alarm_list: Vec<String>,
    rics: Vec<RicConfig>,
}

#[derive(Deserialize)]
struct RicConfig {
    key: String,
    groups: Vec<String>,
    vehicles: Vec<String>,
    members: Vec<String>,
}