use crate::APP_VERSION;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ConfigFile {
    pub version: String,
    pub tags: HashMap<String, Tag>,
}

impl ConfigFile {
    pub fn new() -> ConfigFile {
        ConfigFile {
            version: String::from(APP_VERSION),
            tags: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Tag {
    pub paths: Vec<PathBuf>,
}
