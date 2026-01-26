//! 统一表达式类型定义
//!
//! 本模块定义了查询引擎中使用的统一表达式类型 `Expression`。
//!
//! ## 设计说明
//!
//! `Expression` 是统一的表达式类型，结合了以下来源的特点：
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
//! use crate::core::types::expression::Expression;
//! use crate::core::types::operators::{BinaryOperator, AggregateFunction};
//! use crate::core::Value;
//!
//! // 简单字面量
//! let expression = Expression::literal(Value::Int(42));
//!
//! // 二元运算
//! let sum = Expression::add(Expression::variable("a"), Expression::variable("b"));
//!
//! // 聚合函数
//! let count = Expression::aggregate(
//!     AggregateFunction::Count,
//!     Expression::variable("col"),
//!     false
//! );
//! ```

use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
pub use crate::core::types::DataType;
use crate::core::{NullType, Value};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod expression;
pub mod utils;
pub use expression::{ExpressionId, ExpressionMeta};

pub mod visitor;
pub use visitor::{
    ExpressionDepthFirstVisitor, ExpressionTransformer, ExpressionVisitor,
    ExpressionVisitorExt, ExpressionVisitorState, VisitorError, VisitorResult,
};

/// 统一表达式类型
///
/// 包含位置信息（`span` 字段）的表达式枚举，用于：
/// - Parser 层：错误定位和报告
/// - Core 层：类型检查和执行
/// - 序列化：存储和传输
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// 字面量值
    Literal(Value),
    
    /// 变量引用
    Variable(String),
    
    /// 属性访问
    Property {
        object: Box<Expression>,
        property: String,
    },
    
    /// 二元运算
    Binary {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    
    /// 一元运算
    Unary {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    
    /// 函数调用
    Function {
        name: String,
        args: Vec<Expression>,
    },
    
    /// 聚合函数
    Aggregate {
        func: AggregateFunction,
        arg: Box<Expression>,
        distinct: bool,
    },
    
    /// 列表字面量
    List(Vec<Expression>),
    
    /// 映射字面量
    Map(Vec<(String, Expression)>),
    
    /// 条件表达式
    Case {
        conditions: Vec<(Expression, Expression)>,
        default: Option<Box<Expression>>,
    },
    
    /// 类型转换
    TypeCast {
        expression: Box<Expression>,
        target_type: DataType,
    },
    
    /// 下标访问
    Subscript {
        collection: Box<Expression>,
        index: Box<Expression>,
    },
    
    /// 范围表达式
    Range {
        collection: Box<Expression>,
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
    },
    
    /// 路径表达式
    Path(Vec<Expression>),
    
    /// 标签表达式
    Label(String),
}

impl Expression {
    pub fn literal(value: impl Into<Value>) -> Self {
        Expression::Literal(value.into())
    }

    pub fn variable(name: impl Into<String>) -> Self {
        Expression::Variable(name.into())
    }

    pub fn property(object: Expression, property: impl Into<String>) -> Self {
        Expression::Property {
            object: Box::new(object),
            property: property.into(),
        }
    }

    pub fn binary(left: Expression, op: BinaryOperator, right: Expression) -> Self {
        Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    pub fn unary(op: UnaryOperator, operand: Expression) -> Self {
        Expression::Unary {
            op,
            operand: Box::new(operand),
        }
    }

    pub fn function(name: impl Into<String>, args: Vec<Expression>) -> Self {
        Expression::Function {
            name: name.into(),
            args,
        }
    }

    pub fn aggregate(func: AggregateFunction, arg: Expression, distinct: bool) -> Self {
        Expression::Aggregate {
            func,
            arg: Box::new(arg),
            distinct,
        }
    }

    pub fn list(items: Vec<Expression>) -> Self {
        Expression::List(items)
    }

    pub fn map(pairs: Vec<(impl Into<String>, Expression)>) -> Self {
        Expression::Map(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    pub fn case(conditions: Vec<(Expression, Expression)>, default: Option<Expression>) -> Self {
        Expression::Case {
            conditions,
            default: default.map(Box::new),
        }
    }

    pub fn cast(expression: Expression, target_type: DataType) -> Self {
        Expression::TypeCast {
            expression: Box::new(expression),
            target_type,
        }
    }

    pub fn subscript(collection: Expression, index: Expression) -> Self {
        Expression::Subscript {
            collection: Box::new(collection),
            index: Box::new(index),
        }
    }

    pub fn range(
        collection: Expression,
        start: Option<Expression>,
        end: Option<Expression>,
    ) -> Self {
        Expression::Range {
            collection: Box::new(collection),
            start: start.map(Box::new),
            end: end.map(Box::new),
        }
    }

    pub fn path(items: Vec<Expression>) -> Self {
        Expression::Path(items)
    }

    pub fn label(name: impl Into<String>) -> Self {
        Expression::Label(name.into())
    }

    pub fn children(&self) -> Vec<&Expression> {
        match self {
            Expression::Literal(_) => vec![],
            Expression::Variable(_) => vec![],
            Expression::Property { object, .. } => vec![object.as_ref()],
            Expression::Binary { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            Expression::Unary { operand, .. } => vec![operand.as_ref()],
            Expression::Function { args, .. } => args.iter().collect(),
            Expression::Aggregate { arg, .. } => vec![arg.as_ref()],
            Expression::List(items) => items.iter().collect(),
            Expression::Map(pairs) => pairs.iter().map(|(_, expression)| expression).collect(),
            Expression::Case {
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
            Expression::TypeCast { expression, .. } => vec![expression.as_ref()],
            Expression::Subscript { collection, index } => {
                vec![collection.as_ref(), index.as_ref()]
            }
            Expression::Range {
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
            Expression::Path(items) => items.iter().collect(),
            Expression::Label(_) => vec![],
        }
    }

    pub fn is_constant(&self) -> bool {
        match self {
            Expression::Literal(_) => true,
            Expression::List(items) => items.iter().all(|e| e.is_constant()),
            Expression::Map(pairs) => pairs.iter().all(|(_, e)| e.is_constant()),
            _ => false,
        }
    }

    pub fn contains_aggregate(&self) -> bool {
        match self {
            Expression::Aggregate { .. } => true,
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
            Expression::Variable(name) => {
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

    pub fn is_literal(&self) -> bool {
        matches!(self, Expression::Literal(_))
    }

    pub fn as_literal(&self) -> Option<&Value> {
        match self {
            Expression::Literal(v) => Some(v),
            _ => None,
        }
    }

    pub fn is_variable(&self) -> bool {
        matches!(self, Expression::Variable(_))
    }

    pub fn as_variable(&self) -> Option<&str> {
        match self {
            Expression::Variable(name) => Some(name),
            _ => None,
        }
    }

    pub fn is_aggregate(&self) -> bool {
        matches!(self, Expression::Aggregate { .. })
    }

    pub fn to_expression_string(&self) -> String {
        match self {
            Expression::Literal(v) => format!("{:?}", v),
            Expression::Variable(name) => name.clone(),
            Expression::Property { object, property } => {
                format!("{}.{}", object.to_expression_string(), property)
            }
            Expression::Binary { left, op, right } => {
                format!("({} {} {})", left.to_expression_string(), op.name(), right.to_expression_string())
            }
            Expression::Unary { op, operand } => {
                format!("({} {})", op.name(), operand.to_expression_string())
            }
            Expression::Function { name, args } => {
                let args_str = args.iter()
                    .map(|e| e.to_expression_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}({})", name, args_str)
            }
            Expression::Aggregate { func, arg, distinct } => {
                let distinct_str = if *distinct { "DISTINCT " } else { "" };
                format!("{}({}{})", func.name(), distinct_str, arg.to_expression_string())
            }
            Expression::List(items) => {
                let items_str = items.iter()
                    .map(|e| e.to_expression_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", items_str)
            }
            Expression::Map(pairs) => {
                let pairs_str = pairs.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_expression_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", pairs_str)
            }
            Expression::Case { conditions, default } => {
                let mut result = String::from("CASE ");
                for (cond, value) in conditions {
                    result.push_str(&format!("WHEN {} THEN {} ", cond.to_expression_string(), value.to_expression_string()));
                }
                if let Some(def) = default {
                    result.push_str(&format!("ELSE {} ", def.to_expression_string()));
                }
                result.push_str("END");
                result
            }
            Expression::TypeCast { expression, target_type } => {
                format!("({} AS {:?})", expression.to_expression_string(), target_type)
            }
            Expression::Subscript { collection, index } => {
                format!("{}[{}]", collection.to_expression_string(), index.to_expression_string())
            }
            Expression::Range { collection, start, end } => {
                let start_str = start.as_ref().map(|e| e.to_expression_string()).unwrap_or_default();
                let end_str = end.as_ref().map(|e| e.to_expression_string()).unwrap_or_default();
                format!("{}[{}..{}]", collection.to_expression_string(), start_str, end_str)
            }
            Expression::Path(items) => {
                let items_str = items.iter()
                    .map(|e| e.to_expression_string())
                    .collect::<Vec<_>>()
                    .join("->");
                format!("({})", items_str)
            }
            Expression::Label(name) => format!(":{}", name),
        }
    }
}

impl Expression {
    pub fn bool(value: bool) -> Self {
        Expression::Literal(Value::Bool(value))
    }

    pub fn int(value: i64) -> Self {
        Expression::Literal(Value::Int(value))
    }

    pub fn float(value: f64) -> Self {
        Expression::Literal(Value::Float(value))
    }

    pub fn string(value: impl Into<String>) -> Self {
        Expression::Literal(Value::String(value.into()))
    }

    pub fn null() -> Self {
        Expression::Literal(Value::Null(NullType::Null))
    }

    pub fn eq(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Equal, right)
    }

    pub fn ne(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::NotEqual, right)
    }

    pub fn lt(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::LessThan, right)
    }

    pub fn le(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::LessThanOrEqual, right)
    }

    pub fn gt(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::GreaterThan, right)
    }

    pub fn ge(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::GreaterThanOrEqual, right)
    }

    pub fn add(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Add, right)
    }

    pub fn sub(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Subtract, right)
    }

    pub fn mul(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Multiply, right)
    }

    pub fn div(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Divide, right)
    }

    pub fn and(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::And, right)
    }

    pub fn or(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Or, right)
    }

    pub fn not(expression: Expression) -> Self {
        Self::unary(UnaryOperator::Not, expression)
    }

    pub fn is_null(expression: Expression) -> Self {
        Self::unary(UnaryOperator::IsNull, expression)
    }

    pub fn is_not_null(expression: Expression) -> Self {
        Self::unary(UnaryOperator::IsNotNull, expression)
    }
}

/// Arc 包装的表达式，用于共享
pub type ExprRef = Arc<Expression>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal() {
        let expression = Expression::literal(Value::Int(42));
        assert!(matches!(expression, Expression::Literal(Value::Int(42))));
    }

    #[test]
    fn test_variable() {
        let expression = Expression::variable("count");
        assert!(matches!(expression, Expression::Variable(v) if v == "count"));
    }

    #[test]
    fn test_binary() {
        let a = Expression::variable("a");
        let b = Expression::variable("b");
        let sum = Expression::add(a, b);
        assert!(matches!(sum, Expression::Binary { op: BinaryOperator::Add, .. }));
    }

    #[test]
    fn test_aggregate() {
        let expression = Expression::aggregate(AggregateFunction::Count(None), Expression::variable("col"), false);
        assert!(matches!(expression, Expression::Aggregate { func: AggregateFunction::Count(None), distinct: false, .. }));
    }

    #[test]
    fn test_serde() {
        let expression = Expression::add(Expression::int(1), Expression::int(2));
        let json = serde_json::to_string(&expression).unwrap();
        let parsed: Expression = serde_json::from_str(&json).unwrap();
        assert_eq!(expression, parsed);
    }
}
