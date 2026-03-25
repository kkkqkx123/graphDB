//! Expression Generic Tool Functions
//!
//! Provides generic expression processing tool functions for cross-layer use.
//! These functions do not depend on specific validation, rewriting or planning logic and can be used in multiple layers.

use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::types::expr::visitor_checkers::ConstantChecker;
use crate::core::types::expr::visitor_collectors::PropertyCollector;
use crate::core::types::expr::Expression;
use crate::core::types::expr::ExpressionVisitor;
use crate::core::Value;
use crate::query::planning::planner::PlannerError;

/// Extracting String Values from Expressions
///
/// This method is used to extract a string value from a ContextualExpression.
/// Supports extraction from variables, literals (strings, integers, floats, booleans).
///
/// # Parameters
/// - `expr`: Contextual expression to be extracted from the string
///
/// # Back
/// - `Ok(String)`: 提取到的字符串值
/// - `Err(PlannerError)`: 无法提取字符串时的错误信息
pub fn extract_string_from_expr(expr: &ContextualExpression) -> Result<String, PlannerError> {
    if let Some(var_name) = expr.as_variable() {
        return Ok(var_name);
    }

    if let Some(literal) = expr.as_literal() {
        match literal {
            Value::String(s) => return Ok(s.clone()),
            Value::Int(i) => return Ok(i.to_string()),
            Value::Float(f) => return Ok(f.to_string()),
            Value::Bool(b) => return Ok(b.to_string()),
            _ => {
                return Err(PlannerError::InvalidOperation(format!(
                    "无法从字面量提取字符串: {:?}",
                    literal
                )))
            }
        }
    }

    Err(PlannerError::InvalidOperation(format!(
        "无法从表达式提取字符串: {}",
        expr.to_expression_string()
    )))
}

/// Generating default aliases from ContextualExpression
///
/// This method is used to generate a default alias for an expression.
/// Priority: Variable Name > Function Name > Property Name > Arithmetic Expression > Expression String
///
/// # 参数
/// - `expression`: Contextual expression to generate an alias for
///
/// # 返回
/// Default aliases generated
pub fn generate_default_alias_from_contextual(expression: &ContextualExpression) -> String {
    if let Some(var_name) = expression.as_variable() {
        return var_name;
    }

    if let Some(func_name) = expression.as_function_name() {
        return func_name.to_lowercase();
    }

    if expression.is_aggregate() {
        return "agg".to_string();
    }

    if expression.is_property() {
        if let Some(prop) = expression.as_property_name() {
            return format!("prop.{}", prop);
        }
    }

    if expression.is_binary() {
        return "expr".to_string();
    }

    expression.to_expression_string()
}

/// Extract grouping information
///
/// Extracts grouping keys and aggregates from the YieldColumn list.
/// Expressions that contain aggregation functions are used as grouping items, and expressions that do not are used as grouping keys.
///
/// # 参数
/// - `yield_columns`: list of YieldColumns
///
/// # 返回
/// - (grouped key list, grouped item list)
pub fn extract_group_info(
    yield_columns: &[crate::core::types::YieldColumn],
) -> (Vec<ContextualExpression>, Vec<ContextualExpression>) {
    let mut group_keys = Vec::new();
    let mut group_items = Vec::new();

    for column in yield_columns {
        if column.expression.contains_aggregate() {
            group_items.push(column.expression.clone());
        } else {
            group_keys.push(column.expression.clone());
        }
    }

    group_keys.dedup_by(|a, b| a.equals_by_content(b));
    group_items.dedup_by(|a, b| a.equals_by_content(b));

    (group_keys, group_items)
}

/// Extracting property references in context expressions
///
/// # 参数
/// - `ctx_expr`: context expression
///
/// # 返回
/// Names of all attributes referenced in the expression
pub fn extract_property_refs(ctx_expr: &ContextualExpression) -> Vec<String> {
    let expr_meta = match ctx_expr.expression() {
        Some(e) => e,
        None => return Vec::new(),
    };
    let expr = expr_meta.inner();
    let mut collector = PropertyCollector::new();
    ExpressionVisitor::visit(&mut collector, expr);
    collector.properties
}

