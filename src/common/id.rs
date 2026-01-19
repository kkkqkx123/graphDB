use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

/// A unique identifier for vertices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId(i64);

impl VertexId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for VertexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// A unique identifier for edges
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(i64);

impl EdgeId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl fmt::Display for EdgeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

/// A unique identifier for tags (vertex schemas)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TagId(i32);

impl TagId {
    pub fn new(id: i32) -> Self {
        Self(id)
    }

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for TagId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tag{}", self.0)
    }
}

/// A unique identifier for edge types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeType(i32);

impl EdgeType {
    pub fn new(id: i32) -> Self {
        Self(id)
    }

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for EdgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "edge_type{}", self.0)
    }
}

/// A unique identifier for space (database)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpaceId(i32);

impl SpaceId {
    pub fn new(id: i32) -> Self {
        Self(id)
    }

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for SpaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "space{}", self.0)
    }
}

/// A unique identifier for indexes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IndexId(i32);

impl IndexId {
    pub fn new(id: i32) -> Self {
        Self(id)
    }

    pub fn as_i32(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for IndexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "index{}", self.0)
    }
}

/// Global ID generator
#[derive(Debug)]
pub struct IdGenerator {
    vertex_counter: AtomicU64,
    edge_counter: AtomicU64,
    tag_counter: AtomicU64,
    edge_type_counter: AtomicU64,
    space_counter: AtomicU64,
    index_counter: AtomicU64,
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            vertex_counter: AtomicU64::new(1), // Start from 1 to avoid 0
            edge_counter: AtomicU64::new(1),
            tag_counter: AtomicU64::new(1),
            edge_type_counter: AtomicU64::new(1),
            space_counter: AtomicU64::new(1),
            index_counter: AtomicU64::new(1),
        }
    }

    pub fn generate_vertex_id(&self) -> VertexId {
        let id = self.vertex_counter.fetch_add(1, Ordering::SeqCst) as i64;
        VertexId::new(id)
    }

    pub fn generate_edge_id(&self) -> EdgeId {
        let id = self.edge_counter.fetch_add(1, Ordering::SeqCst) as i64;
        EdgeId::new(id)
    }

    pub fn generate_tag_id(&self) -> TagId {
        let id = self.tag_counter.fetch_add(1, Ordering::SeqCst);
        if id > i32::MAX as u64 {
            panic!("Tag ID overflow: exceeded maximum value of i32");
        }
        TagId::new(id as i32)
    }

    pub fn generate_edge_type(&self) -> EdgeType {
        let id = self.edge_type_counter.fetch_add(1, Ordering::SeqCst);
        if id > i32::MAX as u64 {
            panic!("Edge type ID overflow: exceeded maximum value of i32");
        }
        EdgeType::new(id as i32)
    }

    pub fn generate_space_id(&self) -> SpaceId {
        let id = self.space_counter.fetch_add(1, Ordering::SeqCst);
        if id > i32::MAX as u64 {
            panic!("Space ID overflow: exceeded maximum value of i32");
        }
        SpaceId::new(id as i32)
    }

    pub fn generate_index_id(&self) -> IndexId {
        let id = self.index_counter.fetch_add(1, Ordering::SeqCst);
        if id > i32::MAX as u64 {
            panic!("Index ID overflow: exceeded maximum value of i32");
        }
        IndexId::new(id as i32)
    }

    pub fn reset(&self) {
        self.vertex_counter.store(1, Ordering::SeqCst);
        self.edge_counter.store(1, Ordering::SeqCst);
        self.tag_counter.store(1, Ordering::SeqCst);
        self.edge_type_counter.store(1, Ordering::SeqCst);
        self.space_counter.store(1, Ordering::SeqCst);
        self.index_counter.store(1, Ordering::SeqCst);
    }
}

/// Global ID generator instance
static ID_GENERATOR: once_cell::sync::Lazy<IdGenerator> =
    once_cell::sync::Lazy::new(IdGenerator::new);

/// Get reference to the global ID generator
pub fn id_generator() -> &'static IdGenerator {
    &ID_GENERATOR
}

/// Generate a new vertex ID
pub fn gen_vertex_id() -> VertexId {
    ID_GENERATOR.generate_vertex_id()
}

/// Generate a new edge ID
pub fn gen_edge_id() -> EdgeId {
    ID_GENERATOR.generate_edge_id()
}

/// Generate a new tag ID
pub fn gen_tag_id() -> TagId {
    ID_GENERATOR.generate_tag_id()
}

