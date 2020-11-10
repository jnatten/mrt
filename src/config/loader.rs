use super::models::*;

use super::super::util::expand_pathbuf;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

const CONFIG_ENV_NAME: &str = "MRT_CONFIG_PATH";

pub fn load_config(path: &Path) -> Result<ConfigFile> {
    match read_file_to_string(path) {
        Ok(config_string) => {
            let data: ConfigFile = serde_json::from_str(&config_string)?;
            Ok(expand_config_paths(data))
        }
        _ => {
            eprintln!(
                "Could not open config, at '{}', creating empty one...",
                path.display()
            );
            Ok(ConfigFile::new())
        }
    }
}

pub fn store_previous_paths(mut config: ConfigFile, paths: &Vec<PathBuf>) -> Result<ConfigFile> {
    if !paths.is_empty() {
        config.last_paths = Some(paths.clone());
        save_config(config)
    } else {
        Ok(config)
    }
}

/** Expands paths in config from shorthand to absolute paths */
fn expand_config_paths(mut config: ConfigFile) -> ConfigFile {
    let mut tags_after_expand: HashMap<String, Tag> = HashMap::new();
    for (tag_name, tag) in &config.tags {
        let paths = tag
            .paths
            .iter()
            .map(|p| expand_pathbuf(p.clone()))
            .collect::<Vec<PathBuf>>();
        tags_after_expand.insert(tag_name.clone(), Tag { paths });
    }

    config.tags = tags_after_expand;
    config
}

pub fn save_config(config: ConfigFile) -> Result<ConfigFile> {
    let config_path = get_config_path();
    match config_path {
        Some(path) => save_config_at(path.as_path(), &config).map(|()| config),
        None => Err(anyhow!("Could not detect correct config path")),
    }
}

fn save_config_at(path: &Path, config_struct: &ConfigFile) -> Result<()> {
    let data = serde_json::to_string_pretty(config_struct)?;

    let mut file = File::create(path)?;
    file.write_all(data.as_bytes())?;

    Ok(())
}

fn read_file_to_string(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

pub fn get_config_path() -> Option<PathBuf> {
    match std::env::var(CONFIG_ENV_NAME) {
        Ok(path) => Some(PathBuf::from(path)),
        _ => {
            let config_dir = dirs::home_dir()?;
            let config_filename = Path::new(".mrtconfig.json");
            let combined_path = config_dir.join(config_filename);
            Some(combined_path)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_write_config_file() -> Result<()> {
        let dir = tempdir::TempDir::new("mrttest")?;
        let dir_path = dir.path();
        let test_config_path = dir_path.join("config.json");

        let tag_path1 = dir_path.join("test1");
        let tag_path2 = dir_path.join("test2");

        let tag_to_save = Tag {
            paths: vec![tag_path1, tag_path2],
        };

        let mut tags = HashMap::new();
        tags.insert(String::from("testtag"), tag_to_save);

        let config_to_save = ConfigFile {
            version: crate::APP_VERSION.to_owned(),
            tags,
            last_paths: None,
        };

        save_config_at(&test_config_path, &config_to_save)?;
        let read_config = load_config(test_config_path.as_path())?;

        assert_eq!(config_to_save, read_config);
        dir.close()?;
        Ok(())
    }
}
