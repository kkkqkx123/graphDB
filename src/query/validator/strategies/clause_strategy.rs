//! 子句验证策略（合并版）
//! 负责验证不同查询子句（MATCH、RETURN、WITH、UNWIND等）
//! 合并原expression_validator和clause_validator的功能

use super::super::structs::*;
use super::super::validation_interface::*;
use crate::core::Expression;

/// 子句验证策略
pub struct ClauseValidationStrategy;

impl ClauseValidationStrategy {
    pub fn new() -> Self {
        Self
    }

    /// 验证返回子句
    pub fn validate_return_clause(
        &self,
        context: &ReturnClauseContext,
    ) -> Result<(), ValidationError> {
        // 检查别名可用性
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();

        for col in &context.yield_clause.yield_columns {
            alias_validator.validate_aliases(&[col.expr.clone()], &context.aliases_available)?;
        }

        // 验证分页
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

        // 验证排序
        if let Some(ref order_by) = context.order_by {
            // 在这里可以验证排序条件
            for &(index, _) in &order_by.indexed_order_factors {
                // 检查索引是否有效
                if index >= context.yield_clause.yield_columns.len() {
                    return Err(ValidationError::new(
                        format!("列索引{}超出范围", index),
                        ValidationErrorType::PaginationError,
                    ));
                }
            }
        }

        Ok(())
    }

