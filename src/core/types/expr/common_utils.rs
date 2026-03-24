//! 表达式通用工具函数
//!
//! 提供跨层使用的通用表达式处理工具函数。
//! 这些函数不依赖于特定的验证、重写或规划逻辑，可以在多个层中使用。

use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::types::expression::visitor_checkers::ConstantChecker;
use crate::core::types::expression::visitor_collectors::PropertyCollector;
use crate::core::types::expression::Expression;
use crate::core::types::expression::ExpressionVisitor;
use crate::core::Value;
use crate::query::planner::PlannerError;

/// 从表达式中提取字符串值
///
/// 此方法用于从 ContextualExpression 中提取字符串值。
/// 支持从变量、字面量（字符串、整数、浮点数、布尔值）中提取。
///
/// # 参数
/// - `expr`: 要提取字符串的上下文表达式
///
/// # 返回
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

/// 从 ContextualExpression 生成默认别名
///
/// 此方法用于为表达式生成默认别名。
/// 优先级：变量名 > 函数名 > 属性名 > 算术表达式 > 表达式字符串
///
/// # 参数
/// - `expression`: 要生成别名的上下文表达式
///
/// # 返回
/// 生成的默认别名
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

/// 提取分组信息
///
/// 从 YieldColumn 列表中提取分组键和聚合项。
/// 包含聚合函数的表达式作为分组项，不包含的表达式作为分组键。
///
/// # 参数
/// - `yield_columns`: YieldColumn 列表
///
/// # 返回
/// - (分组键列表, 分组项列表)
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

/// 提取上下文表达式中的属性引用
///
/// # 参数
/// - `ctx_expr`: 上下文表达式
///
/// # 返回
/// 表达式中引用的所有属性名
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

/// 检查上下文表达式是否为常量
///
/// 常量表达式不包含任何变量或属性引用，可以在编译时求值。
///
/// # 参数
/// - `ctx_expr`: 上下文表达式
///
/// # 返回
/// 如果表达式不包含任何变量或属性引用，返回 true
pub fn is_constant(ctx_expr: &ContextualExpression) -> bool {
    let expr_meta = match ctx_expr.expression() {
        Some(e) => e,
        None => return true,
    };
    let expr = expr_meta.inner();
    ConstantChecker::check(expr)
}

/// 检查表达式是否为常量（基于 Expression）
///
/// 常量表达式不包含任何变量或属性引用，可以在编译时求值。
///
/// # 参数
/// - `expr`: 表达式
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
        let meta = crate::core::types::expression::ExpressionMeta::new(expr);
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
        let meta = crate::core::types::expression::ExpressionMeta::new(expr);
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
        let meta = crate::core::types::expression::ExpressionMeta::new(expr);
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
        let meta = crate::core::types::expression::ExpressionMeta::new(expr);
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

        let meta = crate::core::types::expression::ExpressionMeta::new(expr);
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
        let meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, ctx.clone());
        assert!(is_constant(&ctx_expr));

        let expr = Expression::Property {
            object: Box::new(Expression::Variable("v".to_string())),
            property: "a".to_string(),
        };
        let meta = crate::core::types::expression::ExpressionMeta::new(expr);
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
