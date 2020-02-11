use super::configmodels::*;

use crate::APP_VERSION;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::path::Path;

const CONFIG_ENV_NAME: &str = "MRT_CONFIG_PATH";

pub fn load_config(path: &String) -> Result<ConfigFile> {
    let config_string = read_file_to_string(path)?;
    let data: ConfigFile = serde_json::from_str(&config_string)?;
    Ok(data)
}

pub fn save_config(config: ConfigFile) -> Result<ConfigFile> {
    let config_path = get_config_path().unwrap_or(String::from(".mrtconfig.json"));
    save_config_at(&config_path, &config).map(|()| config)
}

fn save_config_at(path: &String, config_struct: &ConfigFile) -> Result<()> {
    let data = serde_json::to_string_pretty(config_struct)?;

    let mut file = File::create(path.as_str())?;
    file.write_all(data.as_bytes())?;

    Ok(())
}

pub fn create_new_empty_config(path: &String) -> Result<ConfigFile> {
    let new_config = ConfigFile {
        version: String::from(APP_VERSION),
        tags: HashMap::new(),
    };

    save_config_at(path, &new_config).map(|()| new_config)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_write_config_file() -> Result<()> {
        let dir = tempdir::TempDir::new("mrttest")?;
        let dir_path = dir.path();
        let test_config_path = dir_path.join("config.json").to_string_lossy().to_string();

        let tag_path1 = dir_path.join("test1").to_string_lossy().to_string();
        let tag_path2 = dir_path.join("test2").to_string_lossy().to_string();

        let tag_to_save = Tag {
            paths: vec![tag_path1, tag_path2],
        };

        let mut tags = HashMap::new();
        tags.insert(String::from("testtag"), tag_to_save);

        let config_to_save = ConfigFile {
            version: crate::APP_VERSION.to_owned(),
            tags,
        };

        save_config_at(&test_config_path, &config_to_save)?;
        let read_config = load_config(&test_config_path)?;

        assert_eq!(config_to_save, read_config);
        dir.close()?;
        Ok(())
    }
}
