use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct GeneralConfig {
    pub apis: Vec<ApiConfig>,
    pub timeout: u64,
    pub source_priority: Vec<String>,
    pub alarm: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiConfig {
    pub name: String,
    pub api: ApiType,
    pub api_key: String,
}

#[derive(Deserialize, Debug, Clone)]
pub enum ApiType {
    Divera,
    Alamos,
    Telegram,
}