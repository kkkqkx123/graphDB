//! Cypher表达式系统

use crate::core::types::operators::{
    BinaryOperator as CoreBinaryOperator, UnaryOperator as CoreUnaryOperator,
};
use std::collections::HashMap;

/// 表达式
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    Property(PropertyExpression),
    FunctionCall(FunctionCall),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Case(CaseExpression),
    List(ListExpression),
    Map(MapExpression),
    PatternExpression(PatternExpression),
}

/// 字面量
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

/// 属性表达式
#[derive(Debug, Clone)]
pub struct PropertyExpression {
    pub expression: Box<Expression>,
    pub property_name: String,
}

/// 函数调用
#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub function_name: String,
    pub arguments: Vec<Expression>,
}

/// 二元表达式
#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: CoreBinaryOperator,
    pub right: Box<Expression>,
}

/// 一元表达式
#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: CoreUnaryOperator,
    pub expression: Box<Expression>,
}

/// CASE表达式
#[derive(Debug, Clone)]
pub struct CaseExpression {
    pub expression: Option<Box<Expression>>,
    pub alternatives: Vec<CaseAlternative>,
    pub default_alternative: Option<Box<Expression>>,
}

/// CASE分支
#[derive(Debug, Clone)]
pub struct CaseAlternative {
    pub when_expression: Expression,
    pub then_expression: Expression,
}

/// 列表表达式
#[derive(Debug, Clone)]
pub struct ListExpression {
    pub elements: Vec<Expression>,
}

/// Map表达式
#[derive(Debug, Clone)]
pub struct MapExpression {
    pub properties: HashMap<String, Expression>,
}

/// 模式表达式
#[derive(Debug, Clone)]
pub struct PatternExpression {
    pub pattern: crate::query::parser::cypher::ast::patterns::Pattern,
}

impl Expression {
    /// 将表达式转换为值
    pub fn to_value(&self) -> crate::core::value::Value {
        use crate::core::value::Value;
        match self {
            Expression::Literal(literal) => match literal {
                Literal::String(s) => Value::String(s.clone()),
                Literal::Integer(i) => Value::Int(*i),
                Literal::Float(f) => Value::Float(*f),
                Literal::Boolean(b) => Value::Bool(*b),
                Literal::Null => Value::Null(crate::core::value::NullType::Null),
            },
            Expression::Variable(name) => Value::String(name.clone()),
            _ => Value::String(format!("{:?}", self)), // 简化处理
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_literal() {
        let expr = Expression::Literal(Literal::String("hello".to_string()));

        match expr {
            Expression::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_binary_expression() {
        let expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(5))),
            operator: CoreBinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::Integer(3))),
        });

        match expr {
            Expression::Binary(bin) => {
                assert!(matches!(
                    *bin.left,
                    Expression::Literal(Literal::Integer(5))
                ));
                assert!(matches!(
                    *bin.right,
                    Expression::Literal(Literal::Integer(3))
                ));
                assert_eq!(bin.operator, CoreBinaryOperator::Add);
            }
            _ => panic!("Expected binary expression"),
        }
    }
}
