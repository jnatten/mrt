use crate::argparse::ParsedArgs;
use crate::config::models::ConfigFile;
use crate::subcommands::{completion, config, status};
use clap::{App, ArgMatches};

pub struct MrtSubcommand {
    pub name: String,
    pub run_subcommand: fn(&ArgMatches, &ParsedArgs, ConfigFile) -> (),
    pub doc: App<'static, 'static>,
    // TODO: (optional?)completion function in here that allows subcommands to specify autocompletion
}

pub fn get_subcommands() -> Vec<MrtSubcommand> {
    vec![status::get(), config::get(), completion::get()]
}
