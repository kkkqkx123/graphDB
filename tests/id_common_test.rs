use graphdb::common::id::*;

#[test]
fn test_vertex_id() {
    let id = VertexId::new(123);
    assert_eq!(id.as_i64(), 123);
    assert_eq!(format!("{}", id), "v123");
}

#[test]
fn test_vertex_id_display() {
    let id = VertexId::new(42);
    let display = format!("{}", id);
    assert_eq!(display, "v42");
}

#[test]
fn test_vertex_id_serialization() {
    let id = VertexId::new(123);
    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: VertexId = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(id, deserialized);
}

#[test]
fn test_edge_id() {
    let id = EdgeId::new(456);
    assert_eq!(id.as_i64(), 456);
    assert_eq!(format!("{}", id), "e456");
}

#[test]
fn test_edge_id_display() {
    let id = EdgeId::new(99);
    let display = format!("{}", id);
    assert_eq!(display, "e99");
}

#[test]
fn test_edge_id_serialization() {
    let id = EdgeId::new(456);
    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: EdgeId = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(id, deserialized);
}

#[test]
fn test_tag_id() {
    let id = TagId::new(1);
    assert_eq!(id.as_i32(), 1);
    assert_eq!(format!("{}", id), "tag1");
}

#[test]
fn test_tag_id_display() {
    let id = TagId::new(5);
    let display = format!("{}", id);
    assert_eq!(display, "tag5");
}

#[test]
fn test_tag_id_serialization() {
    let id = TagId::new(1);
    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: TagId = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(id, deserialized);
}

#[test]
fn test_edge_type() {
    let id = EdgeType::new(2);
    assert_eq!(id.as_i32(), 2);
    assert_eq!(format!("{}", id), "edge_type2");
}

#[test]
fn test_edge_type_display() {
    let id = EdgeType::new(10);
    let display = format!("{}", id);
    assert_eq!(display, "edge_type10");
}

#[test]
fn test_edge_type_serialization() {
    let id = EdgeType::new(2);
    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: EdgeType = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(id, deserialized);
}

#[test]
fn test_space_id() {
    let id = SpaceId::new(10);
    assert_eq!(id.as_i32(), 10);
    assert_eq!(format!("{}", id), "space10");
}

#[test]
fn test_space_id_display() {
    let id = SpaceId::new(3);
    let display = format!("{}", id);
    assert_eq!(display, "space3");
}

#[test]
fn test_space_id_serialization() {
    let id = SpaceId::new(10);
    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: SpaceId = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(id, deserialized);
}

#[test]
fn test_index_id() {
    let id = IndexId::new(5);
    assert_eq!(id.as_i32(), 5);
    assert_eq!(format!("{}", id), "index5");
}

#[test]
fn test_index_id_display() {
    let id = IndexId::new(7);
    let display = format!("{}", id);
    assert_eq!(display, "index7");
}

#[test]
fn test_index_id_serialization() {
    let id = IndexId::new(5);
    let json = serde_json::to_string(&id).expect("Failed to serialize");
    let deserialized: IndexId = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(id, deserialized);
}

#[test]
fn test_id_generator_new() {
    let gen = IdGenerator::new();
    assert!(gen.generate_vertex_id().as_i64() > 0);
}

#[test]
fn test_id_generator_vertex() {
    let gen = IdGenerator::new();
    let id1 = gen.generate_vertex_id();
    let id2 = gen.generate_vertex_id();

    assert_ne!(id1.as_i64(), id2.as_i64());
    assert!(id1.as_i64() > 0);
    assert!(id2.as_i64() > id1.as_i64());
}

#[test]
fn test_id_generator_edge() {
    let gen = IdGenerator::new();
    let id1 = gen.generate_edge_id();
    let id2 = gen.generate_edge_id();

    assert_ne!(id1.as_i64(), id2.as_i64());
    assert!(id1.as_i64() > 0);
    assert!(id2.as_i64() > id1.as_i64());
}

#[test]
fn test_id_generator_tag() {
    let gen = IdGenerator::new();
    let id1 = gen.generate_tag_id();
    let id2 = gen.generate_tag_id();

    assert_ne!(id1.as_i32(), id2.as_i32());
    assert!(id1.as_i32() > 0);
    assert!(id2.as_i32() > id1.as_i32());
}

#[test]
fn test_id_generator_edge_type() {
    let gen = IdGenerator::new();
    let id1 = gen.generate_edge_type();
    let id2 = gen.generate_edge_type();

    assert_ne!(id1.as_i32(), id2.as_i32());
    assert!(id1.as_i32() > 0);
    assert!(id2.as_i32() > id1.as_i32());
}

