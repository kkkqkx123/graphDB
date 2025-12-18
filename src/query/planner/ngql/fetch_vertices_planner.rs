//! FETCH VERTICES查询规划器
//! 处理FETCH VERTICES查询的规划

use crate::query::context::ast::{AstContext, FetchVerticesContext};
use crate::query::context::validate::types::{Column, Variable};
use crate::query::planner::plan::core::common::TagProp;
use crate::query::planner::plan::core::plan_node_traits::{PlanNodeDependencies, PlanNodeMutable};
use crate::query::planner::plan::core::nodes::{ArgumentNode, DedupNode, GetVerticesNode, ProjectNode};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use std::sync::Arc;

/// FETCH VERTICES查询规划器
/// 负责将FETCH VERTICES查询转换为执行计划
#[derive(Debug)]
pub struct FetchVerticesPlanner;

impl FetchVerticesPlanner {
    /// 创建新的FETCH VERTICES规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配FETCH VERTICES查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "FETCH VERTICES"
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

impl Planner for FetchVerticesPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 从ast_ctx创建FetchVerticesContext
        let fetch_ctx = FetchVerticesContext::new(ast_ctx.clone());

        // 实现FETCH VERTICES查询的规划逻辑
        println!("Processing FETCH VERTICES query planning: {:?}", fetch_ctx);

        // 1. 创建参数节点，获取顶点ID
        let mut arg_node = Arc::new(ArgumentNode::new(1, &fetch_ctx.from.user_defined_var_name));
        Arc::get_mut(&mut arg_node)
            .unwrap()
            .set_col_names(vec!["vid".to_string()]);
        Arc::get_mut(&mut arg_node)
            .unwrap()
            .set_output_var(Variable {
                name: "vertex_ids".to_string(),
                columns: vec![],
            });

        // 2. 创建获取顶点的节点
        let mut get_vertices_node = Arc::new(GetVerticesNode::new(1, &fetch_ctx.from.user_defined_var_name));
        Arc::get_mut(&mut get_vertices_node)
            .unwrap()
            .add_dependency(arg_node.clone());
        Arc::get_mut(&mut get_vertices_node)
            .unwrap()
            .set_output_var(Variable {
                name: "fetched_vertices".to_string(),
                columns: vec![],
            });

        // 设置顶点属性
        if let Some(node) = Arc::get_mut(&mut get_vertices_node) {
            let tag_props = fetch_ctx
                .expr_props
                .tag_props
                .iter()
                .map(|(tag, props)| TagProp::new(tag, props.clone()))
                .collect();
            node.set_tag_props(tag_props);
        }

        // 3. 创建投影节点
        let mut project_node = Arc::new(ProjectNode::new(
            get_vertices_node.clone(),
            vec![], // 这里需要提供YieldColumn列表
        )?);
        Arc::get_mut(&mut project_node)
            .unwrap()
            .add_dependency(get_vertices_node.clone());
        let result_columns: Vec<Column> = fetch_ctx
            .from
            .vids
            .iter()
            .map(|vid| Column {
                name: vid.clone(),
                type_: "STRING".to_string(),
            })
            .collect();
        Arc::get_mut(&mut project_node)
            .unwrap()
            .set_output_var(Variable {
                name: "project_result".to_string(),
                columns: result_columns,
            });
        Arc::get_mut(&mut project_node)
            .unwrap()
            .set_col_names(fetch_ctx.from.vids.clone());

        // 4. 如果需要去重，创建去重节点
        let final_node: Arc<dyn crate::query::planner::plan::core::PlanNode> = if fetch_ctx.distinct
        {
            let mut dedup_node = Arc::new(DedupNode::new(project_node.clone())?);
            Arc::get_mut(&mut dedup_node)
                .unwrap()
                .add_dependency(project_node.clone());
            Arc::get_mut(&mut dedup_node)
                .unwrap()
                .set_output_var(Variable {
                    name: "dedup_result".to_string(),
                    columns: vec![],
                });
            dedup_node
        } else {
            project_node
        };

        // 创建SubPlan
        let sub_plan = SubPlan {
            root: Some(final_node),
            tail: Some(arg_node),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for FetchVerticesPlanner {
    fn default() -> Self {
        Self::new()
    }
}
