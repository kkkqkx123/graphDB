//! SUBGRAPH查询规划器
//! 处理Nebula SUBGRAPH查询的规划
//!
//! ## 改进说明
//! - 支持零步扩展（0 STEPS）
//! - 支持 M TO N STEPS 范围
//! - 优化起始点查找策略

use std::sync::Arc;

use crate::core::types::EdgeDirection;
use crate::core::Expression;
use crate::query::QueryContext;
use crate::query::parser::ast::stmt::Steps;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::core::nodes::{
    ArgumentNode as Argument, ExpandAllNode, FilterNode, GetVerticesNode,
    PlanNodeEnum, ProjectNode as Project,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};

/// SUBGRAPH查询规划器
/// 负责将SUBGRAPH查询转换为执行计划
#[derive(Debug, Clone)]
pub struct SubgraphPlanner;

impl SubgraphPlanner {
    /// 创建新的SUBGRAPH规划器
    pub fn new() -> Self {
        Self
    }
}

impl Planner for SubgraphPlanner {
    fn transform(&mut self, validated: &ValidatedStatement, _qctx: Arc<QueryContext>) -> Result<SubPlan, PlannerError> {
        let subgraph_stmt = match &validated.stmt {
            Stmt::Subgraph(subgraph_stmt) => subgraph_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "SubgraphPlanner 需要 Subgraph 语句".to_string()
                ));
            }
        };

        log::debug!("Processing SUBGRAPH query planning");

        let steps = &subgraph_stmt.steps;
        let over = subgraph_stmt.over.as_ref();
        let where_clause = subgraph_stmt.where_clause.clone();

        let (m_steps, n_steps) = match steps {
            Steps::Fixed(n) => (*n, *n),
            Steps::Range { min, max } => (*min, *max),
            Steps::Variable(_) => {
                return Err(PlannerError::InvalidOperation(
                    "SUBGRAPH 不支持变量步数".to_string()
                ));
            }
        };

        log::debug!("SUBGRAPH steps: {} to {}", m_steps, n_steps);

        let var_name = "subgraph_args";
        let arg_node = Argument::new(1, var_name);
        let mut current_node: PlanNodeEnum = PlanNodeEnum::Argument(arg_node.clone());

        if m_steps == 0 {
            log::debug!("SUBGRAPH with 0 steps - returning only start vertices");
            
            let get_vertices_node = GetVerticesNode::new(1, var_name);
            current_node = PlanNodeEnum::GetVertices(get_vertices_node);

            let filters: Vec<Expression> = where_clause.into_iter().collect();
            current_node = self.apply_filters(current_node, &filters)?;

            let project_node = match Project::new(current_node.clone(), vec![]) {
                Ok(node) => PlanNodeEnum::Project(node),
                Err(_) => current_node,
            };
            current_node = project_node;

            let sub_plan = SubPlan::new(Some(current_node), Some(PlanNodeEnum::Argument(arg_node)));
            return Ok(sub_plan);
        }

        let edge_types = over.map(|o| o.edge_types.clone()).unwrap_or_default();
        let direction_str = over.map(|o| {
            match o.direction {
                EdgeDirection::Out => "out",
                EdgeDirection::In => "in",
                EdgeDirection::Both => "both",
            }
        }).unwrap_or("out");

        if m_steps > 0 {
            current_node = self.create_expand_node(
                current_node,
                &edge_types,
                direction_str,
                m_steps as u32,
                n_steps as u32,
            )?;

            if n_steps > m_steps {
                for step in (m_steps + 1)..=n_steps {
                    log::debug!("Adding expansion step {}", step);
                    current_node = self.create_expand_node(
                        current_node,
                        &edge_types,
                        direction_str,
                        step as u32,
                        n_steps as u32,
                    )?;
                }
            }
        }

        let filters: Vec<Expression> = where_clause.into_iter().collect();
        current_node = self.apply_filters(current_node, &filters)?;

        let project_node = match Project::new(current_node.clone(), vec![]) {
            Ok(node) => PlanNodeEnum::Project(node),
            Err(_) => current_node,
        };
        current_node = project_node;

        let sub_plan = SubPlan::new(Some(current_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Subgraph(_))
    }
}

impl SubgraphPlanner {
    /// 创建扩展节点
    fn create_expand_node(
        &self,
        _input: PlanNodeEnum,
        edge_types: &[String],
        direction: &str,
        _current_step: u32,
        max_step: u32,
    ) -> Result<PlanNodeEnum, PlannerError> {
        let mut expand_node = ExpandAllNode::new(1, edge_types.to_vec(), direction);
        expand_node.set_step_limit(max_step);
        
        Ok(PlanNodeEnum::ExpandAll(expand_node))
    }

    /// 应用所有过滤器
    fn apply_filters(
        &self,
        input: PlanNodeEnum,
        filters: &[Expression],
    ) -> Result<PlanNodeEnum, PlannerError> {
        let mut current = input;

        for condition in filters {
            use std::sync::Arc;
            let ctx = Arc::new(crate::core::types::ExpressionContext::new());
            current = match FilterNode::from_expression(current.clone(), condition.clone(), ctx) {
                Ok(node) => PlanNodeEnum::Filter(node),
                Err(_) => current,
            };
        }

        Ok(current)
    }
}

impl Default for SubgraphPlanner {
    fn default() -> Self {
        Self::new()
    }
}
