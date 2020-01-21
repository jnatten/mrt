use std::result::Result;
use std::process::Command;
use super::config::configmodels::ConfigFile;
use super::argparse::ParsedArgs;
use super::mrt_errors::MrtError;
use super::mrt_errors;
use colored::Colorize;

struct ExecutionOutput {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn get_all_paths(tags: Vec<String>, config: ConfigFile) -> Vec<String> {
    let mut all_paths: Vec<String> = tags.iter().flat_map(|t| {
        let tag_without_prefix: &str = t.as_str()[1..].as_ref(); // TODO: slice this in a better way, this may panic!!!
        let x = match config.tags.get(tag_without_prefix) {
            Some(tag) => { tag.paths.clone() }
            None => {
                println!("Config not found for tag '{}', skipping...", t);
                vec![]
            }
        };
        x
    }).collect();

    all_paths.sort();
    all_paths.dedup();
    all_paths
}

fn print_result(path: String, output: ExecutionOutput) -> () {
    let headline = format!("in {}", path.as_str());
    if output.exit_code == 0 {
        println!("{}", headline.bright_black());
    } else {
        let code = format!("{}", output.exit_code);
        println!("{} ({})", headline.bright_black(), code.red());
    }

    if output.stdout.len() > 0 { println!("\n{}", output.stdout); }
    if output.stderr.len() > 0 { eprintln!("\n{}", output.stderr.red()); }
}

pub fn exec(parsed_args: ParsedArgs, config: ConfigFile) -> Result<i8, MrtError> {
    // TODO: Parallelization

    let program = parsed_args.after_tags.first();

    match program {
        None => Err(mrt_errors::new("Nothing to execute")),
        Some(prog) => {
            let args = &parsed_args.after_tags[1..];
            let mut cmd = Command::new(prog);
            cmd.args(args);

            let all_paths = get_all_paths(parsed_args.tags, config);

            for path in all_paths {
                match exec_at_path(&mut cmd, &path) {
                    Ok(res) => print_result(path, res),
                    _ => ()
                }
            }
            Ok(0) // TODO: Somehow handle errors, maybe map over rather than for loop and print all
        }
    }
}

fn exec_at_path(cmd: &mut Command, path: &String) -> Result<ExecutionOutput, MrtError> {
    cmd.current_dir(&path);
    match cmd.output() {
        Ok(output) => {
            let stdout_string = std::str::from_utf8(&output.stdout);
            let stderr_string = std::str::from_utf8(&output.stderr);
            match (stdout_string, stderr_string) {
                (Ok(out), Ok(err)) => {
                    let exit_code: i32 = match output.status.code() {
                        Some(int) => int,
                        _ => -255
                    };

                    let execution = ExecutionOutput {
                        exit_code,
                        stdout: String::from(out),
                        stderr: String::from(err),
                    };

                    Ok(execution)
                }
                _ => {
                    println!("Couldn't convert output to string...");
                    Err(mrt_errors::new("Couldn't convert output to string..."))
                }
            }
        }
        Err(e) => {
            let msg = format!("Something went wrong when executing command at {}:", path);
            println!("{}\n\n{}\n", msg.red(), e);
            Err(mrt_errors::new("Execution failed..."))
        } // TODO: Better msg
    }
}
