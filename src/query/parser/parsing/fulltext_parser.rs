//! Full-Text Search Parser
//!
//! This module implements the parser for full-text search SQL statements,
//! including CREATE FULLTEXT INDEX, SEARCH, and related queries.

use crate::query::parser::ast::{
    AlterFulltextIndex, AlterIndexAction, CreateFulltextIndex, DescribeFulltextIndex,
    DropFulltextIndex, FulltextMatchCondition, FulltextQueryExpr, IndexFieldDef, IndexOptions,
    LookupFulltext, MatchFulltext, OrderClause, OrderDirection, OrderItem, SearchStatement,
    ShowFulltextIndex, WhereClause, WhereCondition, YieldClause, YieldExpression, YieldItem,
    BM25Options, InversearchOptions,
};
use crate::query::parser::parsing::parse_context::ParseContext;
use crate::query::parser::parsing::parser::{Parser, ParserResult};
use crate::core::types::FulltextEngineType;
use crate::core::Value;

/// Full-text search parser
pub struct FulltextParser<'a> {
    ctx: &'a mut ParseContext,
}

impl<'a> FulltextParser<'a> {
    /// Create a new full-text parser
    pub fn new(ctx: &'a mut ParseContext) -> Self {
        Self { ctx }
    }

    /// Parse full-text search statements
    pub fn parse(&mut self) -> ParserResult {
        if self.ctx.check_keyword("CREATE") {
            return self.parse_create_fulltext_index();
        } else if self.ctx.check_keyword("DROP") {
            return self.parse_drop_fulltext_index();
        } else if self.ctx.check_keyword("ALTER") {
            return self.parse_alter_fulltext_index();
        } else if self.ctx.check_keyword("SHOW") {
            return self.parse_show_fulltext_index();
        } else if self.ctx.check_keyword("DESCRIBE") || self.ctx.check_keyword("DESC") {
            return self.parse_describe_fulltext_index();
        } else if self.ctx.check_keyword("SEARCH") {
            return self.parse_search_statement();
        } else if self.ctx.check_keyword("LOOKUP") {
            return self.parse_lookup_fulltext();
        } else if self.ctx.check_keyword("MATCH") {
            return self.parse_match_fulltext();
        }

        ParserResult::error("Not a full-text search statement")
    }

