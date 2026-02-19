//! WITH 子句规划器
//!
//! 负责规划 WITH 子句的执行，是数据流的转换点。
//!
//! WITH 子句的功能：
//! 1. 投影：选择并可能重命名输出列
//! 2. 过滤：通过 WHERE 子句过滤结果
//! 3. 排序：通过 ORDER BY 对结果排序
//! 4. 分页：通过 SKIP/LIMIT 限制结果数量
//! 5. 作用域重置：只保留输出的变量，其他变量不可见

use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::{FilterNode, LimitNode, PlanNodeEnum, ProjectNode};
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::structs::{
    CypherClauseKind, OrderByClauseContext, PaginationContext, WithClauseContext, YieldColumn,
};

/// WITH 子句规划器
#[derive(Debug)]
pub struct WithClausePlanner {}

impl WithClausePlanner {
    /// 创建新的 WITH 子句规划器
    pub fn new() -> Self {
        Self {}
    }

    /// 规划 WITH 子句
    ///
    /// # 参数
    /// - `with_ctx`: WITH 子句上下文，包含投影列、WHERE条件、排序、分页等信息
    /// - `input_plan`: 输入计划
    ///
    /// # 返回
    /// - 成功：生成的子计划
    /// - 失败：规划错误
    pub fn plan_with_clause(
        &self,
        with_ctx: &WithClauseContext,
        input_plan: &SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let mut current_plan = input_plan.clone();

        // 1. 构建投影节点（如果有具体的输出列）
        if !with_ctx.yield_clause.yield_columns.is_empty() {
            let project_node = self.create_project_node(
                &current_plan,
                &with_ctx.yield_clause.yield_columns,
            )?;
            current_plan = SubPlan::new(Some(project_node), current_plan.tail.clone());
        }

        // 2. 处理 WHERE 条件过滤
        if let Some(ref where_ctx) = with_ctx.where_clause {
            if let Some(ref filter) = where_ctx.filter {
                let filter_node = self.create_filter_node(&current_plan, filter.clone())?;
                current_plan = SubPlan::new(Some(filter_node), current_plan.tail.clone());
            }
        }

        // 3. 处理 ORDER BY 排序
        if let Some(ref order_by_ctx) = with_ctx.order_by {
            current_plan = self.apply_order_by(current_plan, order_by_ctx)?;
        }

        // 4. 处理分页（SKIP/LIMIT）
        if let Some(ref pagination) = with_ctx.pagination {
            current_plan = self.apply_pagination(current_plan, pagination)?;
        }

        // 5. 处理 DISTINCT（去重）
        if with_ctx.distinct {
            current_plan = self.apply_distinct(current_plan)?;
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
        condition: Expression,
    ) -> Result<PlanNodeEnum, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        FilterNode::new(input_node.clone(), condition)
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("创建过滤节点失败: {}", e)))
            .map(|node| PlanNodeEnum::Filter(node))
    }

    /// 应用 ORDER BY 排序
    ///
    /// TODO: 当前实现存在架构问题：
    /// - OrderByClauseContext 提供的是 Vec<(usize, OrderDirection)>（列索引+方向）
    /// - SortNode 期望的是 Vec<String>（列名）
    /// - 这种不匹配需要在架构层面解决，可能通过：
    ///   1. 修改 SortNode 支持索引+方向的排序项
    ///   2. 修改 OrderByClauseContext 提供列名而非索引
    ///   3. 在规划阶段维护列名到索引的映射
    fn apply_order_by(
        &self,
        input_plan: SubPlan,
        order_by_ctx: &OrderByClauseContext,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        // 获取输入节点的列名
        let col_names = input_node.col_names();

        // 将索引排序因子转换为排序字段名
        // 注意：这里假设索引对应于列名列表中的位置
        // 如果索引超出范围，使用占位符名称
        let sort_items: Vec<String> = order_by_ctx
            .indexed_order_factors
            .iter()
            .map(|(idx, _dir)| {
                col_names.get(*idx)
                    .cloned()
                    .unwrap_or_else(|| format!("col_{}", idx))
            })
            .collect();

        if sort_items.is_empty() {
            // 如果没有有效的排序项，直接返回输入计划
            return Ok(input_plan);
        }

        // 创建排序节点
        let sort_node = crate::query::planner::plan::core::nodes::SortNode::new(
            input_node.clone(),
            sort_items,
        )
        .map_err(|e| PlannerError::PlanGenerationFailed(format!("创建排序节点失败: {}", e)))?;

        Ok(SubPlan::new(
            Some(PlanNodeEnum::Sort(sort_node)),
            input_plan.tail.clone(),
        ))
    }

    /// 应用分页
    fn apply_pagination(
        &self,
        input_plan: SubPlan,
        pagination: &PaginationContext,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        let limit_node = LimitNode::new(input_node.clone(), pagination.skip, pagination.limit)
            .map_err(|e| PlannerError::PlanGenerationFailed(format!("创建分页节点失败: {}", e)))?;

        Ok(SubPlan::new(
            Some(PlanNodeEnum::Limit(limit_node)),
            input_plan.tail.clone(),
        ))
    }

    /// 应用 DISTINCT（去重）
    fn apply_distinct(&self, input_plan: SubPlan) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        // 创建去重节点（使用 AggregateNode 的简化版本）
        let dedup_node = crate::query::planner::plan::core::nodes::DedupNode::new(
            input_node.clone(),
        )
        .map_err(|e| PlannerError::PlanGenerationFailed(format!("创建去重节点失败: {}", e)))?;

        Ok(SubPlan::new(
            Some(PlanNodeEnum::Dedup(dedup_node)),
            input_plan.tail.clone(),
        ))
    }
}