    /// 构建所有命名别名的列
    pub fn build_columns_for_all_named_aliases(
        &self,
        query_parts: &[QueryPart],
        columns: &mut Vec<YieldColumn>,
    ) -> Result<(), ValidationError> {
        if query_parts.is_empty() {
            return Err(ValidationError::new(
                "没有声明别名。".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let curr_query_part = query_parts.last().ok_or_else(|| {
            ValidationError::new(
                "Query parts should not be empty".to_string(),
                ValidationErrorType::SemanticError,
            )
        })?;

        // 处理前一个查询部分的边界子句
        if query_parts.len() > 1 {
            let prev_query_part = &query_parts[query_parts.len() - 2];
            if let Some(ref boundary) = prev_query_part.boundary {
                match boundary {
                    BoundaryClauseContext::Unwind(unwind_ctx) => {
                        // 添加Unwind子句的别名
                        columns.push(YieldColumn::new(
                            Expression::Label(unwind_ctx.alias.clone()),
                            unwind_ctx.alias.clone(),
                        ));

                        // 添加之前可用的别名
                        for (alias, _) in &prev_query_part.aliases_available {
                            columns.push(YieldColumn::new(
                                Expression::Label(alias.clone()),
                                alias.clone(),
                            ));
                        }

                        // 添加之前生成的别名
                        for (alias, _) in &prev_query_part.aliases_generated {
                            columns.push(YieldColumn::new(
                                Expression::Label(alias.clone()),
                                alias.clone(),
                            ));
                        }
                    }
                    BoundaryClauseContext::With(with_ctx) => {
                        // 添加With子句的列
                        for col in &with_ctx.yield_clause.yield_columns {
                            if !col.alias.is_empty() {
                                columns.push(YieldColumn::new(
                                    Expression::Label(col.alias.clone()),
                                    col.alias.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        // 处理当前查询部分的匹配子句
        for match_ctx in &curr_query_part.matchs {
            for path in &match_ctx.paths {
                // 添加路径中节点和边的别名
                for i in 0..path.edge_infos.len() {
                    if !path.node_infos[i].anonymous {
                        columns.push(YieldColumn::new(
                            Expression::Label(path.node_infos[i].alias.clone()),
                            path.node_infos[i].alias.clone(),
                        ));
                    }

                    if !path.edge_infos[i].anonymous {
                        columns.push(YieldColumn::new(
                            Expression::Label(path.edge_infos[i].alias.clone()),
                            path.edge_infos[i].alias.clone(),
                        ));
                    }
                }

                // 添加最后的节点别名
                let last_node = path.node_infos.last().ok_or_else(|| {
                    ValidationError::new(
                        "Path should have at least one node".to_string(),
                        ValidationErrorType::SemanticError,
                    )
                })?;
                if !last_node.anonymous {
                    columns.push(YieldColumn::new(
                        Expression::Label(last_node.alias.clone()),
                        last_node.alias.clone(),
                    ));
                }
            }

            // 添加路径别名
            for (alias, alias_type) in &match_ctx.aliases_generated {
                if *alias_type == AliasType::Path {
                    columns.push(YieldColumn::new(
                        Expression::Label(alias.clone()),
                        alias.clone(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// 构建输出
    pub fn build_outputs(&self, paths: &mut Vec<Path>) -> Result<(), ValidationError> {
        // 构建查询输出，包括列名和类型
        // 这里会根据路径信息构建最终的输出格式
        for _path in paths {
            // 为每个路径构建输出信息
            // 在实际实现中，这里会构建具体的输出格式
        }
        Ok(())
    }

    /// 验证Yield子句
    pub fn validate_yield_clause(
        &self,
        context: &YieldClauseContext,
    ) -> Result<(), ValidationError> {
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

    /// 验证Match子句上下文
    pub fn validate_match_clause_context(
        &self,
        context: &MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证Match子句的基本结构
        // 检查路径、别名等的有效性

        // 验证路径
        for _path in &context.paths {
            // 验证路径结构
            // 在实际实现中，这里会进行更详细的路径验证
        }

        // 验证WHERE子句（如果存在）
        if let Some(ref _where_clause) = context.where_clause {
            // 验证WHERE子句
            // 在实际实现中，这里会进行更详细的WHERE子句验证
        }

        Ok(())
    }
}

impl ValidationStrategy for ClauseValidationStrategy {
    fn validate(&self, context: &dyn ValidationContext) -> Result<(), ValidationError> {
        // 遍历所有查询部分，验证子句
        for query_part in context.get_query_parts() {
            // 验证Match子句
            for match_ctx in &query_part.matchs {
                self.validate_match_clause_context(match_ctx)?;
            }

            // 验证边界子句
            if let Some(boundary) = &query_part.boundary {
                match boundary {
                    BoundaryClauseContext::With(with_ctx) => {
                        self.validate_yield_clause(&with_ctx.yield_clause)?;
                    }
                    BoundaryClauseContext::Unwind(_unwind_ctx) => {
                        // UNWIND子句的验证在表达式策略中处理
                    }
                }
            }
        }

        Ok(())
    }

    fn strategy_type(&self) -> ValidationStrategyType {
        ValidationStrategyType::Clause
    }

    fn strategy_name(&self) -> &'static str {
        "ClauseValidationStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use std::collections::HashMap;

    #[test]
    fn test_clause_validation_strategy_creation() {
        let strategy = ClauseValidationStrategy::new();
        assert_eq!(strategy.strategy_type(), ValidationStrategyType::Clause);
        assert_eq!(strategy.strategy_name(), "ClauseValidationStrategy");
    }

    #[test]
    fn test_validate_return_clause() {
        let strategy = ClauseValidationStrategy::new();

        // 创建测试数据
        let return_context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: vec![YieldColumn::new(
                    Expression::Literal(crate::core::Value::Int(1)),
                    "col1".to_string(),
                )],
                aliases_available: HashMap::new(),
                aliases_generated: HashMap::new(),
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
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
        };

        assert!(strategy.validate_return_clause(&return_context).is_ok());
    }

    #[test]
    fn test_validate_return_clause_with_pagination() {
        let strategy = ClauseValidationStrategy::new();

        // 创建带分页的测试数据
        let return_context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: vec![YieldColumn::new(
                    Expression::Literal(crate::core::Value::Int(1)),
                    "col1".to_string(),
                )],
                aliases_available: HashMap::new(),
                aliases_generated: HashMap::new(),
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
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            pagination: Some(PaginationContext { skip: 0, limit: 10 }),
            order_by: None,
            distinct: false,
        };

        assert!(strategy.validate_return_clause(&return_context).is_ok());
    }

    #[test]
    fn test_validate_return_clause_invalid_pagination() {
        let strategy = ClauseValidationStrategy::new();

        // 创建无效分页的测试数据
        let return_context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: vec![YieldColumn::new(
                    Expression::Literal(crate::core::Value::Int(1)),
                    "col1".to_string(),
                )],
                aliases_available: HashMap::new(),
                aliases_generated: HashMap::new(),
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
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            pagination: Some(PaginationContext {
                skip: -1,
                limit: 10,
            }),
            order_by: None,
            distinct: false,
        };

        assert!(strategy.validate_return_clause(&return_context).is_err());
    }

    #[test]
    fn test_build_columns_for_all_named_aliases() {
        let strategy = ClauseValidationStrategy::new();

        // 创建测试查询部分
        let query_parts = vec![QueryPart {
            matchs: Vec::new(),
            boundary: None,
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: Vec::new(),
        }];

        let mut columns = Vec::new();

        // 测试空查询部分
        assert!(strategy
            .build_columns_for_all_named_aliases(&[], &mut columns)
            .is_err());

        // 测试有查询部分但无别名
        assert!(strategy
            .build_columns_for_all_named_aliases(&query_parts, &mut columns)
            .is_ok());
    }

    #[test]
    fn test_validate_yield_clause() {
        let strategy = ClauseValidationStrategy::new();

        let mut yield_context = YieldClauseContext {
            yield_columns: vec![YieldColumn::new(
                Expression::Literal(crate::core::Value::Int(1)),
                "col1".to_string(),
            )],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
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

        assert!(strategy.validate_yield_clause(&mut yield_context).is_ok());
    }

    #[test]
    fn test_validate_match_clause_context() {
        let strategy = ClauseValidationStrategy::new();

        let match_context = MatchClauseContext {
            paths: Vec::new(),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        assert!(strategy
            .validate_match_clause_context(&match_context)
            .is_ok());
    }

    #[test]
    fn test_build_outputs() {
        let strategy = ClauseValidationStrategy::new();

        let mut paths = Vec::new();

        // 测试空路径
        assert!(strategy.build_outputs(&mut paths).is_ok());

        // 测试有路径的情况
        let path = Path {
            alias: "test_path".to_string(),
            anonymous: false,
            gen_path: true,
            path_type: PathType::Default,
            node_infos: Vec::new(),
            edge_infos: Vec::new(),
            path_build: None,
            is_pred: false,
            is_anti_pred: false,
            compare_variables: Vec::new(),
            collect_variable: String::new(),
            roll_up_apply: false,
        };

        paths.push(path);
        assert!(strategy.build_outputs(&mut paths).is_ok());
    }
}
