use std::fmt;

use unidecode::unidecode;

use crate::pattern::char::{AsChar, Char};
use crate::pattern::field::Field;
use crate::pattern::integer::parse_integer;
use crate::pattern::number::NumberRange;
use crate::pattern::padding::Padding;
use crate::pattern::reader::Reader;
use crate::pattern::regex::RegexRange;
use crate::pattern::repeat::Repetition;
use crate::pattern::replace::{EmptySubstitution, RegexSubstitution, StringSubstitution};
use crate::pattern::substr::CharIndexRange;
use crate::pattern::switch::RegexSwitch;
use crate::pattern::symbols::REVERSE_INDEX;
use crate::pattern::uuid::random_uuid;
use crate::pattern::{eval, parse, path};

#[derive(Debug, PartialEq)]
pub enum Filter {
    WorkingDir,
    AbsolutePath,
    RelativePath,
    NormalizedPath,
    CanonicalPath,
    ParentDirectory,
    RemoveLastName,
    FileName,
    LastName,
    BaseName,
    RemoveExtension,
    Extension,
    ExtensionWithDot,
    EnsureTrailDirSeparator,
    RemoveTrailDirSeparator,
    Substring(CharIndexRange),
    SubstringRev(CharIndexRange),
    GetField(Field),
    GetFieldRev(Field),
    ReplaceFirst(StringSubstitution),
    ReplaceAll(StringSubstitution),
    ReplaceEmpty(EmptySubstitution),
    RegexMatch(RegexRange),
    RegexMatchRev(RegexRange),
    RegexReplaceFirst(RegexSubstitution),
    RegexReplaceAll(RegexSubstitution),
    RegexSwitch(RegexSwitch),
    RegexCapture(usize),
    Trim,
    ToLowercase,
    ToUppercase,
    ToAscii,
    RemoveNonAscii,
    LeftPad(Padding),
    RightPad(Padding),
    Repeat(Repetition),
    LocalCounter,
    GlobalCounter,
    RandomNumber(NumberRange),
    RandomUuid,
}

impl Filter {
    pub fn parse(reader: &mut Reader<Char>, config: &parse::Config) -> parse::Result<Self> {
        let position = reader.position();

        if let Some(char) = reader.read() {
            match char.as_char() {
                'w' => Ok(Self::WorkingDir),
                'a' => Ok(Self::AbsolutePath),
                'A' => Ok(Self::RelativePath),
                'p' => Ok(Self::NormalizedPath),
                'P' => Ok(Self::CanonicalPath),
                'd' => Ok(Self::ParentDirectory),
                'D' => Ok(Self::RemoveLastName),
                'f' => Ok(Self::FileName),
                'F' => Ok(Self::LastName),
                'b' => Ok(Self::BaseName),
                'B' => Ok(Self::RemoveExtension),
                'e' => Ok(Self::Extension),
                'E' => Ok(Self::ExtensionWithDot),
                'z' => Ok(Self::EnsureTrailDirSeparator),
                'Z' => Ok(Self::RemoveTrailDirSeparator),
                '#' => {
                    if reader.read_expected(REVERSE_INDEX) {
                        Ok(Self::SubstringRev(CharIndexRange::parse(reader)?))
                    } else {
                        Ok(Self::Substring(CharIndexRange::parse(reader)?))
                    }
                }
                '&' => {
                    let separator = &config.separator;
                    if reader.read_expected(REVERSE_INDEX) {
                        Ok(Self::GetFieldRev(Field::parse(reader, separator)?))
                    } else {
                        Ok(Self::GetField(Field::parse(reader, separator)?))
                    }
                }
                'r' => Ok(Self::ReplaceFirst(StringSubstitution::parse(reader)?)),
                'R' => Ok(Self::ReplaceAll(StringSubstitution::parse(reader)?)),
                '?' => Ok(Self::ReplaceEmpty(EmptySubstitution::parse(reader)?)),
                '=' => {
                    if reader.read_expected(REVERSE_INDEX) {
                        Ok(Self::RegexMatchRev(RegexRange::parse(reader)?))
                    } else {
                        Ok(Self::RegexMatch(RegexRange::parse(reader)?))
                    }
                }
                's' => Ok(Self::RegexReplaceFirst(RegexSubstitution::parse(reader)?)),
                'S' => Ok(Self::RegexReplaceAll(RegexSubstitution::parse(reader)?)),
                '@' => Ok(Self::RegexSwitch(RegexSwitch::parse(reader)?)),
                '$' => Ok(Self::RegexCapture(parse_integer(reader)?)),
                't' => Ok(Self::Trim),
                'v' => Ok(Self::ToLowercase),
                '^' => Ok(Self::ToUppercase),
                'i' => Ok(Self::ToAscii),
                'I' => Ok(Self::RemoveNonAscii),
                '<' => Ok(Self::LeftPad(Padding::parse(reader, '<')?)),
                '>' => Ok(Self::RightPad(Padding::parse(reader, '>')?)),
                '*' => Ok(Self::Repeat(Repetition::parse(reader)?)),
                'c' => Ok(Self::LocalCounter),
                'C' => Ok(Self::GlobalCounter),
                'u' => Ok(Self::RandomNumber(NumberRange::parse(reader)?)),
                'U' => Ok(Self::RandomUuid),
                _ => Err(parse::Error {
                    kind: parse::ErrorKind::UnknownFilter(char.clone()),
                    range: position..reader.position(),
                }),
            }
        } else {
            Err(parse::Error {
                kind: parse::ErrorKind::ExpectedFilter,
                range: position..reader.end(),
            })
        }
    }

