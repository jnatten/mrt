mod config;
mod argparse;

const APP_NAME: &str = "Multi Repo Tool";
const APP_VERSION: &str = "0.0.1";

use config::configmodels::ConfigFile;
use argparse::TAG_PREFIX;
use config::loader::get_config_path;
use std::io::Result;
use std::env;
use crate::config::configmodels::Tag;

fn do_stuff(mut config: ConfigFile) -> Result<ConfigFile> {
    let parsed_arguments = argparse::parse_arguments();

    let args = clap::App::new(APP_NAME)
        .version(APP_VERSION)
        .arg(
            clap::Arg::with_name("add_tag")
                .short("a")
                .long("add_tag")
                .value_name("TAG_NAME")
                .multiple(true)
                .help(format!("Adds the current directory with specified {}tag", TAG_PREFIX).as_ref())
        )
        .get_matches_from(parsed_arguments.before_tags);

    match args.values_of("add_tag") {
        Some(tags) => {
            for tag in tags {
                let current_path = env::current_dir()?;
                let cp = String::from(current_path.to_str().unwrap_or(""));

                let inserted_tag = config.tags.entry(tag.to_string())
                    .or_insert(Tag { paths: vec![] });
                inserted_tag.paths.push(cp);
                inserted_tag.paths.sort();
                inserted_tag.paths.dedup();
            }
            config::loader::save_config(config)
        }
        _ => Ok(config),
    }

    // TODO: Call execute function
    // TODO: Parallelization
}


fn main() {
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

    do_stuff(config_to_use);
}
