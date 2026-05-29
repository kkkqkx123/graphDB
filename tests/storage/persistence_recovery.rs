//! Persistence Recovery Integration Tests
//!
//! Test coverage:
//! - Flush to disk and reload data integrity
//! - Data consistency after multiple flush/load cycles
//! - Edge presence after persistence round-trip
//! - Schema operations surviving reload
//! - Index metadata persistence

use super::common;

use common::storage_helpers::{create_test_space, knows_edge_type_info, person_tag_info};
use graphdb::core::types::{Index, IndexField, IndexConfig, IndexType, VertexId};
use graphdb::core::value::DateValue;
use graphdb::core::{Edge, Value, Vertex};
use graphdb::storage::{
    GraphStorage, StorageAdmin, StorageReader, StorageSchemaOps, StorageWriter,
};
use std::path::PathBuf;

fn setup_storage_with_path(path: &PathBuf) -> GraphStorage {
    GraphStorage::new_with_path(path.clone()).expect("Failed to create GraphStorage with path")
}

fn setup_space_and_types(storage: &mut GraphStorage) {
    let mut space = create_test_space("test_space");
    storage.create_space(&mut space).unwrap();
    storage.create_tag("test_space", &person_tag_info()).unwrap();
    storage
        .create_edge_type("test_space", &knows_edge_type_info())
        .unwrap();
}

fn insert_test_data(storage: &mut GraphStorage) {
    // Insert vertices
    let alice = Vertex::new(
        VertexId::from_int64(1),
        vec![graphdb::core::vertex_edge_path::Tag::new(
            "Person".to_string(),
            vec![
                ("name".to_string(), Value::String("Alice".to_string())),
                ("age".to_string(), Value::BigInt(30)),
            ]
            .into_iter()
            .collect(),
        )],
    );
    let bob = Vertex::new(
        VertexId::from_int64(2),
        vec![graphdb::core::vertex_edge_path::Tag::new(
            "Person".to_string(),
            vec![
                ("name".to_string(), Value::String("Bob".to_string())),
                ("age".to_string(), Value::BigInt(25)),
            ]
            .into_iter()
            .collect(),
        )],
    );
    storage.insert_vertex("test_space", alice).unwrap();
    storage.insert_vertex("test_space", bob).unwrap();

    // Insert edge
    let edge = Edge::new(
        VertexId::from_int64(1),
        VertexId::from_int64(2),
        "KNOWS".to_string(),
        0,
        vec![("since".to_string(), Value::Date(DateValue { year: 2020, month: 1, day: 1 }))]
            .into_iter()
            .collect(),
    );
    storage.insert_edge("test_space", edge).unwrap();
}

fn verify_data_after_reload(storage: &GraphStorage) {
    // Check vertices survived
    let alice = storage
        .get_vertex("test_space", &VertexId::from_int64(1))
        .unwrap()
        .expect("Alice should exist after reload");
    assert_eq!(
        alice.properties.get("name"),
        Some(&Value::String("Alice".to_string()))
    );

    let bob = storage
        .get_vertex("test_space", &VertexId::from_int64(2))
        .unwrap()
        .expect("Bob should exist after reload");
    assert_eq!(
        bob.properties.get("age"),
        Some(&Value::BigInt(25))
    );

    // Check edges survived
    let edge = storage
        .get_edge(
            "test_space",
            &VertexId::from_int64(1),
            &VertexId::from_int64(2),
            "KNOWS",
            0,
        )
        .unwrap()
        .expect("Edge should exist after reload");
    assert_eq!(edge.src, VertexId::from_int64(1));
    assert_eq!(edge.dst, VertexId::from_int64(2));

    // Check schema survived
    let tag = storage.get_tag("test_space", "Person").unwrap();
    assert!(tag.is_some());
    let edge_type = storage.get_edge_type("test_space", "KNOWS").unwrap();
    assert!(edge_type.is_some());
}

