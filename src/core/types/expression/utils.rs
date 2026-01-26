//! Core Expression 工具函数

use crate::core::types::expression::Expression;

/// Core Expression 工具函数
pub struct CoreExprUtils;

impl CoreExprUtils {
    /// 查找表达式中的变量
    pub fn find_variables(expression: &Expression) -> Vec<String> {
        let mut variables = Vec::new();
        Self::find_variables_recursive(expression, &mut variables);
        variables
    }

    fn find_variables_recursive(expression: &Expression, variables: &mut Vec<String>) {
        match expression {
            Expression::Variable(name) => variables.push(name.clone()),
            Expression::Binary { left, right, .. } => {
                Self::find_variables_recursive(left, variables);
                Self::find_variables_recursive(right, variables);
            }
            Expression::Unary { operand, .. } => Self::find_variables_recursive(operand, variables),
            Expression::Function { args, .. } => {
                for arg in args {
                    Self::find_variables_recursive(arg, variables);
                }
            }
            Expression::Property { object, .. } => Self::find_variables_recursive(object, variables),
            Expression::List(items) => {
                for item in items {
                    Self::find_variables_recursive(item, variables);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    Self::find_variables_recursive(value, variables);
                }
            }
            Expression::Case { conditions, default, .. } => {
                for (when, then) in conditions {
                    Self::find_variables_recursive(when, variables);
                    Self::find_variables_recursive(then, variables);
                }
                if let Some(expr) = default {
                    Self::find_variables_recursive(expr, variables);
                }
            }
            Expression::Subscript { collection, index, .. } => {
                Self::find_variables_recursive(collection, variables);
                Self::find_variables_recursive(index, variables);
            }
            Expression::TypeCast { expression, .. } => Self::find_variables_recursive(expression, variables),
            Expression::Range { collection, start, end, .. } => {
                Self::find_variables_recursive(collection, variables);
                if let Some(expr) = start {
                    Self::find_variables_recursive(expr, variables);
                }
                if let Some(expr) = end {
                    Self::find_variables_recursive(expr, variables);
                }
            }
            Expression::Path(elements) => {
                for elem in elements {
                    Self::find_variables_recursive(elem, variables);
                }
            }
            _ => {}
        }
    }

    /// 检查表达式是否包含聚合函数
    pub fn contains_aggregate(expression: &Expression) -> bool {
        Self::contains_aggregate_recursive(expression)
    }

    fn contains_aggregate_recursive(expression: &Expression) -> bool {
        match expression {
            Expression::Function { name, .. } => {
                let func_name = name.to_uppercase();
                matches!(
                    func_name.as_str(),
                    "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "COLLECT" | "AGGREGATE"
                )
            }
            Expression::Binary { left, right, .. } => {
                Self::contains_aggregate_recursive(left)
                    || Self::contains_aggregate_recursive(right)
            }
            Expression::Unary { operand, .. } => Self::contains_aggregate_recursive(operand),
            Expression::List(items) => items.iter().any(Self::contains_aggregate_recursive),
            Expression::Map(pairs) => pairs
                .iter()
                .any(|(_, value)| Self::contains_aggregate_recursive(value)),
            Expression::Case { conditions, default, .. } => {
                let match_contains = false;
                let when_contains = conditions.iter().any(|(when, then)| {
                    Self::contains_aggregate_recursive(when)
                        || Self::contains_aggregate_recursive(then)
                });
                let default_contains = default
                    .as_ref()
                    .map_or(false, |expr| Self::contains_aggregate_recursive(expr));
                match_contains || when_contains || default_contains
            }
            Expression::Subscript { collection, index, .. } => {
                Self::contains_aggregate_recursive(collection)
                    || Self::contains_aggregate_recursive(index)
            }
            Expression::TypeCast { expression, .. } => Self::contains_aggregate_recursive(expression),
            Expression::Range { collection, start, end, .. } => {
                let collection_contains = Self::contains_aggregate_recursive(collection);
                let start_contains = start
                    .as_ref()
                    .map_or(false, |expr| Self::contains_aggregate_recursive(expr));
                let end_contains = end
                    .as_ref()
                    .map_or(false, |expr| Self::contains_aggregate_recursive(expr));
                collection_contains || start_contains || end_contains
            }
            Expression::Path(elements) => elements.iter().any(Self::contains_aggregate_recursive),
            _ => false,
        }
    }
}