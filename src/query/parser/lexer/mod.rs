pub mod lexer;

pub use crate::query::parser::{Token, TokenKind};
pub use lexer::Lexer;

use crate::query::parser::core::position::Position;

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub position: Position,
}

impl LexError {
    pub fn new(message: String, position: Position) -> Self {
        LexError { message, position }
    }

    pub fn unterminated_string(position: Position) -> Self {
        LexError::new("Unterminated string literal".to_string(), position)
    }

    pub fn unterminated_comment(position: Position) -> Self {
        LexError::new("Unterminated multi-line comment".to_string(), position)
    }

    pub fn invalid_number(message: String, position: Position) -> Self {
        LexError::new(format!("Invalid number: {}", message), position)
    }

    pub fn invalid_escape_sequence(sequence: String, position: Position) -> Self {
        LexError::new(
            format!("Invalid escape sequence: \\{}", sequence),
            position,
        )
    }

    pub fn unexpected_character(ch: char, position: Position) -> Self {
        LexError::new(
            format!("Unexpected character: '{}'", ch),
            position,
        )
    }
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lex error at line {}, column {}: {}",
            self.position.line, self.position.column, self.message
        )
    }
}

impl std::error::Error for LexError {}