/// Generate a new edge type
pub fn gen_edge_type() -> EdgeType {
    ID_GENERATOR.generate_edge_type()
}

/// Generate a new space ID
pub fn gen_space_id() -> SpaceId {
    ID_GENERATOR.generate_space_id()
}

/// Generate a new index ID
pub fn gen_index_id() -> IndexId {
    ID_GENERATOR.generate_index_id()
}

/// UUID generator utility
pub struct UuidGenerator;

impl UuidGenerator {
    /// Generate a new random UUID
    pub fn generate() -> String {
        Uuid::new_v4().to_string()
    }

    /// Generate a new random UUID and return as Uuid type
    pub fn generate_uuid() -> Uuid {
        Uuid::new_v4()
    }

    /// Generate a UUID from a string (for testing)
    pub fn from_string(s: &str) -> Result<Uuid, uuid::Error> {
        Uuid::parse_str(s)
    }

    /// Check if a string is a valid UUID
    pub fn is_valid(s: &str) -> bool {
        Uuid::parse_str(s).is_ok()
    }
}

/// String-based ID for more complex identification needs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StringId(String);

impl StringId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for StringId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// ID management utilities
pub mod id_utils {
    use super::*;

    /// Convert a string to a vertex ID (useful for external IDs)
    pub fn string_to_vertex_id(s: &str) -> VertexId {
        // Simple hash-based conversion, in production you'd want a more robust solution
        let hash = hash_string(s);
        VertexId::new(hash as i64)
    }

    /// Convert a string to an edge ID (useful for external IDs)
    pub fn string_to_edge_id(s: &str) -> EdgeId {
        // Simple hash-based conversion, in production you'd want a more robust solution
        let hash = hash_string(s);
        EdgeId::new(hash as i64)
    }

    /// Helper function to hash a string to a number
    fn hash_string(s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    /// Validate that an ID is not a default/invalid value
    pub fn is_valid_vertex_id(id: VertexId) -> bool {
        id.as_i64() != 0
    }

    /// Validate that an ID is not a default/invalid value
    pub fn is_valid_edge_id(id: EdgeId) -> bool {
        id.as_i64() != 0
    }

    /// Validate that an ID is not a default/invalid value
    pub fn is_valid_tag_id(id: TagId) -> bool {
        id.as_i32() != 0
    }

    /// Validate that an ID is not a default/invalid value
    pub fn is_valid_edge_type(id: EdgeType) -> bool {
        id.as_i32() != 0
    }

    /// Create a composite ID from two IDs
    pub fn combine_vertex_edge_ids(vertex_id: VertexId, edge_id: EdgeId) -> String {
        format!("{}_{}", vertex_id.as_i64(), edge_id.as_i64())
    }
}

/// A registry to map string IDs to numeric IDs (useful for external IDs)
#[derive(Debug)]
pub struct IdRegistry {
    string_to_vertex: HashMap<String, VertexId>,
    vertex_to_string: HashMap<VertexId, String>,
    string_to_edge: HashMap<String, EdgeId>,
    edge_to_string: HashMap<EdgeId, String>,
}

impl IdRegistry {
    pub fn new() -> Self {
        Self {
            string_to_vertex: HashMap::new(),
            vertex_to_string: HashMap::new(),
            string_to_edge: HashMap::new(),
            edge_to_string: HashMap::new(),
        }
    }

    /// Register a string ID for a vertex
    pub fn register_vertex_string_id(&mut self, string_id: String, vertex_id: VertexId) {
        self.string_to_vertex.insert(string_id.clone(), vertex_id);
        self.vertex_to_string.insert(vertex_id, string_id);
    }

    /// Register a string ID for an edge
    pub fn register_edge_string_id(&mut self, string_id: String, edge_id: EdgeId) {
        self.string_to_edge.insert(string_id.clone(), edge_id);
        self.edge_to_string.insert(edge_id, string_id);
    }

    /// Get the numeric vertex ID for a string ID
    pub fn get_vertex_id(&self, string_id: &str) -> Option<VertexId> {
        self.string_to_vertex.get(string_id).copied()
    }

    /// Get the string ID for a numeric vertex ID
    pub fn get_string_id_for_vertex(&self, vertex_id: VertexId) -> Option<String> {
        self.vertex_to_string.get(&vertex_id).cloned()
    }

    /// Get the numeric edge ID for a string ID
    pub fn get_edge_id(&self, string_id: &str) -> Option<EdgeId> {
        self.string_to_edge.get(string_id).copied()
    }

