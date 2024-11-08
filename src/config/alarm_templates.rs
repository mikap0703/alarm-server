use std::collections::HashMap;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct AlarmTemplates {
    #[serde(flatten)]
    pub templates: HashMap<String, AlarmTemplateConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AlarmTemplateConfig {
    #[serde(flatten)]
    pub apis: HashMap<String, AlarmTemplateReceiver>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum AlarmTemplateReceiver {
    Api {
        members: Option<Vec<String>>,
        groups: Option<Vec<String>>,
        vehicles: Option<Vec<String>>,
    },
    Webhooks(Vec<String>),
}
