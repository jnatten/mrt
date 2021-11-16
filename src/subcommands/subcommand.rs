use crate::argparse::ParsedArgs;
use crate::config::models::ConfigFile;
use crate::subcommands::{config, status, tmux};
use clap::{App, ArgMatches};

pub struct MrtSubcommand {
    pub name: String,
    pub run_subcommand: fn(&ArgMatches, &ParsedArgs, ConfigFile) -> (),
    pub doc: App<'static, 'static>,
}

pub fn get_subcommands() -> Vec<MrtSubcommand> {
    vec![status::get(), config::get(), tmux::get()]
}
