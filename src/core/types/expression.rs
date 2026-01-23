//! 统一表达式类型定义
//!
//! 本模块定义了查询引擎中使用的统一表达式类型 `Expr`。
//!
//! ## 设计说明
//!
//! `Expr` 是统一的表达式类型，结合了以下来源的特点：
//! - **Parser 层 AST**: 提供 `Span` 信息用于错误定位
//! - **Core 层表达式**: 提供序列化支持和聚合函数
//!
//! ## 类型特点
//!
//! - **位置信息**: 可选的 `Span` 字段用于错误报告
//! - **聚合函数**: 支持 `Aggregate` 变体用于聚合查询
//! - **序列化支持**: 通过 `serde` 支持序列化/反序列化
//!
//! ## 变体说明
//!
//! | 变体 | 用途 |
//! |------|------|
//! | `Literal` | 字面量值 |
//! | `Variable` | 变量引用 |
//! | `Property` | 属性访问 |
//! | `Binary` | 二元运算 |
//! | `Unary` | 一元运算 |
//! | `Function` | 函数调用 |
//! | `Aggregate` | 聚合函数 |
//! | `List` | 列表字面量 |
//! | `Map` | 映射字面量 |
//! | `Case` | 条件表达式 |
//! | `TypeCast` | 类型转换 |
//! | `Subscript` | 下标访问 |
//! | `Range` | 范围表达式 |
//! | `Path` | 路径表达式 |
//! | `Label` | 标签表达式 |
//!
//! ## 使用示例
//!
//! ```rust
//! use crate::core::types::expression::Expr;
//! use crate::core::types::operators::{BinaryOperator, AggregateFunction};
//! use crate::core::Value;
//!
//! // 简单字面量
//! let expr = Expr::literal(Value::Int(42));
//!
//! // 二元运算
//! let sum = Expr::add(Expr::variable("a"), Expr::variable("b"));
//!
//! // 聚合函数
//! let count = Expr::aggregate(
//!     AggregateFunction::Count,
//!     Expr::variable("col"),
//!     false
//! );
//! ```

