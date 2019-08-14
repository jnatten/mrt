mod config;

const APP_NAME: &str = "Multi Repo Tool";
const CONFIG_ENV_NAME: &str = "MRT_CONFIG_PATH";
const APP_VERSION: &str = "0.0.1";


use config::configmodels::ConfigFile;
use std::path::Path;

fn do_stuff(_config: ConfigFile) {
    println!("YAY");
}

fn get_config_path() -> Option<String> {
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

fn main() {
    clap::App::new(APP_NAME)
        .version(APP_VERSION)
        .get_matches();

    let config_path = get_config_path().unwrap_or(String::from(".mrtconfig.json"));
    let config_to_use = match config::loader::load_config(&config_path) {
        Ok(config) => config,
        _ => match config::loader::create_new_empty_config(&config_path) {
            Ok(config) => config,
            _ => {
                println!("Something went wrong, exiting");
                ::std::process::exit(1)
            }
        }
    };

    do_stuff(config_to_use)
}
