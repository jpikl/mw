use crate::pattern::lexer::TokenType::Pipe;
use crate::pattern::source::Source;

const EXPR_START: char = '{';
const EXPR_END: char = '}';
const PIPE: char = '|';

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Raw(String),
    ExprStart,
    ExprEnd,
    Pipe,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub typ: TokenType,
    pub position: usize,
}

pub struct Lexer {
    source: Source,
    position: usize,
    character: Option<char>,
    in_expression: bool,
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if self.in_expression {
            self.next_in_expresion()
        } else {
            self.next_outside_expression()
        }
    }
}

impl Lexer {
    pub fn new(string: &str) -> Self {
        let mut lexer = Self {
            source: Source::new(string),
            position: 0,
            character: None,
            in_expression: false,
        };
        lexer.fetch_character();
        lexer
    }

    fn next_outside_expression(&mut self) -> Option<Token> {
        let mut raw = String::new();

        loop {
            match self.character {
                // '{{' is escaped '{'.
                // '}}' is escaped '}'.
                Some(ch @ EXPR_START) | Some(ch @ EXPR_END) => {
                    if self.source.peek() == self.character {
                        self.fetch_character();
                        self.fetch_character();
                        raw.push(ch);
                    } else {
                        break;
                    }
                }
                Some(ch) => {
                    raw.push(ch);
                    self.fetch_character();
                }
                None => {
                    break;
                }
            }
        }

        if !raw.is_empty() {
            return self.make_raw(raw);
        }

        match self.character {
            Some(EXPR_START) => {
                self.in_expression = true;
                self.fetch_character();
                self.make_expr_start()
            }
            Some(EXPR_END) => {
                self.fetch_character();
                self.make_expr_end()
            }
            Some(ch) => {
                // Raw token should have been returned previously!
                panic!("Unexpected character {}", ch);
            }
            None => None,
        }
    }

    fn next_in_expresion(&mut self) -> Option<Token> {
        let mut raw = String::new();

        loop {
            match self.character {
                // '|{' is escaped '{'.
                // '||' is escaped '|'.
                // '|}' is escaped '}'.
                Some(PIPE) => {
                    if let Some(ch @ EXPR_START) | Some(ch @ EXPR_END) | Some(ch @ PIPE) =
                        self.source.peek()
                    {
                        self.fetch_character();
                        self.fetch_character();
                        raw.push(ch);
                    } else {
                        break;
                    }
                }
                Some(EXPR_START) | Some(EXPR_END) | None => break,
                Some(ch) => {
                    self.fetch_character();
                    raw.push(ch);
                }
            }
        }

        if !raw.is_empty() {
            return self.make_raw(raw);
        }

        match self.character {
            Some(EXPR_START) => {
                self.fetch_character();
                self.make_expr_start()
            }
            Some(EXPR_END) => {
                self.in_expression = false;
                self.fetch_character();
                self.make_expr_end()
            }
            Some(PIPE) => {
                self.fetch_character();
                self.make_pipe()
            }
            Some(ch) => {
                // Raw token should have been returned previously!
                panic!("Unexpected character {}", ch);
            }
            None => None,
        }
    }

    fn fetch_character(&mut self) -> Option<char> {
        self.character = self.source.consume();
        self.character
    }

    fn make_raw(&mut self, raw: String) -> Option<Token> {
        self.make_token(TokenType::Raw(raw))
    }

    fn make_expr_start(&mut self) -> Option<Token> {
        self.make_token(TokenType::ExprStart)
    }

    fn make_expr_end(&mut self) -> Option<Token> {
        self.make_token(TokenType::ExprEnd)
    }

    fn make_pipe(&mut self) -> Option<Token> {
        self.make_token(TokenType::Pipe)
    }

