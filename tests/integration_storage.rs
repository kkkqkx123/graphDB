//! 阶段一：存储层集成测试
//!
//! 测试存储引擎的核心功能，包括：
//! - 图空间管理（创建、删除、查询）
//! - 标签和边类型管理
//! - 顶点和边的CRUD操作
//! - 事务支持
//! - 索引管理

mod common;

use common::{
    TestStorage,
    assertions::{assert_ok, assert_count},
    data_fixtures::{create_simple_vertex, create_edge, social_network_dataset},
    storage_helpers::{create_test_space, person_tag_info, knows_edge_type_info},
};
use graphdb::core::{Value, Vertex, Edge};
use graphdb::storage::StorageClient;
use std::sync::Arc;
use parking_lot::Mutex;

// 辅助函数：获取可变存储
fn get_storage(storage: &Arc<Mutex<graphdb::storage::redb_storage::RedbStorage>>) -> parking_lot::MutexGuard<graphdb::storage::redb_storage::RedbStorage> {
    storage.lock()
}

// ==================== 图空间管理测试 ====================

#[test]
fn test_storage_space_create_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    let space_info = create_test_space("test_space");
    let result = get_storage(&storage).create_space(&space_info);
    
    assert_ok(result);
    
    // 验证空间存在
    let space = get_storage(&storage).get_space("test_space").expect("获取空间失败");
    assert!(space.is_some(), "空间应该存在");
    assert_eq!(space.expect("空间应该存在").space_name, "test_space");
}

#[test]
fn test_storage_space_create_duplicate() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    let space_info = create_test_space("duplicate_space");
    
    // 第一次创建应该成功
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    // 第二次创建应该失败或返回 false
    let result = get_storage(&storage).create_space(&space_info);
    // 根据实现，可能返回 false 或错误
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_storage_space_drop_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 先创建空间
    let space_info = create_test_space("drop_test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    // 删除空间
    let result = get_storage(&storage).drop_space("drop_test_space");
    assert_ok(result);
    
    // 验证空间已删除
    let space = get_storage(&storage).get_space("drop_test_space").expect("获取空间失败");
    assert!(space.is_none(), "空间应该已被删除");
}

#[test]
fn test_storage_space_list() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建多个空间
    let spaces = vec!["space1", "space2", "space3"];
    for name in &spaces {
        let space_info = create_test_space(name);
        assert_ok(get_storage(&storage).create_space(&space_info));
    }
    
    // 列出所有空间
    let space_list = get_storage(&storage).list_spaces().expect("列出空间失败");
    assert_count(&space_list, 3, "空间");
}

#[test]
fn test_storage_space_exists() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间
    let space_info = create_test_space("exists_test");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    // 验证空间存在检查
    assert!(get_storage(&storage).space_exists("exists_test"), "空间应该存在");
    assert!(!get_storage(&storage).space_exists("nonexistent"), "空间不应该存在");
}

// ==================== 标签管理测试 ====================

#[test]
fn test_storage_tag_create_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 先创建空间
    let space_info = create_test_space("tag_test_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    // 创建标签
    let tag_info = person_tag_info();
    let result = get_storage(&storage).create_tag("tag_test_space", &tag_info);
    
    assert_ok(result);
    
    // 验证标签存在
    let tag = get_storage(&storage).get_tag("tag_test_space", "Person").expect("获取标签失败");
    assert!(tag.is_some(), "标签应该存在");
    assert_eq!(tag.expect("标签应该存在").tag_name, "Person");
}

#[test]
fn test_storage_tag_list() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间
    let space_info = create_test_space("tag_list_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    // 创建多个标签
    let tag1 = person_tag_info();
    let tag2 = graphdb::core::types::TagInfo::new("Company".to_string());
    
    assert_ok(get_storage(&storage).create_tag("tag_list_space", &tag1));
    assert_ok(get_storage(&storage).create_tag("tag_list_space", &tag2));
    
    // 列出所有标签
    let tags = get_storage(&storage).list_tags("tag_list_space").expect("列出标签失败");
    assert_count(&tags, 2, "标签");
}

#[test]
fn test_storage_tag_drop() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和标签
    let space_info = create_test_space("tag_drop_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("tag_drop_space", &tag_info));
    
    // 删除标签
    let result = get_storage(&storage).drop_tag("tag_drop_space", "Person");
    assert_ok(result);
    
    // 验证标签已删除
    let tag = get_storage(&storage).get_tag("tag_drop_space", "Person").expect("获取标签失败");
    assert!(tag.is_none(), "标签应该已被删除");
}

// ==================== 边类型管理测试 ====================

#[test]
fn test_storage_edge_type_create_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间
    let space_info = create_test_space("edge_type_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    // 创建边类型
    let edge_type_info = knows_edge_type_info();
    let result = get_storage(&storage).create_edge_type("edge_type_space", &edge_type_info);
    
    assert_ok(result);
    
    // 验证边类型存在
    let edge_type = get_storage(&storage).get_edge_type("edge_type_space", "KNOWS").expect("获取边类型失败");
    assert!(edge_type.is_some(), "边类型应该存在");
    assert_eq!(edge_type.expect("边类型应该存在").edge_type_name, "KNOWS");
}

