use super::config::models::*;
use crate::mrt_errors::MrtError;
use crate::subcommands::subcommand::MrtSubcommand;
use args::*;
use clap::ArgMatches;
use std::process::exit;

#[derive(Debug, PartialOrd, PartialEq)]
pub struct ParsedArgs {
    pub tags: Vec<String>,
    pub before_tags: Vec<String>,
    pub after_tags: Vec<String>,
}

impl ParsedArgs {
    fn initial() -> ParsedArgs {
        ParsedArgs {
            tags: Vec::new(),
            before_tags: Vec::new(),
            after_tags: Vec::new(),
        }
    }
}

pub mod args {
    pub const TAG_PREFIX: &str = "+";
    pub const PARALLEL_TAG: &str = "parallel";
    pub const LIST_TAGS_ARG: &str = "list-tags";
    pub const CONTINUOUS_OUTPUT_ARG: &str = "continuous-output";
    pub const SHELL_EXECUTION_ARG: &str = "bash";
    pub const PANIC_ON_NON_ZERO_ARG: &str = "panic-on-nonzero";
}

const TAG_ENV_VAR: &str = "MRT_DEFAULT_TAGS";

fn find_tags_in_args(args: &[String], subcommand_names: &[&String]) -> ParsedArgs {
    let any_tags = args.iter().any(|t| t.starts_with(TAG_PREFIX));
    let mut has_encountered_non_subcommand = false;
    let mut has_encountered_subcommand = false;
    let mut double_dash = false;

    let mut cli_tags = args.iter().fold(ParsedArgs::initial(), |mut acc, arg| {
        if subcommand_names.contains(&arg) && !has_encountered_non_subcommand {
            has_encountered_subcommand = true;
        }

        let arg_is_subcmd = subcommand_names.contains(&arg) || arg.starts_with('-');
        let is_first_arg = acc != ParsedArgs::initial();
        let found_tags = !acc.tags.is_empty();

        let found_tags_or_no_tags =
            (found_tags || (!any_tags && is_first_arg)) && !has_encountered_subcommand;
        let isnt_subcmd = !arg_is_subcmd || has_encountered_non_subcommand;

        let is_tag_before_cmd =
            arg.starts_with(TAG_PREFIX) && (acc.after_tags.is_empty() && !double_dash);

        match arg {
            a if a == "--" => double_dash = true,
            a if is_tag_before_cmd => acc.tags.push(a.clone()),
            a if (found_tags_or_no_tags && isnt_subcmd) || double_dash => {
                has_encountered_non_subcommand = true;
                acc.after_tags.push(a.clone())
            }
            a => acc.before_tags.push(a.clone()),
        };
        acc
    });

    if cli_tags.tags.is_empty() {
        cli_tags.tags = get_tags_from_env();
        cli_tags
    } else {
        cli_tags
    }
}

fn get_tags_from_env() -> Vec<String> {
    match std::env::var(TAG_ENV_VAR) {
        Ok(tag_string) => {
            let split = tag_string.split(',');
            split.map(|t| format!("+{}", t.trim())).collect()
        }
        _ => vec![],
    }
}

pub fn parse_arguments(subcommands: &[MrtSubcommand]) -> ParsedArgs {
    let subcommand_names: Vec<&String> = subcommands.iter().map(|x| &x.name).collect();

    let args = std::env::args();
    let args_vec: Vec<String> = args.collect();
    find_tags_in_args(&args_vec, &subcommand_names)
}

pub fn handle_args_to_self(
    subcommands: Vec<MrtSubcommand>,
    args: &ArgMatches,
    parsed_arguments: &ParsedArgs,
    config: ConfigFile,
) -> std::result::Result<ConfigFile, MrtError> {
    if args.is_present(LIST_TAGS_ARG) {
        println!("Config Version: {}", &config.version);
        for (tag_name, tag) in &config.tags {
            println!("{}:", tag_name);
            for path in &tag.paths {
                println!("\t{:#?}", path);
            }
        }
    }

    match args.subcommand() {
        (subcmd, Some(matched)) => {
            let found_cmd = subcommands.iter().find(|cmd| cmd.name == subcmd);
            if let Some(found) = found_cmd {
                (found.run_subcommand)(matched, parsed_arguments, config)
            };
            exit(0)
        }
        _ => Ok(config),
    }
}

