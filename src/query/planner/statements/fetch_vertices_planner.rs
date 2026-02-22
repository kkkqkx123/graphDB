//! FETCH VERTICES查询规划器
//! 处理FETCH VERTICES查询的规划

use crate::query::QueryContext;
use crate::query::parser::ast::{FetchTarget, Stmt};
use crate::query::planner::plan::core::common::TagProp;
use crate::query::planner::plan::core::nodes::{
    ArgumentNode, GetVerticesNode, PlanNodeEnum, ProjectNode,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use std::sync::Arc;

/// FETCH VERTICES查询规划器
/// 负责将FETCH VERTICES查询转换为执行计划
#[derive(Debug, Clone)]
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
}

impl Planner for FetchVerticesPlanner {
    fn transform(
        &mut self,
        stmt: &Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let fetch_stmt = match stmt {
            Stmt::Fetch(fetch_stmt) => fetch_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "FetchVerticesPlanner 需要 Fetch 语句".to_string()
                ));
            }
        };

        // 检查是否是 FETCH VERTICES
        let (_ids, properties) = match &fetch_stmt.target {
            FetchTarget::Vertices { ids, properties, .. } => (ids, properties),
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "FetchVerticesPlanner 需要 FETCH VERTICES 语句".to_string()
                ));
            }
        };

        let var_name = "v";

        // 1. 创建参数节点，获取顶点ID
        let mut arg_node = ArgumentNode::new(1, var_name);
        arg_node.set_col_names(vec!["vid".to_string()]);
        arg_node.set_output_var("vertex_ids".to_string());

        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        let mut get_vertices_node = GetVerticesNode::new(2, var_name);
        get_vertices_node.add_dependency(arg_node_enum.clone());
        get_vertices_node.set_output_var("fetched_vertices".to_string());

        // 设置标签属性（从 properties 字段获取）
        let tag_props = if let Some(props) = properties {
            vec![TagProp::new("default", props.clone())]
        } else {
            vec![]
        };
        get_vertices_node.set_tag_props(tag_props);

        let get_vertices_node_enum = PlanNodeEnum::GetVertices(get_vertices_node);

        // 3. 创建投影节点
        let project_node = ProjectNode::new(get_vertices_node_enum.clone(), vec![])?;

        let project_node_enum = PlanNodeEnum::Project(project_node);

        // 4. 创建SubPlan
        let sub_plan = SubPlan::new(Some(project_node_enum), Some(arg_node_enum));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Fetch(fetch_stmt) => {
                matches!(&fetch_stmt.target, FetchTarget::Vertices { .. })
            }
            _ => false,
        }
    }
}

impl Default for FetchVerticesPlanner {
    fn default() -> Self {
        Self::new()
    }
}
