pub mod expr_type;
pub mod property;
pub mod binary;
pub mod unary;
pub mod function;
pub mod container;
pub mod aggregate;
pub mod context;
pub mod evaluator;
pub mod visitor;
pub mod error;

// Re-export common types at the root level
pub use expr_type::*;
pub use context::*;
pub use error::*;
pub use evaluator::*;
pub use binary::BinaryOperator;
pub use unary::UnaryOperator;