#[cfg(test)]
mod test {

    use super::super::subcommand::get_subcommands;
    use super::*;

    fn to_string_vec(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(|s| s.to_owned()).collect()
    }

    fn subcmd_names() -> Vec<String> {
        let names: Vec<String> = get_subcommands().iter().map(|x| x.name.clone()).collect();
        names
    }

    #[test]
    fn test_single_tag_is_parsed_correctly() {
        let test_args: Vec<String> = to_string_vec(vec!["mrt", "-p", "+testtag", "ls", "-l", "-h"]);

        let expected = ParsedArgs {
            tags: to_string_vec(vec!["+testtag"]),
            before_tags: to_string_vec(vec!["mrt", "-p"]),
            after_tags: to_string_vec(vec!["ls", "-l", "-h"]),
        };

        let names = subcmd_names();
        let s: Vec<&String> = names.iter().collect();
        let result = find_tags_in_args(&test_args, &s);

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
        };

        let names = subcmd_names();
        let s: Vec<&String> = names.iter().map(|x| x).collect();
        let result = find_tags_in_args(&test_args, &s);

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
        };

        let expected2 = ParsedArgs {
            tags: to_string_vec(vec!["+testtag", "+testaru", "+testari"]),
            before_tags: to_string_vec(vec!["mrt"]),
            after_tags: to_string_vec(vec!["status"]),
        };

        let expected3 = ParsedArgs {
            tags: to_string_vec(vec![]),
            before_tags: to_string_vec(vec!["mrt"]),
            after_tags: to_string_vec(vec!["status"]),
        };

        let expected4 = ParsedArgs {
            tags: to_string_vec(vec![]),
            before_tags: to_string_vec(vec!["mrt", "status"]),
            after_tags: to_string_vec(vec![]),
        };

        let names = subcmd_names();
        let s: Vec<&String> = names.iter().map(|x| x).collect();
        let result1 = find_tags_in_args(&test_args1, &s);
        let result2 = find_tags_in_args(&test_args2, &s);
        let result3 = find_tags_in_args(&test_args3, &s);
        let result4 = find_tags_in_args(&test_args4, &s);

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
        };

        let names = subcmd_names();
        let s: Vec<&String> = names.iter().map(|x| x).collect();
        let result = find_tags_in_args(&test_args, &s);

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
        };

        let expected2 = ParsedArgs {
            tags: to_string_vec(vec!["+testtag"]),
            before_tags: to_string_vec(vec!["mrt"]),
            after_tags: to_string_vec(vec!["git", "status"]),
        };

        let names = subcmd_names();
        let s: Vec<&String> = names.iter().map(|x| x).collect();
        let result1 = find_tags_in_args(&test_args1, &s);
        let result2 = find_tags_in_args(&test_args2, &s);

        assert_eq!(result1, expected1);
        assert_eq!(result2, expected2);
    }

    #[test]
    fn test_sub_subcommands_are_parsed_as_before_tags() {
        let test_args1: Vec<String> =
            to_string_vec(vec!["mrt", "+testtag", "status", "-l", "-a", "apekatt"]);

        let expected1 = ParsedArgs {
            tags: to_string_vec(vec!["+testtag"]),
            before_tags: to_string_vec(vec!["mrt", "status", "-l", "-a", "apekatt"]),
            after_tags: to_string_vec(vec![]),
        };

        let names = subcmd_names();
        let s: Vec<&String> = names.iter().map(|x| x).collect();
        let result1 = find_tags_in_args(&test_args1, &s);

        assert_eq!(result1, expected1);
    }
}
