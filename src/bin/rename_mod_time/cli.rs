use clap::{Arg, ArgAction, Command, crate_version};

const PROGRAM_NAME: &'static str = "rename_mod_time";

const SHORT_DESCRIPTION: &'static str =
    "Rename files with their own modification date and time\nin a specific format.";

const FORMAT_HELP_MESSAGE: &'static str = r#"The format of date and time following Rust chrono's format:
https://docs.rs/chrono/latest/chrono/format/strftime/index.html
"#;

const DEFAULT_TIME_FORMAT: &'static str = "%y-%m-%d_%H-%M-%S";

pub fn get_cli_parser() -> Command {
    Command::new(PROGRAM_NAME)
        .version(crate_version!())
        .about(SHORT_DESCRIPTION)
        .next_line_help(true)
        .arg_required_else_help(true)
        .arg(
            Arg::new("format")
                .short('f')
                .help(FORMAT_HELP_MESSAGE)
                .default_value(DEFAULT_TIME_FORMAT)
                .required(false),
        )
        .arg(
            Arg::new("input_paths")
                .help("The path(s) to the input file(s)")
                .required(true)
                .action(ArgAction::Append),
        )
}