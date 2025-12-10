use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::value::{NullType, Value};

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
        Self { name, properties }
    }
}

/// Represents a vertex in the graph, similar to Nebula's Vertex structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Vertex {
    pub vid: Box<Value>, // Vertex ID can now be any Value type, using Box to break cycles
    pub tags: Vec<Tag>,  // A vertex can have multiple tags
    pub properties: HashMap<String, Value>, // Vertex properties
}

// 手动实现Hash以处理HashMap的Hash
impl std::hash::Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vid.hash(state);
        for tag in &self.tags {
            tag.hash(state);
        }
        // 对于HashMap，我们按键值对的排序顺序进行哈希
        let mut pairs: Vec<_> = self.properties.iter().collect();
        pairs.sort_by_key(|&(k, _)| k);
        for (k, v) in pairs {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl Vertex {
    pub fn new(vid: Value, tags: Vec<Tag>) -> Self {
        Self {
            vid: Box::new(vid),
            tags,
            properties: HashMap::new(),
        }
    }

    pub fn new_with_properties(
        vid: Value,
        tags: Vec<Tag>,
        properties: HashMap<String, Value>,
    ) -> Self {
        Self {
            vid: Box::new(vid),
            tags,
            properties,
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
            properties: HashMap::new(),
        }
    }
}

/// Represents an edge in the graph, similar to Nebula's Edge structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Edge {
    pub src: Box<Value>, // Source vertex ID (can be any Value type, using Box to break cycles)
    pub dst: Box<Value>, // Destination vertex ID (can be any Value type, using Box to break cycles)
    pub edge_type: String, // Edge type name
    pub ranking: i64,    // Edge ranking
    pub props: HashMap<String, Value>, // Edge properties
}

/// 为了兼容性，添加properties字段
impl Edge {
    /// 获取边的属性
    pub fn properties(&self) -> &HashMap<String, Value> {
        &self.props
    }
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
    pub fn new(
        src: Value,
        dst: Value,
        edge_type: String,
        ranking: i64,
        props: HashMap<String, Value>,
    ) -> Self {
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Step {
    pub dst: Box<Vertex>,
    pub edge: Box<Edge>,
}

/// Represents a path in the graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Path {
    pub src: Box<Vertex>,
    pub steps: Vec<Step>,
}

// 手动实现Hash以处理复杂类型的Hash
impl std::hash::Hash for Path {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.src.hash(state);
        for step in &self.steps {
            step.hash(state);
        }
    }
}

impl Path {
    /// 获取路径中的边
    pub fn edges(&self) -> Vec<&Edge> {
        self.steps.iter().map(|step| step.edge.as_ref()).collect()
    }

    /// 获取路径长度（步骤数）
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// 检查路径是否为空（仅包含源顶点）
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
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
