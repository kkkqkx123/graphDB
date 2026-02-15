//! 索引系统集成测试
//!
//! 测试范围：
//! - Tag 索引元数据管理（创建、删除、查询、列出）
//! - Edge 索引元数据管理（创建、删除、查询、列出）
//! - 索引数据管理（更新、删除、查询）
//! - 索引查询（精确查询、范围查询）
//! - 索引缓存

mod common;

use common::{
    TestStorage,
    assertions::{assert_ok, assert_count, assert_some, assert_none},
    storage_helpers::{create_test_space, person_tag_info, knows_edge_type_info},
};
use graphdb::core::{Value, Vertex, Edge};
use graphdb::index::{Index, IndexType, IndexField, IndexStatus};
use graphdb::storage::StorageClient;
use std::sync::Arc;
use parking_lot::Mutex;

fn get_storage(storage: &Arc<Mutex<graphdb::storage::redb_storage::RedbStorage>>) -> parking_lot::MutexGuard<graphdb::storage::redb_storage::RedbStorage> {
    storage.lock()
}

// ==================== Tag 索引元数据管理测试 ====================

#[tokio::test]
async fn test_create_tag_index_metadata() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_name_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        false,
    );

    let result = get_storage(&storage).create_tag_index("test_space", &index);
    let created = result.expect("创建索引应该成功");
    assert!(created, "索引应该被创建");

    let retrieved = get_storage(&storage).get_tag_index("test_space", "person_name_idx");
    let index_opt = retrieved.expect("获取索引应该成功");
    assert_some(&index_opt);

    let retrieved_index = index_opt.expect("索引应该存在");
    assert_eq!(retrieved_index.name, "person_name_idx");
    assert_eq!(retrieved_index.schema_name, "Person");
    assert_eq!(retrieved_index.index_type, IndexType::TagIndex);
}

#[tokio::test]
async fn test_create_tag_index_duplicate() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_name_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index));

    let result = get_storage(&storage).create_tag_index("test_space", &index);
    let created = result.expect("创建重复索引应该返回 false");
    assert!(!created, "重复索引创建应该返回 false");
}

#[tokio::test]
async fn test_drop_tag_index_metadata() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_name_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index));

    let result = get_storage(&storage).drop_tag_index("test_space", "person_name_idx");
    let dropped = result.expect("删除索引应该成功");
    assert!(dropped, "索引应该被删除");

    let retrieved = get_storage(&storage).get_tag_index("test_space", "person_name_idx");
    let index_opt = retrieved.expect("获取索引应该成功");
    assert_none(&index_opt);
}

#[tokio::test]
async fn test_list_tag_indexes() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index1 = Index::new(
        1,
        "person_name_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        false,
    );

    let index2 = Index::new(
        2,
        "person_age_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("age".to_string(), Value::Int(0), false)],
        vec!["age".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index1));
    assert_ok(get_storage(&storage).create_tag_index("test_space", &index2));

    let result = get_storage(&storage).list_tag_indexes("test_space");
    let indexes = result.expect("列出索引应该成功");
    assert_count(&indexes, 2, "索引");

    let index_names: Vec<&str> = indexes.iter().map(|i| i.name.as_str()).collect();
    assert!(index_names.contains(&"person_name_idx"), "应该包含 person_name_idx");
    assert!(index_names.contains(&"person_age_idx"), "应该包含 person_age_idx");
}

#[tokio::test]
async fn test_drop_tag_indexes_by_tag() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index1 = Index::new(
        1,
        "person_name_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        false,
    );

    let index2 = Index::new(
        2,
        "person_age_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("age".to_string(), Value::Int(0), false)],
        vec!["age".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index1));
    assert_ok(get_storage(&storage).create_tag_index("test_space", &index2));

    get_storage(&storage).drop_tag_index("test_space", "person_name_idx").expect("删除标签索引应该成功");
    get_storage(&storage).drop_tag_index("test_space", "person_age_idx").expect("删除标签索引应该成功");

    let indexes = get_storage(&storage).list_tag_indexes("test_space").expect("列出索引应该成功");
    assert_count(&indexes, 0, "索引");
}

// ==================== Edge 索引元数据管理测试 ====================

