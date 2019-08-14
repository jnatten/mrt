use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
    pub version: String,
    pub tags: Vec<Tag>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Tag {
    pub name: String,
    pub paths: Vec<String>,
}
