//! LOOKUP语句规划器
//! 处理Nebula LOOKUP查询的规划

use crate::query::context::ast::{AstContext, LookupContext};
use crate::query::planner::plan::core::{
    DedupNode, FilterNode, GetEdgesNode, GetVerticesNode, ProjectNode,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use std::sync::Arc;

/// LOOKUP查询规划器
/// 负责将LOOKUP语句转换为执行计划
#[derive(Debug)]
pub struct LookupPlanner;

impl LookupPlanner {
    /// 创建新的LOOKUP规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配LOOKUP查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "LOOKUP"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
            priority: 100,
        }
    }
}

impl Planner for LookupPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let lookup_ctx = LookupContext::new(ast_ctx.clone());
        let space_id = lookup_ctx.space_id.unwrap_or(1); // 从上下文获取空间ID，默认为1
        let schema_id = lookup_ctx.schema_id;
        let index_name = lookup_ctx.index_name.as_deref().unwrap_or(""); // 从上下文获取索引名称
        let return_cols = lookup_ctx.idx_return_cols.clone();

        let mut sub_plan = SubPlan {
            root: None,
            tail: None,
        };

        // 基于NebulaGraph的实现，处理两种情况：全文索引和普通索引
        let index_scan_node = if lookup_ctx.is_edge {
            let get_edges = GetEdgesNode::new(space_id, index_name, "", "", "");
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::GetEdges(
                get_edges,
            )
        } else {
            let get_vertices = GetVerticesNode::new(space_id, index_name);
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::GetVertices(
                get_vertices,
            )
        };

        sub_plan.tail = Some(index_scan_node.clone());
        sub_plan.root = Some(index_scan_node);

        if lookup_ctx.is_fulltext_index {
            // 全文索引处理
            // 如果需要评分，添加额外的处理
            if lookup_ctx.has_score {
                // 添加评分相关的处理逻辑
            }
        } else {
            // 普通索引处理
            // 基于NebulaGraph的EdgeIndexFullScan和TagIndexFullScan设计
            // 添加过滤条件
            if let Some(ref condition) = lookup_ctx.filter {
                use crate::core::Expression;
                let expr = Expression::Variable(condition.clone());
                let filter_node = FilterNode::new(index_scan_node, expr)
                    .expect("FilterNode creation should succeed with valid input");
                sub_plan.root = Some(
                    crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Filter(
                        filter_node,
                    ),
                );
            }
        }

        // 创建投影节点（基于NebulaGraph的实现，总是创建投影节点）
        use crate::query::validator::YieldColumn;

        let yield_columns = if let Some(ref yield_expr) = lookup_ctx.yield_expr {
            yield_expr
                .columns
                .iter()
                .map(|col| YieldColumn {
                    expr: crate::core::Expression::Variable(col.name().to_string()),
                    alias: col.alias.clone(),
                    is_matched: false,
                })
                .collect()
        } else {
            vec![YieldColumn {
                expr: crate::core::Expression::Variable("*".to_string()),
                alias: "result".to_string(),
                is_matched: false,
            }]
        };

        let project_node = ProjectNode::new(sub_plan.root.clone().unwrap(), yield_columns)
            .expect("ProjectNode creation should succeed with valid input");
        sub_plan.root = Some(
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Project(
                project_node,
            ),
        );

        // 如果需要去重，创建去重节点
        if lookup_ctx.dedup {
            let dedup_node = DedupNode::new(sub_plan.root.clone().unwrap())
                .expect("DedupNode creation should succeed with valid input");
            sub_plan.root = Some(
                crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Dedup(
                    dedup_node,
                ),
            );
        }

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for LookupPlanner {
    fn default() -> Self {
        Self::new()
    }
}
