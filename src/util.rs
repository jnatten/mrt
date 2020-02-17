use colored::Colorize;
use std::path::Path;

pub fn format_path(path: &String) -> String {
    let as_path = Path::new(path);
    let base_name = as_path
        .file_name()
        .map(|x| x.to_str().unwrap_or(""))
        .unwrap_or("");
    let dir_name = as_path
        .parent()
        .map(|x| x.to_str().unwrap_or(""))
        .unwrap_or("");

    let home_dir = dirs::home_dir();
    let dir_to_use = match home_dir {
        Some(home) => dir_name.replace(home.to_str().unwrap_or(""), "~"),
        _ => dir_name.to_string(),
    };

    let separator = "/";
    let prefix = format!("{}{}", dir_to_use, separator);
    format!("{}{}", prefix.dimmed(), base_name.clear())
}
