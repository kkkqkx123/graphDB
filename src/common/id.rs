use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;
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
        let hash = hash_string(s);
        VertexId::new(hash as i64)
    }

    /// Convert a string to an edge ID (useful for external IDs)
    pub fn string_to_edge_id(s: &str) -> EdgeId {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_id() {
        let id = VertexId::new(123);
        assert_eq!(id.as_i64(), 123);
        assert_eq!(id.to_string(), "v123");
    }

    #[test]
    fn test_edge_id() {
        let id = EdgeId::new(456);
        assert_eq!(id.as_i64(), 456);
        assert_eq!(id.to_string(), "e456");
    }

    #[test]
    fn test_tag_id() {
        let id = TagId::new(789);
        assert_eq!(id.as_i32(), 789);
        assert_eq!(id.to_string(), "tag789");
    }

    #[test]
    fn test_uuid_generator() {
        let uuid1 = UuidGenerator::generate();
        let uuid2 = UuidGenerator::generate();
        assert_ne!(uuid1, uuid2);
        assert!(UuidGenerator::is_valid(&uuid1));
    }

    #[test]
    fn test_string_id() {
        let id = StringId::new("test_id".to_string());
        assert_eq!(id.as_str(), "test_id");
        assert_eq!(id.to_string(), "test_id");
    }

    #[test]
    fn test_id_utils() {
        let vid1 = id_utils::string_to_vertex_id("test1");
        let vid2 = id_utils::string_to_vertex_id("test1");
        let vid3 = id_utils::string_to_vertex_id("test2");

        assert_eq!(vid1.as_i64(), vid2.as_i64());
        assert_ne!(vid1.as_i64(), vid3.as_i64());

        assert!(id_utils::is_valid_vertex_id(vid1));
        assert!(!id_utils::is_valid_vertex_id(VertexId::new(0)));
    }
}
