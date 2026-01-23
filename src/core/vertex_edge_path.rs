use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::value::{NullType, Value};

/// Represents a tag in the graph, similar to Nebula's Tag structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct Vertex {
    pub vid: Box<Value>, // Vertex ID can now be any Value type, using Box to break cycles
    pub id: i64,         // Internal integer ID for indexing and fast lookup
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
            id: 0,
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
            id: 0,
            tags,
            properties,
        }
    }

    /// Add a tag to the vertex
    pub fn add_tag(&mut self, tag: Tag) {
        self.tags.push(tag);
    }

    /// Get vertex ID (internal integer ID)
    pub fn id(&self) -> i64 {
        self.id
    }

    /// Get vertex VID (user-visible ID)
    pub fn vid(&self) -> &Value {
        &self.vid
    }

    /// Get all tags
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Check if vertex has a specific tag
    pub fn has_tag(&self, tag_name: &str) -> bool {
        self.tags.iter().any(|tag| tag.name == tag_name)
    }

    /// Get a specific tag by name
    pub fn get_tag(&self, tag_name: &str) -> Option<&Tag> {
        self.tags.iter().find(|tag| tag.name == tag_name)
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

    /// Get property value from any tag (searches all tags)
    pub fn get_property_any(&self, prop_name: &str) -> Option<&Value> {
        for tag in &self.tags {
            if let Some(value) = tag.properties.get(prop_name) {
                return Some(value);
            }
        }
        // Also check vertex-level properties
        self.properties.get(prop_name)
    }

    /// Get all properties from all tags and vertex-level properties
    pub fn get_all_properties(&self) -> HashMap<String, &Value> {
        let mut all_props = HashMap::new();

        // Add vertex-level properties
        for (name, value) in &self.properties {
            all_props.insert(name.clone(), value);
        }

        // Add tag properties (later tags override earlier ones)
        for tag in &self.tags {
            for (name, value) in &tag.properties {
                all_props.insert(name.clone(), value);
            }
        }

        all_props
    }

    /// Get vertex-level properties
    pub fn vertex_properties(&self) -> &HashMap<String, Value> {
        &self.properties
    }

    /// Set a vertex-level property
    pub fn set_vertex_property(&mut self, name: String, value: Value) {
        self.properties.insert(name, value);
    }

    /// Remove a vertex-level property
    pub fn remove_vertex_property(&mut self, name: &str) -> Option<Value> {
        self.properties.remove(name)
    }

    /// Get the number of tags
    pub fn tag_count(&self) -> usize {
        self.tags.len()
    }

    /// Check if vertex has any properties
    pub fn has_properties(&self) -> bool {
        !self.properties.is_empty() || self.tags.iter().any(|tag| !tag.properties.is_empty())
    }

    /// 比较属性映射的辅助函数
    fn cmp_properties(
        a: &HashMap<String, Value>,
        b: &HashMap<String, Value>,
    ) -> std::cmp::Ordering {
        // 先比较属性数量
        match a.len().cmp(&b.len()) {
            std::cmp::Ordering::Equal => {
                // 按排序后的键值对比较
                let mut a_sorted: Vec<_> = a.iter().collect();
                let mut b_sorted: Vec<_> = b.iter().collect();
                a_sorted.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
                b_sorted.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

                for ((k1, v1), (k2, v2)) in a_sorted.iter().zip(b_sorted.iter()) {
                    match k1.cmp(k2) {
                        std::cmp::Ordering::Equal => match v1.cmp(v2) {
                            std::cmp::Ordering::Equal => continue,
                            ord => return ord,
                        },
                        ord => return ord,
                    }
                }
                std::cmp::Ordering::Equal
            }
            ord => ord,
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            vid: Box::new(Value::Null(NullType::NaN)),
            id: 0,
            tags: Vec::new(),
            properties: HashMap::new(),
        }
    }
}

impl Ord for Vertex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 使用链式比较，提高可读性
        self.vid
            .cmp(&other.vid)
            .then_with(|| self.tags.len().cmp(&other.tags.len()))
            .then_with(|| self.cmp_tags_and_properties(other))
    }
}

impl Vertex {
    /// 比较标签和属性
    fn cmp_tags_and_properties(&self, other: &Self) -> std::cmp::Ordering {
        // 比较标签
        let mut self_tags: Vec<_> = self.tags.iter().collect();
        let mut other_tags: Vec<_> = other.tags.iter().collect();
        self_tags.sort_by(|a, b| a.name.cmp(&b.name));
        other_tags.sort_by(|a, b| a.name.cmp(&b.name));

        // 比较每个标签
        for (tag1, tag2) in self_tags.iter().zip(other_tags.iter()) {
            let tag_cmp = tag1
                .name
                .cmp(&tag2.name)
                .then_with(|| Vertex::cmp_properties(&tag1.properties, &tag2.properties));

            if tag_cmp != std::cmp::Ordering::Equal {
                return tag_cmp;
            }
        }

        // 比较顶点级属性
        Vertex::cmp_properties(&self.properties, &other.properties)
    }
}

