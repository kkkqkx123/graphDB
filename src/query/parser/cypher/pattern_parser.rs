//! Cypher模式解析器
//!
//! 提供Cypher模式解析功能，包括节点和关系模式

use super::ast::*;
use super::parser_core::CypherParserCore;

impl CypherParserCore {
    /// 解析模式列表
    pub fn parse_patterns(&mut self) -> Result<Vec<Pattern>, String> {
        let mut patterns = Vec::new();

        self.skip_whitespace();
        while self.is_current_token_value("(") && !self.is_eof() {
            let pattern = self.parse_pattern()?;
            patterns.push(pattern);

            self.skip_whitespace();
            if self.is_current_token_value(",") {
                self.consume_token(); // 消费 ','
                self.skip_whitespace();
            } else {
                break;
            }
        }

        Ok(patterns)
    }

    /// 解析单个模式
    pub fn parse_pattern(&mut self) -> Result<Pattern, String> {
        let parts = self.parse_pattern_parts()?;
        Ok(Pattern { parts })
    }

    /// 解析模式部分
    pub fn parse_pattern_parts(&mut self) -> Result<Vec<PatternPart>, String> {
        let mut parts = Vec::new();

        self.skip_whitespace();
        while self.is_current_token_value("(") && !self.is_eof() {
            let node = self.parse_node_pattern()?;
            let relationships = self.parse_relationships()?;
            parts.push(PatternPart {
                node,
                relationships,
            });
            self.skip_whitespace();
        }

        Ok(parts)
    }

    /// 解析节点模式
    pub fn parse_node_pattern(&mut self) -> Result<NodePattern, String> {
        self.expect_token_value("(")?; // 消费 '('

        self.skip_whitespace();
        let variable = if self.is_current_token_type(super::lexer::TokenType::Identifier) {
            Some(self.parse_identifier()?)
        } else {
            None
        };

        self.skip_whitespace();
        let labels = self.parse_labels()?;

        self.skip_whitespace();
        let properties = self.parse_properties()?;

        self.skip_whitespace();
        self.expect_token_value(")")?; // 消费 ')'

        Ok(NodePattern {
            variable,
            labels,
            properties,
        })
    }

    /// 解析关系模式列表
    pub fn parse_relationships(&mut self) -> Result<Vec<RelationshipPattern>, String> {
        let mut relationships = Vec::new();

        self.skip_whitespace();
        while (self.is_current_token_value("-") || self.is_current_token_value("<"))
            && !self.is_eof()
        {
            let relationship = self.parse_relationship_pattern()?;
            relationships.push(relationship);
            self.skip_whitespace();
        }

        Ok(relationships)
    }

    /// 解析单个关系模式
    pub fn parse_relationship_pattern(&mut self) -> Result<RelationshipPattern, String> {
        let direction = self.parse_direction()?;

        self.expect_token_value("[")?; // 消费 '['

        self.skip_whitespace();
        let variable = if self.is_current_token_type(super::lexer::TokenType::Identifier) {
            Some(self.parse_identifier()?)
        } else {
            None
        };

        self.skip_whitespace();
        let types = self.parse_types()?;

        self.skip_whitespace();
        let properties = self.parse_properties()?;

        self.skip_whitespace();
        let range = self.parse_range()?;

        self.skip_whitespace();
        self.expect_token_value("]")?; // 消费 ']'

        Ok(RelationshipPattern {
            direction,
            variable,
            types,
            properties,
            range,
        })
    }

    /// 解析关系方向
    pub fn parse_direction(&mut self) -> Result<Direction, String> {
        self.skip_whitespace();

        if self.is_current_token_value("<") {
            self.consume_token(); // 消费 '<'
            self.expect_token_value("-")?; // 消费 '-'
            Ok(Direction::Left)
        } else if self.is_current_token_value("-") {
            self.consume_token(); // 消费 '-'
            self.skip_whitespace();
            if self.is_current_token_value(">") {
                self.consume_token(); // 消费 '>'
                Ok(Direction::Right)
            } else {
                Ok(Direction::Both)
            }
        } else {
            Err(format!(
                "期望关系方向，但得到 '{}' 在位置 {}",
                self.current_token().value,
                self.current_token().position
            ))
        }
    }

