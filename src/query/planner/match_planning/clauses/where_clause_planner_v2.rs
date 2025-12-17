//! 新的 WHERE子句规划器
//! 实现新的 CypherClausePlanner 接口

use crate::query::planner::match_planning::core::{
    CypherClausePlanner, ClauseType, PlanningContext
};
use crate::query::planner::match_planning::paths::match_path_planner::MatchPathPlanner;
use crate::query::planner::match_planning::utils::connector::SegmentsConnector;
use crate::query::planner::plan::{PlanNodeKind, SingleInputNode, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};
use std::sync::Arc;

/// 新的 WHERE子句规划器
/// 实现新的 CypherClausePlanner 接口
#[derive(Debug)]
pub struct WhereClausePlannerV2 {
    need_stable_filter: bool, // 是否需要稳定的过滤器（用于ORDER BY场景）
}

impl WhereClausePlannerV2 {
    pub fn new(need_stable_filter: bool) -> Self {
        Self { need_stable_filter }
    }
}

impl CypherClausePlanner for WhereClausePlannerV2 {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        _input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, crate::query::planner::planner::PlannerError> {
        // 验证输入
        self.validate_input(_input_plan)?;
        
        // 确保有输入计划
        let _input_plan = _input_plan.ok_or_else(|| {
            PlannerError::missing_input("WHERE clause requires input".to_string())
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

        // 处理路径表达式（模式谓词）
        let mut plan = if !where_clause_ctx.paths.is_empty() {
            let mut paths_plan = SubPlan::new(None, None);

            // 为模式表达式构建计划
            for path in &where_clause_ctx.paths {
                let mut path_planner = MatchPathPlanner::new(
                    // 这里需要创建一个临时的MatchClauseContext
                    crate::query::validator::structs::MatchClauseContext {
                        paths: vec![path.clone()],
                        aliases_available: where_clause_ctx.aliases_available.clone(),
                        aliases_generated: where_clause_ctx.aliases_generated.clone(),
                        where_clause: None,
                        is_optional: false,
                        skip: None,
                        limit: None,
                    },
                    path.clone(),
                );

                // 暂时使用旧接口，因为 MatchPathPlanner 还没有更新
                let path_plan = path_planner.transform(None, &mut std::collections::HashSet::new())?;

                let connector = SegmentsConnector::new();
                if path.is_pred {
                    // 构建模式谓词的计划
                    paths_plan = connector.pattern_apply(paths_plan, path_plan, path);
                } else {
                    // 构建路径收集的计划
                    paths_plan = connector.roll_up_apply(paths_plan, path_plan, path);
                }
            }

            paths_plan
        } else {
            SubPlan::new(None, None)
        };

        // 处理过滤条件
        if let Some(filter) = &where_clause_ctx.filter {
            let mut where_plan = SubPlan::new(None, None);

            // 创建过滤器节点
            let filter_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::Filter,
                create_empty_node()?,
            ));

            // 设置过滤条件表达式
            // 这里需要根据filter表达式创建相应的计划节点
            // TODO: 实现完整的过滤逻辑
            let _ = filter; // 暂时避免未使用警告

            where_plan.root = Some(filter_node.clone());
            where_plan.tail = Some(filter_node);

            if plan.root.is_none() {
                return Ok(where_plan);
            }

            let connector = SegmentsConnector::new();
            plan = connector.add_input(where_plan, plan, true);
        }

        Ok(plan)
    }
    
    fn validate_input(&self, input_plan: Option<&SubPlan>) -> Result<(), crate::query::planner::planner::PlannerError> {
        if input_plan.is_none() {
            return Err(PlannerError::missing_input(
                "WHERE clause requires input from previous clauses".to_string()
            ));
        }
        Ok(())
    }
    
    fn clause_type(&self) -> ClauseType {
        ClauseType::Transform
    }
    
    fn can_start_flow(&self) -> bool {
        false  // WHERE 不能开始数据流
    }
    
    fn requires_input(&self) -> bool {
        true   // WHERE 需要输入
    }
    
    fn input_requirements(&self) -> Vec<crate::query::planner::match_planning::core::VariableRequirement> {
        // WHERE 子句需要输入数据，但不强制要求特定变量
        vec![]
    }
    
    fn output_provides(&self) -> Vec<crate::query::planner::match_planning::core::VariableProvider> {
        // WHERE 子句不产生新的变量，只是过滤输入
        vec![]
    }
}

/// 创建空节点
fn create_empty_node() -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::SingleDependencyNode;

    // 创建一个空的计划节点作为占位符
    Ok(Arc::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::match_planning::core::ClauseType;
    
    #[test]
    fn test_where_clause_planner_v2_interface() {
        let planner = WhereClausePlannerV2::new(false);
        assert_eq!(planner.clause_type(), ClauseType::Transform);
        assert!(!planner.can_start_flow());
        assert!(planner.requires_input());
    }
    
    #[test]
    fn test_where_clause_planner_v2_validate_input() {
        let planner = WhereClausePlannerV2::new(false);
        
        // 测试没有输入的情况
        let result = planner.validate_input(None);
        assert!(result.is_err());
        
        // 测试有输入的情况
        let dummy_plan = SubPlan::new(None, None);
        let result = planner.validate_input(Some(&dummy_plan));
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_where_clause_planner_v2_stable_filter() {
        let planner = WhereClausePlannerV2::new(true);
        assert!(planner.need_stable_filter);
        
        let planner = WhereClausePlannerV2::new(false);
        assert!(!planner.need_stable_filter);
    }
}