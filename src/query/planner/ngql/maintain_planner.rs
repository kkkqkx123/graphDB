//! 维护操作规划器
//! 处理维护相关的查询规划（如SUBMIT JOB等）

use crate::query::context::ast::{AstContext, MaintainContext};
use crate::query::planner::plan::core::{ArgumentNode, PlanNodeEnum, ProjectNode};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

/// 维护操作规划器
/// 负责将维护操作转换为执行计划
#[derive(Debug)]
pub struct MaintainPlanner;

impl MaintainPlanner {
    /// 创建新的维护规划器
    pub fn new() -> Self {
        Self
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配维护操作
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        let stmt_type = ast_ctx.statement_type().to_uppercase();
        stmt_type == "SUBMIT JOB"
            || stmt_type.starts_with("CREATE")
            || stmt_type.starts_with("DROP")
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
            priority: 100,
        }
    }
}

impl Planner for MaintainPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 从ast_ctx创建MaintainContext
        let maintain_ctx = MaintainContext {
            base: ast_ctx.clone(),
        };

        // 实现维护操作的规划逻辑
        println!("Processing MAINTENANCE query planning: {:?}", maintain_ctx);

        // 根据操作类型创建相应的计划节点
        let stmt_type = maintain_ctx.base.statement_type().to_uppercase();

        // 1. 创建参数节点来接收操作参数
        let arg_node = ArgumentNode::new(1, "maintain_args");

        // 2. 根据不同类型创建相应的计划节点
        use crate::core::Expression;
        use crate::query::validator::YieldColumn;
        let yield_columns = vec![YieldColumn {
            expr: Expression::Variable(format!("MAINTAIN_{}", stmt_type)),
            alias: "maintain_result".to_string(),
            is_matched: false,
        }];

        let project_node =
            ProjectNode::new(PlanNodeEnum::Argument(arg_node.clone()), yield_columns)
                .expect("ProjectNode creation should succeed with valid input");

        // 3. 不同类型的操作可能需要不同处理
        let final_node = if stmt_type == "SUBMIT JOB" {
            // 提交作业类型的维护操作
            PlanNodeEnum::Project(project_node)
        } else if stmt_type.starts_with("CREATE") {
            // 创建类型的操作
            PlanNodeEnum::Project(project_node)
        } else if stmt_type.starts_with("DROP") {
            // 删除类型的操作
            PlanNodeEnum::Project(project_node)
        } else {
            // 其他类型的维护操作
            PlanNodeEnum::Project(project_node)
        };

        // 创建SubPlan
        let sub_plan = SubPlan::new(Some(final_node), Some(PlanNodeEnum::Argument(arg_node)));

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for MaintainPlanner {
    fn default() -> Self {
        Self::new()
    }
}
