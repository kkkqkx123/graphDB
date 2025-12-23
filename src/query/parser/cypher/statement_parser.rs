//! Cypher语句解析器
//!
//! 提供完整的Cypher语句解析功能

use super::ast::*;
use super::parser_core::CypherParserCore;

impl CypherParserCore {
    /// 解析单个语句
    pub fn parse_statement(&mut self) -> Result<CypherStatement, String> {
        self.skip_whitespace();

        if self.is_eof() {
            return Err("意外的文件结束".to_string());
        }

        // 检查是否是关键字
        let keyword = if self.is_current_token_type(super::lexer::TokenType::Keyword) {
            self.current_token().value.clone()
        } else if self.is_current_token_type(super::lexer::TokenType::Identifier) {
            // 标识符也可能是关键字（如RETURN）
            self.current_token().value.clone()
        } else {
            return Err(format!(
                "期望关键字，但得到 '{}' 在位置 {}",
                self.current_token().value,
                self.current_token().position
            ));
        };

        match keyword.to_uppercase().as_str() {
            "MATCH" => self.parse_match_statement(),
            "RETURN" => self.parse_return_statement(),
            "CREATE" => self.parse_create_statement(),
            "DELETE" => self.parse_delete_statement(),
            "DETACH" => self.parse_detach_statement(), // 处理DETACH DELETE语句
            "SET" => self.parse_set_statement(),
            "REMOVE" => self.parse_remove_statement(),
            "MERGE" => self.parse_merge_statement(),
            "WITH" => self.parse_with_statement(),
            "UNWIND" => self.parse_unwind_statement(),
            "CALL" => self.parse_call_statement(),
            _ => Err(format!("不支持的Cypher关键字: {}", keyword)),
        }
    }

    /// 解析DETACH语句 (如 DETACH DELETE)
    fn parse_detach_statement(&mut self) -> Result<CypherStatement, String> {
        // 先读取DETACH关键字
        self.expect_keyword("DETACH")?;

        // 检查是否是DETACH DELETE语句
        if self.is_current_keyword("DELETE") {
            // 解析DETACH DELETE语句
            let delete_clause = self.parse_delete_clause()?;
            Ok(CypherStatement::Delete(delete_clause))
        } else {
            Err(format!(
                "DETACH关键字后期望DELETE，但得到 '{}' 在位置 {}",
                self.current_token().value,
                self.current_token().position
            ))
        }
    }

    /// 解析MATCH语句
    fn parse_match_statement(&mut self) -> Result<CypherStatement, String> {
        let match_clause = self.parse_match_clause()?;

        // 检查是否有后续的RETURN或WITH子句
        self.skip_whitespace();
        if self.is_current_keyword("RETURN") || self.is_current_keyword("WITH") {
            let return_clause = if self.is_current_keyword("RETURN") {
                Some(self.parse_return_clause()?)
            } else {
                None
            };

            let with_clause = if self.is_current_keyword("WITH") {
                Some(self.parse_with_clause()?)
            } else {
                None
            };

            Ok(CypherStatement::Query(QueryClause {
                match_clause: Some(match_clause),
                where_clause: None, // WHERE子句已经在match_clause中处理
                return_clause,
                with_clause,
            }))
        } else {
            Ok(CypherStatement::Match(match_clause))
        }
    }

    /// 解析RETURN语句
    fn parse_return_statement(&mut self) -> Result<CypherStatement, String> {
        let return_clause = self.parse_return_clause()?;
        Ok(CypherStatement::Return(return_clause))
    }

    /// 解析CREATE语句
    fn parse_create_statement(&mut self) -> Result<CypherStatement, String> {
        let create_clause = self.parse_create_clause()?;
        Ok(CypherStatement::Create(create_clause))
    }

    /// 解析DELETE语句
    fn parse_delete_statement(&mut self) -> Result<CypherStatement, String> {
        let delete_clause = self.parse_delete_clause()?;
        Ok(CypherStatement::Delete(delete_clause))
    }

    /// 解析SET语句
    fn parse_set_statement(&mut self) -> Result<CypherStatement, String> {
        let set_clause = self.parse_set_clause()?;
        Ok(CypherStatement::Set(set_clause))
    }

