//! Core types for the query parser
//!
//! This module provides the fundamental types used throughout
//! the parser including tokens, errors, positions, and parsing context.

pub mod error;
pub mod token;
pub mod position;

pub use error::{ParseError, ParseErrors, ParseErrorKind};
pub use token::{Token, TokenKind, TokenKindExt};
pub use position::{Position, Span, ToSpan};
