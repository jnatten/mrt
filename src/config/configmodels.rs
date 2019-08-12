pub struct ConfigFile {
    pub version: String,
    pub tags: Vec<Tag>
}

pub struct Tag {
    pub paths: Vec<String>
}