    /// Parse CREATE FULLTEXT INDEX statement
    fn parse_create_fulltext_index(&mut self) -> ParserResult {
        self.ctx.consume_keyword("CREATE")?;
        
        let if_not_exists = if self.ctx.check_keyword("IF") {
            self.ctx.consume_keyword("IF")?;
            self.ctx.consume_keyword("NOT")?;
            self.ctx.consume_keyword("EXISTS")?;
            true
        } else {
            false
        };

        self.ctx.consume_keyword("FULLTEXT")?;
        self.ctx.consume_keyword("INDEX")?;

        let index_name = self.ctx.consume_identifier()?;
        self.ctx.consume_keyword("ON")?;
        let schema_name = self.ctx.consume_identifier()?;

        self.ctx.consume_token("(")?;
        let mut fields = Vec::new();

        loop {
            let field_name = self.ctx.consume_identifier()?;
            
            let mut field_def = IndexFieldDef::new(field_name);

            if self.ctx.check_keyword("ANALYZER") {
                self.ctx.consume_keyword("ANALYZER")?;
                field_def.analyzer = Some(self.ctx.consume_string()?);
            }

            if self.ctx.check_keyword("BOOST") {
                self.ctx.consume_keyword("BOOST")?;
                field_def.boost = Some(self.ctx.consume_float()?);
            }

            fields.push(field_def);

            if !self.ctx.consume_optional_token(",") {
                break;
            }
        }

        self.ctx.consume_token(")")?;

        // Parse engine type
        self.ctx.consume_keyword("ENGINE")?;
        let engine_type = if self.ctx.check_keyword("BM25") {
            self.ctx.consume_keyword("BM25")?;
            FulltextEngineType::Bm25
        } else if self.ctx.check_keyword("INVERSEARCH") {
            self.ctx.consume_keyword("INVERSEARCH")?;
            FulltextEngineType::Inversearch
        } else {
            return ParserResult::error("Expected BM25 or INVERSEARCH engine type");
        };

        // Parse options
        let mut options = IndexOptions::default();
        
        if self.ctx.check_keyword("OPTIONS") {
            self.ctx.consume_keyword("OPTIONS")?;
            self.ctx.consume_token("(")?;

            loop {
                let key = self.ctx.consume_identifier()?;
                self.ctx.consume_token("=")?;
                
                match key.to_lowercase().as_str() {
                    // BM25 options
                    "k1" => {
                        if options.bm25_config.is_none() {
                            options.bm25_config = Some(BM25Options::default());
                        }
                        options.bm25_config.as_mut().unwrap().k1 = Some(self.ctx.consume_float()?);
                    }
                    "b" => {
                        if options.bm25_config.is_none() {
                            options.bm25_config = Some(BM25Options::default());
                        }
                        options.bm25_config.as_mut().unwrap().b = Some(self.ctx.consume_float()?);
                    }
                    "analyzer" => {
                        if options.bm25_config.is_none() {
                            options.bm25_config = Some(BM25Options::default());
                        }
                        options.bm25_config.as_mut().unwrap().analyzer = Some(self.ctx.consume_string()?);
                    }
                    // Inversearch options
                    "tokenize_mode" => {
                        if options.inversearch_config.is_none() {
                            options.inversearch_config = Some(InversearchOptions::default());
                        }
                        options.inversearch_config.as_mut().unwrap().tokenize_mode = Some(self.ctx.consume_string()?);
                    }
                    "resolution" => {
                        if options.inversearch_config.is_none() {
                            options.inversearch_config = Some(InversearchOptions::default());
                        }
                        options.inversearch_config.as_mut().unwrap().resolution = Some(self.ctx.consume_int()? as usize);
                    }
                    "depth" => {
                        if options.inversearch_config.is_none() {
                            options.inversearch_config = Some(InversearchOptions::default());
                        }
                        options.inversearch_config.as_mut().unwrap().depth = Some(self.ctx.consume_int()? as usize);
                    }
                    _ => {
                        // Store as common option
                        let value = self.ctx.consume_value()?;
                        options.common_options.insert(key, value);
                    }
                }

                if !self.ctx.consume_optional_token(",") {
                    break;
                }
            }

            self.ctx.consume_token(")")?;
        }

        let mut create = CreateFulltextIndex::new(index_name, schema_name, fields, engine_type);
        create.if_not_exists = if_not_exists;
        create.options = options;

        ParserResult::success(Box::new(create))
    }

    /// Parse DROP FULLTEXT INDEX statement
    fn parse_drop_fulltext_index(&mut self) -> ParserResult {
        self.ctx.consume_keyword("DROP")?;
        self.ctx.consume_keyword("FULLTEXT")?;
        self.ctx.consume_keyword("INDEX")?;

        let if_exists = if self.ctx.check_keyword("IF") {
            self.ctx.consume_keyword("IF")?;
            self.ctx.consume_keyword("EXISTS")?;
            true
        } else {
            false
        };

        let index_name = self.ctx.consume_identifier()?;

        let drop = DropFulltextIndex {
            index_name,
            if_exists,
        };

        ParserResult::success(Box::new(drop))
    }

