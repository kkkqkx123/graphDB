//! Cypher表达式系统

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
    pub distinct: bool,
}

/// 二元表达式
#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

/// 二元操作符
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    And,
    Or,
    Xor,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    In,
    StartsWith,
    EndsWith,
    Contains,
    RegexMatch,
}

/// 一元表达式
#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub expression: Box<Expression>,
}

/// 一元操作符
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    Negate,
    IsNull,
    IsNotNull,
}

/// CASE表达式
#[derive(Debug, Clone)]
pub struct CaseExpression {
    pub expression: Option<Box<Expression>>,
    pub alternatives: Vec<CaseAlternative>,
    pub default: Option<Box<Expression>>,
}

/// CASE分支
#[derive(Debug, Clone)]
pub struct CaseAlternative {
    pub condition: Box<Expression>,
    pub result: Box<Expression>,
}

/// 列表表达式
#[derive(Debug, Clone)]
pub struct ListExpression {
    pub items: Vec<Expression>,
}

/// Map表达式
#[derive(Debug, Clone)]
pub struct MapExpression {
    pub entries: HashMap<String, Expression>,
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
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::Integer(3))),
        });
        
        match expr {
            Expression::Binary(bin) => {
                assert!(matches!(*bin.left, Expression::Literal(Literal::Integer(5))));
                assert!(matches!(*bin.right, Expression::Literal(Literal::Integer(3))));
                assert_eq!(bin.operator, BinaryOperator::Add);
            }
            _ => panic!("Expected binary expression"),
        }
    }
}