#[tokio::test]
async fn test_create_edge_index_metadata() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let edge_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("test_space", &edge_info));

    let index = Index::new(
        1,
        "knows_since_idx".to_string(),
        0,
        "KNOWS".to_string(),
        vec![IndexField::new("since".to_string(), Value::String("".to_string()), false)],
        vec!["since".to_string()],
        IndexType::EdgeIndex,
        false,
    );

    let result = get_storage(&storage).create_edge_index("test_space", &index);
    let created = result.expect("创建索引应该成功");
    assert!(created, "索引应该被创建");

    let retrieved = get_storage(&storage).get_edge_index("test_space", "knows_since_idx");
    let index_opt = retrieved.expect("获取索引应该成功");
    assert_some(&index_opt);

    let retrieved_index = index_opt.expect("索引应该存在");
    assert_eq!(retrieved_index.name, "knows_since_idx");
    assert_eq!(retrieved_index.schema_name, "KNOWS");
    assert_eq!(retrieved_index.index_type, IndexType::EdgeIndex);
}

#[tokio::test]
async fn test_drop_edge_index_metadata() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let edge_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("test_space", &edge_info));

    let index = Index::new(
        1,
        "knows_since_idx".to_string(),
        0,
        "KNOWS".to_string(),
        vec![IndexField::new("since".to_string(), Value::String("".to_string()), false)],
        vec!["since".to_string()],
        IndexType::EdgeIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_edge_index("test_space", &index));

    let result = get_storage(&storage).drop_edge_index("test_space", "knows_since_idx");
    let dropped = result.expect("删除索引应该成功");
    assert!(dropped, "索引应该被删除");

    let retrieved = get_storage(&storage).get_edge_index("test_space", "knows_since_idx");
    let index_opt = retrieved.expect("获取索引应该成功");
    assert_none(&index_opt);
}

#[tokio::test]
async fn test_list_edge_indexes() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let edge_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("test_space", &edge_info));

    let index1 = Index::new(
        1,
        "knows_since_idx".to_string(),
        0,
        "KNOWS".to_string(),
        vec![IndexField::new("since".to_string(), Value::String("".to_string()), false)],
        vec!["since".to_string()],
        IndexType::EdgeIndex,
        false,
    );

    let index2 = Index::new(
        2,
        "knows_weight_idx".to_string(),
        0,
        "KNOWS".to_string(),
        vec![IndexField::new("weight".to_string(), Value::Float(0.0), false)],
        vec!["weight".to_string()],
        IndexType::EdgeIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_edge_index("test_space", &index1));
    assert_ok(get_storage(&storage).create_edge_index("test_space", &index2));

    let result = get_storage(&storage).list_edge_indexes("test_space");
    let indexes = result.expect("列出索引应该成功");
    assert_count(&indexes, 2, "索引");

    let index_names: Vec<&str> = indexes.iter().map(|i| i.name.as_str()).collect();
    assert!(index_names.contains(&"knows_since_idx"), "应该包含 knows_since_idx");
    assert!(index_names.contains(&"knows_weight_idx"), "应该包含 knows_weight_idx");
}

// ==================== 索引数据管理测试 ====================

#[tokio::test]
async fn test_update_vertex_indexes() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_name_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index));

    let vertex_id = Value::Int(1);
    let mut props = std::collections::HashMap::new();
    props.insert("name".to_string(), Value::String("Alice".to_string()));
    let tag = graphdb::core::vertex_edge_path::Tag::new("Person".to_string(), props);
    let vertex = Vertex::new(vertex_id.clone(), vec![tag]);

    get_storage(&storage).insert_vertex("test_space", vertex).expect("插入顶点应该成功");

    let retrieved = get_storage(&storage).lookup_index("test_space", "person_name_idx", &Value::String("Alice".to_string()));
    let vertex_ids = retrieved.expect("索引查询应该成功");
    assert!(vertex_ids.contains(&vertex_id), "索引应该包含顶点 ID");
}

#[tokio::test]
async fn test_update_edge_indexes() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let edge_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("test_space", &edge_info));

    let index = Index::new(
        1,
        "knows_since_idx".to_string(),
        0,
        "KNOWS".to_string(),
        vec![IndexField::new("since".to_string(), Value::String("".to_string()), false)],
        vec!["since".to_string()],
        IndexType::EdgeIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_edge_index("test_space", &index));

    let src = Value::Int(1);
    let dst = Value::Int(2);
    let edge_type = "KNOWS";
    let mut props = std::collections::HashMap::new();
    props.insert("since".to_string(), Value::String("2024-01-01".to_string()));
    let edge = Edge::new(src.clone(), dst.clone(), edge_type.to_string(), 0, props);

    get_storage(&storage).insert_edge("test_space", edge).expect("插入边应该成功");

    let retrieved = get_storage(&storage).lookup_index("test_space", "knows_since_idx", &Value::String("2024-01-01".to_string()));
    let src_ids = retrieved.expect("索引查询应该成功");
    assert!(src_ids.contains(&src), "索引应该包含源顶点 ID");
}

