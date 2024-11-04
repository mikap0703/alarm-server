use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GeneralConfig {
    pub(crate) apis: Vec<ApiConfig>,
    timeout: u64,
    source_priority: Vec<String>,
    alarm: bool,
}

#[derive(Deserialize, Debug)]
pub struct ApiConfig {
    pub name: String,
    pub api: ApiType,
    pub api_key: String,
}

#[derive(Deserialize, Debug)]
pub enum ApiType {
    Divera,
    Alamos,
    Telegram,
}