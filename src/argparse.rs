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

    // Only collect tags if they appear before other arguments
    let mut empty = ParsedArgs {
        tags: Vec::new(),
        before_tags: Vec::new(),
        after_tags: Vec::new(),
    };


    let mut has_seen_a_tag = false;
    let mut last_tag_seen = false;

    let it = args[1..].into_iter();

    for arg in it {
        if arg.starts_with(tag_prefix) && !last_tag_seen {
            has_seen_a_tag = true;
            empty.tags.push(arg.clone());
        } else if has_seen_a_tag {
            last_tag_seen = true;
            empty.after_tags.push(arg.clone());
        } else {
            empty.before_tags.push(arg.clone());
        }
    }

    empty
}

/// Returns tuple with results.
/// First element is tags found at start of arguments.
/// Second element is the remaining arguments.
pub fn parse_arguments() -> ParsedArgs {
    let args = std::env::args();
    let args_vec = args.collect();
    find_tags_in_args(&args_vec)
}
