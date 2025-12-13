//! GO语句规划器
//! 处理Nebula GO查询的规划

use crate::query::context::validate::types::Variable;
use crate::query::context::{AstContext, GoContext};
use crate::query::planner::plan::core::common::{TagProp, EdgeProp};
use crate::query::planner::plan::core::plan_node_traits::{PlanNode, PlanNodeMutable, PlanNodeClonable, PlanNodeDependencies};
use crate::query::planner::plan::{Argument, Start, Expand, ExpandAll, HashLeftJoin, Filter, Project, Dedup};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use std::sync::Arc;

/// GO查询规划器
/// 负责将GO语句转换为执行计划
#[derive(Debug)]
pub struct GoPlanner;

impl GoPlanner {
    /// 创建新的GO规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配GO查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "GO"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }
}

impl Planner for GoPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 从ast_ctx创建GoContext，实际实现中需要从AST解析详细信息
        let go_ctx = GoContext::new(ast_ctx.clone());

        // 实现GO查询的规划逻辑
        println!("Processing GO query planning: {:?}", go_ctx);

        // 创建执行计划节点
        // 1. 首先创建起始节点
        let mut start_node = Arc::new(Start::new(1));

        // 2. 创建参数节点（如果需要）
        let mut arg_node = Arc::new(Argument::new(2, &go_ctx.from.user_defined_var_name));
        let mut arg_node_mut = Arc::get_mut(&mut arg_node).unwrap();
        arg_node_mut.set_col_names(vec!["vid".to_string()]);
        arg_node_mut.set_output_var(Variable {
            name: "start_vids".to_string(),
            columns: vec![],
        });

        // 3. 创建扩展节点
        let mut expand_node = Arc::new(Expand::new(
            3,
            go_ctx.base.statement_type().parse().unwrap_or(1),
            go_ctx.over.edge_types.clone(),
            "out",
        ));
        let mut expand_node_mut = Arc::get_mut(&mut expand_node).unwrap();
        expand_node_mut.add_dependency(arg_node.clone_plan_node());
        expand_node_mut.set_output_var(Variable {
            name: "expanded_vids".to_string(),
            columns: vec![],
        });
        expand_node_mut.set_col_names(vec!["_expand_vid".to_string()]);

        // 如果是双向扩展，设置边类型
        if go_ctx.over.direction == "both" {
            expand_node_mut.edge_types = go_ctx.over.edge_types.clone();
        } else if go_ctx.over.direction == "in" {
            // 对于入边，边类型取负值
            expand_node_mut.edge_types = go_ctx
                .over
                .edge_types
                .iter()
                .map(|et| format!("-{}", et))
                .collect();
        } else {
            // 默认是出边
            expand_node_mut.edge_types = go_ctx.over.edge_types.clone();
        }

        // 4. 创建ExpandAll节点进行多步扩展
        let direction = if go_ctx.over.direction == "both" {
            "both"
        } else if go_ctx.over.direction == "in" {
            "in"
        } else {
            "out"
        };
        let mut expand_all_node = Arc::new(ExpandAll::new(
            4,
            go_ctx.base.statement_type().parse().unwrap_or(1),
            go_ctx.over.edge_types.clone(),
            direction,
        ));
        let mut expand_all_node_mut = Arc::get_mut(&mut expand_all_node).unwrap();
        expand_all_node_mut.add_dependency(expand_node.clone_plan_node());
        expand_all_node_mut.set_output_var(Variable {
            name: "expanded_all_vids".to_string(),
            columns: vec![],
        });
        expand_all_node_mut.set_col_names(vec!["_expandall_vid".to_string()]);

        // 设置ExpandAll的边属性和顶点属性
        expand_all_node_mut.edge_props = go_ctx
            .expr_props
            .edge_props
            .iter()
            .map(|(edge_type, props)| EdgeProp::new(edge_type, props.clone()))
            .collect();

        expand_all_node_mut.vertex_props = go_ctx
            .expr_props
            .src_tag_props
            .iter()
            .map(|(tag, props)| TagProp::new(tag, props.clone()))
            .collect();

        // 5. 如果有JOIN操作，创建JOIN节点
        let mut join_node = if go_ctx.join_dst {
            let mut join = Arc::new(HashLeftJoin::new(5));
            let mut join_mut = Arc::get_mut(&mut join).unwrap();
            join_mut.add_dependency(expand_all_node.clone_plan_node());
            join_mut.set_output_var(Variable {
                name: "joined_result".to_string(),
                columns: vec![],
            });
            Some(join)
        } else {
            None
        };

        // 6. 创建过滤节点（如果有过滤条件）
        let mut filter_node = if let Some(ref condition) = go_ctx.filter {
            let mut filter = Arc::new(Filter::new(6, condition));
            let mut filter_mut = Arc::get_mut(&mut filter).unwrap();
            let dependency_node: Arc<dyn PlanNode> = if let Some(ref join_ref) = join_node {
                join_ref.clone()
            } else {
                expand_all_node.clone()
            };
            filter_mut.add_dependency(dependency_node);
            filter_mut.set_output_var(Variable {
                name: "filtered_result".to_string(),
                columns: vec![],
            });
            Some(filter)
        } else {
            None
        };

        // 7. 创建投影节点
        let mut project_node = Arc::new(Project::new(
            7,
            &go_ctx.yield_expr.clone().unwrap_or("DEFAULT".to_string()),
        ));
        let mut project_node_mut = Arc::get_mut(&mut project_node).unwrap();
        let last_node: Arc<dyn PlanNode> = if let Some(ref filter_ref) = filter_node {
            filter_ref.clone()
        } else if let Some(ref join_ref) = join_node {
            join_ref.clone()
        } else {
            expand_all_node.clone()
        };

        project_node_mut.add_dependency(last_node);
        let result_columns: Vec<crate::query::context::validate::types::Column> = go_ctx
            .col_names
            .iter()
            .map(|name| crate::query::context::validate::types::Column {
                name: name.clone(),
                type_: "String".to_string(), // 使用正确的类型
            })
            .collect();
        project_node_mut.set_output_var(Variable {
            name: "project_result".to_string(),
            columns: result_columns,
        });
        project_node_mut.set_col_names(go_ctx.col_names.clone());

        // 8. 如果需要去重，创建去重节点
        let final_node: Arc<dyn PlanNode> = if go_ctx.distinct {
            let mut dedup_node = Arc::new(Dedup::new(8));
            let mut dedup_node_mut = Arc::get_mut(&mut dedup_node).unwrap();
            dedup_node_mut.add_dependency(project_node.clone_plan_node());
            dedup_node_mut.set_output_var(Variable {
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
            tail: Some(start_node.clone_plan_node()),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for GoPlanner {
    fn default() -> Self {
        Self::new()
    }
}
