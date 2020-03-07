use crate::argparse::{parse_arguments, ParsedArgs};
use crate::config::models::ConfigFile;
use crate::subcommands::subcommand::get_subcommands;
use crate::subcommands::subcommand::MrtSubcommand;
use clap::{AppSettings, ArgMatches, SubCommand};

pub fn get() -> MrtSubcommand {
    MrtSubcommand {
        name: String::from("completion"),
        run_subcommand: completion,
        doc: SubCommand::with_name("completion")
            .about("Subcommand to handle autocompletion")
            .setting(AppSettings::AllowExternalSubcommands),
        // TODO: Subcommand for different shells AT LEAST ZSH
        // TODO: Autocomplete for subcommands
    }
}

fn completion(args: &ArgMatches, _parsed_args: &ParsedArgs, config: ConfigFile) {
    let subcmds: Vec<MrtSubcommand> = get_subcommands();
    let args = std::env::args();
    let args_vec = args.filter(|x| x != "completion").collect::<Vec<String>>(); // TODO: Only filter first?
    let parsed = parse_arguments(args_vec, &subcmds);
    let subcmd_names = subcmds
        .iter()
        .map(|x| x.name.clone())
        .collect::<Vec<String>>();

    let subcommand_exists = subcmd_names
        .iter()
        .map(|n| parsed.before_tags.contains(n))
        .any(|e| e);

    if parsed.after_tags.is_empty() && !subcommand_exists {
        println!("{}", subcmd_names.join(" "));
    }

    if parsed.after_tags.is_empty() {
        let tags = config
            .tags
            .keys()
            .map(|t| format!("+{}", t))
            .filter(|twp| !parsed.tags.contains(twp))
            .collect::<Vec<String>>();
        println!("{}", tags.join(" "));
    }
}

fn get_bash() -> String {
    String::from(
        r#"
_mrt_complete() {
  COMPREPLY=()
  local word="${COMP_WORDS[COMP_CWORD]}"

  local command="${COMP_WORDS[@]:1:${#COMP_WORDS[@]}-2}"
  local completions="$(mrt completion $command)"
  COMPREPLY=( $(compgen -W "$completions" -- "$word") )
}

complete -F _mrt_complete mrt
    "#,
    )
}
