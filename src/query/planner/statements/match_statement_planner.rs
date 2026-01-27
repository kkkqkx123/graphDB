//! 统一 MATCH 语句规划器
//!
//! 实现 StatementPlanner 接口，处理完整的 MATCH 查询规划。
//! 整合了以下功能：
//! - 节点和边模式匹配
//! - WHERE 条件过滤
//! - RETURN 投影
//! - ORDER BY 排序
//! - LIMIT/SKIP 分页

use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::{LimitNode, PlanNodeEnum, ProjectNode, ScanVerticesNode, SortNode};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::planner::statements::statement_planner::StatementPlanner;
use crate::query::validator::OrderByItem;
use crate::query::planner::statements::match_planner::PaginationInfo;
use crate::query::validator::YieldColumn;
use crate::query::validator::structs::CypherClauseKind;

/// MATCH 语句规划器
///
/// 负责将 MATCH 查询转换为可执行的执行计划。
/// 实现 StatementPlanner 接口，提供统一的规划入口。
#[derive(Debug)]
pub struct MatchStatementPlanner {
    config: MatchPlannerConfig,
}

#[derive(Debug, Clone, Default)]
pub struct MatchPlannerConfig {
    pub default_limit: usize,
    pub max_limit: usize,
    pub enable_index_optimization: bool,
}

impl MatchStatementPlanner {
    pub fn new() -> Self {
        Self {
            config: MatchPlannerConfig::default(),
        }
    }

    pub fn with_config(config: MatchPlannerConfig) -> Self {
        Self { config }
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "MATCH"
    }

    fn parse_tag_from_pattern(pattern: &str) -> Option<String> {
        let colon_pos = pattern.find(':')?;
        let closing_paren_pos = pattern.find(')')?;
        if colon_pos < closing_paren_pos {
            let tag_part = &pattern[colon_pos + 1..closing_paren_pos];
            Some(tag_part.to_string())
        } else {
            None
        }
    }
}

impl Planner for MatchStatementPlanner {
    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }

    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let stmt = ast_ctx.sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;

        let space_id = ast_ctx.space().space_id.unwrap_or(1) as i32;
        self.plan_match_pattern(stmt, space_id)
    }

    fn transform_with_full_context(
        &mut self,
        query_context: &mut QueryContext,
        ast_ctx: &AstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        let stmt = ast_ctx.sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;

        let space_id = ast_ctx.space().space_id.unwrap_or(1) as i32;
        let mut current_plan = self.plan_match_pattern(stmt, space_id)?;

        if ast_ctx.query_type() == crate::query::context::ast::QueryType::ReadQuery {
            if let Some(where_condition) = self.extract_where_condition(stmt)? {
                current_plan = self.plan_filter(current_plan, where_condition, space_id)?;
            }

            if let Some(return_columns) = self.extract_return_columns(stmt)? {
                current_plan = self.plan_project(current_plan, return_columns, space_id)?;
            }

            if let Some(order_by) = self.extract_order_by(stmt)? {
                current_plan = self.plan_sort(current_plan, order_by, space_id)?;
            }

            if let Some(pagination) = self.extract_pagination(stmt)? {
                current_plan = self.plan_limit(current_plan, pagination)?;
            }
        }

        let mut plan = ExecutionPlan::new(current_plan.root().clone());
        self.set_plan_id(&mut plan);
        Ok(plan)
    }

    fn name(&self) -> &'static str {
        "MatchStatementPlanner"
    }
}