    /// 解析路径模式
    pub fn parse_path_pattern(&mut self) -> Result<Pattern, String> {
        // 简化实现，实际应该支持更复杂的路径模式
        self.parse_pattern()
    }

    /// 解析最短路径模式
    pub fn parse_shortest_path_pattern(&mut self) -> Result<Pattern, String> {
        self.expect_keyword("SHORTEST")?;
        self.expect_keyword("PATH")?;

        self.skip_whitespace();
        if self.is_current_token_value("(") {
            self.consume_token(); // 消费 '('
            let pattern = self.parse_pattern()?;
            self.expect_token_value(")")?;
            Ok(pattern)
        } else {
            Err("期望路径模式在括号内".to_string())
        }
    }

    /// 解析所有最短路径模式
    pub fn parse_all_shortest_paths_pattern(&mut self) -> Result<Pattern, String> {
        self.expect_keyword("ALL")?;
        self.expect_keyword("SHORTEST")?;
        self.expect_keyword("PATHS")?;

        self.skip_whitespace();
        if self.is_current_token_value("(") {
            self.consume_token(); // 消费 '('
            let pattern = self.parse_pattern()?;
            self.expect_token_value(")")?;
            Ok(pattern)
        } else {
            Err("期望路径模式在括号内".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_node_pattern() {
        let mut parser = CypherParserCore::new("(n:Person)".to_string());
        let node = parser
            .parse_node_pattern()
            .expect("Pattern parser should parse valid node patterns");

        assert_eq!(node.variable, Some("n".to_string()));
        assert_eq!(node.labels, vec!["Person".to_string()]);
        assert!(node.properties.is_none());
    }

    #[test]
    fn test_parse_node_pattern_with_properties() {
        let mut parser = CypherParserCore::new("(n:Person {name: 'Alice', age: 30})".to_string());
        let node = parser
            .parse_node_pattern()
            .expect("Pattern parser should parse valid node patterns");

        assert_eq!(node.variable, Some("n".to_string()));
        assert_eq!(node.labels, vec!["Person".to_string()]);
        assert!(node.properties.is_some());

        let properties = node.properties.expect("Node should have properties");
        assert_eq!(properties.len(), 2);
        assert!(properties.contains_key("name"));
        assert!(properties.contains_key("age"));
    }

    #[test]
    fn test_parse_relationship_pattern() {
        let mut parser = CypherParserCore::new("-[:FRIENDS_WITH]->".to_string());
        let relationship = parser
            .parse_relationship_pattern()
            .expect("Pattern parser should parse valid relationship patterns");

        assert_eq!(relationship.direction, Direction::Right);
        assert!(relationship.variable.is_none());
        assert_eq!(relationship.types, vec!["FRIENDS_WITH".to_string()]);
        assert!(relationship.properties.is_none());
        assert!(relationship.range.is_none());
    }

    #[test]
    fn test_parse_relationship_pattern_with_variable() {
        let mut parser = CypherParserCore::new("-[r:KNOWS]->".to_string());
        let relationship = parser
            .parse_relationship_pattern()
            .expect("Pattern parser should parse valid relationship patterns");

        assert_eq!(relationship.direction, Direction::Right);
        assert_eq!(relationship.variable, Some("r".to_string()));
        assert_eq!(relationship.types, vec!["KNOWS".to_string()]);
    }

    #[test]
    fn test_parse_left_direction() {
        let mut parser = CypherParserCore::new("<-[:KNOWS]-".to_string());
        let relationship = parser
            .parse_relationship_pattern()
            .expect("Pattern parser should parse valid relationship patterns");

        assert_eq!(relationship.direction, Direction::Left);
        assert_eq!(relationship.types, vec!["KNOWS".to_string()]);
    }

    #[test]
    fn test_parse_both_direction() {
        let mut parser = CypherParserCore::new("-[:KNOWS]-".to_string());
        let relationship = parser
            .parse_relationship_pattern()
            .expect("Pattern parser should parse valid relationship patterns");

        assert_eq!(relationship.direction, Direction::Both);
        assert_eq!(relationship.types, vec!["KNOWS".to_string()]);
    }

    #[test]
    fn test_parse_relationship_with_range() {
        let mut parser = CypherParserCore::new("-[:KNOWS*1..3]-".to_string());
        let relationship = parser
            .parse_relationship_pattern()
            .expect("Pattern parser should parse valid relationship patterns");

        assert_eq!(relationship.direction, Direction::Both);
        assert_eq!(relationship.types, vec!["KNOWS".to_string()]);
        assert!(relationship.range.is_some());

        let range = relationship.range.expect("Relationship should have range");
        assert_eq!(range.start, Some(1));
        assert_eq!(range.end, Some(3));
    }

    #[test]
    fn test_parse_simple_pattern() {
        let mut parser =
            CypherParserCore::new("(a:Person)-[:FRIENDS_WITH]->(b:Person)".to_string());
        let pattern = parser
            .parse_pattern()
            .expect("Pattern parser should parse valid patterns");

        assert_eq!(pattern.parts.len(), 1);

        let part = &pattern.parts[0];
        assert_eq!(part.node.variable, Some("a".to_string()));
        assert_eq!(part.node.labels, vec!["Person".to_string()]);
        assert_eq!(part.relationships.len(), 1);

        let relationship = &part.relationships[0];
        assert_eq!(relationship.direction, Direction::Right);
        assert_eq!(relationship.types, vec!["FRIENDS_WITH".to_string()]);
    }

    #[test]
    fn test_parse_multiple_patterns() {
        let mut parser = CypherParserCore::new(
            "(a:Person)-[:KNOWS]->(b:Person), (c:Company)-[:LOCATED_IN]->(d:City)".to_string(),
        );
        let patterns = parser
            .parse_patterns()
            .expect("Pattern parser should parse valid patterns");

        assert_eq!(patterns.len(), 2);

        // 验证第一个模式
        let first_pattern = &patterns[0];
        assert_eq!(first_pattern.parts.len(), 1);
        assert_eq!(first_pattern.parts[0].node.variable, Some("a".to_string()));

        // 验证第二个模式
        let second_pattern = &patterns[1];
        assert_eq!(second_pattern.parts.len(), 1);
        assert_eq!(second_pattern.parts[0].node.variable, Some("c".to_string()));
    }

    #[test]
    fn test_parse_complex_pattern() {
        let mut parser = CypherParserCore::new(
            "(a:Person)-[:KNOWS*1..2]->(b:Person)<-[:WORKS_WITH]-(c:Company)".to_string(),
        );
        let pattern = parser
            .parse_pattern()
            .expect("Pattern parser should parse valid patterns");

        assert_eq!(pattern.parts.len(), 1);

        let part = &pattern.parts[0];
        assert_eq!(part.node.variable, Some("a".to_string()));
        assert_eq!(part.relationships.len(), 2);

        // 验证第一个关系
        let first_rel = &part.relationships[0];
        assert_eq!(first_rel.direction, Direction::Right);
        assert_eq!(first_rel.types, vec!["KNOWS".to_string()]);
        assert!(first_rel.range.is_some());

        // 验证第二个关系
        let second_rel = &part.relationships[1];
        assert_eq!(second_rel.direction, Direction::Left);
        assert_eq!(second_rel.types, vec!["WORKS_WITH".to_string()]);
    }
}
