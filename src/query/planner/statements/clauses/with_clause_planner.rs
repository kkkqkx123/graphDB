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
use crate::core::YieldColumn;
use crate::query::validator::structs::{
    AliasType, CypherClauseKind, OrderByClauseContext, PaginationContext, WithClauseContext,
};
use crate::query::visitor::ExtractGroupSuiteVisitor;
use std::collections::HashMap;

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
    /// 将 OrderByClauseContext 中的索引排序因子转换为排序字段名和方向
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

        // 将索引排序因子转换为排序项（包含列名和方向）
        // 注意：这里假设索引对应于列名列表中的位置
        // 如果索引超出范围，使用占位符名称
        let sort_items: Vec<crate::query::planner::plan::core::nodes::SortItem> = order_by_ctx
            .indexed_order_factors
            .iter()
            .map(|(idx, dir)| {
                let column = col_names.get(*idx)
                    .cloned()
                    .unwrap_or_else(|| format!("col_{}", idx));
                crate::query::planner::plan::core::nodes::SortItem::new(column.clone(), dir.clone())
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
    /// - 收集别名信息
    /// - 处理聚合表达式和分组键
    fn extract_with_context(ast_ctx: &AstContext) -> Result<WithClauseContext, PlannerError> {
        use crate::query::parser::ast::Stmt;
        use crate::core::YieldColumn;
        use crate::query::validator::structs::{YieldClauseContext, OrderByClauseContext, PaginationContext};

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
        let mut aliases_generated = HashMap::new();

        for item in &with_stmt.items {
            match item {
                crate::query::parser::ast::stmt::ReturnItem::All => {
                    // WITH * 表示保留所有列
                    // 使用通配符表达式表示保留所有列
                    yield_columns.push(YieldColumn {
                        expression: Expression::Variable("*".to_string()),
                        alias: "*".to_string(),
                        is_matched: false,
                    });
                }
                crate::query::parser::ast::stmt::ReturnItem::Expression { expression, alias } => {
                    let col_alias = alias.clone().unwrap_or_else(|| {
                        Self::generate_default_alias(expression)
                    });

                    yield_columns.push(YieldColumn {
                        expression: expression.clone(),
                        alias: col_alias.clone(),
                        is_matched: false,
                    });

                    // 收集生成的别名
                    if !col_alias.is_empty() && col_alias != "*" {
                        let alias_type = Self::deduce_alias_type(expression);
                        aliases_generated.insert(col_alias, alias_type);
                    }

                    if Self::has_aggregate_expression(expression) {
                        has_agg = true;
                    }
                }
            }
        }

        // 提取分组键和聚合项
        let (group_keys, group_items) = if has_agg {
            Self::extract_group_info(&yield_columns)
        } else {
            (vec![], vec![])
        };

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
            yield_columns: yield_columns.clone(),
            aliases_available: HashMap::new(), // 从输入计划获取的别名在规划阶段填充
            aliases_generated: aliases_generated.clone(),
            distinct: with_stmt.distinct,
            has_agg,
            group_keys: group_keys.clone(),
            group_items: group_items.clone(),
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
            aliases_available: HashMap::new(), // 从输入计划获取的别名在规划阶段填充
            aliases_generated,
            where_clause: with_stmt.where_clause.clone().map(|condition| {
                crate::query::validator::structs::WhereClauseContext {
                    filter: Some(condition),
                    aliases_available: HashMap::new(),
                    aliases_generated: HashMap::new(),
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

    /// 提取分组信息
    ///
    /// 从 YieldColumn 列表中提取分组键和聚合项
    fn extract_group_info(yield_columns: &[YieldColumn]) -> (Vec<Expression>, Vec<Expression>) {
        let mut group_keys = Vec::new();
        let mut group_items = Vec::new();
        let mut visitor = ExtractGroupSuiteVisitor::new();

        for column in yield_columns {
            if let Ok(suite) = visitor.extract(&column.expression) {
                // 非聚合表达式作为分组键
                if !suite.group_keys.is_empty() {
                    group_keys.extend(suite.group_keys);
                }
                // 聚合表达式作为分组项
                if !suite.aggregates.is_empty() {
                    group_items.extend(suite.aggregates);
                }
            }
        }

        // 去重
        group_keys.dedup_by(|a, b| a == b);
        group_items.dedup_by(|a, b| a == b);

        (group_keys, group_items)
    }

    /// 推断别名类型
    ///
    /// 根据表达式推断别名类型
    fn deduce_alias_type(expression: &Expression) -> AliasType {
        use crate::core::Expression;

        match expression {
            Expression::Variable(_) => AliasType::Variable,
            Expression::Property { object, .. } => {
                if let Expression::Variable(var_name) = object.as_ref() {
                    // 根据变量名推断类型（简化实现）
                    if var_name.starts_with('e') || var_name.starts_with('E') {
                        AliasType::Edge
                    } else if var_name.starts_with('v') || var_name.starts_with('V') {
                        AliasType::Node
                    } else {
                        AliasType::Variable
                    }
                } else {
                    AliasType::Variable
                }
            }
            Expression::Function { name, .. } => {
                let name_lower = name.to_lowercase();
                match name_lower.as_str() {
                    "id" | "src" | "dst" => AliasType::Variable,
                    "nodes" | "relationships" => AliasType::Path,
                    _ => AliasType::Variable,
                }
            }
            Expression::Aggregate { .. } => AliasType::Variable,
            _ => AliasType::Variable,
        }
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
