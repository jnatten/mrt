use super::config;
use super::config::configmodels::*;
use crate::mrt_errors::MrtError;
use args::*;
use clap::{ArgMatches, Values};
use std::env;
use std::io::Result;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct ParsedArgs {
    pub tags: Vec<String>,
    pub before_tags: Vec<String>,
    pub after_tags: Vec<String>,
    pub double_dash: bool, // This is just a field to know whether we have came across a -- argument when parsing
}

impl ParsedArgs {
    fn initial() -> ParsedArgs {
        ParsedArgs {
            tags: Vec::new(),
            before_tags: Vec::new(),
            after_tags: Vec::new(),
            double_dash: false,
        }
    }
}

pub mod args {
    pub const TAG_PREFIX: &str = "+";
    pub const ADD_TAG_ARG: &str = "add-tag";
    pub const DEL_TAG_ARG: &str = "del-tag";
    pub const PARALLEL_TAG: &str = "parallel";
    pub const LIST_TAGS_ARG: &str = "list-tags";
    pub const CONTINUOUS_OUTPUT_ARG: &str = "continuous-output";
    pub static SUBCOMMAND_NAMES: &'static [&str] = &["status"];
}

fn find_tags_in_args(args: &Vec<String>) -> ParsedArgs {
    let any_tags = args.iter().find(|t| t.starts_with(TAG_PREFIX)).is_some();

    let mut has_encountered_non_subcommand = false;

    args.into_iter()
        .fold(ParsedArgs::initial(), |mut acc, arg| {
            let arg_is_subcmd = SUBCOMMAND_NAMES.contains(&arg.as_str()) || arg.starts_with("-");

            match arg {
                a if a == "--" => acc.double_dash = true,
                a if a.starts_with(TAG_PREFIX)
                    && (acc.after_tags.is_empty() && !acc.double_dash) =>
                {
                    acc.tags.push(a.clone())
                }
                a if ((!acc.tags.is_empty() || (!any_tags && acc != ParsedArgs::initial()))
                    && (!arg_is_subcmd || has_encountered_non_subcommand))
                    || acc.double_dash =>
                {
                    has_encountered_non_subcommand = true;
                    acc.after_tags.push(a.clone())
                }
                a => acc.before_tags.push(a.clone()),
            };
            acc
        })
}

pub fn parse_arguments() -> ParsedArgs {
    let args = std::env::args();
    let args_vec = args.collect();
    find_tags_in_args(&args_vec)
}

pub fn handle_args_to_self(
    args: &ArgMatches,
    config: ConfigFile,
) -> std::result::Result<ConfigFile, MrtError> {
    let config_with_added = match args.values_of(ADD_TAG_ARG) {
        Some(tags) => add_tag_to_current_dir(tags, config),
        None => Ok(config),
    };

    let config_with_removed =
        config_with_added.and_then(|conf| match args.values_of(DEL_TAG_ARG) {
            Some(tags) => remove_tag_from_current_dir(tags, conf),
            None => Ok(conf),
        });

    match config_with_removed {
        Ok(conf) => {
            if args.is_present(LIST_TAGS_ARG) {
                println!("Config Version: {}", conf.version);
                for (tag_name, tag) in &conf.tags {
                    println!("{}:", tag_name);
                    for path in &tag.paths {
                        println!("\t{}", path);
                    }
                }
            }
            Ok(conf)
        }
        Err(err) => Err(MrtError::from(err)),
    }
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
            Some(tag) => tag.paths.retain(|path| *path != cp),
            _ => println!("Didn't exist as tag /shrug"),
        }
    }
    config::loader::save_config(config)
}

#[cfg(test)]
mod test {

    use super::*;

    fn to_string_vec(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(|s| s.to_owned()).collect()
    }

