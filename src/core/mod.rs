use std::collections::{HashMap, BTreeSet};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Represents a node in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: u64,
    pub labels: Vec<String>,
    pub properties: HashMap<String, Value>,
}

impl Node {
    pub fn new(labels: Vec<String>, properties: HashMap<String, Value>) -> Self {
        Self {
            id: 0, // Will be assigned by storage layer
            labels,
            properties,
        }
    }
}

/// Represents an edge in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: u64,
    pub from_node: u64,
    pub to_node: u64,
    pub edge_type: String,
    pub properties: HashMap<String, Value>,
}

impl Edge {
    pub fn new(from_node: u64, to_node: u64, edge_type: String, properties: HashMap<String, Value>) -> Self {
        Self {
            id: 0, // Will be assigned by storage layer
            from_node,
            to_node,
            edge_type,
            properties,
        }
    }
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Represents a value that can be stored in node/edge properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
}

// Implement PartialEq manually to handle f64 comparison properly
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => (a == b) || (a.is_nan() && b.is_nan()), // Handle NaN properly
            (Value::String(a), Value::String(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            _ => false,
        }
    }
}

// Implement Eq manually since f64 doesn't implement Eq
impl Eq for Value {}

// Implement Hash manually to handle f64 hashing
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Null => 0u8.hash(state),
            Value::Boolean(b) => b.hash(state),
            Value::Integer(i) => i.hash(state),
            Value::Float(f) => {
                // Create a hash from the bit representation of the float
                if f.is_nan() {
                    // All NaN values should hash to the same value
                    (0x7ff80000u32 as u64).hash(state);
                } else if *f == 0.0 {
                    // Ensure +0.0 and -0.0 hash to the same value
                    0.0_f64.to_bits().hash(state);
                } else {
                    f.to_bits().hash(state);
                }
            },
            Value::String(s) => s.hash(state),
            Value::List(l) => l.hash(state),
            Value::Map(m) => {
                // Hash a map by hashing key-value pairs in sorted order
                let mut pairs: Vec<_> = m.iter().collect();
                pairs.sort_by_key(|&(k, _)| k);
                pairs.hash(state);
            },
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

/// Schema definition for node labels and edge types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub node_labels: BTreeSet<String>,
    pub edge_types: BTreeSet<String>,
    pub property_keys: BTreeSet<String>,
}