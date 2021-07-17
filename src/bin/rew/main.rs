use std::env;
use std::io::Write;

use ::regex::Regex;
use common::help::highlight;
use common::input::Terminator;
use common::run::{exec_run, Io, Result, EXIT_CODE_OK};

use crate::cli::Cli;
use crate::output::write_pattern_error;
use crate::pattern::parse::Separator;
use crate::pattern::regex::RegexHolder;
use crate::pattern::{eval, help, parse, Pattern};

mod cli;
mod counter;
mod input;
mod output;
mod pattern;
mod regex;

const EXIT_CODE_PARSE_ERROR: i32 = 3;
const EXIT_CODE_EVAL_ERROR: i32 = 4;

fn main() {
    exec_run(run);
}

fn run(cli: &Cli, io: &Io) -> Result {
    if cli.help_pattern {
        highlight(&mut io.stdout(), help::PATTERN)?;
        return Ok(EXIT_CODE_OK);
    }

    if cli.help_filters {
        highlight(&mut io.stdout(), help::FILTERS)?;
        return Ok(EXIT_CODE_OK);
    }

    let mut input_values = if cli.values.is_empty() && !cli.no_stdin {
        let required = cli.read_end;
        let terminator = if let Some(value) = cli.read {
            Terminator::Byte { value, required }
        } else if cli.read_nul {
            Terminator::Byte { value: 0, required }
        } else if cli.read_raw {
            Terminator::None
        } else {
            Terminator::Newline { required }
        };
        input::Values::from_stdin(io.stdin(), terminator)
    } else {
        input::Values::from_args(cli.values.as_slice())
    };

    let output_mode = if cli.pretty {
        output::Mode::Pretty
    } else if cli.diff {
        output::Mode::Diff
    } else if cli.json_lines {
        output::Mode::JsonLines
    } else if cli.no_print_end {
        output::Mode::StandardNoEnd
    } else {
        output::Mode::Standard
    };

    let output_terminator = if let Some(terminator) = &cli.print {
        terminator
    } else if cli.print_raw {
        ""
    } else if cli.print_nul {
        "\0"
    } else {
        "\n"
    };

    let mut output_values = output::Values::new(io.stdout(), output_mode, output_terminator);
    let mut exit_code = EXIT_CODE_OK;

    if let Some(raw_pattern) = cli.pattern.as_ref() {
        let separator = if let Some(separator) = &cli.separator {
            Separator::String(separator.clone())
        } else if let Some(separator) = &cli.separator_regex {
            Separator::Regex(RegexHolder(separator.clone()))
        } else {
            Separator::Regex(RegexHolder(
                Regex::new("\\s+").expect("Failed to create default separator from regex"),
            ))
        };

        let parse_config = parse::Config {
            escape: cli.escape.unwrap_or('%'),
            separator,
        };

        let pattern = match Pattern::parse(raw_pattern, &parse_config) {
            Ok(pattern) => pattern,
            Err(error) => {
                let mut stderr = io.stderr();
                write_pattern_error(&mut stderr, &error, raw_pattern)?;

                if let Some(hint) = error.kind.hint() {
                    writeln!(stderr)?;
                    let message = match hint {
                        parse::ErrorHint::RegexSyntax => help::REGEX_HINT,
                        parse::ErrorHint::PatternSyntax => help::PATTERN_HINT,
                        parse::ErrorHint::FilterUsage => help::FILTERS_HINT,
                    };
                    highlight(&mut stderr, message)?;
                }

                return Ok(EXIT_CODE_PARSE_ERROR);
            }
        };

        if cli.explain || cli.explain_filters {
            pattern.explain(&mut io.stdout(), cli.explain)?;
            return Ok(EXIT_CODE_OK);
        }

        let global_counter_used = pattern.uses_global_counter();
        let local_counter_used = pattern.uses_local_counter();
        let regex_capture_used = pattern.uses_regex_capture();

        let global_counter_config = cli.global_counter.unwrap_or_else(counter::Config::default);
        let local_counter_config = cli.local_counter.unwrap_or_else(counter::Config::default);

        let mut global_counter_generator = counter::GlobalGenerator::from(&global_counter_config);
        let mut local_counter_generator = counter::LocalGenerator::from(&local_counter_config);

        let regex_solver = if let Some(regex) = &cli.regex {
            regex::Solver::Value(regex)
        } else if let Some(regex) = &cli.regex_filename {
            regex::Solver::FileName(regex)
        } else {
            regex::Solver::None
        };

        let working_dir = if let Some(working_dir) = &cli.working_directory {
            if working_dir.is_relative() {
                env::current_dir()?.join(working_dir)
            } else {
                working_dir.clone()
            }
        } else {
            env::current_dir()?
        };

        let expression_quotes = match cli.quote {
            0 => None,
            1 => Some('\''),
            _ => Some('"'),
        };

        while let Some(input_value) = input_values.next()? {
            let global_counter = if global_counter_used {
                global_counter_generator.next()
            } else {
                0
            };

            let local_counter = if local_counter_used {
                local_counter_generator.next(input_value)
            } else {
                0
            };

            let regex_captures = if regex_capture_used {
                regex_solver.eval(input_value)
            } else {
                None
            };

            let context = eval::Context {
                working_dir: &working_dir,
                global_counter,
                local_counter,
                regex_captures,
                expression_quotes,
            };

            let output_value = match pattern.eval(input_value, &context) {
                Ok(value) => value,
                Err(error) => {
                    write_pattern_error(&mut io.stderr(), &error, raw_pattern)?;
                    if cli.fail_at_end {
                        exit_code = EXIT_CODE_EVAL_ERROR;
                        continue;
                    } else {
                        return Ok(EXIT_CODE_EVAL_ERROR);
                    }
                }
            };

            output_values.write(input_value, &output_value)?;
        }
    } else {
        while let Some(value) = input_values.next()? {
            output_values.write(value, value)?;
        }
    };

    io.stdout().flush()?; // output::Values may not do flush if there is no last terminator.
    Ok(exit_code)
}
