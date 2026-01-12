//! Cypher模式定义

use crate::query::parser::cypher::ast::expressions::Expression;
use std::collections::HashMap;

/// 模式定义
#[derive(Debug, Clone)]
pub struct Pattern {
    pub parts: Vec<PatternPart>,
}

/// 模式部分
#[derive(Debug, Clone)]
pub struct PatternPart {
    pub node: NodePattern,
    pub relationships: Vec<RelationshipPattern>,
}

/// 节点模式
#[derive(Debug, Clone)]
pub struct NodePattern {
    pub variable: Option<String>,
    pub labels: Vec<String>,
    pub properties: Option<HashMap<String, Expression>>,
}

/// 关系模式
#[derive(Debug, Clone)]
pub struct RelationshipPattern {
    pub direction: Direction,
    pub variable: Option<String>,
    pub types: Vec<String>,
    pub properties: Option<HashMap<String, Expression>>,
    pub range: Option<Range>,
}

/// 方向
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Both,
}

/// 范围
#[derive(Debug, Clone)]
pub struct Range {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_pattern_creation() {
        let node = NodePattern {
            variable: Some("n".to_string()),
            labels: vec!["Person".to_string(), "User".to_string()],
            properties: Some(HashMap::from([
                (
                    "name".to_string(),
                    Expression::Literal(
                        crate::query::parser::cypher::ast::expressions::Literal::String(
                            "Alice".to_string(),
                        ),
                    ),
                ),
                (
                    "age".to_string(),
                    Expression::Literal(
                        crate::query::parser::cypher::ast::expressions::Literal::Integer(25),
                    ),
                ),
            ])),
        };

        assert_eq!(node.variable, Some("n".to_string()));
        assert_eq!(node.labels.len(), 2);
        assert!(node.properties.is_some());
    }
}
