//! Unified expression type definitions
//!
//! This module defines the unified expression type `Expression` that is used in the query engine.
//!
//! ## Design Specifications
//!
//! “Expression” is a unified type of expression that combines the characteristics of the following sources:
//! - **AST at the Parser Layer**: Provides `Span` information for error localization.
//! **Core layer expressions**: Provide serialization support and aggregate functions.
//!
//! ## Type Characteristics
//!
//! – **Location information**: The optional `Span` field is used for error reporting.
//! - **Aggregate functions**: The `Aggregate` variant is supported for aggregate queries.
//! **Serialization support**: Serialization/deserialization is supported via `serde`.
//!
//! ## Explanation of the Variants
//!
//! | Variant | Purpose |
//! |------|------|
//! | `Literal` | Literal value |
//! | `Variable` | Variable reference |
//! `Property` | Access to properties
//! `Binary` | Binary operations
//! `Unary` | Unary operation
//! | `Function` | Function Call |
//! `Aggregate` | Aggregate function
//! `List` | List literal
//! `Map` | Literal map expression
//! | `Case` | Conditional Expression |
//! `TypeCast` | Type conversion
//! **Subscript** | Access using subscripts
//! | `Range` | Range expression |
//! | `Path` | Path expression |
//! | `Label` | Label Expression |
//!
//! ## Usage Examples
//!
//! ```rust
//! use crate::core::types::expr::Expression;
//! use crate::core::types::operators::{BinaryOperator, AggregateFunction};
//! use crate::core::Value;
//!
// Simple literals
//! let expression = Expression::literal(Value::Int(42));
//!
// Binary operations
//! let sum = Expression::variable("a") + Expression::variable("b");
//!
// Aggregate functions
//! let count = Expression::aggregate(
//!     AggregateFunction::Count,
//!     Expression::variable("col"),
//!     false
//! );
//! ```
//!
//! ## Context Explanation
//!
//! This module defines pure data types, which do not contain any context.
//! The type definitions relevant to the context are defined in the `query` module.
//! - **`query::validator::context::ExpressionAnalysisContext`**: 编译时分析上下文，用于验证、优化器、类型推导等阶段
//! - **`query::executor::expression::evaluation_context::ExpressionContext`**: 运行时求值上下文 trait，用于表达式求值
//!
//! Please select the appropriate context type based on the usage scenario.

// Submodule definition
pub mod common_utils;
mod construction;
pub mod contextual;
mod def;
mod display;
pub mod expression;
mod inspection;
pub mod serializable;
mod traverse;
mod type_deduce;
pub mod utils;
pub mod visitor;
pub mod visitor_checkers;
pub mod visitor_collectors;

// Unified Export
pub use contextual::ContextualExpression;
pub use def::Expression;
pub use expression::{ExpressionId, ExpressionMeta};
pub use serializable::SerializableExpression;
pub use visitor::ExpressionVisitor;
pub use visitor_checkers::{ConstantChecker, PropertyContainsChecker};
pub use visitor_collectors::{
    FunctionCollector, OrConditionCollector, PropertyCollector, PropertyPredicate,
    PropertyPredicateCollector, VariableCollector,
};

// Re-export the tool type.
pub use common_utils::{
    extract_group_info, extract_property_refs, extract_string_from_expr,
    generate_default_alias_from_contextual, is_constant, is_constant_expression,
};
pub use utils::extract_group_suite;
pub use utils::GroupSuite;
