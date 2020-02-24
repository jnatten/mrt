use colored::Colorize;
use std::path::{Path, PathBuf};

fn format_path_with_homedir(path: &str, home_dir: Option<PathBuf>, sep: char) -> (String, String) {
    let as_path = Path::new(path);
    let base_name = as_path
        .file_name()
        .map(|x| x.to_str().unwrap_or(""))
        .unwrap_or("")
        .to_string();
    let dir_name = as_path
        .parent()
        .map(|x| x.to_str().unwrap_or(""))
        .unwrap_or("");

    let dir_to_use = match home_dir {
        Some(home) => dir_name.replace(home.to_str().unwrap_or(""), "~"),
        _ => dir_name.to_string(),
    };

    let prefix = format!("{}{}", dir_to_use, sep);
    (prefix, base_name)
}

pub fn format_path(path: &str) -> String {
    let (prefix, basename) = split_on_basename(path);
    format!("{}{}", prefix.dimmed(), basename.normal())
}

pub fn split_on_basename(path: &str) -> (String, String) {
    let home_dir = dirs::home_dir();
    format_path_with_homedir(path, home_dir, std::path::MAIN_SEPARATOR)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unix_formatting() {
        let path1 = String::from("/home/test/dev/mrt");
        let result1 = format_path_with_homedir(&path1, Some(PathBuf::from("/home/test")), '/');
        let expected1 = (String::from("~/dev/"), String::from("mrt"));

        let path2 = String::from("/opt/test/dev/mrt");
        let result2 = format_path_with_homedir(&path2, Some(PathBuf::from("/home/test")), '/');
        let expected2 = (String::from("/opt/test/dev/"), String::from("mrt"));

        assert_eq!(result1, expected1);
        assert_eq!(result2, expected2);
    }
}
