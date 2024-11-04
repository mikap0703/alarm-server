pub mod alarm_sources;
pub mod alarm_templates;
pub mod general;

use std::error::Error;
use std::fs;
use std::path::Path;
use serde::Deserialize;
use crate::config::alarm_sources::AlarmSources;
use crate::config::alarm_templates::AlarmTemplates;
use crate::config::general::GeneralConfig;

pub struct Configs {
    pub alarm_sources: AlarmSources,
    pub alarm_templates: AlarmTemplates,
    pub general: GeneralConfig
}

fn load_config<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let config: T = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn parse_configs() -> Result<Configs, Box<dyn Error>> {
    let config_dir = Path::new("config");

    if !config_dir.exists() {
        println!("Config directory does not exist");
    }

    // print all files in the config directory
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let path = entry.path();
        println!("{:?}", path);
    }

    let alarm_sources_config = Path::new("config/alarm_sources.json");

    println!("lol2");
    let alarm_sources = match load_config::<AlarmSources>(alarm_sources_config) {
        Ok(config) => {
            config
        },
        Err(e) => {
            return Err(e);
        }
    };

    let alarm_templates_config = Path::new("config/alarm_templates.json");

    println!("lol");
    let alarm_templates = match load_config::<AlarmTemplates>(alarm_templates_config) {
        Ok(config) => {
            config
        },
        Err(e) => {
            return Err(e);
        }
    };

    let general_config = Path::new("config/general.json");

    println!("lol");
    let general = match load_config::<GeneralConfig>(general_config) {
        Ok(config) => {
            config
        },
        Err(e) => {
            return Err(e);
        }
    };
    println!("lol");

    // todo: validate configs (check if the api names are in the templates etc.)

    return Ok(Configs{alarm_sources, alarm_templates, general})
}