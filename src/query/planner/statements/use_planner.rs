//! USE 语句规划器
//!
//! 处理 USE <space> 语句的查询规划

use crate::query::context::ast::AstContext;
use crate::query::parser::ast::{UseStmt, Stmt};
use crate::query::planner::plan::core::{
    node_id_generator::next_node_id,
    nodes::{
        ArgumentNode, ProjectNode,
    },
};
use crate::query::planner::plan::{PlanNodeEnum, SubPlan};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::core::{Expression, YieldColumn};

/// USE 语句规划器
/// 负责将 USE 语句转换为执行计划
#[derive(Debug, Clone)]
pub struct UsePlanner;

impl UsePlanner {
    /// 创建新的 USE 规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查 AST 上下文是否匹配 USE 语句
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        matches!(ast_ctx.sentence(), Some(Stmt::Use(_)))
    }

    /// 获取匹配和实例化函数（静态注册版本）
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiateEnum {
        crate::query::planner::planner::MatchAndInstantiateEnum::Use(Self::new())
    }

    /// 从 AstContext 提取 UseStmt
    fn extract_use_stmt(&self, ast_ctx: &AstContext) -> Result<UseStmt, PlannerError> {
        match ast_ctx.sentence() {
            Some(Stmt::Use(use_stmt)) => Ok(use_stmt.clone()),
            _ => Err(PlannerError::PlanGenerationFailed(
                "AST 上下文中不包含 USE 语句".to_string(),
            )),
        }
    }
}

impl Planner for UsePlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let use_stmt = self.extract_use_stmt(ast_ctx)?;

        // 创建参数节点作为输入
        let arg_node = ArgumentNode::new(next_node_id(), "use_input");
        let arg_node_enum = PlanNodeEnum::Argument(arg_node.clone());

        // 构建输出列，显示切换的空间名
        let yield_columns = vec![
            YieldColumn {
                expression: Expression::string(use_stmt.space.clone()),
                alias: "space_name".to_string(),
                is_matched: false,
            }
        ];

        // 创建投影节点
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

impl Default for UsePlanner {
    fn default() -> Self {
        Self::new()
    }
}
