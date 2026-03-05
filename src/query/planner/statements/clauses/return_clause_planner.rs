//! RETURN 子句规划器
//!
//! 负责规划 RETURN 子句的执行，实现结果投影。

use crate::query::validator::context::ExpressionAnalysisContext;
use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::YieldColumn;
use crate::query::parser::ast::Stmt;
use crate::query::planner::plan::core::nodes::data_processing_node::DedupNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::planner::statements::statement_planner::ClausePlanner;
use crate::query::validator::helpers::generate_default_alias_from_contextual;
use crate::query::validator::structs::CypherClauseKind;
use crate::query::QueryContext;
use std::sync::Arc;

pub use crate::query::planner::plan::core::PlanNodeEnum;

/// RETURN 子句规划器
///
/// 负责规划 RETURN 子句的执行，实现结果投影。
#[derive(Debug)]
pub struct ReturnClausePlanner {
    distinct: bool,
}

impl ReturnClausePlanner {
    pub fn new() -> Self {
        Self { distinct: false }
    }

    pub fn with_distinct(distinct: bool) -> Self {
        Self { distinct }
    }

    pub fn from_stmt(stmt: &Stmt) -> Self {
        let distinct = extract_distinct_flag(stmt);
        Self::with_distinct(distinct)
    }
}

fn extract_distinct_flag(stmt: &Stmt) -> bool {
    if let Stmt::Match(match_stmt) = stmt {
        if let Some(return_clause) = &match_stmt.return_clause {
            return return_clause.distinct;
        }
    }
    false
}

fn extract_return_columns(stmt: &Stmt) -> Result<Vec<YieldColumn>, PlannerError> {
    let mut columns = Vec::new();

    if let Stmt::Match(match_stmt) = stmt {
        if let Some(return_clause) = &match_stmt.return_clause {
            for item in &return_clause.items {
                match item {
                    crate::query::parser::ast::stmt::ReturnItem::Expression {
                        expression,
                        alias,
                    } => {
                        let alias = alias.clone().or_else(|| {
                            Some(generate_default_alias_from_contextual(expression))
                        });
                        columns.push(YieldColumn {
                            expression: expression.clone(),
                            alias: alias.unwrap_or_else(|| "expr".to_string()),
                            is_matched: false,
                        });
                    }
                }
            }
        }
    }

    if columns.is_empty() {
        return Err(PlannerError::PlanGenerationFailed(
            "RETURN 子句缺少返回项".to_string()
        ));
    }

    Ok(columns)
}

impl ClausePlanner for ReturnClausePlanner {
    fn clause_kind(&self) -> CypherClauseKind {
        CypherClauseKind::Return
    }

