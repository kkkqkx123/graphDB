//! 表达式访问者模式实现
//!
//! 这个模块提供了表达式访问者模式的基础设施，专注于表达式树的遍历和转换

use crate::core::types::expression::{DataType, Expr};
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;

/// 向后兼容类型别名
pub type Expression = Expr;

pub use crate::core::expression_visitor::{
    ExpressionAcceptor, ExpressionDepthFirstVisitor, ExpressionTransformer, ExpressionVisitor,
};
