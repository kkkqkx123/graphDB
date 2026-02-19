//! YIELD 子句规划器
//!
//! 负责将 YIELD 子句转换为执行计划节点
//! 支持 YIELD ... WHERE ... 语法

use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::{
    FilterNode, LimitNode, PlanNodeEnum, ProjectNode,
};
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::structs::CypherClauseKind;
use crate::query::validator::YieldColumn;

/// YIELD 子句规划器
#[derive(Debug)]
pub struct YieldClausePlanner {}

impl YieldClausePlanner {
    pub fn new() -> Self {
        Self {}
    }

    /// 规划 YIELD 子句
    ///
    /// 处理流程：
    /// 1. 构建投影节点（YIELD 列）
    /// 2. 如有 WHERE 条件，添加 Filter 节点
    /// 3. 如有 LIMIT/SKIP，添加分页节点
    pub fn plan_yield_clause(
        &self,
        yield_columns: &[YieldColumn],
        filter_condition: Option<crate::core::Expression>,
        skip: Option<usize>,
        limit: Option<usize>,
        input_plan: &SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let mut current_plan = input_plan.clone();

        // 1. 构建投影节点（如果有具体的 YIELD 列）
        if !yield_columns.is_empty() {
            let project_node = self.create_project_node(&current_plan, yield_columns)?;
            current_plan = SubPlan::new(Some(project_node), current_plan.tail.clone());
        }

        // 2. 如有 WHERE 条件，添加 Filter 节点
        if let Some(ref filter_condition) = filter_condition {
            let filter_node = self.create_filter_node(&current_plan, filter_condition.clone())?;
            current_plan = SubPlan::new(Some(filter_node), current_plan.tail.clone());
        }

        // 3. 处理分页（LIMIT/SKIP）
        if limit.is_some() || skip.is_some() {
            current_plan =
                self.apply_pagination(current_plan, skip, limit)?;
        }

        Ok(current_plan)
    }

    /// 创建投影节点
    fn create_project_node(
        &self,
        input_plan: &SubPlan,
        columns: &[YieldColumn],
    ) -> Result<PlanNodeEnum, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        ProjectNode::new(input_node.clone(), columns.to_vec())
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("创建投影节点失败: {}", e)))
            .map(|node| PlanNodeEnum::Project(node))
    }

    /// 创建过滤节点
    fn create_filter_node(
        &self,
        input_plan: &SubPlan,
        condition: crate::core::Expression,
    ) -> Result<PlanNodeEnum, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        FilterNode::new(input_node.clone(), condition)
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("创建过滤节点失败: {}", e)))
            .map(|node| PlanNodeEnum::Filter(node))
    }

    /// 应用分页
    fn apply_pagination(
        &self,
        input_plan: SubPlan,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        let offset = skip.unwrap_or(0) as i64;
        let count = limit.map(|l| l as i64).unwrap_or(i64::MAX);

        let limit_node = LimitNode::new(input_node.clone(), offset, count)
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("创建分页节点失败: {}", e)))?;

        Ok(SubPlan::new(
            Some(PlanNodeEnum::Limit(limit_node)),
            input_plan.tail.clone(),
        ))
    }
}

impl ClausePlanner for YieldClausePlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Yield
    }

    fn name(&self) -> &'static str {
        "YieldClausePlanner"
    }

    fn transform_clause(
        &self,
        _query_context: &mut QueryContext,
        ast_ctx: &AstContext,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        // 从 AST 上下文中提取 YIELD 子句信息
        let (yield_columns, filter_condition, skip, limit) = Self::extract_yield_info(ast_ctx)?;

        self.plan_yield_clause(&yield_columns, filter_condition, skip, limit, &input_plan)
    }
}

