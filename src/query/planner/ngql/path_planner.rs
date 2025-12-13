//! PATH查询规划器
//! 处理Nebula PATH查询的规划

use crate::query::context::validate::types::Variable;
use crate::query::context::{AstContext, PathContext};
use crate::query::planner::plan::core::common::{TagProp, EdgeProp};
use crate::query::planner::plan::{Expand, ExpandAll, Filter, Project, Dedup, Argument};
use crate::query::planner::plan::PlanNode;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

/// PATH查询规划器
/// 负责将PATH查询转换为执行计划
#[derive(Debug)]
pub struct PathPlanner;

impl PathPlanner {
    /// 创建新的PATH规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配PATH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "PATH"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }
}

impl Planner for PathPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 从ast_ctx创建PathContext
        let path_ctx = PathContext::new(ast_ctx.clone());

        // 实现PATH查询的规划逻辑
        println!("Processing PATH query planning: {:?}", path_ctx);

        // 1. 创建参数节点，获取起始和结束顶点
        let mut start_arg_node = Box::new(Argument::new(1, &path_ctx.from.user_defined_var_name));
        start_arg_node.set_col_names(vec!["start_vid".to_string()]);
        start_arg_node.set_output_var(Variable {
            name: path_ctx.from_vids_var.clone(),
            columns: vec![],
        });

        let mut end_arg_node = Box::new(Argument::new(2, &path_ctx.to.user_defined_var_name));
        end_arg_node.set_col_names(vec!["end_vid".to_string()]);
        end_arg_node.set_output_var(Variable {
            name: path_ctx.to_vids_var.clone(),
            columns: vec![],
        });

        // 2. 创建GetVertices节点来获取顶点
        let mut get_vertices_node =
            Box::new(crate::query::planner::plan::GetVertices::new(3, 1, &path_ctx.from.user_defined_var_name));
        get_vertices_node.set_dependencies(vec![start_arg_node.clone_plan_node()]);
        get_vertices_node.set_output_var(Variable {
            name: "path_vertices".to_string(),
            columns: vec![],
        });

        // 设置顶点属性
        get_vertices_node.tag_props = path_ctx
            .expr_props
            .src_tag_props
            .iter()
            .map(|(tag, props)| TagProp::new(tag, props.clone()))
            .collect();

        // 3. 创建扩展节点进行路径搜索
        let expand_direction = if path_ctx.over.direction == "both" {
            "both"
        } else if path_ctx.over.direction == "in" {
            "in"
        } else {
            "out"
        };
        let mut expand_node = Box::new(Expand::new(4, 1, path_ctx.over.edge_types.clone(), expand_direction));
        expand_node.set_dependencies(vec![get_vertices_node.clone_plan_node()]);
        expand_node.set_output_var(Variable {
            name: "expanded_path".to_string(),
            columns: vec![],
        });

        // 4. 如果是双向边，设置方向
        if path_ctx.over.direction == "both" {
            expand_node
                .edge_types
                .extend(path_ctx.over.edge_types.iter().map(|et| format!("-{}", et)));
        } else if path_ctx.over.direction == "in" {
            expand_node.edge_types = path_ctx
                .over
                .edge_types
                .iter()
                .map(|et| format!("-{}", et))
                .collect();
        }

        // 5. 创建ExpandAll节点进行多步扩展
        let expand_all_direction = if path_ctx.over.direction == "both" {
            "both"
        } else if path_ctx.over.direction == "in" {
            "in"
        } else {
            "out"
        };
        let mut expand_all_node = Box::new(ExpandAll::new(
            5,
            1,
            path_ctx.over.edge_types.clone(),
            expand_all_direction,
        ));
        expand_all_node.set_dependencies(vec![expand_node.clone_plan_node()]);
        expand_all_node.set_output_var(Variable {
            name: "expanded_all_path".to_string(),
            columns: vec![],
        });

        // 设置边属性和顶点属性
        expand_all_node.edge_props = path_ctx
            .expr_props
            .edge_props
            .iter()
            .map(|(edge_type, props)| EdgeProp::new(edge_type, props.clone()))
            .collect();

        expand_all_node.vertex_props = path_ctx
            .expr_props
            .src_tag_props
            .iter()
            .map(|(tag, props)| TagProp::new(tag, props.clone()))
            .collect();

        // 6. 创建过滤节点（如果有过滤条件）
        let mut filter_node: Box<dyn crate::query::planner::plan::core::PlanNode> = if let Some(ref condition) = path_ctx.filter {
            let mut filter = Box::new(Filter::new(6, condition));
            filter.set_dependencies(vec![expand_all_node.clone_plan_node()]);
            filter.set_output_var(Variable {
                name: "filtered_path".to_string(),
                columns: vec![],
            });
            filter
        } else {
            expand_all_node
        };

        // 7. 创建投影节点
        let mut project_node = Box::new(Project::new(7, &"DEFAULT".to_string()));
        project_node.set_dependencies(vec![filter_node.clone_plan_node()]);
        project_node.set_output_var(Variable {
            name: "projected_path".to_string(),
            columns: vec![],
        });
        project_node.set_col_names(path_ctx.col_names.clone());

        // 8. 如果是查找最短路径，可能需要额外的处理
        let final_node: Box<dyn crate::query::planner::plan::core::PlanNode> = if path_ctx.is_shortest {
            // 需要额外的节点来处理最短路径算法
            let mut dedup_node = Box::new(Dedup::new(8));
            dedup_node.set_dependencies(vec![project_node.clone_plan_node()]);
            dedup_node.set_output_var(Variable {
                name: "shortest_path_result".to_string(),
                columns: vec![],
            });
            dedup_node
        } else {
            project_node
        };

        // 创建SubPlan
        let sub_plan = SubPlan {
            root: Some(final_node),
            tail: Some(start_arg_node.clone_plan_node()),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for PathPlanner {
    fn default() -> Self {
        Self::new()
    }
}
