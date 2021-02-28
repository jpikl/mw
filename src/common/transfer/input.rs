use crate::input::{Splitter, Terminator};
use crate::symbols::{DIFF_IN, DIFF_OUT};
use std::fmt;
use std::io::{BufRead, Error, ErrorKind, Result};
use std::path::PathBuf;

struct Position {
    item: usize,
    offset: usize,
}

impl Position {
    pub fn new() -> Self {
        Self { item: 1, offset: 0 }
    }

    pub fn increment(&mut self, size: usize) {
        self.item += 1;
        self.offset += size;
    }
}

impl fmt::Display for Position {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "item #{} at offset {}", self.item, self.offset)
    }
}

pub struct PathDiff<I: BufRead> {
    splitter: Splitter<I>,
    position: Position,
}

impl<I: BufRead> PathDiff<I> {
    pub fn new(input: I, terminator: Terminator) -> Self {
        Self {
            splitter: Splitter::new(input, terminator),
            position: Position::new(),
        }
    }

    pub fn read(&mut self) -> Result<Option<(PathBuf, PathBuf)>> {
        let (in_path, in_size) = match self.splitter.read()? {
            Some((value, size)) => (extract_path(value, &self.position, DIFF_IN)?, size),
            None => return Ok(None),
        };
        self.position.increment(in_size);

        let (out_path, out_size) = match self.splitter.read()? {
            Some((value, size)) => (extract_path(value, &self.position, DIFF_OUT)?, size),
            None => return Err(make_unexpected_eof_error(&self.position, DIFF_OUT)),
        };
        self.position.increment(out_size);

        Ok(Some((in_path, out_path)))
    }
}

fn extract_path(value: &str, position: &Position, prefix: char) -> Result<PathBuf> {
    if let Some(first_char) = value.chars().next() {
        if first_char == prefix {
            let path = &value[prefix.len_utf8()..];
            if path.is_empty() {
                Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("Expected a path after '{}' ({})", prefix, position),
                ))
            } else {
                Ok(PathBuf::from(path))
            }
        } else {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Expected '{}' but got '{}' ({})",
                    prefix, first_char, position
                ),
            ))
        }
    } else {
        Err(make_unexpected_eof_error(position, prefix))
    }
}

fn make_unexpected_eof_error(position: &Position, prefix: char) -> Error {
    Error::new(
        ErrorKind::UnexpectedEof,
        format!("Expected '{}' ({})", prefix, position),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position() {
        let mut position = Position::new();
        assert_eq!(position.to_string(), "item #1 at offset 0");
        position.increment(1);
        assert_eq!(position.to_string(), "item #2 at offset 1");
        position.increment(2);
        assert_eq!(position.to_string(), "item #3 at offset 3");
    }

    mod path_diff {
        use super::*;
        use crate::testing::unpack_io_error;
        use indoc::indoc;

        #[test]
        fn empty() {
            assert_eq!(
                PathDiff::new(&[][..], Terminator::Newline)
                    .read()
                    .map_err(unpack_io_error),
                Ok(None)
            );
        }

        #[test]
        fn valid() {
            let input = indoc! {"
                <abc
                >def
                < g h i 
                > j k l 
            "};
            let mut path_diff = PathDiff::new(input.as_bytes(), Terminator::Newline);
            assert_eq!(
                path_diff.read().map_err(unpack_io_error),
                Ok(Some((PathBuf::from("abc"), PathBuf::from("def"))))
            );
            assert_eq!(
                path_diff.read().map_err(unpack_io_error),
                Ok(Some((PathBuf::from(" g h i "), PathBuf::from(" j k l "))))
            );
            assert_eq!(path_diff.read().map_err(unpack_io_error), Ok(None));
        }

        #[test]
        fn invalid_in_prefix() {
            assert_eq!(
                PathDiff::new(&b"abc"[..], Terminator::Newline)
                    .read()
                    .map_err(unpack_io_error),
                Err((
                    ErrorKind::InvalidData,
                    String::from("Expected '<' but got 'a' (item #1 at offset 0)")
                ))
            )
        }

        #[test]
        fn invalid_out_prefix() {
            assert_eq!(
                PathDiff::new(&b"<abc\ndef"[..], Terminator::Newline)
                    .read()
                    .map_err(unpack_io_error),
                Err((
                    ErrorKind::InvalidData,
                    String::from("Expected '>' but got 'd' (item #2 at offset 5)")
                ))
            )
        }

        #[test]
        fn no_in_path() {
            assert_eq!(
                PathDiff::new(&b"<"[..], Terminator::Newline)
                    .read()
                    .map_err(unpack_io_error),
                Err((
                    ErrorKind::UnexpectedEof,
                    String::from("Expected a path after '<' (item #1 at offset 0)")
                ))
            )
        }

        #[test]
        fn no_out_path() {
            assert_eq!(
                PathDiff::new(&b"<abc\n>"[..], Terminator::Newline)
                    .read()
                    .map_err(unpack_io_error),
                Err((
                    ErrorKind::UnexpectedEof,
                    String::from("Expected a path after '>' (item #2 at offset 5)")
                ))
            )
        }

        #[test]
        fn no_out() {
            assert_eq!(
                PathDiff::new(&b"<abc"[..], Terminator::Newline)
                    .read()
                    .map_err(unpack_io_error),
                Err((
                    ErrorKind::UnexpectedEof,
                    String::from("Expected '>' (item #2 at offset 4)")
                ))
            )
        }

        #[test]
        fn empty_out() {
            assert_eq!(
                PathDiff::new(&b"<abc\n\n"[..], Terminator::Newline)
                    .read()
                    .map_err(unpack_io_error),
                Err((
                    ErrorKind::UnexpectedEof,
                    String::from("Expected '>' (item #2 at offset 5)")
                ))
            )
        }
    }
}
