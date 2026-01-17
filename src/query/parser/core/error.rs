//! Error handling for the query parser
//!
//! This module defines error types for the parsing process.

use crate::query::QueryError;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    SyntaxError,
    UnexpectedToken,
    UnterminatedString,
    UnterminatedComment,
    InvalidNumber,
    InvalidEscapeSequence,
    UnicodeEscapeError,
    UnexpectedEndOfInput,
    InvalidCharacter,
    UnknownKeyword,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub offset: Option<usize>,
    pub unexpected_token: Option<String>,
    pub expected_tokens: Vec<String>,
    pub context: Option<String>,
}

impl ParseError {
    pub fn new(
        kind: ParseErrorKind,
        message: String,
        line: usize,
        column: usize,
    ) -> Self {
        ParseError {
            kind,
            message,
            line,
            column,
            offset: None,
            unexpected_token: None,
            expected_tokens: Vec::new(),
            context: None,
        }
    }

    #[doc(hidden)]
    pub fn new_simple(message: String, line: usize, column: usize) -> Self {
        ParseError::new(ParseErrorKind::SyntaxError, message, line, column)
    }

    pub fn syntax_error<T: fmt::Display>(msg: T, line: usize, column: usize) -> ParseError {
        ParseError::new(
            ParseErrorKind::SyntaxError,
            format!("Syntax error: {}", msg),
            line,
            column,
        )
    }

    pub fn unexpected_token<T: fmt::Display>(
        token: T,
        line: usize,
        column: usize,
    ) -> ParseError {
        ParseError::new(
            ParseErrorKind::UnexpectedToken,
            format!("Unexpected token: {}", token),
            line,
            column,
        )
    }

    pub fn unterminated_string(line: usize, column: usize) -> ParseError {
        ParseError::new(
            ParseErrorKind::UnterminatedString,
            "Unterminated string literal".to_string(),
            line,
            column,
        )
    }

    pub fn unterminated_comment(line: usize, column: usize) -> ParseError {
        ParseError::new(
            ParseErrorKind::UnterminatedComment,
            "Unterminated multi-line comment".to_string(),
            line,
            column,
        )
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_unexpected_token<T: fmt::Display>(mut self, token: T) -> Self {
        self.unexpected_token = Some(token.to_string());
        self
    }

    pub fn with_expected_tokens(mut self, tokens: Vec<String>) -> Self {
        self.expected_tokens = tokens;
        self
    }

    pub fn with_context<T: fmt::Display>(mut self, context: T) -> Self {
        self.context = Some(context.to_string());
        self
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.line, self.column, self.message
        )?;

        if let Some(ref token) = self.unexpected_token {
            write!(f, "\n  Unexpected token: {}", token)?;
        }

        if !self.expected_tokens.is_empty() {
            write!(
                f,
                "\n  Expected one of: {}",
                self.expected_tokens.join(", ")
            )?;
        }

        if let Some(ref context) = self.context {
            write!(f, "\n  Context: {}", context)?;
        }

        Ok(())
    }
}

impl std::error::Error for ParseError {}

impl From<String> for ParseError {
    fn from(message: String) -> Self {
        ParseError::new(ParseErrorKind::SyntaxError, message, 0, 0)
    }
}

// Convert ParseError to the main QueryError
impl From<ParseError> for QueryError {
    fn from(parse_error: ParseError) -> Self {
        QueryError::ParseError(parse_error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseErrors {
    pub errors: Vec<ParseError>,
}

impl ParseErrors {
    pub fn new() -> Self {
        ParseErrors { errors: Vec::new() }
    }

    pub fn add(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl fmt::Display for ParseErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, error) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseErrors {}

impl From<Vec<ParseError>> for ParseErrors {
    fn from(errors: Vec<ParseError>) -> Self {
        ParseErrors { errors }
    }
}
