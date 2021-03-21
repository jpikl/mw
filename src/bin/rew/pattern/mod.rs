use crate::pattern::filter::Filter;
use crate::pattern::parser::Parser;
use crate::pattern::parser::{Item, ParsedItem};

mod char;
mod column;
mod escape;
pub mod eval;
mod explain;
pub mod filter;
pub mod help;
mod index;
mod integer;
mod lexer;
mod number;
mod padding;
pub mod parse;
mod parser;
pub mod path;
mod range;
mod reader;
pub mod regex;
mod repetition;
mod substitution;
mod switch;
pub mod symbols;
mod uuid;

#[derive(Debug, PartialEq)]
pub struct Pattern {
    source: String,
    items: Vec<ParsedItem>,
}

impl Pattern {
    pub fn parse(source: &str, config: &parse::Config) -> parse::Result<Self> {
        Ok(Self {
            source: source.into(),
            items: Parser::new(source, config).parse_items()?,
        })
    }

    pub fn uses_local_counter(&self) -> bool {
        self.uses_filter(|filter| *filter == Filter::LocalCounter)
    }

    pub fn uses_global_counter(&self) -> bool {
        self.uses_filter(|filter| *filter == Filter::GlobalCounter)
    }

    pub fn uses_regex_capture(&self) -> bool {
        self.uses_filter(|variable| matches!(variable, Filter::RegexCapture(_)))
    }

    fn uses_filter<F: Fn(&Filter) -> bool>(&self, test: F) -> bool {
        self.items.iter().any(|item| {
            if let Item::Expression(filters) = &item.value {
                filters.iter().any(|filter| test(&filter.value))
            } else {
                false
            }
        })
    }

