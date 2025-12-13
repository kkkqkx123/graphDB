//! SUBGRAPH查询规划器
//! 处理Nebula SUBGRAPH查询的规划

use crate::query::context::{AstContext, SubgraphContext};
use crate::query::planner::plan::core::common::{EdgeProp, TagProp};
use crate::query::planner::plan::operations::{Argument, Filter, Project, Expand, ExpandAll};
use crate::query::planner::plan::PlanNode;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::plan_node_traits::{PlanNodeClonable, PlanNodeMutable, PlanNodeDependencies};
use std::sync::Arc;

/// SUBGRAPH查询规划器
/// 负责将SUBGRAPH查询转换为执行计划
#[derive(Debug)]
pub struct SubgraphPlanner;

impl SubgraphPlanner {
    /// 创建新的SUBGRAPH规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配SUBGRAPH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "SUBGRAPH"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
        }
    }
}

impl Planner for SubgraphPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 从ast_ctx创建SubgraphContext
        let subgraph_ctx = SubgraphContext::new(ast_ctx.clone());

        // 实现SUBGRAPH查询的规划逻辑
        println!("Processing SUBGRAPH query planning: {:?}", subgraph_ctx);

        // 1. 创建参数节点，获取起始顶点
        let mut arg_node = Argument::new(1, &subgraph_ctx.from.user_defined_var_name);
        arg_node.set_col_names(vec!["start_vid".to_string()]);
        arg_node.set_output_var(Variable {
            name: "start_vids".to_string(),
            columns: vec![],
        });
        let arg_node = Arc::new(arg_node);

        // 2. 创建扩展节点进行子图扩展
        let mut expand_node = Expand::new(2, 1, subgraph_ctx.edge_types.clone(), "out");
        expand_node.add_dependency(arg_node.clone_plan_node());
        expand_node.set_output_var(Variable {
            name: "expanded_subgraph".to_string(),
            columns: vec![],
        });

        // 设置边类型
        expand_node.edge_types = subgraph_ctx.edge_types.clone();

        // 如果需要双向扩展
        if subgraph_ctx.bi_direct_edge_types.len() > 0 {
            expand_node
                .edge_types
                .extend(subgraph_ctx.bi_direct_edge_types.clone());
        }
        let expand_node = Arc::new(expand_node);

        // 3. 创建ExpandAll节点进行多步扩展
        let mut expand_all_node = ExpandAll::new(
            3,
            1,
            subgraph_ctx.edge_types.clone(),
            "out"
        );
        expand_all_node.add_dependency(expand_node.clone_plan_node());
        expand_all_node.set_output_var(Variable {
            name: "expanded_all_subgraph".to_string(),
            columns: vec![],
        });

        // 设置边属性和顶点属性
        expand_all_node.edge_props = subgraph_ctx
            .expr_props
            .edge_props
            .iter()
            .map(|(edge_type, props)| EdgeProp::new(edge_type, props.clone()))
            .collect();

        expand_all_node.vertex_props = subgraph_ctx
            .expr_props
            .src_tag_props
            .iter()
            .map(|(tag, props)| TagProp::new(tag, props.clone()))
            .collect();
        let expand_all_node = Arc::new(expand_all_node);

        // 4. 创建过滤节点（如果有过滤条件）
        let filter_node: Arc<dyn crate::query::planner::plan::core::PlanNode> = if let Some(ref condition) = subgraph_ctx.filter {
            let mut filter = Filter::new(4, condition);
            filter.add_dependency(expand_all_node.clone_plan_node());
            filter.set_output_var(Variable {
                name: "filtered_subgraph".to_string(),
                columns: vec![],
            });
            Arc::new(filter)
        } else {
            expand_all_node
        };

        // 5. 如果有标签过滤，添加额外过滤
        let tag_filter_node = if let Some(ref tag_condition) = subgraph_ctx.tag_filter {
            let mut filter = Filter::new(5, tag_condition);
            filter.add_dependency(filter_node.clone_plan_node());
            filter.set_output_var(Variable {
                name: "tag_filtered_subgraph".to_string(),
                columns: vec![],
            });
            Arc::new(filter)
        } else {
            filter_node
        };

        // 6. 如果有边过滤，添加额外过滤
        let edge_filter_node = if let Some(ref edge_condition) = subgraph_ctx.edge_filter {
            let mut filter = Filter::new(6, edge_condition);
            filter.add_dependency(tag_filter_node.clone_plan_node());
            filter.set_output_var(Variable {
                name: "edge_filtered_subgraph".to_string(),
                columns: vec![],
            });
            Arc::new(filter)
        } else {
            tag_filter_node
        };

        // 7. 创建投影节点
        let mut project_node = Project::new(7, &"DEFAULT".to_string());
        project_node.add_dependency(edge_filter_node.clone_plan_node());
        project_node.set_output_var(Variable {
            name: "projected_subgraph".to_string(),
            columns: vec![],
        });
        project_node.set_col_names(subgraph_ctx.col_names.clone());
        let project_node = Arc::new(project_node);

        // 8. 如果需要返回属性，设置属性获取
        if subgraph_ctx.get_vertex_prop {
            // 可能需要额外的GetVertices节点来获取顶点属性
        }

        if subgraph_ctx.get_edge_prop {
            // 可能需要额外的GetEdges节点来获取边属性
        }

        // 创建SubPlan
        let sub_plan = SubPlan {
            root: Some(project_node.clone_plan_node()),
            tail: Some(arg_node.clone_plan_node()),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for SubgraphPlanner {
    fn default() -> Self {
        Self::new()
    }
}
