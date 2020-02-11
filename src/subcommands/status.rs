use super::super::argparse::ParsedArgs;
use super::super::config::configmodels::ConfigFile;
use super::super::execute;
use clap::ArgMatches;
use colored::{ColoredString, Colorize};
use std::cmp::max;
use std::process::Command;

pub fn status(args: &ArgMatches, parsed_arguments: &ParsedArgs, config: ConfigFile) {
    let paths = execute::get_all_paths(&parsed_arguments.tags, &config);

    for path in paths {
        println!("{}", run_status(&path));
    }
}

fn run_status(path: &String) -> String {
    let mut cmd = Command::new("git");

    cmd.args(&["-c", "color.ui=always"])
        .args(&["status", "--branch", "--porcelain"])
        .current_dir(path);

    match cmd.output() {
        Ok(output) => format_output(path, &output.stdout),
        _ => String::from(""),
    }
}

fn format_output(path: &String, out: &Vec<u8>) -> String {
    let output_string = String::from_utf8_lossy(out).to_string();
    let lines: Vec<String> = output_string.split('\n').map(String::from).collect();

    let branch = get_colored_branch(&lines);
    let dirtyness = get_dirtyness(&lines);
    let behindness: String = get_behindness(&lines);

    let dirtyness_spaces = get_spaces_with_maxlen(25, dirtyness.len());
    let path_spaces = get_spaces_with_maxlen(50, path.len());

    format!(
        "{}{}{}{}{}{}",
        path, path_spaces, dirtyness, dirtyness_spaces, branch, behindness
    )
}

fn get_spaces_with_maxlen(max_len: i32, string_length: usize) -> String {
    let x = max_len - (string_length as i32);
    let y: usize = max(1, x) as usize;
    " ".repeat(y)
}

fn get_dirtyness(lines: &Vec<String>) -> String {
    let modified_files: Vec<String> = lines
        .clone()
        .into_iter()
        .filter(|l| !(l.starts_with("## ") || l.is_empty()))
        .collect();

    if modified_files.len() > 0 {
        let text = format!("{} modified", modified_files.len());
        String::from(format!("{}", text.red()))
    } else {
        String::from(format!("{}", "Clean".green()))
    }
}

fn get_colored_branch(lines: &Vec<String>) -> ColoredString {
    get_branch(lines)
        .map(|s| {
            // TODO: Consider checking what is default branch rather than assume master
            if s != "master" {
                s.bright_black()
            } else {
                s.normal()
            }
        })
        .unwrap_or("<UNKNOWN>".yellow())
}

fn get_branch(lines: &Vec<String>) -> Option<String> {
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
        while dotsplit.len() > middle_idx {
            dotsplit.pop();
        }
        dotsplit.join("...")
    })
}

fn get_behindness(lines: &Vec<String>) -> String {
    lines
        .first()
        .map(|branch_line| {
            if branch_line.ends_with("]") {
                let mut split: Vec<String> = branch_line.split(" [").map(String::from).collect();
                split.pop().map(|l| format!("[{}", l))
            } else {
                None
            }
        })
        .flatten()
        .map(|b| format!(" {}", b.yellow()))
        .unwrap_or(String::new())
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
}
