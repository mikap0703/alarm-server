use std::collections::HashMap;
use serde_derive::Deserialize;


#[derive(Deserialize, Debug)]
pub struct AlarmTemplates {
    default: AlarmTemplateConfig,
    #[serde(flatten)]
    templates: HashMap<String, AlarmTemplateConfig>,
}

#[derive(Deserialize, Debug)]
struct AlarmTemplateConfig {
    groups: Option<Vec<String>>,
    vehicles: Option<Vec<String>>,
    members: Option<Vec<String>>,
    webhooks: Option<Vec<String>>,
}
