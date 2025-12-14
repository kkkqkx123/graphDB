//! 分页验证策略
//! 负责验证SKIP、LIMIT和分页相关的表达式

use super::super::validation_interface::*;
use super::super::structs::*;
use crate::graph::expression::Expression;
use crate::core::ValueTypeDef;
use crate::config::test_config::test_config;

/// 分页验证策略
pub struct PaginationValidationStrategy;

impl PaginationValidationStrategy {
    pub fn new() -> Self {
        Self
    }
    
    /// 验证分页参数的有效性
    pub fn validate_pagination(
        &self,
        skip_expr: Option<&Expression>,
        limit_expr: Option<&Expression>,
        context: &PaginationContext,
    ) -> Result<(), ValidationError> {
        // 验证分页参数的有效性
        if context.skip < 0 {
            return Err(ValidationError::new(
                "SKIP不能为负数".to_string(),
                ValidationErrorType::PaginationError
            ));
        }
        if context.limit < 0 {
            return Err(ValidationError::new(
                "LIMIT不能为负数".to_string(),
                ValidationErrorType::PaginationError
            ));
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
    pub fn validate_pagination_expr(&self, expr: &Expression, clause_name: &str) -> Result<(), ValidationError> {
        // 简化验证：直接检查表达式是否为整数常量
        match expr {
            Expression::Constant(crate::core::Value::Int(_)) => Ok(()),
            Expression::Constant(_) => Err(ValidationError::new(
                format!("{}表达式必须求值为整数类型", clause_name),
                ValidationErrorType::PaginationError
            )),
            _ => {
                // 对于非常量表达式，使用类型推导
                use crate::query::visitor::DeduceTypeVisitor;
                use crate::storage::NativeStorage;
                
                // 创建临时存储引擎用于类型推导
                let config = test_config();
                let temp_dir = config.temp_storage_path();
                std::fs::create_dir_all(&temp_dir).map_err(|e| ValidationError::new(
                    format!("创建临时目录失败: {}", e),
                    ValidationErrorType::PaginationError
                ))?;
                let storage = NativeStorage::new(&temp_dir).map_err(|e| ValidationError::new(
                    format!("创建存储失败: {}", e),
                    ValidationErrorType::PaginationError
                ))?;
                
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
                    .map_err(|e| ValidationError::new(
                        format!("类型推导失败: {:?}", e),
                        ValidationErrorType::PaginationError
                    ))?;
                
                if expr_type != ValueTypeDef::Int
                    && expr_type != ValueTypeDef::Empty
                    && expr_type != ValueTypeDef::Null {
                    return Err(ValidationError::new(
                        format!("{}表达式必须求值为整数类型，得到{:?}", clause_name, expr_type),
                        ValidationErrorType::PaginationError
                    ));
                }
                
                Ok(())
            }
        }
    }
    
    /// 验证步数范围
    pub fn validate_step_range(&self, range: &MatchStepRange) -> Result<(), ValidationError> {
        if range.min > range.max {
            return Err(ValidationError::new(
                format!("最大跳数必须大于等于最小跳数: {} vs. {}", range.max, range.min),
                ValidationErrorType::PaginationError
            ));
        }
        Ok(())
    }
    
    /// 验证排序子句
    pub fn validate_order_by(
        &self,
        _factors: &[Expression], // 排序因子
        yield_columns: &[YieldColumn],
        context: &OrderByClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证OrderBy子句
        for &(index, _) in &context.indexed_order_factors {
            if index >= yield_columns.len() {
                return Err(ValidationError::new(
                    format!("列索引{}超出范围", index),
                    ValidationErrorType::PaginationError
                ));
            }
        }
        
        Ok(())
    }
}

impl ValidationStrategy for PaginationValidationStrategy {
    fn validate(&self, context: &dyn ValidationContext) -> Result<(), ValidationError> {
        // 遍历所有查询部分，验证分页参数
        for query_part in context.get_query_parts() {
            // 验证Match子句中的分页
            for match_ctx in &query_part.matchs {
                if let Some(skip) = &match_ctx.skip {
                    self.validate_pagination_expr(skip, "SKIP")?;
                }
                if let Some(limit) = &match_ctx.limit {
                    self.validate_pagination_expr(limit, "LIMIT")?;
                }
            }
            
            // 验证边界子句中的分页
            if let Some(boundary) = &query_part.boundary {
                match boundary {
                    BoundaryClauseContext::With(with_ctx) => {
                        if let Some(pagination) = &with_ctx.pagination {
                            self.validate_pagination(None, None, pagination)?;
                        }
                        if let Some(order_by) = &with_ctx.order_by {
                            self.validate_order_by(&[], &with_ctx.yield_clause.yield_columns, order_by)?;
                        }
                    }
                    BoundaryClauseContext::Unwind(_) => {
                        // UNWIND子句没有分页
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn strategy_type(&self) -> ValidationStrategyType {
        ValidationStrategyType::Pagination
    }
    
    fn strategy_name(&self) -> &'static str {
        "PaginationValidationStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::Expression;
    
    #[test]
    fn test_pagination_validation_strategy_creation() {
        let strategy = PaginationValidationStrategy::new();
        assert_eq!(strategy.strategy_type(), ValidationStrategyType::Pagination);
        assert_eq!(strategy.strategy_name(), "PaginationValidationStrategy");
    }
    
    #[test]
    fn test_validate_pagination() {
        let strategy = PaginationValidationStrategy::new();
        
        // 测试有效的分页表达式
        let skip_expr = Expression::Constant(crate::core::Value::Int(1));
        let limit_expr = Expression::Constant(crate::core::Value::Int(10));
        let pagination_ctx = PaginationContext { skip: 0, limit: 10 };
        
        assert!(strategy.validate_pagination(Some(&skip_expr), Some(&limit_expr), &pagination_ctx).is_ok());
        
        // 测试无效的分页参数
        let invalid_pagination_ctx = PaginationContext { skip: -1, limit: 10 };
        assert!(strategy.validate_pagination(None, None, &invalid_pagination_ctx).is_err());
        
        let invalid_pagination_ctx2 = PaginationContext { skip: 0, limit: -5 };
        assert!(strategy.validate_pagination(None, None, &invalid_pagination_ctx2).is_err());
    }
    
    #[test]
    fn test_validate_step_range() {
        let strategy = PaginationValidationStrategy::new();
        
        // 测试有效的范围（min <= max）
        let valid_range = MatchStepRange::new(1, 3);
        assert!(strategy.validate_step_range(&valid_range).is_ok());
        
        // 测试无效的范围（min > max）
        let invalid_range = MatchStepRange::new(3, 1);
        assert!(strategy.validate_step_range(&invalid_range).is_err());
    }
    
    #[test]
    fn test_validate_order_by() {
        let strategy = PaginationValidationStrategy::new();
        
        // 创建测试数据
        let yield_columns = vec![
            YieldColumn::new(Expression::Constant(crate::core::Value::Int(1)), "col1".to_string()),
            YieldColumn::new(Expression::Constant(crate::core::Value::Int(2)), "col2".to_string()),
        ];
        
        let valid_context = OrderByClauseContext {
            indexed_order_factors: vec![(0, OrderType::Asc), (1, OrderType::Desc)],
        };
        
        assert!(strategy.validate_order_by(&[], &yield_columns, &valid_context).is_ok());
        
        // 测试无效的索引
        let invalid_context = OrderByClauseContext {
            indexed_order_factors: vec![(5, OrderType::Asc)], // 索引超出范围
        };
        
        assert!(strategy.validate_order_by(&[], &yield_columns, &invalid_context).is_err());
    }
    
    #[test]
    fn test_pagination_expr_validation() {
        let strategy = PaginationValidationStrategy::new();
        
        // 测试有效的整数表达式
        let int_expr = Expression::Constant(crate::core::Value::Int(10));
        assert!(strategy.validate_pagination_expr(&int_expr, "LIMIT").is_ok());
        
        // 测试无效的字符串表达式
        let string_expr = Expression::Constant(crate::core::Value::String("invalid".to_string()));
        assert!(strategy.validate_pagination_expr(&string_expr, "LIMIT").is_err());
    }
    
    #[test]
    fn test_edge_cases() {
        let strategy = PaginationValidationStrategy::new();
        
        // 测试边界情况
        let zero_pagination = PaginationContext { skip: 0, limit: 0 };
        assert!(strategy.validate_pagination(None, None, &zero_pagination).is_ok());
        
        let large_pagination = PaginationContext { skip: 1000000, limit: 1000000 };
        assert!(strategy.validate_pagination(None, None, &large_pagination).is_ok());
    }
}