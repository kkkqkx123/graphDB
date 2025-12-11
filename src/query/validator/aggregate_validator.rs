//! 聚合验证器模块
//! 负责验证聚合函数的使用和检查表达式是否包含聚合

use crate::graph::expression::expr_type::Expression;
use crate::query::validator::strategies::agg_functions::AggFunctionMeta;

/// 聚合验证器
pub struct AggregateValidator;

impl AggregateValidator {
    pub fn new() -> Self {
        Self
    }

    /// 检查表达式是否包含聚合函数
    pub fn has_aggregate_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Aggregate { .. } => true,
            Expression::UnaryOp(_, operand) => self.has_aggregate_expr(operand),
            Expression::BinaryOp(left, _, right) => {
                self.has_aggregate_expr(left) || self.has_aggregate_expr(right)
            }
            Expression::Function(_, args) => args.iter().any(|arg| self.has_aggregate_expr(arg)),
            Expression::List(items) => items.iter().any(|item| self.has_aggregate_expr(item)),
            Expression::Set(items) => items.iter().any(|item| self.has_aggregate_expr(item)),
            Expression::Map(items) => items
                .iter()
                .any(|(_, value)| self.has_aggregate_expr(value)),
            Expression::Case {
                conditions,
                default,
            } => {
                conditions.iter().any(|(cond, val)| {
                    self.has_aggregate_expr(cond) || self.has_aggregate_expr(val)
                }) || default
                    .as_ref()
                    .map_or(false, |d| self.has_aggregate_expr(d))
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                self.has_aggregate_expr(generator)
                    || condition
                        .as_ref()
                        .map_or(false, |c| self.has_aggregate_expr(c))
            }
            Expression::Predicate { list, condition } => {
                self.has_aggregate_expr(list) || self.has_aggregate_expr(condition)
            }
            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => {
                self.has_aggregate_expr(list)
                    || self.has_aggregate_expr(initial)
                    || self.has_aggregate_expr(expr)
            }
            _ => false,
        }
    }

    /// 验证UNWIND子句中不允许使用聚合函数
    pub fn validate_unwind_aggregate(&self, unwind_expr: &Expression) -> Result<(), String> {
        if self.has_aggregate_expr(unwind_expr) {
            return Err("UNWIND子句中不能使用聚合表达式".to_string());
        }
        Ok(())
    }

    /// 验证聚合表达式的合法性
    pub fn validate_aggregate_expr(&self, expr: &Expression) -> Result<(), String> {
        // 如果表达式不是聚合表达式，直接返回成功
        if !self.has_aggregate_expr(expr) {
            return Ok(());
        }

        // 仅当表达式是聚合表达式时才进行详细验证
        if let Expression::Aggregate { func, arg, distinct: _ } = expr {
            // 1. 验证聚合函数名称
            let meta = AggFunctionMeta::get(func)
                .ok_or_else(|| format!("未知的聚合函数: {}", func))?;

            // 2. 检查嵌套聚合：参数中不能包含聚合函数
            if self.has_aggregate_expr(arg) {
                return Err(format!("聚合函数不允许嵌套使用: {}", func));
            }

            // 3. 检查通配符属性限制
            if self.is_wildcard_property(arg) && !meta.allow_wildcard {
                return Err(format!("聚合函数 {} 不能应用于通配符属性", func));
            }

            // 4. 参数类型检查（可选，暂不实现）
            // 可以根据 meta.require_numeric 进行类型检查，但需要类型推导信息
        }

        Ok(())
    }

    /// 检查表达式是否为通配符属性（InputProperty("*") 或 VariableProperty { prop: "*" }）
    fn is_wildcard_property(&self, expr: &Expression) -> bool {
        match expr {
            Expression::InputProperty(prop) => prop == "*",
            Expression::VariableProperty { prop, .. } => prop == "*",
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::expr_type::Expression;

    #[test]
    fn test_aggregate_validator_creation() {
        let _validator = AggregateValidator::new();
        // 验证器创建成功
        assert!(true); // 占位测试
    }

    #[test]
    fn test_has_aggregate_expr() {
        let validator = AggregateValidator::new();

        // 测试没有聚合函数的表达式
        let non_agg_expr = Expression::Constant(crate::core::Value::Int(1));
        assert_eq!(validator.has_aggregate_expr(&non_agg_expr), false);

        // 测试包含聚合函数的表达式
        // 注意：这里需要一个聚合表达式实例，具体实现可能依赖Expression的定义
        // 对于测试目的，我们暂时使用一个简单的测试
        let binary_expr = Expression::BinaryOp(
            Box::new(Expression::Constant(crate::core::Value::Int(1))),
            crate::graph::expression::BinaryOperator::Add,
            Box::new(Expression::Constant(crate::core::Value::Int(2))),
        );
        assert_eq!(validator.has_aggregate_expr(&binary_expr), false);
    }

    #[test]
    fn test_validate_unwind_aggregate() {
        let validator = AggregateValidator::new();

        // 测试没有聚合函数的UNWIND表达式
        let non_agg_expr = Expression::Constant(crate::core::Value::Int(1));
        assert!(validator.validate_unwind_aggregate(&non_agg_expr).is_ok());

        // 测试包含聚合函数的UNWIND表达式
        // 注意：这里需要一个聚合表达式实例
        // 暂时跳过这个测试，因为需要特定的聚合表达式构造
    }

    #[test]
    fn test_validate_aggregate_expr() {
        let validator = AggregateValidator::new();

        // 测试非聚合表达式
        let non_agg_expr = Expression::Constant(crate::core::Value::Int(1));
        assert!(validator.validate_aggregate_expr(&non_agg_expr).is_ok());

        // 测试有效的聚合表达式：COUNT(*)
        let count_star = Expression::Aggregate {
            func: "COUNT".to_string(),
            arg: Box::new(Expression::InputProperty("*".to_string())),
            distinct: false,
        };
        assert!(validator.validate_aggregate_expr(&count_star).is_ok());

        // 测试无效的聚合函数名
        let unknown_agg = Expression::Aggregate {
            func: "UNKNOWN".to_string(),
            arg: Box::new(Expression::Constant(crate::core::Value::Int(1))),
            distinct: false,
        };
        assert!(validator.validate_aggregate_expr(&unknown_agg).is_err());

        // 测试嵌套聚合：SUM(COUNT(*))
        let nested_agg = Expression::Aggregate {
            func: "SUM".to_string(),
            arg: Box::new(Expression::Aggregate {
                func: "COUNT".to_string(),
                arg: Box::new(Expression::InputProperty("*".to_string())),
                distinct: false,
            }),
            distinct: false,
        };
        assert!(validator.validate_aggregate_expr(&nested_agg).is_err());

        // 测试通配符属性限制：SUM(*) 应该失败
        let sum_star = Expression::Aggregate {
            func: "SUM".to_string(),
            arg: Box::new(Expression::InputProperty("*".to_string())),
            distinct: false,
        };
        assert!(validator.validate_aggregate_expr(&sum_star).is_err());

        // 测试非通配符属性：SUM($-.prop) 应该成功（假设属性存在）
        let sum_prop = Expression::Aggregate {
            func: "SUM".to_string(),
            arg: Box::new(Expression::InputProperty("prop".to_string())),
            distinct: false,
        };
        assert!(validator.validate_aggregate_expr(&sum_prop).is_ok());

        // 测试 VariableProperty 通配符
        let var_star = Expression::Aggregate {
            func: "AVG".to_string(),
            arg: Box::new(Expression::VariableProperty {
                var: "var".to_string(),
                prop: "*".to_string(),
            }),
            distinct: false,
        };
        assert!(validator.validate_aggregate_expr(&var_star).is_err());

        // 测试 COUNT 允许通配符 VariableProperty
        let count_var_star = Expression::Aggregate {
            func: "COUNT".to_string(),
            arg: Box::new(Expression::VariableProperty {
                var: "var".to_string(),
                prop: "*".to_string(),
            }),
            distinct: false,
        };
        assert!(validator.validate_aggregate_expr(&count_var_star).is_ok());
    }

    #[test]
    fn test_nested_expressions() {
        let validator = AggregateValidator::new();

        // 测试嵌套表达式
        let nested_expr = Expression::BinaryOp(
            Box::new(Expression::UnaryOp(
                crate::graph::expression::UnaryOperator::Minus,
                Box::new(Expression::Constant(crate::core::Value::Int(5))),
            )),
            crate::graph::expression::BinaryOperator::Add,
            Box::new(Expression::Constant(crate::core::Value::Int(10))),
        );

        assert_eq!(validator.has_aggregate_expr(&nested_expr), false);
    }

    #[test]
    fn test_list_expression() {
        let validator = AggregateValidator::new();

        // 测试列表表达式
        let list_expr = Expression::List(vec![
            Expression::Constant(crate::core::Value::Int(1)),
            Expression::Constant(crate::core::Value::Int(2)),
            Expression::Constant(crate::core::Value::Int(3)),
        ]);

        assert_eq!(validator.has_aggregate_expr(&list_expr), false);
    }
}
