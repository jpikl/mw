use std::io::{stdin, BufRead, Error, ErrorKind, Read, Result, Stdin};
use std::path::{Path, PathBuf};
use std::slice::Iter;

pub enum Input<'a> {
    Args {
        iter: Iter<'a, PathBuf>,
    },
    Stdin {
        buffer: Vec<u8>,
        stdin: Stdin, // TODO global lock
        delimiter: Option<u8>,
    },
}

impl<'a> Input<'a> {
    pub fn from_args(values: &'a [PathBuf]) -> Self {
        Input::Args {
            iter: values.iter(),
        }
    }

    pub fn from_stdin(delimiter: Option<u8>) -> Self {
        Input::Stdin {
            buffer: Vec::new(),
            stdin: stdin(),
            delimiter,
        }
    }

    pub fn next(&mut self) -> Result<Option<&Path>> {
        match self {
            Self::Args { iter } => Ok(iter.next().map(PathBuf::as_path)),
            Self::Stdin {
                buffer,
                stdin,
                delimiter,
            } => {
                buffer.clear();

                let mut lock = stdin.lock();
                let result = if let Some(delimiter) = delimiter {
                    lock.read_until(*delimiter, buffer)
                } else {
                    lock.read_to_end(buffer)
                };

                match result {
                    Ok(0) => Ok(None),
                    Ok(mut size) => {
                        if let Some(delimiter) = delimiter {
                            if buffer[size - 1] == *delimiter {
                                size -= 1;
                                if *delimiter == b'\n' && size > 0 && buffer[size - 1] == b'\r' {
                                    size -= 1;
                                }
                            }
                        }
                        match std::str::from_utf8(&buffer[..size]) {
                            Ok(str) => Ok(Some(Path::new(str))),
                            Err(error) => Err(Error::new(
                                ErrorKind::InvalidData,
                                format!(
                                    "Input does not have UTF-8 encoding (offset: {})",
                                    error.valid_up_to()
                                ),
                            )),
                        }
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}
