use std::result::Result;
use std::process::Command;
use super::config::configmodels::ConfigFile;
use super::argparse::ParsedArgs;
use super::mrt_errors::MrtError;
use super::mrt_errors;

pub fn exec(parsed_args: ParsedArgs, config: ConfigFile) -> Result<i8, MrtError> {
    // TODO: Parallelization

    let program = parsed_args.after_tags.first();

    match program {
        None => Err(mrt_errors::new("Nothing to execute")),
        Some(prog) => {
            let args = &parsed_args.after_tags[1..];
            let mut cmd = Command::new(prog);
            cmd.args(args);

            let new_list_copy = parsed_args.tags.clone();
            let mut all_paths: Vec<String> = new_list_copy.iter().flat_map(|t| {
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

            all_paths.sort()
            all_paths.dedup();


            match cmd.output() {
                Ok(output) => {
                    let output_string = std::str::from_utf8(&output.stdout);
                    match output_string {
                        Ok(out) => println!("{}", out),
                        _ => println!("Couldn't convert output to string..."),
                    }

                    Ok(0)
                }
                Err(_) => Err(mrt_errors::new("Execution failed...")) // TODO: Better msg
            }
        }
    }
}
