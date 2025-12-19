//! 架构重构集成测试
//!
//! 这个模块包含了对第一阶段架构重构的集成测试

use graphdb::core::{Value, Vertex, Edge, Tag, DBError, DBResult};
use graphdb::core::visitor::{ValueVisitor, TypeCheckerVisitor, JsonSerializationVisitor};
use graphdb::core::visitors::{DeepCloneVisitor, SizeCalculatorVisitor, HashCalculatorVisitor};
use std::collections::HashMap;

#[test]
fn test_unified_error_handling() {
    // 测试统一错误处理
    let storage_err = graphdb::storage::StorageError::NodeNotFound(Value::Int(42));
    let db_err: DBError = storage_err.into();
    assert!(matches!(db_err, DBError::Storage(_)));

    let query_err = graphdb::query::QueryError::ParseError("test error".to_string());
    let db_err: DBError = query_err.into();
    assert!(matches!(db_err, DBError::Query(_)));

    // 测试错误链
    let result: DBResult<i32> = Err(DBError::Query(
        graphdb::query::QueryError::ParseError("syntax error".to_string())
    ));
    assert!(result.is_err());
}

#[test]
fn test_vertex_encapsulation() {
    // 创建测试标签
    let mut tag_props = HashMap::new();
    tag_props.insert("name".to_string(), Value::String("Alice".to_string()));
    tag_props.insert("age".to_string(), Value::Int(30));
    let tag = Tag::new("Person".to_string(), tag_props);

    // 创建顶点
    let mut vertex = Vertex::new(Value::Int(1), vec![tag]);

    // 测试新的访问接口
    assert_eq!(vertex.id(), &Value::Int(1));
    assert_eq!(vertex.tag_count(), 1);
    assert!(vertex.has_tag("Person"));
    assert!(!vertex.has_tag("NonExistent"));

    let retrieved_tag = vertex.get_tag("Person").expect("Person tag should exist in vertex");
    assert_eq!(retrieved_tag.name, "Person");

    // 测试属性访问
    assert_eq!(
        vertex.get_property("Person", "name"),
        Some(&Value::String("Alice".to_string()))
    );
    assert_eq!(
        vertex.get_property_any("age"),
        Some(&Value::Int(30))
    );

    // 测试获取所有属性
    let all_props = vertex.get_all_properties();
    assert_eq!(all_props.len(), 2);
    assert!(all_props.contains_key("name"));
    assert!(all_props.contains_key("age"));

    // 测试顶点级属性
    vertex.set_vertex_property("status".to_string(), Value::String("active".to_string()));
    assert_eq!(
        vertex.vertex_properties().get("status"),
        Some(&Value::String("active".to_string()))
    );

    assert!(vertex.has_properties());
}

#[test]
fn test_edge_encapsulation() {
    // 创建边
    let mut edge_props = HashMap::new();
    edge_props.insert("weight".to_string(), Value::Float(0.5));
    edge_props.insert("since".to_string(), Value::String("2020".to_string()));

    let edge = Edge::new(
        Value::Int(1),
        Value::Int(2),
        "KNOWS".to_string(),
        0,
        edge_props,
    );

    // 测试新的访问接口
    assert_eq!(edge.src(), &Value::Int(1));
    assert_eq!(edge.dst(), &Value::Int(2));
    assert_eq!(edge.edge_type(), "KNOWS");
    assert_eq!(edge.ranking(), 0);

    // 测试属性访问
    assert_eq!(
        edge.get_property("weight"),
        Some(&Value::Float(0.5))
    );
    assert!(edge.has_property("since"));
    assert!(!edge.has_property("nonexistent"));

    // 测试属性计数
    assert_eq!(edge.property_count(), 2);
    assert!(edge.has_properties());

    // 测试调试字符串
    let debug_str = edge.debug_string();
    assert!(debug_str.contains("Edge"));
    assert!(debug_str.contains("KNOWS"));
}

#[test]
fn test_value_visitor_pattern() {
    // 测试类型检查访问者
    let mut type_checker = TypeCheckerVisitor::new();
    
    let int_value = Value::Int(42);
    int_value.accept(&mut type_checker);
    assert!(type_checker.is_numeric());
    assert_eq!(type_checker.get_type_name(), "Numeric");

    type_checker.reset();
    let string_value = Value::String("test".to_string());
    string_value.accept(&mut type_checker);
    assert!(type_checker.is_string());
    assert_eq!(type_checker.get_type_name(), "String");

    type_checker.reset();
    let list_value = Value::List(vec![Value::Int(1), Value::Int(2)]);
    list_value.accept(&mut type_checker);
    assert!(type_checker.is_collection());
    assert_eq!(type_checker.get_type_name(), "Collection");

    // 测试 JSON 序列化访问者
    let json_result = JsonSerializationVisitor::serialize(&int_value);
    assert!(json_result.is_ok());
    assert_eq!(json_result.expect("JSON serialization of int should succeed"), "42");

    let json_result = JsonSerializationVisitor::serialize(&string_value);
    assert!(json_result.is_ok());
    assert_eq!(json_result.expect("JSON serialization of string should succeed"), "\"test\"");
}

