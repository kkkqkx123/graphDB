//! 表达式类型定义 V3 - 简化版本
//!
//! 移除了冗余的类型变体，统一使用核心表达式类型

use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
pub use crate::core::types::DataType;
use crate::core::{NullType, Value};
use serde::{Deserialize, Serialize};

/// 简化后的表达式类型
///
/// 移除了冗余的一元操作、属性访问和占位符类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Value),
    Variable(String),
    Property {
        object: Box<Expression>,
        property: String,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    Unary {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    Function {
        name: String,
        args: Vec<Expression>,
    },
    Aggregate {
        func: AggregateFunction,
        arg: Box<Expression>,
        distinct: bool,
    },
    List(Vec<Expression>),
    Map(Vec<(String, Expression)>),
    Case {
        conditions: Vec<(Expression, Expression)>,
        default: Option<Box<Expression>>,
    },
    TypeCast {
        expr: Box<Expression>,
        target_type: DataType,
    },
    Subscript {
        collection: Box<Expression>,
        index: Box<Expression>,
    },
    Range {
        collection: Box<Expression>,
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
    },
    Path(Vec<Expression>),
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

    pub fn cast(expr: Expression, target_type: DataType) -> Self {
        Expression::TypeCast {
            expr: Box::new(expr),
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
            Expression::Map(pairs) => pairs.iter().map(|(_, expr)| expr).collect(),
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
            Expression::TypeCast { expr, .. } => vec![expr.as_ref()],
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

    pub fn not(expr: Expression) -> Self {
        Self::unary(UnaryOperator::Not, expr)
    }

    pub fn is_null(expr: Expression) -> Self {
        Self::unary(UnaryOperator::IsNull, expr)
    }

    pub fn is_not_null(expr: Expression) -> Self {
        Self::unary(UnaryOperator::IsNotNull, expr)
    }
}
