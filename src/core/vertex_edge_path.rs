use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::value::{Value, NullType};

/// Represents a tag in the graph, similar to Nebula's Tag structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub properties: HashMap<String, Value>,
}

// Implement Hash manually for Tag to handle HashMap hashing
impl std::hash::Hash for Tag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        // For HashMap, we'll hash key-value pairs in sorted order
        let mut pairs: Vec<_> = self.properties.iter().collect();
        pairs.sort_by_key(|&(k, _)| k);
        for (k, v) in pairs {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl Tag {
    pub fn new(name: String, properties: HashMap<String, Value>) -> Self {
        Self {
            name,
            properties,
        }
    }
}

/// Represents a vertex in the graph, similar to Nebula's Vertex structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Vertex {
    pub vid: Box<Value>,             // Vertex ID can now be any Value type, using Box to break cycles
    pub tags: Vec<Tag>,              // A vertex can have multiple tags
}

impl Vertex {
    pub fn new(vid: Value, tags: Vec<Tag>) -> Self {
        Self {
            vid: Box::new(vid),
            tags,
        }
    }

    /// Add a tag to the vertex
    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
    }

    /// Get property value by tag name and property name
    pub fn get_property(&self, tag_name: &str, prop_name: &str) -> Option<&Value> {
        for tag in &self.tags {
            if tag.name == tag_name {
                return tag.properties.get(prop_name);
            }
        }
        None
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            vid: Box::new(Value::Null(NullType::NaN)),
            tags: Vec::new(),
        }
    }
}

/// Represents an edge in the graph, similar to Nebula's Edge structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub src: Box<Value>,            // Source vertex ID (can be any Value type, using Box to break cycles)
    pub dst: Box<Value>,            // Destination vertex ID (can be any Value type, using Box to break cycles)
    pub edge_type: String,          // Edge type name
    pub ranking: i64,               // Edge ranking
    pub props: HashMap<String, Value>,  // Edge properties
}

// Implement Hash manually for Edge to handle HashMap hashing
impl std::hash::Hash for Edge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.src.hash(state);
        self.dst.hash(state);
        self.edge_type.hash(state);
        self.ranking.hash(state);
        // For HashMap, we'll hash key-value pairs in sorted order
        let mut pairs: Vec<_> = self.props.iter().collect();
        pairs.sort_by_key(|&(k, _)| k);
        for (k, v) in pairs {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl Edge {
    pub fn new(src: Value, dst: Value, edge_type: String, ranking: i64,
               props: HashMap<String, Value>) -> Self {
        Self {
            src: Box::new(src),
            dst: Box::new(dst),
            edge_type,
            ranking,
            props,
        }
    }
}

/// Represents a step in a path
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Step {
    pub dst: Box<Vertex>,
    pub edge: Box<Edge>,
}

/// Represents a path in the graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct Path {
    pub src: Box<Vertex>,
    pub steps: Vec<Step>,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            src: Box::new(Vertex::default()),
            steps: Vec::new(),
        }
    }
}

/// Direction for traversing edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction {
    In,
    Out,
    Both,
}