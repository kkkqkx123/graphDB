//! Expression AST definitions for the query parser

use crate::core::Value;
use super::types::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Constant(Value),
    Variable(Identifier),
    FunctionCall(FunctionCall),
    PropertyAccess(Box<Expression>, Identifier),
    AttributeAccess(Box<Expression>, Identifier), // e.g., tagName.propertyName
    Arithmetic(Box<Expression>, ArithmeticOp, Box<Expression>),
    Logical(Box<Expression>, LogicalOp, Box<Expression>),
    Relational(Box<Expression>, RelationalOp, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    List(Vec<Expression>),
    Map(Vec<(Identifier, Expression)>),
    Subscript(Box<Expression>, Box<Expression>), // expr[index] or expr.key
    Case(CaseExpression),
    InList(Box<Expression>, Vec<Expression>),
    NotInList(Box<Expression>, Vec<Expression>),
    Contains(Box<Expression>, Box<Expression>),
    StartsWith(Box<Expression>, Box<Expression>),
    EndsWith(Box<Expression>, Box<Expression>),
    IsNull(Box<Expression>),
    IsNotNull(Box<Expression>),
    All(Box<Expression>, Box<Expression>), // For list predicates
    Single(Box<Expression>, Box<Expression>),
    Any(Box<Expression>, Box<Expression>),
    None(Box<Expression>, Box<Expression>),
    // Add more expression types as needed
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    And,
    Or,
    Xor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationalOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Regex,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: Identifier,
    pub args: Vec<Expression>,
    pub distinct: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseExpression {
    pub match_expr: Option<Box<Expression>>,
    pub when_then_pairs: Vec<(Expression, Expression)>,
    pub default: Option<Box<Expression>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_expression_structures() {
        let expr = Expression::Arithmetic(
            Box::new(Expression::Constant(Value::Int(5))),
            ArithmeticOp::Add,
            Box::new(Expression::Constant(Value::Int(3))),
        );
        
        assert!(matches!(expr, Expression::Arithmetic(_, ArithmeticOp::Add, _)));
    }
}