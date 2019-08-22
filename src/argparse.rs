#[derive(Debug)]
pub struct ParsedArgs {
    pub tags: Vec<String>,
    pub before_tags: Vec<String>,
    pub after_tags: Vec<String>,
}

/// Takes in full list of arguments and returns tuple where
/// first element is tags found at start of arguments and
/// second element is the remaining arguments.
fn find_tags_in_args(args: &Vec<String>) -> ParsedArgs {
    let tag_prefix = "+";

    let empty = ParsedArgs {
        tags: Vec::new(),
        before_tags: Vec::new(),
        after_tags: Vec::new(),
    };

    args[1..].into_iter().fold(empty, |mut acc, arg| {
        match arg {
            a if arg.starts_with(tag_prefix) && acc.after_tags.is_empty() => acc.tags.push(a.clone()),
            a if !acc.tags.is_empty() => acc.after_tags.push(a.clone()),
            a => acc.before_tags.push(a.clone()),
        };
        acc
    })
}

/// Returns tuple with results.
/// First element is tags found at start of arguments.
/// Second element is the remaining arguments.
pub fn parse_arguments() -> ParsedArgs {
    let args = std::env::args();
    let args_vec = args.collect();
    find_tags_in_args(&args_vec)
}
