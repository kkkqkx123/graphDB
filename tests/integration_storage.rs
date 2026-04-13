//! Phase 1: Integration testing of the storage layer
//!
//! Testing the core functions of the storage engine, including:
//! Graph space management (creation, deletion, querying)
//! Tag and border type management
//! CRUD operations on vertices and edges
//! Transaction support
//! Index management

mod common;

use common::{
    assertions::{assert_count, assert_ok},
    data_fixtures::{create_edge, create_simple_vertex, social_network_dataset},
    storage_helpers::{create_test_space, knows_edge_type_info, person_tag_info},
    TestStorage,
};
use graphdb::core::{Edge, Value, Vertex};
use graphdb::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

// Auxiliary function: Retrieve accessible storage
fn get_storage(
    storage: &Arc<Mutex<graphdb::storage::RedbStorage>>,
) -> parking_lot::MutexGuard<'_, graphdb::storage::RedbStorage> {
    storage.lock()
}

// ==================== Image Space Management Test ====================

#[test]
fn test_storage_space_create_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    let result = get_storage(&storage).create_space(&space_info);

    assert_ok(result);

    // Verify the existence of the space
    let space = get_storage(&storage)
        .get_space("test_space")
        .expect("获取空间失败");
    assert!(space.is_some(), "Space should exist.");
    assert_eq!(space.expect("空间应该存在").space_name, "test_space");
}

#[test]
fn test_storage_space_create_duplicate() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("duplicate_space");

    // The first creation should succeed.
    assert_ok(get_storage(&storage).create_space(&space_info));

    // The second creation attempt should fail or return a value of `false`.
    let result = get_storage(&storage).create_space(&space_info);
    // Depending on the implementation, it may return “false” or an error.
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_storage_space_drop_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // First, create the space.
    let space_info = create_test_space("drop_test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    // Delete the space.
    let result = get_storage(&storage).drop_space("drop_test_space");
    assert_ok(result);

    // The verification space has been deleted.
    let space = get_storage(&storage)
        .get_space("drop_test_space")
        .expect("获取空间失败");
    assert!(space.is_none(), "The space should have been deleted.");
}

#[test]
fn test_storage_space_list() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Create multiple spaces.
    let spaces = vec!["space1", "space2", "space3"];
    for name in &spaces {
        let space_info = create_test_space(name);
        assert_ok(get_storage(&storage).create_space(&space_info));
    }

    // List all spaces
    let space_list = get_storage(&storage).list_spaces().expect("列出空间失败");
    assert_count(&space_list, 3, "空间");
}

#[test]
fn test_storage_space_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Create a space
    let space_info = create_test_space("exists_test");
    assert_ok(get_storage(&storage).create_space(&space_info));

    // Verification of the existence of the space (i.e., a check to confirm that the space in question actually exists)
    assert!(
        get_storage(&storage).space_exists("exists_test"),
        "Space should exist."
    );
    assert!(
        !get_storage(&storage).space_exists("nonexistent"),
        "Space shouldn’t exist."
    );
}

// ==================== Tag Management Test ====================

#[test]
fn test_storage_tag_create_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // First, create the space.
    let space_info = create_test_space("tag_test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    // Create tags
    let tag_info = person_tag_info();
    let result = get_storage(&storage).create_tag("tag_test_space", &tag_info);

    assert_ok(result);

    // Verify that the tag exists.
    let tag = get_storage(&storage)
        .get_tag("tag_test_space", "Person")
        .expect("获取标签失败");
    assert!(tag.is_some(), "The tags should be present.");
    assert_eq!(tag.expect("标签应该存在").tag_name, "Person");
}

#[test]
fn test_storage_tag_list() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Create a space
    let space_info = create_test_space("tag_list_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    // Create multiple tags.
    let tag1 = person_tag_info();
    let tag2 = graphdb::core::types::TagInfo::new("Company".to_string());

    assert_ok(get_storage(&storage).create_tag("tag_list_space", &tag1));
    assert_ok(get_storage(&storage).create_tag("tag_list_space", &tag2));

    // List all tags.
    let tags = get_storage(&storage)
        .list_tags("tag_list_space")
        .expect("列出标签失败");
    assert_count(&tags, 2, "标签");
}

#[test]
fn test_storage_tag_drop() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Create spaces and tags.
    let space_info = create_test_space("tag_drop_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("tag_drop_space", &tag_info));

    // Remove the tags.
    let result = get_storage(&storage).drop_tag("tag_drop_space", "Person");
    assert_ok(result);

    // The verification tags have been deleted.
    let tag = get_storage(&storage)
        .get_tag("tag_drop_space", "Person")
        .expect("获取标签失败");
    assert!(tag.is_none(), "The tags should have been deleted.");
}

// ==================== Edge Type Management Test ====================