    pub fn eval(&self, input: &str, context: &eval::Context) -> eval::Result<String> {
        let mut output = String::new();

        for item in &self.items {
            match &item.value {
                Item::Constant(value) => output.push_str(value),
                Item::Expression(filters) => {
                    let mut value = input.to_string();

                    for filter in filters.iter() {
                        match filter.value.eval(value, context) {
                            Ok(result) => value = result,
                            Err(kind) => {
                                return Err(eval::Error {
                                    kind,
                                    value: input.to_string(),
                                    cause: &filter.value,
                                    range: &filter.range,
                                });
                            }
                        }
                    }

                    if let Some(quotes) = context.expression_quotes {
                        output.push(quotes);
                        output.push_str(&value);
                        output.push(quotes);
                    } else {
                        output.push_str(&value);
                    }
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
impl From<Vec<ParsedItem>> for Pattern {
    fn from(items: Vec<ParsedItem>) -> Self {
        Self {
            source: String::new(),
            items,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::filter::Filter;
    use super::parse::Parsed;
    use super::parser::Item;
    use super::substitution::Substitution;
    use super::Pattern;
    use crate::pattern::index::Index;
    use crate::pattern::range::Range;
    use crate::utils::AnyString;
    use test_case::test_case;

    mod parse {
        use super::super::parse::{Config, Error, ErrorKind, Parsed};
        use super::*;

        #[test]
        fn err() {
            assert_eq!(
                Pattern::parse("{", &Config::fixture()),
                Err(Error {
                    kind: ErrorKind::UnmatchedExprStart,
                    range: 0..1
                })
            )
        }

        #[test]
        fn ok() {
            assert_eq!(
                Pattern::parse("_%{{f|v}%}_", &Config::fixture()),
                Ok(Pattern {
                    source: "_%{{f|v}%}_".into(),
                    items: vec![
                        Parsed {
                            value: Item::Constant("_{".into()),
                            range: 0..3,
                        },
                        Parsed {
                            value: Item::Expression(vec![
                                Parsed {
                                    value: Filter::FileName,
                                    range: 4..5,
                                },
                                Parsed {
                                    value: Filter::ToLowercase,
                                    range: 6..7,
                                }
                            ]),
                            range: 3..8,
                        },
                        Parsed {
                            value: Item::Constant("}_".into()),
                            range: 8..11,
                        },
                    ]
                })
            )
        }
    }

    #[test_case(Filter::FileName, false, false, false; "none")]
    #[test_case(Filter::LocalCounter, true, false, false; "local counter")]
    #[test_case(Filter::GlobalCounter, false, true, false; "global counter")]
    #[test_case(Filter::RegexCapture(1), false, false, true; "regex capture")]
    fn uses(filter: Filter, local_counter: bool, global_counter: bool, regex_capture: bool) {
        let pattern = Pattern::from(vec![
            Parsed::from(Item::Constant("a".into())),
            Parsed::from(Item::Expression(vec![Parsed::from(filter)])),
        ]);
        assert_eq!(pattern.uses_local_counter(), local_counter);
        assert_eq!(pattern.uses_global_counter(), global_counter);
        assert_eq!(pattern.uses_regex_capture(), regex_capture);
    }

    mod eval {
        use super::super::eval::{Context, Error, ErrorKind};
        use super::*;

        #[test]
        fn constant() {
            let pattern = Pattern::from(vec![Parsed::from(Item::Constant("abc".into()))]);
            assert_eq!(pattern.eval("", &Context::fixture()), Ok("abc".into()));
        }

        #[test]
        fn empty_expression() {
            let pattern = Pattern::from(vec![Parsed::from(Item::Expression(vec![]))]);
            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Ok("dir/file.ext".into())
            );
        }

        #[test]
        fn single_filter_expression() {
            let pattern = Pattern::from(vec![Parsed::from(Item::Expression(vec![Parsed::from(
                Filter::FileName,
            )]))]);
            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Ok("file.ext".into())
            );
        }

        #[test]
        fn multi_filter_expression() {
            let pattern = Pattern::from(vec![Parsed::from(Item::Expression(vec![
                Parsed::from(Filter::FileName),
                Parsed::from(Filter::ToUppercase),
            ]))]);
            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Ok("FILE.EXT".into())
            );
        }

        #[test]
        fn multi_constant_and_filter_expressions() {
            let pattern = Pattern::from(vec![
                Parsed::from(Item::Constant("prefix_".into())),
                Parsed::from(Item::Expression(vec![
                    Parsed::from(Filter::BaseName),
                    Parsed::from(Filter::Substring(Range::<Index>(0, Some(3)))),
                ])),
                Parsed::from(Item::Constant("_".into())),
                Parsed::from(Item::Expression(vec![Parsed::from(Filter::LocalCounter)])),
                Parsed::from(Item::Constant("_".into())),
                Parsed::from(Item::Expression(vec![Parsed::from(Filter::GlobalCounter)])),
                Parsed::from(Item::Constant(".".into())),
                Parsed::from(Item::Expression(vec![
                    Parsed::from(Filter::Extension),
                    Parsed::from(Filter::ToUppercase),
                    Parsed::from(Filter::ReplaceAll(Substitution {
                        target: "X".into(),
                        replacement: String::new(),
                    })),
                ])),
            ]);

            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Ok("prefix_fil_1_2.ET".into())
            );
        }

        #[test]
        fn quotes() {
            let mut context = Context::fixture();
            context.expression_quotes = Some('\'');

            let pattern = Pattern::from(vec![
                Parsed::from(Item::Constant(" ".into())),
                Parsed::from(Item::Expression(Vec::new())),
                Parsed::from(Item::Constant(" ".into())),
            ]);
            assert_eq!(
                pattern.eval("dir/file.ext", &context),
                Ok(" 'dir/file.ext' ".into())
            );
        }

        #[test]
        fn failure() {
            let pattern = Pattern::from(vec![Parsed::from(Item::Expression(vec![Parsed {
                value: Filter::CanonicalPath,
                range: 1..2,
            }]))]);
            assert_eq!(
                pattern.eval("dir/file.ext", &Context::fixture()),
                Err(Error {
                    kind: ErrorKind::CanonicalizationFailed(AnyString::any()),
                    value: "dir/file.ext".into(),
                    cause: &Filter::CanonicalPath,
                    range: &(1..2usize),
                })
            );
        }
    }
}
