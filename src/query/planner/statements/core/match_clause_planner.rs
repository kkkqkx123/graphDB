/// MATCH子句规划器
/// 架构重构：实现统一的 CypherClausePlanner 接口
///
/// ## 重构说明
///
/// ### 删除冗余方法
/// - 移除 `validate_input`, `can_start_flow`, `requires_input` 等冗余方法
/// - 通过 `flow_direction()` 统一表达数据流行为
///
/// ### 优化上下文管理
/// - 使用 `VariableInfo` 替代简单的字符串映射
/// - 提供完整的变量生命周期管理
///
/// ### 简化实现逻辑
/// - 移除复杂的验证逻辑，内聚到接口中
/// - 专注于核心的路径处理和变量管理
use crate::core::Expression;
use crate::query::planner::connector::SegmentsConnector;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::statements::core::{
    ClauseType, CypherClausePlanner, DataFlowNode, PlanningContext, VariableInfo,
};

use crate::query::planner::plan::factory::PlanNodeFactory;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind, MatchClauseContext, Path};

/// MATCH子句规划器
/// 负责规划 MATCH 子句的执行，是数据流的起始点
///
/// MATCH 子句是 Cypher 查询的核心，用于匹配图中的模式。
/// 它可以包含多个路径，每个路径由节点和边组成。
#[derive(Debug)]
pub struct MatchClausePlanner {}

impl MatchClausePlanner {
    /// 创建新的 MATCH 子句规划器
    pub fn new() -> Self {
        Self {}
    }

    fn plan_path(
        &self,
        path: &Path,
        _context: &mut PlanningContext,
        _match_clause_ctx: &MatchClauseContext,
    ) -> Result<SubPlan, PlannerError> {
        let space_id = 1i32;

        let mut current_plan = SubPlan::new(None, None);

        for node_info in path.node_infos.iter() {
            let scan_node = crate::query::planner::plan::core::nodes::ScanVerticesNode::new(space_id);
            let node_plan = SubPlan::from_root(scan_node.clone().into_enum());

            current_plan = if let Some(existing_root) = current_plan.root.take() {
                SegmentsConnector::cross_join(
                    SubPlan::new(Some(existing_root), current_plan.tail),
                    node_plan,
                )?
            } else {
                node_plan
            };

            if let Some(filter) = &node_info.filter {
                let filter_node = crate::query::planner::plan::core::nodes::FilterNode::new(
                    scan_node.into_enum(),
                    filter.clone(),
                )?;
                current_plan = SubPlan::new(Some(filter_node.into_enum()), current_plan.tail);
            }
        }

        for edge_info in &path.edge_infos {
            let expand_node = crate::query::planner::plan::core::nodes::ExpandAllNode::new(
                space_id,
                edge_info.types.clone(),
                "both",
            );
            let edge_plan = SubPlan::from_root(expand_node.into_enum());

            current_plan = if let Some(existing_root) = current_plan.root.take() {
                SegmentsConnector::cross_join(
                    SubPlan::new(Some(existing_root), current_plan.tail),
                    edge_plan,
                )?
            } else {
                edge_plan
            };
        }

        Ok(current_plan)
    }
}

impl CypherClausePlanner for MatchClausePlanner {
    fn clause_type(&self) -> ClauseType {
        ClauseType::Match
    }

    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        // 验证数据流：MATCH 子句不应该有输入
        self.validate_flow(input_plan)?;

