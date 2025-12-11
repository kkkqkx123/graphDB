//! Parser implementation for the query parser
//!
//! This module implements a recursive descent parser that converts tokens into AST.

use crate::core::Value;
use crate::query::parser::ast::*;
use crate::query::parser::error::{ParseError, ParseErrors};
use crate::query::parser::lexer::Lexer;
use crate::query::parser::token::{Token, TokenKind};

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    errors: ParseErrors,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();

        Parser {
            lexer,
            current_token,
            errors: ParseErrors::new(),
        }
    }

    fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn peek_token(&self) -> TokenKind {
        // Simplified peek implementation - just return current token for now
        // In a real implementation, we would need proper lookahead
        self.current_token.kind.clone()
    }

    fn expect_token(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        if self.current_token.kind == expected {
            let token = self.current_token.clone();
            self.next_token();
            Ok(token)
        } else {
            let error = ParseError::syntax_error(
                format!("Expected {:?}, got {:?}", expected, self.current_token.kind),
                self.current_token.line,
                self.current_token.column,
            );
            self.errors.add(error.clone());
            Err(error)
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current_token.kind, TokenKind::Eof)
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, ParseErrors> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if let Some(stmt) = self
                .parse_statement()
                .map_err(|e| ParseErrors::from(vec![e]))?
            {
                statements.push(stmt);
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors.clone());
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        match self.current_token.kind {
            TokenKind::Create => {
                self.next_token();
                self.parse_create_statement()
            }
            TokenKind::Match => {
                self.next_token();
                self.parse_match_statement()
            }
            TokenKind::Delete => {
                self.next_token();
                self.parse_delete_statement()
            }
            TokenKind::Update => {
                self.next_token();
                self.parse_update_statement()
            }
            TokenKind::Use => {
                self.next_token();
                self.parse_use_statement()
            }
            TokenKind::Show => {
                self.next_token();
                self.parse_show_statement()
            }
            TokenKind::Explain => {
                self.next_token();
                self.parse_explain_statement()
            }
            TokenKind::Semicolon => {
                // Skip standalone semicolons
                self.next_token();
                Ok(None)
            }
            TokenKind::Eof => Ok(None),
            _ => {
                let error = ParseError::syntax_error(
                    format!("Unexpected token: {:?}", self.current_token.kind),
                    self.current_token.line,
                    self.current_token.column,
                );
                self.errors.add(error.clone());
                Err(error)
            }
        }
    }

    fn parse_create_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        match self.current_token.kind {
            TokenKind::Vertex | TokenKind::Vertices => {
                self.next_token();
                self.parse_create_node_statement()
            }
            TokenKind::Edge | TokenKind::Edges => {
                self.next_token();
                self.parse_create_edge_statement()
            }
            _ => {
                let error = ParseError::syntax_error(
                    format!(
                        "Expected VERTEX or EDGE after CREATE, got {:?}",
                        self.current_token.kind
                    ),
                    self.current_token.line,
                    self.current_token.column,
                );
                self.errors.add(error.clone());
                Err(error)
            }
        }
    }

    fn parse_create_node_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let if_not_exists = self.check_and_skip_keyword(TokenKind::If);

        // Skip 'EXISTS' if we found 'IF'
        if if_not_exists {
            self.expect_token(TokenKind::Exists)?;
        }

        // Parse tag list
        let tags = self.parse_tag_list()?;

        // Parse SET clause
        self.expect_token(TokenKind::Set)?;
        let properties = self.parse_property_list()?;

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token.kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        Ok(Some(Statement::CreateNode(CreateNodeStatement {
            if_not_exists,
            tags,
            properties,
            yield_clause,
        })))
    }

    fn parse_create_edge_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let if_not_exists = self.check_and_skip_keyword(TokenKind::If);

        // Skip 'EXISTS' if we found 'IF'
        if if_not_exists {
            self.expect_token(TokenKind::Exists)?;
        }

        // Parse edge type
        let edge_type = self.parse_identifier()?;

        // Parse source and destination
        self.expect_token(TokenKind::LParen)?;
        let src = self.parse_expression()?;
        self.expect_token(TokenKind::RParen)?;

        // Parse edge pattern -> or <-
        let direction = if self.current_token.kind == TokenKind::Arrow {
            // ->
            self.next_token();
            EdgeDirection::Outbound
        } else if self.current_token.kind == TokenKind::BackArrow {
            // <-
            self.next_token();
            EdgeDirection::Inbound
        } else {
            return Err(ParseError::syntax_error(
                format!("Expected -> or <-, got {:?}", self.current_token.kind),
                self.current_token.line,
                self.current_token.column,
            ));
        };

        self.expect_token(TokenKind::LParen)?;
        let dst = self.parse_expression()?;
        self.expect_token(TokenKind::RParen)?;

        // Parse SET clause
        self.expect_token(TokenKind::Set)?;
        let properties = self.parse_property_list()?;

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token.kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        Ok(Some(Statement::CreateEdge(CreateEdgeStatement {
            if_not_exists,
            edge_type,
            src,
            dst,
            ranking: None, // No ranking in basic implementation
            properties,
            yield_clause,
        })))
    }

    fn parse_match_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        // Parse match patterns
        let mut clauses = Vec::new();

        // Parse the pattern part of MATCH
        let patterns = self.parse_match_patterns()?;
        let where_clause = if self.current_token.kind == TokenKind::Where {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        clauses.push(MatchClause::Match(MatchClauseDetail {
            patterns,
            where_clause,
            with_clause: None,
        }));

        // Parse optional RETURN clause
        if self.current_token.kind == TokenKind::Return {
            clauses.push(MatchClause::Return(self.parse_return_clause()?));
        }

        Ok(Some(Statement::Match(MatchStatement {
            clauses,
            return_clause: None,
        })))
    }

    fn parse_match_patterns(&mut self) -> Result<Vec<MatchPath>, ParseError> {
        let mut patterns = Vec::new();

        // For now, just parse a simple path pattern
        // In a real implementation, we'd have more complex pattern parsing
        let path = self.parse_match_path()?;
        patterns.push(path);

        Ok(patterns)
    }

    fn parse_match_path(&mut self) -> Result<MatchPath, ParseError> {
        let mut path = Vec::new();

        // Parse nodes and edges in the path
        loop {
            // Parse a node
            if self.current_token.kind == TokenKind::LParen {
                path.push(MatchPathSegment::Node(self.parse_match_node()?));
            } else {
                break;
            }

            // Check if there's an edge following
            if self.current_token.kind == TokenKind::Arrow
                || self.current_token.kind == TokenKind::BackArrow
                || matches!(self.current_token.kind, TokenKind::Minus)
            {
                path.push(MatchPathSegment::Edge(self.parse_match_edge()?));
            } else {
                break;
            }
        }

        Ok(MatchPath { path })
    }

    fn parse_match_node(&mut self) -> Result<MatchNode, ParseError> {
        self.expect_token(TokenKind::LParen)?;

        // Parse optional identifier
        let identifier = if matches!(self.current_token.kind, TokenKind::Identifier(_)) {
            let id = self.parse_identifier()?;
            if self.current_token.kind == TokenKind::Colon {
                // There's a label following
                Some(id)
            } else {
                // No label, just identifier
                self.expect_token(TokenKind::RParen)?;
                return Ok(MatchNode {
                    identifier: Some(id),
                    labels: vec![],
                    properties: None,
                    predicates: vec![],
                });
            }
        } else {
            None
        };

        // Parse optional label
        let mut labels = Vec::new();
        if self.current_token.kind == TokenKind::Colon {
            self.next_token();
            labels.push(Label {
                name: self.parse_identifier()?,
            });
        }

        // Parse optional properties
        let properties = if self.current_token.kind == TokenKind::LBrace {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect_token(TokenKind::RParen)?;

        Ok(MatchNode {
            identifier,
            labels,
            properties,
            predicates: vec![],
        })
    }

    fn parse_match_edge(&mut self) -> Result<MatchEdge, ParseError> {
        let direction = match self.current_token.kind {
            TokenKind::Arrow => {
                self.next_token();
                EdgeDirection::Outbound
            }
            TokenKind::BackArrow => {
                self.next_token();
                EdgeDirection::Inbound
            }
            TokenKind::Minus => {
                self.next_token();
                EdgeDirection::Bidirectional
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!(
                        "Expected edge direction (->, <-, -), got {:?}",
                        self.current_token.kind
                    ),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };

        // Check if it's followed by an edge type in brackets [type]
        let mut types = Vec::new();
        let mut identifier = None;
        let mut properties = None;

        if self.current_token.kind == TokenKind::LBracket {
            self.next_token();

            // Parse optional identifier or type
            if matches!(self.current_token.kind, TokenKind::Identifier(_)) {
                let id = self.parse_identifier()?;

                // Check if it's an identifier with type or just a type
                if self.current_token.kind == TokenKind::Colon {
                    // It's identifier:type format
                    identifier = Some(id);
                    self.next_token();
                    types.push(self.parse_identifier()?);
                } else {
                    // Just a type
                    types.push(id);
                }
            }

            // Parse optional properties
            if self.current_token.kind == TokenKind::LBrace {
                properties = Some(self.parse_expression()?);
            }

            self.expect_token(TokenKind::RBracket)?;
        }

        Ok(MatchEdge {
            direction,
            identifier,
            types,
            relationship: None,
            properties,
            predicates: vec![],
            range: None,
        })
    }

    fn parse_where_clause(&mut self) -> Result<WhereClause, ParseError> {
        self.next_token(); // Skip WHERE
        let condition = self.parse_expression()?;
        Ok(WhereClause { condition })
    }

    fn parse_return_clause(&mut self) -> Result<ReturnClause, ParseError> {
        self.next_token(); // Skip RETURN

        let distinct = if self.current_token.kind == TokenKind::Distinct {
            self.next_token();
            true
        } else {
            false
        };

        let mut items = Vec::new();

        // Parse return items
        loop {
            if self.current_token.kind == TokenKind::Eof
                || matches!(
                    self.current_token.kind,
                    TokenKind::Semicolon | TokenKind::Order | TokenKind::Limit | TokenKind::Skip
                )
            {
                break;
            }

            if self.current_token.kind == TokenKind::Star {
                items.push(ReturnItem::Asterisk);
                self.next_token();
            } else {
                let expr = self.parse_expression()?;

                let alias = if self.current_token.kind == TokenKind::As {
                    self.next_token();
                    Some(self.parse_identifier()?)
                } else if matches!(self.current_token.kind, TokenKind::Identifier(_))
                    && self.peek_token() != TokenKind::Comma
                {
                    // Potential alias without AS
                    Some(self.parse_identifier()?)
                } else {
                    None
                };

                items.push(ReturnItem::Expression(expr, alias));
            }

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        // Check for optional ORDER BY, LIMIT, SKIP
        let order_by = if self.current_token.kind == TokenKind::Order {
            Some(self.parse_order_by_clause()?)
        } else {
            None
        };

        let limit = if self.current_token.kind == TokenKind::Limit {
            Some(self.parse_limit_clause()?)
        } else {
            None
        };

        let skip = if self.current_token.kind == TokenKind::Skip {
            Some(self.parse_skip_clause()?)
        } else {
            None
        };

        Ok(ReturnClause {
            distinct,
            items,
            order_by,
            limit,
            skip,
        })
    }

    fn parse_order_by_clause(&mut self) -> Result<OrderByClause, ParseError> {
        self.expect_token(TokenKind::Order)?;
        self.expect_token(TokenKind::By)?;

        let mut items = Vec::new();

        loop {
            let expr = self.parse_expression()?;

            let order = if self.current_token.kind == TokenKind::Asc
                || self.current_token.kind == TokenKind::Ascending
            {
                self.next_token();
                OrderType::Asc
            } else if self.current_token.kind == TokenKind::Desc
                || self.current_token.kind == TokenKind::Descending
            {
                self.next_token();
                OrderType::Desc
            } else {
                OrderType::Asc // Default to ascending
            };

            items.push(OrderByItem { expr, order });

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        Ok(OrderByClause { items })
    }

    fn parse_limit_clause(&mut self) -> Result<LimitClause, ParseError> {
        self.next_token(); // Skip LIMIT
        let expr = self.parse_expression()?;
        Ok(LimitClause { expr })
    }

    fn parse_skip_clause(&mut self) -> Result<SkipClause, ParseError> {
        self.next_token(); // Skip SKIP
        let expr = self.parse_expression()?;
        Ok(SkipClause { expr })
    }

    fn parse_delete_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let delete_vertices = match self.current_token.kind {
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
                        self.current_token.kind
                    ),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };

        // For simplicity, just parsing expression list
        let mut vertex_exprs = Vec::new();
        loop {
            vertex_exprs.push(self.parse_expression()?);

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        // Optionally parse WHERE clause
        let where_clause = if self.current_token.kind == TokenKind::Where {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token.kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        Ok(Some(Statement::Delete(DeleteStatement {
            delete_vertices,
            vertex_exprs,
            edge_exprs: None, // Simplified for now
            where_clause,
            yield_clause,
        })))
    }

    fn parse_update_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        let update_vertices = match self.current_token.kind {
            TokenKind::Vertex => {
                self.next_token();
                true
            }
            TokenKind::Edge => {
                self.next_token();
                false
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!(
                        "Expected VERTEX or EDGE after UPDATE, got {:?}",
                        self.current_token.kind
                    ),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };

        // Parse vertex/edge reference
        let vertex_ref = Some(self.parse_expression()?);

        // Parse SET clause
        self.expect_token(TokenKind::Set)?;
        let mut update_items = Vec::new();

        loop {
            let prop = self.parse_property_ref()?;
            self.expect_token(TokenKind::Assign)?;
            let value = self.parse_expression()?;

            update_items.push(Assignment { prop, value });

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        // Optionally parse WHERE clause
        let condition = if self.current_token.kind == TokenKind::Where {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        // Optionally parse YIELD clause
        let yield_clause = if self.current_token.kind == TokenKind::Yield {
            Some(self.parse_yield_clause()?)
        } else {
            None
        };

        Ok(Some(Statement::Update(UpdateStatement {
            update_vertices,
            vertex_ref,
            edge_ref: None, // Simplified for now
            update_items,
            condition,
            yield_clause,
        })))
    }

    fn parse_use_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        self.next_token(); // Skip USE
        let space = self.parse_identifier()?;
        Ok(Some(Statement::Use(UseStatement { space })))
    }

    fn parse_show_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        self.next_token(); // Skip SHOW

        let show_stmt = match self.current_token.kind {
            TokenKind::Spaces => {
                self.next_token();
                ShowStatement::ShowSpaces
            }
            TokenKind::Tags => {
                self.next_token();
                ShowStatement::ShowTags
            }
            TokenKind::Edges => {
                self.next_token();
                ShowStatement::ShowEdges
            }
            TokenKind::Tag => {
                self.next_token();
                ShowStatement::ShowTags
            }
            TokenKind::Edge => {
                self.next_token();
                ShowStatement::ShowEdges
            }
            TokenKind::Users => {
                self.next_token();
                ShowStatement::ShowUsers
            }
            TokenKind::Roles => {
                self.next_token();
                let role = if matches!(self.current_token.kind, TokenKind::Identifier(_)) {
                    Some(self.parse_identifier()?)
                } else {
                    None
                };
                ShowStatement::ShowRoles(role)
            }
            TokenKind::Hosts => {
                self.next_token();
                ShowStatement::ShowHosts
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!(
                        "Unexpected token in SHOW statement: {:?}",
                        self.current_token.kind
                    ),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };

        Ok(Some(Statement::Show(show_stmt)))
    }

    fn parse_explain_statement(&mut self) -> Result<Option<Statement>, ParseError> {
        self.next_token(); // Skip EXPLAIN

        // Parse the statement to explain
        let stmt = self.parse_statement()?;
        if let Some(stmt) = stmt {
            Ok(Some(Statement::Explain(ExplainStatement {
                stmt: Box::new(stmt),
            })))
        } else {
            Err(ParseError::syntax_error(
                "Expected statement after EXPLAIN".to_string(),
                self.current_token.line,
                self.current_token.column,
            ))
        }
    }

    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        match &self.current_token.kind {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.next_token();
                Ok(name)
            }
            _ => {
                let error = ParseError::syntax_error(
                    format!("Expected identifier, got {:?}", self.current_token.kind),
                    self.current_token.line,
                    self.current_token.column,
                );
                self.errors.add(error.clone());
                Err(error)
            }
        }
    }

    fn parse_tag_list(&mut self) -> Result<Vec<TagIdentifier>, ParseError> {
        let mut tags = Vec::new();

        // If we start with a parenthesis, we have tag list: (tag1, tag2, ...)
        if self.current_token.kind == TokenKind::LParen {
            self.next_token(); // Skip '('

            loop {
                let tag_name = self.parse_identifier()?;
                let properties = if self.current_token.kind == TokenKind::LBrace {
                    Some(self.parse_property_map()?)
                } else {
                    None
                };

                tags.push(TagIdentifier {
                    name: tag_name,
                    properties,
                });

                if self.current_token.kind != TokenKind::Comma {
                    break;
                }
                self.next_token(); // Skip comma
            }

            self.expect_token(TokenKind::RParen)?;
        } else {
            // Just a single tag
            let tag_name = self.parse_identifier()?;
            tags.push(TagIdentifier {
                name: tag_name,
                properties: None,
            });
        }

        Ok(tags)
    }

    fn parse_property_list(&mut self) -> Result<Vec<Property>, ParseError> {
        let mut properties = Vec::new();

        if self.current_token.kind == TokenKind::LBrace {
            // In our simplified model, we expect {prop1: val1, prop2: val2}
            self.next_token(); // Skip '{'

            loop {
                if self.current_token.kind == TokenKind::RBrace {
                    break;
                }

                let name = self.parse_identifier()?;
                self.expect_token(TokenKind::Colon)?;
                let value = self.parse_expression()?;

                properties.push(Property { name, value });

                if self.current_token.kind != TokenKind::Comma {
                    break;
                }
                self.next_token(); // Skip comma
            }

            self.expect_token(TokenKind::RBrace)?;
        } else {
            // In a real implementation, we might have different syntax
            // For now, just parse as assignment list
            loop {
                let name = self.parse_identifier()?;
                self.expect_token(TokenKind::Assign)?;
                let value = self.parse_expression()?;

                properties.push(Property { name, value });

                if self.current_token.kind != TokenKind::Comma {
                    break;
                }
                self.next_token(); // Skip comma
            }
        }

        Ok(properties)
    }

    fn parse_property_map(
        &mut self,
    ) -> Result<std::collections::HashMap<String, Expression>, ParseError> {
        let mut map = std::collections::HashMap::new();

        self.expect_token(TokenKind::LBrace)?;

        loop {
            if self.current_token.kind == TokenKind::RBrace {
                break;
            }

            let key = self.parse_identifier()?;
            self.expect_token(TokenKind::Colon)?;
            let value = self.parse_expression()?;

            map.insert(key, value);

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token();
        }

        self.expect_token(TokenKind::RBrace)?;
        Ok(map)
    }

    fn parse_yield_clause(&mut self) -> Result<YieldClause, ParseError> {
        self.next_token(); // Skip YIELD

        let mut items = Vec::new();

        loop {
            let expr = self.parse_expression()?;
            let alias = if self.current_token.kind == TokenKind::As {
                self.next_token();
                Some(self.parse_identifier()?)
            } else if matches!(self.current_token.kind, TokenKind::Identifier(_))
                && self.peek_token() != TokenKind::Comma
            {
                // Potential alias without AS
                Some(self.parse_identifier()?)
            } else {
                None
            };

            items.push(YieldExpression { expr, alias });

            if self.current_token.kind != TokenKind::Comma {
                break;
            }
            self.next_token(); // Skip comma
        }

        Ok(YieldClause { items })
    }

    fn parse_property_ref(&mut self) -> Result<PropertyRef, ParseError> {
        let first = self.parse_identifier()?;

        if self.current_token.kind == TokenKind::Dot {
            self.next_token();
            let second = self.parse_identifier()?;
            Ok(PropertyRef::Prop(first, second))
        } else {
            Ok(PropertyRef::InlineProp(first))
        }
    }

    fn check_and_skip_keyword(&mut self, keyword: TokenKind) -> bool {
        if self.current_token.kind == keyword {
            self.next_token();
            true
        } else {
            false
        }
    }

    // Expression parsing methods
    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_logical_and()?;

        while self.current_token.kind == TokenKind::Or {
            self.next_token();
            let right = self.parse_logical_and()?;
            expr = Expression::Logical(Box::new(expr), LogicalOp::Or, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_equality()?;

        while self.current_token.kind == TokenKind::And {
            self.next_token();
            let right = self.parse_equality()?;
            expr = Expression::Logical(Box::new(expr), LogicalOp::And, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Eq => {
                    self.next_token();
                    RelationalOp::Eq
                }
                TokenKind::Ne => {
                    self.next_token();
                    RelationalOp::Ne
                }
                _ => break,
            };

            let right = self.parse_comparison()?;
            expr = Expression::Relational(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_addition()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Lt => {
                    self.next_token();
                    RelationalOp::Lt
                }
                TokenKind::Le => {
                    self.next_token();
                    RelationalOp::Le
                }
                TokenKind::Gt => {
                    self.next_token();
                    RelationalOp::Gt
                }
                TokenKind::Ge => {
                    self.next_token();
                    RelationalOp::Ge
                }
                TokenKind::Regex => {
                    self.next_token();
                    RelationalOp::Regex
                }
                _ => break,
            };

            let right = self.parse_addition()?;
            expr = Expression::Relational(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_addition(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_multiplication()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Plus => {
                    self.next_token();
                    ArithmeticOp::Add
                }
                TokenKind::Minus => {
                    self.next_token();
                    ArithmeticOp::Sub
                }
                _ => break,
            };

            let right = self.parse_multiplication()?;
            expr = Expression::Arithmetic(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_multiplication(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_unary()?;

        loop {
            let op = match self.current_token.kind {
                TokenKind::Star => {
                    self.next_token();
                    ArithmeticOp::Mul
                }
                TokenKind::Div => {
                    self.next_token();
                    ArithmeticOp::Div
                }
                TokenKind::Mod => {
                    self.next_token();
                    ArithmeticOp::Mod
                }
                _ => break,
            };

            let right = self.parse_unary()?;
            expr = Expression::Arithmetic(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        match self.current_token.kind {
            TokenKind::NotOp => {
                self.next_token();
                let expr = self.parse_exponentiation()?;
                Ok(Expression::Unary(UnaryOp::Not, Box::new(expr)))
            }
            TokenKind::Plus => {
                self.next_token();
                let expr = self.parse_exponentiation()?;
                Ok(Expression::Unary(UnaryOp::Plus, Box::new(expr)))
            }
            TokenKind::Minus => {
                self.next_token();
                let expr = self.parse_exponentiation()?;
                Ok(Expression::Unary(UnaryOp::Minus, Box::new(expr)))
            }
            _ => self.parse_exponentiation(),
        }
    }

    fn parse_exponentiation(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.parse_primary()?;

        // For now, we don't have exponentiation in our grammar
        // This function can be extended later if needed

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let expr = match &self.current_token.kind {
            TokenKind::IntegerLiteral(n) => {
                let value = Value::Int(*n);
                self.next_token();
                Expression::Constant(value)
            }
            TokenKind::FloatLiteral(n) => {
                let value = Value::Float(*n);
                self.next_token();
                Expression::Constant(value)
            }
            TokenKind::StringLiteral(s) => {
                let value = Value::String(s.clone());
                self.next_token();
                Expression::Constant(value)
            }
            TokenKind::BooleanLiteral(b) => {
                let value = Value::Bool(*b);
                self.next_token();
                Expression::Constant(value)
            }
            TokenKind::Null => {
                let value = Value::Null(crate::core::NullType::Null);
                self.next_token();
                Expression::Constant(value)
            }
            TokenKind::LParen => {
                self.next_token(); // Skip '('
                let expr = self.parse_expression()?;
                self.expect_token(TokenKind::RParen)?;
                expr
            }
            TokenKind::LBracket => {
                self.next_token(); // Skip '['
                let mut elements = Vec::new();

                if self.current_token.kind != TokenKind::RBracket {
                    loop {
                        elements.push(self.parse_expression()?);
                        if self.current_token.kind != TokenKind::Comma {
                            break;
                        }
                        self.next_token(); // Skip comma
                    }
                }

                self.expect_token(TokenKind::RBracket)?;
                Expression::List(elements)
            }
            TokenKind::LBrace => {
                self.next_token(); // Skip '{'
                let mut pairs = Vec::new();

                if self.current_token.kind != TokenKind::RBrace {
                    loop {
                        let key = self.parse_identifier()?;
                        self.expect_token(TokenKind::Colon)?;
                        let value = self.parse_expression()?;
                        pairs.push((key, value));

                        if self.current_token.kind != TokenKind::Comma {
                            break;
                        }
                        self.next_token(); // Skip comma
                    }
                }

                self.expect_token(TokenKind::RBrace)?;
                Expression::Map(pairs)
            }
            TokenKind::Identifier(name) => {
                // Check if it's a function call
                if self.peek_token() == TokenKind::LParen {
                    let func_name = name.clone();
                    self.next_token(); // Skip identifier
                    self.expect_token(TokenKind::LParen)?;

                    let mut args = Vec::new();
                    if self.current_token.kind != TokenKind::RParen {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.current_token.kind != TokenKind::Comma {
                                break;
                            }
                            self.next_token(); // Skip comma
                        }
                    }
                    self.expect_token(TokenKind::RParen)?;

                    Expression::FunctionCall(FunctionCall {
                        name: func_name,
                        args,
                        distinct: false, // For now, no DISTINCT
                    })
                } else {
                    // It's a variable or property access
                    let var_name = name.clone();
                    self.next_token(); // Skip identifier

                    // Check if it's followed by a dot for property access
                    if self.current_token.kind == TokenKind::Dot {
                        self.next_token();
                        let prop_name = self.parse_identifier()?;
                        Expression::PropertyAccess(
                            Box::new(Expression::Variable(var_name)),
                            prop_name,
                        )
                    } else {
                        Expression::Variable(var_name)
                    }
                }
            }
            _ => {
                return Err(ParseError::syntax_error(
                    format!(
                        "Unexpected token in expression: {:?}",
                        self.current_token.kind
                    ),
                    self.current_token.line,
                    self.current_token.column,
                ));
            }
        };

        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_match() {
        let input = "MATCH (n) RETURN n";
        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(result.is_ok());
        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            Statement::Match(match_stmt) => {
                assert_eq!(match_stmt.clauses.len(), 2); // Match and Return clauses
            }
            _ => panic!("Expected Match statement"),
        }
    }

    #[test]
    fn test_parse_create_node() {
        let input = "CREATE VERTEX (Person) SET {name: 'Alice', age: 30}";
        let mut parser = Parser::new(input);
        let result = parser.parse();

        assert!(result.is_ok());
        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);

        match &statements[0] {
            Statement::CreateNode(create_node) => {
                assert_eq!(create_node.tags.len(), 1);
                assert_eq!(create_node.tags[0].name, "Person");
                assert_eq!(create_node.properties.len(), 2);
            }
            _ => panic!("Expected CreateNode statement"),
        }
    }

    #[test]
    fn test_parse_simple_expression() {
        let input = "RETURN 1 + 2 * 3";
        let mut parser = Parser::new(input);
        // Skip to the RETURN part
        assert!(matches!(parser.parse(), Ok(stmts) if stmts.len() == 1));
    }
}
