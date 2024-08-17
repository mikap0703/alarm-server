mod alarm_sources;
mod alarm_templates;
mod general;

use std::error::Error;
use std::fs;
use std::path::Path;
use serde::Deserialize;
use crate::config::alarm_sources::AlarmSources;
use crate::config::alarm_templates::AlarmTemplates;
use crate::config::general::GeneralConfig;

pub struct Configs {
    alarm_sources: AlarmSources,
    alarm_templates: AlarmTemplates,
    general: GeneralConfig
}

fn load_config<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let config: T = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn parse_configs() -> Result<Configs, Box<dyn Error>> {
    let alarm_sources_config = Path::new("config/alarm_sources.json");

    let alarm_sources = match load_config::<AlarmSources>(alarm_sources_config) {
        Ok(config) => {
            config
        },
        Err(e) => {
            return Err(e);
        }
    };

    let alarm_templates_config = Path::new("config/alarm_templates.json");

    let alarm_templates = match load_config::<AlarmTemplates>(alarm_templates_config) {
        Ok(config) => {
            config
        },
        Err(e) => {
            return Err(e);
        }
    };

    let general_config = Path::new("config/general.json");

    let general = match load_config::<GeneralConfig>(general_config) {
        Ok(config) => {
            config
        },
        Err(e) => {
            return Err(e);
        }
    };

    // todo: validate configs (check if the api names are in the templates etc.)

    return Ok(Configs{alarm_sources, alarm_templates, general})
}