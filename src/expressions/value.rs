use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::core::{Value, NullType, Vertex, Edge};
use super::operations::{UnaryOp, BinaryOp};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionKind {
    Constant,
    Unary,
    Binary,
    Variable,
    Property,
    FunctionCall,
    List,
    Map,
    Set,
    Case,
    Vertex,
    Edge,
    PathBuild,
    Aggregate,
    ListComprehension,
}

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

impl Expression {
    /// Returns the kind of the expression
    pub fn kind(&self) -> ExpressionKind {
        match self {
            Expression::Constant(_) => ExpressionKind::Constant,
            Expression::Unary { .. } => ExpressionKind::Unary,
            Expression::Binary { .. } => ExpressionKind::Binary,
            Expression::Variable { .. } => ExpressionKind::Variable,
            Expression::Property { .. } => ExpressionKind::Property,
            Expression::FunctionCall { .. } => ExpressionKind::FunctionCall,
            Expression::List(_) => ExpressionKind::List,
            Expression::Map(_) => ExpressionKind::Map,
            Expression::Set(_) => ExpressionKind::Set,
            Expression::Case { .. } => ExpressionKind::Case,
            Expression::Vertex { .. } => ExpressionKind::Vertex,
            Expression::Edge => ExpressionKind::Edge,
            Expression::PathBuild { .. } => ExpressionKind::PathBuild,
            Expression::Aggregate { .. } => ExpressionKind::Aggregate,
            Expression::ListComprehension { .. } => ExpressionKind::ListComprehension,
        }
    }

    /// Returns a vector of child expressions
    pub fn children(&self) -> Vec<&Expression> {
        match self {
            Expression::Constant(_) => vec![],
            Expression::Unary { operand, .. } => vec![operand],
            Expression::Binary { left, right, .. } => vec![left, right],
            Expression::Variable { .. } => vec![],
            Expression::Property { entity, .. } => vec![entity],
            Expression::FunctionCall { args, .. } => args.iter().collect(),
            Expression::List(exprs) => exprs.iter().collect(),
            Expression::Map(pairs) => {
                let mut children = Vec::new();
                for (k, v) in pairs {
                    children.push(k);
                    children.push(v);
                }
                children
            },
            Expression::Set(exprs) => exprs.iter().collect(),
            Expression::Case { conditions, default } => {
                let mut children = Vec::new();
                for (cond, result) in conditions {
                    children.push(cond);
                    children.push(result);
                }
                if let Some(default_expr) = default {
                    children.push(default_expr);
                }
                children
            },
            Expression::Vertex { .. } => vec![],
            Expression::Edge => vec![],
            Expression::PathBuild { items } => items.iter().collect(),
            Expression::Aggregate { arg, .. } => {
                if let Some(arg_expr) = arg {
                    vec![arg_expr]
                } else {
                    vec![]
                }
            },
            Expression::ListComprehension { collection, filter, mapping, .. } => {
                let mut children = vec![collection.as_ref()];
                if let Some(filter_expr) = filter {
                    children.push(filter_expr);
                }
                if let Some(mapping_expr) = mapping {
                    children.push(mapping_expr);
                }
                children
            },
        }
    }
}