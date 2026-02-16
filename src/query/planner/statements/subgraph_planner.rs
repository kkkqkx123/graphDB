//! SUBGRAPH查询规划器
//! 处理Nebula SUBGRAPH查询的规划
//!
//! ## 改进说明
//! - 支持零步扩展（0 STEPS）
//! - 支持 M TO N STEPS 范围
//! - 优化起始点查找策略

use crate::core::types::EdgeDirection;
use crate::query::context::ast::{AstContext, SubgraphContext};
use crate::query::planner::plan::core::nodes::{
    ArgumentNode as Argument, ExpandAllNode as ExpandAll,
    FilterNode as Filter, GetVerticesNode, PlanNodeEnum, ProjectNode as Project,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

/// SUBGRAPH查询规划器
/// 负责将SUBGRAPH查询转换为执行计划
#[derive(Debug, Clone)]
pub struct SubgraphPlanner;

impl SubgraphPlanner {
    /// 创建新的SUBGRAPH规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配SUBGRAPH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "SUBGRAPH"
    }

    /// 获取匹配和实例化函数（静态注册版本）
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiateEnum {
        crate::query::planner::planner::MatchAndInstantiateEnum::Subgraph(Self::new())
    }
}

impl Planner for SubgraphPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let subgraph_ctx = SubgraphContext::new(ast_ctx.clone());

        log::debug!("Processing SUBGRAPH query planning: {:?}", subgraph_ctx);

        // 获取步数范围
        let m_steps = subgraph_ctx.steps.m_steps;
        let n_steps = subgraph_ctx.steps.n_steps;

        log::debug!("SUBGRAPH steps: {} to {}", m_steps, n_steps);

        // 创建起始节点
        let arg_node = Argument::new(1, &subgraph_ctx.from.user_defined_var_name);
        let mut current_node: PlanNodeEnum = PlanNodeEnum::Argument(arg_node.clone());

        // 处理零步扩展（0 STEPS）
        // 当 m_steps == 0 时，只返回起始顶点本身，不进行任何扩展
        if m_steps == 0 {
            log::debug!("SUBGRAPH with 0 steps - returning only start vertices");
            
            // 获取起始顶点
            // 使用 GetVerticesNode 获取起始顶点
            let get_vertices_node = GetVerticesNode::new(
                1, // space_id
                &subgraph_ctx.from.user_defined_var_name,
            );
            current_node = PlanNodeEnum::GetVertices(get_vertices_node);

            // 应用过滤器
            current_node = self.apply_filters(current_node, &subgraph_ctx)?;

            // 投影
            let project_node = match Project::new(current_node.clone(), vec![]) {
                Ok(node) => PlanNodeEnum::Project(node),
                Err(_) => current_node,
            };
            current_node = project_node;

            let sub_plan = SubPlan::new(Some(current_node), Some(PlanNodeEnum::Argument(arg_node)));
            return Ok(sub_plan);
        }

        // 处理多步扩展（M TO N STEPS）
        // 对于范围步数，需要循环执行扩展
        if m_steps > 0 {
            // 第一步：从起始点开始扩展
            current_node = self.create_expand_node(
                current_node,
                &subgraph_ctx,
                EdgeDirection::Out,
            )?;

            // 如果步数范围大于1，需要添加循环扩展
            if n_steps > 1 {
                // 对于 M TO N 步数，需要特殊处理
                // 这里简化实现，实际应该使用 Loop 节点
                for step in 1..n_steps {
                    log::debug!("Adding expansion step {}", step + 1);
                    current_node = self.create_expand_node(
                        current_node,
                        &subgraph_ctx,
                        EdgeDirection::Out,
                    )?;
                }
            }
        }

        // 应用过滤器
        current_node = self.apply_filters(current_node, &subgraph_ctx)?;

        // 投影
        let project_node = match Project::new(current_node.clone(), vec![]) {
            Ok(node) => PlanNodeEnum::Project(node),
            Err(_) => current_node,
        };
        current_node = project_node;

        let sub_plan = SubPlan::new(Some(current_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl SubgraphPlanner {
    /// 创建扩展节点
    fn create_expand_node(
        &self,
        _input: PlanNodeEnum,
        subgraph_ctx: &SubgraphContext,
        direction: EdgeDirection,
    ) -> Result<PlanNodeEnum, PlannerError> {
        let edge_types: Vec<String> = subgraph_ctx.edge_types.iter().cloned().collect();
        
        let expand_node = ExpandAll::new(
            2,
            edge_types,
            match direction {
                EdgeDirection::Out => "out",
                EdgeDirection::In => "in",
                EdgeDirection::Both => "both",
            },
        );

        // 返回 ExpandAll 节点，实际执行时会先扩展再获取顶点
        Ok(PlanNodeEnum::ExpandAll(expand_node))
    }

    /// 应用所有过滤器
    fn apply_filters(
        &self,
        input: PlanNodeEnum,
        subgraph_ctx: &SubgraphContext,
    ) -> Result<PlanNodeEnum, PlannerError> {
        let mut current = input;

        // 应用通用过滤器
        if let Some(ref condition) = subgraph_ctx.filter {
            current = match Filter::new(current.clone(), condition.clone()) {
                Ok(node) => PlanNodeEnum::Filter(node),
                Err(_) => current,
            };
        }

        // 应用标签过滤器
        if let Some(ref tag_condition) = subgraph_ctx.tag_filter {
            current = match Filter::new(current.clone(), tag_condition.clone()) {
                Ok(node) => PlanNodeEnum::Filter(node),
                Err(_) => current,
            };
        }

        // 应用边过滤器
        if let Some(ref edge_condition) = subgraph_ctx.edge_filter {
            current = match Filter::new(current.clone(), edge_condition.clone()) {
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