    /// 解析REMOVE语句
    fn parse_remove_statement(&mut self) -> Result<CypherStatement, String> {
        let remove_clause = self.parse_remove_clause()?;
        Ok(CypherStatement::Remove(remove_clause))
    }

    /// 解析MERGE语句
    fn parse_merge_statement(&mut self) -> Result<CypherStatement, String> {
        let merge_clause = self.parse_merge_clause()?;
        Ok(CypherStatement::Merge(merge_clause))
    }

    /// 解析WITH语句
    fn parse_with_statement(&mut self) -> Result<CypherStatement, String> {
        let with_clause = self.parse_with_clause()?;
        Ok(CypherStatement::With(with_clause))
    }

    /// 解析UNWIND语句
    fn parse_unwind_statement(&mut self) -> Result<CypherStatement, String> {
        let unwind_clause = self.parse_unwind_clause()?;
        Ok(CypherStatement::Unwind(unwind_clause))
    }

    /// 解析CALL语句
    fn parse_call_statement(&mut self) -> Result<CypherStatement, String> {
        let call_clause = self.parse_call_clause()?;
        Ok(CypherStatement::Call(call_clause))
    }

    /// 解析复合查询语句
    pub fn parse_query_statement(&mut self) -> Result<CypherStatement, String> {
        let mut match_clause = None;
        let mut where_clause = None;
        let mut return_clause = None;
        let mut with_clause = None;

        // 解析MATCH子句
        self.skip_whitespace();
        if self.is_current_keyword("MATCH") {
            match_clause = Some(self.parse_match_clause()?);
        }

        // 解析独立的WHERE子句
        self.skip_whitespace();
        if self.is_current_keyword("WHERE") && match_clause.is_some() {
            where_clause = Some(self.parse_where_clause()?);
        }

        // 解析RETURN子句
        self.skip_whitespace();
        if self.is_current_keyword("RETURN") {
            return_clause = Some(self.parse_return_clause()?);
        }

        // 解析WITH子句
        self.skip_whitespace();
        if self.is_current_keyword("WITH") {
            with_clause = Some(self.parse_with_clause()?);
        }

        // 确保至少有一个子句
        if match_clause.is_none() && return_clause.is_none() && with_clause.is_none() {
            return Err("查询语句必须包含至少一个MATCH、RETURN或WITH子句".to_string());
        }

        Ok(CypherStatement::Query(QueryClause {
            match_clause,
            where_clause,
            return_clause,
            with_clause,
        }))
    }

