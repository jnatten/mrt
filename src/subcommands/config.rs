use super::super::config;
use super::super::config::configmodels::*;
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
                Arg::with_name("add-tag")
                    .short("a")
                    .long("add-tag")
                    .value_name("TAG_NAME")
                    .multiple(true)
                    .help("Tags the current directory with the specified tag"),
            )
            .arg(
                Arg::with_name("del-tag")
                    .short("d")
                    .long("del-tag")
                    .value_name("TAG_NAME")
                    .multiple(true)
                    .help("Untags the current directory from the specified tag"),
            )
            .arg(
                Arg::with_name("del-current")
                    .short("r")
                    .long("del-current")
                    .multiple(false)
                    .help("Untags the current directory from all tags"),
            )
            .arg(
                Arg::with_name("del-entire-tag")
                    .short("D")
                    .long("del-entire-tag")
                    .value_name("TAG_NAME")
                    .multiple(true)
                    .help("Untags all directories of the specified tag and removes it entirely"),
            ),
    }
}

fn config(args: &ArgMatches, _parsed_args: &ParsedArgs, config: ConfigFile) -> () {
    let after_add_tag = match args.values_of("add-tag") {
        Some(tags) => add_tag_to_current_dir(tags, config),
        None => Ok(config),
    };

    let after_del_tag = after_add_tag.and_then(|conf| match args.values_of("del-tag") {
        Some(tags) => remove_tag_from_current_dir(tags, conf),
        None => Ok(conf),
    });

    let after_del_entire = after_del_tag.and_then(|conf| match args.values_of("del-entire-tag") {
        Some(tags) => delete_entire_tag(tags, conf),
        None => Ok(conf),
    });

    let _after_del_current = after_del_entire.and_then(|conf| {
        if args.is_present("del-current") {
            del_current_dir(conf)
        } else {
            Ok(conf)
        }
    });

    ()
}

fn del_current_dir(mut config: ConfigFile) -> Result<ConfigFile> {
    let current_path = env::current_dir()?;
    let cp = String::from(current_path.to_str().unwrap_or(""));

    let keys_to_iterate: Vec<String> = config.tags.keys().map(|x| x.clone()).collect();

    for tag_name in keys_to_iterate {
        match config.tags.get_mut(&tag_name) {
            Some(t) => {
                t.paths.retain(|path| *path != cp);
                if t.paths.is_empty() {
                    config.tags.remove(&tag_name);
                };
            }
            _ => (),
        }
    }

    config::loader::save_config(config)
}

fn delete_entire_tag(tags: Values, mut config: ConfigFile) -> Result<ConfigFile> {
    for tag in tags {
        println!("Removing: {:#?}", tag);
        config.tags.remove(tag);
    }
    config::loader::save_config(config)
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
