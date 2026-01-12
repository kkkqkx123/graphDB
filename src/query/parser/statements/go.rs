//! GO语句解析器

use crate::query::parser::ast::*;
use crate::query::parser::expressions::ExpressionParser;
use crate::query::parser::{ParseError, TokenKind};

pub trait GoStatementParser: ExpressionParser {
    /// 解析GO语句
    fn parse_go_statement(&mut self) -> Result<Option<Stmt>, ParseError> {
        // 解析 STEPS
        let steps = if self.current_token().kind == TokenKind::Step {
            self.next_token();
            if self.current_token().kind == TokenKind::Upto {
                // 解析 M TO N STEPS形式
                self.next_token();
                // 这里应该解析具体的步骤数，现在简化处理
                let _from_step = self.parse_expression()?;
                self.expect_token(TokenKind::To)?;
                let _to_step = self.parse_expression()?;
                Steps::Range { min: 1, max: 10 } // 简化处理，使用固定范围
            } else {
                // 解析 N STEPS形式
                let _step_expr = self.parse_expression()?;
                Steps::Fixed(1) // 简化处理，使用固定步数
            }
        } else {
            Steps::Fixed(1) // 默认1步
        };

        // 解析 OVER
        self.expect_token(TokenKind::Over)?;
        let over_clause = self.parse_over_clause()?;

        // 解析 FROM
        self.expect_token(TokenKind::From)?;
        let from_list = self.parse_vertex_list()?;

        // 解析 WHERE (可选)
        let where_clause = if self.current_token().kind == TokenKind::Where {
            self.next_token();
            Some(self.parse_expression()?)
        } else {
            None
        };

        // 解析 YIELD
        self.expect_token(TokenKind::Yield)?;
        let yield_clause = Some(self.parse_yield_clause()?);

        Ok(Some(Stmt::Go(GoStmt {
            span: Span::default(),
            steps,
            from: FromClause {
                span: Span::default(),
                vertices: from_list,
            },
            over: Some(over_clause),
            where_clause,
            yield_clause,
        })))
    }

    fn parse_over_clause(&mut self) -> Result<OverClause, ParseError> {
        // 解析边类型列表
        let edge_types = if self.current_token().kind == TokenKind::Star {
            self.next_token();
            vec!["*".to_string()] // 表示所有边类型
        } else {
            let mut types = Vec::new();
            types.push(self.parse_identifier()?);

            while self.current_token().kind == TokenKind::Comma {
                self.next_token(); // 跳过逗号
                types.push(self.parse_identifier()?);
            }
            types
        };

        // 解析方向 (可选，默认OUT)
        let direction = if self.current_token().kind == TokenKind::Colon {
            self.next_token();
            match self.current_token().kind {
                TokenKind::Out => {
                    self.next_token();
                    EdgeDirection::Outgoing
                }
                TokenKind::In => {
                    self.next_token();
                    EdgeDirection::Incoming
                }
                TokenKind::Both => {
                    self.next_token();
                    EdgeDirection::Both
                }
                _ => {
                    return Err(ParseError::syntax_error(
                        format!(
                            "Expected direction (OUT, IN, or BOTH), got {:?}",
                            self.current_token().kind
                        ),
                        self.current_token().line,
                        self.current_token().column,
                    ))
                }
            }
        } else {
            EdgeDirection::Outgoing // 默认方向
        };

        Ok(OverClause {
            span: Span::default(),
            edge_types,
            direction,
        })
    }

    fn parse_vertex_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut vertices = Vec::new();

        vertices.push(self.parse_expression()?);

        while self.current_token().kind == TokenKind::Comma {
            self.next_token();
            vertices.push(self.parse_expression()?);
        }

        Ok(vertices)
    }

    fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError>;
    fn add_error(&mut self, error: ParseError);
}
