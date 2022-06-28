use super::super::argparse::ParsedArgs;
use super::super::config::models::ConfigFile;
use super::super::execute;
use super::super::util;
use crate::subcommands::subcommand::MrtSubcommand;
use clap::SubCommand;
use colored::{ColoredString, Colorize};
use std::path::Path;
use std::process::Command;
use std::{cmp::max, collections::VecDeque};

const DEFAULT_BRANCH: &str = "master";

pub fn get() -> MrtSubcommand {
    MrtSubcommand {
        name: String::from("status"),
        run_subcommand: |_, parsed_args, config| status(parsed_args, config),
        doc: SubCommand::with_name("status").about("Status of directories with specified tags"),
    }
}

fn status(parsed_arguments: &ParsedArgs, config: ConfigFile) {
    let paths = execute::get_all_paths(&parsed_arguments.tags, &config, false);

    for path in paths {
        println!("{}", run_status(&path));
    }
}

fn run_status(path: &Path) -> String {
    let default_branch = get_default_branch(path);
    match run_status_command(path).output() {
        Ok(output) => format_output(path, &output.stdout, default_branch),
        _ => format_error(path),
    }
}

fn get_remote(path: &Path) -> String {
    let mut cmd = Command::new("git");

    cmd.args(&["remote"]).current_dir(path);

    let maybe_remote = match cmd.output() {
        Ok(output) => {
            let output_string = String::from_utf8_lossy(&output.stdout).to_string();
            let mut lines: VecDeque<String> = output_string.split('\n').map(String::from).collect();
            lines.pop_front()
        }
        _ => None,
    };

    maybe_remote.unwrap_or_else(|| String::from("origin"))
}

fn get_default_branch(path: &Path) -> String {
    let mut cmd = Command::new("git");

    let remote = get_remote(path);
    cmd.args(&["-c", "color.ui=always"])
        .args(&[
            "symbolic-ref",
            format!("refs/remotes/{}/HEAD", remote).as_str(),
        ])
        .current_dir(path);

    let maybe_default = match cmd.output() {
        Ok(output) => {
            let output_string = String::from_utf8_lossy(&output.stdout).to_string();
            let mut lines: VecDeque<String> = output_string.split('\n').map(String::from).collect();
            lines.pop_front().and_then(|l| {
                l.split('/')
                    .map(String::from)
                    .collect::<Vec<String>>()
                    .pop()
            })
        }
        _ => None,
    };

    match maybe_default {
        Some(branch) if branch.is_empty() => String::from(DEFAULT_BRANCH),
        None => String::from(DEFAULT_BRANCH),
        Some(default_branch) => default_branch,
    }
}

pub fn run_status_command(path: &Path) -> Command {
    let mut cmd = Command::new("git");

    cmd.args(&["-c", "color.ui=always"])
        .args(&["status", "--branch", "--porcelain"])
        .current_dir(path);

    cmd
}

fn format_error(path: &Path) -> String {
    let formatted_path = util::format_path(path).red();
    let path_spaces = get_spaces_with_maxlen(50, formatted_path.len());

    format!(
        "{}{}{}",
        formatted_path,
        path_spaces,
        "SOMETHING WRONG".red()
    )
}

fn format_output(path: &Path, out: &[u8], default_branch: String) -> String {
    let output_string = String::from_utf8_lossy(out).to_string();
    let lines: Vec<String> = output_string.split('\n').map(String::from).collect();

    let branch = get_colored_branch(&lines, default_branch);
    let dirtyness = get_dirtyness(&lines);
    let behindness: String = get_colored_behindness(&lines);

    let dirtyness_spaces = get_spaces_with_maxlen(25, dirtyness.len());

    let formatted_path = util::format_path(path);
    let path_spaces = get_spaces_with_maxlen(50, formatted_path.len());

    format!(
        "{}{}{}{}{}{}",
        formatted_path, path_spaces, dirtyness, dirtyness_spaces, branch, behindness
    )
}

fn get_spaces_with_maxlen(max_len: i32, string_length: usize) -> String {
    let x = max_len - (string_length as i32);
    let y: usize = max(1, x) as usize;
    " ".repeat(y)
}

