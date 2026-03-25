//! Implementation of the Insert Operation Node
//!
//! Provide the definition of the planning nodes for vertex insertion and edge insertion.

use crate::core::types::expr::contextual::ContextualExpression;
use crate::define_plan_node;

/// Tag insertion guidelines
#[derive(Debug, Clone)]
pub struct TagInsertSpec {
    pub tag_name: String,
    pub prop_names: Vec<String>,
}

/// Vertex insertion information
/// Supports the insertion of multiple tags.
#[derive(Debug, Clone)]
pub struct VertexInsertInfo {
    pub space_name: String,
    pub tags: Vec<TagInsertSpec>,
    pub values: Vec<(ContextualExpression, Vec<Vec<ContextualExpression>>)>,
}

/// Insert information as needed.
#[derive(Debug, Clone)]
pub struct EdgeInsertInfo {
    pub space_name: String,
    pub edge_name: String,
    pub prop_names: Vec<String>,
    pub edges: Vec<(
        ContextualExpression,
        ContextualExpression,
        Option<ContextualExpression>,
        Vec<ContextualExpression>,
    )>,
}

define_plan_node! {
    pub struct InsertVerticesNode {
        info: VertexInsertInfo,
    }
    enum: InsertVertices
    input: ZeroInputNode
}

impl InsertVerticesNode {
    pub fn new(id: i64, info: VertexInsertInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: vec!["inserted".to_string()],
        }
    }

    pub fn info(&self) -> &VertexInsertInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    /// Retrieve all tag names.
    pub fn tag_names(&self) -> Vec<String> {
        self.info.tags.iter().map(|t| t.tag_name.clone()).collect()
    }

    /// Retrieve the name of the first tag (for backward compatibility).
    pub fn tag_name(&self) -> Option<&str> {
        self.info.tags.first().map(|t| t.tag_name.as_str())
    }

    /// Get a list of the attribute names for all tags.
    pub fn tags(&self) -> &[TagInsertSpec] {
        &self.info.tags
    }

    /// Get the attribute name of the first tag (for backward compatibility)
    pub fn prop_names(&self) -> Option<&[String]> {
        self.info.tags.first().map(|t| t.prop_names.as_slice())
    }

    pub fn values(&self) -> &[(ContextualExpression, Vec<Vec<ContextualExpression>>)] {
        &self.info.values
    }
}

define_plan_node! {
    pub struct InsertEdgesNode {
        info: EdgeInsertInfo,
    }
    enum: InsertEdges
    input: ZeroInputNode
}

impl InsertEdgesNode {
    pub fn new(id: i64, info: EdgeInsertInfo) -> Self {
        Self {
            id,
            info,
            output_var: None,
            col_names: vec!["inserted".to_string()],
        }
    }

    pub fn info(&self) -> &EdgeInsertInfo {
        &self.info
    }

    pub fn space_name(&self) -> &str {
        &self.info.space_name
    }

    pub fn edge_name(&self) -> &str {
        &self.info.edge_name
    }

    pub fn prop_names(&self) -> &[String] {
        &self.info.prop_names
    }

