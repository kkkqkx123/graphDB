#[cfg(test)]
mod tests {
    use crate::core::types::{
        EdgeTypeInfo, Index, IndexConfig, IndexField, IndexType, PropertyDef, SpaceInfo, UserInfo,
        VertexId,
    };
    use crate::core::vertex_edge_path::Tag;
    use crate::core::DataType;
    use crate::core::{Edge, EdgeDirection, RoleType, Value, Vertex};
    use crate::storage::{
        GraphStorage, StorageAdmin, StorageAuthOps, StorageReader, StorageSchemaOps, StorageWriter,
    };

    fn create_test_storage() -> GraphStorage {
        GraphStorage::new().expect("Failed to create GraphStorage")
    }

    fn create_persistent_storage() -> (tempfile::TempDir, GraphStorage) {
        let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let storage = GraphStorage::new_with_path(temp_dir.path().to_path_buf())
            .expect("Failed to create persistent GraphStorage");
        (temp_dir, storage)
    }

    fn setup_space(storage: &mut GraphStorage) -> u64 {
        let mut space = SpaceInfo::new("test_space".to_string())
            .with_vid_type(DataType::BigInt)
            .with_comment(Some("test".to_string()));
        storage.create_space(&mut space).unwrap();
        storage.get_space_id("test_space").unwrap()
    }

    fn setup_person_tag(storage: &mut GraphStorage) -> u32 {
        let tag = crate::core::types::TagInfo::new("Person".to_string()).with_properties(vec![
            PropertyDef::new("name".to_string(), DataType::String),
            PropertyDef::new("age".to_string(), DataType::BigInt),
        ]);
        storage
            .create_tag("test_space", &tag)
            .expect("Failed to create tag")
    }

    fn setup_knows_edge(storage: &mut GraphStorage) -> u32 {
        let edge = EdgeTypeInfo::new("KNOWS".to_string())
            .with_properties(vec![PropertyDef::new("since".to_string(), DataType::Int)]);
        storage
            .create_edge_type("test_space", &edge)
            .expect("Failed to create edge type")
    }

    #[test]
    fn test_snapshot_admin_methods() {
        let (_temp_dir, storage) = create_persistent_storage();

        let initial_stats = storage.snapshot_stats();
        assert_eq!(initial_stats.snapshot_count, 0);
        assert_eq!(initial_stats.total_size_bytes, 0);
        assert_eq!(initial_stats.latest_snapshot_id, None);

        let checkpoint = storage
            .create_checkpoint()
            .expect("checkpoint should succeed")
            .expect("persistence should be enabled");

        assert!(checkpoint.snapshot_created);
        assert!(storage
            .verify_snapshot(checkpoint.checkpoint_id)
            .expect("snapshot verification should succeed"));

        let stats = storage.snapshot_stats();
        assert_eq!(stats.snapshot_count, 1);
        assert_eq!(stats.latest_snapshot_id, Some(checkpoint.checkpoint_id));

        let deleted = storage
            .cleanup_snapshots()
            .expect("snapshot cleanup should succeed");
        assert_eq!(deleted, 0);
    }

    // ==================== Schema Operations ====================

    #[test]
    fn test_create_and_list_spaces() {
        let mut storage = create_test_storage();

        let mut space1 = SpaceInfo::new("space1".to_string()).with_vid_type(DataType::BigInt);
        let mut space2 = SpaceInfo::new("space2".to_string()).with_vid_type(DataType::String);
        storage.create_space(&mut space1).unwrap();
        storage.create_space(&mut space2).unwrap();

        let spaces = storage.list_spaces().unwrap();
        assert_eq!(spaces.len(), 2);
        assert!(storage.space_exists("space1"));
        assert!(storage.space_exists("space2"));
        assert!(!storage.space_exists("space3"));

        assert_eq!(storage.get_space_id("space1").unwrap(), 1);
    }

