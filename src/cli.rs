use regex::Regex;
use std::path::PathBuf;
use structopt::{clap::AppSettings, StructOpt};
use termcolor::ColorChoice;

const COLOR_ALWAYS: &str = "always";
const COLOR_ANSI: &str = "ansi";
const COLOR_AUTO: &str = "auto";
const COLOR_NEVER: &str = "never";

#[derive(Debug, StructOpt)]
#[structopt(
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder)
)]
pub struct Cli {
    /// Output pattern
    pub pattern: String,

    /// Paths to process (read from stdin by default)
    #[structopt(value_name = "path")]
    pub paths: Vec<PathBuf>,

    /// Read paths delimited by NUL, not newline
    #[structopt(short = "z", long = "read-0")]
    pub read_nul: bool,

    /// Print paths delimited by NUL, not newline
    #[structopt(short = "Z", long = "print-0")]
    pub print_nul: bool,

    /// Print paths without any delimiter
    #[structopt(short = "R", long = "print-raw", conflicts_with = "print-nul")]
    pub print_raw: bool,

    /// Regular expression matched against filename
    #[structopt(short = "e", long)]
    pub regex: Option<Regex>,

    /// Regular expression matched against full path
    #[structopt(short = "E", long, value_name = "regex")]
    pub regex_full: Option<Regex>,

    /// Custom escape character to use in pattern
    #[structopt(long, value_name = "char")]
    pub escape: Option<char>,

    /// When to use colors
    #[structopt(
        long,
        value_name = "when",
        possible_values = &[COLOR_AUTO, COLOR_ALWAYS, COLOR_NEVER, COLOR_ANSI],
        parse(try_from_str = parse_color),
    )]
    pub color: Option<ColorChoice>,
}

fn parse_color(string: &str) -> Result<ColorChoice, &'static str> {
    match string {
        COLOR_ALWAYS => Ok(ColorChoice::Always),
        COLOR_ANSI => Ok(ColorChoice::AlwaysAnsi),
        COLOR_AUTO => Ok(ColorChoice::Auto),
        COLOR_NEVER => Ok(ColorChoice::Never),
        _ => Err("invalid value"),
    }
}
