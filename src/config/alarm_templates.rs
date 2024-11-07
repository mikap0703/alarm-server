use std::collections::HashMap;
use serde_derive::Deserialize;


#[derive(Deserialize, Debug, Clone)]
pub struct AlarmTemplates {
    pub default: HashMap<String, AlarmTemplateConfig>,
    #[serde(flatten)]
    pub templates: HashMap<String, AlarmTemplateConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AlarmTemplateConfig {
    pub groups: Option<Vec<String>>,
    pub vehicles: Option<Vec<String>>,
    pub members: Option<Vec<String>>,
    pub webhooks: Option<Vec<String>>,
}