impl ClausePlanner for WithClausePlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::With
    }

    fn name(&self) -> &'static str {
        "WithClausePlanner"
    }

    fn transform_clause(
        &self,
        _query_context: &mut QueryContext,
        ast_ctx: &AstContext,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        // 从 AST 上下文中提取 WITH 子句信息
        let with_ctx = Self::extract_with_context(ast_ctx)?;
        self.plan_with_clause(&with_ctx, &input_plan)
    }
}

impl WithClausePlanner {
    /// 从 AST 上下文中提取 WITH 子句上下文
    ///
    /// 完善后的实现包括：
    /// - 从 Stmt::With 提取完整的 WITH 子句信息
    /// - 构建 YieldClauseContext
    /// - 处理 ORDER BY 和分页
    fn extract_with_context(ast_ctx: &AstContext) -> Result<WithClauseContext, PlannerError> {
        use crate::query::parser::ast::Stmt;
        use crate::query::validator::structs::{YieldClauseContext, YieldColumn, OrderByClauseContext, PaginationContext};

        let sentence = ast_ctx.sentence()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("AST 上下文中没有语句".to_string()))?;

        let with_stmt = match sentence {
            Stmt::With(w) => w,
            _ => {
                return Err(PlannerError::PlanGenerationFailed(
                    "期望 WITH 语句，但得到了其他类型的语句".to_string()
                ));
            }
        };

        // 转换 ReturnItem 到 YieldColumn
        let mut yield_columns = Vec::new();
        let mut has_agg = false;
        
        for item in &with_stmt.items {
            match item {
                crate::query::parser::ast::stmt::ReturnItem::All => {
                    // WITH * 表示保留所有列
                    // TODO: 需要从输入计划获取所有列
                }
                crate::query::parser::ast::stmt::ReturnItem::Expression { expression, alias } => {
                    yield_columns.push(YieldColumn {
                        expression: expression.clone(),
                        alias: alias.clone().unwrap_or_else(|| {
                            Self::generate_default_alias(expression)
                        }),
                        is_matched: false,
                    });
                    
                    if Self::has_aggregate_expression(expression) {
                        has_agg = true;
                    }
                }
            }
        }

        // 构建 ORDER BY 上下文
        let order_by = with_stmt.order_by.as_ref().map(|order| {
            OrderByClauseContext {
                indexed_order_factors: order.items.iter().enumerate().map(|(idx, item)| {
                    (idx, item.direction.clone())
                }).collect(),
            }
        });

        // 构建分页上下文
        let pagination = if with_stmt.skip.is_some() || with_stmt.limit.is_some() {
            Some(PaginationContext {
                skip: with_stmt.skip.unwrap_or(0) as i64,
                limit: with_stmt.limit.unwrap_or(0) as i64,
            })
        } else {
            None
        };

        // 构建 YieldClauseContext
        let yield_clause = YieldClauseContext {
            yield_columns,
            aliases_available: std::collections::HashMap::new(), // TODO: 从输入计划获取可用的别名
            aliases_generated: std::collections::HashMap::new(), // TODO: 从 WITH 子句收集生成的别名
            distinct: with_stmt.distinct,
            has_agg,
            group_keys: vec![], // TODO: 如果有聚合，需要确定分组键
            group_items: vec![], // TODO: 如果有聚合，需要收集聚合项
            need_gen_project: has_agg,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
            query_parts: vec![],
            errors: vec![],
            filter_condition: with_stmt.where_clause.clone(),
            skip: with_stmt.skip,
            limit: with_stmt.limit,
        };

        Ok(WithClauseContext {
            yield_clause,
            aliases_available: std::collections::HashMap::new(), // TODO: 从输入计划获取
            aliases_generated: std::collections::HashMap::new(), // TODO: 从 WITH 子句收集
            where_clause: with_stmt.where_clause.clone().map(|condition| {
                crate::query::validator::structs::WhereClauseContext {
                    filter: Some(condition),
                    aliases_available: std::collections::HashMap::new(),
                    aliases_generated: std::collections::HashMap::new(),
                    paths: vec![],
                    query_parts: vec![],
                    errors: vec![],
                }
            }),
            pagination,
            order_by,
            distinct: with_stmt.distinct,
            query_parts: vec![],
            errors: vec![],
        })
    }

    /// 生成默认别名
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
    fn has_aggregate_expression(expression: &crate::core::Expression) -> bool {
        use crate::core::Expression;
        
        match expression {
            Expression::Function { name, .. } => {
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

impl Default for WithClausePlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_clause_planner_creation() {
        let planner = WithClausePlanner::new();
        assert_eq!(planner.name(), "WithClausePlanner");
        assert_eq!(planner.clause_kind(), CypherClauseKind::With);
    }
}
