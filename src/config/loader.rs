use super::configmodels::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::path::Path;
use crate::APP_VERSION;
use std::collections::HashMap;

const CONFIG_ENV_NAME: &str = "MRT_CONFIG_PATH";

pub fn load_config(path: &String) -> Result<ConfigFile> {
    let config_string = read_file_to_string(path)?;
    let data: ConfigFile = serde_json::from_str(&config_string)?;
    Ok(data)
}

pub fn save_config(config: ConfigFile) -> Result<(ConfigFile)> {
    let config_path = get_config_path().unwrap_or(String::from(".mrtconfig.json"));
    save_config_at(&config_path, config)
}

fn save_config_at(path: &String, config_struct: ConfigFile) -> Result<(ConfigFile)> {
    let data = serde_json::to_string_pretty(&config_struct)?;

    let mut file = File::create(path.as_str())?;
    file.write_all(data.as_bytes())?;

    Ok(config_struct)
}

pub fn create_new_empty_config(path: &String) -> Result<(ConfigFile)> {
    let new_config = ConfigFile {
        version: String::from(APP_VERSION),
        tags: HashMap::new(),
    };

    save_config_at(path, new_config)
}

fn read_file_to_string(path: &String) -> Result<String> {
    let mut file = File::open(path.as_str())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn get_config_path() -> Option<String> {
    let config_path = match std::env::var(CONFIG_ENV_NAME) {
        Ok(path) => Some(path),
        _ => {
            let config_dir = dirs::home_dir()?;
            let config_filename = Path::new(".mrtconfig.json");
            let combined_path = config_dir.join(config_filename);
            match combined_path.to_str() {
                Some(p) => Some(String::from(p)),
                _ => {
                    eprintln!("Could not get a valid config path...");
                    Some(String::from(""))
                }
            }
        }
    };

    config_path
}