    fn transform_clause(
        &self,
        _qctx: Arc<QueryContext>,
        stmt: &Stmt,
        input_plan: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let yield_columns = extract_return_columns(stmt)?;

        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("RETURN 子句需要输入计划".to_string())
        })?;

        let project_node = ProjectNode::new(input_node.clone(), yield_columns)?;

        let final_node = if self.distinct {
            match DedupNode::new(project_node.clone().into_enum()) {
                Ok(dedup) => dedup.into_enum(),
                Err(_) => project_node.into_enum(),
            }
        } else {
            project_node.into_enum()
        };

        Ok(SubPlan::new(Some(final_node), input_plan.tail))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ExpressionAnalysisContext;
    use crate::core::Expression;
    use crate::query::parser::ast::Span;
    use crate::query::planner::plan::core::nodes::StartNode;
    use crate::query::planner::plan::core::PlanNodeEnum;
    use std::sync::Arc;

    #[test]
    fn test_return_clause_planner_creation() {
        let planner = ReturnClausePlanner::new();
        assert_eq!(planner.clause_kind(), CypherClauseKind::Return);
    }

    #[test]
    fn test_return_clause_planner_with_distinct() {
        let planner = ReturnClausePlanner::with_distinct(true);
        assert!(planner.distinct);
    }

    #[test]
    fn test_extract_distinct_flag() {
        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: Some(crate::query::parser::ast::stmt::ReturnClause {
                span: Span::default(),
                items: vec![],
                distinct: true,
                order_by: None,
                limit: None,
                skip: None,
                sample: None,
            }),
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let distinct = extract_distinct_flag(&match_stmt);
        assert!(distinct);
    }

    #[test]
    fn test_extract_return_columns() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("n".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: Some(crate::query::parser::ast::stmt::ReturnClause {
                span: Span::default(),
                items: vec![crate::query::parser::ast::stmt::ReturnItem::Expression {
                    expression: ctx_expr.clone(),
                    alias: None,
                }],
                distinct: false,
                order_by: None,
                limit: None,
                skip: None,
                sample: None,
            }),
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let columns = extract_return_columns(&match_stmt).expect("提取失败");
        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0].alias, "n");
    }

    #[test]
    fn test_extract_return_columns_empty() {
        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: Some(crate::query::parser::ast::stmt::ReturnClause {
                span: Span::default(),
                items: vec![],
                distinct: false,
                order_by: None,
                limit: None,
                skip: None,
                sample: None,
            }),
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let result = extract_return_columns(&match_stmt);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_default_alias() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("n".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let contextual = ContextualExpression::new(id, ctx.clone());
        let alias = generate_default_alias_from_contextual(&contextual);
        assert_eq!(alias, "n");

        let expr = Expression::Property {
            object: Box::new(Expression::Variable("n".to_string())),
            property: "name".to_string(),
        };
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let contextual = ContextualExpression::new(id, ctx.clone());
        let alias = generate_default_alias_from_contextual(&contextual);
        assert_eq!(alias, "prop.name");

        let expr = Expression::Function {
            name: "count".to_string(),
            args: vec![],
        };
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let contextual = ContextualExpression::new(id, ctx.clone());
        let alias = generate_default_alias_from_contextual(&contextual);
        assert_eq!(alias, "count");
    }

    #[test]
    fn test_transform_clause() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("n".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: Some(crate::query::parser::ast::stmt::ReturnClause {
                span: Span::default(),
                items: vec![crate::query::parser::ast::stmt::ReturnItem::Expression {
                    expression: ctx_expr.clone(),
                    alias: None,
                }],
                distinct: false,
                order_by: None,
                limit: None,
                skip: None,
                sample: None,
            }),
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let start_node = StartNode::new();
        let start_node_enum = PlanNodeEnum::Start(start_node.clone());
        let input_plan = SubPlan {
            root: Some(start_node_enum.clone()),
            tail: Some(start_node_enum),
        };

        let planner = ReturnClausePlanner::new();
        let qctx = Arc::new(crate::query::QueryContext::new(
            Arc::new(crate::query::query_request_context::QueryRequestContext {
                session_id: None,
                user_name: None,
                space_name: None,
                query: String::new(),
                parameters: std::collections::HashMap::new(),
            })
        ));

        let result = planner.transform_clause(qctx, &match_stmt, input_plan);
        assert!(result.is_ok());

        let sub_plan = result.expect("transform_clause should succeed");
        assert!(sub_plan.root.is_some());

        match sub_plan.root {
            Some(PlanNodeEnum::Project(_)) => {}
            Some(PlanNodeEnum::Dedup(_)) => {}
            _ => panic!("Expected ProjectNode or DedupNode"),
        }
    }

    #[test]
    fn test_transform_clause_with_distinct() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("n".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: Some(crate::query::parser::ast::stmt::ReturnClause {
                span: Span::default(),
                items: vec![crate::query::parser::ast::stmt::ReturnItem::Expression {
                    expression: ctx_expr.clone(),
                    alias: None,
                }],
                distinct: true,
                order_by: None,
                limit: None,
                skip: None,
                sample: None,
            }),
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let start_node = StartNode::new();
        let start_node_enum = PlanNodeEnum::Start(start_node.clone());
        let input_plan = SubPlan {
            root: Some(start_node_enum.clone()),
            tail: Some(start_node_enum),
        };

        let planner = ReturnClausePlanner::with_distinct(true);
        let qctx = Arc::new(crate::query::QueryContext::new(
            Arc::new(crate::query::query_request_context::QueryRequestContext {
                session_id: None,
                user_name: None,
                space_name: None,
                query: String::new(),
                parameters: std::collections::HashMap::new(),
            })
        ));

        let result = planner.transform_clause(qctx, &match_stmt, input_plan);
        assert!(result.is_ok());

        let sub_plan = result.expect("transform_clause should succeed");
        assert!(sub_plan.root.is_some());

        if let Some(PlanNodeEnum::Dedup(_)) = sub_plan.root {
        } else {
            panic!("Expected DedupNode with distinct=true");
        }
    }

    #[test]
    fn test_transform_clause_empty_input_plan() {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::Variable("n".to_string());
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = crate::core::types::ContextualExpression::new(id, ctx);

        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: Some(crate::query::parser::ast::stmt::ReturnClause {
                span: Span::default(),
                items: vec![crate::query::parser::ast::stmt::ReturnItem::Expression {
                    expression: ctx_expr.clone(),
                    alias: None,
                }],
                distinct: false,
                order_by: None,
                limit: None,
                skip: None,
                sample: None,
            }),
            order_by: None,
            limit: None,
            skip: None,
            optional: false,
        });

        let input_plan = SubPlan {
            root: None,
            tail: None,
        };

        let planner = ReturnClausePlanner::new();
        let qctx = Arc::new(crate::query::QueryContext::new(
            Arc::new(crate::query::query_request_context::QueryRequestContext {
                session_id: None,
                user_name: None,
                space_name: None,
                query: String::new(),
                parameters: std::collections::HashMap::new(),
            })
        ));

        let result = planner.transform_clause(qctx, &match_stmt, input_plan);
        assert!(result.is_err());
    }
}
