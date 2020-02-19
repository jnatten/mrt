use super::super::config;
use super::super::config::configmodels::*;
use crate::argparse::args::{ADD_TAG_ARG, DEL_TAG_ARG};
use crate::argparse::ParsedArgs;
use crate::subcommands::subcommand::MrtSubcommand;
use clap::{Arg, ArgMatches, SubCommand, Values};
use std::env;
use std::io::Result;

pub fn get() -> MrtSubcommand {
    MrtSubcommand {
        name: String::from("config"),
        run_subcommand: config,
        doc: SubCommand::with_name("config")
            .about("Subcommand to add and remove tags, generally configure mrt itself")
            .arg(
                Arg::with_name(ADD_TAG_ARG)
                    .short("a")
                    .long(ADD_TAG_ARG)
                    .value_name("TAG_NAME")
                    .multiple(true)
                    .help("Adds the current directory with specified tag"),
            )
            .arg(
                Arg::with_name(DEL_TAG_ARG)
                    .short("d")
                    .long(DEL_TAG_ARG)
                    .value_name("TAG_NAME")
                    .multiple(true)
                    .help("Deletes the current directory with specified tag"),
            ),
    }
}

fn config(args: &ArgMatches, _parsed_args: &ParsedArgs, config: ConfigFile) -> () {
    let config_with_added = match args.values_of(ADD_TAG_ARG) {
        Some(tags) => add_tag_to_current_dir(tags, config),
        None => Ok(config),
    };

    let _config_with_removed =
        config_with_added.and_then(|conf| match args.values_of(DEL_TAG_ARG) {
            Some(tags) => remove_tag_from_current_dir(tags, conf),
            None => Ok(conf),
        });
    ()
}

fn add_tag_to_current_dir(tags: Values, mut config: ConfigFile) -> Result<ConfigFile> {
    for tag in tags {
        let current_path = env::current_dir()?;
        let cp = String::from(current_path.to_str().unwrap_or(""));

        let inserted_tag = config
            .tags
            .entry(tag.to_string())
            .or_insert(Tag { paths: vec![] });
        inserted_tag.paths.push(cp);
        inserted_tag.paths.sort();
        inserted_tag.paths.dedup();
    }
    config::loader::save_config(config)
}

fn remove_tag_from_current_dir(tags: Values, mut config: ConfigFile) -> Result<ConfigFile> {
    for tag in tags {
        let current_path = env::current_dir()?;
        let cp = String::from(current_path.to_str().unwrap_or(""));
        let tag_to_remove_path_from = config.tags.get_mut(tag);

        match tag_to_remove_path_from {
            Some(tag_to_mod) => {
                tag_to_mod.paths.retain(|path| *path != cp);

                if tag_to_mod.paths.is_empty() {
                    config.tags.remove(tag);
                };
            }
            _ => println!("Didn't exist as tag /shrug"),
        }
    }
    config::loader::save_config(config)
}