    /// 解析多个语句
    pub fn parse_statements(&mut self) -> Result<Vec<CypherStatement>, String> {
        let mut statements = Vec::new();

        while !self.is_eof() {
            self.skip_whitespace();
            if self.is_eof() {
                break;
            }

            let current_pos = self.current_token().position;
            let statement = self.parse_statement();

            match statement {
                Ok(stmt) => {
                    statements.push(stmt);
                }
                Err(e) => {
                    eprintln!("解析错误: {} at position {}", e, current_pos);
                    return Err(e);
                }
            }

            // 跳过语句分隔符
            self.skip_whitespace();
            if self.is_current_token_value(";") {
                self.consume_token();
            }
        }

        if statements.is_empty() {
            Err("没有有效的Cypher语句".to_string())
        } else {
            Ok(statements)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_match_statement() {
        let mut parser = CypherParserCore::new("MATCH (n:Person)".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Match(match_clause) => {
                assert_eq!(match_clause.patterns.len(), 1);
                assert!(match_clause.where_clause.is_none());
            }
            _ => panic!("Expected Match statement"),
        }
    }

    #[test]
    fn test_parse_match_with_where_statement() {
        let mut parser = CypherParserCore::new("MATCH (n:Person) WHERE n.age > 30".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Match(match_clause) => {
                assert_eq!(match_clause.patterns.len(), 1);
                assert!(match_clause.where_clause.is_some());
            }
            _ => panic!("Expected Match statement"),
        }
    }

    #[test]
    fn test_parse_match_return_statement() {
        let mut parser = CypherParserCore::new("MATCH (n:Person) RETURN n.name".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Query(query_clause) => {
                assert!(query_clause.match_clause.is_some());
                assert!(query_clause.return_clause.is_some());
                assert!(query_clause.with_clause.is_none());
            }
            _ => panic!("Expected Query statement"),
        }
    }

    #[test]
    fn test_parse_return_statement() {
        let mut parser = CypherParserCore::new("RETURN n.name, n.age".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Return(return_clause) => {
                assert_eq!(return_clause.return_items.len(), 2);
            }
            _ => panic!("Expected Return statement"),
        }
    }

    #[test]
    fn test_parse_create_statement() {
        let mut parser = CypherParserCore::new("CREATE (n:Person {name: \"Alice\"})".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Create(create_clause) => {
                assert_eq!(create_clause.patterns.len(), 1);
            }
            _ => panic!("Expected Create statement"),
        }
    }

    #[test]
    fn test_parse_delete_statement() {
        let mut parser = CypherParserCore::new("DELETE n".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Delete(delete_clause) => {
                assert_eq!(delete_clause.expressions.len(), 1);
                assert!(!delete_clause.detach);
            }
            _ => panic!("Expected Delete statement"),
        }
    }

    #[test]
    fn test_parse_detach_delete_statement() {
        let mut parser = CypherParserCore::new("DETACH DELETE n".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Delete(delete_clause) => {
                assert_eq!(delete_clause.expressions.len(), 1);
                assert!(delete_clause.detach);
            }
            _ => panic!("Expected Delete statement"),
        }
    }

    #[test]
    fn test_parse_set_statement() {
        let mut parser = CypherParserCore::new("SET n.name = \"Alice\"".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Set(set_clause) => {
                assert_eq!(set_clause.items.len(), 1);
            }
            _ => panic!("Expected Set statement"),
        }
    }

    #[test]
    fn test_parse_remove_statement() {
        let mut parser = CypherParserCore::new("REMOVE n.name".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Remove(remove_clause) => {
                assert_eq!(remove_clause.items.len(), 1);
            }
            _ => panic!("Expected Remove statement"),
        }
    }

    #[test]
    fn test_parse_merge_statement() {
        let mut parser = CypherParserCore::new("MERGE (n:Person {name: \"Alice\"})".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Merge(merge_clause) => {
                assert_eq!(merge_clause.pattern.parts.len(), 1);
            }
            _ => panic!("Expected Merge statement"),
        }
    }

    #[test]
    fn test_parse_with_statement() {
        let mut parser = CypherParserCore::new("WITH n.name AS name".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::With(with_clause) => {
                assert_eq!(with_clause.return_items.len(), 1);
            }
            _ => panic!("Expected With statement"),
        }
    }

    #[test]
    fn test_parse_unwind_statement() {
        let mut parser = CypherParserCore::new("UNWIND [1, 2, 3] AS number".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Unwind(unwind_clause) => {
                assert_eq!(unwind_clause.variable, "number");
            }
            _ => panic!("Expected Unwind statement"),
        }
    }

    #[test]
    fn test_parse_call_statement() {
        let mut parser = CypherParserCore::new("CALL db.info()".to_string());
        let statement = parser
            .parse_statement()
            .expect("Statement parser should parse valid statements");

        match statement {
            CypherStatement::Call(call_clause) => {
                assert_eq!(call_clause.procedure, "db.info");
            }
            _ => panic!("Expected Call statement"),
        }
    }

    #[test]
    fn test_parse_multiple_statements() {
        let mut parser =
            CypherParserCore::new("MATCH (n:Person) RETURN n; MATCH (m:User) RETURN m".to_string());
        let statements = parser
            .parse_statements()
            .expect("Statement parser should parse valid statements");

        assert_eq!(statements.len(), 2);

        match &statements[0] {
            CypherStatement::Query(query_clause) => {
                assert!(query_clause.match_clause.is_some());
                assert!(query_clause.return_clause.is_some());
            }
            _ => panic!("Expected Query statement"),
        }

        match &statements[1] {
            CypherStatement::Query(query_clause) => {
                assert!(query_clause.match_clause.is_some());
                assert!(query_clause.return_clause.is_some());
            }
            _ => panic!("Expected Query statement"),
        }
    }

    #[test]
    fn test_parse_invalid_statement() {
        let mut parser = CypherParserCore::new("INVALID KEYWORD".to_string());
        let result = parser.parse_statement();
        assert!(result.is_err());
    }
}
