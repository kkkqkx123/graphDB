//! LOOKUP语句规划器
//! 处理Nebula LOOKUP查询的规划

use crate::query::context::{AstContext, LookupContext};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::planner::plan::plan_node::PlanNode;
use crate::query::planner::plan::SubPlan;
use crate::query::validator::Variable;
use crate::query::planner::plan::graph_scan::{GetEdges, GetVertices};
use crate::query::planner::plan::operations::data_processing::{Filter, Project, Dedup};

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
        let mut index_scan_node = if lookup_ctx.is_edge {
            // 如果是边的查找，创建GetEdges节点
            let mut get_edges_node = Box::new(GetEdges::new(1, 1, "", "", "", ""));
            get_edges_node.set_output_var(Variable {
                name: "index_scanned_edges".to_string(),
                columns: vec![],
            });
            get_edges_node as Box<dyn crate::query::planner::plan::plan_node::PlanNode>
        } else {
            // 如果是顶点的查找，创建GetVertices节点
            let mut get_vertices_node = Box::new(GetVertices::new(1, 1, ""));
            get_vertices_node.set_output_var(Variable {
                name: "index_scanned_vertices".to_string(),
                columns: vec![],
            });
            get_vertices_node as Box<dyn crate::query::planner::plan::plan_node::PlanNode>
        };

        // 2. 创建过滤节点（基于索引搜索条件）
        if let Some(ref condition) = lookup_ctx.filter {
            let mut filter_node = Box::new(Filter::new(2, condition));
            filter_node.set_dependencies(vec![index_scan_node.clone_plan_node()]);
            filter_node.set_output_var(Variable {
                name: "filtered_result".to_string(),
                columns: vec![],
            });

            // 如果是全文索引
            if lookup_ctx.is_fulltext_index {
                // 添加全文搜索相关逻辑
                if lookup_ctx.has_score {
                    // 包含评分结果
                }
            }

            index_scan_node = filter_node as Box<dyn crate::query::planner::plan::plan_node::PlanNode>;
        }

        // 3. 创建投影节点
        let mut project_node = Box::new(Project::new(3, &lookup_ctx.yield_expr.clone().unwrap_or("*".to_string())));
        project_node.set_dependencies(vec![index_scan_node.clone_plan_node()]);
        let result_columns: Vec<crate::query::validator::Column> = vec![
            crate::query::validator::Column {
                name: "result".to_string(),
                type_: crate::core::ValueTypeDef::String,
            }
        ];
        project_node.set_output_var(Variable {
            name: "project_result".to_string(),
            columns: result_columns,
        });

        // 4. 如果需要去重，创建去重节点
        let final_node = if lookup_ctx.dedup {
            let mut dedup_node = Box::new(Dedup::new(4));
            dedup_node.set_dependencies(vec![project_node.clone_plan_node()]);
            dedup_node.set_output_var(Variable {
                name: "dedup_result".to_string(),
                columns: vec![],
            });
            dedup_node as Box<dyn crate::query::planner::plan::plan_node::PlanNode>
        } else {
            project_node as Box<dyn crate::query::planner::plan::plan_node::PlanNode>
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
