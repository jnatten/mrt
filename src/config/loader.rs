use super::configmodels::*;

use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use crate::APP_VERSION;

pub fn load_config(path: &String) -> Result<ConfigFile> {
    let config_string = read_file_to_string(path)?;
    let data: ConfigFile = serde_json::from_str(&config_string)?;
    Ok(data)
}

fn save_config(path: &String, config_struct: ConfigFile) -> Result<(ConfigFile)> {
    let data = serde_json::to_string_pretty(&config_struct)?;

    let mut file = File::create(path.as_str())?;
    file.write_all(data.as_bytes())?;

    Ok(config_struct)
}

pub fn create_new_empty_config(path: &String) -> Result<(ConfigFile)> {
    let new_config = ConfigFile {
        version: String::from(APP_VERSION),
        tags: vec![],
    };

    save_config(path, new_config)
}

fn read_file_to_string(path: &String) -> Result<String> {
    let mut file = File::open(path.as_str())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
