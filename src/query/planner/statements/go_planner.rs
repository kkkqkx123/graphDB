//! GO语句规划器
//! 处理Nebula GO查询的规划
//!
//! ## 改进说明
//!
//! - 实现完整的表达式过滤逻辑
//! - 改进 JOIN 键处理
//! - 添加属性投影支持

use crate::core::types::EdgeDirection;
use crate::query::QueryContext;
use crate::query::parser::ast::{GoStmt, Stmt};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use std::sync::Arc;

pub use crate::query::planner::plan::core::nodes::{
    ArgumentNode, DedupNode, ExpandAllNode, FilterNode, GetNeighborsNode, HashInnerJoinNode,
    ProjectNode,
};
pub use crate::query::planner::plan::core::PlanNodeEnum;

/// GO查询规划器
/// 负责将GO语句转换为执行计划
#[derive(Debug, Clone)]
pub struct GoPlanner {}

impl GoPlanner {
    /// 创建新的GO规划器
    pub fn new() -> Self {
        Self {}
    }
}

impl Planner for GoPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        _qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let go_stmt = match &validated.stmt {
            Stmt::Go(go_stmt) => go_stmt,
            _ => {
                return Err(PlannerError::InvalidOperation(
                    "GoPlanner 需要 Go 语句".to_string()
                ));
            }
        };

        let from_var = "v";
        let arg_node = ArgumentNode::new(0, from_var);
        let arg_node_enum = PlanNodeEnum::Argument(arg_node);

        let (direction_str, edge_types) = if let Some(over_clause) = &go_stmt.over {
            let direction_str = match over_clause.direction {
                EdgeDirection::Out => "out",
                EdgeDirection::In => "in",
                EdgeDirection::Both => "both",
            };
            (direction_str, over_clause.edge_types.clone())
        } else {
            ("both", vec![])
        };

        let expand_all_node = ExpandAllNode::new(
            1,
            edge_types,
            direction_str,
        );

        let input_for_join = PlanNodeEnum::ExpandAll(expand_all_node);

        let filter_node = if let Some(ref condition) = go_stmt.where_clause {
            use std::sync::Arc;
            let ctx = Arc::new(crate::core::types::ExpressionContext::new());
            match FilterNode::from_expression(input_for_join, condition.clone(), ctx) {
                Ok(filter) => PlanNodeEnum::Filter(filter),
                Err(e) => {
                    return Err(PlannerError::PlanGenerationFailed(format!(
                        "Failed to create filter node: {}",
                        e
                    )));
                }
            }
        } else {
            input_for_join
        };

        let project_columns = Self::build_yield_columns(go_stmt)?;
        let project_node = match ProjectNode::new(filter_node, project_columns) {
            Ok(project) => PlanNodeEnum::Project(project),
            Err(e) => {
                return Err(PlannerError::PlanGenerationFailed(format!(
                    "Failed to create project node: {}",
                    e
                )));
            }
        };

        let sub_plan = SubPlan {
            root: Some(project_node),
            tail: Some(arg_node_enum),
        };

        Ok(sub_plan)
    }

    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Go(_))
    }
}

impl GoPlanner {
    /// 构建YIELD列
    fn build_yield_columns(
        go_stmt: &GoStmt,
    ) -> Result<Vec<crate::core::YieldColumn>, PlannerError> {
        let mut columns = Vec::new();

        if let Some(ref yield_clause) = go_stmt.yield_clause {
            for item in &yield_clause.items {
                columns.push(crate::core::YieldColumn {
                    expression: item.expression.clone(),
                    alias: item.alias.clone().unwrap_or_default(),
                    is_matched: false,
                });
            }
        } else {
            columns.push(crate::core::YieldColumn {
                expression: crate::core::Expression::Variable("_expandall_dst".to_string()),
                alias: "dst".to_string(),
                is_matched: false,
            });

            columns.push(crate::core::YieldColumn {
                expression: crate::core::Expression::Variable("_expandall_props".to_string()),
                alias: "properties".to_string(),
                is_matched: false,
            });
        }

        if columns.is_empty() {
            columns.push(crate::core::YieldColumn {
                expression: crate::core::Expression::Variable("*".to_string()),
                alias: "result".to_string(),
                is_matched: false,
            });
        }

        Ok(columns)
    }
}

impl Default for GoPlanner {
    fn default() -> Self {
        Self::new()
    }
}
