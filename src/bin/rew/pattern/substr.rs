use std::fmt;

use crate::pattern::range::{Range, RangeType};

pub type CharIndexRange = Range<CharIndexRangeType>;

#[derive(PartialEq, Debug)]
pub struct CharIndexRangeType;

impl RangeType for CharIndexRangeType {
    type Value = usize;

    const INDEX: bool = true;
    const EMPTY_ALLOWED: bool = false;
    const DELIMITER_REQUIRED: bool = false;
    const LENGTH_ALLOWED: bool = true;
}

impl CharIndexRange {
    pub fn substr(&self, mut value: String) -> String {
        if let Some((start, _)) = value.char_indices().nth(self.start()) {
            value.replace_range(..start, "");
        } else {
            value.clear();
        }

        if let Some(length) = self.length() {
            if let Some((end, _)) = value.char_indices().nth(length) {
                value.truncate(end);
            }
        }

        value
    }

    pub fn substr_rev(&self, mut value: String) -> String {
        let start = self.start();
        if start > 0 {
            if let Some((start, _)) = value.char_indices().nth_back(start - 1) {
                value.truncate(start);
            } else {
                value.clear();
            }
        }

        if let Some(length) = self.length() {
            if length > 0 {
                if let Some((end, _)) = value.char_indices().nth_back(length - 1) {
                    value.replace_range(..end, "");
                }
            } else {
                value.clear();
            }
        }

        value
    }
}

impl fmt::Display for CharIndexRange {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Range(start, Some(end)) => write!(formatter, "{}..{}", start + 1, end),
            Range(start, None) => write!(formatter, "{}..", start + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::pattern::error::ErrorRange;

    mod parse {
        use test_case::test_case;

        use super::*;
        use crate::pattern::parse::{Error, ErrorKind};
        use crate::pattern::reader::Reader;

        #[test_case("",     0..0, ErrorKind::ExpectedRange                             ; "empty")]
        #[test_case("-",    0..1, ErrorKind::RangeInvalid("-".into())                  ; "invalid")]
        #[test_case("0-",   0..1, ErrorKind::IndexZero                                 ; "zero start")]
        #[test_case("1-0",  2..3, ErrorKind::IndexZero                                 ; "zero end")]
        #[test_case("2-1",  0..3, ErrorKind::RangeStartOverEnd("2".into(), "1".into()) ; "start above end")]
        #[test_case("1+",   2..2, ErrorKind::ExpectedRangeLength                       ; "start no length")]
        #[test_case("1+ab", 2..4, ErrorKind::ExpectedRangeLength                       ; "start no length but chars")]
        fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
            assert_eq!(
                CharIndexRange::parse(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("1",   0, Some(1) ; "no delimiter")]
        #[test_case("2ab", 1, Some(2) ; "no delimiter but chars")]
        #[test_case("1-",  0, None    ; "no end")]
        #[test_case("1-2", 0, Some(2) ; "start below end")]
        #[test_case("1-1", 0, Some(1) ; "start equals end")]
        #[test_case("2+0", 1, Some(1) ; "start with zero length")]
        #[test_case("2+3", 1, Some(4) ; "start with positive length")]
        #[test_case(&format!("2+{}", usize::MAX), 1, None ; "start with overflow length")]
        fn ok(input: &str, start: usize, end: Option<usize>) {
            assert_eq!(
                CharIndexRange::parse(&mut Reader::from(input)),
                Ok(CharIndexRange::new(start, end))
            );
        }
    }

    #[test_case("",     0, None,    ""     ; "empty")]
    #[test_case("ábčd", 0, None,    "ábčd" ; "before first")]
    #[test_case("ábčd", 3, None,    "d"    ; "before last")]
    #[test_case("ábčd", 4, None,    ""     ; "after last")]
    #[test_case("ábčd", 5, None,    ""     ; "over last")]
    #[test_case("ábčd", 0, Some(0), ""     ; "before first before first")]
    #[test_case("ábčd", 0, Some(1), "á"    ; "before first after first")]
    #[test_case("ábčd", 0, Some(3), "ábč"  ; "before first before last")]
    #[test_case("ábčd", 0, Some(4), "ábčd" ; "before first after last")]
    #[test_case("ábčd", 0, Some(5), "ábčd" ; "before first over last")]
    #[test_case("ábčd", 3, Some(3), ""     ; "before last before last")]
    #[test_case("ábčd", 3, Some(4), "d"    ; "before last after last")]
    #[test_case("ábčd", 3, Some(5), "d"    ; "before last over last")]
    #[test_case("ábčd", 4, Some(4), ""     ; "after last after last")]
    #[test_case("ábčd", 4, Some(5), ""     ; "after last over last")]
    #[test_case("ábčd", 5, Some(5), ""     ; "over last over last")]
    fn substr(input: &str, start: usize, end: Option<usize>, output: &str) {
        assert_eq!(CharIndexRange::new(start, end).substr(input.into()), output);
    }

    #[test_case("",     0, None,    ""     ; "empty")]
    #[test_case("ábčd", 0, None,    "ábčd" ; "before first")]
    #[test_case("ábčd", 3, None,    "á"    ; "before last")]
    #[test_case("ábčd", 4, None,    ""     ; "after last")]
    #[test_case("ábčd", 5, None,    ""     ; "over last")]
    #[test_case("ábčd", 0, Some(0), ""     ; "before first before first")]
    #[test_case("ábčd", 0, Some(1), "d"    ; "before first after first")]
    #[test_case("ábčd", 0, Some(3), "bčd"  ; "before first before last")]
    #[test_case("ábčd", 0, Some(4), "ábčd" ; "before first after last")]
    #[test_case("ábčd", 0, Some(5), "ábčd" ; "before first over last")]
    #[test_case("ábčd", 3, Some(3), ""     ; "before last before last")]
    #[test_case("ábčd", 3, Some(4), "á"    ; "before last after last")]
    #[test_case("ábčd", 3, Some(5), "á"    ; "before last over last")]
    #[test_case("ábčd", 4, Some(4), ""     ; "after last after last")]
    #[test_case("ábčd", 4, Some(5), ""     ; "after last over last")]
    #[test_case("ábčd", 5, Some(5), ""     ; "over last over last")]
    fn substr_rev(input: &str, start: usize, end: Option<usize>, output: &str) {
        assert_eq!(
            CharIndexRange::new(start, end).substr_rev(input.into()),
            output
        );
    }

    #[test_case(1, None,    "2.."  ; "open")]
    #[test_case(1, Some(3), "2..3" ; "closed")]
    fn display(start: usize, end: Option<usize>, result: &str) {
        assert_eq!(CharIndexRange::new(start, end).to_string(), result);
    }
}
