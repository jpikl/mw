use std::borrow::Cow;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::ops::Deref;

use lazy_static::lazy_static;
use regex::Regex;

use crate::pattern::char::Char;
use crate::pattern::parse::ErrorKind::ExpectedRegexRange;
use crate::pattern::parse::{Error, ErrorKind, Result};
use crate::pattern::range::{Range, RangeType};
use crate::pattern::reader::Reader;
use crate::pattern::utils::AnyString;

lazy_static! {
    static ref CAPTURE_GROUP_VAR_REGEX: Regex = Regex::new(r"\$(\d+)").unwrap();
}

pub fn add_capture_group_brackets(string: &str) -> Cow<str> {
    if string.contains('$') {
        CAPTURE_GROUP_VAR_REGEX.replace_all(string, r"$${${1}}")
    } else {
        Cow::Borrowed(string)
    }
}

#[derive(Debug, Clone)]
pub struct RegexHolder(pub Regex);

impl RegexHolder {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        let value_start = reader.position();
        let value = reader.read_to_end();

        if value.is_empty() {
            Err(Error {
                kind: ErrorKind::ExpectedRegex,
                range: value_start..value_start,
            })
        } else {
            value.to_string().try_into().map_err(|kind| Error {
                kind,
                range: value_start..reader.position(),
            })
        }
    }
}

#[cfg(test)]
impl From<&str> for RegexHolder {
    fn from(value: &str) -> Self {
        Self(Regex::new(value).unwrap())
    }
}

impl TryFrom<String> for RegexHolder {
    type Error = ErrorKind;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match Regex::new(&value) {
            Ok(regex) => Ok(Self(regex)),
            Err(error) => Err(ErrorKind::RegexInvalid(AnyString(error.to_string()))),
        }
    }
}

impl Deref for RegexHolder {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for RegexHolder {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl fmt::Display for RegexHolder {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(PartialEq, Debug)]
pub struct RegexRangeType;

#[derive(PartialEq, Debug)]
pub struct RegexRange {
    range: Range<RegexRangeType>,
    regex: RegexHolder,
}

impl RangeType for RegexRangeType {
    type Value = usize;

    const INDEX: bool = true;
    const EMPTY_ALLOWED: bool = true;
    const DELIMITER_REQUIRED: bool = false;
    const LENGTH_ALLOWED: bool = false;
}

impl RegexRange {
    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        if reader.peek().is_some() {
            let range = Range::parse(reader)?;
            if reader.read().is_some() {
                let regex = RegexHolder::parse(reader)?;
                Ok(Self { range, regex })
            } else {
                Err(Error {
                    kind: ErrorKind::ExpectedDelimiterChar,
                    range: reader.position()..reader.position(),
                })
            }
        } else {
            Err(Error {
                kind: ExpectedRegexRange,
                range: reader.position()..reader.position(),
            })
        }
    }

    pub fn find(&self, value: &str) -> String {
        self.find_range(value, self.range.start(), self.range.length())
    }

    pub fn find_rev(&self, value: &str) -> String {
        // Regex does not support DoubleEndedIterator
        let count = self.regex.find_iter(value).count();
        if self.range.start() < count {
            let end = count - self.range.start();
            let start = match self.range.length() {
                Some(length) if end > length => end - length,
                Some(_) | None => 0,
            };
            self.find_range(value, start, Some(end - start))
        } else {
            String::new()
        }
    }

