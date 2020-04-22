use crate::pattern::eval::{EvalContext, EvalErrorKind};
use crate::pattern::number::parse_usize;
use crate::pattern::parse::{ParseError, ParseErrorKind, ParseResult};
use crate::pattern::reader::Reader;
use std::ffi::OsStr;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum Variable {
    Filename,
    Basename,
    Extension,
    ExtensionWithDot,
    FullDirname,
    ParentDirname,
    FullPath,
    LocalCounter,
    GlobalCounter,
    RegexCapture(usize),
    Uuid,
}

impl Variable {
    pub fn parse(reader: &mut Reader) -> ParseResult<Self> {
        let position = reader.position();

        if let Some('0'..='9') = reader.peek_value() {
            let number = parse_usize(reader)?;
            if number > 0 {
                Ok(Variable::RegexCapture(number))
            } else {
                Err(ParseError {
                    kind: ParseErrorKind::RegexCaptureZero,
                    start: position,
                    end: reader.position(),
                })
            }
        } else if let Some(char) = reader.read() {
            match char.value() {
                'f' => Ok(Variable::Filename),
                'b' => Ok(Variable::Basename),
                'e' => Ok(Variable::Extension),
                'E' => Ok(Variable::ExtensionWithDot),
                'd' => Ok(Variable::FullDirname),
                'D' => Ok(Variable::ParentDirname),
                'p' => Ok(Variable::FullPath),
                'c' => Ok(Variable::LocalCounter),
                'C' => Ok(Variable::GlobalCounter),
                'u' => Ok(Variable::Uuid),
                _ => Err(ParseError {
                    kind: ParseErrorKind::UnknownVariable(char.clone()),
                    start: position,
                    end: reader.position(),
                }),
            }
        } else {
            Err(ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                start: position,
                end: reader.end(),
            })
        }
    }

    pub fn eval(&self, context: &EvalContext) -> Result<String, EvalErrorKind> {
        match self {
            Variable::Filename => Ok(context
                .path
                .file_name()
                .map_or_else(String::new, os_str_to_string)),

            Variable::Basename => Ok(context
                .path
                .file_stem()
                .map_or_else(String::new, os_str_to_string)),

            Variable::Extension => Ok(context
                .path
                .extension()
                .map_or_else(String::new, os_str_to_string)),

            Variable::ExtensionWithDot => {
                Ok(context.path.extension().map_or_else(String::new, |s| {
                    let mut string = os_str_to_string(s);
                    string.insert(0, '.');
                    string
                }))
            }

            Variable::FullDirname => Ok(context
                .path
                .parent()
                .map(Path::as_os_str)
                .map_or_else(String::new, os_str_to_string)),

            Variable::ParentDirname => Ok(context
                .path
                .parent()
                .and_then(Path::file_name)
                .map_or_else(String::new, os_str_to_string)),

            Variable::FullPath => Ok(os_str_to_string(context.path.as_os_str())),
            Variable::LocalCounter => Ok(context.local_counter.to_string()),
            Variable::GlobalCounter => Ok(context.global_counter.to_string()),

            Variable::RegexCapture(index) => Ok(context
                .regex_captures
                .as_ref()
                .and_then(|captures| captures.get(*index))
                .map(|r#match| r#match.as_str())
                .map_or_else(String::new, String::from)),

            Variable::Uuid => {
                let mut buffer = Uuid::encode_buffer();
                let str = Uuid::new_v4().to_hyphenated().encode_lower(&mut buffer);
                Ok((*str).to_string())
            }
        }
    }
}

