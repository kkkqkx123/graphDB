//! 更新操作规划器
//!
//! 处理 UPDATE VERTEX/EDGE 语句的查询规划

use crate::query::QueryContext;
use crate::query::parser::ast::{UpdateStmt, UpdateTarget, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        ArgumentNode, ProjectNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::core::{Expression, YieldColumn};
use std::sync::Arc;

/// 更新操作规划器
/// 负责将 UPDATE 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct UpdatePlanner;

impl UpdatePlanner {
    /// 创建新的更新规划器
    pub fn new() -> Self {
        Self
    }

    /// 从 Stmt 提取 UpdateStmt
    fn extract_update_stmt(&self, stmt: &Stmt) -> Result<UpdateStmt, PlannerError> {
        match stmt {
            Stmt::Update(update_stmt) => Ok(update_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "语句不包含 UPDATE".to_string(),
            )),
        }
    }
}

impl Planner for UpdatePlanner {
    fn transform(
        &mut self,
        stmt: &Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let update_stmt = self.extract_update_stmt(stmt)?;

        // 创建参数节点作为输入
        let arg_node = ArgumentNode::new(next_node_id(), "update_input");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // 根据更新目标类型构建输出列
        let target_name = match &update_stmt.target {
            UpdateTarget::Vertex(..) => "vertex",
            UpdateTarget::Edge { .. } => "edge",
            UpdateTarget::Tag(..) => "tag",
            UpdateTarget::TagOnVertex { .. } => "vertex_tag",
        };

        let yield_columns = vec![
            YieldColumn {
                expression: Expression::Variable(format!("updated_{}", target_name)),
                alias: "updated_count".to_string(),
                is_matched: false,
            }
        ];

        // 创建投影节点输出更新结果
        let project_node = ProjectNode::new(
            arg_node_enum.clone(),
            yield_columns,
        ).map_err(|e| PlannerError::PlanGenerationFailed(format!(
            "Failed to create ProjectNode: {}",
            e
        )))?;

        let final_node = PlanNodeEnum::Project(project_node);

        // 创建 SubPlan
        let sub_plan = SubPlan::new(
            Some(final_node),
            Some(arg_node_enum),
        );

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Update(_))
    }
}

impl Default for UpdatePlanner {
    fn default() -> Self {
        Self::new()
    }
}
