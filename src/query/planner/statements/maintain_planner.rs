//! 维护操作规划器
//! 处理维护相关的查询规划（如SUBMIT JOB等）

use crate::query::parser::ast::{AlterTarget, ShowTarget, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id, AlterSpaceNode, ArgumentNode, ClearSpaceNode, PlanNodeEnum,
    ProjectNode, ShowStatsNode, ShowStatsType,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::QueryContext;
use std::sync::Arc;

/// 维护操作规划器
/// 负责将维护操作转换为执行计划
#[derive(Debug, Clone)]
pub struct MaintainPlanner;

impl MaintainPlanner {
    /// 创建新的维护规划器
    pub fn new() -> Self {
        Self
    }
}

impl Planner for MaintainPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let stmt_type = validated.stmt().kind().to_uppercase();

        // 1. 创建参数节点来接收操作参数
        let arg_node = ArgumentNode::new(1, "maintain_args");

        // 2. 根据不同类型创建相应的计划节点
        // 维护操作通常不需要表达式，直接返回操作结果
        let yield_columns = Vec::new();

        let project_node = ProjectNode::new(
            PlanNodeEnum::Argument(arg_node.clone()),
            yield_columns,
        )
        .map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Failed to create ProjectNode: {}", e))
        })?;

        // 3. 不同类型的操作可能需要不同处理
        let final_node = if stmt_type == "SHOW" {
            // 处理 SHOW STATS 语句
            if let Stmt::Show(show_stmt) = validated.stmt() {
                if show_stmt.target == ShowTarget::Stats {
                    let stats_node = ShowStatsNode::new(
                        next_node_id(),
                        ShowStatsType::Storage,
                    );
                    PlanNodeEnum::ShowStats(stats_node)
                } else {
                    // 其他 SHOW 语句使用 PassThrough 节点
                    PlanNodeEnum::PassThrough(crate::query::planner::plan::core::PassThroughNode::new(1))
                }
            } else {
                PlanNodeEnum::PassThrough(crate::query::planner::plan::core::PassThroughNode::new(1))
            }
        } else if stmt_type == "SUBMIT JOB" {
            // 提交作业类型的维护操作
            PlanNodeEnum::Project(project_node)
        } else if stmt_type.starts_with("CREATE") {
            // 创建类型的操作
            PlanNodeEnum::Project(project_node)
        } else if stmt_type.starts_with("ALTER") {
            // 处理 ALTER SPACE 语句
            if let Stmt::Alter(alter_stmt) = validated.stmt() {
                if let AlterTarget::Space { space_name, comment } = &alter_stmt.target {
                    let options = comment.as_ref().map(|c| {
                        vec![crate::query::planner::plan::core::nodes::SpaceAlterOption::Comment(c.clone())]
                    }).unwrap_or_default();
                    let alter_space_node = AlterSpaceNode::new(
                        next_node_id(),
                        space_name.clone(),
                        options,
                    );
                    PlanNodeEnum::AlterSpace(alter_space_node)
                } else {
                    PlanNodeEnum::Project(project_node)
                }
            } else {
                PlanNodeEnum::Project(project_node)
            }
        } else if stmt_type == "CLEAR SPACE" {
            // 处理 CLEAR SPACE 语句
            if let Stmt::ClearSpace(clear_stmt) = validated.stmt() {
                let clear_space_node = ClearSpaceNode::new(
                    next_node_id(),
                    clear_stmt.space_name.clone(),
                );
                PlanNodeEnum::ClearSpace(clear_space_node)
            } else {
                PlanNodeEnum::Project(project_node)
            }
        } else {
            // 其他类型的维护操作
            PlanNodeEnum::Project(project_node)
        };

        // 创建SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        let stmt_type = stmt.kind().to_uppercase();
        stmt_type == "SUBMIT JOB"
            || stmt_type.starts_with("CREATE")
            || stmt_type.starts_with("DROP")
            || stmt_type.starts_with("SHOW")
            || stmt_type == "DESC"
            || stmt_type.starts_with("ALTER")
            || stmt_type.starts_with("DESCRIBE")
            || stmt_type == "CLEAR SPACE"
    }
}

impl Default for MaintainPlanner {
    fn default() -> Self {
        Self::new()
    }
}
