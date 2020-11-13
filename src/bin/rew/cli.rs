use clap::{crate_name, crate_version, AppSettings, Clap};
use common::color::{parse_color, COLOR_VALUES};
use common::run::Options;
use termcolor::ColorChoice;

#[derive(Debug, Clap)]
#[clap(
    name = crate_name!(),
    version = crate_version!(),
    after_help = "Use `-h` for short descriptions and `--help` for more details.",
    setting(AppSettings::ColoredHelp),
    setting(AppSettings::DeriveDisplayOrder),
    setting(AppSettings::UnifiedHelpMessage),
    setting(AppSettings::DontCollapseArgsInUsage),
)]
/// Rewrite FS paths according to a pattern
pub struct Cli {
    /// Output pattern
    ///
    /// If not provided, input values are directly written to stdout.
    ///
    /// Use `--explain` flag to print explanation of a given pattern.
    /// Use `--help-pattern` flag to print description of patter syntax.
    /// Use `--help-filters` flag to print filter reference.
    #[clap(verbatim_doc_comment)]
    pub pattern: Option<String>,

    /// Input values (read from stdin by default)
    #[clap(value_name = "value")]
    pub values: Vec<String>,

    /// Read values delimited by a specific character, not newline
    #[clap(
    short = 'd',
    long,
    value_name = "char",
    conflicts_with_all = &["read-nul", "read-raw"],
    parse(try_from_str = parse_single_byte_char)
    )]
    pub read: Option<u8>,

    /// Read values delimited by NUL, not newline
    #[clap(short = 'z', long, conflicts_with_all = &["read-raw", "read"])]
    pub read_nul: bool,

    /// Read the whole input into memory as a single value
    #[clap(short = 'r', long, conflicts_with_all = &["read-nul", "read"])]
    pub read_raw: bool,

    /// Print results delimited by NUL, not newline
    #[clap(short = 'Z', long, conflicts_with = "print-raw")]
    pub print_nul: bool,

    /// Print results without any delimiter
    #[clap(short = 'R', long, conflicts_with = "print-nul")]
    pub print_raw: bool,

    /// Print machine-readable transformations as a results
    ///
    /// Such output can be processed by accompanying `mvb` and `cpb` utilities to perform bulk move/copy of files and directories.
    #[clap(short = 'b', long, conflicts_with = "pretty")]
    pub bulk: bool,

    /// Print human-readable transformations as a results
    #[clap(short = 'p', long, conflicts_with = "bulk")]
    pub pretty: bool,

    /// When to use colors
    #[clap(
    long,
    value_name = "when",
    possible_values = COLOR_VALUES,
    parse(try_from_str = parse_color),
    )]
    pub color: Option<ColorChoice>,

    /// Continue processing after an error, fail at end
    #[clap(short = 'c', long)]
    pub fail_at_end: bool,

    /// Global counter initial value
    #[clap(long, value_name = "number")]
    pub gc_init: Option<u32>,

    /// Global counter step
    #[clap(long, value_name = "number")]
    pub gc_step: Option<u32>,

    /// Local counter initial value
    #[clap(long, value_name = "number")]
    pub lc_init: Option<u32>,

    /// Local counter step
    #[clap(long, value_name = "number")]
    pub lc_step: Option<u32>,

    /// Custom escape character to use in pattern
    #[clap(long, value_name = "char")]
    pub escape: Option<char>,

    /// Print explanation of a given pattern
    #[clap(long, requires = "pattern")]
    pub explain: bool,

    /// Print help information
    #[clap(short = 'h', long)]
    pub help: bool,

    /// Print description of pattern syntax
    #[clap(long)]
    pub help_pattern: bool,

    /// Print filter reference
    #[clap(long)]
    pub help_filters: bool,

    /// Print version information
    #[clap(long)]
    pub version: bool,
}

impl Options for Cli {
    fn color(&self) -> Option<ColorChoice> {
        self.color
    }
}

pub fn parse_single_byte_char(string: &str) -> Result<u8, &'static str> {
    if string.chars().count() != 1 {
        Err("value must be a single character")
    } else if string.len() != 1 {
        Err("multi-byte characters are not supported")
    } else {
        Ok(string.as_bytes()[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init() {
        assert!(Cli::try_parse_from(&["rew", "pattern"]).is_ok());
    }

    #[test]
    fn color() {
        let cli = Cli::try_parse_from(&["rew", "pattern", "--color=always"]).unwrap();
        assert_eq!(Options::color(&cli), Some(ColorChoice::Always));
    }

    #[test]
    fn parses_single_byte_char() {
        assert_eq!(parse_single_byte_char("a"), Ok(b'a'));
        assert_eq!(
            parse_single_byte_char("á"),
            Err("multi-byte characters are not supported",)
        );
        assert_eq!(
            parse_single_byte_char("aa"),
            Err("value must be a single character")
        );
    }
}
