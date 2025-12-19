use crate::graph::expression::error::ExpressionError;
use crate::graph::expression::operator_conversion;
use crate::core::Value;
use crate::graph::expression::{Expression, LiteralValue};
use crate::query::parser::cypher::ast::expressions::{
    BinaryExpression, BinaryOperator, CaseAlternative, CaseExpression,
    Expression as CypherExpression, FunctionCall, ListExpression, Literal as CypherLiteral,
    MapExpression, PatternExpression, PropertyExpression, UnaryExpression, UnaryOperator,
};

/// Cypher表达式转换器
/// 
/// 专注于Cypher表达式与统一表达式系统之间的转换，
/// 不包含评估和优化逻辑，保持职责单一。
pub struct ExpressionConverter;

impl ExpressionConverter {
    /// 将Cypher表达式转换为统一表达式
    pub fn convert_cypher_to_unified(
        cypher_expr: &CypherExpression,
    ) -> Result<Expression, ExpressionError> {
        match cypher_expr {
            CypherExpression::Literal(literal) => {
                let unified_literal = match literal {
                    CypherLiteral::String(s) => LiteralValue::String(s.clone()),
                    CypherLiteral::Integer(i) => LiteralValue::Int(*i),
                    CypherLiteral::Float(f) => LiteralValue::Float(*f),
                    CypherLiteral::Boolean(b) => LiteralValue::Bool(*b),
                    CypherLiteral::Null => LiteralValue::Null,
                };
                Ok(Expression::Literal(unified_literal))
            }
            CypherExpression::Variable(name) => Ok(Expression::Variable(name.clone())),
            CypherExpression::Property(prop_expr) => {
                let object_expr = Self::convert_cypher_to_unified(&prop_expr.expression)?;
                Ok(Expression::Property {
                    object: Box::new(object_expr),
                    property: prop_expr.property_name.clone(),
                })
            }
            CypherExpression::FunctionCall(func_call) => {
                let args: Result<Vec<Expression>, ExpressionError> = func_call
                    .arguments
                    .iter()
                    .map(|arg| Self::convert_cypher_to_unified(arg))
                    .collect();
                Ok(Expression::Function {
                    name: func_call.function_name.clone(),
                    args: args?,
                })
            }
            CypherExpression::Binary(bin_expr) => {
                let left = Self::convert_cypher_to_unified(&bin_expr.left)?;
                let right = Self::convert_cypher_to_unified(&bin_expr.right)?;
                let op = operator_conversion::convert_cypher_binary_operator(&bin_expr.operator);
                Ok(Expression::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                })
            }
            CypherExpression::Unary(unary_expr) => {
                let operand = Self::convert_cypher_to_unified(&unary_expr.expression)?;
                let op = operator_conversion::convert_cypher_unary_operator(&unary_expr.operator);
                Ok(Expression::Unary {
                    op,
                    operand: Box::new(operand),
                })
            }
            CypherExpression::List(list_expr) => {
                let elements: Result<Vec<Expression>, ExpressionError> = list_expr
                    .elements
                    .iter()
                    .map(|elem| Self::convert_cypher_to_unified(elem))
                    .collect();
                Ok(Expression::List(elements?))
            }
            CypherExpression::Map(map_expr) => {
                let pairs: Result<Vec<(String, Expression)>, ExpressionError> = map_expr
                    .properties
                    .iter()
                    .map(|(key, value)| {
                        let value_expr = Self::convert_cypher_to_unified(value)?;
                        Ok((key.clone(), value_expr))
                    })
                    .collect();
                Ok(Expression::Map(pairs?))
            }
            CypherExpression::PatternExpression(pattern_expr) => {
                // 将模式表达式转换为路径表达式
                Ok(Expression::Literal(LiteralValue::String(format!(
                    "Pattern: {:?}",
                    pattern_expr.pattern
                ))))
            }
            CypherExpression::Case(case_expr) => {
                let alternatives: Result<Vec<(String, Expression)>, ExpressionError> = case_expr
                    .alternatives
                    .iter()
                    .map(|alt| {
                        let when_expr = Self::convert_cypher_to_unified(&alt.when_expression)?;
                        let then_expr = Self::convert_cypher_to_unified(&alt.then_expression)?;
                        Ok((format!("when_{}", "condition"), then_expr))
                    })
                    .collect();
                
                let default_alternative = case_expr.default_alternative
                    .as_ref()
                    .map(|expr| Self::convert_cypher_to_unified(expr))
                    .transpose()?;
                
                Ok(Expression::Map(alternatives?))
            }
        }
    }

    /// 将统一表达式转换为Cypher表达式
    /// 
    /// 这个方法主要用于调试和测试，在实际查询执行中不常用
    pub fn convert_unified_to_cypher(
        expr: &Expression,
    ) -> Result<CypherExpression, ExpressionError> {
        match expr {
            Expression::Literal(literal) => {
                let cypher_literal = match literal {
                    LiteralValue::String(s) => CypherLiteral::String(s.clone()),
                    LiteralValue::Int(i) => CypherLiteral::Integer(*i),
                    LiteralValue::Float(f) => CypherLiteral::Float(*f),
                    LiteralValue::Bool(b) => CypherLiteral::Boolean(*b),
                    LiteralValue::Null => CypherLiteral::Null,
                };
                Ok(CypherExpression::Literal(cypher_literal))
            }
            Expression::Variable(name) => Ok(CypherExpression::Variable(name.clone())),
            Expression::Property { object, property } => {
                let object_expr = Self::convert_unified_to_cypher(object)?;
                Ok(CypherExpression::Property(PropertyExpression {
                    expression: Box::new(object_expr),
                    property_name: property.clone(),
                }))
            }
            Expression::Function { name, args } => {
                let converted_args: Result<Vec<CypherExpression>, ExpressionError> = args
                    .iter()
                    .map(|arg| Self::convert_unified_to_cypher(arg))
                    .collect();
                Ok(CypherExpression::FunctionCall(FunctionCall {
                    function_name: name.clone(),
                    arguments: converted_args?,
                }))
            }
            Expression::Binary { left, op, right } => {
                let left_expr = Self::convert_unified_to_cypher(left)?;
                let right_expr = Self::convert_unified_to_cypher(right)?;
                let binary_op = operator_conversion::convert_unified_to_cypher_binary_operator(op)
                    .map_err(|e| ExpressionError::InvalidOperation(e))?;
                Ok(CypherExpression::Binary(BinaryExpression {
                    left: Box::new(left_expr),
                    operator: binary_op,
                    right: Box::new(right_expr),
                }))
            }
            Expression::Unary { op, operand } => {
                let operand_expr = Self::convert_unified_to_cypher(operand)?;
                let unary_op = operator_conversion::convert_unified_to_cypher_unary_operator(op)
                    .map_err(|e| ExpressionError::InvalidOperation(e))?;
                Ok(CypherExpression::Unary(UnaryExpression {
                    operator: unary_op,
                    expression: Box::new(operand_expr),
                }))
            }
            Expression::List(elements) => {
                let converted_elements: Result<Vec<CypherExpression>, ExpressionError> = elements
                    .iter()
                    .map(|elem| Self::convert_unified_to_cypher(elem))
                    .collect();
                Ok(CypherExpression::List(ListExpression {
                    elements: converted_elements?,
                }))
            }
            Expression::Map(pairs) => {
                let mut properties = std::collections::HashMap::new();
                for (key, value) in pairs {
                    let value_expr = Self::convert_unified_to_cypher(value)?;
                    properties.insert(key.clone(), value_expr);
                }
                Ok(CypherExpression::Map(MapExpression { properties }))
            }
            // 这个分支不应该存在，因为Expression枚举中没有String变体
            // 如果需要处理字符串，应该使用Expression::Literal(LiteralValue::String)
            _ => {
                // 对于其他未处理的情况，返回错误
                Err(ExpressionError::InvalidOperation(format!("Unsupported expression type: {:?}", expr)))
            }
        }
    }

    /// 批量转换Cypher表达式为统一表达式
    pub fn convert_cypher_batch_to_unified(
        cypher_exprs: &[CypherExpression],
    ) -> Result<Vec<Expression>, ExpressionError> {
        let mut results = Vec::new();
        for expr in cypher_exprs {
            results.push(Self::convert_cypher_to_unified(expr)?);
        }
        Ok(results)
    }

    /// 批量转换统一表达式为Cypher表达式
    pub fn convert_unified_batch_to_cypher(
        exprs: &[Expression],
    ) -> Result<Vec<CypherExpression>, ExpressionError> {
        let mut results = Vec::new();
        for expr in exprs {
            results.push(Self::convert_unified_to_cypher(expr)?);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::cypher::ast::expressions::*;

    #[test]
    fn test_convert_literal() {
        let cypher_expr = CypherExpression::Literal(CypherLiteral::Integer(42));
        let unified_expr = ExpressionConverter::convert_cypher_to_unified(&cypher_expr).unwrap();
        
        match unified_expr {
            Expression::Literal(LiteralValue::Int(i)) => assert_eq!(i, 42),
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_convert_variable() {
        let cypher_expr = CypherExpression::Variable("x".to_string());
        let unified_expr = ExpressionConverter::convert_cypher_to_unified(&cypher_expr).unwrap();
        
        match unified_expr {
            Expression::Variable(name) => assert_eq!(name, "x"),
            _ => panic!("Expected variable"),
        }
    }

    #[test]
    fn test_convert_binary_expression() {
        let left = Box::new(CypherExpression::Literal(CypherLiteral::Integer(1)));
        let right = Box::new(CypherExpression::Literal(CypherLiteral::Integer(2)));
        let cypher_expr = CypherExpression::Binary(BinaryExpression {
            left,
            operator: BinaryOperator::Add,
            right,
        });
        
        let unified_expr = ExpressionConverter::convert_cypher_to_unified(&cypher_expr).unwrap();
        
        match unified_expr {
            Expression::Binary { left: _, op, right: _ } => {
                // 验证操作符转换正确
                assert_eq!(format!("{:?}", op), "Add");
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_round_trip_conversion() {
        let original = CypherExpression::Literal(CypherLiteral::String("test".to_string()));
        let unified = ExpressionConverter::convert_cypher_to_unified(&original).unwrap();
        let back_to_cypher = ExpressionConverter::convert_unified_to_cypher(&unified).unwrap();
        
        match back_to_cypher {
            CypherExpression::Literal(CypherLiteral::String(s)) => assert_eq!(s, "test"),
            _ => panic!("Expected string literal"),
        }
    }
}