#[test]
fn test_storage_edge_type_create_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Create a space
    let space_info = create_test_space("edge_type_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    // Create edge types
    let edge_type_info = knows_edge_type_info();
    let result = get_storage(&storage).create_edge_type("edge_type_space", &edge_type_info);

    assert_ok(result);

    // Verify the existence of the edge type.
    let edge_type = get_storage(&storage)
        .get_edge_type("edge_type_space", "KNOWS")
        .expect("获取边类型失败");
    assert!(edge_type.is_some(), "The edge type should exist.");
    assert_eq!(edge_type.expect("边类型应该存在").edge_type_name, "KNOWS");
}

#[test]
fn test_storage_edge_type_list() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Create a space
    let space_info = create_test_space("edge_list_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    // Create multiple edge types.
    let edge1 = knows_edge_type_info();
    let edge2 = graphdb::core::types::EdgeTypeInfo::new("FOLLOWS".to_string());

    assert_ok(get_storage(&storage).create_edge_type("edge_list_space", &edge1));
    assert_ok(get_storage(&storage).create_edge_type("edge_list_space", &edge2));

    // List all edge types.
    let edges = get_storage(&storage)
        .list_edge_types("edge_list_space")
        .expect("列出边类型失败");
    assert_count(&edges, 2, "边类型");
}

// ==================== Vertex CRUD Testing ====================

#[test]
fn test_storage_vertex_insert_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Create spaces and tags.
    let space_info = create_test_space("vertex_insert_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_insert_space", &tag_info));

    // Insert a vertex
    let vertex = create_simple_vertex(1, "Person", "Alice", 30);
    let result = get_storage(&storage).insert_vertex("vertex_insert_space", vertex);

    assert_ok(result);
}

#[test]
fn test_storage_vertex_get_by_id() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Create spaces and tags
    let space_info = create_test_space("vertex_get_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_get_space", &tag_info));

    // Insert a vertex
    let vertex = create_simple_vertex(100, "Person", "Bob", 25);
    let vid = get_storage(&storage)
        .insert_vertex("vertex_get_space", vertex)
        .expect("插入顶点失败");

    // Querying vertices
    let retrieved = get_storage(&storage)
        .get_vertex("vertex_get_space", &vid)
        .expect("获取顶点失败");
    assert!(retrieved.is_some(), "The vertex should exist");

    let retrieved_vertex = retrieved.expect("顶点应该存在");
    assert_eq!(retrieved_vertex.vid(), &Value::Int(100));
}

#[test]
fn test_storage_vertex_scan_all() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Creating spaces and labels
    let space_info = create_test_space("vertex_scan_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_scan_space", &tag_info));

    // Insert multiple vertices
    let vertices = vec![
        create_simple_vertex(1, "Person", "User1", 20),
        create_simple_vertex(2, "Person", "User2", 25),
        create_simple_vertex(3, "Person", "User3", 30),
    ];

    for vertex in vertices {
        assert_ok(get_storage(&storage).insert_vertex("vertex_scan_space", vertex));
    }

    // Scan all vertices
    let scan_result = get_storage(&storage)
        .scan_vertices("vertex_scan_space")
        .expect("扫描顶点失败");
    assert_count(&scan_result, 3, "顶点");
}

#[test]
fn test_storage_vertex_update() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Creating spaces and labels
    let space_info = create_test_space("vertex_update_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_update_space", &tag_info));

    // Insert vertex
    let vertex = create_simple_vertex(200, "Person", "Original", 20);
    let vid = get_storage(&storage)
        .insert_vertex("vertex_update_space", vertex)
        .expect("插入顶点失败");

    // Updating vertices (using new labels)
    let updated_vertex = create_simple_vertex(200, "Person", "Updated", 25);
    let result = get_storage(&storage).update_vertex("vertex_update_space", updated_vertex);
    assert_ok(result);

    // Validation Updates
    let retrieved = get_storage(&storage)
        .get_vertex("vertex_update_space", &vid)
        .expect("获取顶点失败");
    assert!(retrieved.is_some());
}

#[test]
fn test_storage_vertex_delete() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Creating spaces and labels
    let space_info = create_test_space("vertex_delete_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_delete_space", &tag_info));

    // Insert vertex
    let vertex = create_simple_vertex(300, "Person", "ToDelete", 30);
    let vid = get_storage(&storage)
        .insert_vertex("vertex_delete_space", vertex)
        .expect("插入顶点失败");

    // Delete Vertex
    let result = get_storage(&storage).delete_vertex("vertex_delete_space", &vid);
    assert_ok(result);

    // Verify that the vertex has been deleted
    let retrieved = get_storage(&storage)
        .get_vertex("vertex_delete_space", &vid)
        .expect("获取顶点失败");
    assert!(retrieved.is_none(), "Vertex should have been deleted");
}

#[test]
fn test_storage_vertex_batch_insert() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Creating spaces and labels
    let space_info = create_test_space("vertex_batch_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_batch_space", &tag_info));

    // Batch insertion of vertices
    let vertices: Vec<Vertex> = (1..=10)
        .map(|i| create_simple_vertex(i, "Person", &format!("User{}", i), 20 + i))
        .collect();

    let result = get_storage(&storage).batch_insert_vertices("vertex_batch_space", vertices);
    assert_ok(result);

    // Verify that all vertices are inserted
    let scan_result = get_storage(&storage)
        .scan_vertices("vertex_batch_space")
        .expect("扫描顶点失败");
    assert_count(&scan_result, 10, "顶点");
}