#[tokio::test]
async fn test_delete_vertex_indexes() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_name_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index));

    let vertex_id = Value::Int(1);
    let mut props = std::collections::HashMap::new();
    props.insert("name".to_string(), Value::String("Alice".to_string()));
    let tag = graphdb::core::vertex_edge_path::Tag::new("Person".to_string(), props);
    let vertex = Vertex::new(vertex_id.clone(), vec![tag]);

    assert_ok(get_storage(&storage).insert_vertex("test_space", vertex));

    assert_ok(get_storage(&storage).delete_vertex("test_space", &vertex_id));

    let retrieved = get_storage(&storage).lookup_index("test_space", "person_name_idx", &Value::String("Alice".to_string()));
    let vertex_ids = retrieved.expect("索引查询应该成功");
    assert!(!vertex_ids.contains(&vertex_id), "索引不应该包含已删除的顶点 ID");
}

#[tokio::test]
async fn test_delete_edge_indexes() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let edge_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("test_space", &edge_info));

    let index = Index::new(
        1,
        "knows_since_idx".to_string(),
        0,
        "KNOWS".to_string(),
        vec![IndexField::new("since".to_string(), Value::String("".to_string()), false)],
        vec!["since".to_string()],
        IndexType::EdgeIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_edge_index("test_space", &index));

    let src = Value::Int(1);
    let dst = Value::Int(2);
    let edge_type = "KNOWS";
    let mut props = std::collections::HashMap::new();
    props.insert("since".to_string(), Value::String("2024-01-01".to_string()));
    let edge = Edge::new(src.clone(), dst.clone(), edge_type.to_string(), 0, props);

    assert_ok(get_storage(&storage).insert_edge("test_space", edge));

    assert_ok(get_storage(&storage).delete_edge("test_space", &src, &dst, edge_type));

    let retrieved = get_storage(&storage).lookup_index("test_space", "knows_since_idx", &Value::String("2024-01-01".to_string()));
    let src_ids = retrieved.expect("索引查询应该成功");
    assert!(!src_ids.contains(&src), "索引不应该包含已删除边的源顶点 ID");
}

// ==================== 索引查询测试 ====================

#[tokio::test]
async fn test_index_exact_query() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_name_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index));

    let vertices = vec![
        (Value::Int(1), Value::String("Alice".to_string())),
        (Value::Int(2), Value::String("Bob".to_string())),
        (Value::Int(3), Value::String("Charlie".to_string())),
    ];

    for (vid, name) in &vertices {
        let mut props = std::collections::HashMap::new();
        props.insert("name".to_string(), name.clone());
        let tag = graphdb::core::vertex_edge_path::Tag::new("Person".to_string(), props);
        let vertex = Vertex::new(vid.clone(), vec![tag]);
        assert_ok(get_storage(&storage).insert_vertex("test_space", vertex));
    }

    let retrieved = get_storage(&storage).lookup_index("test_space", "person_name_idx", &Value::String("Alice".to_string()));
    let vertex_ids = retrieved.expect("索引精确查询应该成功");
    assert_count(&vertex_ids, 1, "匹配的顶点");
    assert_eq!(vertex_ids[0], Value::Int(1), "应该返回 Alice 的顶点 ID");
}

#[tokio::test]
async fn test_index_query_multiple_matches() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_age_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("age".to_string(), Value::Int(0), false)],
        vec!["age".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index));

    let vertices = vec![
        (Value::Int(1), Value::Int(30)),
        (Value::Int(2), Value::Int(30)),
        (Value::Int(3), Value::Int(25)),
    ];

    for (vid, age) in &vertices {
        let mut props = std::collections::HashMap::new();
        props.insert("age".to_string(), age.clone());
        let tag = graphdb::core::vertex_edge_path::Tag::new("Person".to_string(), props);
        let vertex = Vertex::new(vid.clone(), vec![tag]);
        assert_ok(get_storage(&storage).insert_vertex("test_space", vertex));
    }

    let retrieved = get_storage(&storage).lookup_index("test_space", "person_age_idx", &Value::Int(30));
    let vertex_ids = retrieved.expect("索引查询应该成功");
    assert_count(&vertex_ids, 2, "匹配的顶点");
    assert!(vertex_ids.contains(&Value::Int(1)), "应该包含顶点 1");
    assert!(vertex_ids.contains(&Value::Int(2)), "应该包含顶点 2");
}

