use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
    pub version: String,
    pub tags: HashMap<String, Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub paths: Vec<String>,
}
