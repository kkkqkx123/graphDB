use crate::core::{Tag, Value};

#[derive(Debug, Clone)]
pub enum Query {
    CreateNode {
        id: Option<Value>,
        tags: Vec<Tag>,
    },
    CreateEdge {
        src: Value,
        dst: Value,
        edge_type: String,
        name: String,
        ranking: i64,
        properties: std::collections::HashMap<String, Value>,
    },
    MatchNodes {
        tags: Option<Vec<String>>, // Filter by tag names
        conditions: Vec<Condition>,
    },
    DeleteNode {
        id: Value,
    },
    UpdateNode {
        id: Value,
        tags: Vec<Tag>,
    },
}

#[derive(Debug, Clone)]
pub enum Condition {
    PropertyEquals(String, Value),
    PropertyGreaterThan(String, Value),
    PropertyLessThan(String, Value),
    PropertyIn(String, Vec<Value>),
}