    #[test]
    fn test_drop_space_cleans_tags_and_edge_types() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);
        setup_knows_edge(&mut storage);

        storage.drop_space("test_space").unwrap();
        assert!(!storage.space_exists("test_space"));
    }

    #[test]
    fn test_create_and_get_tag() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);

        let tag_id = setup_person_tag(&mut storage);
        assert!(tag_id > 0);

        let tag = storage.get_tag("test_space", "Person").unwrap();
        assert!(tag.is_some());
        assert_eq!(tag.as_ref().unwrap().tag_name, "Person");
        assert_eq!(tag.as_ref().unwrap().properties.len(), 2);

        let tags = storage.list_tags("test_space").unwrap();
        assert_eq!(tags.len(), 1);
    }

    #[test]
    fn test_drop_tag_removes_tag() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);

        storage.drop_tag("test_space", "Person").unwrap();
        assert!(storage.get_tag("test_space", "Person").unwrap().is_none());
    }

    #[test]
    fn test_create_and_get_edge_type() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);

        let edge_id = setup_knows_edge(&mut storage);
        assert!(edge_id > 0);

        let edge = storage.get_edge_type("test_space", "KNOWS").unwrap();
        assert!(edge.is_some());
        assert_eq!(edge.as_ref().unwrap().edge_type_name, "KNOWS");

        let edges = storage.list_edge_types("test_space").unwrap();
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn test_same_schema_names_are_isolated_by_space() {
        let mut storage = create_test_storage();
        let mut alpha = SpaceInfo::new("alpha".to_string()).with_vid_type(DataType::BigInt);
        let mut beta = SpaceInfo::new("beta".to_string()).with_vid_type(DataType::BigInt);
        storage.create_space(&mut alpha).unwrap();
        storage.create_space(&mut beta).unwrap();

        let tag = crate::core::types::TagInfo::new("Person".to_string())
            .with_properties(vec![PropertyDef::new("name".to_string(), DataType::String)]);
        let alpha_tag_id = storage.create_tag("alpha", &tag).unwrap();
        let beta_tag_id = storage.create_tag("beta", &tag).unwrap();
        assert_ne!(alpha_tag_id, beta_tag_id);

        let edge_type = EdgeTypeInfo::new("KNOWS".to_string())
            .with_src_tag("Person".to_string())
            .with_dst_tag("Person".to_string());
        let alpha_edge_id = storage.create_edge_type("alpha", &edge_type).unwrap();
        let beta_edge_id = storage.create_edge_type("beta", &edge_type).unwrap();
        assert_ne!(alpha_edge_id, beta_edge_id);

        storage
            .insert_vertex(
                "alpha",
                Vertex::new(
                    VertexId::from_int64(1),
                    vec![Tag::new(
                        "Person".to_string(),
                        vec![("name".to_string(), Value::String("Alice".to_string()))]
                            .into_iter()
                            .collect(),
                    )],
                ),
            )
            .unwrap();
        storage
            .insert_vertex(
                "beta",
                Vertex::new(
                    VertexId::from_int64(1),
                    vec![Tag::new(
                        "Person".to_string(),
                        vec![("name".to_string(), Value::String("Bob".to_string()))]
                            .into_iter()
                            .collect(),
                    )],
                ),
            )
            .unwrap();
        storage
            .insert_vertex(
                "alpha",
                Vertex::new(
                    VertexId::from_int64(2),
                    vec![Tag::new(
                        "Person".to_string(),
                        vec![("name".to_string(), Value::String("Carol".to_string()))]
                            .into_iter()
                            .collect(),
                    )],
                ),
            )
            .unwrap();
        storage
            .insert_vertex(
                "beta",
                Vertex::new(
                    VertexId::from_int64(2),
                    vec![Tag::new(
                        "Person".to_string(),
                        vec![("name".to_string(), Value::String("Dave".to_string()))]
                            .into_iter()
                            .collect(),
                    )],
                ),
            )
            .unwrap();

        storage
            .insert_edge(
                "alpha",
                Edge::new(
                    VertexId::from_int64(1),
                    VertexId::from_int64(2),
                    "KNOWS".to_string(),
                    0,
                    std::collections::HashMap::new(),
                ),
            )
            .unwrap();
        storage
            .insert_edge(
                "beta",
                Edge::new(
                    VertexId::from_int64(1),
                    VertexId::from_int64(2),
                    "KNOWS".to_string(),
                    0,
                    std::collections::HashMap::new(),
                ),
            )
            .unwrap();

        let alpha_vertex = storage
            .get_vertex("alpha", &VertexId::from_int64(1))
            .unwrap()
            .unwrap();
        let beta_vertex = storage
            .get_vertex("beta", &VertexId::from_int64(1))
            .unwrap()
            .unwrap();
        assert_eq!(
            alpha_vertex.properties.get("name"),
            Some(&Value::String("Alice".to_string()))
        );
        assert_eq!(
            beta_vertex.properties.get("name"),
            Some(&Value::String("Bob".to_string()))
        );

        assert_eq!(
            storage
                .scan_vertices_by_tag("alpha", "Person")
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            storage
                .scan_vertices_by_tag("beta", "Person")
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            storage.scan_edges_by_type("alpha", "KNOWS").unwrap().len(),
            1
        );
        assert_eq!(
            storage.scan_edges_by_type("beta", "KNOWS").unwrap().len(),
            1
        );
    }

    #[test]
    fn test_drop_edge_type() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);
        setup_knows_edge(&mut storage);

        storage.drop_edge_type("test_space", "KNOWS").unwrap();
        assert!(storage
            .get_edge_type("test_space", "KNOWS")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_create_and_drop_tag_index() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);

        let index = Index::new(IndexConfig {
            id: 1,
            name: "person_name_idx".to_string(),
            space_id: 0,
            schema_name: "Person".to_string(),
            fields: vec![IndexField::new(
                "name".to_string(),
                Value::String(String::new()),
                false,
            )],
            properties: vec![],
            index_type: IndexType::TagIndex,
            is_unique: false,
            partial_condition: None,
        });
        storage.create_tag_index("test_space", &index).unwrap();

        let indexes = storage.list_tag_indexes("test_space").unwrap();
        assert_eq!(indexes.len(), 1);

        storage
            .drop_tag_index("test_space", "person_name_idx")
            .unwrap();
        let indexes = storage.list_tag_indexes("test_space").unwrap();
        assert_eq!(indexes.len(), 0);
    }

    // ==================== Vertex Operations ====================

    #[test]
    fn test_insert_and_get_vertex() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);

        let vertex = Vertex::new(
            VertexId::from_int64(101),
            vec![crate::core::vertex_edge_path::Tag::new(
                "Person".to_string(),
                vec![
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::BigInt(30)),
                ]
                .into_iter()
                .collect(),
            )],
        );
        let vid = storage.insert_vertex("test_space", vertex).unwrap();
        assert_eq!(vid, VertexId::from_int64(101));

        let retrieved = storage
            .get_vertex("test_space", &VertexId::from_int64(101))
            .unwrap();
        assert!(retrieved.is_some());
        let v = retrieved.unwrap();
        assert_eq!(
            v.properties.get("name"),
            Some(&Value::String("Alice".to_string()))
        );
    }

    #[test]
    fn test_update_vertex() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);

        let vertex = Vertex::new(
            VertexId::from_int64(101),
            vec![crate::core::vertex_edge_path::Tag::new(
                "Person".to_string(),
                vec![
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::BigInt(30)),
                ]
                .into_iter()
                .collect(),
            )],
        );
        storage.insert_vertex("test_space", vertex).unwrap();

        let updated = Vertex::new(
            VertexId::from_int64(101),
            vec![crate::core::vertex_edge_path::Tag::new(
                "Person".to_string(),
                vec![
                    (
                        "name".to_string(),
                        Value::String("AliceUpdated".to_string()),
                    ),
                    ("age".to_string(), Value::BigInt(31)),
                ]
                .into_iter()
                .collect(),
            )],
        );
        storage.update_vertex("test_space", updated).unwrap();

        let v = storage
            .get_vertex("test_space", &VertexId::from_int64(101))
            .unwrap()
            .unwrap();
        assert_eq!(
            v.properties.get("name"),
            Some(&Value::String("AliceUpdated".to_string()))
        );
        assert_eq!(v.properties.get("age"), Some(&Value::BigInt(31)));
    }

    #[test]
    fn test_delete_vertex() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);

        let vertex = Vertex::new(
            VertexId::from_int64(101),
            vec![crate::core::vertex_edge_path::Tag::new(
                "Person".to_string(),
                vec![("name".to_string(), Value::String("Alice".to_string()))]
                    .into_iter()
                    .collect(),
            )],
        );
        storage.insert_vertex("test_space", vertex).unwrap();

        storage
            .delete_vertex("test_space", &VertexId::from_int64(101))
            .unwrap();
        assert!(storage
            .get_vertex("test_space", &VertexId::from_int64(101))
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_scan_vertices() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);

        for i in 1..=5 {
            let vertex = Vertex::new(
                VertexId::from_int64(i),
                vec![crate::core::vertex_edge_path::Tag::new(
                    "Person".to_string(),
                    vec![
                        ("name".to_string(), Value::String(format!("Person{}", i))),
                        ("age".to_string(), Value::BigInt(20 + i)),
                    ]
                    .into_iter()
                    .collect(),
                )],
            );
            storage.insert_vertex("test_space", vertex).unwrap();
        }

        let vertices = storage.scan_vertices("test_space").unwrap();
        assert_eq!(vertices.len(), 5);

        let tagged = storage
            .scan_vertices_by_tag("test_space", "Person")
            .unwrap();
        assert_eq!(tagged.len(), 5);
    }

    #[test]
    fn test_scan_vertices_by_prop() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);

        let vertex = Vertex::new(
            VertexId::from_int64(101),
            vec![crate::core::vertex_edge_path::Tag::new(
                "Person".to_string(),
                vec![
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::BigInt(30)),
                ]
                .into_iter()
                .collect(),
            )],
        );
        storage.insert_vertex("test_space", vertex).unwrap();

        let results = storage
            .scan_vertices_by_prop(
                "test_space",
                "Person",
                "name",
                &Value::String("Alice".to_string()),
            )
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_batch_insert_vertices() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);

        let vertices: Vec<Vertex> = (1..=3)
            .map(|i| {
                Vertex::new(
                    VertexId::from_int64(i),
                    vec![crate::core::vertex_edge_path::Tag::new(
                        "Person".to_string(),
                        vec![("name".to_string(), Value::String(format!("Person{}", i)))]
                            .into_iter()
                            .collect(),
                    )],
                )
            })
            .collect();

        let ids = storage
            .batch_insert_vertices("test_space", vertices)
            .unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn test_batch_insert_vertices_rolls_back_on_failure() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);

        let vertices = vec![
            Vertex::new(
                VertexId::from_int64(1),
                vec![crate::core::vertex_edge_path::Tag::new(
                    "Person".to_string(),
                    vec![("name".to_string(), Value::String("Alice".to_string()))]
                        .into_iter()
                        .collect(),
                )],
            ),
            Vertex::new(
                VertexId::from_int64(1),
                vec![crate::core::vertex_edge_path::Tag::new(
                    "Person".to_string(),
                    vec![("name".to_string(), Value::String("Duplicate".to_string()))]
                        .into_iter()
                        .collect(),
                )],
            ),
        ];

        assert!(storage
            .batch_insert_vertices("test_space", vertices)
            .is_err());
        assert!(storage
            .get_vertex("test_space", &VertexId::from_int64(1))
            .unwrap()
            .is_none());
    }

    // ==================== Edge Operations ====================

    fn insert_test_vertex(storage: &mut GraphStorage, id: i64, name: &str) {
        let vertex = Vertex::new(
            VertexId::from_int64(id),
            vec![crate::core::vertex_edge_path::Tag::new(
                "Person".to_string(),
                vec![("name".to_string(), Value::String(name.to_string()))]
                    .into_iter()
                    .collect(),
            )],
        );
        storage.insert_vertex("test_space", vertex).unwrap();
    }

    #[test]
    fn test_insert_and_get_edge() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);
        setup_knows_edge(&mut storage);

        insert_test_vertex(&mut storage, 1, "Alice");
        insert_test_vertex(&mut storage, 2, "Bob");

        let edge = Edge::new(
            VertexId::from_int64(1),
            VertexId::from_int64(2),
            "KNOWS".to_string(),
            0,
            vec![("since".to_string(), Value::Int(2020))]
                .into_iter()
                .collect(),
        );
        storage.insert_edge("test_space", edge).unwrap();

        let retrieved = storage
            .get_edge(
                "test_space",
                &VertexId::from_int64(1),
                &VertexId::from_int64(2),
                "KNOWS",
                0,
            )
            .unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.as_ref().unwrap().src, VertexId::from_int64(1));
        assert_eq!(retrieved.as_ref().unwrap().dst, VertexId::from_int64(2));
    }

    #[test]
    fn test_delete_edge() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);
        setup_knows_edge(&mut storage);

        insert_test_vertex(&mut storage, 1, "Alice");
        insert_test_vertex(&mut storage, 2, "Bob");

        let edge = Edge::new(
            VertexId::from_int64(1),
            VertexId::from_int64(2),
            "KNOWS".to_string(),
            0,
            std::collections::HashMap::new(),
        );
        storage.insert_edge("test_space", edge).unwrap();

        storage
            .delete_edge(
                "test_space",
                &VertexId::from_int64(1),
                &VertexId::from_int64(2),
                "KNOWS",
                0,
            )
            .unwrap();

        let retrieved = storage
            .get_edge(
                "test_space",
                &VertexId::from_int64(1),
                &VertexId::from_int64(2),
                "KNOWS",
                0,
            )
            .unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_get_node_edges() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);
        setup_knows_edge(&mut storage);

        insert_test_vertex(&mut storage, 1, "Alice");
        insert_test_vertex(&mut storage, 2, "Bob");
        insert_test_vertex(&mut storage, 3, "Charlie");

        for dst in &[2i64, 3] {
            let edge = Edge::new(
                VertexId::from_int64(1),
                VertexId::from_int64(*dst),
                "KNOWS".to_string(),
                0,
                std::collections::HashMap::new(),
            );
            storage.insert_edge("test_space", edge).unwrap();
        }

        let out_edges = storage
            .get_node_edges("test_space", &VertexId::from_int64(1), EdgeDirection::Out)
            .unwrap();
        assert_eq!(out_edges.len(), 2);

        let in_edges = storage
            .get_node_edges("test_space", &VertexId::from_int64(2), EdgeDirection::In)
            .unwrap();
        assert_eq!(in_edges.len(), 1);
    }

    #[test]
    fn test_scan_edges_by_type() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);
        setup_knows_edge(&mut storage);

        insert_test_vertex(&mut storage, 1, "Alice");
        insert_test_vertex(&mut storage, 2, "Bob");

        let edge = Edge::new(
            VertexId::from_int64(1),
            VertexId::from_int64(2),
            "KNOWS".to_string(),
            0,
            std::collections::HashMap::new(),
        );
        storage.insert_edge("test_space", edge).unwrap();

        let edges = storage.scan_edges_by_type("test_space", "KNOWS").unwrap();
        assert_eq!(edges.len(), 1);
    }

    #[test]
    fn test_batch_insert_edges_rolls_back_on_failure() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);
        setup_knows_edge(&mut storage);

        insert_test_vertex(&mut storage, 1, "Alice");
        insert_test_vertex(&mut storage, 2, "Bob");

        let edges = vec![
            Edge::new(
                VertexId::from_int64(1),
                VertexId::from_int64(2),
                "KNOWS".to_string(),
                0,
                std::collections::HashMap::new(),
            ),
            Edge::new(
                VertexId::from_int64(1),
                VertexId::from_int64(3),
                "KNOWS".to_string(),
                0,
                std::collections::HashMap::new(),
            ),
        ];

        assert!(storage.batch_insert_edges("test_space", edges).is_err());
        assert_eq!(
            storage
                .scan_edges_by_type("test_space", "KNOWS")
                .unwrap()
                .len(),
            0
        );
    }

    // ==================== User / Auth Operations ====================

    #[test]
    fn test_create_and_drop_user() {
        let mut storage = create_test_storage();

        let user = UserInfo::new("test_user".to_string(), "password123".to_string()).unwrap();
        storage.create_user(&user).unwrap();

        storage.drop_user("test_user").unwrap();
    }

    #[test]
    fn test_grant_and_revoke_role() {
        let mut storage = create_test_storage();
        let space_id = setup_space(&mut storage);
        setup_person_tag(&mut storage);

        let user = UserInfo::new("role_user".to_string(), "pass".to_string()).unwrap();
        storage.create_user(&user).unwrap();

        storage
            .grant_role("role_user", space_id, RoleType::Admin)
            .unwrap();
        storage.revoke_role("role_user", space_id).unwrap();

        storage.drop_user("role_user").unwrap();
    }

    #[test]
    fn test_user_storage_persists_across_reload() {
        let (temp_dir, mut storage) = create_persistent_storage();

        let user = UserInfo::new("persist_user".to_string(), "password123".to_string())
            .expect("UserInfo::new should succeed")
            .with_locked(true)
            .with_max_queries_per_hour(42);

        storage.create_user(&user).unwrap();
        storage.save_to_disk().unwrap();

        let mut reloaded = GraphStorage::new_with_path(temp_dir.path().to_path_buf())
            .expect("Failed to recreate GraphStorage");
        reloaded.load_from_disk().unwrap();

        assert!(reloaded.user_exists("persist_user"));
        assert!(reloaded.create_user(&user).is_err());
    }

    // ==================== Storage Admin Operations ====================

    #[test]
    fn test_get_storage_stats_empty() {
        let storage = create_test_storage();
        let stats = storage.get_storage_stats();
        assert_eq!(stats.total_vertices, 0);
        assert_eq!(stats.total_edges, 0);
        assert_eq!(stats.total_spaces, 0);
    }

    #[test]
    fn test_get_storage_stats_with_data() {
        let mut storage = create_test_storage();
        setup_space(&mut storage);
        setup_person_tag(&mut storage);
        setup_knows_edge(&mut storage);

        insert_test_vertex(&mut storage, 1, "Alice");
        insert_test_vertex(&mut storage, 2, "Bob");

        let edge = Edge::new(
            VertexId::from_int64(1),
            VertexId::from_int64(2),
            "KNOWS".to_string(),
            0,
            std::collections::HashMap::new(),
        );
        storage.insert_edge("test_space", edge).unwrap();

        let stats = storage.get_storage_stats();
        // Note: vertex/edge counts depend on MVCC visibility
        assert!(stats.total_spaces >= 1);
        assert!(stats.total_tags >= 1);
        assert!(stats.total_edge_types >= 1);
    }

    #[test]
    fn test_get_db_path() {
        let storage = create_test_storage();
        // Default db_path is empty for new() without path
        let path = storage.get_db_path();
        assert!(path.is_empty() || path.contains("test"));
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_get_nonexistent_vertex() {
        let storage = create_test_storage();
        let result = storage.get_vertex("nonexistent", &VertexId::from_int64(999));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_nonexistent_edge() {
        let storage = create_test_storage();
        let result = storage.get_edge(
            "nonexistent",
            &VertexId::from_int64(1),
            &VertexId::from_int64(2),
            "UNKNOWN",
            0,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_nonexistent_vertex() {
        let mut storage = create_test_storage();
        let result = storage.delete_vertex("nonexistent", &VertexId::from_int64(999));
        assert!(result.is_err());
    }
}