impl StatementPlanner for MatchStatementPlanner {
    fn statement_type(&self) -> &'static str {
        "MATCH"
    }

    fn supported_clause_kinds(&self) -> Vec<CypherClauseKind> {
        vec![
            CypherClauseKind::Match,
            CypherClauseKind::Where,
            CypherClauseKind::Return,
            CypherClauseKind::OrderBy,
            CypherClauseKind::Pagination,
        ]
    }

    fn extract_clauses(&self, ast_ctx: &AstContext) -> Vec<CypherClauseKind> {
        let mut clauses = Vec::new();
        clauses.push(CypherClauseKind::Match);

        let stmt = ast_ctx.sentence();
        if let Some(crate::query::parser::ast::Stmt::Match(match_stmt)) = stmt {
            if match_stmt.where_clause.is_some() {
                clauses.push(CypherClauseKind::Where);
            }
            if match_stmt.return_clause.is_some() {
                clauses.push(CypherClauseKind::Return);
            }
            if match_stmt.order_by.is_some() {
                clauses.push(CypherClauseKind::OrderBy);
            }
            if match_stmt.skip.is_some() || match_stmt.limit.is_some() {
                clauses.push(CypherClauseKind::Pagination);
            }
        }
        clauses
    }

    fn make_statement_planner() -> Box<dyn StatementPlanner>
    where
        Self: Sized,
    {
        Box::new(Self::new())
    }

    fn create_initial_plan(&self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let stmt = ast_ctx.sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;
        let space_id = ast_ctx.space().space_id.unwrap_or(1) as i32;
        self.plan_match_pattern(stmt, space_id)
    }

    fn create_default_plan(&self, ast_ctx: &AstContext) -> Result<ExecutionPlan, PlannerError> {
        let mut planner = MatchStatementPlanner::new();
        planner.transform_with_full_context(&mut QueryContext::new(), ast_ctx)
    }
}