    fn make_token(&mut self, typ: TokenType) -> Option<Token> {
        let token = Token {
            typ,
            position: self.position,
        };
        // We expect that the character for the next token is already fetched.
        self.position = self.source.position() - 1;
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct LexerTester {
        lexer: Lexer,
    }

    impl LexerTester {
        fn new(string: &str) -> Self {
            Self {
                lexer: Lexer::new(string),
            }
        }

        fn assert_none(&mut self) {
            assert_eq!(self.lexer.next(), None);
        }

        fn assert_token(&mut self, typ: TokenType, position: usize) {
            assert_eq!(self.lexer.next(), Some(Token { typ, position }));
        }

        fn assert_raw(&mut self, raw: &str, position: usize) {
            self.assert_token(TokenType::Raw(raw.to_string()), position);
        }

        fn assert_expr_start(&mut self, position: usize) {
            self.assert_token(TokenType::ExprStart, position);
        }

        fn assert_expr_end(&mut self, position: usize) {
            self.assert_token(TokenType::ExprEnd, position);
        }

        fn assert_pipe(&mut self, position: usize) {
            self.assert_token(TokenType::Pipe, position);
        }
    }

    #[test]
    fn empty_input() {
        LexerTester::new("").assert_none();
    }

    #[test]
    fn raw() {
        let mut tester = LexerTester::new("a");
        tester.assert_raw("a", 0);
        tester.assert_none();
    }

    #[test]
    fn long_raw() {
        let mut tester = LexerTester::new("abc");
        tester.assert_raw("abc", 0);
        tester.assert_none();
    }

    #[test]
    fn expression_start() {
        let mut tester = LexerTester::new("{");
        tester.assert_expr_start(0);
        tester.assert_none();
    }

    #[test]
    fn expression_end() {
        let mut tester = LexerTester::new("}");
        tester.assert_expr_end(0);
        tester.assert_none();
    }

    #[test]
    fn escaped_expression_start() {
        let mut tester = LexerTester::new("{{");
        tester.assert_raw("{", 0);
        tester.assert_none();
    }

    #[test]
    fn escaped_expression_end() {
        let mut tester = LexerTester::new("}}");
        tester.assert_raw("}", 0);
        tester.assert_none();
    }

    #[test]
    fn pipe_outside_expression() {
        let mut tester = LexerTester::new("|");
        tester.assert_raw("|", 0);
        tester.assert_none();
    }

    #[test]
    fn pipe_inside_expression() {
        let mut tester = LexerTester::new("{|");
        tester.assert_expr_start(0);
        tester.assert_pipe(1);
        tester.assert_none();
    }

    #[test]
    fn escaped_pipe_inside_expression() {
        let mut tester = LexerTester::new("{||");
        tester.assert_expr_start(0);
        tester.assert_raw("|", 1);
        tester.assert_none();
    }

    #[test]
    fn raw_inside_expression() {
        let mut tester = LexerTester::new("{a");
        tester.assert_expr_start(0);
        tester.assert_raw("a", 1);
        tester.assert_none();
    }

    #[test]
    fn long_raw_inside_expression() {
        let mut tester = LexerTester::new("{abc");
        tester.assert_expr_start(0);
        tester.assert_raw("abc", 1);
        tester.assert_none();
    }

    #[test]
    fn expression_start_inside_expression() {
        let mut tester = LexerTester::new("{ {");
        tester.assert_expr_start(0);
        tester.assert_raw(" ", 1);
        tester.assert_expr_start(2);
        tester.assert_none();
    }

    #[test]
    fn escaped_expression_start_inside_expression() {
        let mut tester = LexerTester::new("{|{");
        tester.assert_expr_start(0);
        tester.assert_raw("{", 1);
        tester.assert_none();
    }

    #[test]
    fn empty_expression() {
        let mut tester = LexerTester::new("{}");
        tester.assert_expr_start(0);
        tester.assert_expr_end(1);
        tester.assert_none();
    }

    #[test]
    fn escaped_expression_end_inside_expression() {
        let mut tester = LexerTester::new("{|}");
        tester.assert_expr_start(0);
        tester.assert_raw("}", 1);
        tester.assert_none();
    }

    #[test]
    fn expression_with_pipe() {
        let mut tester = LexerTester::new("{| }");
        tester.assert_expr_start(0);
        tester.assert_pipe(1);
        tester.assert_raw(" ", 2);
        tester.assert_expr_end(3);
        tester.assert_none();
    }

    #[test]
    fn expression_with_raw() {
        let mut tester = LexerTester::new("{a}");
        tester.assert_expr_start(0);
        tester.assert_raw("a", 1);
        tester.assert_expr_end(2);
        tester.assert_none();
    }

    #[test]
    fn expression_with_long_raw() {
        let mut tester = LexerTester::new("{abc}");
        tester.assert_expr_start(0);
        tester.assert_raw("abc", 1);
        tester.assert_expr_end(4);
        tester.assert_none();
    }

    #[test]
    fn complex_expression() {
        let mut tester = LexerTester::new("{a|bc|||def|{|}}");
        tester.assert_expr_start(0);
        tester.assert_raw("a", 1);
        tester.assert_pipe(2);
        tester.assert_raw("bc|", 3);
        tester.assert_pipe(7);
        tester.assert_raw("def{}", 8);
        tester.assert_expr_end(15);
        tester.assert_none();
    }

    #[test]
    fn complex_escaped_raw() {
        let mut tester = LexerTester::new("{{}}{{{{}}}}a{{b}}c{{{{d}}}}e{{f{{g}}h}}i}}");
        tester.assert_raw("{}{{}}a{b}c{{d}}e{f{g}h}i}", 0);
        tester.assert_none();
    }

    #[test]
    fn multiple_expressions() {
        let mut tester = LexerTester::new("{}{a}{bc}");
        tester.assert_expr_start(0);
        tester.assert_expr_end(1);
        tester.assert_expr_start(2);
        tester.assert_raw("a", 3);
        tester.assert_expr_end(4);
        tester.assert_expr_start(5);
        tester.assert_raw("bc", 6);
        tester.assert_expr_end(8);
        tester.assert_none();
    }

    #[test]
    fn multiple_raws_and_expressions() {
        let mut tester = LexerTester::new("a{}bc{de}ghi");
        tester.assert_raw("a", 0);
        tester.assert_expr_start(1);
        tester.assert_expr_end(2);
        tester.assert_raw("bc", 3);
        tester.assert_expr_start(5);
        tester.assert_raw("de", 6);
        tester.assert_expr_end(8);
        tester.assert_raw("ghi", 9);
        tester.assert_none();
    }

    #[test]
    fn multiple_escaped_raws_and_expressions() {
        let mut tester = LexerTester::new("{{}}{{{}}}");
        tester.assert_raw("{}{", 0);
        tester.assert_expr_start(6);
        tester.assert_expr_end(7);
        tester.assert_raw("}", 8);
        tester.assert_none();
    }

    #[test]
    fn complex_input() {
        let mut tester = LexerTester::new("name_{{{c}}}.{e|s1-3}");
        tester.assert_raw("name_{", 0);
        tester.assert_expr_start(7);
        tester.assert_raw("c", 8);
        tester.assert_expr_end(9);
        tester.assert_raw("}.", 10);
        tester.assert_expr_start(13);
        tester.assert_raw("e", 14);
        tester.assert_pipe(15);
        tester.assert_raw("s1-3", 16);
        tester.assert_expr_end(20);
        tester.assert_none();
    }
}
