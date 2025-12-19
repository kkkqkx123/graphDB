//! 表达式验证策略
//! 负责验证各种表达式类型和结构

use super::super::structs::*;
use super::super::validation_interface::*;
use crate::config::test_config::test_config;
use crate::core::ValueTypeDef;
use crate::graph::expression::Expression;

/// 表达式验证策略
pub struct ExpressionValidationStrategy;

impl ExpressionValidationStrategy {
    pub fn new() -> Self {
        Self
    }

    /// 验证过滤条件
    pub fn validate_filter(
        &self,
        filter: &Expression,
        context: &WhereClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证过滤表达式
        // 检查表达式中的别名是否已定义
        // 验证表达式的类型

        // 使用别名验证器验证别名
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        alias_validator.validate_aliases(&[filter.clone()], &context.aliases_available)?;

        // 简化验证：直接检查布尔常量
        match filter {
            Expression::Literal(crate::graph::expression::expression::LiteralValue::Bool(_)) => {
                Ok(())
            }
            Expression::Literal(_) => Err(ValidationError::new(
                "WHERE表达式必须求值为布尔类型".to_string(),
                ValidationErrorType::TypeError,
            )),
            _ => {
                // 对于非常量表达式，使用类型推导
                use crate::query::visitor::DeduceTypeVisitor;
                use crate::storage::NativeStorage;

                // 创建临时存储引擎用于类型推导
                let config = test_config();
                let temp_dir = config.temp_storage_path();
                std::fs::create_dir_all(&temp_dir).map_err(|e| {
                    ValidationError::new(
                        format!("创建临时目录失败: {}", e),
                        ValidationErrorType::TypeError,
                    )
                })?;
                let storage = NativeStorage::new(&temp_dir).map_err(|e| {
                    ValidationError::new(
                        format!("创建存储失败: {}", e),
                        ValidationErrorType::TypeError,
                    )
                })?;

                let inputs = vec![];
                let space = "default".to_string();

                let default_context = Default::default();
                let mut type_visitor =
                    DeduceTypeVisitor::new(&storage, &default_context, inputs, space);

                let expr_type = type_visitor.deduce_type(filter).map_err(|e| {
                    ValidationError::new(
                        format!("类型推导失败: {:?}", e),
                        ValidationErrorType::TypeError,
                    )
                })?;

                if expr_type != ValueTypeDef::Bool
                    && expr_type != ValueTypeDef::Empty
                    && expr_type != ValueTypeDef::Null
                {
                    return Err(ValidationError::new(
                        format!("WHERE表达式必须求值为布尔类型，得到{:?}", expr_type),
                        ValidationErrorType::TypeError,
                    ));
                }

                Ok(())
            }
        }
    }

    /// 验证Match路径
    pub fn validate_path(
        &self,
        path: &Expression,
        context: &MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证Match路径表达式
        // 检查路径中的节点和边定义
        // 验证路径模式的有效性

        // 这里应该解析路径表达式，提取节点和边的信息
        // 但由于当前的路径表示可能不同，我们暂时实现基本验证

        // 检查路径中是否存在有效的节点和边结构
        match path {
            Expression::MatchPathPattern { patterns, .. } => {
                for pattern in patterns {
                    // 验证每个路径模式
                    self.validate_single_path_pattern(pattern, context)?;
                }
            }
            _ => {
                return Err(ValidationError::new(
                    "无效的路径模式表达式".to_string(),
                    ValidationErrorType::SyntaxError,
                ));
            }
        }

        Ok(())
    }

    /// 验证单个路径模式
    pub fn validate_single_path_pattern(
        &self,
        _pattern: &Expression,
        _context: &MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证单个路径模式的结构
        // 在实际实现中，这里会检查节点、边的定义等
        Ok(())
    }

    /// 验证Return子句
    pub fn validate_return(
        &self,
        return_expr: &Expression,
        _query_parts: &[QueryPart],
        context: &ReturnClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证Return子句中的表达式
        // 检查使用的别名是否在作用域内

        // 使用别名验证器验证别名
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        alias_validator.validate_aliases(&[return_expr.clone()], &context.aliases_available)
    }

