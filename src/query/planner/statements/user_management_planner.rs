//! 用户管理规划器
//! 处理用户管理相关的查询规划（CREATE USER、ALTER USER、DROP USER、CHANGE PASSWORD）

use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::core::{ArgumentNode, PlanNodeEnum};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};
use std::sync::Arc;

/// 用户管理规划器
/// 负责将用户管理操作转换为执行计划
#[derive(Debug, Clone)]
pub struct UserManagementPlanner;

impl UserManagementPlanner {
    pub fn new() -> Self {
        Self
    }
}

impl Planner for UserManagementPlanner {
    fn transform(
        &mut self,
        stmt: &Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let arg_node = ArgumentNode::new(1, "user_management_args");

        let final_node = match stmt {
            Stmt::CreateUser(create_stmt) => {
                let mut node = crate::query::planner::plan::core::nodes::CreateUserNode::new(
                    1,
                    create_stmt.username.clone(),
                    create_stmt.password.clone(),
                );
                if let Some(ref role) = create_stmt.role {
                    node = node.with_role(role.clone());
                }
                PlanNodeEnum::CreateUser(node)
            }
            Stmt::AlterUser(alter_stmt) => {
                let mut node = crate::query::planner::plan::core::nodes::AlterUserNode::new(
                    2,
                    alter_stmt.username.clone(),
                );
                if let Some(ref role) = alter_stmt.new_role {
                    node = node.with_role(role.clone());
                }
                if let Some(locked) = alter_stmt.is_locked {
                    node = node.with_locked(locked);
                }
                PlanNodeEnum::AlterUser(node)
            }
            Stmt::DropUser(drop_stmt) => {
                let node = crate::query::planner::plan::core::nodes::DropUserNode::new(
                    3,
                    drop_stmt.username.clone(),
                );
                PlanNodeEnum::DropUser(node)
            }
            Stmt::ChangePassword(change_stmt) => {
                let password_info = crate::core::types::metadata::PasswordInfo {
                    username: change_stmt.username.clone(),
                    old_password: change_stmt.old_password.clone(),
                    new_password: change_stmt.new_password.clone(),
                };

                let node = crate::query::planner::plan::core::nodes::ChangePasswordNode::new(
                    4,
                    password_info,
                );
                PlanNodeEnum::ChangePassword(node)
            }
            _ => {
                return Err(PlannerError::PlanGenerationFailed(format!(
                    "Unsupported user management operation: {:?}",
                    stmt
                )))
            }
        };

        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt,
            Stmt::CreateUser(_) |
            Stmt::AlterUser(_) |
            Stmt::DropUser(_) |
            Stmt::ChangePassword(_)
        )
    }
}

impl Default for UserManagementPlanner {
    fn default() -> Self {
        Self::new()
    }
}
