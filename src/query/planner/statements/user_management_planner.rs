//! 用户管理规划器
//! 处理用户管理相关的查询规划（CREATE USER、ALTER USER、DROP USER、CHANGE PASSWORD）

use crate::query::context::ast::AstContext;
use crate::query::planner::plan::core::{ArgumentNode, PlanNodeEnum};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

#[derive(Debug, Clone)]
pub struct UserManagementContext {
    pub base: AstContext,
}

/// 用户管理规划器
/// 负责将用户管理操作转换为执行计划
#[derive(Debug, Clone)]
pub struct UserManagementPlanner;

impl UserManagementPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        let stmt_type = ast_ctx.statement_type().to_uppercase();
        stmt_type == "CREATE_USER"
            || stmt_type == "ALTER_USER"
            || stmt_type == "DROP_USER"
            || stmt_type == "CHANGE_PASSWORD"
    }

    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiateEnum {
        crate::query::planner::planner::MatchAndInstantiateEnum::UserManagement(Self::new())
    }
}

impl Planner for UserManagementPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let user_ctx = UserManagementContext {
            base: ast_ctx.clone(),
        };

        let stmt_type = user_ctx.base.statement_type().to_uppercase();

        let arg_node = ArgumentNode::new(1, "user_management_args");

        let final_node = match stmt_type.as_str() {
            "CREATE_USER" => {
                let username = self.extract_string_value(ast_ctx, "username")?;
                let password = self.extract_string_value(ast_ctx, "password")?;
                let role = self.extract_optional_string_value(ast_ctx, "role");
                let if_not_exists = self.extract_bool_value(ast_ctx, "if_not_exists").unwrap_or(false);

                let mut node = crate::query::planner::plan::core::nodes::CreateUserNode::new(
                    1,
                    username,
                    password,
                );
                if let Some(r) = role {
                    node = node.with_role(r);
                }
                PlanNodeEnum::CreateUser(node)
            }
            "ALTER_USER" => {
                let username = self.extract_string_value(ast_ctx, "username")?;
                let new_role = self.extract_optional_string_value(ast_ctx, "new_role");
                let is_locked = self.extract_optional_bool_value(ast_ctx, "is_locked");

                let mut alter_info = crate::core::types::metadata::UserAlterInfo::new(username);
                if let Some(role) = new_role {
                    alter_info = alter_info.with_role(role);
                }
                if let Some(locked) = is_locked {
                    alter_info = alter_info.with_locked(locked);
                }

                let node = crate::query::planner::plan::core::nodes::AlterUserNode::new(
                    2,
                    alter_info,
                );
                PlanNodeEnum::AlterUser(node)
            }
            "DROP_USER" => {
                let username = self.extract_string_value(ast_ctx, "username")?;
                let if_exists = self.extract_bool_value(ast_ctx, "if_exists").unwrap_or(false);

                let node = crate::query::planner::plan::core::nodes::DropUserNode::new(
                    3,
                    username,
                    if_exists,
                );
                PlanNodeEnum::DropUser(node)
            }
            "CHANGE_PASSWORD" => {
                let username = self.extract_string_value(ast_ctx, "username")?;
                let old_password = self.extract_string_value(ast_ctx, "old_password")?;
                let new_password = self.extract_string_value(ast_ctx, "new_password")?;

                let password_info = crate::core::types::metadata::PasswordInfo {
                    username,
                    old_password,
                    new_password,
                };

                let node = crate::query::planner::plan::core::nodes::ChangePasswordNode::new(
                    4,
                    password_info,
                );
                PlanNodeEnum::ChangePassword(node)
            }
            _ => {
                return Err(PlannerError::PlanGenerationFailed(format!(
                    "Unsupported user management operation: {}",
                    stmt_type
                )))
            }
        };

        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for UserManagementPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl UserManagementPlanner {
    fn extract_string_value(&self, ast_ctx: &AstContext, key: &str) -> Result<String, PlannerError> {
        ast_ctx.get_parameter(key)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| PlannerError::PlanGenerationFailed(format!(
                "Missing required parameter: {}",
                key
            )))
    }

    fn extract_optional_string_value(&self, ast_ctx: &AstContext, key: &str) -> Option<String> {
        ast_ctx.get_parameter(key)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    fn extract_bool_value(&self, ast_ctx: &AstContext, key: &str) -> Option<bool> {
        ast_ctx.get_parameter(key)
            .and_then(|v| v.as_bool())
    }

    fn extract_optional_bool_value(&self, ast_ctx: &AstContext, key: &str) -> Option<bool> {
        ast_ctx.get_parameter(key)
            .and_then(|v| v.as_bool())
    }
}
