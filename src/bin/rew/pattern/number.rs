use std::fmt;

use rand::{thread_rng, Rng};

use crate::pattern::range::{Range, RangeType};

pub type Number = u64;
pub type NumberRange = Range<NumberRangeType>;

#[derive(PartialEq, Debug)]
pub struct NumberRangeType;

impl RangeType for NumberRangeType {
    type Value = Number;

    const INDEX: bool = false;
    const EMPTY_ALLOWED: bool = true;
    const DELIMITER_REQUIRED: bool = true;
    const LENGTH_ALLOWED: bool = false;
}

impl NumberRange {
    pub fn random(&self) -> Number {
        let start = self.start();
        let end = self.end().unwrap_or(Number::MAX);

        if start == 0 && end == Number::MAX {
            thread_rng().gen() // gen_range(start..=end) would cause an overflow in rand lib
        } else {
            thread_rng().gen_range(start..=end)
        }
    }
}

impl fmt::Display for NumberRange {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Range(start, Some(end)) => write!(formatter, "[{}, {}]", start, end - 1),
            Range(start, None) => write!(
                formatter,
                "[{}, 2^{})",
                start,
                std::mem::size_of::<Number>() * 8
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    mod parse {
        use test_case::test_case;

        use super::*;
        use crate::pattern::error::ErrorRange;
        use crate::pattern::parse::{Error, ErrorKind};
        use crate::pattern::reader::Reader;

        #[test_case("2-1", 0..3, ErrorKind::RangeStartOverEnd("2".into(), "1".into()) ; "start above end")]
        #[test_case("0+1", 1..2, ErrorKind::ExpectedRangeDelimiter(Some('+'.into()))  ; "start with length")]
        #[test_case("1",   1..1, ErrorKind::ExpectedRangeDelimiter(None)              ; "no delimiter")]
        #[test_case("1ab", 1..2, ErrorKind::ExpectedRangeDelimiter(Some('a'.into()))  ; "wrong delimiter")]
        fn err(input: &str, range: ErrorRange, kind: ErrorKind) {
            assert_eq!(
                NumberRange::parse(&mut Reader::from(input)),
                Err(Error { kind, range })
            );
        }

        #[test_case("",    0, None    ; "empty")]
        #[test_case("-",   0, None    ; "empty ignored chars")]
        #[test_case("0-",  0, None    ; "start no end")]
        #[test_case("0-1", 0, Some(2) ; "start below end")]
        #[test_case("1-1", 1, Some(2) ; "start equals end")]
        fn ok(input: &str, start: Number, end: Option<Number>) {
            assert_eq!(
                NumberRange::parse(&mut Reader::from(input)),
                Ok(NumberRange::new(start, end))
            );
        }
    }

    mod random {
        use test_case::test_case;

        use super::*;

        const MAX: Number = Number::MAX;

        #[test_case(0,   Some(0), 0   ; "lowest")]
        #[test_case(MAX, None,    MAX ; "highest")]
        fn certain(start: Number, end: Option<Number>, result: Number) {
            assert_eq!(NumberRange::new(start, end).random(), result);
        }

        #[test_case(0, Some(MAX)     ; "from 0 to max")] // Should not overflow
        #[test_case(1, Some(MAX)     ; "from 1 to max")]
        #[test_case(0, Some(MAX - 1) ; "from 0 to max-1")]
        fn uncertain(start: Number, end: Option<Number>) {
            NumberRange::new(start, end).random();
        }
    }

    #[test_case(1, None,    "[1, 2^64)" ; "open")]
    #[test_case(1, Some(3), "[1, 2]"    ; "closed")]
    fn display(start: Number, end: Option<Number>, result: &str) {
        assert_eq!(NumberRange::new(start, end).to_string(), result);
    }
}