    pub fn find_range(&self, value: &str, start: usize, length: Option<usize>) -> String {
        let mut matches = self.regex.find_iter(value).skip(start);

        if let Some(first) = matches.next() {
            let match_start = first.start();
            let mut match_end = first.end();

            for _ in 1..length.unwrap_or(usize::MAX) {
                if let Some(next) = matches.next() {
                    match_end = next.end();
                } else {
                    break;
                }
            }
            value[match_start..match_end].into()
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case("",            ""                  ; "empty")]
    #[test_case("ab",          "ab"                ; "zero")]
    #[test_case("a$1b",        "a${1}b"            ; "one")]
    #[test_case("$1a$12b$123", "${1}a${12}b${123}" ; "multiple")]
    fn add_capture_group_brackets(input: &str, output: &str) {
        assert_eq!(super::add_capture_group_brackets(input), output)
    }

    mod regex_holder {
        use test_case::test_case;

        use super::*;

        mod try_from {
            use test_case::test_case;

            use super::*;

            #[test]
            fn err() {
                assert_eq!(
                    RegexHolder::try_from(String::from("[0-9")),
                    Err(ErrorKind::RegexInvalid(AnyString::any()))
                );
            }

            #[test_case("",       ""       ; "empty")]
            #[test_case("[a-z]+", "[a-z]+" ; "noempty")]
            fn ok(input: &str, output: &str) {
                assert_eq!(
                    RegexHolder::try_from(String::from(input)),
                    Ok(output.into())
                );
            }
        }

        mod parse {
            use test_case::test_case;

            use super::*;
            use crate::pattern::error::ErrorRange;

            #[test_case("",     0..0, ErrorKind::ExpectedRegex                  ; "empty")]
            #[test_case("[0-9", 0..4, ErrorKind::RegexInvalid(AnyString::any()) ; "invalid")]
            fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
                assert_eq!(
                    RegexHolder::parse(&mut Reader::from(input)),
                    Err(Error { kind, range })
                );
            }

            #[test]
            fn ok() {
                assert_eq!(
                    RegexHolder::parse(&mut Reader::from("[0-9]")),
                    Ok("[0-9]".into())
                );
            }
        }

        #[test_case("",       "",       true  ; "empty")]
        #[test_case("[a-z]+", "[a-z]+", true  ; "same")]
        #[test_case("[a-z]+", "[a-z]*", false ; "different")]
        fn partial_eq(left: &str, right: &str, result: bool) {
            assert_eq!(RegexHolder::from(left) == RegexHolder::from(right), result);
        }

        #[test]
        fn display() {
            assert_eq!(RegexHolder::from("[a-z]+").to_string(), "[a-z]+");
        }
    }

    mod regex_range {
        use test_case::test_case;

        use super::*;

        mod parse {
            use test_case::test_case;

            use super::*;
            use crate::pattern::error::ErrorRange;

            #[test_case(":[a-z]+",    0, None,    "[a-z]+" ; "empty")]
            #[test_case("1:[a-z]+",   0, Some(1), "[a-z]+" ; "single position")]
            #[test_case("1+[a-z]+",   0, Some(1), "[a-z]+" ; "single position plus delimiter")]
            #[test_case("1-:[a-z]+",  0, None,    "[a-z]+" ; "start no end")]
            #[test_case("2-3:[a-z]+", 1, Some(3), "[a-z]+" ; "start below end")]
            #[test_case("2-2:[a-z]+", 1, Some(2), "[a-z]+" ; "start equals end")]
            fn ok(input: &str, start: usize, end: Option<usize>, regex: &str) {
                assert_eq!(
                    RegexRange::parse(&mut Reader::from(input)),
                    Ok(RegexRange {
                        range: Range::new(start, end),
                        regex: regex.into()
                    })
                );
            }

            #[test_case("",       0..0, ErrorKind::ExpectedRegexRange                        ; "empty")]
            #[test_case("-",      1..1, ErrorKind::ExpectedRegex                             ; "dash delimiter missing regex")]
            #[test_case("0-",     0..1, ErrorKind::IndexZero                                 ; "zero start")]
            #[test_case("1-0",    2..3, ErrorKind::IndexZero                                 ; "zero end")]
            #[test_case("2-1",    0..3, ErrorKind::RangeStartOverEnd("2".into(), "1".into()) ; "start above end")]
            #[test_case("1",      1..1, ErrorKind::ExpectedDelimiterChar                     ; "missing delimiter")]
            #[test_case(":",      1..1, ErrorKind::ExpectedRegex                             ; "empty range missing regex")]
            #[test_case(":[0-9",  1..5, ErrorKind::RegexInvalid(AnyString::any())            ; "empty range invalid regex")]
            #[test_case("1:",     2..2, ErrorKind::ExpectedRegex                             ; "nonempty range missing regex")]
            #[test_case("1:[0-9", 2..6, ErrorKind::RegexInvalid(AnyString::any())            ; "nonempty range invalid regex")]
            fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
                assert_eq!(
                    RegexRange::parse(&mut Reader::from(input)),
                    Err(Error { kind, range })
                );
            }
        }

