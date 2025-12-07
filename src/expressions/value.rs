use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::core::{Value, NullType, Vertex, Edge};
use super::operations::{UnaryOp, BinaryOp};

/// The core Expression enum representing all possible expression types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    /// Constant value expression
    Constant(Value),

    /// Unary operation: op operand
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },

    /// Binary operation: left op right
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Variable reference expression
    Variable {
        name: String,
    },

    /// Property access expression (entity.property or entity[prop])
    Property {
        entity: Box<Expression>,
        property: String,
    },

    /// Function call expression
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },

    /// List container expression
    List(Vec<Expression>),

    /// Map container expression (key-value pairs)
    Map(Vec<(Expression, Expression)>),

    /// Set container expression
    Set(Vec<Expression>),

    /// Case expression (similar to if-else or switch)
    Case {
        conditions: Vec<(Expression, Expression)>, // (condition, result)
        default: Option<Box<Expression>>,
    },

    /// Graph-specific expressions
    /// Represents a vertex expression
    Vertex {
        name: String,  // Default: "VERTEX", "$^" for source vertex, "$$" for destination
    },

    /// Represents an edge expression
    Edge,

    /// Path building expression
    PathBuild {
        items: Vec<Expression>,
    },

    /// Aggregate expression (for functions like COUNT, SUM, AVG, etc.)
    Aggregate {
        name: String,      // Function name like "COUNT", "SUM", "AVG", etc.
        arg: Option<Box<Expression>>,  // Argument to the aggregation function
        distinct: bool,    // Whether to apply DISTINCT to the argument
    },

    /// List comprehension expression [expr FOR var IN collection IF condition]
    ListComprehension {
        inner_var: String,        // The variable name in the comprehension
        collection: Box<Expression>,  // The collection to iterate over
        filter: Option<Box<Expression>>,  // Optional filter condition
        mapping: Option<Box<Expression>>,  // Optional mapping expression
    },
}