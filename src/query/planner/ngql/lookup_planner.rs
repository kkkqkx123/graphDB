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
        // 创建LOOKUP查询上下文
        let lookup_ctx = LookupContext::new(ast_ctx.clone());
        
        // 从AstContext中获取空间信息
        let space_id = ast_ctx.space().space_id;
        
        // LOOKUP查询规划逻辑
        // 根据查询类型（边/顶点）和索引类型（全文/普通）创建不同的执行计划
        let mut sub_plan = SubPlan {
            root: None,
            tail: None,
        };

        // 创建索引扫描节点 - IndexScan实现
        let index_scan_node = if lookup_ctx.is_edge {
            // 边索引扫描：使用EdgeIndexScan节点
            // 在完整实现中，这里应该从元数据中获取索引信息
            let edge_index_scan = crate::query::planner::plan::algorithms::IndexScan::new(
                -1, // 节点ID（临时值）
                space_id.unwrap_or(0) as i32,
                lookup_ctx.schema_id, // 使用schema_id作为索引ID
                lookup_ctx.schema_id, // 使用schema_id作为标签ID
                if lookup_ctx.is_fulltext_index { "FULLTEXT" } else { "RANGE" }
            );
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::IndexScan(
                edge_index_scan,
            )
        } else {
            // 顶点索引扫描：使用TagIndexScan节点
            let tag_index_scan = crate::query::planner::plan::algorithms::IndexScan::new(
                -1, // 节点ID（临时值）
                space_id.unwrap_or(0) as i32,
                lookup_ctx.schema_id, // 使用schema_id作为索引ID
                lookup_ctx.schema_id, // 使用schema_id作为标签ID
                if lookup_ctx.is_fulltext_index { "FULLTEXT" } else { "RANGE" }
            );
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::IndexScan(
                tag_index_scan,
            )
        };

        // 设置执行计划的尾节点
        sub_plan.tail = Some(index_scan_node.clone());
        
        // 处理过滤条件
        let mut current_node = index_scan_node;
        
        if let Some(ref condition) = lookup_ctx.filter {
            // 创建过滤节点
            use crate::core::Expression;
            let expr = Expression::Variable(condition.clone());
            let filter_node = FilterNode::new(current_node, expr)
                .expect("FilterNode creation should succeed with valid input");
            current_node = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Filter(
                filter_node,
            );
        }

        // 处理全文索引的特殊逻辑
        if lookup_ctx.is_fulltext_index && lookup_ctx.has_score {
            // 全文索引需要额外的评分处理
            // 这里可以添加ScoreProjection节点来处理评分
        }

        // 设置执行计划的根节点
        sub_plan.root = Some(current_node);

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
