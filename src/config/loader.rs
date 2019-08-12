use super::configmodels;

pub fn load_config(path: &'static str) -> configmodels::ConfigFile {
    let tag1 = configmodels::Tag {
        paths: vec![String::from("Jonas"), String::from("Kari")]
    };

    let tags: Vec<configmodels::Tag> = vec![tag1];

    let config_file = configmodels::ConfigFile {
        version: String::from("apekatt"),
        tags: tags,
    };

    config_file
}