    /// Parse ALTER FULLTEXT INDEX statement
    fn parse_alter_fulltext_index(&mut self) -> ParserResult {
        self.ctx.consume_keyword("ALTER")?;
        self.ctx.consume_keyword("FULLTEXT")?;
        self.ctx.consume_keyword("INDEX")?;

        let index_name = self.ctx.consume_identifier()?;
        let mut actions = Vec::new();

        loop {
            if self.ctx.check_keyword("ADD") {
                self.ctx.consume_keyword("ADD")?;
                self.ctx.consume_keyword("FIELD")?;
                
                let field_name = self.ctx.consume_identifier()?;
                let mut field_def = IndexFieldDef::new(field_name);

                if self.ctx.check_keyword("ANALYZER") {
                    self.ctx.consume_keyword("ANALYZER")?;
                    field_def.analyzer = Some(self.ctx.consume_string()?);
                }

                actions.push(AlterIndexAction::AddField(field_def));
            } else if self.ctx.check_keyword("DROP") {
                self.ctx.consume_keyword("DROP")?;
                self.ctx.consume_keyword("FIELD")?;
                let field_name = self.ctx.consume_identifier()?;
                actions.push(AlterIndexAction::DropField(field_name));
            } else if self.ctx.check_keyword("SET") {
                self.ctx.consume_keyword("SET")?;
                let key = self.ctx.consume_identifier()?;
                self.ctx.consume_token("=")?;
                let value = self.ctx.consume_value()?;
                actions.push(AlterIndexAction::SetOption(key, value));
            } else if self.ctx.check_keyword("REBUILD") {
                self.ctx.consume_keyword("REBUILD")?;
                actions.push(AlterIndexAction::Rebuild);
            } else if self.ctx.check_keyword("OPTIMIZE") {
                self.ctx.consume_keyword("OPTIMIZE")?;
                actions.push(AlterIndexAction::Optimize);
            } else {
                return ParserResult::error("Expected ALTER INDEX action");
            }

            if !self.ctx.consume_optional_token(",") {
                break;
            }
        }

        let alter = AlterFulltextIndex {
            index_name,
            actions,
        };

        ParserResult::success(Box::new(alter))
    }

    /// Parse SHOW FULLTEXT INDEX statement
    fn parse_show_fulltext_index(&mut self) -> ParserResult {
        self.ctx.consume_keyword("SHOW")?;
        self.ctx.consume_keyword("FULLTEXT")?;
        self.ctx.consume_keyword("INDEX")?;

        let mut pattern = None;
        let mut from_schema = None;

        if self.ctx.check_keyword("LIKE") {
            self.ctx.consume_keyword("LIKE")?;
            pattern = Some(self.ctx.consume_string()?);
        }

        if self.ctx.check_keyword("FROM") || self.ctx.check_keyword("IN") {
            self.ctx.consume_keyword("FROM")?;
            from_schema = Some(self.ctx.consume_identifier()?);
        }

        let show = ShowFulltextIndex {
            pattern,
            from_schema,
        };

        ParserResult::success(Box::new(show))
    }

    /// Parse DESCRIBE FULLTEXT INDEX statement
    fn parse_describe_fulltext_index(&mut self) -> ParserResult {
        self.ctx.consume_keyword("DESCRIBE")?;
        self.ctx.consume_keyword("FULLTEXT")?;
        self.ctx.consume_keyword("INDEX")?;

        let index_name = self.ctx.consume_identifier()?;

        let describe = DescribeFulltextIndex {
            index_name,
        };

        ParserResult::success(Box::new(describe))
    }

    /// Parse SEARCH statement
    fn parse_search_statement(&mut self) -> ParserResult {
        self.ctx.consume_keyword("SEARCH")?;
        self.ctx.consume_keyword("INDEX")?;

        let index_name = self.ctx.consume_identifier()?;
        self.ctx.consume_keyword("MATCH")?;

        let query = self.parse_fulltext_query_expr()?;

        let mut search = SearchStatement::new(index_name, query);

        // Parse YIELD clause
        if self.ctx.check_keyword("YIELD") {
            self.ctx.consume_keyword("YIELD")?;
            let yield_clause = self.parse_yield_clause()?;
            search.yield_clause = Some(yield_clause);
        }

        // Parse WHERE clause
        if self.ctx.check_keyword("WHERE") {
            self.ctx.consume_keyword("WHERE")?;
            let where_clause = self.parse_where_clause()?;
            search.where_clause = Some(where_clause);
        }

        // Parse ORDER BY clause
        if self.ctx.check_keyword("ORDER") {
            self.ctx.consume_keyword("ORDER")?;
            self.ctx.consume_keyword("BY")?;
            let order_clause = self.parse_order_clause()?;
            search.order_clause = Some(order_clause);
        }

        // Parse LIMIT
        if self.ctx.check_keyword("LIMIT") {
            self.ctx.consume_keyword("LIMIT")?;
            search.limit = Some(self.ctx.consume_int()? as usize);
        }

        // Parse OFFSET
        if self.ctx.check_keyword("OFFSET") {
            self.ctx.consume_keyword("OFFSET")?;
            let offset = self.ctx.consume_int()? as usize;
            search.offset = Some(offset);
        }

        ParserResult::success(Box::new(search))
    }