#[test]
fn test_additional_visitors() {
    // 创建复杂值进行测试
    let complex_value = Value::List(vec![
        Value::Int(42),
        Value::String("test".to_string()),
        Value::Map(HashMap::from([
            ("key".to_string(), Value::Bool(true))
        ])),
    ]);

    // 测试深度克隆访问者
    let cloned = DeepCloneVisitor::clone_value(&complex_value);
    assert_eq!(complex_value, cloned);

    // 测试大小计算访问者
    let size = SizeCalculatorVisitor::calculate_size(&complex_value);
    assert!(size > 0);

    // 测试哈希计算访问者
    let hash1 = HashCalculatorVisitor::calculate_hash(&complex_value);
    let hash2 = HashCalculatorVisitor::calculate_hash(&cloned);
    assert_eq!(hash1, hash2);

    // 测试相同值的哈希相同
    let same_value = Value::List(vec![
        Value::Int(42),
        Value::String("test".to_string()),
        Value::Map(HashMap::from([
            ("key".to_string(), Value::Bool(true))
        ])),
    ]);
    let hash3 = HashCalculatorVisitor::calculate_hash(&same_value);
    assert_eq!(hash1, hash3);
}

#[test]
fn test_vertex_edge_with_visitor() {
    // 创建顶点和边
    let mut tag_props = HashMap::new();
    tag_props.insert("name".to_string(), Value::String("Alice".to_string()));
    let tag = Tag::new("Person".to_string(), tag_props);
    let vertex = Vertex::new(Value::Int(1), vec![tag]);

    let edge = Edge::new(
        Value::Int(1),
        Value::Int(2),
        "KNOWS".to_string(),
        0,
        HashMap::new(),
    );

    // 创建包含顶点和边的值
    let vertex_value = Value::Vertex(Box::new(vertex));
    let edge_value = Value::Edge(edge);

    // 测试类型检查访问者
    let mut type_checker = TypeCheckerVisitor::new();
    vertex_value.accept(&mut type_checker);
    assert!(type_checker.is_graph_element());

    type_checker.reset();
    edge_value.accept(&mut type_checker);
    assert!(type_checker.is_graph_element());

    // 测试 JSON 序列化
    let vertex_json = JsonSerializationVisitor::serialize(&vertex_value);
    assert!(vertex_json.is_ok());
    let vertex_json_str = vertex_json.expect("Vertex JSON serialization should succeed");
    assert!(vertex_json_str.contains("vertex"));

    let edge_json = JsonSerializationVisitor::serialize(&edge_value);
    assert!(edge_json.is_ok());
    let edge_json_str = edge_json.expect("Edge JSON serialization should succeed");
    assert!(edge_json_str.contains("src"));
    assert!(edge_json_str.contains("dst"));
}

#[test]
fn test_error_propagation() {
    // 测试错误传播
    fn function_that_fails() -> DBResult<()> {
        Err(DBError::Validation("test validation error".to_string()))
    }

    fn function_that_calls_failing() -> DBResult<()> {
        function_that_fails()?;
        Ok(())
    }

    let result = function_that_calls_failing();
    assert!(result.is_err());
    
    if let Err(DBError::Validation(msg)) = result {
        assert_eq!(msg, "test validation error");
    } else {
        panic!("Expected ValidationError");
    }
}

#[test]
fn test_performance_benchmarks() {
    use std::time::Instant;

    // 创建大型数据结构
    let mut large_list = Vec::new();
    for i in 0..1000 {
        large_list.push(Value::Int(i));
    }
    let large_value = Value::List(large_list);

    // 测试访问者模式的性能
    let start = Instant::now();
    let mut type_checker = TypeCheckerVisitor::new();
    large_value.accept(&mut type_checker);
    let type_check_duration = start.elapsed();

    let start = Instant::now();
    let size = SizeCalculatorVisitor::calculate_size(&large_value);
    let size_calc_duration = start.elapsed();

    let start = Instant::now();
    let hash = HashCalculatorVisitor::calculate_hash(&large_value);
    let hash_calc_duration = start.elapsed();

    // 验证结果
    assert!(type_checker.is_collection());
    assert!(size > 0);
    assert!(hash != 0);

    // 输出性能信息（仅用于调试）
    println!("Type check: {:?}", type_check_duration);
    println!("Size calculation: {:?}", size_calc_duration);
    println!("Hash calculation: {:?}", hash_calc_duration);

    // 确保性能在合理范围内（这些是宽松的限制）
    assert!(type_check_duration.as_millis() < 100);
    assert!(size_calc_duration.as_millis() < 100);
    assert!(hash_calc_duration.as_millis() < 100);
}