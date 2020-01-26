use super::argparse::ParsedArgs;
use super::config::configmodels::ConfigFile;
use super::mrt_errors::MrtError;
use crate::argparse::args::*;
use clap::ArgMatches;
use colored::Colorize;
use rayon::prelude::*;
use std::process::Command;
use std::result::Result;

struct ExecutionOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn get_all_paths(tags: Vec<String>, config: ConfigFile) -> Vec<String> {
    let mut all_paths: Vec<String> = tags
        .iter()
        .flat_map(|t| {
            let tag_without_prefix: &str = t.as_str()[1..].as_ref(); // TODO: slice this in a better way, this may panic!!!
            match config.tags.get(tag_without_prefix) {
                Some(tag) => tag.paths.clone(),
                None => {
                    println!("Config not found for tag '{}', skipping...", t);
                    vec![]
                }
            }
        })
        .collect();

    all_paths.sort();
    all_paths.dedup();
    all_paths
}

fn print_result(path: &String, output: &ExecutionOutput) -> () {
    let headline = format!("\nin {}", path.as_str());
    if output.exit_code == 0 {
        println!("{}", headline.bright_black());
    } else {
        let code = format!("{}", output.exit_code);
        println!("{} ({})", headline.bright_black(), code.red());
    }

    if output.stdout.len() > 0 {
        println!("\n{}", output.stdout);
    }
    if output.stderr.len() > 0 {
        eprintln!("\n{}", output.stderr.red());
    }
}

pub fn exec(
    clap_args: &ArgMatches,
    parsed_args: ParsedArgs,
    config: ConfigFile,
) -> Result<i8, MrtError> {
    let program = parsed_args.after_tags.first();

    match program {
        None => Err(MrtError::new("Nothing to execute")),
        Some(prog) => {
            let args = &parsed_args.after_tags[1..];

            let all_paths = get_all_paths(parsed_args.tags, config);

            let should_print_instantly = (!clap_args.is_present(PARALLEL_TAG))
                || clap_args.is_present(CONTINUOUS_OUTPUT_ARG);

            let execute_output = exec_all(
                all_paths,
                prog,
                args,
                clap_args.is_present(PARALLEL_TAG),
                should_print_instantly,
            )?;

            if !should_print_instantly {
                for (path, output) in execute_output {
                    match output {
                        Ok(res) => print_result(&path, &res),
                        _ => (),
                    }
                }
            }

            Ok(0) // TODO: Somehow handle errors, maybe map over rather than for loop and print all
        }
    }
}

fn exec_all(
    all_paths: Vec<String>,
    prog: &String,
    args: &[String],
    in_parallel: bool,
    should_print_instantly: bool,
) -> Result<Vec<(String, Result<ExecutionOutput, MrtError>)>, MrtError> {
    let execute_func = |path: &String| {
        (
            path.to_string(),
            exec_at_path(
                path.to_string(),
                prog.to_string(),
                args,
                should_print_instantly,
            ),
        )
    };

    if in_parallel {
        rayon::ThreadPoolBuilder::new()
            .num_threads(all_paths.len())
            .build_global()?;

        Ok(all_paths.par_iter().map(execute_func).collect())
    } else {
        Ok(all_paths.iter().map(execute_func).collect())
    }
}

fn exec_at_path(
    path: String,
    cmd: String,
    args: &[String],
    print: bool,
) -> Result<ExecutionOutput, MrtError> {
    let mut cmd = Command::new(cmd);
    cmd.args(args);
    cmd.current_dir(&path);
    let output = cmd.output()?;

    let stdout_string = std::str::from_utf8(&output.stdout);
    let stderr_string = std::str::from_utf8(&output.stderr);
    match (stdout_string, stderr_string) {
        (Ok(out), Ok(err)) => {
            let exit_code: i32 = match output.status.code() {
                Some(int) => int,
                _ => -255,
            };

            let execution = ExecutionOutput {
                exit_code,
                stdout: String::from(out),
                stderr: String::from(err),
            };

            if print {
                print_result(&path, &execution);
            }

            Ok(execution)
        }
        _ => {
            println!("Couldn't convert output to string...");
            Err(MrtError::new("Couldn't convert output to string..."))
        }
    }
}
