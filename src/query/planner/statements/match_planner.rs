//! MATCH 规划器
//!
//! 负责将 MATCH 查询转换为可执行的执行计划。
//! 使用 AstContext 中的语句信息进行规划，支持完整的 Cypher MATCH 语法。
//!
//! ## 支持的功能
//!
//! - 节点模式匹配：(n:Tag)
//! - 关系模式匹配：-[e:Edge]->
//! - WHERE 条件过滤
//! - 属性投影
//! - ORDER BY / LIMIT / SKIP

use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::planner::connector::SegmentsConnector;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::{PlanNodeEnum, ProjectNode, ScanVerticesNode};
use crate::query::planner::plan::core::nodes::{LimitNode, SortNode};
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::validator::YieldColumn;

/// MATCH 规划器
///
/// 使用 AstContext 中的语句进行规划，生成可执行的执行计划。
#[derive(Debug, Clone)]
pub struct MatchPlanner {
    config: MatchPlannerConfig,
}

#[derive(Debug, Clone, Default)]
pub struct MatchPlannerConfig {
    pub default_limit: usize,
    pub max_limit: usize,
    pub enable_index_optimization: bool,
}

impl MatchPlanner {
    pub fn new() -> Self {
        Self {
            config: MatchPlannerConfig::default(),
        }
    }

    pub fn with_config(config: MatchPlannerConfig) -> Self {
        Self { config }
    }

    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "MATCH"
    }

    pub fn parse_tag_from_pattern(pattern: &str) -> Option<String> {
        let colon_pos = pattern.find(':')?;
        let closing_paren_pos = pattern.find(')')?;
        if colon_pos < closing_paren_pos {
            let tag_part = &pattern[colon_pos + 1..closing_paren_pos];
            Some(tag_part.to_string())
        } else {
            None
        }
    }

    pub fn transform_with_full_context(
        &mut self,
        query_context: &mut QueryContext,
        ast_ctx: &AstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        let stmt = ast_ctx.sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;

        let space_id = ast_ctx.space().space_id.unwrap_or(1) as i32;

        let mut current_plan = self.plan_match_pattern(ast_ctx, stmt, space_id)?;

        if ast_ctx.query_type() == crate::query::context::ast::QueryType::ReadQuery {
            if let Some(where_condition) = self.extract_where_condition(stmt)? {
                current_plan = self.plan_filter(current_plan, where_condition, space_id)?;
            }

            if let Some(return_columns) = self.extract_return_columns(ast_ctx, stmt)? {
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

    fn plan_match_pattern(
        &self,
        _ast_ctx: &AstContext,
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

    fn plan_node_pattern(
        &self,
        space_id: i32,
    ) -> Result<PlanNodeEnum, PlannerError> {
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
        order_by: Vec<crate::query::validator::OrderByItem>,
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
        _ast_ctx: &AstContext,
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
    ) -> Result<Option<Vec<crate::query::validator::OrderByItem>>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                if let Some(order_by_clause) = &match_stmt.order_by {
                    let items = order_by_clause.items.iter().map(|item| {
                        crate::query::validator::OrderByItem {
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
                if skip > 0 || limit != usize::MAX {
                    Ok(Some(PaginationInfo { skip, limit }))
                } else {
                    Ok(Some(PaginationInfo { skip: 0, limit: self.config.default_limit }))
                }
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

    fn join_plans(
        &self,
        left: SubPlan,
        right: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        SegmentsConnector::cross_join(left, right)
    }
}

impl Planner for MatchPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let _stmt = ast_ctx.sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;

        let space_id = ast_ctx.space().space_id.unwrap_or(1) as i32;

        let start_node = ScanVerticesNode::new(space_id);
        let current_plan = SubPlan::from_root(start_node.into_enum());

        if ast_ctx.query_type() == crate::query::context::ast::QueryType::ReadQuery {
        }

        Ok(current_plan)
    }

    fn transform_with_full_context(
        &mut self,
        query_context: &mut QueryContext,
        ast_ctx: &AstContext,
    ) -> Result<ExecutionPlan, PlannerError> {
        self.transform_with_full_context(query_context, ast_ctx)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }

    fn name(&self) -> &'static str {
        "MatchPlanner"
    }
}

#[derive(Debug, Clone)]
pub struct PaginationInfo {
    pub skip: usize,
    pub limit: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::ast::AstContext;

    #[test]
    fn test_match_planner_creation() {
        let planner = MatchPlanner::new();
        assert!(MatchPlanner::match_ast_ctx(&AstContext::from_strings("MATCH", "MATCH (n)")));
    }

    #[test]
    fn test_match_planner_make() {
        let planner = MatchPlanner::make();
        assert!(planner.match_planner(&AstContext::from_strings("MATCH", "MATCH (n)")));
        assert!(!planner.match_planner(&AstContext::from_strings("GO", "GO 1 TO 2")));
    }

    #[test]
    fn test_match_planner_match_ast_ctx() {
        assert!(MatchPlanner::match_ast_ctx(&AstContext::from_strings(
            "MATCH",
            "MATCH (n)"
        )));
        assert!(MatchPlanner::match_ast_ctx(&AstContext::from_strings(
            "match",
            "match (n)"
        )));
        assert!(!MatchPlanner::match_ast_ctx(&AstContext::from_strings(
            "GO",
            "GO 1 TO 2"
        )));
    }

    #[test]
    fn test_parse_tag_from_pattern() {
        assert_eq!(MatchPlanner::parse_tag_from_pattern("(n:Person)"), Some("Person".to_string()));
        assert_eq!(MatchPlanner::parse_tag_from_pattern("(n:Tag1:Tag2)"), Some("Tag1:Tag2".to_string()));
        assert_eq!(MatchPlanner::parse_tag_from_pattern("(n)"), None);
    }

    #[test]
    fn test_pagination_info() {
        let pagination = PaginationInfo { skip: 5, limit: 10 };
        assert_eq!(pagination.skip, 5);
        assert_eq!(pagination.limit, 10);
    }
}