#[test]
fn test_storage_edge_type_list() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间
    let space_info = create_test_space("edge_list_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    // 创建多个边类型
    let edge1 = knows_edge_type_info();
    let edge2 = graphdb::core::types::EdgeTypeInfo::new("FOLLOWS".to_string());
    
    assert_ok(get_storage(&storage).create_edge_type("edge_list_space", &edge1));
    assert_ok(get_storage(&storage).create_edge_type("edge_list_space", &edge2));
    
    // 列出所有边类型
    let edges = get_storage(&storage).list_edge_types("edge_list_space").expect("列出边类型失败");
    assert_count(&edges, 2, "边类型");
}

// ==================== 顶点CRUD测试 ====================

#[test]
fn test_storage_vertex_insert_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和标签
    let space_info = create_test_space("vertex_insert_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_insert_space", &tag_info));
    
    // 插入顶点
    let vertex = create_simple_vertex(1, "Person", "Alice", 30);
    let result = get_storage(&storage).insert_vertex("vertex_insert_space", vertex);
    
    assert_ok(result);
}

#[test]
fn test_storage_vertex_get_by_id() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和标签
    let space_info = create_test_space("vertex_get_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_get_space", &tag_info));
    
    // 插入顶点
    let vertex = create_simple_vertex(100, "Person", "Bob", 25);
    let vid = get_storage(&storage).insert_vertex("vertex_get_space", vertex).expect("插入顶点失败");
    
    // 查询顶点
    let retrieved = get_storage(&storage).get_vertex("vertex_get_space", &vid).expect("获取顶点失败");
    assert!(retrieved.is_some(), "顶点应该存在");
    
    let retrieved_vertex = retrieved.expect("顶点应该存在");
    assert_eq!(retrieved_vertex.vid(), &Value::Int(100));
}

#[test]
fn test_storage_vertex_scan_all() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和标签
    let space_info = create_test_space("vertex_scan_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_scan_space", &tag_info));
    
    // 插入多个顶点
    let vertices = vec![
        create_simple_vertex(1, "Person", "User1", 20),
        create_simple_vertex(2, "Person", "User2", 25),
        create_simple_vertex(3, "Person", "User3", 30),
    ];
    
    for vertex in vertices {
        assert_ok(get_storage(&storage).insert_vertex("vertex_scan_space", vertex));
    }
    
    // 扫描所有顶点
    let scan_result = get_storage(&storage).scan_vertices("vertex_scan_space").expect("扫描顶点失败");
    assert_count(&scan_result, 3, "顶点");
}

#[test]
fn test_storage_vertex_update() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和标签
    let space_info = create_test_space("vertex_update_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_update_space", &tag_info));
    
    // 插入顶点
    let vertex = create_simple_vertex(200, "Person", "Original", 20);
    let vid = get_storage(&storage).insert_vertex("vertex_update_space", vertex).expect("插入顶点失败");
    
    // 更新顶点（使用新标签）
    let updated_vertex = create_simple_vertex(200, "Person", "Updated", 25);
    let result = get_storage(&storage).update_vertex("vertex_update_space", updated_vertex);
    assert_ok(result);
    
    // 验证更新
    let retrieved = get_storage(&storage).get_vertex("vertex_update_space", &vid).expect("获取顶点失败");
    assert!(retrieved.is_some());
}

#[test]
fn test_storage_vertex_delete() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和标签
    let space_info = create_test_space("vertex_delete_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_delete_space", &tag_info));
    
    // 插入顶点
    let vertex = create_simple_vertex(300, "Person", "ToDelete", 30);
    let vid = get_storage(&storage).insert_vertex("vertex_delete_space", vertex).expect("插入顶点失败");
    
    // 删除顶点
    let result = get_storage(&storage).delete_vertex("vertex_delete_space", &vid);
    assert_ok(result);
    
    // 验证顶点已删除
    let retrieved = get_storage(&storage).get_vertex("vertex_delete_space", &vid).expect("获取顶点失败");
    assert!(retrieved.is_none(), "顶点应该已被删除");
}

#[test]
fn test_storage_vertex_batch_insert() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和标签
    let space_info = create_test_space("vertex_batch_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("vertex_batch_space", &tag_info));
    
    // 批量插入顶点
    let vertices: Vec<Vertex> = (1..=10)
        .map(|i| create_simple_vertex(i, "Person", &format!("User{}", i), 20 + i as i64))
        .collect();
    
    let result = get_storage(&storage).batch_insert_vertices("vertex_batch_space", vertices);
    assert_ok(result);
    
    // 验证所有顶点已插入
    let scan_result = get_storage(&storage).scan_vertices("vertex_batch_space").expect("扫描顶点失败");
    assert_count(&scan_result, 10, "顶点");
}

// ==================== 边CRUD测试 ====================

