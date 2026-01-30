//! Fetch Vertices查询上下文

use crate::core::types::expression::Expression;
use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Over, Starts, StepClause, YieldColumns};
use crate::query::validator::structs::clause_structs::YieldColumn;

/// Fetch Vertices查询上下文
///
/// 获取点数据的查询上下文
#[derive(Debug, Clone)]
pub struct FetchVerticesContext {
    pub base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub filter: Option<Expression>,
    pub col_names: Vec<String>,
    pub expr_props: ExpressionProps,
    pub input_var_name: String,
    pub distinct: bool,
    pub yield_expression: Option<YieldColumns>,
}

impl FetchVerticesContext {
    pub fn new(base: AstContext) -> Self {
        Self {
            base,
            from: Starts::new(FromType::default()),
            steps: StepClause::new(),
            over: Over::new(),
            filter: None,
            col_names: Vec::new(),
            expr_props: ExpressionProps::default(),
            input_var_name: String::new(),
            distinct: false,
            yield_expression: None,
        }
    }

    pub fn from_sentence(
        base: AstContext,
        fetch_stmt: &crate::query::parser::ast::stmt::FetchStmt,
    ) -> Self {
        let mut ctx = Self::new(base);

        match &fetch_stmt.target {
            crate::query::parser::ast::stmt::FetchTarget::Vertices { ids, properties } => {
                ctx.from.user_defined_var_name = String::from("FETCH_VERTICES_INPUT");
                ctx.col_names = vec!["vid".to_string()];
                ctx.yield_expression = Some(YieldColumns {
                    columns: properties.as_ref().map_or(vec![], |props| {
                        props.iter().map(|prop| {
                            YieldColumn {
                                expression: crate::core::Expression::Variable(prop.clone()),
                                alias: prop.clone(),
                                is_matched: false,
                            }
                        }).collect()
                    }),
                });
            }
            _ => {}
        }

        ctx
    }
}