pub fn get_num_dirty_files(lines: &[String]) -> usize {
    let modified_files = lines
        .iter()
        .filter(|l| !(l.starts_with("## ") || l.is_empty()));

    modified_files.count()
}

fn get_dirtyness(lines: &[String]) -> String {
    let num_modified = get_num_dirty_files(lines);

    if num_modified != 0 {
        let text = format!("{} modified", num_modified);
        format!("{}", text.red())
    } else {
        format!("{}", "Clean".green())
    }
}

fn get_colored_branch(lines: &[String], default_branch: String) -> ColoredString {
    get_branch(lines)
        .map(|s| {
            if s != default_branch {
                s.bright_black()
            } else {
                s.normal()
            }
        })
        .unwrap_or_else(|| "<UNKNOWN>".yellow())
}

fn get_branch(lines: &[String]) -> Option<String> {
    lines.first().map(|branch_line| {
        let mut split: Vec<String> = branch_line.split("## ").map(String::from).collect();
        if split.len() > 1 {
            split.reverse();
            split.pop();
            split.reverse();
        }
        let joined: String = split.join("## ");

        let mut dotsplit: Vec<String> = joined.split("...").map(String::from).collect();
        let middle_idx = dotsplit.len() / 2;

        // If no remote
        if middle_idx == 0 {
            dotsplit.pop().unwrap_or_default()
        } else {
            while dotsplit.len() > middle_idx {
                dotsplit.pop();
            }
            dotsplit.join("...")
        }
    })
}

fn get_behindness(lines: &[String]) -> Option<String> {
    lines.first().and_then(|branch_line| {
        if branch_line.ends_with(']') {
            let mut split: Vec<String> = branch_line.split(" [").map(String::from).collect();
            split.pop().map(|l| format!("[{}", l))
        } else {
            None
        }
    })
}

fn get_colored_behindness(lines: &[String]) -> String {
    get_behindness(lines)
        .map(|b| format!(" {}", b.yellow()))
        .unwrap_or_default()
}

#[cfg(test)]
mod test {
    use super::*;

    fn to_string_vec(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(|s| s.to_owned()).collect()
    }

    #[test]
    fn test_get_behindness_func() {
        let input1 = to_string_vec(vec!["## mas...[ter...origin/mas...[ter [behind 1]"]);
        let input2 = to_string_vec(vec!["## master...origin/master [behind 2]"]);
        let input3 = to_string_vec(vec!["## mas...[ter...origin/mas...[ter"]);
        let input4 = to_string_vec(vec!["## master...origin/master"]);

        let expected1 = String::from("[behind 1]");
        let expected2 = String::from("[behind 2]");

        assert_eq!(get_behindness(&input1), Some(expected1));
        assert_eq!(get_behindness(&input2), Some(expected2));
        assert_eq!(get_behindness(&input3), None);
        assert_eq!(get_behindness(&input4), None);
    }

    #[test]
    fn test_get_branch_func() {
        let input1 = to_string_vec(vec!["## master...origin/master"]);
        let input2 = to_string_vec(vec!["## mas## ter...origin/mas## ter"]);
        let input3 = to_string_vec(vec!["## mas...## ter...origin/mas...## ter"]);
        let input4 = to_string_vec(vec!["## mas...[ter...origin/mas...[ter [behind 1]"]);
        let input5 = to_string_vec(vec!["## master...origin/master [behind 1]"]);

        let expected1 = String::from("master");
        let expected2 = String::from("mas## ter");
        let expected3 = String::from("mas...## ter");
        let expected4 = String::from("mas...[ter");
        let expected5 = String::from("master");

        assert_eq!(get_branch(&input1), Some(expected1));
        assert_eq!(get_branch(&input2), Some(expected2));
        assert_eq!(get_branch(&input3), Some(expected3));
        assert_eq!(get_branch(&input4), Some(expected4));
        assert_eq!(get_branch(&input5), Some(expected5));
    }

    #[test]
    fn test_get_branch_without_remote() {
        let input = to_string_vec(vec!["## some-branch"]);
        let expected = String::from("some-branch");
        assert_eq!(get_branch(&input), Some(expected));
    }
}
