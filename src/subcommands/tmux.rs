use std::{path::PathBuf, process::Command};

use super::subcommand::MrtSubcommand;
use crate::{argparse::ParsedArgs, config::models::ConfigFile, execute, APP_SHORT_NAME};
use anyhow::Result;
use clap::{Arg, ArgMatches, SubCommand};
use uuid::Uuid;

pub fn get() -> MrtSubcommand {
    MrtSubcommand {
        name: String::from("tmux"),
        run_subcommand: |args, parsed_args, config| tmux(args, parsed_args, config),
        doc: SubCommand::with_name("tmux")
            .about("Launch a tmux session, with panes opened in directories of the specified tags")
            .arg(
                Arg::with_name("detached")
                    .short("d")
                    .long("detached")
                    .help("Whether or not the tmux session should spawn in a detached state."),
            ),
    }
}

fn tmux(args: &ArgMatches, parsed_arguments: &ParsedArgs, config: ConfigFile) -> () {
    match open_tmux(args, parsed_arguments, config) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("We got an error when trying to open tmux:\n{}", e)
        }
    };
}

fn open_tmux(args: &ArgMatches, parsed_arguments: &ParsedArgs, config: ConfigFile) -> Result<()> {
    let session_name = spawn_new_session()?;
    let paths = execute::get_all_paths(&parsed_arguments.tags, &config, false);
    open_panes(session_name.as_str(), paths)?;
    if !args.is_present("detached") {
        attach_tmux(session_name.as_str())?;
    }
    Ok(())
}

fn attach_tmux(session_name: &str) -> Result<()> {
    let mut cmd = Command::new("tmux");
    cmd.args(&["attach", "-t", session_name]);
    cmd.spawn()?.wait()?;
    Ok(())
}

fn set_layout(session_name: &str) -> Result<()> {
    let mut cmd = Command::new("tmux");
    cmd.args(&["select-layout", "-t", session_name, "tiled"]);
    cmd.spawn()?.wait()?;

    Ok(())
}

fn open_panes(session_name: &str, paths: Vec<PathBuf>) -> Result<()> {
    for (idx, path) in paths.iter().enumerate() {
        let is_last = idx + 1 == paths.len();
        open_pane_at(session_name, path, is_last)?;
    }

    Ok(())
}

fn open_pane_at(session_name: &str, path: &PathBuf, skip_split: bool) -> Result<()> {
    let mut cd_cmd = Command::new("tmux");
    let cd = format!("cd {}", path.to_string_lossy());
    cd_cmd.args(&["send-keys", "-t", session_name, cd.as_str(), "Enter"]);
    cd_cmd.spawn()?.wait()?;

    if !skip_split {
        let mut split_cmd = Command::new("tmux");
        split_cmd.args(&["split-window", "-v", "-t", session_name]);
        split_cmd.spawn()?.wait()?;
    }

    set_layout(session_name)?;

    Ok(())
}

fn generate_session_name() -> String {
    let uuid = Uuid::new_v4();
    format!("{}-{}", APP_SHORT_NAME, uuid)
}

fn spawn_new_session() -> Result<String> {
    let session_name = generate_session_name();
    println!("Spawning tmux session: '{}'", session_name);

    let mut cmd = Command::new("tmux");
    cmd.args(&["new-session", "-d", "-s", session_name.as_str()]);
    cmd.spawn()?.wait()?; // Consider failing if exit-code != 0

    Ok(session_name)
}
