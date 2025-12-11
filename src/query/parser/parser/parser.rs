//! Parser implementation for the query parser
//!
//! This module implements a recursive descent parser that converts tokens into AST.
//! The implementation has been split across multiple modules:
//! - statement_parser.rs: Statement parsing logic
//! - expression_parser.rs: Expression parsing logic
//! - pattern_parser.rs: Pattern matching parsing logic
//! - utils.rs: Utility functions for parsing

use crate::query::parser::lexer::lexer::Lexer;
use crate::query::parser::core::token::{Token, TokenKind};
use crate::query::parser::ast::*;
use crate::query::parser::core::error::{ParseError, ParseErrors};

pub struct Parser {
    pub lexer: Lexer,
    pub current_token: Token,
    pub errors: ParseErrors,
}