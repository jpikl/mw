use num_traits::{CheckedAdd, One, Zero};

use crate::pattern::char::Char;
use crate::pattern::index::shift_index;
use crate::pattern::integer::{parse_integer, ParsableInt};
use crate::pattern::parse::{BaseResult, Error, ErrorKind, Result};
use crate::pattern::reader::Reader;
use crate::pattern::symbols::{RANGE_OF_LENGTH, RANGE_TO};

pub trait RangeType {
    type Value: ParsableInt;

    const INDEX: bool;
    const EMPTY_ALLOWED: bool;
    const DELIMITER_REQUIRED: bool;
    const LENGTH_ALLOWED: bool;
}

#[derive(Debug, PartialEq)]
pub struct Range<T: RangeType>(pub T::Value, pub Option<T::Value>);

impl<T: RangeType> Range<T> {
    #[allow(dead_code)] // Clippy bug
    pub fn new(start: T::Value, end: Option<T::Value>) -> Self {
        Self(start, end)
    }

    pub fn parse(reader: &mut Reader<Char>) -> Result<Self> {
        match reader.peek_char() {
            Some('0'..='9') => {
                let start_pos = reader.position();
                let start_val = parse_integer(reader)?;

                let start = Self::maybe_shift(start_val).map_err(|kind| Error {
                    kind,
                    range: start_pos..reader.position(),
                })?;

                match reader.peek_char() {
                    Some(RANGE_TO) => {
                        reader.seek();

                        if let Some('0'..='9') = reader.peek_char() {
                            let end_pos = reader.position();
                            let end_val = parse_integer(reader)?;

                            let end = Self::maybe_shift(end_val).map_err(|kind| Error {
                                kind,
                                range: end_pos..reader.position(),
                            })?;

                            if start_val > end_val {
                                Err(Error {
                                    kind: ErrorKind::RangeStartOverEnd(
                                        start_val.to_string(),
                                        end_val.to_string(),
                                    ),
                                    range: start_pos..reader.position(),
                                })
                            } else if let Some(end) = end.checked_add(&T::Value::one()) {
                                Ok(Self(start, Some(end)))
                            } else {
                                Ok(Self(start, None))
                            }
                        } else {
                            Ok(Self(start, None))
                        }
                    }

                    Some(RANGE_OF_LENGTH) if T::LENGTH_ALLOWED => {
                        reader.seek();

                        if let Some('0'..='9') = reader.peek_char() {
                            let length: T::Value = parse_integer(reader)?;

                            if let Some(end) = start.checked_add(&length) {
                                Ok(Self(start, Some(end)))
                            } else {
                                Ok(Self(start, None))
                            }
                        } else {
                            Err(Error {
                                kind: ErrorKind::ExpectedRangeLength,
                                range: reader.position()..reader.end(),
                            })
                        }
                    }

                    _ if T::DELIMITER_REQUIRED => {
                        let position = reader.position();
                        let char = reader.read();

                        Err(Error {
                            kind: ErrorKind::ExpectedRangeDelimiter(char.map(Clone::clone)),
                            range: position..reader.position(),
                        })
                    }

                    _ => Ok(Self(start, start.checked_add(&T::Value::one()))),
                }
            }

            Some(_) | None if T::EMPTY_ALLOWED => Ok(Self(T::Value::zero(), None)),

            Some(_) => Err(Error {
                kind: ErrorKind::RangeInvalid(reader.peek_to_end().to_string()),
                range: reader.position()..reader.end(),
            }),

            None => Err(Error {
                kind: ErrorKind::ExpectedRange,
                range: reader.position()..reader.end(),
            }),
        }
    }

    fn maybe_shift(value: T::Value) -> BaseResult<T::Value> {
        if T::INDEX {
            shift_index(value)
        } else {
            Ok(value)
        }
    }

    pub fn start(&self) -> T::Value {
        self.0
    }

    pub fn end(&self) -> Option<T::Value> {
        self.1
    }

    pub fn length(&self) -> Option<T::Value> {
        self.1.map(|end| {
            let start = self.start();
            assert!(end >= start, "Range start {} >= end {}", start, end);
            end - start
        })
    }
}

#[cfg(test)]
pub mod tests {
    use test_case::test_case;

    use super::*;

    struct TestType;
    type TestValue = u32;

    impl RangeType for TestType {
        type Value = TestValue;

        const INDEX: bool = false;
        const EMPTY_ALLOWED: bool = false;
        const DELIMITER_REQUIRED: bool = false;
        const LENGTH_ALLOWED: bool = false;
    }

    #[test_case(0, None,    0 ; "from 0 to none")]
    #[test_case(1, None,    1 ; "from 1 to none")]
    #[test_case(0, Some(0), 0 ; "from 0 to 0")]
    #[test_case(0, Some(1), 0 ; "from 0 to 1")]
    #[test_case(1, Some(1), 1 ; "from 1 to 1")]
    fn start(start: TestValue, end: Option<TestValue>, result: TestValue) {
        assert_eq!(Range::<TestType>(start, end).start(), result);
    }

    #[test_case(0, None,    None    ; "from 0 to none")]
    #[test_case(1, None,    None    ; "from 1 to none")]
    #[test_case(0, Some(0), Some(0) ; "from 0 to 0")]
    #[test_case(0, Some(1), Some(1) ; "from 0 to 1")]
    #[test_case(1, Some(1), Some(1) ; "from 1 to 1")]
    fn end(start: TestValue, end: Option<TestValue>, result: Option<TestValue>) {
        assert_eq!(Range::<TestType>(start, end).end(), result);
    }

    #[test_case(0, None,    None    ; "from 0 to none")]
    #[test_case(1, None,    None    ; "from 1 to none")]
    #[test_case(0, Some(0), Some(0) ; "from 0 to 0")]
    #[test_case(0, Some(1), Some(1) ; "from 0 to 1")]
    #[test_case(1, Some(1), Some(0) ; "from 1 to 1")]
    fn length(start: TestValue, end: Option<TestValue>, result: Option<TestValue>) {
        assert_eq!(Range::<TestType>(start, end).length(), result);
    }
}