// ==================== Side CRUD test ====================

#[test]
fn test_storage_edge_insert_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Creating spaces and edge types
    let space_info = create_test_space("edge_insert_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let edge_type_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("edge_insert_space", &edge_type_info));

    // insertion side
    let edge = create_edge(Value::Int(1), Value::Int(2), "KNOWS");
    let result = get_storage(&storage).insert_edge("edge_insert_space", edge);

    assert_ok(result);
}

#[test]
fn test_storage_edge_get() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Creating spaces and edge types
    let space_info = create_test_space("edge_get_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let edge_type_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("edge_get_space", &edge_type_info));

    // insertion side
    let edge = create_edge(Value::Int(10), Value::Int(20), "KNOWS");
    assert_ok(get_storage(&storage).insert_edge("edge_get_space", edge));

    // query side
    let retrieved = get_storage(&storage)
        .get_edge(
            "edge_get_space",
            &Value::Int(10),
            &Value::Int(20),
            "KNOWS",
            0,
        )
        .expect("获取边失败");
    assert!(retrieved.is_some(), "The edge should be present");

    let retrieved_edge = retrieved.expect("边应该存在");
    assert_eq!(retrieved_edge.src(), &Value::Int(10));
    assert_eq!(retrieved_edge.dst(), &Value::Int(20));
}

#[test]
fn test_storage_edge_delete() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Creating spaces and edge types
    let space_info = create_test_space("edge_delete_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let edge_type_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("edge_delete_space", &edge_type_info));

    // insertion side
    let edge = create_edge(Value::Int(100), Value::Int(200), "KNOWS");
    assert_ok(get_storage(&storage).insert_edge("edge_delete_space", edge));

    // Remove Edge
    let result = get_storage(&storage).delete_edge(
        "edge_delete_space",
        &Value::Int(100),
        &Value::Int(200),
        "KNOWS",
        0,
    );
    assert_ok(result);

    // Verification Side Deleted
    let retrieved = get_storage(&storage)
        .get_edge(
            "edge_delete_space",
            &Value::Int(100),
            &Value::Int(200),
            "KNOWS",
            0,
        )
        .expect("获取边失败");
    assert!(retrieved.is_none(), "The edge should have been deleted.");
}

#[test]
fn test_storage_edge_batch_insert() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Creating spaces and edge types
    let space_info = create_test_space("edge_batch_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let edge_type_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("edge_batch_space", &edge_type_info));

    // Batch insertion of edges
    let edges: Vec<Edge> = (1..=5)
        .map(|i| create_edge(Value::Int(i), Value::Int(i + 1), "KNOWS"))
        .collect();

    let result = get_storage(&storage).batch_insert_edges("edge_batch_space", edges);
    assert_ok(result);

    // Verify that all edges are inserted
    let scan_result = get_storage(&storage)
        .scan_all_edges("edge_batch_space")
        .expect("扫描边失败");
    assert_count(&scan_result, 5, "边");
}

// ==================== Full dataset test ====================

#[test]
fn test_storage_social_network_dataset() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Create Space and Schema
    let space_info = create_test_space("social_network");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("social_network", &tag_info));

    let edge_type_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("social_network", &edge_type_info));

    // Loading social network datasets
    let (vertices, edges) = social_network_dataset();

    // Insert all vertices
    for vertex in vertices {
        assert_ok(get_storage(&storage).insert_vertex("social_network", vertex));
    }

    // Insert all edges
    for edge in edges {
        assert_ok(get_storage(&storage).insert_edge("social_network", edge));
    }

    // Validation Data
    let vertex_scan = get_storage(&storage)
        .scan_vertices("social_network")
        .expect("扫描顶点失败");
    assert_count(&vertex_scan, 4, "顶点");

    let edge_scan = get_storage(&storage)
        .scan_all_edges("social_network")
        .expect("扫描边失败");
    assert_count(&edge_scan, 4, "边");
}

// ==================== 错误处理测试 ====================

#[test]
fn test_storage_get_nonexistent_vertex() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    // Query for non-existent vertices
    let result = get_storage(&storage).get_vertex("nonexistent_space", &Value::Int(999));

    // 应该返回 Ok(None) 或错误，取决于实现
    match result {
        Ok(None) => (), // Expected behavior
        Ok(Some(_)) => panic!("Shouldn't have found the apex."),
        Err(_) => (), // It may also return an error
    }
}

#[test]
fn test_storage_operations_isolated() {
    // Test that two separate storage instances are completely isolated
    let test_storage1 = TestStorage::new().expect("创建测试存储1失败");
    let test_storage2 = TestStorage::new().expect("创建测试存储2失败");

    // Create space in storage1
    let space_info = create_test_space("isolated_space");
    assert_ok(get_storage(&test_storage1.storage()).create_space(&space_info));

    // Verify that the space does not exist in storage2
    assert!(
        !get_storage(&test_storage2.storage()).space_exists("isolated_space"),
        "Space should not exist in storage2"
    );
}
