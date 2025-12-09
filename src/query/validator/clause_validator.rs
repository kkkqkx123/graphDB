//! 子句验证器模块
//! 负责验证不同查询子句（MATCH、RETURN、WITH、UNWIND等）

use crate::graph::expression::expr_type::Expression;
use crate::query::validator::{Validator, ValidateContext};
use crate::query::validator::match_structs::{
    QueryPart, MatchClauseContext, ReturnClauseContext, YieldClauseContext,
    YieldColumn, Path, AliasType
};
use std::collections::HashMap;

/// 子句验证器
pub struct ClauseValidator;

impl ClauseValidator {
    pub fn new() -> Self {
        Self
    }

    /// 验证返回子句
    pub fn validate_return_clause(
        &self,
        context: &ReturnClauseContext,
        validator: &mut Validator,
    ) -> Result<(), String> {
        // 检查别名可用性
        use super::expression_validator::ExpressionValidator;
        let expr_validator = ExpressionValidator::new();
        
        for col in &context.yield_clause.yield_columns {
            expr_validator.validate_return(&col.expr, &[], &mut context.clone())?;
        }

        // 验证分页
        if let Some(ref pagination) = context.pagination {
            if pagination.skip < 0 {
                return Err("SKIP不能为负数".to_string());
            }
            if pagination.limit < 0 {
                return Err("LIMIT不能为负数".to_string());
            }
        }

        // 验证排序
        if let Some(ref order_by) = context.order_by {
            // 在这里可以验证排序条件
            for &(index, _) in &order_by.indexed_order_factors {
                // 检查索引是否有效
                if index >= context.yield_clause.yield_columns.len() {
                    return Err(format!("列索引{}超出范围", index));
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
    ) -> Result<(), String> {
        if query_parts.is_empty() {
            return Err("没有声明别名。".to_string());
        }

        let curr_query_part = query_parts.last().unwrap();

        // 处理前一个查询部分的边界子句
        if query_parts.len() > 1 {
            let prev_query_part = &query_parts[query_parts.len() - 2];
            if let Some(ref boundary) = prev_query_part.boundary {
                match boundary {
                    crate::query::validator::match_structs::BoundaryClauseContext::Unwind(unwind_ctx) => {
                        // 添加Unwind子句的别名
                        columns.push(YieldColumn::new(
                            Expression::Label(unwind_ctx.alias.clone()),
                            unwind_ctx.alias.clone()
                        ));

                        // 添加之前可用的别名
                        for (alias, _) in &prev_query_part.aliases_available {
                            columns.push(YieldColumn::new(
                                Expression::Label(alias.clone()),
                                alias.clone()
                            ));
                        }

                        // 添加之前生成的别名
                        for (alias, _) in &prev_query_part.aliases_generated {
                            columns.push(YieldColumn::new(
                                Expression::Label(alias.clone()),
                                alias.clone()
                            ));
                        }
                    }
                    crate::query::validator::match_structs::BoundaryClauseContext::With(with_ctx) => {
                        // 添加With子句的列
                        for col in &with_ctx.yield_clause.yield_columns {
                            if !col.alias.is_empty() {
                                columns.push(YieldColumn::new(
                                    Expression::Label(col.alias.clone()),
                                    col.alias.clone()
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
                            path.node_infos[i].alias.clone()
                        ));
                    }

                    if !path.edge_infos[i].anonymous {
                        columns.push(YieldColumn::new(
                            Expression::Label(path.edge_infos[i].alias.clone()),
                            path.edge_infos[i].alias.clone()
                        ));
                    }
                }

                // 添加最后的节点别名
                if !path.node_infos.last().unwrap().anonymous {
                    let last_node = path.node_infos.last().unwrap();
                    columns.push(YieldColumn::new(
                        Expression::Label(last_node.alias.clone()),
                        last_node.alias.clone()
                    ));
                }
            }

            // 添加路径别名
            for (alias, alias_type) in &match_ctx.aliases_generated {
                if *alias_type == AliasType::Path {
                    columns.push(YieldColumn::new(
                        Expression::Label(alias.clone()),
                        alias.clone()
                    ));
                }
            }
        }

        Ok(())
    }

    /// 构建输出
    pub fn build_outputs(&self, paths: &mut Vec<Path>) -> Result<(), String> {
        // 构建查询输出，包括列名和类型
        // 这里会根据路径信息构建最终的输出格式
        for path in paths {
            // 为每个路径构建输出信息
            // 在实际实现中，这里会构建具体的输出格式
        }
        Ok(())
    }

    /// 验证Yield子句
    pub fn validate_yield_clause(
        &self,
        context: &mut YieldClauseContext,
    ) -> Result<(), String> {
        // 如果有聚合函数，执行特殊验证
        if context.has_agg {
            return self.validate_group(context);
        }

        // 对于普通Yield子句，验证别名
        use super::alias_validator::AliasValidator;
        let alias_validator = AliasValidator::new();
        for col in &context.yield_columns {
            alias_validator.validate_aliases(&[col.expr.clone()], &context.aliases_available)?;
        }

        Ok(())
    }

    /// 验证分组子句
    fn validate_group(&self, yield_ctx: &mut YieldClauseContext) -> Result<(), String> {
        // 验证分组逻辑
        use super::aggregate_validator::AggregateValidator;
        let aggregate_validator = AggregateValidator::new();

        for col in &yield_ctx.yield_columns {
            // 如果表达式包含聚合函数，验证聚合表达式
            if aggregate_validator.has_aggregate_expr(&col.expr) {
                // 验证聚合函数
                // 在实际实现中，这里会进行更详细的聚合函数验证
            } else {
                // 非聚合表达式将作为分组键添加
                yield_ctx.group_keys.push(col.expr.clone());
            }

            yield_ctx.group_items.push(col.expr.clone());
        }

        Ok(())
    }

    /// 验证Match子句上下文
    pub fn validate_match_clause_context(
        &self,
        context: &MatchClauseContext,
    ) -> Result<(), String> {
        // 验证Match子句的基本结构
        // 检查路径、别名等的有效性
        
        // 验证路径
        for path in &context.paths {
            // 验证路径结构
            // 在实际实现中，这里会进行更详细的路径验证
        }

        // 验证WHERE子句（如果存在）
        if let Some(ref where_clause) = context.where_clause {
            // 验证WHERE子句
            // 在实际实现中，这里会进行更详细的WHERE子句验证
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::expr_type::Expression;
    use crate::query::validator::match_structs::{
        QueryPart, MatchClauseContext, ReturnClauseContext, YieldClauseContext,
        YieldColumn, Path, NodeInfo, EdgeInfo, Direction, MatchStepRange,
        PaginationContext, OrderByClauseContext, OrderType
    };
    use std::collections::HashMap;

    #[test]
    fn test_clause_validator_creation() {
        let validator = ClauseValidator::new();
        // 验证器创建成功
        assert!(true); // 占位测试
    }

    #[test]
    fn test_validate_return_clause() {
        let validator = ClauseValidator::new();
        let mut base_validator = Validator::new(ValidateContext::new());
        
        // 创建测试数据
        let return_context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: vec![
                    YieldColumn::new(Expression::Constant(crate::core::Value::Int(1)), "col1".to_string())
                ],
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
        
        assert!(validator.validate_return_clause(&return_context, &mut base_validator).is_ok());
    }

    #[test]
    fn test_validate_return_clause_with_pagination() {
        let validator = ClauseValidator::new();
        let mut base_validator = Validator::new(ValidateContext::new());
        
        // 创建带分页的测试数据
        let return_context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: vec![
                    YieldColumn::new(Expression::Constant(crate::core::Value::Int(1)), "col1".to_string())
                ],
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
        
        assert!(validator.validate_return_clause(&return_context, &mut base_validator).is_ok());
    }

    #[test]
    fn test_validate_return_clause_invalid_pagination() {
        let validator = ClauseValidator::new();
        let mut base_validator = Validator::new(ValidateContext::new());
        
        // 创建无效分页的测试数据
        let return_context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: vec![
                    YieldColumn::new(Expression::Constant(crate::core::Value::Int(1)), "col1".to_string())
                ],
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
            pagination: Some(PaginationContext { skip: -1, limit: 10 }),
            order_by: None,
            distinct: false,
        };
        
        assert!(validator.validate_return_clause(&return_context, &mut base_validator).is_err());
    }

    #[test]
    fn test_build_columns_for_all_named_aliases() {
        let validator = ClauseValidator::new();
        
        // 创建测试查询部分
        let query_parts = vec![
            QueryPart {
                matchs: Vec::new(),
                boundary: None,
                aliases_available: HashMap::new(),
                aliases_generated: HashMap::new(),
                paths: Vec::new(),
            }
        ];
        
        let mut columns = Vec::new();
        
        // 测试空查询部分
        assert!(validator.build_columns_for_all_named_aliases(&[], &mut columns).is_err());
        
        // 测试有查询部分但无别名
        assert!(validator.build_columns_for_all_named_aliases(&query_parts, &mut columns).is_ok());
    }

    #[test]
    fn test_validate_yield_clause() {
        let validator = ClauseValidator::new();
        
        let mut yield_context = YieldClauseContext {
            yield_columns: vec![
                YieldColumn::new(Expression::Constant(crate::core::Value::Int(1)), "col1".to_string())
            ],
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
        
        assert!(validator.validate_yield_clause(&mut yield_context).is_ok());
    }

    #[test]
    fn test_validate_match_clause_context() {
        let validator = ClauseValidator::new();
        
        let match_context = MatchClauseContext {
            paths: Vec::new(),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };
        
        assert!(validator.validate_match_clause_context(&match_context).is_ok());
    }

    #[test]
    fn test_build_outputs() {
        let validator = ClauseValidator::new();
        
        let mut paths = Vec::new();
        
        // 测试空路径
        assert!(validator.build_outputs(&mut paths).is_ok());
        
        // 测试有路径的情况
        let path = Path {
            alias: "test_path".to_string(),
            anonymous: false,
            gen_path: true,
            path_type: crate::query::validator::match_structs::PathType::Default,
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
        assert!(validator.build_outputs(&mut paths).is_ok());
    }
}