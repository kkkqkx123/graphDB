//! 表达式工具函数
//!
//! 提供表达式重写专用的工具函数
//!
//! # 设计说明
//!
//! Rewrite 层的职责是重写表达式，这需要：
//! 1. 分析现有表达式的结构
//! 2. 创建新的表达式
//! 3. 将新表达式注册到 ExpressionContext
//!
//! 因此，Rewrite 层需要访问 Expression 的内部结构。
//! 这是设计上的必要权衡，因为：
//! - ContextualExpression 是轻量级引用，不包含表达式结构
//! - 重写操作需要创建新的 Expression
//! - 新 Expression 必须注册到 ExpressionContext 才能使用
//!
//! # 注意
//!
//! 通用的表达式工具函数（如 extract_property_refs、is_constant）已移至
//! `core::types::expression::common_utils`，本模块仅保留重写专用的函数。

use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::types::expression::ExpressionMeta;
use crate::core::types::expression::PropertyContainsChecker;
use crate::core::types::operators::BinaryOperator;
use crate::core::Expression;
use crate::query::validator::context::ExpressionAnalysisContext;
use std::sync::Arc;

/// 检查表达式是否包含指定的属性名
///
/// # 参数
/// - `property_names`: 属性名列表
/// - `expr`: 要检查的表达式
///
/// # 返回
/// 如果表达式包含属性名列表中的任一属性，返回 true
pub fn check_col_name(property_names: &[String], expr: &Expression) -> bool {
    PropertyContainsChecker::check(expr, property_names)
}

/// 重写上下文表达式
///
/// 根据 rewrite_map 重写 ContextualExpression，并将结果注册到 ExpressionContext
///
/// # 参数
/// - `expr`: 要重写的 ContextualExpression
/// - `rewrite_map`: 重写映射表，键为变量名，值为要替换的 ContextualExpression
/// - `expr_context`: 表达式上下文，用于注册新的表达式
///
/// # 返回
/// 重写后的 ContextualExpression
pub fn rewrite_contextual_expression(
    expr: &ContextualExpression,
    rewrite_map: &std::collections::HashMap<String, ContextualExpression>,
    expr_context: Arc<ExpressionAnalysisContext>,
) -> ContextualExpression {
    let expr_meta = match expr.expression() {
        Some(e) => e,
        None => return expr.clone(),
    };
    let inner_expr = expr_meta.inner();

    let rewritten_expr = rewrite_expression_with_map(inner_expr, rewrite_map, expr_context.clone());
    let meta = ExpressionMeta::new(rewritten_expr);
    let id = expr_context.register_expression(meta);
    ContextualExpression::new(id, expr_context)
}