    pub fn edges(
        &self,
    ) -> &[(
        ContextualExpression,
        ContextualExpression,
        Option<ContextualExpression>,
        Vec<ContextualExpression>,
    )] {
        &self.info.edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expr::{ContextualExpression, ExpressionMeta};
    use crate::core::{Expression, Value};
    use crate::query::validator::context::ExpressionAnalysisContext;
    use std::sync::Arc;

    // Auxiliary function: Creating constant expressions
    fn lit(val: Value) -> ContextualExpression {
        let expr_ctx = Arc::new(ExpressionAnalysisContext::new());
        let expr = Expression::literal(val);
        let meta = ExpressionMeta::new(expr);
        let id = expr_ctx.register_expression(meta);
        ContextualExpression::new(id, expr_ctx)
    }

    #[test]
    fn test_vertex_insert_info_creation() {
        let info = VertexInsertInfo {
            space_name: "test_space".to_string(),
            tags: vec![TagInsertSpec {
                tag_name: "person".to_string(),
                prop_names: vec!["name".to_string(), "age".to_string()],
            }],
            values: vec![(
                lit(Value::Int(1)),
                vec![vec![
                    lit(Value::String("Alice".to_string())),
                    lit(Value::Int(30)),
                ]],
            )],
        };

        assert_eq!(info.space_name, "test_space");
        assert_eq!(info.tags.len(), 1);
        assert_eq!(info.tags[0].tag_name, "person");
        assert_eq!(info.values.len(), 1);
    }

    #[test]
    fn test_edge_insert_info_creation() {
        let info = EdgeInsertInfo {
            space_name: "test_space".to_string(),
            edge_name: "follow".to_string(),
            prop_names: vec!["since".to_string()],
            edges: vec![(
                lit(Value::Int(1)),
                lit(Value::Int(2)),
                Some(lit(Value::Int(0))),
                vec![lit(Value::String("2023".to_string()))],
            )],
        };

        assert_eq!(info.space_name, "test_space");
        assert_eq!(info.edge_name, "follow");
        assert_eq!(info.prop_names.len(), 1);
        assert_eq!(info.edges.len(), 1);
    }

    #[test]
    fn test_insert_vertices_node_creation() {
        let info = VertexInsertInfo {
            space_name: "test_space".to_string(),
            tags: vec![TagInsertSpec {
                tag_name: "person".to_string(),
                prop_names: vec!["name".to_string()],
            }],
            values: vec![(
                lit(Value::Int(1)),
                vec![vec![lit(Value::String("Alice".to_string()))]],
            )],
        };

        let node = InsertVerticesNode::new(100, info);

        assert_eq!(node.id(), 100);
        assert_eq!(node.space_name(), "test_space");
        assert_eq!(node.tag_name(), Some("person"));
        assert_eq!(node.prop_names().map(|p| p.len()), Some(1));
        assert_eq!(node.values().len(), 1);
        assert_eq!(node.col_names(), &["inserted"]);
    }

    #[test]
    fn test_insert_edges_node_creation() {
        let info = EdgeInsertInfo {
            space_name: "test_space".to_string(),
            edge_name: "follow".to_string(),
            prop_names: vec!["since".to_string()],
            edges: vec![(
                lit(Value::Int(1)),
                lit(Value::Int(2)),
                Some(lit(Value::Int(0))),
                vec![lit(Value::String("2023".to_string()))],
            )],
        };

        let node = InsertEdgesNode::new(200, info);

        assert_eq!(node.id(), 200);
        assert_eq!(node.space_name(), "test_space");
        assert_eq!(node.edge_name(), "follow");
        assert_eq!(node.prop_names().len(), 1);
        assert_eq!(node.edges().len(), 1);
        assert_eq!(node.col_names(), &["inserted"]);
    }

    #[test]
    fn test_insert_vertices_node_with_multiple_values() {
        let info = VertexInsertInfo {
            space_name: "test_space".to_string(),
            tags: vec![TagInsertSpec {
                tag_name: "person".to_string(),
                prop_names: vec!["name".to_string(), "age".to_string()],
            }],
            values: vec![
                (
                    lit(Value::Int(1)),
                    vec![vec![
                        lit(Value::String("Alice".to_string())),
                        lit(Value::Int(30)),
                    ]],
                ),
                (
                    lit(Value::Int(2)),
                    vec![vec![
                        lit(Value::String("Bob".to_string())),
                        lit(Value::Int(25)),
                    ]],
                ),
                (
                    lit(Value::Int(3)),
                    vec![vec![
                        lit(Value::String("Charlie".to_string())),
                        lit(Value::Int(35)),
                    ]],
                ),
            ],
        };

        let node = InsertVerticesNode::new(1, info);
        assert_eq!(node.values().len(), 3);
    }

    #[test]
    fn test_insert_edges_node_with_multiple_edges() {
        let info = EdgeInsertInfo {
            space_name: "test_space".to_string(),
            edge_name: "follow".to_string(),
            prop_names: vec!["since".to_string()],
            edges: vec![
                (
                    lit(Value::Int(1)),
                    lit(Value::Int(2)),
                    Some(lit(Value::Int(0))),
                    vec![lit(Value::String("2021".to_string()))],
                ),
                (
                    lit(Value::Int(2)),
                    lit(Value::Int(3)),
                    Some(lit(Value::Int(1))),
                    vec![lit(Value::String("2022".to_string()))],
                ),
            ],
        };

        let node = InsertEdgesNode::new(1, info);
        assert_eq!(node.edges().len(), 2);
    }

    #[test]
    fn test_insert_vertices_node_info_method() {
        let info = VertexInsertInfo {
            space_name: "test_space".to_string(),
            tags: vec![TagInsertSpec {
                tag_name: "person".to_string(),
                prop_names: vec!["name".to_string()],
            }],
            values: vec![],
        };

        let node = InsertVerticesNode::new(1, info.clone());
        let retrieved_info = node.info();

        assert_eq!(retrieved_info.space_name, "test_space");
        assert_eq!(retrieved_info.tags.len(), 1);
        assert_eq!(retrieved_info.tags[0].tag_name, "person");
    }

    #[test]
    fn test_insert_edges_node_info_method() {
        let info = EdgeInsertInfo {
            space_name: "test_space".to_string(),
            edge_name: "follow".to_string(),
            prop_names: vec!["since".to_string()],
            edges: vec![],
        };

        let node = InsertEdgesNode::new(1, info.clone());
        let retrieved_info = node.info();

        assert_eq!(retrieved_info.space_name, "test_space");
        assert_eq!(retrieved_info.edge_name, "follow");
    }

    #[test]
    fn test_multi_tag_insert() {
        let info = VertexInsertInfo {
            space_name: "test_space".to_string(),
            tags: vec![
                TagInsertSpec {
                    tag_name: "person".to_string(),
                    prop_names: vec!["name".to_string()],
                },
                TagInsertSpec {
                    tag_name: "student".to_string(),
                    prop_names: vec!["student_id".to_string()],
                },
            ],
            values: vec![(
                lit(Value::Int(1)),
                vec![
                    vec![lit(Value::String("Alice".to_string()))],
                    vec![lit(Value::String("S001".to_string()))],
                ],
            )],
        };

        let node = InsertVerticesNode::new(1, info);

        assert_eq!(node.tag_names().len(), 2);
        assert_eq!(node.tag_names(), vec!["person", "student"]);
        assert_eq!(node.tags().len(), 2);
    }
}
