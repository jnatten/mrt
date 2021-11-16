#![deny(clippy::all, clippy::nursery, clippy::unwrap_used)]
mod argparse;
mod config;
mod execute;
mod subcommands;
mod util;

const APP_NAME: &str = "Multi Repo Tool";
const APP_SHORT_NAME: &str = "mrt";
const APP_VERSION: &str = "0.0.3";

use crate::subcommands::subcommand;
use crate::subcommands::subcommand::MrtSubcommand;
use anyhow::Result;
use argparse::args::*;
use clap::Arg;
use colored::Colorize;
use config::loader::get_config_path;
use config::models::ConfigFile;
use std::path::PathBuf;
use std::process::exit;

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
        "$ mrt config -a backend",
        "# Remove tag `backend` from current directory".bright_black(),
        "$ mrt config -d backend",
        "# List tagged directories".bright_black(),
        "$ mrt -l",
        "# Execute command in all directories tagged with `backend`".bright_black(),
        "$ mrt +backend sed -i 's/someversion = \"1.0.0\"/someversion = \"1.2.0\"/g' build.sbt",
        "# Execute command in all directories tagged with `backend` in parallel".bright_black(),
        "$ mrt -p +backend git pull",
        "# Execute command in all directories tagged with `backend` and `frontend` in parallel"
            .bright_black(),
        "$ mrt -p +backend +frontend git pull",
        "# List status of all directories tagged with `backend`".bright_black(),
        "$ mrt +backend status",
        "# Removes the `backend` tag entirely, leaving the directories intact".bright_black(),
        "$ mrt config -D backend",
        "# Removes all tags from current directory".bright_black(),
        "$ mrt config -r",
        "# Execute command in specified directory".bright_black(),
        "$ mrt +/opt/somedir ls -l",
        "# Execute command in dirty repositories".bright_black(),
        "$ mrt -m git diff",
        "# Launch a tmux session with a pane for each of the directories tagged with `backend`"
            .bright_black(),
        "$ mrt +backend tmux"
    )
}

fn start_with_config(config: ConfigFile) -> Result<i32> {
    let subcmds: Vec<MrtSubcommand> = subcommand::get_subcommands();
    let parsed_arguments = argparse::parse_arguments(&subcmds);

    let args = clap::App::new(APP_NAME)
        .version(APP_VERSION)
        .usage(format!("{} [FLAGS] [+tag ..] [--] [command]", APP_SHORT_NAME).as_ref())
        .after_help(help_text().as_ref())
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
        .arg(
            Arg::with_name(SHELL_EXECUTION_ARG)
                .short("s")
                .long(SHELL_EXECUTION_ARG)
                .multiple(false)
                .help("Will make command be executed in the context of a shell. \nIE: `bash -c '<command>'`\n`powershell /C '<command>' on windows.")
        )
        .arg(
            Arg::with_name(PANIC_ON_NON_ZERO_ARG)
                .short("P")
                .long(PANIC_ON_NON_ZERO_ARG)
                .multiple(false)
                .help("Makes mrt quit if it encounters a non-zero exit code.")
        )
        .arg(
            Arg::with_name(ONLY_IN_MODIFIED)
                .short("m")
                .long(ONLY_IN_MODIFIED)
                .multiple(false)
                .help("Only execute command in modified repos (Modification detected by git-status).")
        )
        .arg(
            Arg::with_name(USE_LAST_PATHS)
                .short("L")
                .long(USE_LAST_PATHS)
                .multiple(false)
                .help("Execute command in paths from previous execution of mrt.")
        )
        .subcommands(subcmds.iter().map(|cmd| cmd.doc.to_owned()))
        .get_matches_from(&parsed_arguments.before_tags);

    argparse::handle_args_to_self(subcmds, &args, &parsed_arguments, config)
        .and_then(|c| execute::exec(&args, parsed_arguments, c))
}

#[cfg(target_os = "windows")]
fn configure_colored_crate() {
    colored::control::set_virtual_terminal(true).expect(
        "Something is really wrong, colored::controll::set_virtual_terminal should always be OK",
    );
}

// This empty function exists so compilation doesn't fail on platforms that doesn't need the configuration of colored
#[cfg(not(target_os = "windows"))]
const fn configure_colored_crate() {}

fn main() {
    configure_colored_crate();

    let config_path = get_config_path().unwrap_or_else(|| PathBuf::from(".mrtconfig.json"));
    let config_to_use = match config::loader::load_config(config_path.as_path()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}\n{}", "ERROR: Could not load config...".red(), e);
            exit(2);
        }
    };

    if let Ok(exit_code) = start_with_config(config_to_use) {
        exit(exit_code)
    } else {
        exit(1)
    }
}