    /// Parse full-text query expression
    fn parse_fulltext_query_expr(&mut self) -> ParserResult {
        // Simple text query
        if let Some(text) = self.ctx.try_consume_string() {
            return ParserResult::success(Box::new(FulltextQueryExpr::Simple(text)));
        }

        // Field-specific query
        if self.ctx.peek_token().is_identifier() {
            let field = self.ctx.consume_identifier()?;
            if self.ctx.consume_optional_token(":") {
                let query = self.ctx.consume_string()?;
                return ParserResult::success(Box::new(FulltextQueryExpr::Field(field, query)));
            }
        }

        // Phrase query
        if let Some(text) = self.ctx.try_consume_quoted_string() {
            return ParserResult::success(Box::new(FulltextQueryExpr::Phrase(text)));
        }

        ParserResult::error("Expected full-text query expression")
    }

    /// Parse YIELD clause
    fn parse_yield_clause(&mut self) -> Result<YieldClause, crate::query::parser::ParseError> {
        let mut items = Vec::new();

        loop {
            let expr = if self.ctx.check_keyword("score") {
                self.ctx.consume_identifier()?;
                YieldExpression::Score(None)
            } else if self.ctx.check_keyword("highlight") {
                self.ctx.consume_identifier()?;
                self.ctx.consume_token("(")?;
                let field = self.ctx.consume_identifier()?;
                
                let mut params = None;
                if self.ctx.consume_optional_token(",") {
                    // Parse optional parameters
                    params = None; // Simplified for now
                }
                
                self.ctx.consume_token(")")?;
                YieldExpression::Highlight(field, params)
            } else if self.ctx.check_keyword("matched_fields") {
                self.ctx.consume_identifier()?;
                YieldExpression::MatchedFields
            } else if self.ctx.consume_optional_token("*") {
                YieldExpression::All
            } else {
                let field = self.ctx.consume_identifier()?;
                YieldExpression::Field(field)
            };

            let alias = if self.ctx.check_keyword("AS") {
                self.ctx.consume_keyword("AS")?;
                Some(self.ctx.consume_identifier()?)
            } else {
                None
            };

            items.push(YieldItem { expr, alias });

            if !self.ctx.consume_optional_token(",") {
                break;
            }
        }

        Ok(YieldClause::new(items))
    }

    /// Parse WHERE clause
    fn parse_where_clause(&mut self) -> Result<WhereClause, crate::query::parser::ParseError> {
        let condition = self.parse_where_condition()?;
        Ok(WhereClause { condition: crate::query::parser::ast::ContextualExpression::new(
            crate::query::parser::ast::Expression::Boolean(true),
            crate::query::parser::ast::Span::default(),
        )})
    }

    /// Parse WHERE condition
    fn parse_where_condition(&mut self) -> Result<WhereCondition, crate::query::parser::ParseError> {
        // Simplified implementation
        if self.ctx.check_keyword("score") {
            self.ctx.consume_identifier()?;
            let op = self.parse_comparison_op()?;
            let value = self.ctx.consume_value()?;
            Ok(WhereCondition::Comparison("score".to_string(), op, value))
        } else {
            // Default to simple condition
            Ok(WhereCondition::Comparison(
                "field".to_string(),
                crate::query::parser::ast::ComparisonOp::Eq,
                Value::Bool(true),
            ))
        }
    }

    /// Parse comparison operator
    fn parse_comparison_op(&mut self) -> Result<crate::query::parser::ast::ComparisonOp, crate::query::parser::ParseError> {
        if self.ctx.consume_optional_token("=") {
            Ok(crate::query::parser::ast::ComparisonOp::Eq)
        } else if self.ctx.consume_optional_token("!=") {
            Ok(crate::query::parser::ast::ComparisonOp::Ne)
        } else if self.ctx.consume_optional_token("<") {
            Ok(crate::query::parser::ast::ComparisonOp::Lt)
        } else if self.ctx.consume_optional_token("<=") {
            Ok(crate::query::parser::ast::ComparisonOp::Le)
        } else if self.ctx.consume_optional_token(">") {
            Ok(crate::query::parser::ast::ComparisonOp::Gt)
        } else if self.ctx.consume_optional_token(">=") {
            Ok(crate::query::parser::ast::ComparisonOp::Ge)
        } else {
            Err(crate::query::parser::ParseError::new(
                crate::query::parser::ParseErrorType::SyntaxError,
                "Expected comparison operator".to_string(),
            ))
        }
    }

