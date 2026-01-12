//! Cypher解析器
//!
//! 重构后的Cypher查询语言解析器，采用模块化设计

use super::ast::*;
use super::parser_core::CypherParserCore;

/// Cypher解析器
#[derive(Debug)]
pub struct CypherParser {
    core: CypherParserCore,
}

impl CypherParser {
    /// 创建新的Cypher解析器
    pub fn new(input: String) -> Self {
        Self {
            core: CypherParserCore::new(input),
        }
    }

    /// 解析Cypher查询
    pub fn parse(&mut self) -> Result<Vec<CypherStatement>, String> {
        self.core.parse_statements()
    }

    /// 解析单个语句
    pub fn parse_statement(&mut self) -> Result<CypherStatement, String> {
        self.core.parse_statement()
    }

    /// 解析查询语句
    pub fn parse_query(&mut self) -> Result<CypherStatement, String> {
        self.core.parse_query_statement()
    }

    /// 解析表达式
    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.core.parse_expression_full()
    }

    /// 解析模式
    pub fn parse_pattern(&mut self) -> Result<Pattern, String> {
        self.core.parse_pattern()
    }

    /// 解析模式列表
    pub fn parse_patterns(&mut self) -> Result<Vec<Pattern>, String> {
        self.core.parse_patterns()
    }

    /// 解析节点模式
    pub fn parse_node_pattern(&mut self) -> Result<NodePattern, String> {
        self.core.parse_node_pattern()
    }

    /// 解析关系模式
    pub fn parse_relationship_pattern(&mut self) -> Result<RelationshipPattern, String> {
        self.core.parse_relationship_pattern()
    }

    /// 解析MATCH子句
    pub fn parse_match_clause(&mut self) -> Result<MatchClause, String> {
        self.core.parse_match_clause()
    }

    /// 解析WHERE子句
    pub fn parse_where_clause(&mut self) -> Result<WhereClause, String> {
        self.core.parse_where_clause()
    }

    /// 解析RETURN子句
    pub fn parse_return_clause(&mut self) -> Result<ReturnClause, String> {
        self.core.parse_return_clause()
    }

    /// 解析WITH子句
    pub fn parse_with_clause(&mut self) -> Result<WithClause, String> {
        self.core.parse_with_clause()
    }

    /// 解析CREATE子句
    pub fn parse_create_clause(&mut self) -> Result<CreateClause, String> {
        self.core.parse_create_clause()
    }

    /// 解析DELETE子句
    pub fn parse_delete_clause(&mut self) -> Result<DeleteClause, String> {
        self.core.parse_delete_clause()
    }

    /// 解析SET子句
    pub fn parse_set_clause(&mut self) -> Result<SetClause, String> {
        self.core.parse_set_clause()
    }

    /// 解析REMOVE子句
    pub fn parse_remove_clause(&mut self) -> Result<RemoveClause, String> {
        self.core.parse_remove_clause()
    }

    /// 解析MERGE子句
    pub fn parse_merge_clause(&mut self) -> Result<MergeClause, String> {
        self.core.parse_merge_clause()
    }

    /// 解析UNWIND子句
    pub fn parse_unwind_clause(&mut self) -> Result<UnwindClause, String> {
        self.core.parse_unwind_clause()
    }

    /// 解析CALL子句
    pub fn parse_call_clause(&mut self) -> Result<CallClause, String> {
        self.core.parse_call_clause()
    }

    /// 解析ORDER BY子句
    pub fn parse_order_by_clause(&mut self) -> Result<OrderByClause, String> {
        self.core.parse_order_by_clause()
    }

    /// 解析SKIP子句
    pub fn parse_skip_clause(&mut self) -> Result<SkipClause, String> {
        self.core.parse_skip_clause()
    }

    /// 解析LIMIT子句
    pub fn parse_limit_clause(&mut self) -> Result<LimitClause, String> {
        self.core.parse_limit_clause()
    }

    /// 获取解析器状态信息
    pub fn get_parser_info(&self) -> ParserInfo {
        ParserInfo {
            current_position: self.core.current_token_index,
            total_tokens: self.core.tokens.len(),
            current_token: if self.core.current_token_index < self.core.tokens.len() {
                Some(self.core.current_token().clone())
            } else {
                None
            },
        }
    }

    /// 重置解析器到开始位置
    pub fn reset(&mut self) {
        self.core.current_token_index = 0;
    }

    /// 检查是否还有更多标记
    pub fn has_more_tokens(&self) -> bool {
        !self.core.is_eof()
    }

    /// 获取剩余的输入
    pub fn get_remaining_input(&self) -> String {
        if self.core.current_token_index < self.core.tokens.len() {
            self.core.tokens[self.core.current_token_index..]
                .iter()
                .map(|t| t.value.clone())
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        }
    }
}