    #[test]
    fn test_single_tag_is_parsed_correctly() {
        let test_args: Vec<String> = to_string_vec(vec!["mrt", "-p", "+testtag", "ls", "-l", "-h"]);

        let expected = ParsedArgs {
            tags: to_string_vec(vec!["+testtag"]),
            before_tags: to_string_vec(vec!["mrt", "-p"]),
            after_tags: to_string_vec(vec!["ls", "-l", "-h"]),
            double_dash: false,
        };

        let result = find_tags_in_args(&test_args);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_tags_are_parsed_correctly() {
        let test_args: Vec<String> = to_string_vec(vec![
            "mrt", "+testtag", "+testaru", "+testari", "+x", "ls", "-l", "-h",
        ]);

        let expected = ParsedArgs {
            tags: to_string_vec(vec!["+testtag", "+testaru", "+testari", "+x"]),
            before_tags: to_string_vec(vec!["mrt"]),
            after_tags: to_string_vec(vec!["ls", "-l", "-h"]),
            double_dash: false,
        };

        let result = find_tags_in_args(&test_args);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_double_dash_makes_subcommands_after_tags() {
        let test_args1: Vec<String> =
            to_string_vec(vec!["mrt", "+testtag", "+testaru", "+testari", "status"]);
        let test_args2: Vec<String> = to_string_vec(vec![
            "mrt", "+testtag", "+testaru", "+testari", "--", "status",
        ]);

        let test_args3: Vec<String> = to_string_vec(vec!["mrt", "--", "status"]);

        let test_args4: Vec<String> = to_string_vec(vec!["mrt", "status"]);

        let expected1 = ParsedArgs {
            tags: to_string_vec(vec!["+testtag", "+testaru", "+testari"]),
            before_tags: to_string_vec(vec!["mrt", "status"]),
            after_tags: to_string_vec(vec![]),
            double_dash: false,
        };

        let expected2 = ParsedArgs {
            tags: to_string_vec(vec!["+testtag", "+testaru", "+testari"]),
            before_tags: to_string_vec(vec!["mrt"]),
            after_tags: to_string_vec(vec!["status"]),
            double_dash: true,
        };

        let expected3 = ParsedArgs {
            tags: to_string_vec(vec![]),
            before_tags: to_string_vec(vec!["mrt"]),
            after_tags: to_string_vec(vec!["status"]),
            double_dash: true,
        };

        let expected4 = ParsedArgs {
            tags: to_string_vec(vec![]),
            before_tags: to_string_vec(vec!["mrt", "status"]),
            after_tags: to_string_vec(vec![]),
            double_dash: false,
        };

        let result1 = find_tags_in_args(&test_args1);
        let result2 = find_tags_in_args(&test_args2);
        let result3 = find_tags_in_args(&test_args3);
        let result4 = find_tags_in_args(&test_args4);

        assert_eq!(result1, expected1);
        assert_eq!(result2, expected2);
        assert_eq!(result3, expected3);
        assert_eq!(result4, expected4);
    }

    #[test]
    fn test_external_commands_are_parsed_without_tags() {
        let test_args: Vec<String> = to_string_vec(vec!["mrt", "testingsaru"]);

        let expected = ParsedArgs {
            tags: to_string_vec(vec![]),
            before_tags: to_string_vec(vec!["mrt"]),
            after_tags: to_string_vec(vec!["testingsaru"]),
            double_dash: false,
        };

        let result = find_tags_in_args(&test_args);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_that_subcommands_are_not_subcommands_when_after_external() {
        let test_args1: Vec<String> = to_string_vec(vec!["mrt", "+testtag", "status"]);
        let test_args2: Vec<String> = to_string_vec(vec!["mrt", "+testtag", "git", "status"]);

        let expected1 = ParsedArgs {
            tags: to_string_vec(vec!["+testtag"]),
            before_tags: to_string_vec(vec!["mrt", "status"]),
            after_tags: to_string_vec(vec![]),
            double_dash: false,
        };

        let expected2 = ParsedArgs {
            tags: to_string_vec(vec!["+testtag"]),
            before_tags: to_string_vec(vec!["mrt"]),
            after_tags: to_string_vec(vec!["git", "status"]),
            double_dash: false,
        };

        let result1 = find_tags_in_args(&test_args1);
        let result2 = find_tags_in_args(&test_args2);

        assert_eq!(result1, expected1);
        assert_eq!(result2, expected2);
    }

    #[test]
    fn test_sub_subcommands_are_parsed_as_before_tags() {
        let test_args1: Vec<String> = to_string_vec(vec!["mrt", "+testtag", "status", "-l", "-a", "apekatt"]);

        let expected1 = ParsedArgs {
            tags: to_string_vec(vec!["+testtag"]),
            before_tags: to_string_vec(vec!["mrt", "status", "-l", "-a", "apekatt"]),
            after_tags: to_string_vec(vec![]),
            double_dash: false,
        };


        let result1 = find_tags_in_args(&test_args1);

        assert_eq!(result1, expected1);
    }
}
