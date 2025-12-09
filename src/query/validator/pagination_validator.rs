//! 分页验证器模块
//! 负责验证SKIP、LIMIT和分页相关的表达式

use crate::graph::expression::expr_type::Expression;
use crate::core::ValueTypeDef;
use crate::query::validator::structs::PaginationContext;
use crate::config::test_config::test_config;

/// 分页验证器
pub struct PaginationValidator;

impl PaginationValidator {
    pub fn new() -> Self {
        Self
    }

    /// 验证分页参数
    pub fn validate_pagination(
        &self,
        skip_expr: Option<&Expression>,
        limit_expr: Option<&Expression>,
        context: &PaginationContext,
    ) -> Result<(), String> {
        // 验证分页参数的有效性
        if context.skip < 0 {
            return Err("SKIP不能为负数".to_string());
        }
        if context.limit < 0 {
            return Err("LIMIT不能为负数".to_string());
        }

        // 验证表达式类型（如果提供了表达式）
        if let Some(skip) = skip_expr {
            self.validate_pagination_expr(skip, "SKIP")?;
        }

        if let Some(limit) = limit_expr {
            self.validate_pagination_expr(limit, "LIMIT")?;
        }

        Ok(())
    }

    /// 验证分页表达式
    pub fn validate_pagination_expr(&self, expr: &Expression, clause_name: &str) -> Result<(), String> {
        // 简化验证：直接检查表达式是否为整数常量
        match expr {
            Expression::Constant(crate::core::Value::Int(_)) => Ok(()),
            Expression::Constant(_) => Err(format!(
                "{}表达式必须求值为整数类型",
                clause_name
            )),
            _ => {
                // 对于非常量表达式，使用类型推导
                use crate::query::visitor::DeduceTypeVisitor;
                use crate::storage::NativeStorage;

                // 创建临时存储引擎用于类型推导
                let config = test_config();
                let temp_dir = config.temp_storage_path();
                std::fs::create_dir_all(&temp_dir).map_err(|e| format!("创建临时目录失败: {}", e))?;
                let storage = NativeStorage::new(&temp_dir).map_err(|e| format!("创建存储失败: {}", e))?;

                let inputs = vec![];
                let space = "default".to_string();

                let default_context = Default::default();
                let mut type_visitor = DeduceTypeVisitor::new(
                    &storage,
                    &default_context,
                    inputs,
                    space,
                );

                let expr_type = type_visitor
                    .deduce_type(expr)
                    .map_err(|e| format!("类型推导失败: {:?}", e))?;

                if expr_type != ValueTypeDef::Int
                    && expr_type != ValueTypeDef::Empty
                    && expr_type != ValueTypeDef::Null {
                    return Err(format!(
                        "{}表达式必须求值为整数类型，得到{:?}",
                        clause_name, expr_type
                    ));
                }

                Ok(())
            }
        }
    }

    /// 验证步数范围
    pub fn validate_step_range(&self, range: &crate::query::validator::structs::MatchStepRange) -> Result<(), String> {
        if range.min > range.max {
            return Err(format!(
                "最大跳数必须大于等于最小跳数: {} vs. {}",
                range.max, range.min
            ));
        }
        Ok(())
    }

    /// 验证排序子句
    pub fn validate_order_by(
        &self,
        _factors: &[Expression], // 排序因子
        yield_columns: &[crate::query::validator::structs::YieldColumn],
        context: &crate::query::validator::structs::OrderByClauseContext,
    ) -> Result<(), String> {
        // 验证OrderBy子句
        for &(index, _) in &context.indexed_order_factors {
            if index >= yield_columns.len() {
                return Err(format!("列索引{}超出范围", index));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::expr_type::Expression;
    use crate::query::validator::structs::{MatchStepRange, OrderByClauseContext, OrderType, YieldColumn};

    #[test]
    fn test_pagination_validator_creation() {
        let _validator = PaginationValidator::new();
        // 验证器创建成功
        assert!(true); // 占位测试
    }

    #[test]
    fn test_validate_pagination() {
        let validator = PaginationValidator::new();

        // 测试有效的分页表达式
        let skip_expr = Expression::Constant(crate::core::Value::Int(1));
        let limit_expr = Expression::Constant(crate::core::Value::Int(10));
        let pagination_ctx = PaginationContext { skip: 0, limit: 10 };

        assert!(validator.validate_pagination(Some(&skip_expr), Some(&limit_expr), &pagination_ctx).is_ok());

        // 测试无效的分页参数
        let invalid_pagination_ctx = PaginationContext { skip: -1, limit: 10 };
        assert!(validator.validate_pagination(None, None, &invalid_pagination_ctx).is_err());

        let invalid_pagination_ctx2 = PaginationContext { skip: 0, limit: -5 };
        assert!(validator.validate_pagination(None, None, &invalid_pagination_ctx2).is_err());
    }

    #[test]
    fn test_validate_step_range() {
        let validator = PaginationValidator::new();

        // 测试有效的范围（min <= max）
        let valid_range = MatchStepRange::new(1, 3);
        assert!(validator.validate_step_range(&valid_range).is_ok());

        // 测试无效的范围（min > max）
        let invalid_range = MatchStepRange::new(3, 1);
        assert!(validator.validate_step_range(&invalid_range).is_err());
    }

    #[test]
    fn test_validate_order_by() {
        let validator = PaginationValidator::new();

        // 创建测试数据
        let yield_columns = vec![
            YieldColumn::new(Expression::Constant(crate::core::Value::Int(1)), "col1".to_string()),
            YieldColumn::new(Expression::Constant(crate::core::Value::Int(2)), "col2".to_string()),
        ];

        let valid_context = OrderByClauseContext {
            indexed_order_factors: vec![(0, OrderType::Asc), (1, OrderType::Desc)],
        };

        assert!(validator.validate_order_by(&[], &yield_columns, &valid_context).is_ok());

        // 测试无效的索引
        let invalid_context = OrderByClauseContext {
            indexed_order_factors: vec![(5, OrderType::Asc)], // 索引超出范围
        };

        assert!(validator.validate_order_by(&[], &yield_columns, &invalid_context).is_err());
    }

    #[test]
    fn test_pagination_expr_validation() {
        let validator = PaginationValidator::new();

        // 测试有效的整数表达式
        let int_expr = Expression::Constant(crate::core::Value::Int(10));
        assert!(validator.validate_pagination_expr(&int_expr, "LIMIT").is_ok());

        // 测试无效的字符串表达式
        let string_expr = Expression::Constant(crate::core::Value::String("invalid".to_string()));
        assert!(validator.validate_pagination_expr(&string_expr, "LIMIT").is_err());
    }

    #[test]
    fn test_edge_cases() {
        let validator = PaginationValidator::new();

        // 测试边界情况
        let zero_pagination = PaginationContext { skip: 0, limit: 0 };
        assert!(validator.validate_pagination(None, None, &zero_pagination).is_ok());

        let large_pagination = PaginationContext { skip: 1000000, limit: 1000000 };
        assert!(validator.validate_pagination(None, None, &large_pagination).is_ok());
    }
}