/// 解析器状态信息
#[derive(Debug, Clone)]
pub struct ParserInfo {
    /// 当前标记位置
    pub current_position: usize,
    /// 总标记数
    pub total_tokens: usize,
    /// 当前标记
    pub current_token: Option<Token>,
}

/// 解析结果
#[derive(Debug, Clone)]
pub struct ParseResult<T> {
    /// 解析结果
    pub result: T,
    /// 解析警告
    pub warnings: Vec<String>,
    /// 解析信息
    pub info: ParserInfo,
}

impl<T> ParseResult<T> {
    /// 创建成功的解析结果
    pub fn success(result: T, info: ParserInfo) -> Self {
        Self {
            result,
            warnings: Vec::new(),
            info,
        }
    }

    /// 创建带警告的解析结果
    pub fn success_with_warnings(result: T, warnings: Vec<String>, info: ParserInfo) -> Self {
        Self {
            result,
            warnings,
            info,
        }
    }

    /// 添加警告
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// 获取警告数量
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }
}

// 为了兼容性，重新导出Token类型
pub use super::lexer::Token;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = CypherParser::new("MATCH (n)".to_string());
        assert!(parser.has_more_tokens());
    }

    #[test]
    fn test_parse_simple_query() {
        let mut parser = CypherParser::new("MATCH (n:Person) RETURN n".to_string());
        let result = parser.parse();
        assert!(result.is_ok());

        let statements = result.expect("Expected successful parse of simple query");
        assert_eq!(statements.len(), 1);
    }

    #[test]
    fn test_parse_multiple_statements() {
        let input = "MATCH (n:Person) RETURN n; MATCH (m:User) RETURN m".to_string();
        let mut parser = CypherParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());

        let statements = result.expect("Expected successful parse of multiple statements");
        assert_eq!(statements.len(), 2);
    }

    #[test]
    fn test_parser_info() {
        let mut parser = CypherParser::new("MATCH (n)".to_string());
        let info = parser.get_parser_info();
        assert_eq!(info.current_position, 0);
        assert!(info.total_tokens > 0);
        assert!(info.current_token.is_some());
    }

    #[test]
    fn test_parser_reset() {
        let mut parser = CypherParser::new("MATCH (n) RETURN n".to_string());

        // 解析第一个语句
        let _ = parser.parse_statement();

        // 重置解析器
        parser.reset();

        // 再次解析应该成功
        let result = parser.parse_statement();
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_remaining_input() {
        let mut parser = CypherParser::new("MATCH (n) RETURN n".to_string());

        // 消费一些标记
        let _ = parser.parse_statement();

        // 获取剩余输入
        let remaining = parser.get_remaining_input();
        assert!(!remaining.is_empty());
    }

    #[test]
    fn test_parse_result() {
        let info = ParserInfo {
            current_position: 0,
            total_tokens: 5,
            current_token: None,
        };

        let result = ParseResult::success("test".to_string(), info.clone());
        assert_eq!(result.result, "test");
        assert_eq!(result.warning_count(), 0);

        let mut result_with_warnings = ParseResult::success_with_warnings(
            "test".to_string(),
            vec!["Warning 1".to_string(), "Warning 2".to_string()],
            info,
        );
        assert_eq!(result_with_warnings.warning_count(), 2);

        result_with_warnings.add_warning("Warning 3".to_string());
        assert_eq!(result_with_warnings.warning_count(), 3);
    }
}
