

pub struct ParsedArgs {
    tags: Vec<String>,
    before_tags: Vec<String>,
    after_tags: Vec<String>,
}

/// Takes in full list of arguments and returns tuple where
/// first element is tags found at start of arguments and
/// second element is the remaining arguments.
fn find_tags_in_args(args: &Vec<String>) -> (Vec<String>, Vec<String>) {
    let tag_prefix = "+";
    let iter = args[1..].into_iter();

    // Only collect tags if they appear before other arguments
    let (_, tags, remaining_args) = iter.fold((true, Vec::new(), Vec::new()), |(keep_collecting, mut tags, mut remaining_args), arg| {
        if keep_collecting && arg.starts_with(tag_prefix) {
            tags.push(arg.clone());
            (true, tags, remaining_args)
        } else {
            // Found something other than a tag, stop collecting them.
            remaining_args.push(arg.clone());
            (false, tags, remaining_args)
        }
    });
    (tags, remaining_args)
}

/// Returns tuple with results.
/// First element is tags found at start of arguments.
/// Second element is the remaining arguments.
pub fn parse_arguments() -> ParsedArgs {
    let args = std::env::args();
    let args_vec = args.collect();
    find_tags_in_args(&args_vec)
}