    /// 验证With子句
    pub fn validate_with(
        &self,
        with_expr: &Expression,
        _query_parts: &[QueryPart],
        context: &WithClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证With子句中的表达式别名

        // 使用别名验证器验证别名
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        alias_validator.validate_aliases(&[with_expr.clone()], &context.aliases_available)?;

        // 验证With子句的分页
        if let Some(ref pagination) = context.pagination {
            if pagination.skip < 0 {
                return Err(ValidationError::new(
                    "SKIP不能为负数".to_string(),
                    ValidationErrorType::PaginationError,
                ));
            }
            if pagination.limit < 0 {
                return Err(ValidationError::new(
                    "LIMIT不能为负数".to_string(),
                    ValidationErrorType::PaginationError,
                ));
            }
        }

        // 验证是否包含聚合表达式
        use super::aggregate_strategy::AggregateValidationStrategy;
        let aggregate_validator = AggregateValidationStrategy::new();
        if aggregate_validator.has_aggregate_expr(with_expr) {
            // 这里需要修改context，但在策略模式中不应该直接修改
            // 应该在主验证器中处理
        }

        Ok(())
    }

    /// 验证Unwind子句
    pub fn validate_unwind(
        &self,
        unwind_expr: &Expression,
        context: &UnwindClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证Unwind表达式中的别名

        // 使用别名验证器验证别名
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        alias_validator.validate_aliases(&[unwind_expr.clone()], &context.aliases_available)?;

        // 检查是否有聚合表达式（在UNWIND中不允许）
        use super::aggregate_strategy::AggregateValidationStrategy;
        let aggregate_validator = AggregateValidationStrategy::new();
        if aggregate_validator.has_aggregate_expr(unwind_expr) {
            return Err(ValidationError::new(
                "UNWIND子句中不能使用聚合表达式".to_string(),
                ValidationErrorType::AggregateError,
            ));
        }

        Ok(())
    }

    /// 验证Yield子句
    pub fn validate_yield(&self, context: &YieldClauseContext) -> Result<(), ValidationError> {
        // 如果有聚合函数，执行特殊验证
        if context.has_agg {
            return self.validate_group(context);
        }

        // 对于普通Yield子句，验证别名
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        for col in &context.yield_columns {
            alias_validator.validate_aliases(&[col.expr.clone()], &context.aliases_available)?;
        }

        Ok(())
    }

    /// 验证分组子句
    fn validate_group(&self, yield_ctx: &YieldClauseContext) -> Result<(), ValidationError> {
        // 验证分组逻辑
        use super::aggregate_strategy::AggregateValidationStrategy;
        let aggregate_validator = AggregateValidationStrategy::new();

        for col in &yield_ctx.yield_columns {
            // 如果表达式包含聚合函数，验证聚合表达式
            if aggregate_validator.has_aggregate_expr(&col.expr) {
                // 验证聚合函数
                // 在实际实现中，这里会进行更详细的聚合函数验证
            } else {
                // 非聚合表达式将作为分组键添加
                // 这里需要修改context，但在策略模式中不应该直接修改
                // 应该在主验证器中处理
            }
        }

        Ok(())
    }
}

