use std::convert::Infallible;
use std::ops::Range;
use std::{error, fmt, result};

use crate::pattern::char::{Char, EscapeSequence};
use crate::pattern::error::{ErrorRange, GetErrorRange};
use crate::pattern::escape::{escape_char, escape_str};
use crate::pattern::regex::RegexHolder;
use crate::pattern::symbols::{EXPR_END, EXPR_START, PIPE, RANGE_OF_LENGTH, RANGE_TO};
use crate::pattern::utils::AnyString;

pub struct Config {
    pub escape: char,
    pub separator: Separator,
}

#[cfg(test)]
impl Config {
    pub fn fixture() -> Self {
        Self {
            escape: '%',
            separator: Separator::Regex("\\s+".into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Separator {
    String(String),
    Regex(RegexHolder),
}

impl fmt::Display for Separator {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Separator::String(separator) => write!(formatter, "'{}'", escape_str(&separator)),
            Separator::Regex(separator) => write!(formatter, "regular expression '{}'", separator),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Parsed<T> {
    pub value: T,
    pub range: Range<usize>,
}

#[cfg(test)]
impl<T> From<T> for Parsed<T> {
    fn from(value: T) -> Self {
        Self { value, range: 0..0 }
    }
}

pub type Result<T> = result::Result<T, Error>;
pub type BaseResult<T> = result::Result<T, ErrorKind>;

#[derive(Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub range: ErrorRange,
}

impl error::Error for Error {}

impl GetErrorRange for Error {
    fn error_range(&self) -> &ErrorRange {
        &self.range
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Invalid pattern: {}", self.kind)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorKind {
    ExpectedDelimiterChar,
    ExpectedFieldSeparator,
    ExpectedFilter,
    ExpectedNumber,
    ExpectedFilterOrExprEnd,
    ExpectedPipeOrExprEnd,
    ExpectedRange,
    ExpectedRangeDelimiter(Option<Char>),
    ExpectedRangeLength,
    ExpectedRegex,
    ExpectedRegexRange,
    ExpectedRegexSwitch,
    ExpectedRepetition,
    ExpectedSubstitution,
    ExprStartInsideExpr,
    IndexZero,
    IntegerOverflow(String),
    PaddingPrefixInvalid(char, Option<Char>),
    PipeOutsideExpr,
    RangeInvalid(String),
    RangeStartOverEnd(String, String),
    RegexInvalid(AnyString),
    RegexSwitchWithoutMatcher(Char, usize),
    SubstitutionWithoutTarget(Char),
    UnknownEscapeSequence(EscapeSequence),
    UnknownFilter(Char),
    UnmatchedExprEnd,
    UnmatchedExprStart,
    UnterminatedEscapeSequence(char),
}

#[derive(PartialEq, Debug)]
pub enum ErrorHint {
    RegexSyntax,
    PatternSyntax,
    FilterUsage,
}

impl ErrorKind {
    pub fn hint(&self) -> Option<ErrorHint> {
        match self {
            Self::ExpectedDelimiterChar => Some(ErrorHint::FilterUsage),
            Self::ExpectedFieldSeparator => Some(ErrorHint::FilterUsage),
            Self::ExpectedFilter => Some(ErrorHint::PatternSyntax),
            Self::ExpectedNumber => Some(ErrorHint::FilterUsage),
            Self::ExpectedFilterOrExprEnd => Some(ErrorHint::PatternSyntax),
            Self::ExpectedPipeOrExprEnd => Some(ErrorHint::PatternSyntax),
            Self::ExpectedRange => Some(ErrorHint::FilterUsage),
            Self::ExpectedRangeDelimiter(_) => Some(ErrorHint::FilterUsage),
            Self::ExpectedRangeLength => Some(ErrorHint::FilterUsage),
            Self::ExpectedRegex => Some(ErrorHint::FilterUsage),
            Self::ExpectedRegexRange => Some(ErrorHint::FilterUsage),
            Self::ExpectedRegexSwitch => Some(ErrorHint::FilterUsage),
            Self::ExpectedRepetition => Some(ErrorHint::FilterUsage),
            Self::ExpectedSubstitution => Some(ErrorHint::FilterUsage),
            Self::ExprStartInsideExpr => Some(ErrorHint::PatternSyntax),
            Self::IndexZero => Some(ErrorHint::FilterUsage),
            Self::IntegerOverflow(_) => None,
            Self::PaddingPrefixInvalid(_, _) => Some(ErrorHint::FilterUsage),
            Self::PipeOutsideExpr => Some(ErrorHint::PatternSyntax),
            Self::RangeInvalid(_) => Some(ErrorHint::FilterUsage),
            Self::RangeStartOverEnd(_, _) => Some(ErrorHint::FilterUsage),
            Self::RegexInvalid(_) => Some(ErrorHint::RegexSyntax),
            Self::RegexSwitchWithoutMatcher(_, _) => Some(ErrorHint::FilterUsage),
            Self::SubstitutionWithoutTarget(_) => Some(ErrorHint::FilterUsage),
            Self::UnknownEscapeSequence(_) => Some(ErrorHint::PatternSyntax),
            Self::UnknownFilter(_) => Some(ErrorHint::FilterUsage),
            Self::UnmatchedExprEnd => Some(ErrorHint::PatternSyntax),
            Self::UnmatchedExprStart => Some(ErrorHint::PatternSyntax),
            Self::UnterminatedEscapeSequence(_) => Some(ErrorHint::PatternSyntax),
        }
    }
}

impl From<Infallible> for ErrorKind {
    fn from(_: Infallible) -> Self {
        unreachable!("Infallible to parse::ErrorKind conversion should never happen");
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ExpectedDelimiterChar => write!(formatter, "Expected delimiter character"),
            Self::ExpectedFieldSeparator => write!(formatter, "Expected field separator"),
            Self::ExpectedFilter => write!(formatter, "Expected filter after '{}'", PIPE),
            Self::ExpectedNumber => write!(formatter, "Expected number"),
            Self::ExpectedFilterOrExprEnd => {
                write!(formatter, "Expected filter or closing '{}'", EXPR_END)
            }
            Self::ExpectedPipeOrExprEnd => {
                write!(formatter, "Expected '{}' or closing '{}'", PIPE, EXPR_END)
            }
            Self::ExpectedRange => write!(
                formatter,
                "Expected range 'A{}B', 'A{}', 'A' or 'A{}B'",
                RANGE_TO, RANGE_TO, RANGE_OF_LENGTH
            ),
            Self::ExpectedRangeDelimiter(None) => {
                write!(formatter, "Expected range delimiter '{}'", RANGE_TO)
            }
            Self::ExpectedRangeDelimiter(Some(char)) => write!(
                formatter,
                "Expected range delimiter '{}' but got {}",
                RANGE_TO, char
            ),
            Self::ExpectedRangeLength => {
                write!(
                    formatter,
                    "Expected range length after '{}'",
                    RANGE_OF_LENGTH
                )
            }
            Self::ExpectedRegex => write!(formatter, "Expected regular expression"),
            Self::ExpectedRegexRange => write!(
                formatter,
                "Expected regular expression range 'A{}B:X', 'A{}:X', 'A:X' or ':X'",
                RANGE_TO, RANGE_TO
            ),
            Self::ExpectedRegexSwitch => write!(
                formatter,
                "Expected regular expression switch ':X1:Y1:...:Xn:Yn:D'"
            ),
            Self::ExpectedRepetition => {
                write!(formatter, "Expected repetition 'N:V' or 'N'")
            }
            Self::ExpectedSubstitution => write!(formatter, "Expected substitution ':A:B' or ':A'"),
            Self::ExprStartInsideExpr => {
                write!(formatter, "Unescaped '{}' inside expression", EXPR_START)
            }
            Self::IndexZero => write!(formatter, "Indices start from 1, not 0"),
            Self::IntegerOverflow(max) => {
                write!(formatter, "Cannot parse value greater than {}", max)
            }
            Self::PaddingPrefixInvalid(fixed_prefix, None) => {
                write!(formatter, "Expected '{}' prefix or number", fixed_prefix)
            }
            Self::PaddingPrefixInvalid(expected, Some(char)) => write!(
                formatter,
                "Expected '{}' prefix or number but got {}",
                expected, char
            ),
            Self::PipeOutsideExpr => write!(formatter, "Unescaped '{}' outside expression", PIPE),
            Self::RangeInvalid(value) => {
                write!(formatter, "Invalid range '{}'", escape_str(&value))
            }
            Self::RangeStartOverEnd(start, end) => write!(
                formatter,
                "Range start {} is greater than end {}",
                start, end
            ),
            Self::RegexInvalid(value) => {
                write!(formatter, "Invalid regular expression '{}'", value)
            }
            Self::RegexSwitchWithoutMatcher(char, index) => write!(
                formatter,
                "Regular expression switch is missing value after {} delimiter #{}",
                char,
                index + 1,
            ),
            Self::SubstitutionWithoutTarget(char) => write!(
                formatter,
                "Substitution is missing value after {} delimiter",
                char
            ),
            Self::UnknownEscapeSequence(sequence) => write!(
                formatter,
                "Unknown escape sequence '{}{}'",
                escape_char(sequence[0]),
                escape_char(sequence[1])
            ),
            Self::UnknownFilter(char) => {
                write!(formatter, "Unknown filter {}", char)
            }
            Self::UnmatchedExprEnd => write!(
                formatter,
                "No matching '{}' before expression end",
                EXPR_START
            ),
            Self::UnmatchedExprStart => write!(
                formatter,
                "No matching '{}' after expression start",
                EXPR_END
            ),
            Self::UnterminatedEscapeSequence(escape) => {
                write!(
                    formatter,
                    "Unterminated escape sequence '{}'",
                    escape_char(*escape)
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case(Separator::String(",".into()),   "','"                       ; "string")]
    #[test_case(Separator::Regex("\\s+".into()), "regular expression '\\s+'" ; "regex")]
    fn separator_display(separator: Separator, result: &str) {
        assert_eq!(separator.to_string(), result);
    }

    mod error {
        use super::*;

        #[test]
        fn range() {
            assert_eq!(
                Error {
                    kind: ErrorKind::ExpectedNumber,
                    range: 1..2
                }
                .error_range(),
                &(1..2)
            )
        }

        #[test]
        fn display() {
            assert_eq!(
                Error {
                    kind: ErrorKind::ExpectedNumber,
                    range: 1..2
                }
                .to_string(),
                "Invalid pattern: Expected number"
            );
        }
    }

    mod error_kind {
        use test_case::test_case;

        use super::*;

        type E = ErrorKind;
        type H = ErrorHint;

        #[test_case(E::ExpectedDelimiterChar,                     Some(H::FilterUsage)   ; "expected delimiter char")]
        #[test_case(E::ExpectedFieldSeparator,                    Some(H::FilterUsage)   ; "expected field separator")]
        #[test_case(E::ExpectedFilter,                            Some(H::PatternSyntax) ; "expected filter")]
        #[test_case(E::ExpectedNumber,                            Some(H::FilterUsage)   ; "expected number")]
        #[test_case(E::ExpectedFilterOrExprEnd,                   Some(H::PatternSyntax) ; "expected filter or expr end")]
        #[test_case(E::ExpectedPipeOrExprEnd,                     Some(H::PatternSyntax) ; "expected pipe or expr end")]
        #[test_case(E::ExpectedRange,                             Some(H::FilterUsage)   ; "expected range")]
        #[test_case(E::ExpectedRangeDelimiter(None),              Some(H::FilterUsage)   ; "expected delimiter got none")]
        #[test_case(E::ExpectedRangeLength,                       Some(H::FilterUsage)   ; "expected range length")]
        #[test_case(E::ExpectedRegex,                             Some(H::FilterUsage)   ; "expected regex")]
        #[test_case(E::ExpectedRegexRange,                        Some(H::FilterUsage)   ; "expected regex range")]
        #[test_case(E::ExpectedRegexSwitch,                       Some(H::FilterUsage)   ; "expected regex switch")]
        #[test_case(E::ExpectedRepetition,                        Some(H::FilterUsage)   ; "expected repetition")]
        #[test_case(E::ExpectedSubstitution,                      Some(H::FilterUsage)   ; "expected substitution")]
        #[test_case(E::ExprStartInsideExpr,                       Some(H::PatternSyntax) ; "expr start inside expr")]
        #[test_case(E::IndexZero,                                 Some(H::FilterUsage)   ; "index zero")]
        #[test_case(E::IntegerOverflow("255".into()),             None                   ; "integer overflow")]
        #[test_case(E::PaddingPrefixInvalid('<', None),           Some(H::FilterUsage)   ; "padding prefix missing")]
        #[test_case(E::PipeOutsideExpr,                           Some(H::PatternSyntax) ; "pipe outside expr")]
        #[test_case(E::RangeInvalid("abc".into()),                Some(H::FilterUsage)   ; "range invalid")]
        #[test_case(E::RangeStartOverEnd("2".into(), "1".into()), Some(H::FilterUsage)   ; "range start over end")]
        #[test_case(E::RegexInvalid("abc".into()),                Some(H::RegexSyntax)   ; "regex invalid")]
        #[test_case(E::RegexSwitchWithoutMatcher('_'.into(), 0),  Some(H::FilterUsage)   ; "regex switch without matcher")]
        #[test_case(E::SubstitutionWithoutTarget('_'.into()),     Some(H::FilterUsage)   ; "substitution without target")]
        #[test_case(E::UnknownEscapeSequence(['%', 'x']),         Some(H::PatternSyntax) ; "unknown escape sequence" )]
        #[test_case(E::UnknownFilter('x'.into()),                 Some(H::FilterUsage)   ; "unknown filter")]
        #[test_case(E::UnmatchedExprEnd,                          Some(H::PatternSyntax) ; "unmatched expr end")]
        #[test_case(E::UnmatchedExprStart,                        Some(H::PatternSyntax) ; "unmatched expr start")]
        #[test_case(E::UnterminatedEscapeSequence('%'),           Some(H::PatternSyntax) ; "unterminated escape sequence")]
        fn hint(kind: ErrorKind, hint: Option<ErrorHint>) {
            assert_eq!(kind.hint(), hint);
        }

        #[test_case(E::ExpectedDelimiterChar,                       "Expected delimiter character"                                      ; "expected delimiter character")]
        #[test_case(E::ExpectedFieldSeparator,                      "Expected field separator"                                          ; "expected field separator")]
        #[test_case(E::ExpectedFilter,                              "Expected filter after '|'"                                         ; "expected filter")]
        #[test_case(E::ExpectedNumber,                              "Expected number"                                                   ; "expected number")]
        #[test_case(E::ExpectedFilterOrExprEnd,                     "Expected filter or closing '}'"                                    ; "expected filter or expr end")]
        #[test_case(E::ExpectedPipeOrExprEnd,                       "Expected '|' or closing '}'"                                       ; "expected pipe or expr end")]
        #[test_case(E::ExpectedRange,                               "Expected range 'A-B', 'A-', 'A' or 'A+B'"                          ; "expected range")]
        #[test_case(E::ExpectedRangeDelimiter(None),                "Expected range delimiter '-'"                                      ; "expected delimiter got none")]
        #[test_case(E::ExpectedRangeDelimiter(Some('x'.into())),    "Expected range delimiter '-' but got 'x'"                          ; "expected delimiter got invalid")]
        #[test_case(E::ExpectedRangeLength,                         "Expected range length after '+'"                                   ; "expected range length")]
        #[test_case(E::ExpectedRegex,                               "Expected regular expression"                                       ; "expected regex")]
        #[test_case(E::ExpectedRegexRange,                          "Expected regular expression range 'A-B:X', 'A-:X', 'A:X' or ':X'"  ; "expected regex range")]
        #[test_case(E::ExpectedRegexSwitch,                         "Expected regular expression switch ':X1:Y1:...:Xn:Yn:D'"           ; "expected regex switch")]
        #[test_case(E::ExpectedRepetition,                          "Expected repetition 'N:V' or 'N'"                                  ; "expected repetition")]
        #[test_case(E::ExpectedSubstitution,                        "Expected substitution ':A:B' or ':A'"                              ; "expected substitution")]
        #[test_case(E::ExprStartInsideExpr,                         "Unescaped '{' inside expression"                                   ; "expr start inside expr")]
        #[test_case(E::IndexZero,                                   "Indices start from 1, not 0"                                       ; "index zero")]
        #[test_case(E::IntegerOverflow("255".into()),               "Cannot parse value greater than 255"                               ; "integer overflow")]
        #[test_case(E::PaddingPrefixInvalid('<', None),             "Expected '<' prefix or number"                                     ; "padding prefix missing")]
        #[test_case(E::PaddingPrefixInvalid('<', Some('x'.into())), "Expected '<' prefix or number but got 'x'"                         ; "padding prefix invalid")]
        #[test_case(E::PipeOutsideExpr,                             "Unescaped '|' outside expression"                                  ; "pipe outside expr")]
        #[test_case(E::RangeInvalid("abc".into()),                  "Invalid range 'abc'"                                               ; "range invalid")]
        #[test_case(E::RangeStartOverEnd("2".into(), "1".into()),   "Range start 2 is greater than end 1"                               ; "range start over end")]
        #[test_case(E::RegexInvalid("abc".into()),                  "Invalid regular expression 'abc'"                                  ; "regex invalid")]
        #[test_case(E::RegexSwitchWithoutMatcher('_'.into(), 0),    "Regular expression switch is missing value after '_' delimiter #1" ; "switch without matcher")]
        #[test_case(E::SubstitutionWithoutTarget('_'.into()),       "Substitution is missing value after '_' delimiter"                 ; "substitution without target")]
        #[test_case(E::UnknownEscapeSequence(['%', 'x']),           "Unknown escape sequence '%x'"                                      ; "unknown escape sequence" )]
        #[test_case(E::UnknownFilter('x'.into()),                   "Unknown filter 'x'"                                                ; "unknown filter")]
        #[test_case(E::UnmatchedExprEnd,                            "No matching '{' before expression end"                             ; "unmatched expr end")]
        #[test_case(E::UnmatchedExprStart,                          "No matching '}' after expression start"                            ; "unmatched expr start")]
        #[test_case(E::UnterminatedEscapeSequence('%'),             "Unterminated escape sequence '%'"                                  ; "unterminated escape sequence")]
        fn display(kind: ErrorKind, result: &str) {
            assert_eq!(kind.to_string(), result);
        }
    }
}
