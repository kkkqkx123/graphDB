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
        // 从ast_ctx创建LookupContext
        let lookup_ctx = LookupContext::new(ast_ctx.clone());

        // 实现LOOKUP查询的规划逻辑
        println!("Processing LOOKUP query planning: {:?}", lookup_ctx);

        // 1. 创建索引扫描节点
        let mut index_scan_node: Arc<dyn crate::query::planner::plan::core::PlanNode> =
            if lookup_ctx.is_edge {
                // 如果是边的查找，创建GetEdges节点
                let get_edges_node = Arc::new(GetEdgesNode::new(1, "", "", "", ""));
                get_edges_node
            } else {
                // 如果是顶点的查找，创建GetVertices节点
                let get_vertices_node = Arc::new(GetVerticesNode::new(1, ""));
                get_vertices_node
            };

        // 2. 创建过滤节点（基于索引搜索条件）
        if let Some(ref condition) = lookup_ctx.filter {
            // 这里需要将condition转换为Expression类型
            // 暂时使用空表达式作为占位符
            use crate::expression::Expression;
            let expr = Expression::Variable(condition.clone());

            let filter_node = Arc::new(
                FilterNode::new(index_scan_node.clone(), expr)
                    .expect("FilterNode creation should succeed with valid input"),
            );
            index_scan_node = filter_node;

            // 如果是全文索引
            if lookup_ctx.is_fulltext_index {
                // 添加全文搜索相关逻辑
                if lookup_ctx.has_score {
                    // 包含评分结果
                }
            }
        }

        // 3. 创建投影节点
        use crate::query::validator::YieldColumn;
        let yield_columns = vec![YieldColumn {
            expr: crate::expression::Expression::Variable(
                lookup_ctx.yield_expr.clone().unwrap_or("*".to_string()),
            ),
            alias: "result".to_string(),
            is_matched: false,
        }];

        let project_node = Arc::new(
            ProjectNode::new(index_scan_node.clone(), yield_columns)
                .expect("ProjectNode creation should succeed with valid input"),
        );

        // 4. 如果需要去重，创建去重节点
        let final_node: Arc<dyn crate::query::planner::plan::core::PlanNode> = if lookup_ctx.dedup {
            let dedup_node = Arc::new(
                DedupNode::new(project_node)
                    .expect("DedupNode creation should succeed with valid input"),
            );
            dedup_node
        } else {
            project_node
        };

        // 创建SubPlan
        let sub_plan = SubPlan {
            root: Some(final_node),
            tail: Some(index_scan_node),
        };

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
