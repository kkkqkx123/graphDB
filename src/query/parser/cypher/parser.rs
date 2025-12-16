//! Cypher解析器

use super::ast::*;
use std::collections::HashMap;

/// Cypher解析器
#[derive(Debug)]
pub struct CypherParser {
    input: String,
    position: usize,
}

impl CypherParser {
    /// 创建新的Cypher解析器
    pub fn new(input: String) -> Self {
        Self { input, position: 0 }
    }

    /// 解析Cypher查询
    pub fn parse(&mut self) -> Result<Vec<CypherStatement>, String> {
        let mut statements = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }

            let current_pos = self.position;
            let statement = self.parse_statement();
            
            match statement {
                Ok(stmt) => {
                    statements.push(stmt);
                }
                Err(e) => {
                    eprintln!("解析错误: {} at position {}", e, current_pos);
                    eprintln!("剩余输入: '{}'", &self.input[self.position..]);
                    return Err(e);
                }
            }

            // 跳过语句分隔符
            self.skip_whitespace();
            if self.peek_char() == Some(';') {
                self.consume_char();
            }
        }

        if statements.is_empty() {
            Err("没有有效的Cypher语句".to_string())
        } else {
            Ok(statements)
        }
    }

    /// 解析单个语句
    fn parse_statement(&mut self) -> Result<CypherStatement, String> {
        self.skip_whitespace();

        let keyword = self.parse_keyword()?;
        
        match keyword.to_uppercase().as_str() {
            "MATCH" => {
                let patterns = self.parse_patterns()?;
                
                let where_clause = if self.peek_keyword("WHERE") {
                    Some(self.parse_where_clause()?)
                } else {
                    None
                };
                
                // 检查是否有后续的RETURN或WITH子句
                if self.peek_keyword("RETURN") || self.peek_keyword("WITH") {
                    let return_clause = if self.peek_keyword("RETURN") {
                        self.parse_keyword()?; // 跳过RETURN
                        Some(self.parse_return_clause()?)
                    } else {
                        None
                    };
                    
                    let with_clause = if self.peek_keyword("WITH") {
                        self.parse_keyword()?; // 跳过WITH
                        Some(self.parse_with_clause()?)
                    } else {
                        None
                    };
                    
                    Ok(CypherStatement::Query(QueryClause {
                        match_clause: Some(MatchClause {
                            patterns,
                            where_clause,
                        }),
                        where_clause: None,
                        return_clause,
                        with_clause,
                    }))
                } else {
                    Ok(CypherStatement::Match(MatchClause {
                        patterns,
                        where_clause,
                    }))
                }
            }
            "RETURN" => {
                let return_items = self.parse_return_items()?;
                let distinct = self.peek_keyword("DISTINCT");
                if distinct {
                    self.parse_keyword()?; // 跳过DISTINCT
                }

                let order_by = if self.peek_keyword("ORDER") {
                    Some(self.parse_order_by_clause()?)
                } else {
                    None
                };

                let skip = if self.peek_keyword("SKIP") {
                    Some(self.parse_skip_clause()?)
                } else {
                    None
                };

                let limit = if self.peek_keyword("LIMIT") {
                    Some(self.parse_limit_clause()?)
                } else {
                    None
                };

                Ok(CypherStatement::Return(ReturnClause {
                    return_items,
                    distinct,
                    order_by,
                    skip,
                    limit,
                }))
            }
            "CREATE" => {
                let patterns = self.parse_patterns()?;
                Ok(CypherStatement::Create(CreateClause { patterns }))
            }
            "DELETE" => {
                let detach = self.peek_keyword("DETACH");
                if detach {
                    self.parse_keyword()?; // 跳过DETACH
                }
                let expressions = self.parse_expressions()?;
                Ok(CypherStatement::Delete(DeleteClause {
                    expressions,
                    detach,
                }))
            }
            "SET" => {
                let items = self.parse_set_items()?;
                Ok(CypherStatement::Set(SetClause { items }))
            }
            "REMOVE" => {
                let items = self.parse_remove_items()?;
                Ok(CypherStatement::Remove(RemoveClause { items }))
            }
            "MERGE" => {
                let pattern = self.parse_pattern()?;
                let actions = self.parse_merge_actions()?;
                Ok(CypherStatement::Merge(MergeClause { pattern, actions }))
            }
            "WITH" => {
                let return_items = self.parse_return_items()?;
                let distinct = self.peek_keyword("DISTINCT");
                if distinct {
                    self.parse_keyword()?;
                }

                let where_clause = if self.peek_keyword("WHERE") {
                    Some(self.parse_where_clause()?)
                } else {
                    None
                };

                let order_by = if self.peek_keyword("ORDER") {
                    Some(self.parse_order_by_clause()?)
                } else {
                    None
                };

                let skip = if self.peek_keyword("SKIP") {
                    Some(self.parse_skip_clause()?)
                } else {
                    None
                };

                let limit = if self.peek_keyword("LIMIT") {
                    Some(self.parse_limit_clause()?)
                } else {
                    None
                };

                Ok(CypherStatement::With(WithClause {
                    return_items,
                    where_clause,
                    distinct,
                    order_by,
                    skip,
                    limit,
                }))
            }
            "UNWIND" => {
                let expression = self.parse_expression()?;
                self.expect_keyword("AS")?;
                let variable = self.parse_identifier()?;
                Ok(CypherStatement::Unwind(UnwindClause {
                    expression,
                    variable,
                }))
            }
            "CALL" => {
                let procedure = self.parse_identifier()?;
                let arguments = if self.peek_char() == Some('(') {
                    self.parse_function_arguments()?
                } else {
                    Vec::new()
                };

                let yield_items = if self.peek_keyword("YIELD") {
                    self.parse_keyword()?; // 跳过YIELD
                    Some(self.parse_yield_items()?)
                } else {
                    None
                };

                Ok(CypherStatement::Call(CallClause {
                    procedure,
                    arguments,
                    yield_items,
                }))
            }
            _ => Err(format!("不支持的Cypher关键字: {}", keyword)),
        }
    }

    // 基础解析方法
    fn parse_keyword(&mut self) -> Result<String, String> {
        self.skip_whitespace();
        let mut keyword = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphabetic() {
                keyword.push(ch);
                self.consume_char();
            } else {
                break;
            }
        }

        if keyword.is_empty() {
            Err("期望关键字".to_string())
        } else {
            Ok(keyword)
        }
    }

    fn parse_identifier(&mut self) -> Result<String, String> {
        self.skip_whitespace();
        let mut identifier = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' {
                identifier.push(ch);
                self.consume_char();
            } else {
                break;
            }
        }

        if identifier.is_empty() {
            Err("期望标识符".to_string())
        } else {
            Ok(identifier)
        }
    }

    fn parse_patterns(&mut self) -> Result<Vec<Pattern>, String> {
        let mut patterns = Vec::new();

        while !self.is_eof() {
            // 跳过空白字符
            self.skip_whitespace();
            
            // 先检查是否能够解析模式（当前位置是否是'('）
            if self.peek_char() != Some('(') {
                break;
            }
            
            let pattern = self.parse_pattern()?;
            patterns.push(pattern);

            // 跳过空白字符后检查逗号
            self.skip_whitespace();
            if self.peek_char() == Some(',') {
                self.consume_char();
            } else {
                break;
            }
        }

        Ok(patterns)
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        let parts = self.parse_pattern_parts()?;
        Ok(Pattern { parts })
    }

    fn parse_pattern_parts(&mut self) -> Result<Vec<PatternPart>, String> {
        let mut parts = Vec::new();

        while !self.is_eof() && self.peek_char() == Some('(') {
            let node = self.parse_node_pattern()?;
            let relationships = self.parse_relationships()?;
            parts.push(PatternPart {
                node,
                relationships,
            });
        }

        Ok(parts)
    }

    fn parse_node_pattern(&mut self) -> Result<NodePattern, String> {
        self.expect_char('(')?;

        let variable = if self.peek_char().map(|c| c.is_alphabetic()).unwrap_or(false) {
            Some(self.parse_identifier()?)
        } else {
            None
        };

        // 跳过空白字符后再解析标签
        self.skip_whitespace();
        let labels = self.parse_labels()?;

        // 跳过空白字符后再解析属性
        self.skip_whitespace();
        let properties = self.parse_properties()?;

        self.expect_char(')')?;

        Ok(NodePattern {
            variable,
            labels,
            properties,
        })
    }

    fn parse_relationships(&mut self) -> Result<Vec<RelationshipPattern>, String> {
        let mut relationships = Vec::new();

        while self.peek_char() == Some('-') || self.peek_char() == Some('<') {
            let relationship = self.parse_relationship_pattern()?;
            relationships.push(relationship);
        }

        Ok(relationships)
    }

    fn parse_relationship_pattern(&mut self) -> Result<RelationshipPattern, String> {
        let direction = self.parse_direction()?;

        self.expect_char('[')?;

        let variable = if self.peek_char().map(|c| c.is_alphabetic()).unwrap_or(false) {
            Some(self.parse_identifier()?)
        } else {
            None
        };

        let types = self.parse_types()?;
        let properties = self.parse_properties()?;
        let range = self.parse_range()?;

        self.expect_char(']')?;

        Ok(RelationshipPattern {
            direction,
            variable,
            types,
            properties,
            range,
        })
    }

    fn parse_direction(&mut self) -> Result<Direction, String> {
        if self.peek_char() == Some('<') {
            self.consume_char();
            self.expect_char('-')?;
            Ok(Direction::Left)
        } else if self.peek_char() == Some('-') {
            self.consume_char();
            if self.peek_char() == Some('>') {
                self.consume_char();
                Ok(Direction::Right)
            } else {
                Ok(Direction::Both)
            }
        } else {
            Err("期望关系方向".to_string())
        }
    }

    fn parse_labels(&mut self) -> Result<Vec<String>, String> {
        let mut labels = Vec::new();

        while self.peek_char() == Some(':') {
            self.consume_char();
            let label = self.parse_identifier()?;
            labels.push(label);
        }

        Ok(labels)
    }

    fn parse_types(&mut self) -> Result<Vec<String>, String> {
        let mut types = Vec::new();

        while self.peek_char() == Some(':') {
            self.consume_char();
            let type_name = self.parse_identifier()?;
            types.push(type_name);
        }

        Ok(types)
    }

    fn parse_properties(&mut self) -> Result<Option<HashMap<String, Expression>>, String> {
        if self.peek_char() == Some('{') {
            self.consume_char();
            let mut properties = HashMap::new();

            while self.peek_char() != Some('}') {
                let key = self.parse_identifier()?;
                self.expect_char(':')?;
                let value = self.parse_expression()?;
                properties.insert(key, value);

                if self.peek_char() == Some(',') {
                    self.consume_char();
                }
            }

            self.expect_char('}')?;
            Ok(Some(properties))
        } else {
            Ok(None)
        }
    }

    fn parse_range(&mut self) -> Result<Option<Range>, String> {
        if self.peek_char() == Some('*') {
            self.consume_char();

            let start = if self.peek_char().map(|c| c.is_digit(10)).unwrap_or(false) {
                Some(self.parse_integer()?)
            } else {
                None
            };

            if self.peek_char() == Some('.') {
                self.consume_char();
                self.expect_char('.')?;

                let end = if self.peek_char().map(|c| c.is_digit(10)).unwrap_or(false) {
                    Some(self.parse_integer()?)
                } else {
                    None
                };

                Ok(Some(Range { start, end }))
            } else {
                Ok(Some(Range { start, end: start }))
            }
        } else {
            Ok(None)
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        // 解析基础表达式（变量或字面量）
        self.skip_whitespace();

        if let Some(ch) = self.peek_char() {
            match ch {
                '"' | '\'' => {
                    let string = self.parse_string()?;
                    Ok(Expression::Literal(Literal::String(string)))
                }
                '0'..='9' => {
                    let integer = self.parse_integer()?;
                    Ok(Expression::Literal(Literal::Integer(integer)))
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let identifier = self.parse_identifier()?;
                    
                    // 检查是否是属性表达式
                    self.skip_whitespace();
                    if self.peek_char() == Some('.') {
                        self.consume_char(); // 消耗点号
                        let property_name = self.parse_identifier()?;
                        Ok(Expression::Property(PropertyExpression {
                            expression: Box::new(Expression::Variable(identifier)),
                            property_name,
                        }))
                    } else {
                        Ok(Expression::Variable(identifier))
                    }
                }
                _ => Err(format!("不支持的表达式字符: {}", ch)),
            }
        } else {
            Err("期望表达式".to_string())
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        let quote_char = self.peek_char().unwrap();
        self.consume_char();
        let mut string = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == quote_char {
                break;
            }
            string.push(ch);
            self.consume_char();
        }

        self.expect_char(quote_char)?;
        Ok(string)
    }

    fn parse_integer(&mut self) -> Result<i64, String> {
        let mut number = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_digit(10) {
                number.push(ch);
                self.consume_char();
            } else {
                break;
            }
        }

        number.parse().map_err(|e| format!("解析整数失败: {}", e))
    }

    // 辅助方法
    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn consume_char(&mut self) {
        self.position += 1;
    }

    fn expect_char(&mut self, expected: char) -> Result<(), String> {
        if self.peek_char() == Some(expected) {
            self.consume_char();
            Ok(())
        } else {
            Err(format!("期望字符 '{}'", expected))
        }
    }

    fn expect_keyword(&mut self, expected: &str) -> Result<(), String> {
        let keyword = self.parse_keyword()?;
        if keyword.to_uppercase() == expected.to_uppercase() {
            Ok(())
        } else {
            Err(format!("期望关键字 '{}'", expected))
        }
    }

    fn peek_keyword(&mut self, keyword: &str) -> bool {
        let current_pos = self.position;
        
        // 先跳过空白字符
        self.skip_whitespace();
        
        // 检查当前位置是否是字母字符（关键字的开始）
        if let Some(ch) = self.peek_char() {
            if !ch.is_alphabetic() {
                self.position = current_pos;
                return false;
            }
        } else {
            self.position = current_pos;
            return false;
        }
        
        // 尝试解析关键字，但如果失败则返回false
        let result = match self.parse_keyword() {
            Ok(k) => k.to_uppercase() == keyword.to_uppercase(),
            Err(_) => false
        };
        self.position = current_pos;
        result
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.consume_char();
            } else {
                break;
            }
        }
    }

    fn is_eof(&self) -> bool {
        self.position >= self.input.len()
    }

    // 解析RETURN子句
    fn parse_return_clause(&mut self) -> Result<ReturnClause, String> {
        let return_items = self.parse_return_items()?;
        let distinct = self.peek_keyword("DISTINCT");
        if distinct {
            self.parse_keyword()?; // 跳过DISTINCT
        }

        let order_by = if self.peek_keyword("ORDER") {
            Some(self.parse_order_by_clause()?)
        } else {
            None
        };

        let skip = if self.peek_keyword("SKIP") {
            Some(self.parse_skip_clause()?)
        } else {
            None
        };

        let limit = if self.peek_keyword("LIMIT") {
            Some(self.parse_limit_clause()?)
        } else {
            None
        };

        Ok(ReturnClause {
            return_items,
            distinct,
            order_by,
            skip,
            limit,
        })
    }

    // 解析WITH子句
    fn parse_with_clause(&mut self) -> Result<WithClause, String> {
        let return_items = self.parse_return_items()?;
        let distinct = self.peek_keyword("DISTINCT");
        if distinct {
            self.parse_keyword()?;
        }

        let where_clause = if self.peek_keyword("WHERE") {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        let order_by = if self.peek_keyword("ORDER") {
            Some(self.parse_order_by_clause()?)
        } else {
            None
        };

        let skip = if self.peek_keyword("SKIP") {
            Some(self.parse_skip_clause()?)
        } else {
            None
        };

        let limit = if self.peek_keyword("LIMIT") {
            Some(self.parse_limit_clause()?)
        } else {
            None
        };

        Ok(WithClause {
            return_items,
            where_clause,
            distinct,
            order_by,
            skip,
            limit,
        })
    }

    // 简化实现的其他方法
    fn parse_return_items(&mut self) -> Result<Vec<ReturnItem>, String> {
        let mut items = Vec::new();
        
        while !self.is_eof() {
            self.skip_whitespace();
            
            // 检查是否还有更多项目
            if self.peek_char().is_none() {
                break;
            }
            
            // 尝试解析表达式作为返回项目
            let expression = self.parse_expression()?;
            
            // 创建返回项目
            let item = ReturnItem {
                expression,
                alias: None, // 简化实现，不支持别名
            };
            items.push(item);
            
            // 检查是否有逗号分隔更多项目
            self.skip_whitespace();
            if self.peek_char() == Some(',') {
                self.consume_char();
            } else {
                break;
            }
        }
        
        Ok(items)
    }

    fn parse_where_clause(&mut self) -> Result<WhereClause, String> {
        // 解析WHERE后面的表达式
        let expression = self.parse_comparison_expression()?;
        Ok(WhereClause { expression })
    }

    fn parse_comparison_expression(&mut self) -> Result<Expression, String> {
        let left = self.parse_expression()?;
        
        self.skip_whitespace();
        
        // 检查是否有比较操作符
        if let Some(ch) = self.peek_char() {
            match ch {
                '>' => {
                    self.consume_char();
                    let right = self.parse_expression()?;
                    Ok(Expression::Binary(BinaryExpression {
                        left: Box::new(left),
                        operator: BinaryOperator::GreaterThan,
                        right: Box::new(right),
                    }))
                }
                '<' => {
                    self.consume_char();
                    let right = self.parse_expression()?;
                    Ok(Expression::Binary(BinaryExpression {
                        left: Box::new(left),
                        operator: BinaryOperator::LessThan,
                        right: Box::new(right),
                    }))
                }
                '=' => {
                    self.consume_char();
                    let right = self.parse_expression()?;
                    Ok(Expression::Binary(BinaryExpression {
                        left: Box::new(left),
                        operator: BinaryOperator::Equal,
                        right: Box::new(right),
                    }))
                }
                _ => Ok(left), // 如果没有操作符，返回原始表达式
            }
        } else {
            Ok(left)
        }
    }

    fn parse_order_by_clause(&mut self) -> Result<OrderByClause, String> {
        // 简化实现
        Ok(OrderByClause { items: Vec::new() })
    }

    fn parse_skip_clause(&mut self) -> Result<SkipClause, String> {
        // 简化实现
        Ok(SkipClause {
            expression: Expression::Literal(Literal::Integer(0)),
        })
    }

    fn parse_limit_clause(&mut self) -> Result<LimitClause, String> {
        // 简化实现
        Ok(LimitClause {
            expression: Expression::Literal(Literal::Integer(10)),
        })
    }

    fn parse_set_items(&mut self) -> Result<Vec<SetItem>, String> {
        let mut items = Vec::new();
        
        while !self.is_eof() {
            self.skip_whitespace();
            
            // 解析属性表达式作为left
            let left = self.parse_expression()?;
            
            self.skip_whitespace();
            self.expect_char('=')?;
            self.skip_whitespace();
            
            // 解析值作为right
            let right = self.parse_expression()?;
            
            items.push(SetItem {
                left,
                right,
            });
            
            // 检查是否有更多项目
            self.skip_whitespace();
            if self.peek_char() == Some(',') {
                self.consume_char();
            } else {
                break;
            }
        }
        
        Ok(items)
    }

    fn parse_remove_items(&mut self) -> Result<Vec<RemoveItem>, String> {
        // 简化实现
        Ok(Vec::new())
    }

    fn parse_merge_actions(&mut self) -> Result<Vec<MergeAction>, String> {
        // 简化实现
        Ok(Vec::new())
    }

    fn parse_expressions(&mut self) -> Result<Vec<Expression>, String> {
        let mut expressions = Vec::new();
        
        while !self.is_eof() {
            self.skip_whitespace();
            
            // 尝试解析节点模式作为表达式
            if self.peek_char() == Some('(') {
                let node_pattern = self.parse_node_pattern()?;
                expressions.push(Expression::Variable(
                    node_pattern.variable.unwrap_or_else(|| "node".to_string())
                ));
            } else {
                // 尝试解析普通表达式
                let expr = self.parse_expression()?;
                expressions.push(expr);
            }
            
            // 检查是否有更多表达式
            self.skip_whitespace();
            if self.peek_char() == Some(',') {
                self.consume_char();
            } else {
                break;
            }
        }
        
        Ok(expressions)
    }

    fn parse_function_arguments(&mut self) -> Result<Vec<Expression>, String> {
        // 简化实现
        Ok(Vec::new())
    }

    fn parse_yield_items(&mut self) -> Result<Vec<String>, String> {
        // 简化实现
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_match() {
        let input = "MATCH (n:Person) RETURN n".to_string();
        let mut parser = CypherParser::new(input);

        let result = parser.parse();
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].statement_type(), "MATCH");
    }

    #[test]
    fn test_parse_multiple_statements() {
        let input = "MATCH (n:Person) RETURN n; MATCH (m:User) RETURN m".to_string();
        let mut parser = CypherParser::new(input);

        let result = parser.parse();
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 2);
        assert_eq!(statements[0].statement_type(), "MATCH");
        assert_eq!(statements[1].statement_type(), "MATCH");
    }

    #[test]
    fn test_parse_create_statement() {
        let input = "CREATE (n:Person {name: \"Alice\", age: 25})".to_string();
        let mut parser = CypherParser::new(input);

        let result = parser.parse();
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].statement_type(), "CREATE");
    }

    #[test]
    fn test_parse_return_statement() {
        let input = "RETURN n.name, n.age".to_string();
        let mut parser = CypherParser::new(input);

        let result = parser.parse();
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].statement_type(), "RETURN");
    }

    #[test]
    fn test_parse_where_clause() {
        let input = "MATCH (n:Person) WHERE n.age > 30 RETURN n".to_string();
        let mut parser = CypherParser::new(input);

        let result = parser.parse();
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].statement_type(), "MATCH");
    }

    #[test]
    fn test_parse_relationship() {
        let input = "MATCH (a:Person)-[:FRIENDS_WITH]->(b:Person)".to_string();
        let mut parser = CypherParser::new(input);

        let result = parser.parse();
        assert!(result.is_ok());

        let statements = result.unwrap();
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].statement_type(), "MATCH");
    }

    #[test]
    fn test_parse_invalid_input() {
        let input = "INVALID KEYWORD".to_string();
        let mut parser = CypherParser::new(input);

        let result = parser.parse();
        assert!(result.is_err());
    }
}
