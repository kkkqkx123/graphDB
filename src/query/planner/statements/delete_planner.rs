//! 删除操作规划器
//!
//! 处理 DELETE VERTEX/EDGE/TAG 语句的查询规划

use crate::query::context::ast::AstContext;
use crate::query::parser::ast::{DeleteStmt, DeleteTarget, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        ArgumentNode, ProjectNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::core::{Expression, YieldColumn};

/// 删除操作规划器
/// 负责将 DELETE 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct DeletePlanner;

impl DeletePlanner {
    /// 创建新的删除规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查 AST 上下文是否匹配删除操作
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        matches!(ast_ctx.sentence(), Some(Stmt::Delete(_)))
    }

    /// 获取匹配和实例化函数（静态注册版本）
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiateEnum {
        crate::query::planner::planner::MatchAndInstantiateEnum::Delete(Self::new())
    }

    /// 从 AstContext 提取 DeleteStmt
    fn extract_delete_stmt(&self, ast_ctx: &AstContext) -> Result<DeleteStmt, PlannerError> {
        match ast_ctx.sentence() {
            Some(Stmt::Delete(delete_stmt)) => Ok(delete_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "AST 上下文中不包含 DELETE 语句".to_string(),
            )),
        }
    }
}

impl Planner for DeletePlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let delete_stmt = self.extract_delete_stmt(ast_ctx)?;

        // 创建参数节点作为输入
        let arg_node = ArgumentNode::new(next_node_id(), "delete_input");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // 根据删除目标类型构建不同的计划
        let yield_columns = match &delete_stmt.target {
            DeleteTarget::Vertices(..) => {
                vec![YieldColumn {
                    expression: Expression::Variable("deleted_vertices".to_string()),
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
            DeleteTarget::Edges { .. } => {
                vec![YieldColumn {
                    expression: Expression::Variable("deleted_edges".to_string()),
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
            DeleteTarget::Tags { .. } => {
                vec![YieldColumn {
                    expression: Expression::Variable("deleted_tags".to_string()),
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
            DeleteTarget::Index(..) => {
                vec![YieldColumn {
                    expression: Expression::Variable("deleted_index".to_string()),
                    alias: "deleted_count".to_string(),
                    is_matched: false,
                }]
            }
        };

        // 创建投影节点输出删除结果
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

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for DeletePlanner {
    fn default() -> Self {
        Self::new()
    }
}
