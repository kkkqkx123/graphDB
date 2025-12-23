//! Cypher子句解析器
//!
//! 提供各种Cypher子句的解析功能

use super::ast::*;
use super::parser_core::CypherParserCore;

impl CypherParserCore {
    /// 解析MATCH子句
    pub fn parse_match_clause(&mut self) -> Result<MatchClause, String> {
        self.expect_keyword("MATCH")?;

        let optional = self.is_current_keyword("OPTIONAL");
        if optional {
            self.consume_token(); // 消费 OPTIONAL
        }

        let patterns = self.parse_patterns()?;

        let where_clause = if self.is_current_keyword("WHERE") {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        Ok(MatchClause {
            patterns,
            where_clause,
            optional,
        })
    }

    /// 解析WHERE子句
    pub fn parse_where_clause(&mut self) -> Result<WhereClause, String> {
        self.expect_keyword("WHERE")?;
        let expression = self.parse_expression_full()?;
        Ok(WhereClause { expression })
    }

    /// 解析RETURN子句
    pub fn parse_return_clause(&mut self) -> Result<ReturnClause, String> {
        self.expect_keyword("RETURN")?;

        let distinct = self.is_current_keyword("DISTINCT");
        if distinct {
            self.consume_token(); // 消费 DISTINCT
        }

        let return_items = self.parse_return_items()?;

        let order_by = if self.is_current_keyword("ORDER") {
            Some(self.parse_order_by_clause()?)
        } else {
            None
        };

        let skip = if self.is_current_keyword("SKIP") {
            Some(self.parse_skip_clause()?)
        } else {
            None
        };

        let limit = if self.is_current_keyword("LIMIT") {
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

    /// 解析WITH子句
    pub fn parse_with_clause(&mut self) -> Result<WithClause, String> {
        self.expect_keyword("WITH")?;

        let distinct = self.is_current_keyword("DISTINCT");
        if distinct {
            self.consume_token(); // 消费 DISTINCT
        }

        let return_items = self.parse_return_items()?;

        let where_clause = if self.is_current_keyword("WHERE") {
            Some(self.parse_where_clause()?)
        } else {
            None
        };

        let order_by = if self.is_current_keyword("ORDER") {
            Some(self.parse_order_by_clause()?)
        } else {
            None
        };

        let skip = if self.is_current_keyword("SKIP") {
            Some(self.parse_skip_clause()?)
        } else {
            None
        };

        let limit = if self.is_current_keyword("LIMIT") {
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

    /// 解析CREATE子句
    pub fn parse_create_clause(&mut self) -> Result<CreateClause, String> {
        self.expect_keyword("CREATE")?;
        let patterns = self.parse_patterns()?;
        Ok(CreateClause { patterns })
    }

    /// 解析DELETE子句
    pub fn parse_delete_clause(&mut self) -> Result<DeleteClause, String> {
        // 检查是否是DETACH DELETE语句
        let detach = self.is_current_keyword("DETACH");
        if detach {
            self.consume_token(); // 消费 DETACH
        }

        self.expect_keyword("DELETE")?; // 消费 DELETE

        let expressions = self.parse_expressions()?;
        Ok(DeleteClause {
            expressions,
            detach,
        })
    }

    /// 解析SET子句
    pub fn parse_set_clause(&mut self) -> Result<SetClause, String> {
        self.expect_keyword("SET")?;
        let items = self.parse_set_items()?;
        Ok(SetClause { items })
    }

    /// 解析REMOVE子句
    pub fn parse_remove_clause(&mut self) -> Result<RemoveClause, String> {
        self.expect_keyword("REMOVE")?;
        let items = self.parse_remove_items()?;
        Ok(RemoveClause { items })
    }

    /// 解析MERGE子句
    pub fn parse_merge_clause(&mut self) -> Result<MergeClause, String> {
        self.expect_keyword("MERGE")?;
        let pattern = self.parse_pattern()?;
        let actions = self.parse_merge_actions()?;
        Ok(MergeClause { pattern, actions })
    }

    /// 解析UNWIND子句
    pub fn parse_unwind_clause(&mut self) -> Result<UnwindClause, String> {
        self.expect_keyword("UNWIND")?;
        let expression = self.parse_expression_full()?;
        self.expect_keyword("AS")?;
        let variable = self.parse_identifier()?;
        Ok(UnwindClause {
            expression,
            variable,
        })
    }

    /// 解析CALL子句
    pub fn parse_call_clause(&mut self) -> Result<CallClause, String> {
        self.expect_keyword("CALL")?;

        let mut procedure = self.parse_identifier()?;

        // 检查是否有命名空间
        self.skip_whitespace();
        if self.is_current_token_value(".") {
            self.consume_token(); // 消费 '.'
            let namespace = procedure;
            let procedure_name = self.parse_identifier()?;
            procedure = format!("{}.{}", namespace, procedure_name);
        }

        let arguments = if self.is_current_token_value("(") {
            self.consume_token(); // 消费 '('
            let args = self.parse_function_arguments_public()?;
            self.expect_token_value(")")?;
            args
        } else {
            Vec::new()
        };

        let yield_items = if self.is_current_keyword("YIELD") {
            self.consume_token(); // 消费 YIELD
            Some(self.parse_yield_items()?)
        } else {
            None
        };

        Ok(CallClause {
            procedure,
            arguments,
            yield_items,
        })
    }

    /// 解析返回项目
    pub fn parse_return_items(&mut self) -> Result<Vec<ReturnItem>, String> {
        let mut items = Vec::new();

        self.skip_whitespace();
        if self.is_current_token_value("*") {
            self.consume_token(); // 消费 '*'
            items.push(ReturnItem {
                expression: Expression::Variable("*".to_string()),
                alias: None,
            });
        } else {
            let expression = self.parse_expression_full()?;

            self.skip_whitespace();
            let alias = if self.is_current_keyword("AS") {
                self.consume_token(); // 消费 AS
                Some(self.parse_identifier()?)
            } else {
                None
            };

            items.push(ReturnItem { expression, alias });

            self.skip_whitespace();
            while self.is_current_token_value(",") {
                self.consume_token(); // 消费 ','
                self.skip_whitespace();

                let expression = self.parse_expression_full()?;

                self.skip_whitespace();
                let alias = if self.is_current_keyword("AS") {
                    self.consume_token(); // 消费 AS
                    Some(self.parse_identifier()?)
                } else {
                    None
                };

                items.push(ReturnItem { expression, alias });
                self.skip_whitespace();
            }
        }

        Ok(items)
    }

    /// 解析ORDER BY子句
    pub fn parse_order_by_clause(&mut self) -> Result<OrderByClause, String> {
        self.expect_keyword("ORDER")?;
        self.expect_keyword("BY")?;

        let mut items = Vec::new();

        let expression = self.parse_expression_full()?;

        self.skip_whitespace();
        let ordering = if self.is_current_keyword("ASC") {
            self.consume_token(); // 消费 ASC
            Ordering::Ascending
        } else if self.is_current_keyword("DESC") {
            self.consume_token(); // 消费 DESC
            Ordering::Descending
        } else {
            Ordering::Ascending // 默认升序
        };

        items.push(OrderByItem {
            expression,
            ordering,
        });

        self.skip_whitespace();
        while self.is_current_token_value(",") {
            self.consume_token(); // 消费 ','
            self.skip_whitespace();

            let expression = self.parse_expression_full()?;

            self.skip_whitespace();
            let ordering = if self.is_current_keyword("ASC") {
                self.consume_token(); // 消费 ASC
                Ordering::Ascending
            } else if self.is_current_keyword("DESC") {
                self.consume_token(); // 消费 DESC
                Ordering::Descending
            } else {
                Ordering::Ascending // 默认升序
            };

            items.push(OrderByItem {
                expression,
                ordering,
            });
            self.skip_whitespace();
        }

        Ok(OrderByClause { items })
    }

    /// 解析SKIP子句
    pub fn parse_skip_clause(&mut self) -> Result<SkipClause, String> {
        self.expect_keyword("SKIP")?;
        let expression = self.parse_expression_full()?;
        Ok(SkipClause { expression })
    }

    /// 解析LIMIT子句
    pub fn parse_limit_clause(&mut self) -> Result<LimitClause, String> {
        self.expect_keyword("LIMIT")?;
        let expression = self.parse_expression_full()?;
        Ok(LimitClause { expression })
    }

    /// 解析SET项目
    pub fn parse_set_items(&mut self) -> Result<Vec<SetItem>, String> {
        let mut items = Vec::new();

        loop {
            let left = self.parse_expression_full()?;
            self.skip_whitespace();

            let operator = if self.is_current_token_value("+=") {
                self.consume_token(); // 消费 '+='
                SetOperator::Add
            } else if self.is_current_token_value("-=") {
                self.consume_token(); // 消费 '-='
                SetOperator::Subtract
            } else {
                self.expect_token_value("=")?; // 消费 '='
                SetOperator::Replace
            };

            self.skip_whitespace();
            let right = self.parse_expression_full()?;

            items.push(SetItem {
                left,
                operator,
                right,
            });

            self.skip_whitespace();
            if !self.is_current_token_value(",") {
                break;
            }

            self.consume_token(); // 消费 ','
            self.skip_whitespace();
        }

        Ok(items)
    }

    /// 解析REMOVE项目
    pub fn parse_remove_items(&mut self) -> Result<Vec<RemoveItem>, String> {
        let mut items = Vec::new();

        let expression = self.parse_expression_full()?;

        // 确定移除类型
        let item_type = match &expression {
            Expression::Property(_) => RemoveItemType::Property,
            Expression::Variable(_) => RemoveItemType::Label,
            _ => return Err("不支持的REMOVE项目类型".to_string()),
        };

        items.push(RemoveItem {
            expression,
            item_type,
        });

        self.skip_whitespace();
        while self.is_current_token_value(",") {
            self.consume_token(); // 消费 ','
            self.skip_whitespace();

            let expression = self.parse_expression_full()?;

            // 确定移除类型
            let item_type = match &expression {
                Expression::Property(_) => RemoveItemType::Property,
                Expression::Variable(_) => RemoveItemType::Label,
                _ => return Err("不支持的REMOVE项目类型".to_string()),
            };

            items.push(RemoveItem {
                expression,
                item_type,
            });
            self.skip_whitespace();
        }

        Ok(items)
    }

    /// 解析MERGE动作
    pub fn parse_merge_actions(&mut self) -> Result<Vec<MergeAction>, String> {
        let mut actions = Vec::new();

        self.skip_whitespace();
        while self.is_current_keyword("ON") {
            self.consume_token(); // 消费 ON

            let action_type = if self.is_current_keyword("CREATE") {
                self.consume_token(); // 消费 CREATE
                MergeActionType::OnCreate
            } else if self.is_current_keyword("MATCH") {
                self.consume_token(); // 消费 MATCH
                MergeActionType::OnMatch
            } else {
                return Err("期望 CREATE 或 MATCH 在 ON 之后".to_string());
            };

            // 解析SET子句
            self.skip_whitespace();
            let set_items = self.parse_set_items()?;

            actions.push(MergeAction {
                action_type,
                set_items,
            });

            self.skip_whitespace();
        }

        Ok(actions)
    }

    /// 解析表达式列表
    pub fn parse_expressions(&mut self) -> Result<Vec<Expression>, String> {
        let mut expressions = Vec::new();

        self.skip_whitespace();
        let expression = self.parse_expression_full()?;
        expressions.push(expression);

        self.skip_whitespace();
        while self.is_current_token_value(",") {
            self.consume_token(); // 消费 ','
            self.skip_whitespace();
            let expression = self.parse_expression_full()?;
            expressions.push(expression);
            self.skip_whitespace();
        }

        Ok(expressions)
    }

    /// 解析YIELD项目
    pub fn parse_yield_items(&mut self) -> Result<Vec<String>, String> {
        let mut items = Vec::new();

        self.skip_whitespace();
        if self.is_current_token_value("*") {
            self.consume_token(); // 消费 '*'
            items.push("*".to_string());
        } else {
            let item = self.parse_identifier()?;
            items.push(item);

            self.skip_whitespace();
            while self.is_current_token_value(",") {
                self.consume_token(); // 消费 ','
                self.skip_whitespace();
                let item = self.parse_identifier()?;
                items.push(item);
                self.skip_whitespace();
            }
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_match_clause() {
        let mut parser = CypherParserCore::new("MATCH (n:Person) WHERE n.age > 30".to_string());
        let match_clause = parser
            .parse_match_clause()
            .expect("Clause parser should parse valid match clauses");

        assert_eq!(match_clause.patterns.len(), 1);
        assert!(match_clause.where_clause.is_some());
        assert!(!match_clause.optional);
    }

    #[test]
    fn test_parse_optional_match_clause() {
        let mut parser = CypherParserCore::new("OPTIONAL MATCH (n:Person)".to_string());
        let match_clause = parser
            .parse_match_clause()
            .expect("Clause parser should parse valid match clauses");

        assert_eq!(match_clause.patterns.len(), 1);
        assert!(match_clause.where_clause.is_none());
        assert!(match_clause.optional);
    }

    #[test]
    fn test_parse_return_clause() {
        let mut parser = CypherParserCore::new("RETURN DISTINCT n.name AS name, n.age".to_string());
        let return_clause = parser
            .parse_return_clause()
            .expect("Clause parser should parse valid return clauses");

        assert!(return_clause.distinct);
        assert_eq!(return_clause.return_items.len(), 2);
        assert_eq!(
            return_clause.return_items[0].alias,
            Some("name".to_string())
        );
        assert_eq!(return_clause.return_items[1].alias, None);
    }

    #[test]
    fn test_parse_return_all() {
        let mut parser = CypherParserCore::new("RETURN *".to_string());
        let return_clause = parser
            .parse_return_clause()
            .expect("Clause parser should parse valid return clauses");

        assert!(!return_clause.distinct);
        assert_eq!(return_clause.return_items.len(), 1);
        match &return_clause.return_items[0].expression {
            Expression::Variable(var) => assert_eq!(var, "*"),
            _ => panic!("Expected variable"),
        }
    }

    #[test]
    fn test_parse_order_by_clause() {
        let mut parser = CypherParserCore::new("ORDER BY n.name DESC, n.age ASC".to_string());
        let order_by_clause = parser
            .parse_order_by_clause()
            .expect("Clause parser should parse valid order by clauses");

        assert_eq!(order_by_clause.items.len(), 2);
        assert_eq!(order_by_clause.items[0].ordering, Ordering::Descending);
        assert_eq!(order_by_clause.items[1].ordering, Ordering::Ascending);
    }

    #[test]
    fn test_parse_create_clause() {
        let mut parser = CypherParserCore::new("CREATE (n:Person {name: \"Alice\"})".to_string());
        let create_clause = parser
            .parse_create_clause()
            .expect("Clause parser should parse valid create clauses");

        assert_eq!(create_clause.patterns.len(), 1);
    }

    #[test]
    fn test_parse_delete_clause() {
        let mut parser = CypherParserCore::new("DELETE n, m".to_string());
        let delete_clause = parser
            .parse_delete_clause()
            .expect("Clause parser should parse valid delete clauses");

        assert!(!delete_clause.detach);
        assert_eq!(delete_clause.expressions.len(), 2);
    }

    #[test]
    fn test_parse_detach_delete_clause() {
        let mut parser = CypherParserCore::new("DETACH DELETE n".to_string());
        let delete_clause = parser
            .parse_delete_clause()
            .expect("Clause parser should parse valid delete clauses");

        assert!(delete_clause.detach);
        assert_eq!(delete_clause.expressions.len(), 1);
    }

    #[test]
    fn test_parse_set_clause() {
        let mut parser = CypherParserCore::new("SET n.name = \"Alice\", n.age += 1".to_string());
        let set_clause = parser
            .parse_set_clause()
            .expect("Clause parser should parse valid set clauses");

        assert_eq!(set_clause.items.len(), 2);
        assert_eq!(set_clause.items[0].operator, SetOperator::Replace);
        assert_eq!(set_clause.items[1].operator, SetOperator::Add);
    }

    #[test]
    fn test_parse_with_clause() {
        let mut parser =
            CypherParserCore::new("WITH n.name AS name, n.age WHERE n.age > 30".to_string());
        let with_clause = parser
            .parse_with_clause()
            .expect("Clause parser should parse valid with clauses");

        assert!(!with_clause.distinct);
        assert_eq!(with_clause.return_items.len(), 2);
        assert!(with_clause.where_clause.is_some());
    }

    #[test]
    fn test_parse_unwind_clause() {
        let mut parser = CypherParserCore::new("UNWIND [1, 2, 3] AS number".to_string());
        let unwind_clause = parser
            .parse_unwind_clause()
            .expect("Clause parser should parse valid unwind clauses");

        match unwind_clause.expression {
            Expression::List(_) => {} // 验证是列表表达式
            _ => panic!("Expected list expression"),
        }
        assert_eq!(unwind_clause.variable, "number");
    }

    #[test]
    fn test_parse_call_clause() {
        let mut parser = CypherParserCore::new("CALL db.info()".to_string());
        let call_clause = parser
            .parse_call_clause()
            .expect("Clause parser should parse valid call clauses");

        assert_eq!(call_clause.procedure, "db.info");
        assert!(call_clause.arguments.is_empty());
        assert!(call_clause.yield_items.is_none());
    }

    #[test]
    fn test_parse_call_clause_with_yield() {
        let mut parser = CypherParserCore::new("CALL db.info() YIELD name, value".to_string());
        let call_clause = parser
            .parse_call_clause()
            .expect("Clause parser should parse valid call clauses");

        assert_eq!(call_clause.procedure, "db.info");
        assert!(call_clause.arguments.is_empty());
        assert!(call_clause.yield_items.is_some());

        let yield_items = call_clause
            .yield_items
            .expect("Call clause should have yield items");
        assert_eq!(yield_items.len(), 2);
        assert_eq!(yield_items[0], "name");
        assert_eq!(yield_items[1], "value");
    }
}
