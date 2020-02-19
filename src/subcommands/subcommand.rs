use crate::argparse::ParsedArgs;
use crate::config::configmodels::ConfigFile;
use clap::{App, ArgMatches};

pub struct MrtSubcommand {
    pub name: String,
    pub run_subcommand: fn(&ArgMatches, &ParsedArgs, ConfigFile) -> (),
    pub doc: App<'static, 'static>,
}

pub trait SubCmd {
    fn get() -> MrtSubcommand;
}