impl ValidationStrategy for ExpressionValidationStrategy {
    fn validate(&self, context: &dyn ValidationContext) -> Result<(), ValidationError> {
        // 遍历所有查询部分，验证表达式
        for query_part in context.get_query_parts() {
            // 验证Match子句中的表达式
            for match_ctx in &query_part.matchs {
                if let Some(where_clause) = &match_ctx.where_clause {
                    if let Some(filter) = &where_clause.filter {
                        self.validate_filter(filter, where_clause)?;
                    }
                }
            }

            // 验证边界子句中的表达式
            if let Some(boundary) = &query_part.boundary {
                match boundary {
                    BoundaryClauseContext::With(with_ctx) => {
                        if let Some(where_clause) = &with_ctx.where_clause {
                            if let Some(filter) = &where_clause.filter {
                                self.validate_filter(filter, where_clause)?;
                            }
                        }
                    }
                    BoundaryClauseContext::Unwind(unwind_ctx) => {
                        self.validate_unwind(&unwind_ctx.unwind_expr, unwind_ctx)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn strategy_type(&self) -> ValidationStrategyType {
        ValidationStrategyType::Expression
    }

    fn strategy_name(&self) -> &'static str {
        "ExpressionValidationStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::Expression;

    #[test]
    fn test_expression_validation_strategy_creation() {
        let strategy = ExpressionValidationStrategy::new();
        assert_eq!(strategy.strategy_type(), ValidationStrategyType::Expression);
        assert_eq!(strategy.strategy_name(), "ExpressionValidationStrategy");
    }

    #[test]
    fn test_validate_filter() {
        let strategy = ExpressionValidationStrategy::new();

        // 创建测试数据
        let where_context = WhereClauseContext {
            filter: None,
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            paths: Vec::new(),
        };

        // 测试布尔表达式
        let bool_expr = Expression::Literal(
            crate::graph::expression::expression::LiteralValue::Bool(true),
        );
        assert!(strategy.validate_filter(&bool_expr, &where_context).is_ok());
    }

    #[test]
    fn test_validate_path() {
        let _strategy = ExpressionValidationStrategy::new();

        let _match_context = MatchClauseContext {
            paths: Vec::new(),
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        // 测试路径验证
        // 注意：这里需要一个有效的路径表达式
        // 暂时跳过这个测试，因为需要特定的路径表达式构造
    }

    #[test]
    fn test_validate_return() {
        let strategy = ExpressionValidationStrategy::new();

        let return_context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: Vec::new(),
                aliases_available: std::collections::HashMap::new(),
                aliases_generated: std::collections::HashMap::new(),
                distinct: false,
                has_agg: false,
                group_keys: Vec::new(),
                group_items: Vec::new(),
                need_gen_project: false,
                agg_output_column_names: Vec::new(),
                proj_output_column_names: Vec::new(),
                proj_cols: Vec::new(),
                paths: Vec::new(),
            },
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
        };

        // 测试Return子句验证
        let return_expr =
            Expression::Literal(crate::graph::expression::expression::LiteralValue::Int(1));
        assert!(strategy
            .validate_return(&return_expr, &[], &return_context)
            .is_ok());
    }

    #[test]
    fn test_validate_with() {
        let strategy = ExpressionValidationStrategy::new();

        let with_context = WithClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: Vec::new(),
                aliases_available: std::collections::HashMap::new(),
                aliases_generated: std::collections::HashMap::new(),
                distinct: false,
                has_agg: false,
                group_keys: Vec::new(),
                group_items: Vec::new(),
                need_gen_project: false,
                agg_output_column_names: Vec::new(),
                proj_output_column_names: Vec::new(),
                proj_cols: Vec::new(),
                paths: Vec::new(),
            },
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            pagination: None,
            order_by: None,
            distinct: false,
        };

        // 测试With子句验证
        let with_expr =
            Expression::Literal(crate::graph::expression::expression::LiteralValue::Int(1));
        assert!(strategy
            .validate_with(&with_expr, &[], &with_context)
            .is_ok());
    }

    #[test]
    fn test_validate_unwind() {
        let strategy = ExpressionValidationStrategy::new();

        let unwind_context = UnwindClauseContext {
            alias: "test".to_string(),
            unwind_expr: Expression::Literal(
                crate::graph::expression::expression::LiteralValue::Int(1),
            ),
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            paths: Vec::new(),
        };

        // 测试Unwind子句验证
        let unwind_expr =
            Expression::Literal(crate::graph::expression::expression::LiteralValue::Int(1));
        assert!(strategy
            .validate_unwind(&unwind_expr, &unwind_context)
            .is_ok());
    }

    #[test]
    fn test_validate_yield() {
        let strategy = ExpressionValidationStrategy::new();

        let yield_context = YieldClauseContext {
            yield_columns: vec![YieldColumn::new(
                Expression::Literal(crate::graph::expression::expression::LiteralValue::Int(1)),
                "col1".to_string(),
            )],
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: Vec::new(),
            group_items: Vec::new(),
            need_gen_project: false,
            agg_output_column_names: Vec::new(),
            proj_output_column_names: Vec::new(),
            proj_cols: Vec::new(),
            paths: Vec::new(),
        };

        // 测试Yield子句验证
        assert!(strategy.validate_yield(&yield_context).is_ok());
    }

    #[test]
    fn test_single_path_pattern() {
        let strategy = ExpressionValidationStrategy::new();

        let mut match_context = MatchClauseContext {
            paths: Vec::new(),
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        // 测试单个路径模式验证
        let pattern =
            Expression::Literal(crate::graph::expression::expression::LiteralValue::Int(1));
        assert!(strategy
            .validate_single_path_pattern(&pattern, &mut match_context)
            .is_ok());
    }
}
