//! Pattern AST definitions for the query parser

use super::{expression::*, types::*};

// MatchPath represents a pattern in a MATCH clause
#[derive(Debug, Clone, PartialEq)]
pub struct MatchPath {
    pub path: Vec<MatchPathSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPathSegment {
    Node(MatchNode),
    Edge(MatchEdge),
    // For more complex path patterns
    PathPattern(MatchPathPattern),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchNode {
    pub identifier: Option<Identifier>,
    pub labels: Vec<Label>,
    pub properties: Option<Expression>,
    pub predicates: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchEdge {
    pub direction: EdgeDirection,
    pub identifier: Option<Identifier>,
    pub types: Vec<Identifier>,
    pub relationship: Option<Identifier>,
    pub properties: Option<Expression>,
    pub predicates: Vec<Expression>,
    pub range: Option<StepRange>, // For variable length paths
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeDirection {
    Outbound,      // ->
    Inbound,       // <-
    Bidirectional, // -
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepRange {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchPathPattern {
    // Complex path patterns can be nested structures
    pub path: Vec<MatchPathSegment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub name: Identifier,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_structures() {
        let match_node = MatchNode {
            identifier: Some("n".to_string()),
            labels: vec![Label {
                name: "Person".to_string(),
            }],
            properties: None,
            predicates: vec![],
        };

        assert_eq!(match_node.identifier, Some("n".to_string()));
        assert_eq!(match_node.labels[0].name, "Person");
    }
}