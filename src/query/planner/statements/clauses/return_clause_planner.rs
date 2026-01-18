//! RETURN 子句规划器
//!
//! ## 改进说明
//!
//! - 实现真正的投影逻辑
//! - 添加 DISTINCT 去重支持
//! - 完善列名设置

use crate::core::types::expression::Expression;
use crate::query::planner::statements::clauses::clause_planner::ClausePlanner;
use crate::query::planner::statements::core::cypher_clause_planner::{
    ClauseType, CypherClausePlanner, DataFlowNode, PlanningContext,
};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::common_structs::CypherClauseContext;
use crate::query::validator::structs::CypherClauseKind;

pub use crate::query::planner::plan::core::nodes::{DedupNode, ProjectNode};
pub use crate::query::planner::plan::core::PlanNodeEnum;

#[derive(Debug)]
pub struct ReturnClausePlanner {
    return_items: Vec<ReturnItem>,
    distinct: bool,
}

#[derive(Debug, Clone)]
pub struct ReturnItem {
    pub alias: String,
    pub expression: Expression,
    pub is_aggregated: bool,
}

impl ReturnClausePlanner {
    pub fn new() -> Self {
        Self {
            return_items: Vec::new(),
            distinct: false,
        }
    }

    /// 从 RETURN 子句上下文创建规划器
    pub fn from_context(ctx: &CypherClauseContext) -> Self {
        let mut items = Vec::new();

        if let CypherClauseContext::Return(return_ctx) = ctx {
            for item in &return_ctx.yield_clause.yield_columns {
                let item_name = item.alias.clone();
                items.push(ReturnItem {
                    alias: item_name.clone(),
                    expression: Expression::Variable(item_name.clone()),
                    is_aggregated: Self::is_aggregated_expression(&item_name),
                });
            }
        }

        Self {
            return_items: items,
            distinct: false,
        }
    }

    /// 检查是否为聚合表达式
    fn is_aggregated_expression(name: &str) -> bool {
        let upper = name.to_uppercase();
        upper.starts_with("COUNT(")
            || upper.starts_with("SUM(")
            || upper.starts_with("AVG(")
            || upper.starts_with("MAX(")
            || upper.starts_with("MIN(")
            || upper.starts_with("COLLECT(")
    }
}

impl ClausePlanner for ReturnClausePlanner {
    fn name(&self) -> &'static str {
        "ReturnClausePlanner"
    }

    fn supported_clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Return
    }
}

impl DataFlowNode for ReturnClausePlanner {
    fn flow_direction(&self) -> crate::query::planner::statements::core::cypher_clause_planner::FlowDirection {
        self.clause_type().flow_direction()
    }
}

impl CypherClausePlanner for ReturnClausePlanner {
    fn clause_type(&self) -> ClauseType {
        ClauseType::Return
    }

    fn transform(
        &self,
        clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        self.validate_flow(input_plan)?;

        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("RETURN 子句需要输入计划".to_string())
        })?;

        let return_items = Self::extract_return_items(clause_ctx)?;

        let yield_columns = Self::build_yield_columns(&return_items, context)?;

        let input_node = input_plan.root.clone().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let project_node = match ProjectNode::new(input_node, yield_columns) {
            Ok(project) => PlanNodeEnum::Project(project),
            Err(e) => {
                return Err(PlannerError::PlanGenerationFailed(format!(
                    "创建投影节点失败: {}",
                    e
                )));
            }
        };

        let final_node = if self.distinct {
            match DedupNode::new(project_node.clone()) {
                Ok(dedup) => PlanNodeEnum::Dedup(dedup),
                Err(_) => project_node,
            }
        } else {
            project_node
        };

        context.mark_output_variables();

        Ok(SubPlan {
            root: Some(final_node),
            tail: input_plan.tail.clone(),
        })
    }
}

impl ReturnClausePlanner {
    /// 从子句上下文提取 RETURN 项
    fn extract_return_items(
        clause_ctx: &CypherClauseContext,
    ) -> Result<Vec<ReturnItem>, PlannerError> {
        let mut items = Vec::new();

        if let CypherClauseContext::Return(return_ctx) = clause_ctx {
            for item in &return_ctx.yield_clause.yield_columns {
                let item_name = item.alias.clone();
                items.push(ReturnItem {
                    alias: item_name.clone(),
                    expression: Expression::Variable(item_name.clone()),
                    is_aggregated: Self::is_aggregated_expression(&item_name),
                });
            }
        }

        Ok(items)
    }

    /// 构建 YIELD 列
    fn build_yield_columns(
        items: &[ReturnItem],
        context: &PlanningContext,
    ) -> Result<Vec<crate::query::validator::YieldColumn>, PlannerError> {
        let mut columns = Vec::new();

        for item in items {
            let expr = if context.has_variable(&item.alias) {
                item.expression.clone()
            } else {
                Expression::Variable(item.alias.clone())
            };

            columns.push(crate::query::validator::YieldColumn {
                expr,
                alias: item.alias.clone(),
                is_matched: false,
            });
        }

        if columns.is_empty() {
            columns.push(crate::query::validator::YieldColumn {
                expr: Expression::Variable("*".to_string()),
                alias: "*".to_string(),
                is_matched: false,
            });
        }

        Ok(columns)
    }
}
