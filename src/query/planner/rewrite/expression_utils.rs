//! 表达式工具函数
//!
//! 提供表达式处理和操作的工具函数

use crate::core::Expression;
use crate::core::types::operators::BinaryOperator;

/// 检查表达式是否包含指定的属性名
///
/// # 参数
/// - `property_names`: 属性名列表
/// - `expr`: 要检查的表达式
///
/// # 返回
/// 如果表达式包含属性名列表中的任一属性，返回 true
pub fn check_col_name(property_names: &[String], expr: &Expression) -> bool {
    match expr {
        Expression::Property { property, .. } => property_names.contains(property),
        Expression::Binary { left, right, .. } => {
            check_col_name(property_names, left) || check_col_name(property_names, right)
        }
        Expression::Unary { operand, .. } => check_col_name(property_names, operand),
        Expression::Function { args, .. } => {
            args.iter().any(|arg| check_col_name(property_names, arg))
        }
        Expression::Case { conditions, default, .. } => {
            let has_in_conditions = conditions.iter().any(|(when, then)| {
                check_col_name(property_names, when) || check_col_name(property_names, then)
            });
            let has_in_default = default
                .as_ref()
                .map(|e| check_col_name(property_names, e))
                .unwrap_or(false);
            has_in_conditions || has_in_default
        }
        _ => false,
    }
}

/// 分割过滤条件
///
/// 将复合过滤条件（如 AND 连接的条件）分割为两部分：
/// - 符合选择器函数的部分
/// - 剩余的部分
///
/// # 参数
/// - `condition`: 过滤条件表达式
/// - `picker`: 选择器函数，返回 true 表示该部分应该被选中
///
/// # 返回
/// (选中的部分, 剩余的部分)
pub fn split_filter<F>(condition: &Expression, picker: F) -> (Option<Expression>, Option<Expression>)
where
    F: Fn(&Expression) -> bool,
{
    split_filter_impl(condition, &picker)
}

fn split_filter_impl<F>(condition: &Expression, picker: &F) -> (Option<Expression>, Option<Expression>)
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

/// 提取表达式中的属性引用
///
/// # 参数
/// - `expr`: 表达式
///
/// # 返回
/// 表达式中引用的所有属性名
pub fn extract_property_refs(expr: &Expression) -> Vec<String> {
    let mut props = Vec::new();
    extract_property_refs_recursive(expr, &mut props);
    props
}

fn extract_property_refs_recursive(expr: &Expression, props: &mut Vec<String>) {
    match expr {
        Expression::Property { property, .. } => {
            if !props.contains(property) {
                props.push(property.clone());
            }
        }
        Expression::Binary { left, right, .. } => {
            extract_property_refs_recursive(left, props);
            extract_property_refs_recursive(right, props);
        }
        Expression::Unary { operand, .. } => {
            extract_property_refs_recursive(operand, props);
        }
        Expression::Function { args, .. } => {
            for arg in args {
                extract_property_refs_recursive(arg, props);
            }
        }
        Expression::Case { conditions, default, .. } => {
            for (when, then) in conditions {
                extract_property_refs_recursive(when, props);
                extract_property_refs_recursive(then, props);
            }
            if let Some(default_expr) = default {
                extract_property_refs_recursive(default_expr, props);
            }
        }
        _ => {}
    }
}

/// 检查表达式是否为常量
///
/// # 参数
/// - `expr`: 表达式
///
/// # 返回
/// 如果表达式不包含任何属性引用，返回 true
pub fn is_constant(expr: &Expression) -> bool {
    extract_property_refs(expr).is_empty()
}

/// 合并两个过滤条件使用 AND
///
/// # 参数
/// - `left`: 左侧条件
/// - `right`: 右侧条件
///
/// # 返回
/// 合并后的条件
pub fn and_condition(left: Option<Expression>, right: Option<Expression>) -> Option<Expression> {
    match (left, right) {
        (Some(l), Some(r)) => Some(Expression::Binary {
            op: BinaryOperator::And,
            left: Box::new(l),
            right: Box::new(r),
        }),
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
pub fn and_conditions(conditions: Vec<Option<Expression>>) -> Option<Expression> {
    conditions.into_iter().fold(None, and_condition)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_check_col_name() {
        let property_names = vec!["a".to_string(), "b".to_string()];
        
        // 简单属性引用
        let expr = Expression::Property {
            object: Box::new(Expression::Variable("v".to_string())),
            property: "a".to_string(),
        };
        assert!(check_col_name(&property_names, &expr));
        
        // 不在列表中的属性
        let expr = Expression::Property {
            object: Box::new(Expression::Variable("v".to_string())),
            property: "c".to_string(),
        };
        assert!(!check_col_name(&property_names, &expr));
        
        // 二元表达式
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
        // 创建测试条件: a = 1 AND b = 2 AND c = 3
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

        // 选择包含 "a" 或 "b" 的条件
        let picker = |expr: &Expression| -> bool {
            let props = extract_property_refs(expr);
            props.contains(&"a".to_string()) || props.contains(&"b".to_string())
        };

        let (picked, remained) = split_filter(&condition, picker);

        // 验证选中的部分包含 a 和 b
        assert!(picked.is_some());
        let picked_props = extract_property_refs(&picked.expect("Failed to get picked expression"));
        assert!(picked_props.contains(&"a".to_string()));
        assert!(picked_props.contains(&"b".to_string()));

        // 验证剩余的部分包含 c
        assert!(remained.is_some());
        let remained_props = extract_property_refs(&remained.expect("Failed to get remained expression"));
        assert!(remained_props.contains(&"c".to_string()));
    }

    #[test]
    fn test_extract_property_refs() {
        // a = 1 AND b = 2
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

        let props = extract_property_refs(&expr);
        assert_eq!(props.len(), 2);
        assert!(props.contains(&"a".to_string()));
        assert!(props.contains(&"b".to_string()));
    }

    #[test]
    fn test_is_constant() {
        // 常量表达式
        let expr = Expression::Literal(Value::Int(1));
        assert!(is_constant(&expr));

        // 包含属性的表达式
        let expr = Expression::Property {
            object: Box::new(Expression::Variable("v".to_string())),
            property: "a".to_string(),
        };
        assert!(!is_constant(&expr));
    }
}
