use crate::query::parser::lexer::Lexer;
use crate::query::parser::lexer::LexError;
use crate::query::parser::Token;
use crate::query::parser::core::error::{ParseError, ParseErrorKind};
use crate::core::types::{Position, Span};
use crate::query::parser::TokenKind;
use crate::query::parser::ParseErrors;

pub struct ParseContext<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    errors: ParseErrors,
    compat_mode: bool,
    recursion_depth: usize,
    max_recursion_depth: usize,
}

impl<'a> ParseContext<'a> {
    pub fn new(input: &'a str) -> Self {
        let lexer = Lexer::new(input);
        let current_token = lexer.current_token().clone();

        Self {
            lexer,
            current_token,
            errors: ParseErrors::new(),
            compat_mode: false,
            recursion_depth: 0,
            max_recursion_depth: 100,
        }
    }

    pub fn from_string(input: String) -> Self {
        let lexer = Lexer::from_string(input);
        let current_token = lexer.current_token().clone();

        Self {
            lexer,
            current_token,
            errors: ParseErrors::new(),
            compat_mode: false,
            recursion_depth: 0,
            max_recursion_depth: 100,
        }
    }

    pub fn lexer(&self) -> &Lexer<'a> {
        &self.lexer
    }

    pub fn lexer_mut(&mut self) -> &mut Lexer<'a> {
        &mut self.lexer
    }

    pub fn set_compat_mode(&mut self, enabled: bool) {
        self.compat_mode = enabled;
    }

    pub fn enter_recursion(&mut self) -> Result<(), ParseError> {
        self.recursion_depth += 1;
        if self.recursion_depth > self.max_recursion_depth {
            let pos = self.current_position();
            Err(ParseError::new(
                ParseErrorKind::SyntaxError,
                "Recursion limit exceeded".to_string(),
                pos,
            ))
        } else {
            Ok(())
        }
    }

    pub fn exit_recursion(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }

    pub fn add_error(&mut self, error: ParseError) {
        self.errors.add(error);
    }

    pub fn add_lex_error(&mut self, error: LexError) {
        self.errors.add(error.into());
    }

    pub fn errors(&self) -> &ParseErrors {
        &self.errors
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty() || self.lexer.has_errors()
    }

    pub fn take_errors(&mut self) -> ParseErrors {
        for lex_error in self.lexer.take_errors() {
            self.errors.add(lex_error.into());
        }
        std::mem::take(&mut self.errors)
    }

    pub fn current_position(&self) -> Position {
        self.lexer.current_position()
    }

    pub fn current_span(&self) -> Span {
        let pos = self.current_position();
        Span::new(pos, pos)
    }

    pub fn merge_span(&self, start: Position, end: Position) -> Span {
        Span::new(start, end)
    }

    pub fn current_token(&self) -> &Token {
        &self.current_token
    }

    pub fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    pub fn peek_token(&self) -> &Token {
        &self.current_token
    }

    pub fn match_token(&mut self, expected: TokenKind) -> bool {
        if self.current_token.kind == expected {
            self.next_token();
            true
        } else {
            false
        }
    }

    pub fn check_token(&self, expected: TokenKind) -> bool {
        self.current_token.kind == expected
    }

    pub fn expect_token(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        if self.current_token.kind == expected {
            self.next_token();
            Ok(())
        } else {
            let pos = self.current_position();
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                format!(
                    "Expected {:?}, found {:?}",
                    expected, self.current_token.kind
                ),
                pos,
            ))
        }
    }

    pub fn expect_identifier(&mut self) -> Result<String, ParseError> {
        match &self.current_token.kind {
            TokenKind::Identifier(s) => {
                let id = s.clone();
                self.next_token();
                Ok(id)
            }
            // 允许某些关键字作为标识符使用
            TokenKind::Count => {
                self.next_token();
                Ok("count".to_string())
            }
            TokenKind::Sum => {
                self.next_token();
                Ok("sum".to_string())
            }
            TokenKind::Avg => {
                self.next_token();
                Ok("avg".to_string())
            }
            TokenKind::Min => {
                self.next_token();
                Ok("min".to_string())
            }
            TokenKind::Max => {
                self.next_token();
                Ok("max".to_string())
            }
            TokenKind::Weight => {
                self.next_token();
                Ok("weight".to_string())
            }
            _ => {
                let pos = self.current_position();
                Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    format!("Expected identifier, found {:?}", self.current_token.kind),
                    pos,
                ))
            }
        }
    }

    pub fn expect_string_literal(&mut self) -> Result<String, ParseError> {
        match &self.current_token.kind {
            TokenKind::StringLiteral(s) => {
                let s = s.clone();
                self.next_token();
                Ok(s)
            }
            _ => {
                let pos = self.current_position();
                Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    format!("Expected string literal, found {:?}", self.current_token.kind),
                    pos,
                ))
            }
        }
    }

    pub fn expect_integer_literal(&mut self) -> Result<i64, ParseError> {
        match &self.current_token.kind {
            TokenKind::IntegerLiteral(n) => {
                let n = *n;
                self.next_token();
                Ok(n)
            }
            _ => {
                let pos = self.current_position();
                Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    format!("Expected integer literal, found {:?}", self.current_token.kind),
                    pos,
                ))
            }
        }
    }

    pub fn expect_float_literal(&mut self) -> Result<f64, ParseError> {
        match &self.current_token.kind {
            TokenKind::FloatLiteral(f) => {
                let f = *f;
                self.next_token();
                Ok(f)
            }
            _ => {
                let pos = self.current_position();
                Err(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    format!("Expected float literal, found {:?}", self.current_token.kind),
                    pos,
                ))
            }
        }
    }

    pub fn is_identifier_or_in_token(&self) -> bool {
        matches!(
            self.current_token.kind,
            TokenKind::Identifier(_) | TokenKind::In
        )
    }
}
