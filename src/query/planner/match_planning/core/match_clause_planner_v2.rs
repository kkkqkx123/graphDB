//! 新的 MATCH子句规划器
//! 实现新的 CypherClausePlanner 接口

use crate::query::planner::match_planning::core::{
    CypherClausePlanner, ClauseType, PlanningContext
};
use crate::query::planner::match_planning::utils::finder::Finder;
use crate::query::planner::plan::{PlanNodeKind, SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{CypherClauseContext, CypherClauseKind};
use crate::graph::expression::Expression;
use std::collections::HashSet;

/// 新的 MATCH子句规划器
/// 实现新的 CypherClausePlanner 接口
#[derive(Debug)]
pub struct MatchClausePlannerV2 {
    paths: Vec<crate::query::validator::structs::Path>,
}

impl MatchClausePlannerV2 {
    pub fn new(paths: Vec<crate::query::validator::structs::Path>) -> Self {
        Self { paths }
    }
}

impl CypherClausePlanner for MatchClausePlannerV2 {
    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, crate::query::planner::planner::PlannerError> {
        // 验证输入
        self.validate_input(input_plan)?;
        
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
        let mut available_aliases = HashSet::new();

        for path in &match_clause_ctx.paths {
            // 创建查找器
            let _finder = Finder::new();
            
            // 暂时创建一个简单的计划，因为 find_path 方法不存在
            // TODO: 实现路径处理逻辑
            let path_plan = SubPlan::new(None, None);
            
            // 更新可用的别名
            for node_info in &path.node_infos {
                if !node_info.alias.is_empty() {
                    available_aliases.insert(node_info.alias.clone());
                    context.add_variable(node_info.alias.clone());
                }
            }
            
            for edge_info in &path.edge_infos {
                if !edge_info.alias.is_empty() {
                    available_aliases.insert(edge_info.alias.clone());
                    context.add_variable(edge_info.alias.clone());
                }
            }
            
            // 连接路径计划
            if plan.root.is_none() {
                plan = path_plan;
            } else {
                // 使用连接器连接多个路径
                let connector = crate::query::planner::match_planning::utils::connector::SegmentsConnector::new();
                plan = connector.cartesian_product(plan, path_plan);
            }
        }

        // 处理 WHERE 子句（如果存在）
        // 暂时跳过WHERE子句处理，因为接口不兼容
        // TODO: 实现WHERE子句处理逻辑
        // if let Some(where_clause) = &match_clause_ctx.where_clause {
        //     let mut where_planner = WhereClausePlanner::new(false);
        //     let where_clause_ctx = CypherClauseContext::Where(where_clause.clone());
        //     let where_plan = where_planner.transform(&where_clause_ctx)?;
        //     let connector = crate::query::planner::match_planning::utils::connector::SegmentsConnector::new();
        //     plan = connector.add_input(where_plan, plan, true);
        // }

        // 处理分页（如果存在）
        if let Some(skip) = &match_clause_ctx.skip {
            // 检查 skip 是否大于 0，需要将 Expression 转换为数值
            let skip_value = match skip {
                Expression::Literal(crate::graph::expression::expression::LiteralValue::Int(v)) => *v,
                _ => 0, // 默认值为 0
            };
            
            if skip_value > 0 {
                // Skip 节点不存在，暂时跳过
                // TODO: 实现 Skip 节点或使用其他方式处理跳过逻辑
            }
        }
        
        if let Some(limit) = &match_clause_ctx.limit {
            // 检查 limit 是否不是最大值，需要将 Expression 转换为数值
            let limit_value = match limit {
                Expression::Literal(crate::graph::expression::expression::LiteralValue::Int(v)) => *v,
                _ => i64::MAX, // 默认值为最大值
            };
            
            if limit_value != i64::MAX {
                // 创建限制节点
                let limit_node = crate::query::planner::plan::SingleInputNode::new(
                    PlanNodeKind::Limit,
                    plan.root.ok_or_else(|| {
                        PlannerError::PlanGenerationFailed(
                            "Cannot create limit node without root".to_string()
                        )
                    })?,
                );
                
                let limit_node_arc = std::sync::Arc::new(limit_node);
                plan = SubPlan::new(
                    Some(limit_node_arc.clone()),
                    Some(limit_node_arc),
                );
            }
        }

        Ok(plan)
    }
    
    fn validate_input(&self, input_plan: Option<&SubPlan>) -> Result<(), crate::query::planner::planner::PlannerError> {
        // MATCH 子句可以开始数据流，所以不应该有输入
        if input_plan.is_some() {
            return Err(PlannerError::PlanGenerationFailed(
                "MATCH clause should not have input".to_string()
            ));
        }
        Ok(())
    }
    
    fn clause_type(&self) -> ClauseType {
        ClauseType::Source
    }
    
    fn can_start_flow(&self) -> bool {
        true   // MATCH 可以开始数据流
    }
    
    fn requires_input(&self) -> bool {
        false  // MATCH 不需要输入
    }
    
    fn input_requirements(&self) -> Vec<crate::query::planner::match_planning::core::VariableRequirement> {
        // MATCH 子句不需要输入变量
        vec![]
    }
    
    fn output_provides(&self) -> Vec<crate::query::planner::match_planning::core::VariableProvider> {
        // MATCH 子句提供路径中定义的变量
        let mut providers = Vec::new();
        
        for path in &self.paths {
            for node_info in &path.node_infos {
                if !node_info.alias.is_empty() {
                    providers.push(crate::query::planner::match_planning::core::VariableProvider {
                        name: node_info.alias.clone(),
                        var_type: crate::query::planner::match_planning::core::VariableType::Vertex,
                        nullable: false,
                    });
                }
            }
            
            for edge_info in &path.edge_infos {
                if !edge_info.alias.is_empty() {
                    providers.push(crate::query::planner::match_planning::core::VariableProvider {
                        name: edge_info.alias.clone(),
                        var_type: crate::query::planner::match_planning::core::VariableType::Edge,
                        nullable: false,
                    });
                }
            }
        }
        
        providers
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::match_planning::core::ClauseType;
    use crate::query::validator::structs::{NodeInfo, Path, PathType};
    
    #[test]
    fn test_match_clause_planner_v2_interface() {
        let planner = MatchClausePlannerV2::new(vec![]);
        assert_eq!(planner.clause_type(), ClauseType::Source);
        assert!(planner.can_start_flow());
        assert!(!planner.requires_input());
    }
    
    #[test]
    fn test_match_clause_planner_v2_validate_input() {
        let planner = MatchClausePlannerV2::new(vec![]);
        
        // 测试有输入的情况（应该失败）
        let dummy_plan = SubPlan::new(None, None);
        let result = planner.validate_input(Some(&dummy_plan));
        assert!(result.is_err());
        
        // 测试没有输入的情况（应该成功）
        let result = planner.validate_input(None);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_match_clause_planner_v2_output_provides() {
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
            path_type: PathType::Default,
            node_infos: vec![node_info],
            edge_infos: vec![],
            path_build: None,
            is_pred: false,
            is_anti_pred: false,
            compare_variables: vec![],
            collect_variable: String::new(),
            roll_up_apply: false,
        };
        
        let planner = MatchClausePlannerV2::new(vec![path.clone()]);
        let providers = planner.output_provides();
        
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].name, "n");
        assert_eq!(providers[0].var_type, crate::query::planner::match_planning::core::VariableType::Vertex);
        assert!(!providers[0].nullable);
    }
}