#[test]
fn test_id_generator_space_id() {
    let gen = IdGenerator::new();
    let id1 = gen.generate_space_id();
    let id2 = gen.generate_space_id();

    assert_ne!(id1.as_i32(), id2.as_i32());
    assert!(id1.as_i32() > 0);
    assert!(id2.as_i32() > id1.as_i32());
}

#[test]
fn test_id_generator_index_id() {
    let gen = IdGenerator::new();
    let id1 = gen.generate_index_id();
    let id2 = gen.generate_index_id();

    assert_ne!(id1.as_i32(), id2.as_i32());
    assert!(id1.as_i32() > 0);
    assert!(id2.as_i32() > id1.as_i32());
}

#[test]
#[ignore]
fn test_id_generator_reset() {
    let gen = IdGenerator::new();
    let id1 = gen.generate_vertex_id();
    gen.reset();
    let id2 = gen.generate_vertex_id();

    assert_ne!(id1.as_i64(), id2.as_i64());
}

#[test]
fn test_global_id_generator() {
    let id1 = gen_vertex_id();
    let id2 = gen_vertex_id();

    assert_ne!(id1.as_i64(), id2.as_i64());
}

#[test]
fn test_global_gen_functions() {
    let vertex_id = gen_vertex_id();
    let edge_id = gen_edge_id();
    let tag_id = gen_tag_id();
    let edge_type = gen_edge_type();
    let space_id = gen_space_id();
    let index_id = gen_index_id();

    assert!(vertex_id.as_i64() > 0);
    assert!(edge_id.as_i64() > 0);
    assert!(tag_id.as_i32() > 0);
    assert!(edge_type.as_i32() > 0);
    assert!(space_id.as_i32() > 0);
    assert!(index_id.as_i32() > 0);
}

#[test]
fn test_uuid_generator_generate() {
    let uuid_str = UuidGenerator::generate();
    assert!(!uuid_str.is_empty());
    assert_eq!(uuid_str.len(), 36); // Standard UUID format with hyphens
}

#[test]
fn test_uuid_generator_generate_uuid() {
    let uuid = UuidGenerator::generate_uuid();
    let uuid_str = uuid.to_string();
    assert_eq!(uuid_str.len(), 36);
}

#[test]
fn test_uuid_generator_from_string() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let uuid = UuidGenerator::from_string(uuid_str);
    assert!(uuid.is_ok());
    assert_eq!(uuid.unwrap().to_string(), uuid_str);
}

#[test]
fn test_uuid_generator_from_string_invalid() {
    let invalid_uuid = "not-a-uuid";
    let result = UuidGenerator::from_string(invalid_uuid);
    assert!(result.is_err());
}

#[test]
fn test_uuid_generator_is_valid() {
    let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
    let invalid_uuid = "not-a-uuid";

    assert!(UuidGenerator::is_valid(valid_uuid));
    assert!(!UuidGenerator::is_valid(invalid_uuid));
}

#[test]
fn test_string_id_new() {
    let string_id = StringId::new("test_id".to_string());
    assert_eq!(string_id.as_str(), "test_id");
    assert_eq!(string_id.into_string(), "test_id");
}

#[test]
fn test_string_id_display() {
    let string_id = StringId::new("display_test".to_string());
    assert_eq!(format!("{}", string_id), "display_test");
}

#[test]
fn test_string_id_as_ref() {
    let string_id = StringId::new("ref_test".to_string());
    assert_eq!(string_id.as_ref(), "ref_test");
}

#[test]
fn test_string_id_serialization() {
    let string_id = StringId::new("serialize_test".to_string());
    let json = serde_json::to_string(&string_id).expect("Failed to serialize");
    let deserialized: StringId = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(string_id, deserialized);
}

#[test]
fn test_id_registry_new() {
    let registry = IdRegistry::new();
    assert!(registry.get_vertex_id("test").is_none());
}

#[test]
fn test_id_registry_vertex_string_id() {
    let mut registry = IdRegistry::new();

    let vertex_id = VertexId::new(100);
    registry.register_vertex_string_id("vertex_100".to_string(), vertex_id);

    assert_eq!(registry.get_vertex_id("vertex_100"), Some(vertex_id));
    assert_eq!(
        registry.get_string_id_for_vertex(vertex_id),
        Some("vertex_100".to_string())
    );
}

