use crate::APP_VERSION;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct ConfigFile {
    pub version: String,
    pub tags: HashMap<String, Tag>,
    pub last_paths: Option<Vec<PathBuf>>,
}

impl ConfigFile {
    pub fn new() -> Self {
        Self {
            version: String::from(APP_VERSION),
            tags: HashMap::new(),
            last_paths: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Tag {
    pub paths: Vec<PathBuf>,
}