    pub fn eval(&self, mut value: String, context: &eval::Context) -> eval::BaseResult<String> {
        match self {
            Self::WorkingDir => path::to_string(context.working_dir),
            Self::AbsolutePath => path::to_absolute(value, context.working_dir),
            Self::RelativePath => path::to_relative(value, context.working_dir),
            Self::NormalizedPath => path::normalize(&value),
            Self::CanonicalPath => path::canonicalize(value, context.working_dir),
            Self::ParentDirectory => path::get_parent_directory(value),
            Self::RemoveLastName => path::remove_last_name(value),
            Self::FileName => path::get_file_name(&value),
            Self::LastName => path::get_last_name(&value),
            Self::BaseName => path::get_base_name(&value),
            Self::RemoveExtension => path::remove_extension(value),
            Self::Extension => path::get_extension(&value),
            Self::ExtensionWithDot => path::get_extension_with_dot(&value),
            Self::EnsureTrailDirSeparator => Ok(path::ensure_trailing_dir_separator(value)),
            Self::RemoveTrailDirSeparator => Ok(path::remove_trailing_dir_separator(value)),
            Self::Substring(range) => Ok(range.substr(value)),
            Self::SubstringRev(range) => Ok(range.substr_rev(value)),
            Self::GetField(field) => Ok(field.get(&value).to_string()),
            Self::GetFieldRev(field) => Ok(field.get_rev(&value).to_string()),
            Self::ReplaceFirst(substitution) => Ok(substitution.replace_first(&value)),
            Self::ReplaceAll(substitution) => Ok(substitution.replace_all(&value)),
            Self::ReplaceEmpty(substitution) => Ok(substitution.replace(value)),
            Self::RegexMatch(range) => Ok(range.find(&value)),
            Self::RegexMatchRev(range) => Ok(range.find_rev(&value)),
            Self::RegexReplaceFirst(substitution) => Ok(substitution.replace_first(&value)),
            Self::RegexReplaceAll(substitution) => Ok(substitution.replace_all(&value)),
            Self::RegexSwitch(switch) => Ok(switch.eval(&value).to_string()),
            Self::RegexCapture(number) => Ok(context.regex_capture(*number).to_string()),
            Self::Trim => Ok(value.trim().to_string()),
            Self::ToLowercase => Ok(value.to_lowercase()),
            Self::ToUppercase => Ok(value.to_uppercase()),
            Self::ToAscii => Ok(unidecode(&value)),
            Self::RemoveNonAscii => {
                value.retain(|ch| ch.is_ascii());
                Ok(value)
            }
            Self::LeftPad(padding) => Ok(padding.apply_left(value)),
            Self::RightPad(padding) => Ok(padding.apply_right(value)),
            Self::Repeat(repetition) => Ok(repetition.expand(&value)),
            Self::LocalCounter => Ok(context.local_counter.to_string()),
            Self::GlobalCounter => Ok(context.global_counter.to_string()),
            Self::RandomNumber(range) => Ok(range.random().to_string()),
            Self::RandomUuid => Ok(random_uuid()),
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WorkingDir => write!(formatter, "Working directory"),
            Self::AbsolutePath => write!(formatter, "Absolute path"),
            Self::RelativePath => write!(formatter, "Relative path"),
            Self::NormalizedPath => write!(formatter, "Normalized path"),
            Self::CanonicalPath => write!(formatter, "Canonical path"),
            Self::ParentDirectory => write!(formatter, "Parent directory"),
            Self::RemoveLastName => write!(formatter, "Remove last name"),
            Self::FileName => write!(formatter, "File name"),
            Self::LastName => write!(formatter, "Last name"),
            Self::BaseName => write!(formatter, "Base name"),
            Self::RemoveExtension => write!(formatter, "Remove extension"),
            Self::Extension => write!(formatter, "Extension"),
            Self::ExtensionWithDot => write!(formatter, "Extension with dot"),
            Self::EnsureTrailDirSeparator => {
                write!(formatter, "Ensure trailing directory separator")
            }
            Self::RemoveTrailDirSeparator => {
                write!(formatter, "Remove trailing directory separator")
            }
            Self::Substring(range) => write!(formatter, "Substring from {}", range),
            Self::SubstringRev(range) => {
                write!(formatter, "Substring from {} backward", range)
            }
            Self::GetField(field) => write!(formatter, "Get {}", field),
            Self::GetFieldRev(field) => write!(formatter, "Get {} backward", field),
            Self::ReplaceFirst(substitution) => write!(formatter, "Replace first {}", substitution),
            Self::ReplaceAll(substitution) => write!(formatter, "Replace all {}", substitution),
            Self::ReplaceEmpty(substitution) => {
                write!(formatter, "Replace {}", substitution)
            }
            Self::RegexMatch(_) => write!(formatter, "TODO"),
            Self::RegexMatchRev(_) => {
                write!(formatter, "TODO backward")
            }
            Self::RegexReplaceFirst(substitution) => write!(
                formatter,
                "Replace first match of regular expression {}",
                substitution
            ),
            Self::RegexReplaceAll(substitution) => write!(
                formatter,
                "Replace all matches of regular expression {}",
                substitution
            ),
            Self::RegexSwitch(switch) => {
                write!(formatter, "Regular expression switch with {}", switch)
            }
            Self::RegexCapture(number) => {
                write!(
                    formatter,
                    "Capture group #{} of a global regular expression",
                    number
                )
            }
            Self::Trim => write!(formatter, "Trim"),
            Self::ToLowercase => write!(formatter, "To lowercase"),
            Self::ToUppercase => write!(formatter, "To uppercase"),
            Self::ToAscii => write!(formatter, "To ASCII"),
            Self::RemoveNonAscii => write!(formatter, "Remove non-ASCII"),
            Self::LeftPad(padding) => write!(formatter, "Left pad with {}", padding),
            Self::RightPad(padding) => write!(formatter, "Right pad with {}", padding),
            Self::Repeat(repetition) => write!(formatter, "Repeat {}", repetition),
            Self::LocalCounter => write!(formatter, "Local counter"),
            Self::GlobalCounter => write!(formatter, "Global counter"),
            Self::RandomNumber(interval) => write!(formatter, "Random number from {}", interval),
            Self::RandomUuid => write!(formatter, "Random UUID"),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate regex;

    use test_case::test_case;

    use super::Filter;
    use crate::pattern::error::ErrorRange;
    use crate::pattern::field::Field;
    use crate::pattern::number::NumberRange;
    use crate::pattern::padding::Padding;
    use crate::pattern::parse::Separator;
    use crate::pattern::repeat::Repetition;
    use crate::pattern::replace::{
        EmptySubstitution, RegexSubstitution, StringSubstitution, Substitution,
    };
    use crate::pattern::substr::CharIndexRange;
    use crate::pattern::switch::{Case, RegexSwitch};
    use crate::pattern::utils::{AnyString, Empty};

    type F = Filter;

    mod parse {
        use test_case::test_case;

        use super::*;
        use crate::pattern::parse::{Config, Error, ErrorKind};
        use crate::pattern::reader::Reader;

        type E = ErrorKind;

        #[test_case("",         0..0, E::ExpectedFilter                              ; "empty")]
        #[test_case("-",        0..1, E::UnknownFilter('-'.into())                   ; "unknown")]
        #[test_case("#",        1..1, E::ExpectedRange                               ; "substring expected range")]
        #[test_case("#-",       2..2, E::ExpectedRange                               ; "substring rev expected range")]
        #[test_case("&",        1..1, E::ExpectedNumber                              ; "field expected number")]
        #[test_case("&-",       2..2, E::ExpectedNumber                              ; "field rev expected number")]
        #[test_case("&1:",      3..3, E::ExpectedFieldSeparator                      ; "field expected separator")]
        #[test_case("&-1:",     4..4, E::ExpectedFieldSeparator                      ; "field rev expected separator")]
        #[test_case("&1/[0-9",  3..7, E::RegexInvalid(AnyString::any())              ; "field regex invalid")]
        #[test_case("&-1/[0-9", 4..8, E::RegexInvalid(AnyString::any())              ; "field rev regex invalid")]
        #[test_case("r",        1..1, E::ExpectedSubstitution                        ; "replace expected substitution")]
        #[test_case("R",        1..1, E::ExpectedSubstitution                        ; "replace all expected substitution")]
        // #[test_case("=",        1..1, E::ExpectedRegex                               ; "regex match expected regex")]
        // #[test_case("=[0",      1..3, E::RegexInvalid(AnyString::any())              ; "regex match invalid regex")]
        #[test_case("s",        1..1, E::ExpectedSubstitution                        ; "regex replace expected substitution")]
        #[test_case("s/[0/",    2..4, E::RegexInvalid(AnyString::any())              ; "regex replace invalid regex")]
        #[test_case("S",        1..1, E::ExpectedSubstitution                        ; "regex replace all expected substitution")]
        #[test_case("S/[0/",    2..4, E::RegexInvalid(AnyString::any())              ; "regex replace all invalid regex")]
        #[test_case("@:[0:X:Y", 2..4, E::RegexInvalid(AnyString::any())              ; "regex switch invalid regex")]
        #[test_case("$",        1..1, E::ExpectedNumber                              ; "regex capture expected number")]
        #[test_case("<x",       1..2, E::PaddingPrefixInvalid('<', Some('x'.into())) ; "padding left prefix invalid")]
        #[test_case(">y",       1..2, E::PaddingPrefixInvalid('>', Some('y'.into())) ; "padding right prefix invalid")]
        fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
            assert_eq!(
                Filter::parse(&mut Reader::from(input), &Config::fixture()),
                Err(Error { kind, range }),
            )
        }

        #[test_case("w",            F::WorkingDir                           ; "working dir")]
        #[test_case("a",            F::AbsolutePath                         ; "absolute path")]
        #[test_case("A",            F::RelativePath                         ; "relative path")]
        #[test_case("p",            F::NormalizedPath                       ; "normalized path")]
        #[test_case("P",            F::CanonicalPath                        ; "canonical path")]
        #[test_case("d",            F::ParentDirectory                      ; "parent directory")]
        #[test_case("D",            F::RemoveLastName                       ; "remove last name")]
        #[test_case("f",            F::FileName                             ; "file name")]
        #[test_case("F",            F::LastName                             ; "last name")]
        #[test_case("b",            F::BaseName                             ; "base name")]
        #[test_case("B",            F::RemoveExtension                      ; "remove extension")]
        #[test_case("e",            F::Extension                            ; "extension")]
        #[test_case("E",            F::ExtensionWithDot                     ; "extension with dot")]
        #[test_case("z",            F::EnsureTrailDirSeparator              ; "ensure trail dir separator")]
        #[test_case("Z",            F::RemoveTrailDirSeparator              ; "remove trail dir separator")]
        #[test_case("#2",           F::Substring(index_range_at())          ; "substring at")]
        #[test_case("#2-",          F::Substring(index_range_from())        ; "substring from")]
        #[test_case("#2-3",         F::Substring(index_range_between())     ; "substring between")]
        #[test_case("#2+3",         F::Substring(index_range_length())      ; "substring length")]
        #[test_case("#-2",          F::SubstringRev(index_range_at())       ; "substring rev at")]
        #[test_case("#-2-",         F::SubstringRev(index_range_from())     ; "substring rev from")]
        #[test_case("#-2-3",        F::SubstringRev(index_range_between())  ; "substring rev between")]
        #[test_case("#-2+3",        F::SubstringRev(index_range_length())   ; "substring rev length")]
        #[test_case("&2",           F::GetField(field_default())            ; "field default separator")]
        #[test_case("&2:,",         F::GetField(field_string())             ; "field string separator")]
        #[test_case("&2/[, ]+",     F::GetField(field_regex())              ; "field regex separator")]
        #[test_case("&-2",          F::GetFieldRev(field_default())         ; "field rev default separator")]
        #[test_case("&-2:,",        F::GetFieldRev(field_string())          ; "field rev string separator")]
        #[test_case("&-2/[, ]+",    F::GetFieldRev(field_regex())           ; "field rev regex separator")]
        #[test_case("r/ab",         F::ReplaceFirst(subst_string_1())       ; "remove first")]
        #[test_case("r/ab/x",       F::ReplaceFirst(subst_string_2())       ; "replace first")]
        #[test_case("R/ab",         F::ReplaceAll(subst_string_1())         ; "remove all")]
        #[test_case("R/ab/x",       F::ReplaceAll(subst_string_2())         ; "replace all")]
        #[test_case("?x",           F::ReplaceEmpty(substitution_empty())   ; "replace empty")]
        // #[test_case("=[0-9]+",      F::RegexMatch("[0-9]+".into())          ; "regex match")]
        #[test_case("s/[0-9]+",     F::RegexReplaceFirst(subst_regex_1())   ; "regex remove first")]
        #[test_case("s/[0-9]+/x",   F::RegexReplaceFirst(subst_regex_2())   ; "regex replace first")]
        #[test_case("S/[0-9]+",     F::RegexReplaceAll(subst_regex_1())     ; "regex remove all")]
        #[test_case("S/[0-9]+/x",   F::RegexReplaceAll(subst_regex_2())     ; "regex replace all")]
        #[test_case("@:[0-9]+:X:Y", F::RegexSwitch(regex_switch())          ; "regex switch ")]
        #[test_case("$0",           F::RegexCapture(0)                      ; "regex capture 0")]
        #[test_case("$10",          F::RegexCapture(10)                     ; "regex capture 10")]
        #[test_case("t",            F::Trim                                 ; "trim")]
        #[test_case("v",            F::ToLowercase                          ; "to lowercase")]
        #[test_case("^",            F::ToUppercase                          ; "to uppercase")]
        #[test_case("i",            F::ToAscii                              ; "to ascii")]
        #[test_case("I",            F::RemoveNonAscii                       ; "remove non-ascii")]
        #[test_case("<<abcd",       F::LeftPad(padding_fixed())             ; "left pad fixed")]
        #[test_case("<2:abc",       F::LeftPad(padding_repeated())          ; "left pad repeated")]
        #[test_case(">>abcd",       F::RightPad(padding_fixed())            ; "right pad fixed")]
        #[test_case(">2:abc",       F::RightPad(padding_repeated())         ; "right pad repeated")]
        #[test_case("*2",           F::Repeat(repetition_input())           ; "repetition input ")]
        #[test_case("*2:abc",       F::Repeat(repetition_value())           ; "repetition value ")]
        #[test_case("c",            F::LocalCounter                         ; "local counter")]
        #[test_case("C",            F::GlobalCounter                        ; "global counter")]
        #[test_case("u",            F::RandomNumber(number_range_full())    ; "random number")]
        #[test_case("u2-",          F::RandomNumber(number_range_from())    ; "random number from")]
        #[test_case("u2-10",        F::RandomNumber(number_range_between()) ; "random number between")]
        #[test_case("U",            F::RandomUuid                           ; "random uuid")]
        fn ok(input: &str, filter: Filter) {
            assert_eq!(
                Filter::parse(&mut Reader::from(input), &Config::fixture()),
                Ok(filter),
            )
        }

        #[test_case("a_",    1 ; "no params")]
        #[test_case("#1-2_", 4 ; "with params")]
        #[test_case("#_",    1 ; "error params")]
        fn keep_chars_after(input: &str, position: usize) {
            let mut reader = Reader::from(input);
            Filter::parse(&mut reader, &Config::fixture()).unwrap_or(Filter::FileName);
            assert_eq!(reader.position(), position);
        }
    }

    mod eval {
        use test_case::test_case;

        use super::*;
        use crate::pattern::eval::{Context, ErrorKind};
        use crate::pattern::uuid::assert_uuid;

        #[test_case("non-existent", F::CanonicalPath, ErrorKind::CanonicalizationFailed(AnyString::any()) ; "canonicalization failed")]
        fn err(input: &str, filter: Filter, kind: ErrorKind) {
            assert_eq!(filter.eval(input.into(), &Context::fixture()), Err(kind))
        }

        #[cfg_attr(unix, test_case("",              F::WorkingDir,              "/work"            ; "working dir"))]
        #[cfg_attr(unix, test_case("b/c.d",         F::AbsolutePath,            "/work/b/c.d"      ; "absolute path"))]
        #[cfg_attr(unix, test_case("a/b/../e/./f/", F::NormalizedPath,          "a/e/f"            ; "normalized path"))]
        #[cfg_attr(unix, test_case("/b/c.d",        F::RelativePath,            "../b/c.d"         ; "relative path"))]
        #[cfg_attr(unix, test_case("./Cargo.toml",  F::CanonicalPath,           "/work/Cargo.toml" ; "canonical path"))]
        #[cfg_attr(unix, test_case("a/b",           F::EnsureTrailDirSeparator, "a/b/"             ; "ensure trail dir separator"))]
        #[cfg_attr(windows, test_case("",                    F::WorkingDir,              "C:\\work"             ; "working dir"))]
        #[cfg_attr(windows, test_case("b\\c.d",              F::AbsolutePath,            "C:\\work\\b\\c.d"     ; "absolute path"))]
        #[cfg_attr(windows, test_case("C:\\b\\c.d",          F::RelativePath,            "..\\b\\c.d"           ; "relative path"))]
        #[cfg_attr(windows, test_case("a\\b\\..\\e\\.\\f\\", F::NormalizedPath,          "a\\e\\f"              ; "normalized path"))]
        #[cfg_attr(windows, test_case("./Cargo.toml",        F::CanonicalPath,           "C:\\work\\Cargo.toml" ; "canonical path"))]
        #[cfg_attr(windows, test_case("a\\b",                F::EnsureTrailDirSeparator, "a\\b\\"               ; "ensure trail dir separator"))]
        #[test_case("a/b/c.d",       F::ParentDirectory,                     "a/b"      ; "parent directory")]
        #[test_case("a/b/c.d",       F::RemoveLastName,                      "a/b"      ; "remove last name")]
        #[test_case("a/b/c.d",       F::FileName,                            "c.d"      ; "file name")]
        #[test_case("a/b/c.d",       F::LastName,                            "c.d"      ; "last name")]
        #[test_case("a/b/c.d",       F::BaseName,                            "c"        ; "base name")]
        #[test_case("a/b/c.d",       F::RemoveExtension,                     "a/b/c"    ; "remove extension")]
        #[test_case("a/b/c.d",       F::Extension,                           "d"        ; "extension")]
        #[test_case("a/b/c.d",       F::ExtensionWithDot,                    ".d"       ; "extension with dot")]
        #[test_case("a/b/",          F::RemoveTrailDirSeparator,             "a/b"      ; "remove trail dir separator")]
        #[test_case("abcde",         F::Substring(index_range_at()),         "b"        ; "substring at")]
        #[test_case("abcde",         F::Substring(index_range_from()),       "bcde"     ; "substring from")]
        #[test_case("abcde",         F::Substring(index_range_between()),    "bc"       ; "substring between")]
        #[test_case("abcde",         F::Substring(index_range_length()),     "bcd"      ; "substring length")]
        #[test_case("abcde",         F::SubstringRev(index_range_at()),      "d"        ; "substring rev at")]
        #[test_case("abcde",         F::SubstringRev(index_range_from()),    "abcd"     ; "substring rev from")]
        #[test_case("abcde",         F::SubstringRev(index_range_between()), "cd"       ; "substring rev between")]
        #[test_case("abcde",         F::SubstringRev(index_range_length()),  "bcd"      ; "substring rev length")]
        #[test_case("a , b , c , d", F::GetField(field_string()),            " b "      ; "field string separator")]
        #[test_case("a , b , c , d", F::GetField(field_regex()),             "b"        ; "field regex separator")]
        #[test_case("a , b , c , d", F::GetFieldRev(field_string()),         " c "      ; "field rev string separator")]
        #[test_case("a , b , c , d", F::GetFieldRev(field_regex()),          "c"        ; "field rev regex separator")]
        #[test_case("abcd_abcd",     F::ReplaceFirst(subst_string_1()),      "cd_abcd"  ; "remove first")]
        #[test_case("abcd_abcd",     F::ReplaceFirst(subst_string_2()),      "xcd_abcd" ; "replace first")]
        #[test_case("abcd_abcd",     F::ReplaceAll(subst_string_1()),        "cd_cd"    ; "remove all")]
        #[test_case("abcd_abcd",     F::ReplaceAll(subst_string_2()),        "xcd_xcd"  ; "replace all")]
        #[test_case("",              F::ReplaceEmpty(substitution_empty()),  "x"        ; "replace empty")]
        // #[test_case("a123y",         F::RegexMatch("[0-9]+".into()),         "123"      ; "regex match")]
        #[test_case("12_34",         F::RegexReplaceFirst(subst_regex_1()),  "_34"      ; "regex remove first")]
        #[test_case("12_34",         F::RegexReplaceFirst(subst_regex_2()),  "x_34"     ; "regex replace first")]
        #[test_case("12_34",         F::RegexReplaceAll(subst_regex_1()),    "_"        ; "regex remove all")]
        #[test_case("12_34",         F::RegexReplaceAll(subst_regex_2()),    "x_x"      ; "regex replace all")]
        #[test_case("1",             F::RegexSwitch(regex_switch()),         "X"        ; "regex switch case")]
        #[test_case("a",             F::RegexSwitch(regex_switch()),         "Y"        ; "regex switch default")]
        #[test_case("",              F::RegexCapture(1),                     "a"        ; "regex capture")]
        #[test_case(" abcd ",        F::Trim,                                "abcd"     ; "trim")]
        #[test_case("ábčdÁBČD",      F::ToLowercase,                         "ábčdábčd" ; "to lowercase")]
        #[test_case("ábčdÁBČD",      F::ToUppercase,                         "ÁBČDÁBČD" ; "to uppercase")]
        #[test_case("ábčdÁBČD",      F::ToAscii,                             "abcdABCD" ; "to ascii")]
        #[test_case("ábčdÁBČD",      F::RemoveNonAscii,                      "bdBD"     ; "remove non-ascii")]
        #[test_case("01",            F::LeftPad(padding_fixed()),            "ab01"     ; "left pad fixed")]
        #[test_case("01",            F::LeftPad(padding_repeated()),         "abca01"   ; "left pad repeated")]
        #[test_case("01",            F::RightPad(padding_fixed()),           "01cd"     ; "right pad fixed")]
        #[test_case("01",            F::RightPad(padding_repeated()),        "01cabc"   ; "right pad repeated")]
        #[test_case("01",            F::Repeat(repetition_input()),          "0101"     ; "repetition input ")]
        #[test_case("01",            F::Repeat(repetition_value()),          "abcabc"   ; "repetition value ")]
        #[test_case("",              F::LocalCounter,                        "1"        ; "local counter")]
        #[test_case("",              F::GlobalCounter,                       "2"        ; "global counter")]
        #[test_case("",              F::RandomNumber(number_range_zero()),   "0"        ; "random number")]
        #[test_case("",              F::RandomUuid,                          ""         ; "random uuid")]
        fn ok(input: &str, filter: Filter, output: &str) {
            match filter {
                Filter::CanonicalPath => {
                    let real_working_dir = std::env::current_dir().unwrap(); // Canonical path filter actually checks existence of the directory
                    let mut context = Context::fixture();
                    let output = output.replace(
                        context.working_dir.to_str().unwrap(),
                        real_working_dir.to_str().unwrap(),
                    );
                    context.working_dir = &real_working_dir;
                    assert_eq!(filter.eval(input.into(), &context), Ok(output))
                }
                Filter::RandomUuid => {
                    assert_uuid(&filter.eval(input.into(), &Context::fixture()).unwrap());
                }
                _ => {
                    assert_eq!(
                        filter.eval(input.into(), &Context::fixture()),
                        Ok(output.into())
                    )
                }
            }
        }
    }

    #[test_case(F::WorkingDir,                          "Working directory"                   ; "working directory")]
    #[test_case(F::AbsolutePath,                        "Absolute path"                       ; "absolute path")]
    #[test_case(F::RelativePath,                        "Relative path"                       ; "relative path")]
    #[test_case(F::NormalizedPath,                      "Normalized path"                     ; "normalized path")]
    #[test_case(F::CanonicalPath,                       "Canonical path"                      ; "canonical path")]
    #[test_case(F::ParentDirectory,                     "Parent directory"                    ; "parent directory")]
    #[test_case(F::RemoveLastName,                      "Remove last name"                    ; "remove last name")]
    #[test_case(F::FileName,                            "File name"                           ; "file name")]
    #[test_case(F::LastName,                            "Last name"                           ; "last name")]
    #[test_case(F::BaseName,                            "Base name"                           ; "base name")]
    #[test_case(F::RemoveExtension,                     "Remove extension"                    ; "remove extension")]
    #[test_case(F::Extension,                           "Extension"                           ; "extension")]
    #[test_case(F::ExtensionWithDot,                    "Extension with dot"                  ; "extension with dot")]
    #[test_case(F::EnsureTrailDirSeparator,             "Ensure trailing directory separator" ; "ensure trail dir separator")]
    #[test_case(F::RemoveTrailDirSeparator,             "Remove trailing directory separator" ; "remove trail dir separator")]
    #[test_case(F::Substring(index_range_at()),         "Substring from 2..2"                 ; "substring at")]
    #[test_case(F::Substring(index_range_from()),       "Substring from 2.."                  ; "substring from")]
    #[test_case(F::Substring(index_range_between()),    "Substring from 2..3"                 ; "substring between")]
    #[test_case(F::Substring(index_range_length()),     "Substring from 2..4"                 ; "substring length")]
    #[test_case(F::SubstringRev(index_range_at()),      "Substring from 2..2 backward"        ; "substring rev at")]
    #[test_case(F::SubstringRev(index_range_from()),    "Substring from 2.. backward"         ; "substring rev from")]
    #[test_case(F::SubstringRev(index_range_between()), "Substring from 2..3 backward"        ; "substring rev between")]
    #[test_case(F::SubstringRev(index_range_length()),  "Substring from 2..4 backward"        ; "substring rev length")]
    #[test_case(F::GetField(field_string()),            "Get field #2 (',' separator)"                                  ; "field string separator")]
    #[test_case(F::GetField(field_regex()),             "Get field #2 (regular expression '[, ]+' separator)"           ; "field regex separator")]
    #[test_case(F::GetFieldRev(field_string()),         "Get field #2 (',' separator) backward"                         ; "field rev string separator")]
    #[test_case(F::GetFieldRev(field_regex()),          "Get field #2 (regular expression '[, ]+' separator) backward"  ; "field rev regex separator")]
    #[test_case(F::ReplaceFirst(subst_string_1()),      "Replace first 'ab' with ''"                                    ; "remove first")]
    #[test_case(F::ReplaceFirst(subst_string_2()),      "Replace first 'ab' with 'x'"                                   ; "replace first")]
    #[test_case(F::ReplaceAll(subst_string_1()),        "Replace all 'ab' with ''"                                      ; "remove all")]
    #[test_case(F::ReplaceAll(subst_string_2()),        "Replace all 'ab' with 'x'"                                     ; "replace all")]
    #[test_case(F::ReplaceEmpty(substitution_empty()),  "Replace empty with 'x'"                                        ; "replace empty")]
    // #[test_case(F::RegexMatch("[0-9]+".into()),         "Match of regular expression '[0-9]+'"                          ; "regex match")]
    #[test_case(F::RegexReplaceFirst(subst_regex_1()),  "Replace first match of regular expression '[0-9]+' with ''"    ; "regex remove first")]
    #[test_case(F::RegexReplaceFirst(subst_regex_2()),  "Replace first match of regular expression '[0-9]+' with 'x'"   ; "regex replace first")]
    #[test_case(F::RegexReplaceAll(subst_regex_1()),    "Replace all matches of regular expression '[0-9]+' with ''"    ; "regex remove all")]
    #[test_case(F::RegexReplaceAll(subst_regex_2()),    "Replace all matches of regular expression '[0-9]+' with 'x'"   ; "regex replace all")]
    #[test_case(
        F::RegexSwitch(regex_switch()),
        "Regular expression switch with variable output:\n\n    if input matches '[0-9]+'\n        output is 'X'\n    else\n        output is 'Y'";
        "regex switch "
    )]
    #[test_case(F::RegexCapture(1),                      "Capture group #1 of a global regular expression" ; "regex capture")]
    #[test_case(F::Trim,                                 "Trim"                                            ; "trim")]
    #[test_case(F::ToLowercase,                          "To lowercase"                                    ; "to lowercase")]
    #[test_case(F::ToUppercase,                          "To uppercase"                                    ; "to uppercase")]
    #[test_case(F::ToAscii,                              "To ASCII"                                        ; "to ascii")]
    #[test_case(F::RemoveNonAscii,                       "Remove non-ASCII"                                ; "remove non-ascii")]
    #[test_case(F::LeftPad(padding_fixed()),             "Left pad with 'abcd'"                            ; "left pad fixed")]
    #[test_case(F::LeftPad(padding_repeated()),          "Left pad with 2x 'abc'"                          ; "left pad repeated")]
    #[test_case(F::RightPad(padding_fixed()),            "Right pad with 'abcd'"                           ; "right pad fixed")]
    #[test_case(F::RightPad(padding_repeated()),         "Right pad with 2x 'abc'"                         ; "right pad repeated")]
    #[test_case(F::Repeat(repetition_input()),           "Repeat 2x"                                       ; "repetition input ")]
    #[test_case(F::Repeat(repetition_value()),           "Repeat 2x 'abc'"                                 ; "repetition value ")]
    #[test_case(F::LocalCounter,                         "Local counter"                                   ; "local counter")]
    #[test_case(F::GlobalCounter,                        "Global counter"                                  ; "global counter")]
    #[test_case(F::RandomNumber(number_range_full()),    "Random number from [0, 2^64)"                    ; "random number")]
    #[test_case(F::RandomNumber(number_range_from()),    "Random number from [2, 2^64)"                    ; "random number from")]
    #[test_case(F::RandomNumber(number_range_between()), "Random number from [2, 10]"                      ; "random number between")]
    #[test_case(F::RandomUuid,                           "Random UUID"                                     ; "random uuid")]
    fn display(filter: Filter, result: &str) {
        assert_eq!(filter.to_string(), result);
    }

    fn index_range_at() -> CharIndexRange {
        CharIndexRange::new(1, Some(2))
    }

    fn index_range_from() -> CharIndexRange {
        CharIndexRange::new(1, None)
    }

    fn index_range_between() -> CharIndexRange {
        CharIndexRange::new(1, Some(3))
    }

    fn index_range_length() -> CharIndexRange {
        CharIndexRange::new(1, Some(4))
    }

    fn field_default() -> Field {
        Field {
            index: 1,
            separator: Separator::Regex("\\s+".into()),
        }
    }

    fn field_string() -> Field {
        Field {
            index: 1,
            separator: Separator::String(','.into()),
        }
    }

    fn field_regex() -> Field {
        Field {
            index: 1,
            separator: Separator::Regex("[, ]+".into()),
        }
    }

    fn substitution_empty() -> EmptySubstitution {
        Substitution {
            target: Empty,
            replacement: "x".into(),
        }
    }

    fn subst_string_1() -> StringSubstitution {
        Substitution {
            target: "ab".into(),
            replacement: "".into(),
        }
    }

    fn subst_string_2() -> StringSubstitution {
        Substitution {
            target: "ab".into(),
            replacement: "x".into(),
        }
    }

    fn subst_regex_1() -> RegexSubstitution {
        Substitution {
            target: "[0-9]+".into(),
            replacement: "".into(),
        }
    }

    fn subst_regex_2() -> RegexSubstitution {
        Substitution {
            target: "[0-9]+".into(),
            replacement: "x".into(),
        }
    }

    fn regex_switch() -> RegexSwitch {
        RegexSwitch {
            cases: vec![Case {
                matcher: "[0-9]+".into(),
                result: "X".into(),
            }],
            default: "Y".into(),
        }
    }

    fn padding_fixed() -> Padding {
        Padding::Fixed("abcd".into())
    }

    fn padding_repeated() -> Padding {
        Padding::Repeated(repetition_value())
    }

    fn repetition_input() -> Repetition {
        Repetition {
            count: 2,
            value: None,
        }
    }

    fn repetition_value() -> Repetition {
        Repetition {
            count: 2,
            value: Some("abc".into()),
        }
    }

    fn number_range_full() -> NumberRange {
        NumberRange::new(0, None)
    }

    fn number_range_from() -> NumberRange {
        NumberRange::new(2, None)
    }

    fn number_range_between() -> NumberRange {
        NumberRange::new(2, Some(11))
    }

    fn number_range_zero() -> NumberRange {
        NumberRange::new(0, Some(0))
    }
}
