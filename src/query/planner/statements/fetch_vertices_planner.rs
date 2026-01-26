//! FETCH VERTICES查询规划器
//! 处理FETCH VERTICES查询的规划

use crate::query::context::ast::{AstContext, FetchVerticesContext};
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::common::TagProp;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::{
    ArgumentNode, DedupNode, GetVerticesNode, PlanNodeEnum, ProjectNode,
};

use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

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
        let mut arg_node = ArgumentNode::new(1, &fetch_ctx.from.user_defined_var_name);
        arg_node.set_col_names(vec!["vid".to_string()]);
        arg_node.set_output_var(Variable {
            name: "vertex_ids".to_string(),
            columns: vec![],
        });

        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        let mut get_vertices_node = GetVerticesNode::new(2, &fetch_ctx.from.user_defined_var_name);
        get_vertices_node.add_dependency(arg_node_enum.clone());
        get_vertices_node.set_output_var(Variable {
            name: "fetched_vertices".to_string(),
            columns: vec![],
        });

        let tag_props = fetch_ctx
            .expr_props
            .tag_props
            .iter()
            .map(|(tag, props)| TagProp::new(tag, props.clone()))
            .collect();
        get_vertices_node.set_tag_props(tag_props);

        let get_vertices_node_enum = PlanNodeEnum::GetVertices(get_vertices_node);

        // 3. 创建投影节点
        let project_node = ProjectNode::new(get_vertices_node_enum.clone(), vec![])?;

        let project_node_enum = PlanNodeEnum::Project(project_node);

        // 4. 如果需要去重，创建去重节点
        let final_node: PlanNodeEnum = if fetch_ctx.distinct {
            let mut dedup_node = DedupNode::new(project_node_enum.clone())?;
            dedup_node.set_output_var(Variable {
                name: "dedup_result".to_string(),
                columns: vec![],
            });
            PlanNodeEnum::Dedup(dedup_node)
        } else {
            project_node_enum
        };

        // 创建SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(arg_node_enum));

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