#[tokio::test]
async fn test_index_query_no_match() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_name_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index));

    let vertex_id = Value::Int(1);
    let mut props = std::collections::HashMap::new();
    props.insert("name".to_string(), Value::String("Alice".to_string()));
    let tag = graphdb::core::vertex_edge_path::Tag::new("Person".to_string(), props);
    let vertex = Vertex::new(vertex_id.clone(), vec![tag]);

    assert_ok(get_storage(&storage).insert_vertex("test_space", vertex));

    let retrieved = get_storage(&storage).lookup_index("test_space", "person_name_idx", &Value::String("Bob".to_string()));
    let vertex_ids = retrieved.expect("索引查询应该成功");
    assert_count(&vertex_ids, 0, "匹配的顶点");
}

// ==================== 索引状态测试 ====================

#[tokio::test]
async fn test_index_status_active() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_name_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index));

    let retrieved = get_storage(&storage).get_tag_index("test_space", "person_name_idx");
    let index_opt = retrieved.expect("获取索引应该成功");
    assert_some(&index_opt);

    let retrieved_index = index_opt.expect("索引应该存在");
    assert_eq!(retrieved_index.status, IndexStatus::Active, "新创建的索引应该是 Active 状态");
}

#[tokio::test]
async fn test_unique_index() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_name_unique_idx".to_string(),
        0,
        "Person".to_string(),
        vec![IndexField::new("name".to_string(), Value::String("".to_string()), false)],
        vec!["name".to_string()],
        IndexType::TagIndex,
        true,
    );

    let result = get_storage(&storage).create_tag_index("test_space", &index);
    result.expect("创建唯一索引应该成功");

    let retrieved = get_storage(&storage).get_tag_index("test_space", "person_name_unique_idx");
    let index_opt = retrieved.expect("获取索引应该成功");
    assert_some(&index_opt);

    let retrieved_index = index_opt.expect("索引应该存在");
    assert!(retrieved_index.is_unique, "索引应该是唯一索引");
}

// ==================== 复合索引测试 ====================

#[tokio::test]
async fn test_composite_index() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();

    let space_info = create_test_space("test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));

    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("test_space", &tag_info));

    let index = Index::new(
        1,
        "person_name_age_idx".to_string(),
        0,
        "Person".to_string(),
        vec![
            IndexField::new("name".to_string(), Value::String("".to_string()), false),
            IndexField::new("age".to_string(), Value::Int(0), false),
        ],
        vec!["name".to_string(), "age".to_string()],
        IndexType::TagIndex,
        false,
    );

    assert_ok(get_storage(&storage).create_tag_index("test_space", &index));

    let vertices = vec![
        (Value::Int(1), Value::String("Alice".to_string()), Value::Int(30)),
        (Value::Int(2), Value::String("Alice".to_string()), Value::Int(25)),
        (Value::Int(3), Value::String("Bob".to_string()), Value::Int(30)),
    ];

    for (vid, name, age) in &vertices {
        let mut props = std::collections::HashMap::new();
        props.insert("name".to_string(), name.clone());
        props.insert("age".to_string(), age.clone());
        let tag = graphdb::core::vertex_edge_path::Tag::new("Person".to_string(), props);
        let vertex = Vertex::new(vid.clone(), vec![tag]);
        assert_ok(get_storage(&storage).insert_vertex("test_space", vertex));
    }

    let retrieved = get_storage(&storage).lookup_index("test_space", "person_name_age_idx", &Value::String("Alice".to_string()));
    let vertex_ids = retrieved.expect("复合索引查询应该成功");
    assert_count(&vertex_ids, 2, "匹配的顶点（两个 Alice）");
    assert!(vertex_ids.contains(&Value::Int(1)), "应该包含顶点 1");
    assert!(vertex_ids.contains(&Value::Int(2)), "应该包含顶点 2");
}
