use super::super::argparse::ParsedArgs;

pub fn status(parsed_args: &ParsedArgs) {
    println!("These are your tags {:#?}", parsed_args.tags);
}
