//! DELETE语句解析器

use crate::query::parser::ast::*;
use crate::query::parser::expressions::ExpressionParser;
use crate::query::parser::{ParseError, TokenKind};

pub trait DeleteStmtParser: ExpressionParser {
    /// 解析DELETE语句
    fn parse_delete_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        let delete_vertices = match self.current_token().kind {
            TokenKind::Vertex | TokenKind::Vertices => {
                self.next_token();
                true
            }
            TokenKind::Edge | TokenKind::Edges => {
                self.next_token();
                false
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!(
                        "Expected VERTEX or EDGE after DELETE, got {:?}",
                        self.current_token().kind
                    ),
                    self.current_token().line,
                    self.current_token().column,
                ));
            }
        };

        // For simplicity, just parsing expression list
        let mut vertex_exprs = Vec::new();
        loop {
            vertex_exprs.push(self.parse_expression()?);

            if self.current_token().kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        // Optionally parse WHERE clause
        let where_clause = if self.current_token().kind == TokenKind::Where {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        // Optionally parse YIELD clause
        let _yield_clause = if self.current_token().kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        Ok(Some(Stmt::Delete(DeleteStmt {
            span: Span::default(),
            target: if delete_vertices {
                DeleteTarget::Vertices(vertex_exprs)
            } else {
                // 简化的边删除实现
                DeleteTarget::Edges {
                    src: Expr::Variable(VariableExpr::new("src".to_string(), Span::default())),
                    dst: Expr::Variable(VariableExpr::new("dst".to_string(), Span::default())),
                    edge_type: None,
                    rank: None,
                }
            },
            where_clause,
        })))
    }

    fn parse_where_clause(&mut self) -> Result<Expr, ParseError>;
    fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError>;
}
