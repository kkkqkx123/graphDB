//! MATCH 规划器
//! 架构重构：使用新的统一子句规划器接口和数据流管理
//!
//! ## 重构说明
//!
//! ### 使用新的数据流管理
//! - 使用 `DataFlowManager` 替代复杂的 `DataFlowValidator`
//! - 简化数据流验证逻辑
//!
//! ### 优化上下文传播
//! - 使用 `ContextPropagator` 进行统一的上下文传播
//! - 改进变量生命周期管理
//!
//! ### 简化规划流程
//! - 移除复杂的验证步骤
/// - 专注于核心的规划逻辑
use crate::query::context::ast::AstContext;
use crate::query::planner::statements::clauses::{
    ReturnClausePlanner, WhereClausePlanner, WithClausePlanner,
};
use crate::query::planner::statements::core::{
    ContextPropagator, CypherClausePlanner, DataFlowManager, MatchClausePlanner, PlanningContext,
    QueryInfo,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::validator::structs::CypherClauseContext;

/// MATCH 规划器
///
/// 使用新的子句规划器接口和数据流管理机制。
/// 负责将 MATCH 查询转换为可执行的执行计划。
#[derive(Debug)]
pub struct MatchPlanner {
    query_context: AstContext,
}

impl MatchPlanner {
    pub fn new(query_context: AstContext) -> Self {
        Self { query_context }
    }

    pub fn make() -> Box<dyn Planner> {
        // 创建一个默认的查询上下文
        let query_context = AstContext::from_strings("MATCH", "MATCH (n)");
        Box::new(Self::new(query_context))
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        // 检查是否是 MATCH 语句
        ast_ctx.statement_type().to_uppercase() == "MATCH"
    }

    /// 解析查询并构建子句上下文列表
    fn parse_clauses(&self) -> Result<Vec<CypherClauseContext>, PlannerError> {
        // 这里应该解析查询文本并构建子句上下文
        // 暂时返回一个简单的 MATCH 子句
        let match_clause = crate::query::validator::structs::MatchClauseContext {
            paths: vec![],
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
            query_parts: Vec::new(),
            errors: Vec::new(),
        };

        Ok(vec![CypherClauseContext::Match(match_clause)])
    }
}

impl Planner for MatchPlanner {
    fn transform(&mut self, _ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 解析查询子句
        let clauses = self.parse_clauses()?;

        if clauses.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "No clauses found in query".to_string(),
            ));
        }

        // 创建查询信息
        let query_info = QueryInfo {
            query_id: "match_query".to_string(),
            statement_type: "MATCH".to_string(),
        };

        // 创建规划上下文
        let mut context = PlanningContext::new(query_info);

        // 创建子句规划器
        let mut clause_planners: Vec<Box<dyn CypherClausePlanner>> = Vec::new();

        for clause_ctx in &clauses {
            match clause_ctx.kind() {
                crate::query::validator::structs::CypherClauseKind::Match => {
                    if let CypherClauseContext::Match(match_ctx) = clause_ctx {
                        clause_planners
                            .push(Box::new(MatchClausePlanner::new(match_ctx.paths.clone())));
                    }
                }
                crate::query::validator::structs::CypherClauseKind::Where => {
                    clause_planners.push(Box::new(WhereClausePlanner::new(None)));
                }
                crate::query::validator::structs::CypherClauseKind::With => {
                    clause_planners.push(Box::new(WithClausePlanner::new()));
                }
                crate::query::validator::structs::CypherClauseKind::Return => {
                    clause_planners.push(Box::new(ReturnClausePlanner::new()));
                }
                _ => {
                    return Err(PlannerError::UnsupportedOperation(format!(
                        "Unsupported clause kind: {:?}",
                        clause_ctx.kind()
                    )));
                }
            }
        }

        // 验证查询的数据流 - 使用新的 DataFlowManager
        let clause_planner_refs: Vec<&dyn CypherClausePlanner> =
            clause_planners.iter().map(|p| p.as_ref()).collect();
        DataFlowManager::validate_clause_sequence(&clause_planner_refs)?;

        // 创建上下文传播器
        let context_propagator = ContextPropagator;

        // 执行规划
        let mut current_plan: Option<SubPlan> = None;

        for (i, planner) in clause_planners.iter().enumerate() {
            // 传播上下文到当前子句
            let _clause_context =
                context_propagator.propagate_to_clause(&context, planner.clause_type());

            let input_plan = current_plan.as_ref();
            let plan = planner.transform(&clauses[i], input_plan, &mut context)?;
            current_plan = Some(plan);
        }

        current_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("Failed to generate execution plan".to_string())
        })
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::ast::AstContext;

    #[test]
    fn test_match_planner_creation() {
        let query_ctx = AstContext::from_strings("MATCH", "MATCH (n)");
        let planner = MatchPlanner::new(query_ctx);
        assert_eq!(planner.query_context.statement_type(), "MATCH");
    }

    #[test]
    fn test_match_planner_make() {
        let planner = MatchPlanner::make();
        assert!(planner.match_planner(&AstContext::from_strings("MATCH", "MATCH (n)")));
        assert!(!planner.match_planner(&AstContext::from_strings("GO", "GO 1 TO 2")));
    }

    #[test]
    fn test_match_planner_match_ast_ctx() {
        assert!(MatchPlanner::match_ast_ctx(&AstContext::from_strings(
            "MATCH",
            "MATCH (n)"
        )));
        assert!(MatchPlanner::match_ast_ctx(&AstContext::from_strings(
            "match",
            "match (n)"
        )));
        assert!(!MatchPlanner::match_ast_ctx(&AstContext::from_strings(
            "GO",
            "GO 1 TO 2"
        )));
    }
}