use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
pub use crate::core::types::DataType;
use crate::core::{NullType, Value};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 统一表达式类型
///
/// 包含位置信息（`span` 字段）的表达式枚举，用于：
/// - Parser 层：错误定位和报告
/// - Core 层：类型检查和执行
/// - 序列化：存储和传输
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// 字面量值
    Literal(Value),
    
    /// 变量引用
    Variable(String),
    
    /// 属性访问
    Property {
        object: Box<Expr>,
        property: String,
    },
    
    /// 二元运算
    Binary {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    
    /// 一元运算
    Unary {
        op: UnaryOperator,
        operand: Box<Expr>,
    },
    
    /// 函数调用
    Function {
        name: String,
        args: Vec<Expr>,
    },
    
    /// 聚合函数
    Aggregate {
        func: AggregateFunction,
        arg: Box<Expr>,
        distinct: bool,
    },
    
    /// 列表字面量
    List(Vec<Expr>),
    
    /// 映射字面量
    Map(Vec<(String, Expr)>),
    
    /// 条件表达式
    Case {
        conditions: Vec<(Expr, Expr)>,
        default: Option<Box<Expr>>,
    },
    
    /// 类型转换
    TypeCast {
        expr: Box<Expr>,
        target_type: DataType,
    },
    
    /// 下标访问
    Subscript {
        collection: Box<Expr>,
        index: Box<Expr>,
    },
    
    /// 范围表达式
    Range {
        collection: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    
    /// 路径表达式
    Path(Vec<Expr>),
    
    /// 标签表达式
    Label(String),
}

impl Expr {
    pub fn literal(value: impl Into<Value>) -> Self {
        Expr::Literal(value.into())
    }

    pub fn variable(name: impl Into<String>) -> Self {
        Expr::Variable(name.into())
    }

    pub fn property(object: Expr, property: impl Into<String>) -> Self {
        Expr::Property {
            object: Box::new(object),
            property: property.into(),
        }
    }

    pub fn binary(left: Expr, op: BinaryOperator, right: Expr) -> Self {
        Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    pub fn unary(op: UnaryOperator, operand: Expr) -> Self {
        Expr::Unary {
            op,
            operand: Box::new(operand),
        }
    }

    pub fn function(name: impl Into<String>, args: Vec<Expr>) -> Self {
        Expr::Function {
            name: name.into(),
            args,
        }
    }

    pub fn aggregate(func: AggregateFunction, arg: Expr, distinct: bool) -> Self {
        Expr::Aggregate {
            func,
            arg: Box::new(arg),
            distinct,
        }
    }

    pub fn list(items: Vec<Expr>) -> Self {
        Expr::List(items)
    }

    pub fn map(pairs: Vec<(impl Into<String>, Expr)>) -> Self {
        Expr::Map(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    pub fn case(conditions: Vec<(Expr, Expr)>, default: Option<Expr>) -> Self {
        Expr::Case {
            conditions,
            default: default.map(Box::new),
        }
    }

    pub fn cast(expr: Expr, target_type: DataType) -> Self {
        Expr::TypeCast {
            expr: Box::new(expr),
            target_type,
        }
    }

    pub fn subscript(collection: Expr, index: Expr) -> Self {
        Expr::Subscript {
            collection: Box::new(collection),
            index: Box::new(index),
        }
    }

    pub fn range(
        collection: Expr,
        start: Option<Expr>,
        end: Option<Expr>,
    ) -> Self {
        Expr::Range {
            collection: Box::new(collection),
            start: start.map(Box::new),
            end: end.map(Box::new),
        }
    }

    pub fn path(items: Vec<Expr>) -> Self {
        Expr::Path(items)
    }

    pub fn label(name: impl Into<String>) -> Self {
        Expr::Label(name.into())
    }

    pub fn children(&self) -> Vec<&Expr> {
        match self {
            Expr::Literal(_) => vec![],
            Expr::Variable(_) => vec![],
            Expr::Property { object, .. } => vec![object.as_ref()],
            Expr::Binary { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            Expr::Unary { operand, .. } => vec![operand.as_ref()],
            Expr::Function { args, .. } => args.iter().collect(),
            Expr::Aggregate { arg, .. } => vec![arg.as_ref()],
            Expr::List(items) => items.iter().collect(),
            Expr::Map(pairs) => pairs.iter().map(|(_, expr)| expr).collect(),
            Expr::Case {
                conditions,
                default,
            } => {
                let mut children = Vec::new();
                for (cond, value) in conditions {
                    children.push(cond);
                    children.push(value);
                }
                if let Some(def) = default {
                    children.push(def);
                }
                children
            }
            Expr::TypeCast { expr, .. } => vec![expr.as_ref()],
            Expr::Subscript { collection, index } => {
                vec![collection.as_ref(), index.as_ref()]
            }
            Expr::Range {
                collection,
                start,
                end,
            } => {
                let mut children = vec![collection.as_ref()];
                if let Some(s) = start {
                    children.push(s.as_ref());
                }
                if let Some(e) = end {
                    children.push(e.as_ref());
                }
                children
            }
            Expr::Path(items) => items.iter().collect(),
            Expr::Label(_) => vec![],
        }
    }

    pub fn is_constant(&self) -> bool {
        match self {
            Expr::Literal(_) => true,
            Expr::List(items) => items.iter().all(|e| e.is_constant()),
            Expr::Map(pairs) => pairs.iter().all(|(_, e)| e.is_constant()),
            _ => false,
        }
    }

    pub fn contains_aggregate(&self) -> bool {
        match self {
            Expr::Aggregate { .. } => true,
            _ => self.children().iter().any(|e| e.contains_aggregate()),
        }
    }

    pub fn get_variables(&self) -> Vec<String> {
        let mut variables = Vec::new();
        self.collect_variables(&mut variables);
        variables.sort();
        variables.dedup();
        variables
    }

    fn collect_variables(&self, variables: &mut Vec<String>) {
        match self {
            Expr::Variable(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            }
            _ => {
                for child in self.children() {
                    child.collect_variables(variables);
                }
            }
        }
    }
}

impl Expr {
    pub fn bool(value: bool) -> Self {
        Expr::Literal(Value::Bool(value))
    }

    pub fn int(value: i64) -> Self {
        Expr::Literal(Value::Int(value))
    }

    pub fn float(value: f64) -> Self {
        Expr::Literal(Value::Float(value))
    }

    pub fn string(value: impl Into<String>) -> Self {
        Expr::Literal(Value::String(value.into()))
    }

    pub fn null() -> Self {
        Expr::Literal(Value::Null(NullType::Null))
    }

    pub fn eq(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::Equal, right)
    }

    pub fn ne(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::NotEqual, right)
    }

    pub fn lt(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::LessThan, right)
    }

    pub fn le(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::LessThanOrEqual, right)
    }

    pub fn gt(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::GreaterThan, right)
    }

    pub fn ge(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::GreaterThanOrEqual, right)
    }

    pub fn add(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::Add, right)
    }

    pub fn sub(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::Subtract, right)
    }

    pub fn mul(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::Multiply, right)
    }

    pub fn div(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::Divide, right)
    }

    pub fn and(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::And, right)
    }

    pub fn or(left: Expr, right: Expr) -> Self {
        Self::binary(left, BinaryOperator::Or, right)
    }

    pub fn not(expr: Expr) -> Self {
        Self::unary(UnaryOperator::Not, expr)
    }

    pub fn is_null(expr: Expr) -> Self {
        Self::unary(UnaryOperator::IsNull, expr)
    }

    pub fn is_not_null(expr: Expr) -> Self {
        Self::unary(UnaryOperator::IsNotNull, expr)
    }
}

/// Arc 包装的表达式，用于共享
pub type ExprRef = Arc<Expr>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let expr = Expr::literal(Value::Int(42));
        assert!(matches!(expr, Expr::Literal(Value::Int(42))));
    }

    #[test]
    fn test_variable() {
        let expr = Expr::variable("count");
        assert!(matches!(expr, Expr::Variable(v) if v == "count"));
    }

    #[test]
    fn test_binary() {
        let a = Expr::variable("a");
        let b = Expr::variable("b");
        let sum = Expr::add(a, b);
        assert!(matches!(sum, Expr::Binary { op: BinaryOperator::Add, .. }));
    }

    #[test]
    fn test_aggregate() {
        let expr = Expr::aggregate(AggregateFunction::Count(None), Expr::variable("col"), false);
        assert!(matches!(expr, Expr::Aggregate { func: AggregateFunction::Count(None), distinct: false, .. }));
    }

    #[test]
    fn test_serde() {
        let expr = Expr::add(Expr::int(1), Expr::int(2));
        let json = serde_json::to_string(&expr).unwrap();
        let parsed: Expr = serde_json::from_str(&json).unwrap();
        assert_eq!(expr, parsed);
    }
}