/// Check if the context expression is a constant
///
/// Constant expressions do not contain any variable or property references and can be evaluated at compile time.
///
/// # 参数
/// - `ctx_expr`: 上下文表达式
///
/// # 返回
/// Returns true if the expression does not contain any variable or property references.
pub fn is_constant(ctx_expr: &ContextualExpression) -> bool {
    let expr_meta = match ctx_expr.expression() {
        Some(e) => e,
        None => return true,
    };
    let expr = expr_meta.inner();
    ConstantChecker::check(expr)
}

/// Checking if an expression is a constant (based on Expression)
///
/// 常量表达式不包含任何变量或属性引用，可以在编译时求值。
///
/// # 参数
/// - `expr`: expression
///
/// # 返回
/// 如果表达式不包含任何变量或属性引用，返回 true
pub fn is_constant_expression(expr: &Expression) -> bool {
    ConstantChecker::check(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;
    use crate::query::validator::context::ExpressionAnalysisContext;
    use std::sync::Arc;

    #[test]
    fn test_extract_string_from_expr_variable() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("test_var".to_string());
        let meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, ctx);

        let result = extract_string_from_expr(&ctx_expr);
        assert!(result.is_ok());
        assert_eq!(result.expect("提取变量字符串失败"), "test_var");
    }

    #[test]
    fn test_extract_string_from_expr_literal_string() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Literal(Value::String("hello".to_string()));
        let meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, ctx);

        let result = extract_string_from_expr(&ctx_expr);
        assert!(result.is_ok());
        assert_eq!(result.expect("提取字符串字面量失败"), "hello");
    }

    #[test]
    fn test_extract_string_from_expr_literal_int() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Literal(Value::Int(42));
        let meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, ctx);

        let result = extract_string_from_expr(&ctx_expr);
        assert!(result.is_ok());
        assert_eq!(result.expect("提取整数字面量失败"), "42");
    }

    #[test]
    fn test_generate_default_alias_from_contextual_variable() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("my_var".to_string());
        let meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, ctx);

        let alias = generate_default_alias_from_contextual(&ctx_expr);
        assert_eq!(alias, "my_var");
    }

    #[test]
    fn test_extract_property_refs() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());

        let expr = Expression::Binary {
            op: BinaryOperator::And,
            left: Box::new(Expression::Binary {
                op: BinaryOperator::Equal,
                left: Box::new(Expression::Property {
                    object: Box::new(Expression::Variable("v".to_string())),
                    property: "a".to_string(),
                }),
                right: Box::new(Expression::Literal(Value::Int(1))),
            }),
            right: Box::new(Expression::Binary {
                op: BinaryOperator::Equal,
                left: Box::new(Expression::Property {
                    object: Box::new(Expression::Variable("v".to_string())),
                    property: "b".to_string(),
                }),
                right: Box::new(Expression::Literal(Value::Int(2))),
            }),
        };

        let meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, ctx);

        let props = extract_property_refs(&ctx_expr);
        assert_eq!(props.len(), 2);
        assert!(props.contains(&"a".to_string()));
        assert!(props.contains(&"b".to_string()));
    }

    #[test]
    fn test_is_constant() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());

        let expr = Expression::Literal(Value::Int(1));
        let meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, ctx.clone());
        assert!(is_constant(&ctx_expr));

        let expr = Expression::Property {
            object: Box::new(Expression::Variable("v".to_string())),
            property: "a".to_string(),
        };
        let meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, ctx);
        assert!(!is_constant(&ctx_expr));
    }

    #[test]
    fn test_is_constant_expression() {
        let expr = Expression::Literal(Value::Int(1));
        assert!(is_constant_expression(&expr));

        let expr = Expression::Variable("v".to_string());
        assert!(!is_constant_expression(&expr));

        let expr = Expression::Property {
            object: Box::new(Expression::Variable("v".to_string())),
            property: "a".to_string(),
        };
        assert!(!is_constant_expression(&expr));
    }
}
