use colored::Colorize;
use std::path::PathBuf;

fn format_path_with_homedir(
    path: &PathBuf,
    home_dir: Option<PathBuf>,
    sep: char,
) -> (String, String) {
    let base_name = path.file_name().map(|x| x.to_str().unwrap_or(""));
    let dir_name = path.parent().map(|x| x.to_str().unwrap_or(""));

    let dir_to_use = match home_dir {
        Some(home) => dir_name.map(|dn| dn.replace(home.to_str().unwrap_or(""), "~")),
        _ => dir_name.map(|x| x.to_string()),
    };

    let sep_to_use = match &dir_to_use {
        Some(d) if d.ends_with('/') => String::default(),
        _ => sep.to_string(),
    };

    let prefix = format!("{}{}", dir_to_use.unwrap_or_default(), sep_to_use);

    (prefix, base_name.unwrap_or("").to_string())
}

pub fn format_path(path: &PathBuf) -> String {
    let (prefix, basename) = split_on_basename(path);
    format!("{}{}", prefix.dimmed(), basename.normal())
}

pub fn split_on_basename(path: &PathBuf) -> (String, String) {
    let home_dir = dirs::home_dir();
    format_path_with_homedir(path, home_dir, std::path::MAIN_SEPARATOR)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_unix_formatting() {
        let path1 = PathBuf::from("/home/test/dev/mrt");
        let result1 = format_path_with_homedir(&path1, Some(PathBuf::from("/home/test")), '/');
        let expected1 = (String::from("~/dev/"), String::from("mrt"));

        let path2 = PathBuf::from("/opt/test/dev/mrt");
        let result2 = format_path_with_homedir(&path2, Some(PathBuf::from("/home/test")), '/');
        let expected2 = (String::from("/opt/test/dev/"), String::from("mrt"));

        assert_eq!(result1, expected1);
        assert_eq!(result2, expected2);
    }

    #[test]
    fn test_formatting_on_root() {
        let path1 = PathBuf::from("/home");
        let result1 = format_path_with_homedir(&path1, Some(PathBuf::from("/home/test")), '/');
        let expected1 = (String::from("/"), String::from("home"));

        assert_eq!(result1, expected1);
    }
}