    /// Get the string ID for a numeric edge ID
    pub fn get_string_id_for_edge(&self, edge_id: EdgeId) -> Option<String> {
        self.edge_to_string.get(&edge_id).cloned()
    }

    /// Check if a string ID exists for a vertex
    pub fn has_vertex_string_id(&self, string_id: &str) -> bool {
        self.string_to_vertex.contains_key(string_id)
    }

    /// Check if a string ID exists for an edge
    pub fn has_edge_string_id(&self, string_id: &str) -> bool {
        self.string_to_edge.contains_key(string_id)
    }
}

/// ID configuration
#[derive(Debug, Clone)]
pub struct IdConfig {
    pub enable_string_ids: bool,
    pub string_id_prefix: String,
    pub max_id_value: i64,
    pub use_uuid: bool,
}

impl Default for IdConfig {
    fn default() -> Self {
        Self {
            enable_string_ids: false,
            string_id_prefix: "ext_".to_string(),
            max_id_value: i64::MAX,
            use_uuid: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_id() {
        let id = VertexId::new(123);
        assert_eq!(id.as_i64(), 123);
        assert_eq!(format!("{}", id), "v123");
    }

    #[test]
    fn test_edge_id() {
        let id = EdgeId::new(456);
        assert_eq!(id.as_i64(), 456);
        assert_eq!(format!("{}", id), "e456");
    }

    #[test]
    fn test_tag_id() {
        let id = TagId::new(1);
        assert_eq!(id.as_i32(), 1);
        assert_eq!(format!("{}", id), "tag1");
    }

    #[test]
    fn test_edge_type() {
        let id = EdgeType::new(2);
        assert_eq!(id.as_i32(), 2);
        assert_eq!(format!("{}", id), "edge_type2");
    }

    #[test]
    fn test_space_id() {
        let id = SpaceId::new(10);
        assert_eq!(id.as_i32(), 10);
        assert_eq!(format!("{}", id), "space10");
    }

    #[test]
    fn test_index_id() {
        let id = IndexId::new(5);
        assert_eq!(id.as_i32(), 5);
        assert_eq!(format!("{}", id), "index5");
    }

    #[test]
    fn test_id_generator() {
        let gen = IdGenerator::new();

        let id1 = gen.generate_vertex_id();
        let id2 = gen.generate_vertex_id();

        assert!(id_utils::is_valid_vertex_id(id1));
        assert!(id_utils::is_valid_vertex_id(id2));
        assert_ne!(id1.as_i64(), id2.as_i64());
    }

    #[test]
    fn test_global_id_generator() {
        let id1 = gen_vertex_id();
        let id2 = gen_vertex_id();

        assert_ne!(id1.as_i64(), id2.as_i64());
    }

    #[test]
    fn test_uuid_generator() {
        let uuid_str = UuidGenerator::generate();
        assert!(UuidGenerator::is_valid(&uuid_str));

        let uuid = UuidGenerator::generate_uuid();
        assert_eq!(uuid.to_string().len(), 36); // Standard UUID length
    }

    #[test]
    fn test_string_id() {
        let string_id = StringId::new("test_id".to_string());
        assert_eq!(string_id.as_str(), "test_id");
        assert_eq!(format!("{}", string_id), "test_id");
    }

    #[test]
    fn test_id_registry() {
        let mut registry = IdRegistry::new();

        let vertex_id = VertexId::new(100);
        registry.register_vertex_string_id("vertex_100".to_string(), vertex_id);

        assert_eq!(registry.get_vertex_id("vertex_100"), Some(vertex_id));
        assert_eq!(
            registry.get_string_id_for_vertex(vertex_id),
            Some("vertex_100".to_string())
        );

        let edge_id = EdgeId::new(200);
        registry.register_edge_string_id("edge_200".to_string(), edge_id);

        assert_eq!(registry.get_edge_id("edge_200"), Some(edge_id));
        assert_eq!(
            registry.get_string_id_for_edge(edge_id),
            Some("edge_200".to_string())
        );
    }

    #[test]
    fn test_id_utils() {
        let vertex_id = id_utils::string_to_vertex_id("test_vertex");
        assert!(id_utils::is_valid_vertex_id(vertex_id));

        let edge_id = id_utils::string_to_edge_id("test_edge");
        assert!(id_utils::is_valid_edge_id(edge_id));

        let combined = id_utils::combine_vertex_edge_ids(VertexId::new(1), EdgeId::new(2));
        assert_eq!(combined, "1_2");
    }
}
