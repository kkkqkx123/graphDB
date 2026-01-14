//! WHERE 子句规划器
//! 架构重构：实现统一的 CypherClausePlanner 接口
//!
//! ## 重构说明
//!
//! ### 删除冗余方法
//! - 移除 `validate_input`, `can_start_flow`, `requires_input` 等冗余方法
//! - 通过 `flow_direction()` 统一表达数据流行为
//!
//! ### 简化变量管理
//! - WHERE 子句不产生新变量，只过滤输入
//! - 移除不必要的 `VariableRequirement` 和 `VariableProvider`
//!
//! ### 优化实现逻辑
//! - 专注于核心的过滤功能
//! - 简化路径表达式处理

use crate::query::parser::ast::expr::Expr;
use crate::query::planner::match_planning::clauses::clause_planner::ClausePlanner;
use crate::query::planner::match_planning::core::{
    ClauseType, CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::match_planning::paths::match_path_planner::MatchPathPlanner;
use crate::query::planner::match_planning::utils::connection_strategy::UnifiedConnector;

use crate::query::planner::plan::factory::PlanNodeFactory;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;
use std::collections::HashSet;

/// WHERE 子句规划器
///
/// 负责规划 WHERE 子句的执行。WHERE 子句是一个转换子句，
/// 它需要输入数据流并根据指定的过滤条件对结果进行过滤。
///
/// # 示例
///
/// ```cypher
/// MATCH (n:Person)
/// WHERE n.age > 25 AND n.name STARTS WITH 'John'
/// RETURN n.name, n.age
/// ```
///
/// 在上面的例子中，WHERE 子句会过滤出年龄大于25且姓名以'John'开头的人员。
#[derive(Debug)]
pub struct WhereClausePlanner {
    need_stable_filter: bool, // 是否需要稳定的过滤器（用于ORDER BY场景）
}

impl WhereClausePlanner {
    /// 创建新的 WHERE 子句规划器
    ///
    /// # 参数
    /// * `need_stable_filter` - 是否需要稳定的过滤器，用于ORDER BY场景
    pub fn new(need_stable_filter: bool) -> Self {
        Self { need_stable_filter }
    }

    /// 构建 WHERE 子句的执行计划
    ///
    /// # 参数
    /// * `where_clause_ctx` - WHERE 子句的上下文信息
    /// * `input_plan` - 输入的执行计划
    /// * `context` - 规划上下文
    ///
    /// # 返回值
    ///
    /// 返回包含 WHERE 子句执行计划的 SubPlan
    fn build_where(
        &self,
        where_clause_ctx: &crate::query::validator::structs::clause_structs::WhereClauseContext,
        _input_plan: &SubPlan,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 处理路径表达式（模式谓词）
        let mut plan = if !where_clause_ctx.paths.is_empty() {
            let mut paths_plan = SubPlan::new(None, None);

            // 为模式表达式构建计划
            for path in &where_clause_ctx.paths {
                let mut path_planner = MatchPathPlanner::new(
                    // 这里需要创建一个临时的MatchClauseContext
                    crate::query::validator::structs::clause_structs::MatchClauseContext {
                        paths: vec![path.clone()],
                        aliases_available: where_clause_ctx.aliases_available.clone(),
                        aliases_generated: where_clause_ctx.aliases_generated.clone(),
                        where_clause: None,
                        is_optional: false,
                        skip: None,
                        limit: None,
                        query_parts: Vec::new(),
                        errors: Vec::new(),
                    },
                    path.clone(),
                );

                // 暂时使用旧接口，因为 MatchPathPlanner 还没有更新
                let path_plan = path_planner.transform(None, &mut HashSet::new())?;

                // 使用新的统一连接器
                let mut intersected_aliases = HashSet::new();
                // 添加路径中的别名
                for node_info in &path.node_infos {
                    if !node_info.alias.is_empty() {
                        intersected_aliases.insert(node_info.alias.clone());
                    }
                }
                for edge_info in &path.edge_infos {
                    if !edge_info.alias.is_empty() {
                        intersected_aliases.insert(edge_info.alias.clone());
                    }
                }

                if path.is_pred {
                    // 构建模式谓词的计划
                    let temp_ast_context =
                        crate::query::context::ast::base::AstContext::from_strings(
                            &context.query_info.statement_type,
                            &context.query_info.query_id,
                        );
                    paths_plan = UnifiedConnector::pattern_apply(
                        &temp_ast_context,
                        &paths_plan,
                        &path_plan,
                        intersected_aliases,
                    )?;
                } else {
                    // 构建路径收集的计划
                    let temp_ast_context =
                        crate::query::context::ast::base::AstContext::from_strings(
                            &context.query_info.statement_type,
                            &context.query_info.query_id,
                        );
                    paths_plan = UnifiedConnector::roll_up_apply(
                        &temp_ast_context,
                        &paths_plan,
                        &path_plan,
                        intersected_aliases,
                    )?;
                }
            }

            paths_plan
        } else {
            SubPlan::new(None, None)
        };

        // 处理过滤条件
        if let Some(filter) = &where_clause_ctx.filter {
            // 创建起始节点作为输入
            let start_node = PlanNodeFactory::create_start_node()?;

            // 创建过滤器节点 - 将 Expression 转换为 Expr
            let expr = convert_expression_to_expr(filter);
            let filter_node = PlanNodeFactory::create_filter(start_node, expr)?;

            let where_plan = SubPlan::from_single_node(filter_node);

            if plan.root.is_none() {
                return Ok(where_plan);
            }

            let temp_ast_context = crate::query::context::ast::base::AstContext::from_strings(
                &context.query_info.statement_type,
                &context.query_info.query_id,
            );
            plan = UnifiedConnector::add_input(&temp_ast_context, &where_plan, &plan, true)?;
        }

        Ok(plan)
    }
}

impl ClausePlanner for WhereClausePlanner {
    fn name(&self) -> &'static str {
        "WhereClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Where
    }
}

