use crate::core::ExpressionError;
use crate::core::{Expression, LiteralValue};
use crate::query::parser::cypher::ast::expressions::{
    BinaryExpression, Expression as CypherExpression, FunctionCall, ListExpression,
    Literal as CypherLiteral, MapExpression, PropertyExpression, UnaryExpression,
};
use crate::query::parser::cypher::ast::{BinaryOperator, UnaryOperator};

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
                // 使用共享的字面量转换逻辑
                let value =
                    super::cypher_evaluator::CypherEvaluator::cypher_literal_to_value(literal)?;
                let unified_literal = match value {
                    crate::core::Value::String(s) => LiteralValue::String(s),
                    crate::core::Value::Int(i) => LiteralValue::Int(i),
                    crate::core::Value::Float(f) => LiteralValue::Float(f),
                    crate::core::Value::Bool(b) => LiteralValue::Bool(b),
                    crate::core::Value::Null(_) => LiteralValue::Null,
                    _ => {
                        return Err(ExpressionError::invalid_operation(
                            "Unsupported literal type".to_string(),
                        ))
                    }
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
                    .map(|arg| Self::convert_cypher_to_unified(&arg))
                    .collect();
                Ok(Expression::Function {
                    name: func_call.function_name.clone(),
                    args: args?,
                })
            }
            CypherExpression::Binary(bin_expr) => {
                let left = Self::convert_cypher_to_unified(&bin_expr.left)?;
                let right = Self::convert_cypher_to_unified(&bin_expr.right)?;
                // BinaryOperator已统一，直接使用
                Ok(Expression::Binary {
                    left: Box::new(left),
                    op: bin_expr.operator,
                    right: Box::new(right),
                })
            }
            CypherExpression::Unary(unary_expr) => {
                let operand = Self::convert_cypher_to_unified(&unary_expr.expression)?;
                // UnaryOperator已统一，直接使用
                Ok(Expression::Unary {
                    op: unary_expr.operator,
                    operand: Box::new(operand),
                })
            }
            CypherExpression::List(list_expr) => {
                let elements: Result<Vec<Expression>, ExpressionError> = list_expr
                    .elements
                    .iter()
                    .map(|elem| Self::convert_cypher_to_unified(&elem))
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
                        let _when_expr = Self::convert_cypher_to_unified(&alt.when_expression)?;
                        let then_expr = Self::convert_cypher_to_unified(&alt.then_expression)?;
                        Ok((format!("when_{}", "condition"), then_expr))
                    })
                    .collect();

                let _default_alternative = case_expr
                    .default_alternative
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
                // 使用共享的字面量转换逻辑
                let value = match literal {
                    LiteralValue::String(s) => crate::core::Value::String(s.clone()),
                    LiteralValue::Int(i) => crate::core::Value::Int(*i),
                    LiteralValue::Float(f) => crate::core::Value::Float(*f),
                    LiteralValue::Bool(b) => crate::core::Value::Bool(*b),
                    LiteralValue::Null => crate::core::Value::Null(crate::core::NullType::Null),
                };

                let cypher_literal = match value {
                    crate::core::Value::String(s) => CypherLiteral::String(s),
                    crate::core::Value::Int(i) => CypherLiteral::Integer(i),
                    crate::core::Value::Float(f) => CypherLiteral::Float(f),
                    crate::core::Value::Bool(b) => CypherLiteral::Boolean(b),
                    crate::core::Value::Null(_) => CypherLiteral::Null,
                    _ => {
                        return Err(ExpressionError::invalid_operation(
                            "Unsupported value type".to_string(),
                        ))
                    }
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
                    .map(|arg| Self::convert_unified_to_cypher(&arg))
                    .collect();
                Ok(CypherExpression::FunctionCall(FunctionCall {
                    function_name: name.clone(),
                    arguments: converted_args?,
                }))
            }
            Expression::Binary { left, op, right } => {
                let left_expr = Self::convert_unified_to_cypher(left)?;
                let right_expr = Self::convert_unified_to_cypher(right)?;
                // BinaryOperator已统一，直接使用
                Ok(CypherExpression::Binary(BinaryExpression {
                    left: Box::new(left_expr),
                    operator: *op,
                    right: Box::new(right_expr),
                }))
            }
            Expression::Unary { op, operand } => {
                let operand_expr = Self::convert_unified_to_cypher(operand)?;
                // UnaryOperator已统一，直接使用
                Ok(CypherExpression::Unary(UnaryExpression {
                    operator: *op,
                    expression: Box::new(operand_expr),
                }))
            }
            Expression::List(elements) => {
                let converted_elements: Result<Vec<CypherExpression>, ExpressionError> = elements
                    .iter()
                    .map(|elem| Self::convert_unified_to_cypher(&elem))
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
                Err(ExpressionError::invalid_operation(format!(
                    "Unsupported expression type: {:?}",
                    expr
                )))
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
    use crate::core::{Expression, LiteralValue};
    use crate::query::parser::cypher::ast::expressions::{
        BinaryExpression, CaseAlternative, CaseExpression, Expression as CypherExpression,
        FunctionCall, ListExpression, Literal as CypherLiteral, MapExpression, PropertyExpression,
        UnaryExpression,
    };
    use crate::query::parser::cypher::ast::{BinaryOperator, UnaryOperator};

    #[test]
    fn test_convert_literal() {
        let cypher_expr = CypherExpression::Literal(CypherLiteral::Integer(42));
        let unified_expr = ExpressionConverter::convert_cypher_to_unified(&cypher_expr)
            .expect("Conversion from cypher to unified should succeed for literals");

        match unified_expr {
            Expression::Literal(LiteralValue::Int(i)) => assert_eq!(i, 42),
            _ => panic!("Expected integer literal"),
        }
    }

    #[test]
    fn test_convert_variable() {
        let cypher_expr = CypherExpression::Variable("x".to_string());
        let unified_expr = ExpressionConverter::convert_cypher_to_unified(&cypher_expr)
            .expect("Conversion from cypher to unified should succeed for variables");

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

        let unified_expr = ExpressionConverter::convert_cypher_to_unified(&cypher_expr)
            .expect("Conversion from cypher to unified should succeed for binary operations");

        match unified_expr {
            Expression::Binary {
                left: _,
                op,
                right: _,
            } => {
                // 验证操作符转换正确
                assert_eq!(format!("{:?}", op), "Add");
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_round_trip_conversion() {
        let original = CypherExpression::Literal(CypherLiteral::String("test".to_string()));
        let unified = ExpressionConverter::convert_cypher_to_unified(&original)
            .expect("Conversion from cypher to unified should succeed for round trip");
        let back_to_cypher = ExpressionConverter::convert_unified_to_cypher(&unified)
            .expect("Conversion from unified to cypher should succeed for round trip");

        match back_to_cypher {
            CypherExpression::Literal(CypherLiteral::String(s)) => assert_eq!(s, "test"),
            _ => panic!("Expected string literal"),
        }
    }
}
