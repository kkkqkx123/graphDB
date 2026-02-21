//! GO查询上下文

use crate::core::types::expression::Expression;
use crate::core::types::EdgeDirection as CoreEdgeDirection;
use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Over, Starts, StepClause, YieldColumns};
use crate::core::YieldColumn;

/// GO查询上下文
///
/// GO遍历查询的上下文信息，包含：
/// - 公共遍历字段
/// - Yield表达式
/// - 查询选项（distinct, random, limits等）
/// - 属性表达式（src, dst, edge）
/// - VID列名
#[derive(Debug, Clone)]
pub struct GoContext {
    pub base: AstContext,
    pub from: Starts,
    pub steps: StepClause,
    pub over: Over,
    pub filter: Option<Expression>,
    pub col_names: Vec<String>,
    pub expr_props: ExpressionProps,
    pub input_var_name: String,
    pub yield_expression: Option<YieldColumns>,
    pub distinct: bool,
    pub random: bool,
    pub limits: Vec<i64>,
    pub vids_var: String,
    pub join_input: bool,
    pub join_dst: bool,
    pub is_simple: bool,
    pub dst_props_expression: Option<YieldColumns>,
    pub src_props_expression: Option<YieldColumns>,
    pub edge_props_expression: Option<YieldColumns>,
    pub src_vid_col_name: String,
    pub dst_vid_col_name: String,
}

impl GoContext {
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
            yield_expression: None,
            distinct: false,
            random: false,
            limits: Vec::new(),
            vids_var: String::new(),
            join_input: false,
            join_dst: false,
            is_simple: false,
            dst_props_expression: None,
            src_props_expression: None,
            edge_props_expression: None,
            src_vid_col_name: String::new(),
            dst_vid_col_name: String::new(),
        }
    }

    pub fn from_sentence(
        base: AstContext,
        go_stmt: &crate::query::parser::ast::stmt::GoStmt,
    ) -> Self {
        let mut ctx = Self::new(base);

        ctx.from = Starts::new(FromType::InstantExpression);
        ctx.from.user_defined_var_name = match &go_stmt.from {
            crate::query::parser::ast::stmt::FromClause { vertices, .. } => {
                vertices.first().map_or(String::new(), |expr| extract_var_name(expr))
            }
        };

        ctx.steps = StepClause::new();
        match &go_stmt.steps {
            crate::query::parser::ast::stmt::Steps::Fixed(n) => {
                ctx.steps.m_steps = *n;
                ctx.steps.n_steps = *n;
            }
            crate::query::parser::ast::stmt::Steps::Range { min, max } => {
                ctx.steps.m_steps = *min;
                ctx.steps.n_steps = *max;
                ctx.steps.is_m_to_n = true;
            }
            crate::query::parser::ast::stmt::Steps::Variable(_) => {
                ctx.steps.m_steps = 1;
                ctx.steps.n_steps = 1;
            }
        };

        ctx.filter = go_stmt.where_clause.clone();

        if let Some(ref over) = go_stmt.over {
            ctx.over.edge_types = over.edge_types.clone();
            ctx.over.direction = match over.direction {
                crate::query::parser::ast::types::EdgeDirection::Out => {
                    CoreEdgeDirection::Out
                }
                crate::query::parser::ast::types::EdgeDirection::In => {
                    CoreEdgeDirection::In
                }
                crate::query::parser::ast::types::EdgeDirection::Both => {
                    CoreEdgeDirection::Both
                }
            };
        }

        if let Some(ref yield_clause) = go_stmt.yield_clause {
            ctx.yield_expression = Some(YieldColumns {
                columns: yield_clause.items.iter().map(|item| {
                    YieldColumn {
                        expression: item.expression.clone(),
                        alias: item.alias.clone().unwrap_or_default(),
                        is_matched: false,
                    }
                }).collect(),
            });
        }

        ctx
    }
}

fn extract_var_name(expr: &crate::core::Expression) -> String {
    match expr {
        crate::core::Expression::Variable(name) => name.clone(),
        _ => String::new(),
    }
}