impl MatchStatementPlanner {
    fn plan_match_pattern(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(_match_stmt) => {
                let start_node = self.plan_node_pattern(space_id)?;
                Ok(SubPlan::from_root(start_node))
            }
            _ => Err(PlannerError::InvalidOperation(
                "Expected MATCH statement".to_string()
            ))
        }
    }

    fn plan_node_pattern(&self, space_id: i32) -> Result<PlanNodeEnum, PlannerError> {
        let scan_node = ScanVerticesNode::new(space_id);
        Ok(scan_node.into_enum())
    }

    fn plan_edge_pattern(
        &self,
        _edge: &str,
        _space_id: i32,
    ) -> Result<PlanNodeEnum, PlannerError> {
        let expand_node = crate::query::planner::plan::core::nodes::ExpandAllNode::new(
            _space_id,
            vec![],
            "both",
        );
        Ok(expand_node.into_enum())
    }

    fn plan_filter(
        &self,
        input_plan: SubPlan,
        condition: Expression,
        _space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let filter_node = FilterNode::new(input_node.clone(), condition)?;
        Ok(SubPlan::new(Some(filter_node.into_enum()), input_plan.tail))
    }

    fn plan_project(
        &self,
        input_plan: SubPlan,
        columns: Vec<YieldColumn>,
        _space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let project_node = ProjectNode::new(input_node.clone(), columns)?;
        Ok(SubPlan::new(Some(project_node.into_enum()), input_plan.tail))
    }

    fn plan_sort(
        &self,
        input_plan: SubPlan,
        order_by: Vec<OrderByItem>,
        _space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let sort_items: Vec<String> = order_by
            .into_iter()
            .map(|item| self.expression_to_string(&item.expression))
            .collect();

        let sort_node = SortNode::new(input_node.clone(), sort_items)?;
        Ok(SubPlan::new(Some(sort_node.into_enum()), input_plan.tail))
    }

    fn expression_to_string(&self, expr: &Expression) -> String {
        match expr {
            Expression::Variable(name) => name.clone(),
            Expression::Property { object, property } => {
                let obj_str = self.expression_to_string(object);
                format!("{}.{}", obj_str, property)
            }
            Expression::Function { name, args } => {
                let args_str: Vec<String> = args
                    .iter()
                    .map(|arg| self.expression_to_string(arg))
                    .collect();
                format!("{}({})", name, args_str.join(", "))
            }
            Expression::Literal(value) => format!("{}", value),
            Expression::Binary { left, op, right } => {
                let left_str = self.expression_to_string(left);
                let right_str = self.expression_to_string(right);
                let op_str = match op {
                    crate::core::BinaryOperator::Add => "+",
                    crate::core::BinaryOperator::Subtract => "-",
                    crate::core::BinaryOperator::Multiply => "*",
                    crate::core::BinaryOperator::Divide => "/",
                    crate::core::BinaryOperator::Modulo => "%",
                    crate::core::BinaryOperator::Equal => "=",
                    crate::core::BinaryOperator::NotEqual => "!=",
                    crate::core::BinaryOperator::LessThan => "<",
                    crate::core::BinaryOperator::LessThanOrEqual => "<=",
                    crate::core::BinaryOperator::GreaterThan => ">",
                    crate::core::BinaryOperator::GreaterThanOrEqual => ">=",
                    crate::core::BinaryOperator::And => "AND",
                    crate::core::BinaryOperator::Or => "OR",
                    crate::core::BinaryOperator::Xor => "XOR",
                    crate::core::BinaryOperator::StringConcat => "+",
                    crate::core::BinaryOperator::Contains => "CONTAINS",
                    crate::core::BinaryOperator::StartsWith => "STARTS WITH",
                    crate::core::BinaryOperator::EndsWith => "ENDS WITH",
                    crate::core::BinaryOperator::In => "IN",
                    crate::core::BinaryOperator::NotIn => "NOT IN",
                    crate::core::BinaryOperator::Like => "LIKE",
                    crate::core::BinaryOperator::Exponent => "^",
                    _ => "?",
                };
                format!("{} {} {}", left_str, op_str, right_str)
            }
            Expression::Unary { op, operand } => {
                let operand_str = self.expression_to_string(operand);
                let op_str = match op {
                    crate::core::UnaryOperator::Not => "NOT ",
                    crate::core::UnaryOperator::IsNull => "IS NULL",
                    crate::core::UnaryOperator::IsNotNull => "IS NOT NULL",
                    crate::core::UnaryOperator::IsEmpty => "IS EMPTY",
                    crate::core::UnaryOperator::IsNotEmpty => "IS NOT EMPTY",
                    crate::core::UnaryOperator::Plus => "+",
                    crate::core::UnaryOperator::Minus => "-",
                };
                format!("{}{}", op_str, operand_str)
            }
            Expression::Subscript { collection, index } => {
                let coll_str = self.expression_to_string(collection);
                let idx_str = self.expression_to_string(index);
                format!("{}[{}]", coll_str, idx_str)
            }
            Expression::Case { .. } => "CASE".to_string(),
            Expression::TypeCast { expression, target_type } => {
                let expr_str = self.expression_to_string(expression);
                format!("{} AS {:?}", expr_str, target_type)
            }
            Expression::List(items) => {
                let items_str: Vec<String> = items
                    .iter()
                    .map(|item| self.expression_to_string(item))
                    .collect();
                format!("[{}]", items_str.join(", "))
            }
            Expression::Map(pairs) => {
                let pairs_str: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.expression_to_string(v)))
                    .collect();
                format!("{{{}}}", pairs_str.join(", "))
            }
            Expression::Range { collection, start, end } => {
                let coll_str = self.expression_to_string(collection);
                let start_str = start.as_ref().map(|s| self.expression_to_string(s)).unwrap_or_default();
                let end_str = end.as_ref().map(|e| self.expression_to_string(e)).unwrap_or_default();
                if start_str.is_empty() {
                    format!("RANGE({}, {})", coll_str, end_str)
                } else {
                    format!("RANGE({}, {}, {})", coll_str, start_str, end_str)
                }
            }
            Expression::Path(_) => "<path>".to_string(),
            Expression::Label(_) => "<label>".to_string(),
            Expression::ListComprehension { .. } => "<list_comprehension>".to_string(),
            Expression::Aggregate { func, .. } => format!("<aggregate:{:?}>", func),
            _ => "<?>".to_string(),
        }
    }

    fn plan_limit(
        &self,
        input_plan: SubPlan,
        pagination: PaginationInfo,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let limit_node = LimitNode::new(input_node.clone(), pagination.skip as i64, pagination.limit as i64)?;
        let limit_node_enum = limit_node.into_enum();
        Ok(SubPlan::new(Some(limit_node_enum), input_plan.tail))
    }

    fn extract_where_condition(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<Expression>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                Ok(match_stmt.where_clause.clone())
            }
            _ => Ok(None),
        }
    }

    fn extract_return_columns(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<Vec<YieldColumn>>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                if let Some(return_clause) = &match_stmt.return_clause {
                    let mut columns = Vec::new();
                    for item in &return_clause.items {
                        match item {
                            crate::query::parser::ast::stmt::ReturnItem::Expression { expression, alias } => {
                                columns.push(YieldColumn {
                                    expression: expression.clone(),
                                    alias: alias.clone().unwrap_or_default(),
                                    is_matched: false,
                                });
                            }
                            crate::query::parser::ast::stmt::ReturnItem::All => {
                                columns.push(YieldColumn {
                                    expression: crate::core::Expression::Variable("*".to_string()),
                                    alias: "*".to_string(),
                                    is_matched: false,
                                });
                            }
                        }
                    }
                    if columns.is_empty() {
                        columns.push(YieldColumn {
                            expression: crate::core::Expression::Variable("*".to_string()),
                            alias: "*".to_string(),
                            is_matched: false,
                        });
                    }
                    Ok(Some(columns))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn extract_order_by(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<Vec<OrderByItem>>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                if let Some(order_by_clause) = &match_stmt.order_by {
                    let items = order_by_clause.items.iter().map(|item| {
                        OrderByItem {
                            expression: item.expression.clone(),
                            desc: item.direction == crate::query::parser::ast::types::OrderDirection::Desc,
                        }
                    }).collect();
                    Ok(Some(items))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn extract_pagination(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<PaginationInfo>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                let skip = match_stmt.skip.unwrap_or(0);
                let limit = match_stmt.limit.unwrap_or(self.config.default_limit);
                Ok(Some(PaginationInfo { skip, limit }))
            }
            _ => Ok(None),
        }
    }

    fn set_plan_id(&self, plan: &mut ExecutionPlan) {
        let uuid = uuid::Uuid::new_v4();
        let uuid_bytes = uuid.as_bytes();
        let id = i64::from_ne_bytes([
            uuid_bytes[0],
            uuid_bytes[1],
            uuid_bytes[2],
            uuid_bytes[3],
            uuid_bytes[4],
            uuid_bytes[5],
            uuid_bytes[6],
            uuid_bytes[7],
        ]);
        plan.set_id(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::ast::AstContext;

    #[test]
    fn test_match_statement_planner_creation() {
        let planner = MatchStatementPlanner::new();
        assert_eq!(planner.statement_type(), "MATCH");
        assert_eq!(planner.name(), "MatchStatementPlanner");
    }

    #[test]
    fn test_supported_clauses() {
        let planner = MatchStatementPlanner::new();
        let clauses = planner.supported_clause_kinds();
        assert!(clauses.contains(&CypherClauseKind::Match));
        assert!(clauses.contains(&CypherClauseKind::Where));
        assert!(clauses.contains(&CypherClauseKind::Return));
        assert!(clauses.contains(&CypherClauseKind::OrderBy));
        assert!(clauses.contains(&CypherClauseKind::Pagination));
    }

    #[test]
    fn test_extract_clauses_simple() {
        let planner = MatchStatementPlanner::new();
        let ast_ctx = AstContext::from_strings("MATCH", "MATCH (n)");
        let clauses = planner.extract_clauses(&ast_ctx);
        assert_eq!(clauses.len(), 1);
        assert!(clauses.contains(&CypherClauseKind::Match));
    }

    #[test]
    fn test_parse_tag_from_pattern() {
        assert_eq!(MatchStatementPlanner::parse_tag_from_pattern("(n:Person)"), Some("Person".to_string()));
        assert_eq!(MatchStatementPlanner::parse_tag_from_pattern("(n:Tag1:Tag2)"), Some("Tag1:Tag2".to_string()));
        assert_eq!(MatchStatementPlanner::parse_tag_from_pattern("(n)"), None);
    }
}
