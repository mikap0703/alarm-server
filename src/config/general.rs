use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct GeneralConfig {
    pub apis: Vec<ApiConfig>,
    #[serde(alias = "timeout")]
    pub alarm_window_seconds: u64,
    pub source_priority: Vec<String>,
    pub alarm: bool,
    #[serde(default)]
    pub delay: u64,
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
    Typst,
}
