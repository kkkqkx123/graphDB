//! AST module
//!
//! This module provides an AST (Abstract Syntax Tree) design based on enumerations, which reduces the amount of样板 code and the runtime overhead.

// Definition of basic types
pub mod types;
pub use types::*;

// Statement definition
pub mod stmt;
pub use stmt::*;

// Pattern definition
pub mod pattern;
pub use pattern::*;

// Utility functions
pub mod utils;
pub use utils::*;
