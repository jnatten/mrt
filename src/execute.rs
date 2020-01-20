use std::result::Result;
use std::process::Command;
use super::config::configmodels::ConfigFile;
use super::argparse::ParsedArgs;
use super::mrt_errors::MrtError;
use super::mrt_errors;
use colored::Colorize;

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
                let result = exec_at_path(&mut cmd, &path);
            }
            Ok(0) // TODO: Somehow handle errors, maybe map over rather than for loop and print all
        }
    }
}

fn exec_at_path(cmd: &mut Command, path: &String) -> Result<(i32, String), MrtError> {
    cmd.current_dir(&path);
    match cmd.output() {
        Ok(output) => {
            let output_string = std::str::from_utf8(&output.stdout);
            match output_string {
                Ok(out) => {
                    let exit_code: i32 = match output.status.code() {
                        Some(int) => int,
                        _ => -255
                    };
                    let headline = format!("in {}", path.as_str());
                    println!("{}\n\n{}", headline.bright_black(), out);
                    Ok((exit_code, String::from(out)))
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