#[test]
fn test_flush_and_reload_preserves_vertices() {
    let dir = std::env::temp_dir()
        .join("graphdb_int_test")
        .join("flush_vertices");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Phase 1: setup and flush
    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        insert_test_data(&mut storage);

        storage.flush().expect("Flush should succeed");
    }

    // Phase 2: reload and verify
    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.load_from_disk().expect("Load from disk should succeed");

        verify_data_after_reload(&storage);
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_multiple_flush_cycles() {
    let dir = std::env::temp_dir()
        .join("graphdb_int_test")
        .join("multi_flush");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // Phase 1: initial data
    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        insert_test_data(&mut storage);
        storage.flush().expect("First flush should succeed");
    }

    // Phase 2: add more data, flush again
    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.load_from_disk().unwrap();

        // Add Charlie
        let charlie = Vertex::new(
            VertexId::from_int64(3),
            vec![graphdb::core::vertex_edge_path::Tag::new(
                "Person".to_string(),
                vec![
                    ("name".to_string(), Value::String("Charlie".to_string())),
                    ("age".to_string(), Value::BigInt(35)),
                ]
                .into_iter()
                .collect(),
            )],
        );
        storage.insert_vertex("test_space", charlie).unwrap();
        storage.flush().expect("Second flush should succeed");
    }

    // Phase 3: verify all data survives
    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.load_from_disk().unwrap();

        verify_data_after_reload(&storage);

        let charlie = storage
            .get_vertex("test_space", &VertexId::from_int64(3))
            .unwrap()
            .expect("Charlie should exist after second reload");
        assert_eq!(
            charlie.properties.get("name"),
            Some(&Value::String("Charlie".to_string()))
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_flush_after_vertex_update() {
    let dir = std::env::temp_dir()
        .join("graphdb_int_test")
        .join("flush_update");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        insert_test_data(&mut storage);
        storage.flush().unwrap();
    }

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.load_from_disk().unwrap();

        // Update Alice's age
        let updated = Vertex::new(
            VertexId::from_int64(1),
            vec![graphdb::core::vertex_edge_path::Tag::new(
                "Person".to_string(),
                vec![
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::BigInt(31)),
                ]
                .into_iter()
                .collect(),
            )],
        );
        storage.update_vertex("test_space", updated).unwrap();
        storage.flush().unwrap();
    }

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.load_from_disk().unwrap();

        let alice = storage
            .get_vertex("test_space", &VertexId::from_int64(1))
            .unwrap()
            .unwrap();
        assert_eq!(
            alice.properties.get("age"),
            Some(&Value::BigInt(31))
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_flush_after_edge_delete() {
    let dir = std::env::temp_dir()
        .join("graphdb_int_test")
        .join("flush_edge_delete");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        insert_test_data(&mut storage);
        storage.flush().unwrap();
    }

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.load_from_disk().unwrap();

        storage
            .delete_edge(
                "test_space",
                &VertexId::from_int64(1),
                &VertexId::from_int64(2),
                "KNOWS",
                0,
            )
            .unwrap();
        storage.flush().unwrap();
    }

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.load_from_disk().unwrap();

        let edge = storage
            .get_edge(
                "test_space",
                &VertexId::from_int64(1),
                &VertexId::from_int64(2),
                "KNOWS",
                0,
            )
            .unwrap();
        assert!(edge.is_none(), "Edge should be deleted after reload");
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_flush_with_index_metadata() {
    let dir = std::env::temp_dir()
        .join("graphdb_int_test")
        .join("flush_index");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);

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
            properties: vec!["name".to_string()],
            index_type: IndexType::TagIndex,
            is_unique: false,
            partial_condition: None,
        });
        storage
            .create_tag_index("test_space", &index)
            .unwrap();
        insert_test_data(&mut storage);
        storage.flush().unwrap();
    }

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.load_from_disk().unwrap();

        // Index metadata should persist
        let indexes = storage.list_tag_indexes("test_space").unwrap();
        assert!(!indexes.is_empty(), "Index metadata should survive flush");
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_flush_and_reload_empty_storage() {
    let dir = std::env::temp_dir()
        .join("graphdb_int_test")
        .join("flush_empty");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.flush().unwrap();
    }

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.load_from_disk().unwrap();

        assert!(storage.space_exists("test_space"));
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_save_to_disk_and_load_back() {
    let dir = std::env::temp_dir()
        .join("graphdb_int_test")
        .join("save_load");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        insert_test_data(&mut storage);

        // Use save_to_disk (StorageAdmin trait)
        StorageAdmin::save_to_disk(&storage).unwrap();
    }

    {
        let mut storage = setup_storage_with_path(&dir);
        setup_space_and_types(&mut storage);
        storage.load_from_disk().unwrap();

        verify_data_after_reload(&storage);
    }

    let _ = std::fs::remove_dir_all(&dir);
}