mod argparse;
mod config;
mod execute;
mod mrt_errors;

const APP_NAME: &str = "Multi Repo Tool";
const APP_SHORT_NAME: &str = "mrt";
const APP_VERSION: &str = "0.0.1";

use argparse::args::*;
use colored::Colorize;
use config::configmodels::ConfigFile;
use config::loader::get_config_path;
use std::process::exit;
use std::result::Result;
use clap::Arg;

fn help_text() -> String {
    format!(
        "EXAMPLES:
    {}
    {}

    {}
    {}

    {}
    {}

    {}
    {}

    {}
    {}

    {}
    {}
    ",
        "# Tag current directory with tag `backend`".bright_black(),
        "$ mrt -a backend",
        "# Remove tag `backend` from current directory".bright_black(),
        "$ mrt -d backend",
        "# List tagged directories".bright_black(),
        "$ mrt -l",
        "# Execute command in all directories tagged with `backend`".bright_black(),
        "$ mrt +backend sed -i 's/someversion = \"1.0.0\"/someversion = \"1.2.0\"/g build.sbt",
        "# Execute command in all directories tagged with `backend` in parallel".bright_black(),
        "$ mrt -p +backend git pull",
        "# Execute command in all directories tagged with `backend` and `frontend` in parallel"
            .bright_black(),
        "$ mrt -p +backend +frontend git pull"
    )
}

fn start_with_config(config: ConfigFile) -> Result<i8, mrt_errors::MrtError> {
    let parsed_arguments = argparse::parse_arguments();
    let input_args: Vec<String> = std::env::args().collect();

    // Dynamically generate hidden args based on tags
    // TODO: Move this to a separate function when you get better at rust, and probably clean up code as well
    let mut tag_vec = vec![];
    for tag in config.tags.keys() {
        tag_vec.push(
            format!("+{}", tag).to_owned()
        );
    }

    let str_vec: Vec<&str> = tag_vec.iter().map(|t| {
        t.as_ref()
    }).collect();

    let arg_vec: Vec<Arg> = str_vec.iter().map(|t| {
        Arg::with_name(t)
            .long(t)
            .hidden(false)
    }).collect();


    let args = clap::App::new(APP_NAME)
        .version(APP_VERSION)
        // .usage(format!("{} [FLAGS] [OPTIONS] [+tag ..] [--] [command]", APP_SHORT_NAME).as_ref())
        .after_help(help_text().as_ref())
        .arg(
            Arg::with_name(ADD_TAG_ARG)
                .short("a")
                .long(ADD_TAG_ARG)
                .value_name("TAG_NAME")
                .multiple(true)
                .help(format!("Adds the current directory with specified {}tag", TAG_PREFIX).as_ref())
        )
        .arg(
            Arg::with_name(DEL_TAG_ARG)
                .short("d")
                .long(DEL_TAG_ARG)
                .value_name("TAG_NAME")
                .multiple(true)
                .help(format!("Deletes the current directory with specified {}tag", TAG_PREFIX).as_ref())
        )
        .arg(
            Arg::with_name(LIST_TAGS_ARG)
                .short("l")
                .long(LIST_TAGS_ARG)
                .multiple(false)
                .help(format!("List all specified {}tag's and paths that are tagged...", TAG_PREFIX).as_ref())
        )
        .arg(
            Arg::with_name(PARALLEL_TAG)
                .short("p")
                .long(PARALLEL_TAG)
                .multiple(false)
                .help(format!("Execute at each tagged path in parallel\nThis stores output until all executions are finished and then prints them in sequence, unless --{} specified.", CONTINUOUS_OUTPUT_ARG).as_ref())
        )
        .arg(
            Arg::with_name(CONTINUOUS_OUTPUT_ARG)
                .short("c")
                .long(CONTINUOUS_OUTPUT_ARG)
                .multiple(false)
                .help(format!("Will make output from commands executed in parallel with --{} argument print to terminal before every command has been executed.", PARALLEL_TAG).as_ref())
        )
        .args(&arg_vec)
        .get_matches_from(&input_args);

    println!("Test {:#?}", input_args);
    Ok(0)

    /*
    argparse::handle_args_to_self(&args, config)
        .and_then(|c| {
            execute::exec(&args, parsed_arguments, c)
        })
    */
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
        },
    };

    let result = start_with_config(config_to_use);
    if result.is_ok() {
        exit(0)
    } else {
        exit(1)
    }
}