/// 使用 ContextualExpression 映射表重写表达式
///
/// # 参数
/// - `expr`: 要重写的 Expression
/// - `rewrite_map`: 重写映射表，键为变量名，值为要替换的 ContextualExpression
/// - `expr_context`: 表达式上下文，用于注册新的表达式
///
/// # 返回
/// 重写后的 Expression
fn rewrite_expression_with_map(
    expr: &Expression,
    rewrite_map: &std::collections::HashMap<String, ContextualExpression>,
    expr_context: Arc<ExpressionAnalysisContext>,
) -> Expression {
    match expr {
        Expression::Variable(name) => {
            if let Some(new_ctx_expr) = rewrite_map.get(name) {
                let new_expr_meta = match new_ctx_expr.expression() {
                    Some(e) => e,
                    None => return expr.clone(),
                };
                new_expr_meta.inner().clone()
            } else {
                expr.clone()
            }
        }
        Expression::Property { object, property } => {
            if let Expression::Variable(obj_name) = object.as_ref() {
                let full_name = format!("{}.{}", obj_name, property);
                if let Some(new_ctx_expr) = rewrite_map.get(&full_name) {
                    let new_expr_meta = match new_ctx_expr.expression() {
                        Some(e) => e,
                        None => return expr.clone(),
                    };
                    return new_expr_meta.inner().clone();
                }
                if let Some(new_ctx_expr) = rewrite_map.get(property) {
                    let new_expr_meta = match new_ctx_expr.expression() {
                        Some(e) => e,
                        None => return expr.clone(),
                    };
                    return Expression::Property {
                        object: Box::new(new_expr_meta.inner().clone()),
                        property: property.clone(),
                    };
                }
            }
            Expression::Property {
                object: Box::new(rewrite_expression_with_map(
                    object,
                    rewrite_map,
                    expr_context,
                )),
                property: property.clone(),
            }
        }
        Expression::Binary { left, op, right } => Expression::Binary {
            left: Box::new(rewrite_expression_with_map(
                left,
                rewrite_map,
                expr_context.clone(),
            )),
            op: *op,
            right: Box::new(rewrite_expression_with_map(
                right,
                rewrite_map,
                expr_context,
            )),
        },
        Expression::Unary { op, operand } => Expression::Unary {
            op: *op,
            operand: Box::new(rewrite_expression_with_map(
                operand,
                rewrite_map,
                expr_context,
            )),
        },
        Expression::Function { name, args } => Expression::Function {
            name: name.clone(),
            args: args
                .iter()
                .map(|arg| rewrite_expression_with_map(arg, rewrite_map, expr_context.clone()))
                .collect(),
        },
        Expression::Aggregate {
            func,
            arg,
            distinct,
        } => Expression::Aggregate {
            func: func.clone(),
            arg: Box::new(rewrite_expression_with_map(arg, rewrite_map, expr_context)),
            distinct: *distinct,
        },
        Expression::List(list) => Expression::List(
            list.iter()
                .map(|item| rewrite_expression_with_map(item, rewrite_map, expr_context.clone()))
                .collect(),
        ),
        Expression::Map(map) => Expression::Map(
            map.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        rewrite_expression_with_map(v, rewrite_map, expr_context.clone()),
                    )
                })
                .collect(),
        ),
        Expression::Case {
            test_expr,
            conditions,
            default,
        } => Expression::Case {
            test_expr: test_expr.as_ref().map(|e| {
                Box::new(rewrite_expression_with_map(
                    e,
                    rewrite_map,
                    expr_context.clone(),
                ))
            }),
            conditions: conditions
                .iter()
                .map(|(w, t)| {
                    (
                        rewrite_expression_with_map(w, rewrite_map, expr_context.clone()),
                        rewrite_expression_with_map(t, rewrite_map, expr_context.clone()),
                    )
                })
                .collect(),
            default: default
                .as_ref()
                .map(|e| Box::new(rewrite_expression_with_map(e, rewrite_map, expr_context))),
        },
        Expression::TypeCast {
            expression,
            target_type,
        } => Expression::TypeCast {
            expression: Box::new(rewrite_expression_with_map(
                expression,
                rewrite_map,
                expr_context,
            )),
            target_type: target_type.clone(),
        },
        Expression::Subscript { collection, index } => Expression::Subscript {
            collection: Box::new(rewrite_expression_with_map(
                collection,
                rewrite_map,
                expr_context.clone(),
            )),
            index: Box::new(rewrite_expression_with_map(
                index,
                rewrite_map,
                expr_context,
            )),
        },
        _ => expr.clone(),
    }
}

/// 分割过滤条件
///
/// 将复合过滤条件（如 AND 连接的条件）分割为两部分：
/// - 符合选择器函数的部分
/// - 剩余的部分
///
/// # 参数
/// - `ctx_expr`: 过滤条件上下文表达式
/// - `picker`: 选择器函数，返回 true 表示该部分应该被选中
///
/// # 返回
/// (选中的部分, 剩余的部分)
pub fn split_filter<F>(
    ctx_expr: &ContextualExpression,
    picker: F,
) -> (Option<ContextualExpression>, Option<ContextualExpression>)
where
    F: Fn(&Expression) -> bool,
{
    let expr_meta = match ctx_expr.expression() {
        Some(e) => e,
        None => return (None, None),
    };
    let expr = expr_meta.inner();
    let (picked_expr, remained_expr) = split_filter_impl(expr, &picker);

    let expr_context = ctx_expr.context().clone();
    let picked = picked_expr.map(|e| {
        let meta = ExpressionMeta::new(e);
        let id = expr_context.register_expression(meta);
        ContextualExpression::new(id, expr_context.clone())
    });

    let remained = remained_expr.map(|e| {
        let meta = ExpressionMeta::new(e);
        let id = expr_context.register_expression(meta);
        ContextualExpression::new(id, expr_context.clone())
    });

    (picked, remained)
}

fn split_filter_impl<F>(
    condition: &Expression,
    picker: &F,
) -> (Option<Expression>, Option<Expression>)
where
    F: Fn(&Expression) -> bool,
{
    match condition {
        Expression::Binary {
            op: BinaryOperator::And,
            left,
            right,
        } => {
            // 递归处理左右两侧
            let (left_picked, left_remained) = split_filter_impl(left, picker);
            let (right_picked, right_remained) = split_filter_impl(right, picker);

            // 合并选中的部分
            let picked = match (left_picked, right_picked) {
                (Some(l), Some(r)) => Some(Expression::Binary {
                    op: BinaryOperator::And,
                    left: Box::new(l),
                    right: Box::new(r),
                }),
                (Some(l), None) => Some(l),
                (None, Some(r)) => Some(r),
                (None, None) => None,
            };

            // 合并剩余的部分
            let remained = match (left_remained, right_remained) {
                (Some(l), Some(r)) => Some(Expression::Binary {
                    op: BinaryOperator::And,
                    left: Box::new(l),
                    right: Box::new(r),
                }),
                (Some(l), None) => Some(l),
                (None, Some(r)) => Some(r),
                (None, None) => None,
            };

            (picked, remained)
        }
        _ => {
            // 基本情况：检查当前表达式是否符合选择器
            if picker(condition) {
                (Some(condition.clone()), None)
            } else {
                (None, Some(condition.clone()))
            }
        }
    }
}