fn os_str_to_string(str: &OsStr) -> String {
    // TODO return error instead of lossy conversion
    str.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::char::Char;
    use regex::Regex;
    use std::path::Path;

    #[test]
    fn parse_filename() {
        assert_ok("f", Variable::Filename);
    }

    #[test]
    fn parse_basename() {
        assert_ok("b", Variable::Basename);
    }

    #[test]
    fn parse_extension() {
        assert_ok("e", Variable::Extension);
    }

    #[test]
    fn parse_extension_with_dot() {
        assert_ok("E", Variable::ExtensionWithDot);
    }

    #[test]
    fn parse_full_dirname() {
        assert_ok("d", Variable::FullDirname);
    }

    #[test]
    fn parse_parent_dirname() {
        assert_ok("D", Variable::ParentDirname);
    }

    #[test]
    fn parse_full_path() {
        assert_ok("p", Variable::FullPath);
    }

    #[test]
    fn parse_local_counter() {
        assert_ok("c", Variable::LocalCounter);
    }

    #[test]
    fn parse_global_counter() {
        assert_ok("C", Variable::GlobalCounter);
    }

    #[test]
    fn parse_regex_capture() {
        assert_ok("1", Variable::RegexCapture(1));
        assert_ok("2", Variable::RegexCapture(2));
        assert_ok("3", Variable::RegexCapture(3));
        assert_ok("4", Variable::RegexCapture(4));
        assert_ok("5", Variable::RegexCapture(5));
        assert_ok("6", Variable::RegexCapture(6));
        assert_ok("7", Variable::RegexCapture(7));
        assert_ok("8", Variable::RegexCapture(8));
        assert_ok("9", Variable::RegexCapture(9));
        assert_ok("10", Variable::RegexCapture(10));
    }

    #[test]
    fn parse_uuid() {
        assert_ok("u", Variable::Uuid);
    }

    #[test]
    fn parse_ignore_remaning_chars_after_variable() {
        let mut reader = Reader::from("f_");
        Variable::parse(&mut reader).unwrap();
        assert_eq!(reader.position(), 1);
    }

    #[test]
    fn parse_ignore_remaning_chars_capture_group_variable() {
        let mut reader = Reader::from("123_");
        Variable::parse(&mut reader).unwrap();
        assert_eq!(reader.position(), 3);
    }

    #[test]
    fn parse_empty_error() {
        assert_err(
            "",
            ParseError {
                kind: ParseErrorKind::ExpectedVariable,
                start: 0,
                end: 0,
            },
        )
    }

    #[test]
    fn parse_unknown_variable_error() {
        assert_err(
            "-_",
            ParseError {
                kind: ParseErrorKind::UnknownVariable(Char::Raw('-')),
                start: 0,
                end: 1,
            },
        );
    }

    // TODO replace by inline assert_eq!
    fn assert_ok(string: &str, variable: Variable) {
        assert_eq!(Variable::parse(&mut Reader::from(string)), Ok(variable));
    }

    // TODO replace by inline assert_eq!
    fn assert_err(string: &str, error: ParseError) {
        assert_eq!(Variable::parse(&mut Reader::from(string)), Err(error));
    }

    #[test]
    fn eval_filename() {
        assert_eq!(
            Variable::Filename.eval(&make_context()),
            Ok("file.ext".to_string())
        );
    }

    #[test]
    fn eval_basename() {
        assert_eq!(
            Variable::Basename.eval(&make_context()),
            Ok("file".to_string())
        );
    }

    #[test]
    fn eval_extension() {
        assert_eq!(
            Variable::Extension.eval(&make_context()),
            Ok("ext".to_string())
        );
    }

    #[test]
    fn eval_extension_no_ext() {
        let mut context = make_context();
        context.path = Path::new("root/parent/file");
        assert_eq!(Variable::Extension.eval(&context), Ok("".to_string()));
    }

    #[test]
    fn eval_extension_with_dot() {
        assert_eq!(
            Variable::ExtensionWithDot.eval(&make_context()),
            Ok(".ext".to_string())
        );
    }

    #[test]
    fn eval_extension_with_dot_no_ext() {
        let mut context = make_context();
        context.path = Path::new("root/parent/file");
        assert_eq!(
            Variable::ExtensionWithDot.eval(&context),
            Ok("".to_string())
        );
    }

    #[test]
    fn eval_full_dirname() {
        assert_eq!(
            Variable::FullDirname.eval(&make_context()),
            Ok("root/parent".to_string())
        );
    }

    #[test]
    fn eval_full_dirname_no_parent() {
        let mut context = make_context();
        context.path = Path::new("file.ext");
        assert_eq!(Variable::FullDirname.eval(&context), Ok(String::new()));
    }

    #[test]
    fn eval_parent_dirname() {
        assert_eq!(
            Variable::ParentDirname.eval(&make_context()),
            Ok("parent".to_string())
        );
    }

    #[test]
    fn eval_parent_dirname_no_parent() {
        let mut context = make_context();
        context.path = Path::new("file.ext");
        assert_eq!(Variable::ParentDirname.eval(&context), Ok(String::new()));
    }

    #[test]
    fn eval_full_path() {
        assert_eq!(
            Variable::FullPath.eval(&make_context()),
            Ok("root/parent/file.ext".to_string())
        );
    }

    #[test]
    fn eval_local_counter() {
        assert_eq!(
            Variable::LocalCounter.eval(&make_context()),
            Ok("1".to_string())
        );
    }

    #[test]
    fn eval_global_counter() {
        assert_eq!(
            Variable::GlobalCounter.eval(&make_context()),
            Ok("2".to_string())
        );
    }

    #[test]
    fn eval_regex_capture() {
        assert_eq!(
            Variable::RegexCapture(1).eval(&make_context()),
            Ok("abc".to_string())
        );
    }

    #[test]
    fn eval_regex_capture_overflow() {
        assert_eq!(
            Variable::RegexCapture(2).eval(&make_context()),
            Ok(String::new())
        );
    }

    #[test]
    fn eval_uuid() {
        let uuid = Variable::Uuid.eval(&make_context()).unwrap();
        let uuid_regex =
            Regex::new("^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$")
                .unwrap();
        assert!(uuid_regex.is_match(&uuid), format!("{} is UUID v4", uuid));
    }

    fn make_context<'a>() -> EvalContext<'a> {
        EvalContext {
            path: Path::new("root/parent/file.ext"),
            local_counter: 1,
            global_counter: 2,
            regex_captures: Regex::new("(.*)").unwrap().captures("abc"),
        }
    }
}