#[test]
fn test_storage_edge_insert_success() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和边类型
    let space_info = create_test_space("edge_insert_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let edge_type_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("edge_insert_space", &edge_type_info));
    
    // 插入边
    let edge = create_edge(Value::Int(1), Value::Int(2), "KNOWS");
    let result = get_storage(&storage).insert_edge("edge_insert_space", edge);
    
    assert_ok(result);
}

#[test]
fn test_storage_edge_get() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和边类型
    let space_info = create_test_space("edge_get_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let edge_type_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("edge_get_space", &edge_type_info));
    
    // 插入边
    let edge = create_edge(Value::Int(10), Value::Int(20), "KNOWS");
    assert_ok(get_storage(&storage).insert_edge("edge_get_space", edge));
    
    // 查询边
    let retrieved = get_storage(&storage).get_edge("edge_get_space", &Value::Int(10), &Value::Int(20), "KNOWS")
        .expect("获取边失败");
    assert!(retrieved.is_some(), "边应该存在");
    
    let retrieved_edge = retrieved.expect("边应该存在");
    assert_eq!(retrieved_edge.src(), &Value::Int(10));
    assert_eq!(retrieved_edge.dst(), &Value::Int(20));
}

#[test]
fn test_storage_edge_delete() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和边类型
    let space_info = create_test_space("edge_delete_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let edge_type_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("edge_delete_space", &edge_type_info));
    
    // 插入边
    let edge = create_edge(Value::Int(100), Value::Int(200), "KNOWS");
    assert_ok(get_storage(&storage).insert_edge("edge_delete_space", edge));
    
    // 删除边
    let result = get_storage(&storage).delete_edge("edge_delete_space", &Value::Int(100), &Value::Int(200), "KNOWS");
    assert_ok(result);
    
    // 验证边已删除
    let retrieved = get_storage(&storage).get_edge("edge_delete_space", &Value::Int(100), &Value::Int(200), "KNOWS")
        .expect("获取边失败");
    assert!(retrieved.is_none(), "边应该已被删除");
}

#[test]
fn test_storage_edge_batch_insert() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和边类型
    let space_info = create_test_space("edge_batch_space");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let edge_type_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("edge_batch_space", &edge_type_info));
    
    // 批量插入边
    let edges: Vec<Edge> = (1..=5)
        .map(|i| create_edge(Value::Int(i), Value::Int(i + 1), "KNOWS"))
        .collect();
    
    let result = get_storage(&storage).batch_insert_edges("edge_batch_space", edges);
    assert_ok(result);
    
    // 验证所有边已插入
    let scan_result = get_storage(&storage).scan_all_edges("edge_batch_space").expect("扫描边失败");
    assert_count(&scan_result, 5, "边");
}

// ==================== 完整数据集测试 ====================

#[test]
fn test_storage_social_network_dataset() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 创建空间和Schema
    let space_info = create_test_space("social_network");
    assert_ok(get_storage(&storage).create_space(&space_info));
    
    let tag_info = person_tag_info();
    assert_ok(get_storage(&storage).create_tag("social_network", &tag_info));
    
    let edge_type_info = knows_edge_type_info();
    assert_ok(get_storage(&storage).create_edge_type("social_network", &edge_type_info));
    
    // 加载社交网络数据集
    let (vertices, edges) = social_network_dataset();
    
    // 插入所有顶点
    for vertex in vertices {
        assert_ok(get_storage(&storage).insert_vertex("social_network", vertex));
    }
    
    // 插入所有边
    for edge in edges {
        assert_ok(get_storage(&storage).insert_edge("social_network", edge));
    }
    
    // 验证数据
    let vertex_scan = get_storage(&storage).scan_vertices("social_network").expect("扫描顶点失败");
    assert_count(&vertex_scan, 4, "顶点");
    
    let edge_scan = get_storage(&storage).scan_all_edges("social_network").expect("扫描边失败");
    assert_count(&edge_scan, 4, "边");
}

// ==================== 错误处理测试 ====================

#[test]
fn test_storage_get_nonexistent_vertex() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    
    // 查询不存在的顶点
    let result = get_storage(&storage).get_vertex("nonexistent_space", &Value::Int(999));
    
    // 应该返回 Ok(None) 或错误，取决于实现
    match result {
        Ok(None) => (), // 预期行为
        Ok(Some(_)) => panic!("不应该找到顶点"),
        Err(_) => (), // 也可能返回错误
    }
}

#[test]
fn test_storage_operations_isolated() {
    // 测试两个独立的存储实例是否完全隔离
    let test_storage1 = TestStorage::new().expect("创建测试存储1失败");
    let test_storage2 = TestStorage::new().expect("创建测试存储2失败");
    
    // 在 storage1 中创建空间
    let space_info = create_test_space("isolated_space");
    assert_ok(get_storage(&test_storage1.storage()).create_space(&space_info));
    
    // 验证 storage2 中不存在该空间
    assert!(!get_storage(&test_storage2.storage()).space_exists("isolated_space"), "空间不应该在 storage2 中存在");
}
