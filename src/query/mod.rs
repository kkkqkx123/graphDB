use std::collections::HashMap;
use crate::core::{Node, Edge, Value};
use crate::storage::{StorageEngine, StorageError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

#[derive(Debug, Clone)]
pub enum Query {
    CreateNode {
        labels: Vec<String>,
        properties: HashMap<String, Value>,
    },
    CreateEdge {
        from: u64,
        to: u64,
        edge_type: String,
        properties: HashMap<String, Value>,
    },
    MatchNodes {
        labels: Option<Vec<String>>,
        conditions: Vec<Condition>,
    },
    DeleteNode {
        id: u64,
    },
    UpdateNode {
        id: u64,
        updates: HashMap<String, Value>,
    },
}

#[derive(Debug, Clone)]
pub enum Condition {
    PropertyEquals(String, Value),
    PropertyGreaterThan(String, Value),
    PropertyLessThan(String, Value),
    PropertyIn(String, Vec<Value>),
}

#[derive(Debug)]
pub enum QueryResult {
    NodeId(u64),
    EdgeId(u64),
    Nodes(Vec<Node>),
    Edges(Vec<Edge>),
    Count(usize),
    Success,
}

pub struct QueryExecutor<S: StorageEngine> {
    storage: S,
}

impl<S: StorageEngine> QueryExecutor<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    pub fn execute(&mut self, query: Query) -> Result<QueryResult, QueryError> {
        match query {
            Query::CreateNode { labels, properties } => {
                let node = Node::new(labels, properties);
                let id = self.storage.insert_node(node)?;
                Ok(QueryResult::NodeId(id))
            }
            Query::CreateEdge { from, to, edge_type, properties } => {
                // Verify that both nodes exist before creating an edge
                if self.storage.get_node(from)?.is_none() {
                    return Err(QueryError::InvalidQuery(format!("Source node {} does not exist", from)));
                }
                if self.storage.get_node(to)?.is_none() {
                    return Err(QueryError::InvalidQuery(format!("Target node {} does not exist", to)));
                }

                let edge = Edge::new(from, to, edge_type, properties);
                let id = self.storage.insert_edge(edge)?;
                Ok(QueryResult::EdgeId(id))
            }
            Query::MatchNodes { labels, conditions } => {
                // Improved implementation - in a real system, this would utilize indexes
                // For now we implement a basic property-based filter
                let all_nodes = self.get_all_nodes()?;

                let filtered_nodes = all_nodes.into_iter()
                    .filter(|node| {
                        // Check labels if specified
                        if let Some(ref req_labels) = labels {
                            if !req_labels.iter().all(|label| node.labels.contains(label)) {
                                return false;
                            }
                        }

                        // Check conditions if specified
                        for condition in &conditions {
                            if !self.check_condition(node, condition) {
                                return false;
                            }
                        }

                        true
                    })
                    .collect();

                Ok(QueryResult::Nodes(filtered_nodes))
            }
            Query::DeleteNode { id } => {
                self.storage.delete_node(id)?;
                Ok(QueryResult::Success)
            }
            Query::UpdateNode { id, updates } => {
                if let Some(mut node) = self.storage.get_node(id)? {
                    for (key, value) in updates {
                        node.properties.insert(key, value);
                    }
                    self.storage.update_node(node)?;
                    Ok(QueryResult::Success)
                } else {
                    Err(QueryError::InvalidQuery(format!("Node {} not found", id)))
                }
            }
        }
    }

    // Helper function to get all nodes (in a real implementation this might use indexes)
    fn get_all_nodes(&mut self) -> Result<Vec<Node>, QueryError> {
        // This is a simplified implementation
        // In a real system, you might maintain an index of all node IDs
        // or implement a scan operation on the storage layer
        Ok(Vec::new()) // Placeholder - in a real implementation, we'd scan all nodes
    }

    // Helper function to check if a node matches a condition
    fn check_condition(&self, node: &Node, condition: &Condition) -> bool {
        match condition {
            Condition::PropertyEquals(key, expected_value) => {
                node.properties.get(key).map_or(false, |actual_value| actual_value == expected_value)
            }
            Condition::PropertyGreaterThan(key, expected_value) => {
                match (node.properties.get(key), expected_value) {
                    (Some(Value::Integer(a)), Value::Integer(b)) => a > b,
                    (Some(Value::Float(a)), Value::Float(b)) => a > b,
                    _ => false,
                }
            }
            Condition::PropertyLessThan(key, expected_value) => {
                match (node.properties.get(key), expected_value) {
                    (Some(Value::Integer(a)), Value::Integer(b)) => a < b,
                    (Some(Value::Float(a)), Value::Float(b)) => a < b,
                    _ => false,
                }
            }
            Condition::PropertyIn(key, valid_values) => {
                node.properties.get(key).map_or(false, |actual_value| {
                    valid_values.contains(actual_value)
                })
            }
        }
    }
}

pub struct QueryParser;

impl QueryParser {
    pub fn parse(&self, query_string: &str) -> Result<Query, QueryError> {
        // Simplified parser for demonstration
        // A real implementation would have a proper parser
        
        let lower_query = query_string.trim().to_uppercase();
        
        if lower_query.starts_with("CREATE NODE") {
            // Parse CREATE NODE query
            // This is a simplified example - real parsing would be more robust
            Ok(Query::CreateNode {
                labels: vec!["Default".to_string()],
                properties: HashMap::new(),
            })
        } else if lower_query.starts_with("MATCH") {
            // Parse MATCH query
            Ok(Query::MatchNodes {
                labels: None,
                conditions: Vec::new(),
            })
        } else {
            Err(QueryError::ParseError("Unsupported query type".to_string()))
        }
    }
}