    /// Parse ORDER BY clause
    fn parse_order_clause(&mut self) -> Result<OrderClause, crate::query::parser::ParseError> {
        let mut items = Vec::new();

        loop {
            let expr = self.ctx.consume_identifier()?;
            let order = if self.ctx.check_keyword("ASC") {
                self.ctx.consume_keyword("ASC")?;
                OrderDirection::Asc
            } else if self.ctx.check_keyword("DESC") {
                self.ctx.consume_keyword("DESC")?;
                OrderDirection::Desc
            } else {
                OrderDirection::Asc
            };

            items.push(OrderItem { expr, order });

            if !self.ctx.consume_optional_token(",") {
                break;
            }
        }

        Ok(OrderClause { items })
    }

    /// Parse LOOKUP FULLTEXT statement
    fn parse_lookup_fulltext(&mut self) -> ParserResult {
        self.ctx.consume_keyword("LOOKUP")?;
        self.ctx.consume_keyword("ON")?;
        
        let schema_name = self.ctx.consume_identifier()?;
        self.ctx.consume_keyword("INDEX")?;
        let index_name = self.ctx.consume_identifier()?;
        
        self.ctx.consume_keyword("WHERE")?;
        let query = self.ctx.consume_string()?;

        let mut lookup = LookupFulltext {
            schema_name,
            index_name,
            query,
            yield_clause: None,
            limit: None,
        };

        if self.ctx.check_keyword("YIELD") {
            self.ctx.consume_keyword("YIELD")?;
            lookup.yield_clause = Some(self.parse_yield_clause()?);
        }

        if self.ctx.check_keyword("LIMIT") {
            self.ctx.consume_keyword("LIMIT")?;
            lookup.limit = Some(self.ctx.consume_int()? as usize);
        }

        ParserResult::success(Box::new(lookup))
    }

    /// Parse MATCH with full-text
    fn parse_match_fulltext(&mut self) -> ParserResult {
        // Simplified implementation
        self.ctx.consume_keyword("MATCH")?;
        let pattern = self.ctx.consume_string()?;
        
        self.ctx.consume_keyword("WHERE")?;
        self.ctx.consume_keyword("FULLTEXT_MATCH")?;
        self.ctx.consume_token("(")?;
        let field = self.ctx.consume_identifier()?;
        self.ctx.consume_token(",")?;
        let query = self.ctx.consume_string()?;
        self.ctx.consume_token(")")?;

        let condition = FulltextMatchCondition {
            field,
            query,
            index_name: None,
        };

        let mut match_stmt = MatchFulltext {
            pattern,
            fulltext_condition: condition,
            yield_clause: None,
        };

        if self.ctx.check_keyword("YIELD") {
            self.ctx.consume_keyword("YIELD")?;
            match_stmt.yield_clause = Some(self.parse_yield_clause()?);
        }

        ParserResult::success(Box::new(match_stmt))
    }
}

impl IndexFieldDef {
    fn new(field_name: String) -> Self {
        Self {
            field_name,
            analyzer: None,
            boost: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::parsing::parser::Parser;

    #[test]
    fn test_parse_create_fulltext_index() {
        let sql = r#"CREATE FULLTEXT INDEX idx_article_content 
                     ON article(title, content)
                     ENGINE BM25"#;
        
        let mut parser = Parser::new(sql);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_search_statement() {
        let sql = r#"SEARCH INDEX idx_article MATCH 'database'
                     YIELD doc_id, score() AS s
                     LIMIT 10"#;
        
        let mut parser = Parser::new(sql);
        let result = parser.parse();
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_drop_index() {
        let sql = "DROP FULLTEXT INDEX idx_article";
        
        let mut parser = Parser::new(sql);
        let result = parser.parse();
        assert!(result.is_ok());
    }
}
