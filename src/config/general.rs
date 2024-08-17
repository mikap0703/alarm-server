use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GeneralConfig {
    apis: Vec<ApiConfig>,
    timeout: u64,
    serial_dme: bool,
    mail: bool,
    alarm: bool,
}

#[derive(Deserialize, Debug)]
struct ApiConfig {
    name: String,
    api: ApiType,
    api_key: String,
}

#[derive(Deserialize, Debug)]
enum ApiType {
    Divera,
    Alamos,
    Telegram,
}