use crate::core::types::expression::contextual::ContextualExpression;
use crate::query::planner::PlannerError;

/// 从表达式中提取字符串值
///
/// 此方法用于在验证阶段提取字符串值，避免在 Planner 层进行此类操作
pub fn extract_string_from_expr(expr: &ContextualExpression) -> Result<String, PlannerError> {
    // 使用 ContextualExpression 的辅助方法，避免直接访问 inner
    if let Some(var_name) = expr.as_variable() {
        return Ok(var_name);
    }

    if let Some(literal) = expr.as_literal() {
        match literal {
            crate::core::Value::String(s) => return Ok(s.clone()),
            crate::core::Value::Int(i) => return Ok(i.to_string()),
            crate::core::Value::Float(f) => return Ok(f.to_string()),
            crate::core::Value::Bool(b) => return Ok(b.to_string()),
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
/// 此方法用于在验证阶段生成默认别名，避免在 Planner 层进行此类操作
pub fn generate_default_alias_from_contextual(expression: &ContextualExpression) -> String {
    // 优先使用变量名
    if let Some(var_name) = expression.as_variable() {
        return var_name;
    }

    // 函数调用使用函数名
    if let Some(func_name) = expression.as_function_name() {
        return func_name.to_lowercase();
    }

    // 聚合函数使用函数名
    if expression.is_aggregate() {
        // 聚合函数名无法直接获取，使用默认值
        return "agg".to_string();
    }

    // 属性访问
    if expression.is_property() {
        if let Some(prop) = expression.as_property_name() {
            return prop;
        }
    }

    // 算术表达式（二元运算）
    if expression.is_binary() {
        return "expr".to_string();
    }

    // 逻辑表达式（二元运算）
    if expression.is_binary() {
        return "expr".to_string();
    }

    // 默认使用表达式字符串
    expression.to_expression_string()
}

/// 提取分组信息
///
/// 从 YieldColumn 列表中提取分组键和聚合项
/// 此方法用于在验证阶段提取分组信息，避免在 Planner 层进行此类操作
pub fn extract_group_info(
    yield_columns: &[crate::core::types::YieldColumn],
) -> (Vec<ContextualExpression>, Vec<ContextualExpression>) {
    let mut group_keys = Vec::new();
    let mut group_items = Vec::new();

    for column in yield_columns {
        // 直接使用 YieldColumn 中的 ContextualExpression
        if column.expression.contains_aggregate() {
            // 包含聚合函数的表达式作为分组项
            group_items.push(column.expression.clone());
        } else {
            // 不包含聚合函数的表达式作为分组键
            group_keys.push(column.expression.clone());
        }
    }

    // 去重
    group_keys.dedup_by(|a, b| a.equals_by_content(b));
    group_items.dedup_by(|a, b| a.equals_by_content(b));

    (group_keys, group_items)
}
