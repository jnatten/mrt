
#[derive(Deserialize, Debug)]
pub struct ConfigFile {
    pub version: String,
    pub tags: Vec<Tag>
}

#[derive(Deserialize, Debug)]
pub struct Tag {
    pub name: String,
    pub paths: Vec<String>
}
