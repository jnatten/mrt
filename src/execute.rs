use super::argparse::ParsedArgs;
use super::config::configmodels::ConfigFile;
use super::mrt_errors::MrtError;
use crate::argparse::args::*;
use clap::ArgMatches;
use colored::Colorize;
use rayon::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::result::Result;

struct ExecutionOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

pub fn get_all_paths(tags: &[String], config: &ConfigFile) -> Vec<PathBuf> {
    let mut all_paths: Vec<PathBuf> = if tags.is_empty() {
        let nested_paths: Vec<Vec<PathBuf>> = config
            .tags
            .iter()
            .map(|(_tag_name, tag)| tag.paths.to_vec())
            .collect();

        nested_paths.into_iter().flatten().collect()
    } else {
        tags.iter()
            .flat_map(|t| {
                let tag_without_prefix = if t.is_empty() {
                    eprintln!("Got tag without content, this is probably a bug");
                    ""
                } else {
                    &t[1..]
                };
                match config.tags.get(tag_without_prefix) {
                    Some(tag) => tag.paths.clone(),
                    None => {
                        println!("Config not found for tag '{}', skipping...", t);
                        vec![]
                    }
                }
            })
            .collect()
    };

    all_paths.sort();
    all_paths.dedup();
    all_paths
}

fn print_result(path: &str, output: &ExecutionOutput) {
    let headline = format!("\nin {}", path);
    if output.exit_code == 0 {
        println!("{}", headline.bright_black());
    } else {
        let code = format!("{}", output.exit_code);
        println!("{} ({})", headline.bright_black(), code.red());
    }

    if !output.stdout.is_empty() {
        println!("\n{}", output.stdout);
    }
    if !output.stderr.is_empty() {
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

            let all_paths = get_all_paths(&parsed_args.tags, &config);

            let should_print_instantly = (!clap_args.is_present(PARALLEL_TAG))
                || clap_args.is_present(CONTINUOUS_OUTPUT_ARG);

            let execute_in_shell = clap_args.is_present(SHELL_EXECUTION_ARG);

            let execute_output = exec_all(
                all_paths,
                prog,
                args,
                clap_args.is_present(PARALLEL_TAG),
                should_print_instantly,
                execute_in_shell,
            )?;

            if !should_print_instantly {
                for (path, output) in execute_output {
                    if let Ok(res) = output {
                        print_result(path.to_str().unwrap_or("<missing>"), &res)
                    }
                }
            }

            Ok(0) // TODO: Somehow handle errors, maybe map over rather than for loop and print all
        }
    }
}

type ExecuteResult = Result<ExecutionOutput, MrtError>;
type ExecuteResultForAllPaths = Result<Vec<(PathBuf, ExecuteResult)>, MrtError>;

fn exec_all(
    all_paths: Vec<PathBuf>,
    prog: &str,
    args: &[String],
    in_parallel: bool,
    should_print_instantly: bool,
    execute_in_shell: bool,
) -> ExecuteResultForAllPaths {
    let execute_func = |path: &PathBuf| {
        (
            path.clone(), // TODO: Kanskje bare gjøre string conversion her?
            exec_at_path(
                path,
                prog.to_string(),
                args,
                should_print_instantly,
                execute_in_shell,
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
    path: &PathBuf,
    command: String,
    args: &[String],
    print: bool,
    execute_in_shell: bool,
) -> ExecuteResult {
    let color_args = get_color_args(&command);

    let mut cmd = if execute_in_shell {
        let bash_command_arg = format!("{} {} {}", &command, color_args.join(" "), args.join(" "));

        let mut cmd = Command::new("bash");
        cmd.args(&["-c", bash_command_arg.as_str()]);
        cmd
    } else {
        let mut cmd = Command::new(&command);
        cmd.args(color_args);
        cmd.args(args);
        cmd
    };

    cmd.current_dir(path);

    let mut stdout_l: Vec<String> = Vec::new();
    let mut stderr_l: Vec<String> = Vec::new();

    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let stdout = child.stdout.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    let stderr = child.stderr.as_mut().unwrap();
    let stderr_reader = BufReader::new(stderr);
    let stderr_lines = stderr_reader.lines();

    let headline = format!("\n\nin {}", path.to_str().unwrap_or("<missing>"));
    if print {
        println!("{}\n", headline.bright_black());
    }

    for line in stdout_lines {
        let l = line?;
        if print {
            println!("{}", &l);
        }
        stdout_l.push(l);
    }

    for line in stderr_lines {
        let l = line?;
        if print {
            println!("{}", &l);
        }
        stderr_l.push(l);
    }
    let code = child.wait()?;

    let execution = ExecutionOutput {
        exit_code: code.code().unwrap_or(-1),
        stdout: stdout_l.join("\n"),
        stderr: stderr_l.join("\n"),
    };
    Ok(execution)
}

fn get_color_args(cmd_name: &str) -> Vec<&str> {
    // TODO: Is it possible/easy to simulate a tty here so auto coloring for most apps could work?
    if cmd_name == "git" {
        vec!["-c", "color.ui=always"]
    } else if cmd_name == "ls" {
        vec!["--color=always"]
    } else {
        vec![]
    }
}
