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

    /// Estimate the memory usage of the label.
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();

        // Calculate the actual size of the variable `name` (including heap allocation).
        size += std::mem::size_of::<String>() + self.name.capacity();

        // Calculating the capacity overhead of a HashMap
        size += self.properties.capacity()
            * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());

        for (k, v) in &self.properties {
            size += k.capacity();
            size += v.estimated_size();
        }

        size
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

// Implementing a hash function manually to handle the hashing of a HashMap
impl std::hash::Hash for Vertex {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.vid.hash(state);
        for tag in &self.tags {
            tag.hash(state);
        }
        // For a HashMap, the hashing is performed based on the order in which the key-value pairs are sorted.
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

    /// Create a vertex with only the VID (simplified constructor)
    pub fn with_vid(vid: Value) -> Self {
        Self {
            vid: Box::new(vid),
            id: 0,
            tags: Vec::new(),
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

    /// Auxiliary function for comparing attribute mappings
    fn cmp_properties(
        a: &HashMap<String, Value>,
        b: &HashMap<String, Value>,
    ) -> std::cmp::Ordering {
        // First, compare the number of attributes.
        match a.len().cmp(&b.len()) {
            std::cmp::Ordering::Equal => {
                // Compare the sorted key-value pairs.
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
        // Use chained comparisons to improve readability.
        self.vid
            .cmp(&other.vid)
            .then_with(|| self.tags.len().cmp(&other.tags.len()))
            .then_with(|| self.cmp_tags_and_properties(other))
    }
}

impl Vertex {
    /// Comparing tags and attributes
    fn cmp_tags_and_properties(&self, other: &Self) -> std::cmp::Ordering {
        // Compare tags
        let mut self_tags: Vec<_> = self.tags.iter().collect();
        let mut other_tags: Vec<_> = other.tags.iter().collect();
        self_tags.sort_by(|a, b| a.name.cmp(&b.name));
        other_tags.sort_by(|a, b| a.name.cmp(&b.name));

        // Compare each tag.
        for (tag1, tag2) in self_tags.iter().zip(other_tags.iter()) {
            let tag_cmp = tag1
                .name
                .cmp(&tag2.name)
                .then_with(|| Vertex::cmp_properties(&tag1.properties, &tag2.properties));

            if tag_cmp != std::cmp::Ordering::Equal {
                return tag_cmp;
            }
        }

        // Comparing vertex-level attributes
        Vertex::cmp_properties(&self.properties, &other.properties)
    }
}

impl PartialOrd for Vertex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Vertex {
    /// Estimating the memory usage of the vertex
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();

        // Calculate the actual size of the vid (including the heap allocation for the Box and the content of the Value).
        size += std::mem::size_of::<Box<Value>>() + self.vid.estimated_size();

        // Calculate the memory overhead for the capacity of VecTagName>
        size += self.tags.capacity() * std::mem::size_of::<Tag>();

        for tag in &self.tags {
            // Calculate the actual size of a String (including heap allocation)
            size += std::mem::size_of::<String>() + tag.name.capacity();

            // Calculating the capacity overhead of a HashMap
            size += tag.properties.capacity()
                * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());

            for (k, v) in &tag.properties {
                size += k.capacity();
                size += v.estimated_size();
            }
        }

        // Calculating the capacity overhead of a HashMap<String, Value>
        size += self.properties.capacity()
            * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());

        for (k, v) in &self.properties {
            size += k.capacity();
            size += v.estimated_size();
        }

        size
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

/// For compatibility purposes, the `properties` field has been added.
impl Edge {
    /// Obtaining the properties of an edge
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

    /// Estimate the memory usage of the edges.
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();

        // Calculate the actual sizes of src and dst (including the heap allocation for the Box and the content of the Value).
        size += std::mem::size_of::<Box<Value>>() + self.src.estimated_size();
        size += std::mem::size_of::<Box<Value>>() + self.dst.estimated_size();

        // Calculate the actual size of edge_type (including heap allocation).
        size += std::mem::size_of::<String>() + self.edge_type.capacity();

        // Calculating the capacity overhead of a HashMap
        size +=
            self.props.capacity() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>());

        for (k, v) in &self.props {
            size += k.capacity();
            size += v.estimated_size();
        }

        size
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
        // Use chained comparisons to improve readability.
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
        // Comparison order: dst -> edge
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

impl Step {
    /// Create a new step.
    pub fn new(dst: Vertex, edge_type: String, _edge_name: String, ranking: i64) -> Self {
        // Create an empty edge, which will be filled in later when constructing the path.
        use crate::core::NullType;
        let edge = Edge::new_empty(
            Value::Null(NullType::Null),
            Value::Null(NullType::Null),
            edge_type,
            ranking,
        );
        Self {
            dst: Box::new(dst),
            edge: Box::new(edge),
        }
    }

    /// Steps for creating a model with complete edge information
    pub fn new_with_edge(dst: Vertex, edge: Edge) -> Self {
        Self {
            dst: Box::new(dst),
            edge: Box::new(edge),
        }
    }

    /// Obtain the ID of the source vertex.
    pub fn src_vid(&self) -> &Value {
        &self.edge.src
    }

    /// Obtain the ID of the target vertex.
    pub fn dst_vid(&self) -> &Value {
        &self.dst.vid
    }

    /// Obtaining the ranking of the edges
    pub fn ranking(&self) -> i64 {
        self.edge.ranking
    }

    /// Estimating the memory usage of the steps in a process
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();

        // Calculate the actual sizes of dst and edge (including the heap allocation for the Box and its content).
        size += std::mem::size_of::<Box<Vertex>>() + self.dst.estimated_size();
        size += std::mem::size_of::<Box<Edge>>() + self.edge.estimated_size();

        size
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
        // Use chained comparisons to improve readability.
        self.src
            .cmp(&other.src)
            .then_with(|| self.steps.len().cmp(&other.steps.len()))
            .then_with(|| self.cmp_steps(other))
    }
}

impl Path {
    /// Compare the steps in the paths
    fn cmp_steps(&self, other: &Self) -> std::cmp::Ordering {
        // Compare each step.
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

// Implementing a hash function manually to handle complex data types
impl std::hash::Hash for Path {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.src.hash(state);
        for step in &self.steps {
            step.hash(state);
        }
    }
}

impl Path {
    /// Create a new path.
    pub fn new(src: Vertex) -> Self {
        Self {
            src: Box::new(src),
            steps: Vec::new(),
        }
    }

    /// Estimate the memory usage of the path
    pub fn estimated_size(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();

        // Calculate the actual size of `src` (including the heap allocation for the `Box` and the content of the `Vertex`).
        size += std::mem::size_of::<Box<Vertex>>() + self.src.estimated_size();

        // Calculate the memory overhead for the capacity of Vec<Step>
        size += self.steps.capacity() * std::mem::size_of::<Step>();

        for step in &self.steps {
            size += step.estimated_size();
        }

        size
    }

    /// Add steps to the path
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    /// Obtain the edges in the path
    pub fn edges(&self) -> Vec<&Edge> {
        self.steps.iter().map(|step| step.edge.as_ref()).collect()
    }

    /// Get the path length (number of steps)
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// 获取路径长度（步骤数）
    pub fn length(&self) -> usize {
        self.steps.len()
    }

    /// Obtain the steps in the path
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Check whether the path is empty (contains only the source vertex).
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Check for duplicate edges in the path
    pub fn has_duplicate_edges(&self) -> bool {
        let mut seen_edges: std::collections::HashSet<(Value, Value, String)> =
            std::collections::HashSet::new();

        for step in &self.steps {
            let edge_key = (
                (*step.edge.src).clone(),
                (*step.edge.dst).clone(),
                step.edge.edge_type.clone(),
            );

            if !seen_edges.insert(edge_key) {
                return true;
            }
        }

        false
    }

    /// inversion path
    pub fn reverse(&mut self) {
        if self.steps.is_empty() {
            return;
        }

        // Reverse the steps
        self.steps.reverse();

        // Update the side direction at each step
        for step in &mut self.steps {
            // Swap side src and dst
            std::mem::swap(&mut step.edge.src, &mut step.edge.dst);
        }

        // Update the source vertex to the target vertex of the last step
        if let Some(last_step) = self.steps.first() {
            *self.src = Vertex::new((*last_step.edge.src).clone(), vec![]);
        }
    }

    /// Append reverse path (for bidirectional BFS path splicing)
    /// Append the reverse of another path to the current path
    pub fn append_reverse(&mut self, other: Path) {
        if other.steps.is_empty() {
            return;
        }

        // Reverse steps to get other paths
        let mut other_steps: Vec<Step> = other.steps.into_iter().rev().collect();

        // Reverse the direction of each edge
        for step in &mut other_steps {
            std::mem::swap(&mut step.edge.src, &mut step.edge.dst);
        }

        // Append to current path
        self.steps.extend(other_steps);
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
