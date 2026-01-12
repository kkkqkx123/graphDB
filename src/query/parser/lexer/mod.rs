pub mod lexer;

pub use crate::query::parser::{Token, TokenKind};
pub use lexer::Lexer;

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
}

impl LexError {
    pub fn new(message: String) -> Self {
        LexError { message }
    }
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lex error: {}", self.message)
    }
}

impl std::error::Error for LexError {}
