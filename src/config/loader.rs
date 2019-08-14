use super::configmodels::ConfigFile;

use std::fs::File;
use std::io::prelude::*;
use std::io;

pub fn load_config(path: &str) -> Result<ConfigFile, io::Error> {
    let config_string = read_file_to_string(path)?;
    let data: ConfigFile = serde_json::from_str(&config_string)?;
    Ok(data)
}


fn read_file_to_string(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}