#[test]
fn test_id_registry_edge_string_id() {
    let mut registry = IdRegistry::new();

    let edge_id = EdgeId::new(200);
    registry.register_edge_string_id("edge_200".to_string(), edge_id);

    assert_eq!(registry.get_edge_id("edge_200"), Some(edge_id));
    assert_eq!(
        registry.get_string_id_for_edge(edge_id),
        Some("edge_200".to_string())
    );
}

#[test]
fn test_id_registry_has_vertex_string_id() {
    let mut registry = IdRegistry::new();

    assert!(!registry.has_vertex_string_id("test"));

    let vertex_id = VertexId::new(100);
    registry.register_vertex_string_id("test".to_string(), vertex_id);

    assert!(registry.has_vertex_string_id("test"));
}

#[test]
fn test_id_registry_has_edge_string_id() {
    let mut registry = IdRegistry::new();

    assert!(!registry.has_edge_string_id("test"));

    let edge_id = EdgeId::new(200);
    registry.register_edge_string_id("test".to_string(), edge_id);

    assert!(registry.has_edge_string_id("test"));
}

#[test]
fn test_id_config_default() {
    let config = IdConfig::default();
    assert!(!config.enable_string_ids);
    assert_eq!(config.string_id_prefix, "ext_");
    assert_eq!(config.max_id_value, i64::MAX);
    assert!(!config.use_uuid);
}

#[test]
fn test_id_config_custom() {
    let config = IdConfig {
        enable_string_ids: true,
        string_id_prefix: "custom_".to_string(),
        max_id_value: 1000,
        use_uuid: true,
    };

    assert!(config.enable_string_ids);
    assert_eq!(config.string_id_prefix, "custom_");
    assert_eq!(config.max_id_value, 1000);
    assert!(config.use_uuid);
}

#[test]
fn test_id_utils_string_to_vertex_id() {
    let vertex_id = id_utils::string_to_vertex_id("test_vertex");
    assert!(id_utils::is_valid_vertex_id(vertex_id));
}

#[test]
fn test_id_utils_string_to_edge_id() {
    let edge_id = id_utils::string_to_edge_id("test_edge");
    assert!(id_utils::is_valid_edge_id(edge_id));
}

#[test]
fn test_id_utils_is_valid_vertex_id() {
    let valid_id = VertexId::new(1);
    let invalid_id = VertexId::new(0);

    assert!(id_utils::is_valid_vertex_id(valid_id));
    assert!(!id_utils::is_valid_vertex_id(invalid_id));
}

#[test]
fn test_id_utils_is_valid_edge_id() {
    let valid_id = EdgeId::new(1);
    let invalid_id = EdgeId::new(0);

    assert!(id_utils::is_valid_edge_id(valid_id));
    assert!(!id_utils::is_valid_edge_id(invalid_id));
}

#[test]
fn test_id_utils_is_valid_tag_id() {
    let valid_id = TagId::new(1);
    let invalid_id = TagId::new(0);

    assert!(id_utils::is_valid_tag_id(valid_id));
    assert!(!id_utils::is_valid_tag_id(invalid_id));
}

#[test]
fn test_id_utils_is_valid_edge_type() {
    let valid_id = EdgeType::new(1);
    let invalid_id = EdgeType::new(0);

    assert!(id_utils::is_valid_edge_type(valid_id));
    assert!(!id_utils::is_valid_edge_type(invalid_id));
}

#[test]
fn test_id_utils_combine_vertex_edge_ids() {
    let combined = id_utils::combine_vertex_edge_ids(VertexId::new(1), EdgeId::new(2));
    assert_eq!(combined, "1_2");
}

#[test]
fn test_id_clone() {
    let id1 = VertexId::new(123);
    let id2 = id1;

    assert_eq!(id1.as_i64(), 123);
    assert_eq!(id2.as_i64(), 123);
}

#[test]
fn test_id_partial_eq() {
    let id1 = VertexId::new(123);
    let id2 = VertexId::new(123);
    let id3 = VertexId::new(456);

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_id_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let id1 = VertexId::new(123);
    let id2 = VertexId::new(123);
    let id3 = VertexId::new(456);

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    let mut hasher3 = DefaultHasher::new();

    id1.hash(&mut hasher1);
    id2.hash(&mut hasher2);
    id3.hash(&mut hasher3);

    assert_eq!(hasher1.finish(), hasher2.finish());
    assert_ne!(hasher1.finish(), hasher3.finish());
}