        // 验证上下文类型
        if !matches!(clause_ctx.kind(), CypherClauseKind::Match) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for MatchClausePlanner".to_string(),
            ));
        }

        let match_clause_ctx = match clause_ctx {
            CypherClauseContext::Match(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected MatchClauseContext".to_string(),
                ))
            }
        };

        // 验证 MATCH 子句上下文的完整性
        if match_clause_ctx.paths.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "MATCH 子句必须至少包含一个路径".to_string(),
            ));
        }

        // 处理路径
        let mut plan = SubPlan::new(None, None);

        for path in &match_clause_ctx.paths {
            let path_plan = self.plan_path(path, context, match_clause_ctx)?;

            // 更新上下文中的变量信息
            for node_info in &path.node_infos {
                if !node_info.alias.is_empty() {
                    let variable_info = VariableInfo {
                        name: node_info.alias.clone(),
                        var_type: "Vertex".to_string(),
                        source_clause: ClauseType::Match,
                        is_output: false,
                    };
                    context.add_variable(variable_info);
                }
            }

            for edge_info in &path.edge_infos {
                if !edge_info.alias.is_empty() {
                    let variable_info = VariableInfo {
                        name: edge_info.alias.clone(),
                        var_type: "Edge".to_string(),
                        source_clause: ClauseType::Match,
                        is_output: false,
                    };
                    context.add_variable(variable_info);
                }
            }

            // 连接路径计划
            plan = if let Some(existing_root) = plan.root.take() {
                SegmentsConnector::cross_join(
                    SubPlan::new(Some(existing_root), plan.tail),
                    path_plan,
                )?
            } else {
                path_plan
            };
        }

        // 处理分页（如果存在）
        if let Some(skip) = &match_clause_ctx.skip {
            let skip_value = match skip {
                Expression::Literal(crate::core::Value::Int(v)) => *v,
                _ => 0,
            };

            if skip_value > 0 {
                let skip_node = PlanNodeFactory::create_placeholder_node()?;
                plan = SubPlan::new(Some(skip_node.clone()), Some(skip_node));
            }
        }

        if let Some(limit) = &match_clause_ctx.limit {
            let limit_value = match limit {
                Expression::Literal(crate::core::Value::Int(v)) => *v,
                _ => i64::MAX,
            };

            if limit_value != i64::MAX {
                let limit_node = PlanNodeFactory::create_placeholder_node()?;
                plan = SubPlan::new(Some(limit_node.clone()), Some(limit_node));
            }
        }

        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::validator::structs::{NodeInfo, Path, PathYieldType};

    #[test]
    fn test_match_clause_planner_interface() {
        let planner = MatchClausePlanner::new();
        assert_eq!(planner.clause_type(), ClauseType::Match);
        assert_eq!(<MatchClausePlanner as DataFlowNode>::flow_direction(&planner), crate::query::planner::statements::core::cypher_clause_planner::FlowDirection::Source);
        assert!(!planner.requires_input());
    }

    #[test]
    fn test_match_clause_planner_validate_flow() {
        let planner = MatchClausePlanner::new();

        // 测试有输入的情况（应该失败）
        let dummy_plan = SubPlan::new(None, None);
        let result = planner.validate_flow(Some(&dummy_plan));
        assert!(result.is_err());

        // 测试没有输入的情况（应该成功）
        let result = planner.validate_flow(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_match_clause_planner_context_variables() {
        let node_info = NodeInfo {
            alias: "n".to_string(),
            labels: vec!["Person".to_string()],
            props: None,
            anonymous: false,
            filter: None,
            tids: vec![1],
            label_props: vec![None],
        };

        let path = Path {
            alias: "p".to_string(),
            anonymous: false,
            gen_path: false,
            path_type: PathYieldType::Default,
            node_infos: vec![node_info],
            edge_infos: vec![],
            path_build: None,
            is_pred: false,
            is_anti_pred: false,
            compare_variables: vec![],
            collect_variable: String::new(),
            roll_up_apply: false,
        };

        let planner = MatchClausePlanner::new();

        let query_info =
            crate::query::planner::statements::core::cypher_clause_planner::QueryInfo {
                query_id: "test".to_string(),
                statement_type: "MATCH".to_string(),
            };
        let mut context = PlanningContext::new(query_info);

        // 创建一个简单的 MATCH 上下文
        let match_clause_ctx = crate::query::validator::structs::MatchClauseContext {
            paths: vec![path],
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
            query_parts: Vec::new(),
            errors: Vec::new(),
        };

        let clause_ctx = CypherClauseContext::Match(match_clause_ctx);

        // 执行转换以更新上下文
        let _result = planner.transform(&clause_ctx, None, &mut context);

        // 验证变量被添加到上下文
        assert!(context.has_variable("n"));

        if let Some(variable) = context.get_variable("n") {
            assert_eq!(variable.name, "n");
            assert_eq!(variable.var_type, "Vertex");
            assert_eq!(variable.source_clause, ClauseType::Match);
            assert!(!variable.is_output);
        }
    }
}

impl DataFlowNode for MatchClausePlanner {
    fn flow_direction(
        &self,
    ) -> crate::query::planner::statements::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}