/// 合并两个过滤条件使用 AND
///
/// # 参数
/// - `left`: 左侧条件
/// - `right`: 右侧条件
///
/// # 返回
/// 合并后的条件
pub fn and_condition(
    left: Option<ContextualExpression>,
    right: Option<ContextualExpression>,
) -> Option<ContextualExpression> {
    match (left, right) {
        (Some(l), Some(r)) => {
            let expr_context = l.context().clone();
            let l_expr = match l.expression() {
                Some(e) => e,
                None => return Some(r),
            };
            let r_expr = match r.expression() {
                Some(e) => e,
                None => return Some(l),
            };
            let combined_expr = Expression::Binary {
                op: BinaryOperator::And,
                left: Box::new(l_expr.inner().clone()),
                right: Box::new(r_expr.inner().clone()),
            };
            let meta = ExpressionMeta::new(combined_expr);
            let id = expr_context.register_expression(meta);
            Some(ContextualExpression::new(id, expr_context))
        }
        (Some(l), None) => Some(l),
        (None, Some(r)) => Some(r),
        (None, None) => None,
    }
}

/// 合并多个过滤条件使用 AND
///
/// # 参数
/// - `conditions`: 条件列表
///
/// # 返回
/// 合并后的条件
pub fn and_conditions(
    conditions: Vec<Option<ContextualExpression>>,
) -> Option<ContextualExpression> {
    conditions.into_iter().fold(None, and_condition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_check_col_name() {
        let property_names = vec!["a".to_string(), "b".to_string()];

        let expr = Expression::Property {
            object: Box::new(Expression::Variable("v".to_string())),
            property: "a".to_string(),
        };
        assert!(check_col_name(&property_names, &expr));

        let expr = Expression::Property {
            object: Box::new(Expression::Variable("v".to_string())),
            property: "c".to_string(),
        };
        assert!(!check_col_name(&property_names, &expr));

        let expr = Expression::Binary {
            op: BinaryOperator::Equal,
            left: Box::new(Expression::Property {
                object: Box::new(Expression::Variable("v".to_string())),
                property: "a".to_string(),
            }),
            right: Box::new(Expression::Literal(Value::Int(1))),
        };
        assert!(check_col_name(&property_names, &expr));
    }

    #[test]
    fn test_split_filter() {
        let expr_context = Arc::new(ExpressionAnalysisContext::new());

        let condition = Expression::Binary {
            op: BinaryOperator::And,
            left: Box::new(Expression::Binary {
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
            }),
            right: Box::new(Expression::Binary {
                op: BinaryOperator::Equal,
                left: Box::new(Expression::Property {
                    object: Box::new(Expression::Variable("v".to_string())),
                    property: "c".to_string(),
                }),
                right: Box::new(Expression::Literal(Value::Int(3))),
            }),
        };

        let meta = ExpressionMeta::new(condition);
        let id = expr_context.register_expression(meta);
        let ctx_condition = ContextualExpression::new(id, expr_context.clone());

        let picker = |expr: &Expression| -> bool {
            let mut collector =
                crate::core::types::expression::visitor_collectors::PropertyCollector::new();
            crate::core::types::expression::ExpressionVisitor::visit(&mut collector, expr);
            collector.properties.contains(&"a".to_string())
                || collector.properties.contains(&"b".to_string())
        };

        let (picked, remained) = split_filter(&ctx_condition, picker);

        assert!(picked.is_some());
        let picked_props = crate::core::types::expression::common_utils::extract_property_refs(
            &picked.as_ref().expect("Failed to get picked expression"),
        );
        assert!(picked_props.contains(&"a".to_string()));
        assert!(picked_props.contains(&"b".to_string()));

        assert!(remained.is_some());
        let remained_props = crate::core::types::expression::common_utils::extract_property_refs(
            &remained
                .as_ref()
                .expect("Failed to get remained expression"),
        );
        assert!(remained_props.contains(&"c".to_string()));
    }
}