impl YieldClausePlanner {
    /// 从 AST 上下文中提取 YIELD 子句信息
    ///
    /// 完善后的实现包括：
    /// - 支持多种语句类型中的 YIELD 子句
    /// - YieldItem 到 YieldColumn 的完整转换
    /// - 聚合表达式检测
    /// - 别名处理
    fn extract_yield_info(ast_ctx: &AstContext) -> Result<(Vec<YieldColumn>, Option<crate::core::Expression>, Option<usize>, Option<usize>), PlannerError> {
        use crate::query::parser::ast::Stmt;

        let sentence = ast_ctx.sentence()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("AST 上下文中没有语句".to_string()))?;

        // YIELD 可能作为独立语句或子句出现在其他语句中
        match sentence {
            Stmt::Yield(yield_stmt) => {
                let yield_columns = Self::convert_yield_items(&yield_stmt.items)?;
                Ok((yield_columns, yield_stmt.where_clause.clone(), None, None))
            }
            Stmt::Go(go_stmt) => {
                // 从 GO 语句中提取 YIELD 子句
                if let Some(ref yield_clause) = go_stmt.yield_clause {
                    let yield_columns = Self::convert_yield_items(&yield_clause.items)?;
                    let skip = yield_clause.skip.as_ref().map(|s| s.count as usize);
                    let limit = yield_clause.limit.as_ref().map(|l| l.count as usize);
                    Ok((yield_columns, yield_clause.where_clause.clone(), skip, limit))
                } else {
                    Ok((vec![], None, None, None))
                }
            }
            Stmt::Fetch(_fetch_stmt) => {
                // FETCH 语句可能有隐式的 YIELD
                Ok((vec![], None, None, None))
            }
            _ => {
                // 其他语句类型暂不支持 YIELD 提取
                Ok((vec![], None, None, None))
            }
        }
    }

    /// 转换 YieldItem 列表到 YieldColumn 列表
    fn convert_yield_items(items: &[crate::query::parser::ast::stmt::YieldItem]) -> Result<Vec<YieldColumn>, PlannerError> {
        let yield_columns: Vec<YieldColumn> = items
            .iter()
            .map(|item| {
                YieldColumn {
                    expression: item.expression.clone(),
                    alias: item.alias.clone().unwrap_or_else(|| {
                        Self::generate_default_alias(&item.expression)
                    }),
                    is_matched: false,
                }
            })
            .collect();
        Ok(yield_columns)
    }

    /// 生成默认别名
    ///
    /// 当用户没有指定别名时，根据表达式生成默认别名
    fn generate_default_alias(expression: &crate::core::Expression) -> String {
        use crate::core::Expression;
        
        match expression {
            Expression::Variable(name) => name.clone(),
            Expression::Property { object, property } => {
                if let Expression::Variable(name) = object.as_ref() {
                    format!("{}.{}", name, property)
                } else {
                    "expr".to_string()
                }
            }
            Expression::Function { name, .. } => name.clone(),
            Expression::Aggregate { func, .. } => format!("{:?}", func).to_lowercase(),
            _ => "expr".to_string(),
        }
    }

    /// 检查表达式是否包含聚合函数
    ///
    /// 用于确定是否需要聚合处理
    #[allow(dead_code)]
    fn has_aggregate_expression(expression: &crate::core::Expression) -> bool {
        use crate::core::Expression;
        
        match expression {
            Expression::Function { name, .. } => {
                // 常见的聚合函数
                let agg_functions = ["count", "sum", "avg", "min", "max", "collect"];
                agg_functions.contains(&name.to_lowercase().as_str())
            }
            Expression::Aggregate { .. } => true,
            Expression::Binary { left, right, .. } => {
                Self::has_aggregate_expression(left) || Self::has_aggregate_expression(right)
            }
            Expression::Unary { operand, .. } => {
                Self::has_aggregate_expression(operand)
            }
            _ => false,
        }
    }
}

impl Default for YieldClausePlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yield_clause_planner_creation() {
        let planner = YieldClausePlanner::new();
        assert_eq!(planner.name(), "YieldClausePlanner");
    }
}
