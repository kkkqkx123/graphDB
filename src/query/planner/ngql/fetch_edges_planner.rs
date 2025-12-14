//! FETCH EDGES查询规划器
//! 处理FETCH EDGES查询的规划

use crate::query::context::{AstContext, FetchEdgesContext};
use crate::query::planner::plan::core::common::EdgeProp;
use crate::query::planner::plan::core::plan_node_traits::{PlanNodeClonable, PlanNodeMutable, PlanNodeDependencies};
use crate::query::planner::plan::execution_plan::SubPlan;
use crate::query::planner::plan::operations::{Argument, Dedup, Filter, GetEdges, Project};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::context::validate::types::Variable;
use std::sync::Arc;

/// FETCH EDGES查询规划器
/// 负责将FETCH EDGES查询转换为执行计划
#[derive(Debug)]
pub struct FetchEdgesPlanner;

impl FetchEdgesPlanner {
    /// 创建新的FETCH EDGES规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配FETCH EDGES查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "FETCH EDGES"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }
}

impl Planner for FetchEdgesPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 从ast_ctx创建FetchEdgesContext
        let fetch_ctx = FetchEdgesContext::new(ast_ctx.clone());

        // 实现FETCH EDGES查询的规划逻辑
        println!("Processing FETCH EDGES query planning: {:?}", fetch_ctx);

        // 1. 创建参数节点，获取边的条件
        let mut arg_node = Arc::new(Argument::new(1, &fetch_ctx.input_var_name));
        std::sync::Arc::get_mut(&mut arg_node).unwrap().set_col_names(vec!["edge_condition".to_string()]);
        std::sync::Arc::get_mut(&mut arg_node).unwrap().set_output_var(Variable {
            name: "edge_condition".to_string(),
            columns: vec![],
        });

        // 2. 创建获取边的节点
        let mut get_edges_node = Arc::new(GetEdges::new(
            2,
            1,
            &fetch_ctx.src.clone().unwrap_or_default(),
            &fetch_ctx.edge_type.clone().unwrap_or_default(),
            &fetch_ctx.rank.clone().unwrap_or_default(),
            &fetch_ctx.dst.clone().unwrap_or_default(),
        ));
        std::sync::Arc::get_mut(&mut get_edges_node).unwrap().add_dependency(arg_node.clone_plan_node());
        std::sync::Arc::get_mut(&mut get_edges_node).unwrap().set_output_var(Variable {
            name: "fetched_edges".to_string(),
            columns: vec![],
        });

        // 设置边属性
        if let Some(mut_node) = std::sync::Arc::get_mut(&mut get_edges_node) {
            mut_node.edge_props = fetch_ctx
                .expr_props
                .edge_props
                .iter()
                .map(|(edge_type, props)| EdgeProp::new(edge_type, props.clone()))
                .collect();
        }

        // 3. 创建过滤空边的节点
        let mut filter_node = Arc::new(Filter::new(
            3,
            &format!("{} IS NOT EMPTY", fetch_ctx.edge_name),
        ));
        std::sync::Arc::get_mut(&mut filter_node).unwrap().add_dependency(get_edges_node.clone_plan_node());
        std::sync::Arc::get_mut(&mut filter_node).unwrap().set_output_var(Variable {
            name: "filtered_edges".to_string(),
            columns: vec![],
        });

        // 4. 创建投影节点
        let mut project_node = Arc::new(Project::new(
            4,
            &fetch_ctx.yield_expr.clone().unwrap_or("*".to_string()),
        ));
        std::sync::Arc::get_mut(&mut project_node).unwrap().add_dependency(filter_node.clone_plan_node());
        let result_columns: Vec<crate::query::context::validate::types::Column> = vec![
            crate::query::context::validate::types::Column {
                name: "src".to_string(),
                type_: "STRING".to_string(),
            },
            crate::query::context::validate::types::Column {
                name: "dst".to_string(),
                type_: "STRING".to_string(),
            },
            crate::query::context::validate::types::Column {
                name: "rank".to_string(),
                type_: "INT".to_string(),
            },
        ];
        std::sync::Arc::get_mut(&mut project_node).unwrap().set_output_var(Variable {
            name: "project_result".to_string(),
            columns: result_columns,
        });

        // 5. 如果需要去重，创建去重节点
        let final_node: Arc<dyn crate::query::planner::plan::core::PlanNode> = if fetch_ctx.distinct
        {
            let mut dedup_node = Arc::new(Dedup::new(5));
            std::sync::Arc::get_mut(&mut dedup_node).unwrap().add_dependency(project_node.clone_plan_node());
            std::sync::Arc::get_mut(&mut dedup_node).unwrap().set_output_var(Variable {
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
            tail: Some(arg_node.clone_plan_node()),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for FetchEdgesPlanner {
    fn default() -> Self {
        Self::new()
    }
}