        #[test_case("a12b34c56d", 0, Some(1), "\\d+", "12";       "first")]
        #[test_case("a12b34c56d", 0, Some(2), "\\d+", "12b34";    "first to second")]
        #[test_case("a12b34c56d", 0, Some(3), "\\d+", "12b34c56"; "first to last")]
        #[test_case("a12b34c56d", 0, Some(4), "\\d+", "12b34c56"; "first to over")]
        #[test_case("a12b34c56d", 0, None,    "\\d+", "12b34c56"; "first to end")]
        #[test_case("a12b34c56d", 1, Some(2), "\\d+", "34";       "second")]
        #[test_case("a12b34c56d", 1, Some(3), "\\d+", "34c56";    "second to last")]
        #[test_case("a12b34c56d", 1, Some(4), "\\d+", "34c56";    "second to over")]
        #[test_case("a12b34c56d", 1, None,    "\\d+", "34c56";    "second to end")]
        #[test_case("a12b34c56d", 2, Some(3), "\\d+", "56";       "third")]
        #[test_case("a12b34c56d", 2, Some(4), "\\d+", "56";       "third to over")]
        #[test_case("a12b34c56d", 2, None,    "\\d+", "56";       "third to end")]
        #[test_case("a12b34c56d", 3, Some(4), "\\d+", "";         "over to over")]
        #[test_case("a12b34c56d", 3, None,    "\\d+", "";         "over to end")]
        fn find(input: &str, start: usize, end: Option<usize>, regex: &str, output: &str) {
            assert_eq!(
                RegexRange {
                    range: Range::new(start, end),
                    regex: regex.into()
                }
                .find(input),
                output
            );
        }

        #[test_case("a12b34c56d", 0, Some(1), "\\d+", "56";       "first")]
        #[test_case("a12b34c56d", 0, Some(2), "\\d+", "34c56";    "first to second")]
        #[test_case("a12b34c56d", 0, Some(3), "\\d+", "12b34c56"; "first to last")]
        #[test_case("a12b34c56d", 0, Some(4), "\\d+", "12b34c56"; "first to over")]
        #[test_case("a12b34c56d", 0, None,    "\\d+", "12b34c56"; "first to end")]
        #[test_case("a12b34c56d", 1, Some(2), "\\d+", "34";       "second")]
        #[test_case("a12b34c56d", 1, Some(3), "\\d+", "12b34";    "second to last")]
        #[test_case("a12b34c56d", 1, Some(4), "\\d+", "12b34";    "second to over")]
        #[test_case("a12b34c56d", 1, None,    "\\d+", "12b34";    "second to end")]
        #[test_case("a12b34c56d", 2, Some(3), "\\d+", "12";       "third")]
        #[test_case("a12b34c56d", 2, Some(4), "\\d+", "12";       "third to over")]
        #[test_case("a12b34c56d", 2, None,    "\\d+", "12";       "third to end")]
        #[test_case("a12b34c56d", 3, Some(4), "\\d+", "";         "over to over")]
        #[test_case("a12b34c56d", 3, None,    "\\d+", "";         "over to end")]
        fn find_rev(input: &str, start: usize, end: Option<usize>, regex: &str, output: &str) {
            assert_eq!(
                RegexRange {
                    range: Range::new(start, end),
                    regex: regex.into()
                }
                .find_rev(input),
                output
            );
        }
    }
}