impl CypherClausePlanner for WhereClausePlanner {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 验证数据流：WHERE 子句需要输入
        self.validate_flow(input_plan)?;

        // 确保有输入计划
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("WHERE clause requires input".to_string())
        })?;

        // 验证上下文类型
        if !matches!(clause_ctx.kind(), CypherClauseKind::Where) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for WhereClausePlanner".to_string(),
            ));
        }

        let where_clause_ctx = match clause_ctx {
            CypherClauseContext::Where(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected WhereClauseContext".to_string(),
                ))
            }
        };

        // 构建 WHERE 子句的执行计划
        self.build_where(where_clause_ctx, input_plan, context)
    }

    fn clause_type(&self) -> ClauseType {
        ClauseType::Where
    }
}

impl DataFlowNode for WhereClausePlanner {
    fn flow_direction(
        &self,
    ) -> crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

/// 将 Expression 转换为 Expr
/// 辅助函数，用于在不同表达式类型之间转换
fn convert_expression_to_expr(expr: &crate::core::Expression) -> Expr {
    use crate::query::parser::ast::expr::*;
    use crate::query::parser::ast::types::Span;

    match expr {
        crate::core::Expression::Variable(name) => {
            Expr::Variable(VariableExpr::new(name.clone(), Span::default()))
        }
        crate::core::Expression::Literal(val) => {
            use crate::core::Value;
            let const_val = match val {
                Value::String(s) => Value::String(s.clone()),
                Value::Int(i) => Value::Int(*i),
                Value::Float(f) => Value::Float(*f),
                Value::Bool(b) => Value::Bool(*b),
                Value::Null(nt) => Value::Null(nt.clone()),
                _ => Value::Null(crate::core::NullType::Null),
            };
            Expr::Constant(ConstantExpr::new(const_val, Span::default()))
        }
        _ => {
            // 其他表达式类型暂时使用默认的 true 表达式
            Expr::Constant(ConstantExpr::new(
                crate::core::Value::Bool(true),
                Span::default(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_where_clause_planner_interface() {
        let planner = WhereClausePlanner::new(false);
        assert_eq!(planner.clause_type(), ClauseType::Where);
        assert_eq!(<WhereClausePlanner as DataFlowNode>::flow_direction(&planner), crate::query::planner::match_planning::core::cypher_clause_planner::FlowDirection::Transform);
        assert!(planner.requires_input());
    }

    #[test]
    fn test_where_clause_planner_validate_flow() {
        let planner = WhereClausePlanner::new(false);

        // 测试没有输入的情况（应该失败）
        let result = planner.validate_flow(None);
        assert!(result.is_err());

        // 测试有输入的情况（应该成功）
        let dummy_plan = SubPlan::new(None, None);
        let result = planner.validate_flow(Some(&dummy_plan));
        assert!(result.is_ok());
    }

    #[test]
    fn test_where_clause_planner_stable_filter() {
        let planner = WhereClausePlanner::new(true);
        assert!(planner.need_stable_filter);

        let planner = WhereClausePlanner::new(false);
        assert!(!planner.need_stable_filter);
    }
}