impl PartialOrd for Vertex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents an edge in the graph, similar to Nebula's Edge structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct Edge {
    pub src: Box<Value>, // Source vertex ID (can be any Value type, using Box to break cycles)
    pub dst: Box<Value>, // Destination vertex ID (can be any Value type, using Box to break cycles)
    pub edge_type: String, // Edge type name
    pub ranking: i64,    // Edge ranking
    pub id: i64,         // Internal integer ID for indexing and fast lookup
    pub props: HashMap<String, Value>, // Edge properties
}

/// 为了兼容性，添加properties字段
impl Edge {
    /// 获取边的属性
    pub fn properties(&self) -> &HashMap<String, Value> {
        &self.props
    }

    /// Get source vertex ID
    pub fn src(&self) -> &Value {
        &self.src
    }

    /// Get destination vertex ID
    pub fn dst(&self) -> &Value {
        &self.dst
    }

    /// Get edge type name
    pub fn edge_type(&self) -> &str {
        &self.edge_type
    }

    /// Get edge ranking
    pub fn ranking(&self) -> i64 {
        self.ranking
    }

    /// Get all edge properties
    pub fn get_all_properties(&self) -> &HashMap<String, Value> {
        &self.props
    }

    /// Get a specific property by name
    pub fn get_property(&self, name: &str) -> Option<&Value> {
        self.props.get(name)
    }

    /// Set a property value
    pub fn set_property(&mut self, name: String, value: Value) {
        self.props.insert(name, value);
    }

    /// Remove a property
    pub fn remove_property(&mut self, name: &str) -> Option<Value> {
        self.props.remove(name)
    }

    /// Check if edge has a specific property
    pub fn has_property(&self, name: &str) -> bool {
        self.props.contains_key(name)
    }

    /// Get the number of properties
    pub fn property_count(&self) -> usize {
        self.props.len()
    }

    /// Check if edge has any properties
    pub fn has_properties(&self) -> bool {
        !self.props.is_empty()
    }

    /// Create a new edge with empty properties
    pub fn new_empty(src: Value, dst: Value, edge_type: String, ranking: i64) -> Self {
        Self {
            src: Box::new(src),
            dst: Box::new(dst),
            edge_type,
            ranking,
            id: 0,
            props: HashMap::new(),
        }
    }

    /// Get a string representation of the edge for debugging
    pub fn debug_string(&self) -> String {
        format!(
            "Edge({:?} -> {:?}, type: {}, ranking: {})",
            self.src, self.dst, self.edge_type, self.ranking
        )
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
            id: 0,
            props,
        }
    }
}

impl Ord for Edge {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 使用链式比较，提高可读性
        self.src
            .cmp(&other.src)
            .then_with(|| self.dst.cmp(&other.dst))
            .then_with(|| self.edge_type.cmp(&other.edge_type))
            .then_with(|| self.ranking.cmp(&other.ranking))
            .then_with(|| Vertex::cmp_properties(&self.props, &other.props))
    }
}

impl PartialOrd for Edge {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a step in a path
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct Step {
    pub dst: Box<Vertex>,
    pub edge: Box<Edge>,
}

impl std::hash::Hash for Step {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.dst.hash(state);
        self.edge.hash(state);
    }
}

impl Ord for Step {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 比较顺序：dst -> edge
        match self.dst.cmp(&other.dst) {
            std::cmp::Ordering::Equal => self.edge.cmp(&other.edge),
            ord => ord,
        }
    }
}

impl PartialOrd for Step {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Represents a path in the graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub struct Path {
    pub src: Box<Vertex>,
    pub steps: Vec<Step>,
}

impl Ord for Path {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // 使用链式比较，提高可读性
        self.src
            .cmp(&other.src)
            .then_with(|| self.steps.len().cmp(&other.steps.len()))
            .then_with(|| self.cmp_steps(other))
    }
}

impl Path {
    /// 比较路径中的步骤
    fn cmp_steps(&self, other: &Self) -> std::cmp::Ordering {
        // 比较每个步骤
        for (step1, step2) in self.steps.iter().zip(other.steps.iter()) {
            let step_cmp = step1.cmp(step2);
            if step_cmp != std::cmp::Ordering::Equal {
                return step_cmp;
            }
        }
        std::cmp::Ordering::Equal
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
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

    /// 获取路径长度（步骤数）
    pub fn length(&self) -> usize {
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
