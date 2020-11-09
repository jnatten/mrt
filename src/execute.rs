use super::argparse::ParsedArgs;
use super::config::models::ConfigFile;
use super::util;
use crate::argparse::args::*;
use crate::subcommands::status::{get_num_dirty_files, run_status_command};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use colored::Colorize;
use rayon::prelude::*;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};

struct ExecutionOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

pub fn get_all_paths(tags: &[String], config: &ConfigFile, only_in_modified: bool) -> Vec<PathBuf> {
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
                        let path = util::expand_path(tag_without_prefix);
                        if path.exists() {
                            vec![path]
                        } else {
                            println!("Tag or Path '{}' not found, skipping...", t);
                            vec![]
                        }
                    }
                }
            })
            .collect()
    };

    all_paths.sort();
    all_paths.dedup();
    if only_in_modified {
        match get_modified_paths(all_paths.clone()) {
            Ok(ps) => ps,
            Err(e) => {
                eprintln!(
                    "Error when detecting whether paths where modified or not: {}",
                    e
                );
                all_paths
            }
        }
    } else {
        all_paths
    }
}

fn get_modified_paths(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(paths.len())
        .build_global()?;

    paths
        .into_iter()
        .filter_map(|p| match is_modified(&p) {
            Ok(modified) if modified => Some(Ok(p)),
            Ok(_) => None,
            Err(e) => Some(Err(e)),
        })
        .collect::<Result<Vec<PathBuf>>>()
}

fn is_modified(path: &PathBuf) -> Result<bool> {
    let output = run_status_command(path).output()?;
    let output_string = String::from_utf8_lossy(&output.stdout).to_string();
    let lines: Vec<String> = output_string.split('\n').map(String::from).collect();
    let is_dirty = get_num_dirty_files(&lines) != 0;

    Ok(is_dirty)
}

fn get_headline(path: &PathBuf) -> String {
    let (prefix, basename) = util::split_on_basename(path);
    format!(
        "\n\n{} {}{}",
        "in".bright_black().dimmed(),
        prefix.bright_black().dimmed(),
        basename.bright_black()
    )
}

fn print_result(path: &PathBuf, output: &ExecutionOutput) {
    let headline = get_headline(&path);
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

pub fn exec(clap_args: &ArgMatches, parsed_args: ParsedArgs, config: ConfigFile) -> Result<i8> {
    let program = parsed_args.after_tags.first();

    match program {
        None => Err(anyhow!("Nothing to execute")),
        Some(prog) => {
            let args = &parsed_args.after_tags[1..];

            let only_in_modified = clap_args.is_present(ONLY_IN_MODIFIED);
            let all_paths = get_all_paths(&parsed_args.tags, &config, only_in_modified);

            let should_print_instantly = (!clap_args.is_present(PARALLEL_TAG))
                || clap_args.is_present(CONTINUOUS_OUTPUT_ARG);

            let execute_in_shell = clap_args.is_present(SHELL_EXECUTION_ARG);
            let panic_on_nonzero = clap_args.is_present(PANIC_ON_NON_ZERO_ARG);

            let execute_output = exec_all(
                all_paths,
                prog,
                args,
                clap_args.is_present(PARALLEL_TAG),
                should_print_instantly,
                execute_in_shell,
                panic_on_nonzero,
            )?;

            if !should_print_instantly {
                for (path, output) in execute_output {
                    if let Ok(res) = output {
                        print_result(&path, &res)
                    }
                }
            }

            Ok(0) // TODO: Somehow handle errors, maybe map over rather than for loop and print all
        }
    }
}

type ExecuteResult = Result<ExecutionOutput>;
type ExecuteResultForAllPaths = Result<Vec<(PathBuf, ExecuteResult)>>;

fn exec_all(
    all_paths: Vec<PathBuf>,
    prog: &str,
    args: &[String],
    in_parallel: bool,
    should_print_instantly: bool,
    execute_in_shell: bool,
    panic_on_nonzero_exitcode: bool,
) -> ExecuteResultForAllPaths {
    let execute_func = |path: &PathBuf| {
        (
            path.clone(),
            exec_at_path(
                path,
                prog.to_string(),
                args,
                should_print_instantly,
                execute_in_shell,
                panic_on_nonzero_exitcode,
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
    panic_on_nonzero_exitcode: bool,
) -> ExecuteResult {
    let color_args = get_color_args(&command);

    let mut cmd = if execute_in_shell {
        if cfg!(target_os = "windows") {
            let powershell_command_arg = format!("{} {}", &command, args.join(" "));
            let mut powershell = Command::new("powershell");
            powershell.args(&["/C", powershell_command_arg.as_str()]);
            powershell
        } else {
            let bash_command_arg =
                format!("{} {} {}", &command, color_args.join(" "), args.join(" "));
            let mut bash = Command::new("bash");
            bash.args(&["-c", bash_command_arg.as_str()]);
            bash
        }
    } else {
        let mut prog = Command::new(&command);
        prog.args(color_args);
        prog.args(args);
        prog
    };

    cmd.current_dir(path);

    let execution = if print {
        exec_with_connected_outputs(cmd, path)?
    } else {
        exec_with_captured_output(cmd)?
    };

    if execution.exit_code != 0 && panic_on_nonzero_exitcode {
        eprintln!(
            "\n\n{}",
            "Encountered non-zero exit code, quitting...".red()
        );
        exit(1)
    }

    Ok(execution)
}

/// Executes command and captures output in a `ExecutionOuput` struct if `Ok`
/// Useful for when we want to run commands in parallel and we don't want to print output immediately
fn exec_with_captured_output(mut cmd: Command) -> ExecuteResult {
    let mut stdout_l: Vec<String> = Vec::new();
    let mut stderr_l: Vec<String> = Vec::new();

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    let stdout = child.stdout.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    let stderr = child.stderr.as_mut().unwrap();
    let stderr_reader = BufReader::new(stderr);
    let stderr_lines = stderr_reader.lines();

    for line in stdout_lines {
        let l = line?;
        stdout_l.push(l);
    }

    for line in stderr_lines {
        let l = line?;
        stderr_l.push(l);
    }
    let code = child.wait()?;

    let exec_output = ExecutionOutput {
        exit_code: code.code().unwrap_or(-1),
        stdout: stdout_l.join("\n"),
        stderr: stderr_l.join("\n"),
    };

    Ok(exec_output)
}

/// Executes the command with the outputs attached.
/// This is useful when we want the subprocess to be able to control their own outputs completely
/// Example when using vim as a subcommand
fn exec_with_connected_outputs(mut cmd: Command, path: &PathBuf) -> ExecuteResult {
    let headline = get_headline(path);
    println!("{}\n", headline);

    let mut child = cmd.spawn()?;
    let waited = child.wait()?;
    let output = ExecutionOutput {
        exit_code: waited.code().unwrap_or(-1),
        stdout: String::default(),
        stderr: String::default(),
    };
    Ok(output)
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
