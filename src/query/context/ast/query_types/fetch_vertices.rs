//! Fetch Vertices查询上下文

use crate::core::types::expression::Expression;
use crate::query::context::ast::{AstContext, ExpressionProps, FromType, Over, Starts, StepClause, YieldColumns};
use crate::core::YieldColumn;

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
                // 处理顶点ID列表，类似于nebula-graph中的validateStarts函数
                ctx.from.vids = ids.iter()
                    .map(|expr| {
                        // 将Expression转换为字符串表示
                        // 在实际实现中，这里应该对表达式进行求值，类似于nebula-graph中的expr->eval(ctx(nullptr))
                        // 但现在我们暂时简单地将其转换为字符串
                        match expr {
                            crate::core::Expression::Literal(value) => {
                                // 如果是常量，直接转换为字符串
                                format!("{}", value)
                            }
                            crate::core::Expression::Variable(var_name) => {
                                // 如果是变量，返回变量名
                                var_name.clone()
                            }
                            _ => {
                                // 其他情况，使用调试格式
                                format!("{:?}", expr)
                            }
                        }
                    })
                    .collect();
                
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
