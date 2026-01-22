//! Error handling for the query parser
//!
//! This module defines error types for the parsing process,
//! providing unified error reporting with position information,
//! hints, and context support.

use crate::query::QueryError;
use std::fmt;
use std::error::Error;

use super::position::Position;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    LexicalError,
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
    RecursionLimitExceeded,
    UnsupportedFeature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub position: Position,
    pub offset: Option<usize>,
    pub unexpected_token: Option<String>,
    pub expected_tokens: Vec<String>,
    pub context: Option<Box<dyn Error + Send + Sync>>,
    pub hints: Vec<String>,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, message: String, position: Position) -> Self {
        ParseError {
            kind,
            message,
            position,
            offset: None,
            unexpected_token: None,
            expected_tokens: Vec::new(),
            context: None,
            hints: Vec::new(),
        }
    }

    pub fn new_simple(message: String, position: Position) -> Self {
        ParseError::new(ParseErrorKind::SyntaxError, message, position)
    }

    pub fn syntax_error<T: fmt::Display>(msg: T, position: Position) -> ParseError {
        ParseError::new(
            ParseErrorKind::SyntaxError,
            format!("Syntax error: {}", msg),
            position,
        )
    }

    pub fn unexpected_token<T: fmt::Display>(
        token: T,
        position: Position,
    ) -> ParseError {
        ParseError::new(
            ParseErrorKind::UnexpectedToken,
            format!("Unexpected token: {}", token),
            position,
        )
    }

    pub fn unterminated_string(position: Position) -> ParseError {
        ParseError::new(
            ParseErrorKind::UnterminatedString,
            "Unterminated string literal".to_string(),
            position,
        )
    }

    pub fn unterminated_comment(position: Position) -> ParseError {
        ParseError::new(
            ParseErrorKind::UnterminatedComment,
            "Unterminated multi-line comment".to_string(),
            position,
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

    pub fn with_context<E: Error + Send + Sync + 'static>(mut self, context: E) -> Self {
        self.context = Some(Box::new(context));
        self
    }

    pub fn with_hint(mut self, hint: String) -> Self {
        self.hints.push(hint);
        self
    }

    pub fn with_hints(mut self, hints: Vec<String>) -> Self {
        self.hints.extend(hints);
        self
    }

    pub fn add_hint(&mut self, hint: String) {
        self.hints.push(hint);
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.position.line, self.position.column, self.message
        )?;

        if let Some(ref token) = self.unexpected_token {
            writeln!(f, "\n  Unexpected token: {}", token)?;
        }

        if !self.expected_tokens.is_empty() {
            writeln!(f, "\n  Expected one of: {}", self.expected_tokens.join(", "))?;
        }

        if let Some(ref context) = self.context {
            writeln!(f, "\n  Context: {}", context)?;
        }

        if !self.hints.is_empty() {
            writeln!(f, "\n  Hint(s):")?;
            for hint in &self.hints {
                writeln!(f, "    - {}", hint)?;
            }
        }

        Ok(())
    }
}

impl Error for ParseError {}

impl From<String> for ParseError {
    fn from(message: String) -> Self {
        ParseError::new(
            ParseErrorKind::SyntaxError,
            message,
            Position::new(0, 0),
        )
    }
}

impl From<super::lexer::LexError> for ParseError {
    fn from(lex_error: super::lexer::LexError) -> Self {
        ParseError::new(
            ParseErrorKind::LexicalError,
            lex_error.message,
            lex_error.position,
        )
    }
}

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
        ParseErrors {
            errors: Vec::new(),
        }
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

    pub fn push(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    pub fn extend(&mut self, errors: &mut ParseErrors) {
        self.errors.extend(errors.errors.drain(..));
    }

    pub fn take(&mut self) -> Vec<ParseError> {
        std::mem::take(&mut self.errors)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ParseError> {
        self.errors.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = ParseError> {
        self.errors.into_iter()
    }
}

impl Default for ParseErrors {
    fn default() -> Self {
        ParseErrors::new()
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

impl Error for ParseErrors {}

impl From<Vec<ParseError>> for ParseErrors {
    fn from(errors: Vec<ParseError>) -> Self {
        ParseErrors { errors }
    }
}

impl From<ParseErrors> for QueryError {
    fn from(parse_errors: ParseErrors) -> Self {
        QueryError::ParseError(parse_errors.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let error = ParseError::unexpected_token(
            "IDENTIFIER",
            Position::new(10, 5),
        );
        let display = error.to_string();
        assert!(display.contains("line 10, column 5"));
        assert!(display.contains("Unexpected token: IDENTIFIER"));
    }

    #[test]
    fn test_parse_error_with_hint() {
        let error = ParseError::syntax_error(
            "invalid syntax",
            Position::new(5, 10),
        ).with_hint("Try adding a semicolon at the end".to_string());

        let display = error.to_string();
        assert!(display.contains("Hint"));
        assert!(display.contains("semicolon"));
    }

    #[test]
    fn test_parse_error_with_context() {
        let context_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error = ParseError::syntax_error(
            "error",
            Position::new(1, 1),
        ).with_context(context_error);

        let display = error.to_string();
        assert!(display.contains("Context"));
    }
}
