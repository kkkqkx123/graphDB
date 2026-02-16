//! PATH查询规划器
//! 处理Nebula PATH查询的规划
//!
//! ## 改进说明
//!
//! - 实现最短路径规划
//! - 实现所有路径规划
//! - 支持带权最短路径
//! - 完善路径过滤逻辑

use crate::query::context::ast::{AstContext, PathContext};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::algorithms::{ShortestPath, AllPaths};
use crate::query::planner::plan::core::PlanNode;
use crate::query::planner::planner::{Planner, PlannerError};

pub use crate::query::planner::plan::core::nodes::{
    ArgumentNode, DedupNode, ExpandAllNode, FilterNode, GetNeighborsNode, ProjectNode,
    StartNode,
};
pub use crate::query::planner::plan::core::PlanNodeEnum;

/// PATH查询规划器
/// 负责将PATH查询转换为执行计划
#[derive(Debug, Clone)]
pub struct PathPlanner {}

impl PathPlanner {
    /// 创建新的PATH规划器
    pub fn new() -> Self {
        Self {}
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配PATH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "PATH"
            || ast_ctx.statement_type().to_uppercase() == "FIND PATH"
    }

    /// 从AST构建PathContext
    fn build_path_context(&self, ast_ctx: &AstContext) -> PathContext {
        let mut path_ctx = PathContext::new(ast_ctx.clone());

        // 检查是否为最短路径查询
        path_ctx.is_shortest = self.is_shortest_path(ast_ctx);
        path_ctx.single_shortest = self.is_single_shortest_path(ast_ctx);

        // 检查是否为带权查询并获取权重、启发式表达式
        path_ctx.is_weight = self.is_weighted_path(ast_ctx);
        if let Some(stmt) = ast_ctx.sentence() {
            if let crate::query::parser::ast::Stmt::FindPath(find_path_stmt) = stmt {
                path_ctx.weight_expression = find_path_stmt.weight_expression.clone();
                path_ctx.heuristic_expression = find_path_stmt.heuristic_expression.clone();
            }
        }

        path_ctx
    }

    /// 检查是否为带权路径查询
    fn is_weighted_path(&self, ast_ctx: &AstContext) -> bool {
        // 从AST语句中检查是否有weight表达式
        if let Some(stmt) = ast_ctx.sentence() {
            if let crate::query::parser::ast::Stmt::FindPath(find_path_stmt) = stmt {
                return find_path_stmt.weight_expression.is_some();
            }
        }
        false
    }

    /// 获取最大步数
    fn get_max_steps(&self, ast_ctx: &AstContext) -> usize {
        if let Some(stmt) = ast_ctx.sentence() {
            if let crate::query::parser::ast::Stmt::FindPath(find_path_stmt) = stmt {
                return find_path_stmt.max_steps.unwrap_or(5);
            }
        }
        5
    }

    /// 获取边类型
    fn get_edge_types(&self, ast_ctx: &AstContext) -> Vec<String> {
        if let Some(stmt) = ast_ctx.sentence() {
            if let crate::query::parser::ast::Stmt::FindPath(find_path_stmt) = stmt {
                if let Some(ref over) = find_path_stmt.over {
                    return over.edge_types.clone();
                }
            }
        }
        Vec::new()
    }
}

impl Planner for PathPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let path_ctx = self.build_path_context(ast_ctx);

        // 创建起始节点
        let start_node = StartNode::new();
        let start_node_enum = PlanNodeEnum::Start(start_node);

        let edge_types = self.get_edge_types(ast_ctx);
        let max_steps = self.get_max_steps(ast_ctx);

        // 根据查询类型选择不同的计划策略
        let root_node = if path_ctx.is_shortest {
            // 最短路径查询
            self.build_shortest_path_plan(
                &path_ctx,
                start_node_enum.clone(),
                edge_types,
                max_steps,
            )?
        } else {
            // 所有路径查询
            self.build_all_paths_plan(
                &path_ctx,
                start_node_enum.clone(),
                edge_types,
                max_steps,
            )?
        };

        let sub_plan = SubPlan {
            root: Some(root_node),
            tail: Some(start_node_enum),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl PathPlanner {
    /// 构建最短路径计划
    fn build_shortest_path_plan(
        &self,
        path_ctx: &PathContext,
        left_input: PlanNodeEnum,
        edge_types: Vec<String>,
        max_steps: usize,
    ) -> Result<PlanNodeEnum, PlannerError> {
        // 创建右侧输入节点（终点）
        let right_node = StartNode::new();
        let right_node_enum = PlanNodeEnum::Start(right_node);

        // 创建ShortestPath计划节点
        let mut shortest_path_node = ShortestPath::new(
            2,
            left_input,
            right_node_enum,
            edge_types,
            max_steps,
        );

        // 设置权重表达式（如果存在）
        if let Some(ref weight_expr) = path_ctx.weight_expression {
            shortest_path_node.set_weight_expression(weight_expr.clone());
        }

        // 设置启发式表达式（如果存在）
        if let Some(ref heuristic_expr) = path_ctx.heuristic_expression {
            shortest_path_node.set_heuristic_expression(heuristic_expr.clone());
        }

        Ok(shortest_path_node.into_enum())
    }

    /// 构建所有路径计划
    fn build_all_paths_plan(
        &self,
        _path_ctx: &PathContext,
        left_input: PlanNodeEnum,
        edge_types: Vec<String>,
        max_steps: usize,
    ) -> Result<PlanNodeEnum, PlannerError> {
        // 创建右侧输入节点（终点）
        let right_node = StartNode::new();
        let right_node_enum = PlanNodeEnum::Start(right_node);

        // 创建AllPaths计划节点
        let all_paths_node = AllPaths::new(
            2,
            left_input,
            right_node_enum,
            max_steps,
            edge_types,
            1,
            max_steps,
            false,
        );

        Ok(all_paths_node.into_enum())
    }

    /// 检查是否为最短路径查询
    pub fn is_shortest_path(&self, ast_ctx: &AstContext) -> bool {
        let statement = ast_ctx.statement_type().to_uppercase();
        statement.contains("SHORTEST")
    }

    /// 检查是否为单最短路径查询
    pub fn is_single_shortest_path(&self, ast_ctx: &AstContext) -> bool {
        let statement = ast_ctx.statement_type().to_uppercase();
        statement.contains("SHORTEST") && !statement.contains("ALL SHORTEST")
    }

    /// 检查是否为所有路径查询
    pub fn is_all_paths(&self, ast_ctx: &AstContext) -> bool {
        let statement = ast_ctx.statement_type().to_uppercase();
        statement.contains("ALL PATH") || !statement.contains("SHORTEST")
    }
}

impl Default for PathPlanner {
    fn default() -> Self {
        Self::new()
    }
}
