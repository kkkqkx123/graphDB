//! FETCH VERTICES查询规划器
//! 处理FETCH VERTICES查询的规划

use crate::query::context::{AstContext, FetchVerticesContext};
use crate::query::planner::plan::core::common::TagProp;
use crate::query::planner::plan::core::plan_node::PlanNode;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::{Argument, Dedup, GetVertices, Project};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::validator::Variable;

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
        let mut arg_node = Box::new(Argument::new(1, &fetch_ctx.from.user_defined_var_name));
        arg_node.set_col_names(vec!["vid".to_string()]);
        arg_node.set_output_var(Variable {
            name: "vertex_ids".to_string(),
            columns: vec![],
        });

        // 2. 创建获取顶点的节点
        let mut get_vertices_node = Box::new(GetVertices::new(
            2,
            1,
            &fetch_ctx.from.user_defined_var_name,
        ));
        get_vertices_node.set_dependencies(vec![arg_node.clone_plan_node()]);
        get_vertices_node.set_output_var(Variable {
            name: "fetched_vertices".to_string(),
            columns: vec![],
        });

        // 设置顶点属性
        get_vertices_node.tag_props = fetch_ctx
            .expr_props
            .tag_props
            .iter()
            .map(|(tag, props)| TagProp::new(tag, props.clone()))
            .collect();

        // 3. 创建投影节点
        let mut project_node = Box::new(Project::new(
            3,
            &fetch_ctx.yield_expr.clone().unwrap_or("*".to_string()),
        ));
        project_node.set_dependencies(vec![get_vertices_node.clone_plan_node()]);
        let result_columns: Vec<crate::query::validator::Column> = fetch_ctx
            .from
            .vids
            .iter()
            .map(|vid| crate::query::validator::Column {
                name: vid.clone(),
                type_: crate::core::ValueTypeDef::String, // 使用正确的类型
            })
            .collect();
        project_node.set_output_var(Variable {
            name: "project_result".to_string(),
            columns: result_columns,
        });
        project_node.set_col_names(fetch_ctx.from.vids.clone());

        // 4. 如果需要去重，创建去重节点
        let final_node: Box<dyn PlanNode> = if fetch_ctx.distinct {
            let mut dedup_node = Box::new(Dedup::new(4));
            dedup_node.set_dependencies(vec![project_node.clone_plan_node()]);
            dedup_node.set_output_var(Variable {
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

impl Default for FetchVerticesPlanner {
    fn default() -> Self {
        Self::new()
    }
}
