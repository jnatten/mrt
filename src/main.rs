mod config;
mod argparse;
mod execute;
mod mrt_errors;

const APP_NAME: &str = "Multi Repo Tool";
const APP_VERSION: &str = "0.0.1";

use config::configmodels::ConfigFile;
use argparse::TAG_PREFIX;
use argparse::ADD_TAG_ARG;
use argparse::LIST_TAGS_ARG;
use argparse::PARALLEL_TAG;
use config::loader::get_config_path;
use std::result::Result;
use std::process::exit;
use crate::argparse::DEL_TAG_ARG;

fn start_with_config(config: ConfigFile) -> Result<i8, mrt_errors::MrtError> {
    let parsed_arguments = argparse::parse_arguments();

    let args = clap::App::new(APP_NAME)
        .version(APP_VERSION)
        .arg(
            clap::Arg::with_name(ADD_TAG_ARG)
                .short("a")
                .long(ADD_TAG_ARG)
                .value_name("TAG_NAME")
                .multiple(true)
                .help(format!("Adds the current directory with specified {}tag", TAG_PREFIX).as_ref())
        )
        .arg(
            clap::Arg::with_name(DEL_TAG_ARG)
                .short("d")
                .long(DEL_TAG_ARG)
                .value_name("TAG_NAME")
                .multiple(true)
                .help(format!("Deletes the current directory with specified {}tag", TAG_PREFIX).as_ref())
        )
        .arg(
            clap::Arg::with_name(LIST_TAGS_ARG)
                .short("l")
                .long(LIST_TAGS_ARG)
                .multiple(false)
                .help(format!("List all specified {}tag's and paths that are tagged...", TAG_PREFIX).as_ref())
        )
        .arg(
            clap::Arg::with_name(PARALLEL_TAG)
                .short("p")
                .long(PARALLEL_TAG)
                .multiple(false)
                .help("Execute at each tagged path in parallel\nThis stores output until all executions are finished and then prints them in sequence.")
        )
        .get_matches_from(&parsed_arguments.before_tags);


    argparse::handle_args_to_self(&args, config).and_then(|c| {
        execute::exec(parsed_arguments, c)
    })
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

    let result = start_with_config(config_to_use);
    if result.is_ok() {
        exit(0)
    } else {
        exit(1)
    }
}
