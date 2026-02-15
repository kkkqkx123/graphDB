//! 内置函数集成测试
//!
//! 测试范围:
//! - 图相关函数: id, tags, labels, properties, type, src, dst, rank
//! - 容器操作函数: head, last, tail, size, range, keys
//! - 路径函数: nodes, relationships
//! - 数学函数: bit_and, bit_or, bit_xor, asin, acos, atan, cbrt, hypot
//! - 字符串函数: split
//! - 实用函数: coalesce, hash

mod common;

use graphdb::core::{Value, List, NullType};
use graphdb::core::vertex_edge_path::{Tag, Vertex, Edge, Path, Step};
use graphdb::expression::functions::registry::FunctionRegistry;
use std::collections::HashMap;

/// 创建测试用的顶点
fn create_test_vertex(vid: i64, tags: Vec<(&str, HashMap<&str, Value>)>) -> Vertex {
    let tags: Vec<Tag> = tags
        .into_iter()
        .map(|(name, props)| {
            let props: HashMap<String, Value> = props
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect();
            Tag::new(name.to_string(), props)
        })
        .collect();
    Vertex::new(Value::Int(vid), tags)
}

/// 创建测试用的边
fn create_test_edge(src: i64, dst: i64, edge_type: &str, rank: i64, props: HashMap<&str, Value>) -> Edge {
    let props: HashMap<String, Value> = props
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    Edge::new(Value::Int(src), Value::Int(dst), edge_type.to_string(), rank, props)
}

/// 创建测试用的路径
fn create_test_path() -> Path {
    let v1 = create_test_vertex(1, vec![
        ("Person", {
            let mut m = HashMap::new();
            m.insert("name", Value::String("Alice".to_string()));
            m.insert("age", Value::Int(30));
            m
        }),
    ]);
    let v2 = create_test_vertex(2, vec![
        ("Person", {
            let mut m = HashMap::new();
            m.insert("name", Value::String("Bob".to_string()));
            m.insert("age", Value::Int(25));
            m
        }),
    ]);
    let v3 = create_test_vertex(3, vec![
        ("Person", {
            let mut m = HashMap::new();
            m.insert("name", Value::String("Charlie".to_string()));
            m.insert("age", Value::Int(35));
            m
        }),
    ]);

    let e1 = create_test_edge(1, 2, "KNOWS", 0, HashMap::new());
    let e2 = create_test_edge(2, 3, "KNOWS", 0, HashMap::new());

    let mut path = Path::new(v1);
    path.add_step(Step { edge: Box::new(e1), dst: Box::new(v2.clone()) });
    path.add_step(Step { edge: Box::new(e2), dst: Box::new(v3.clone()) });
    path
}

// ==================== 图相关函数测试 ====================

#[test]
fn test_id_function() {
    let registry = FunctionRegistry::new();
    let vertex = create_test_vertex(100, vec![("Person", HashMap::new())]);

    let result = registry.execute("id", &[Value::Vertex(Box::new(vertex))]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(100));
}

#[test]
fn test_tags_function() {
    let registry = FunctionRegistry::new();
    let vertex = create_test_vertex(1, vec![
        ("Person", HashMap::new()),
        ("Employee", HashMap::new()),
    ]);

    let result = registry.execute("tags", &[Value::Vertex(Box::new(vertex))]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert_eq!(list.values.len(), 2);
    } else {
        panic!("期望返回列表类型");
    }
}

#[test]
fn test_labels_function() {
    let registry = FunctionRegistry::new();
    let vertex = create_test_vertex(1, vec![
        ("Person", HashMap::new()),
    ]);

    let result = registry.execute("labels", &[Value::Vertex(Box::new(vertex))]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert_eq!(list.values.len(), 1);
        assert_eq!(list.values[0], Value::String("Person".to_string()));
    } else {
        panic!("期望返回列表类型");
    }
}

#[test]
fn test_properties_vertex_function() {
    let registry = FunctionRegistry::new();
    let vertex = create_test_vertex(1, vec![
        ("Person", {
            let mut m = HashMap::new();
            m.insert("name", Value::String("Alice".to_string()));
            m.insert("age", Value::Int(30));
            m
        }),
    ]);

    let result = registry.execute("properties", &[Value::Vertex(Box::new(vertex))]);
    assert!(result.is_ok());
    
    if let Value::Map(map) = result.unwrap() {
        assert!(map.contains_key("name"));
        assert!(map.contains_key("age"));
        assert_eq!(map.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(map.get("age"), Some(&Value::Int(30)));
    } else {
        panic!("期望返回映射类型");
    }
}

#[test]
fn test_type_function() {
    let registry = FunctionRegistry::new();
    let edge = create_test_edge(1, 2, "KNOWS", 0, HashMap::new());

    let result = registry.execute("type", &[Value::Edge(edge)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("KNOWS".to_string()));
}

#[test]
fn test_src_function() {
    let registry = FunctionRegistry::new();
    let edge = create_test_edge(100, 200, "KNOWS", 0, HashMap::new());

    let result = registry.execute("src", &[Value::Edge(edge)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(100));
}

#[test]
fn test_dst_function() {
    let registry = FunctionRegistry::new();
    let edge = create_test_edge(100, 200, "KNOWS", 0, HashMap::new());

    let result = registry.execute("dst", &[Value::Edge(edge)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(200));
}

#[test]
fn test_rank_function() {
    let registry = FunctionRegistry::new();
    let edge = create_test_edge(1, 2, "KNOWS", 42, HashMap::new());

    let result = registry.execute("rank", &[Value::Edge(edge)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

// ==================== 容器操作函数测试 ====================

#[test]
fn test_head_function() {
    let registry = FunctionRegistry::new();
    let list = Value::List(List {
        values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
    });

    let result = registry.execute("head", &[list]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(1));
}

#[test]
fn test_last_function() {
    let registry = FunctionRegistry::new();
    let list = Value::List(List {
        values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
    });

    let result = registry.execute("last", &[list]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(3));
}

#[test]
fn test_tail_function() {
    let registry = FunctionRegistry::new();
    let list = Value::List(List {
        values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
    });

    let result = registry.execute("tail", &[list]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert_eq!(list.values.len(), 2);
        assert_eq!(list.values[0], Value::Int(2));
        assert_eq!(list.values[1], Value::Int(3));
    } else {
        panic!("期望返回列表类型");
    }
}

#[test]
fn test_size_list_function() {
    let registry = FunctionRegistry::new();
    let list = Value::List(List {
        values: vec![Value::Int(1), Value::Int(2), Value::Int(3)],
    });

    let result = registry.execute("size", &[list]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(3));
}

#[test]
fn test_size_string_function() {
    let registry = FunctionRegistry::new();
    let string = Value::String("hello".to_string());

    let result = registry.execute("size", &[string]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(5));
}

#[test]
fn test_range_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("range", &[Value::Int(1), Value::Int(5)]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert_eq!(list.values.len(), 5);
        assert_eq!(list.values[0], Value::Int(1));
        assert_eq!(list.values[4], Value::Int(5));
    } else {
        panic!("期望返回列表类型");
    }
}

#[test]
fn test_range_with_step_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("range", &[Value::Int(0), Value::Int(10), Value::Int(2)]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert_eq!(list.values.len(), 6);
        assert_eq!(list.values[0], Value::Int(0));
        assert_eq!(list.values[1], Value::Int(2));
        assert_eq!(list.values[5], Value::Int(10));
    } else {
        panic!("期望返回列表类型");
    }
}

#[test]
fn test_keys_map_function() {
    let registry = FunctionRegistry::new();
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String("Alice".to_string()));
    map.insert("age".to_string(), Value::Int(30));
    let map_value = Value::Map(map);

    let result = registry.execute("keys", &[map_value]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert_eq!(list.values.len(), 2);
        assert!(list.values.contains(&Value::String("name".to_string())));
        assert!(list.values.contains(&Value::String("age".to_string())));
    } else {
        panic!("期望返回列表类型");
    }
}

// ==================== 路径函数测试 ====================

#[test]
fn test_nodes_function() {
    let registry = FunctionRegistry::new();
    let path = create_test_path();

    let result = registry.execute("nodes", &[Value::Path(path)]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert_eq!(list.values.len(), 3);
    } else {
        panic!("期望返回列表类型");
    }
}

#[test]
fn test_relationships_function() {
    let registry = FunctionRegistry::new();
    let path = create_test_path();

    let result = registry.execute("relationships", &[Value::Path(path)]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert_eq!(list.values.len(), 2);
    } else {
        panic!("期望返回列表类型");
    }
}

// ==================== 数学函数测试 ====================

#[test]
fn test_bit_and_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("bit_and", &[Value::Int(0b1010), Value::Int(0b1100)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(0b1000));
}

#[test]
fn test_bit_or_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("bit_or", &[Value::Int(0b1010), Value::Int(0b1100)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(0b1110));
}

#[test]
fn test_bit_xor_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("bit_xor", &[Value::Int(0b1010), Value::Int(0b1100)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(0b0110));
}

#[test]
fn test_asin_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("asin", &[Value::Float(0.5)]);
    assert!(result.is_ok());
    
    if let Value::Float(val) = result.unwrap() {
        assert!((val - std::f64::consts::PI / 6.0).abs() < 1e-10);
    } else {
        panic!("期望返回浮点类型");
    }
}

#[test]
fn test_acos_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("acos", &[Value::Float(0.5)]);
    assert!(result.is_ok());
    
    if let Value::Float(val) = result.unwrap() {
        assert!((val - std::f64::consts::PI / 3.0).abs() < 1e-10);
    } else {
        panic!("期望返回浮点类型");
    }
}

#[test]
fn test_atan_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("atan", &[Value::Float(1.0)]);
    assert!(result.is_ok());
    
    if let Value::Float(val) = result.unwrap() {
        assert!((val - std::f64::consts::PI / 4.0).abs() < 1e-10);
    } else {
        panic!("期望返回浮点类型");
    }
}

#[test]
fn test_cbrt_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("cbrt", &[Value::Float(27.0)]);
    assert!(result.is_ok());
    
    if let Value::Float(val) = result.unwrap() {
        assert!((val - 3.0).abs() < 1e-10);
    } else {
        panic!("期望返回浮点类型");
    }
}

#[test]
fn test_hypot_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("hypot", &[Value::Float(3.0), Value::Float(4.0)]);
    assert!(result.is_ok());
    
    if let Value::Float(val) = result.unwrap() {
        assert!((val - 5.0).abs() < 1e-10);
    } else {
        panic!("期望返回浮点类型");
    }
}

// ==================== 字符串函数测试 ====================

#[test]
fn test_split_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("split", &[
        Value::String("hello,world,test".to_string()),
        Value::String(",".to_string()),
    ]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert_eq!(list.values.len(), 3);
        assert_eq!(list.values[0], Value::String("hello".to_string()));
        assert_eq!(list.values[1], Value::String("world".to_string()));
        assert_eq!(list.values[2], Value::String("test".to_string()));
    } else {
        panic!("期望返回列表类型");
    }
}

// ==================== 实用函数测试 ====================

#[test]
fn test_coalesce_function() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("coalesce", &[
        Value::Null(NullType::Null),
        Value::Int(42),
        Value::String("test".to_string()),
    ]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));
}

#[test]
fn test_coalesce_all_null() {
    let registry = FunctionRegistry::new();

    let result = registry.execute("coalesce", &[
        Value::Null(NullType::Null),
        Value::Null(NullType::Null),
    ]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null(NullType::Null));
}

#[test]
fn test_hash_string_function() {
    let registry = FunctionRegistry::new();

    let result1 = registry.execute("hash", &[Value::String("test".to_string())]);
    let result2 = registry.execute("hash", &[Value::String("test".to_string())]);
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert_eq!(result1.unwrap(), result2.unwrap());
}

#[test]
fn test_hash_int_function() {
    let registry = FunctionRegistry::new();

    let result1 = registry.execute("hash", &[Value::Int(12345)]);
    let result2 = registry.execute("hash", &[Value::Int(12345)]);
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert_eq!(result1.unwrap(), result2.unwrap());
}

// ==================== NULL 处理测试 ====================

#[test]
fn test_null_handling() {
    let registry = FunctionRegistry::new();

    // 测试 id(NULL)
    let result = registry.execute("id", &[Value::Null(NullType::Null)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null(NullType::Null));

    // 测试 tags(NULL)
    let result = registry.execute("tags", &[Value::Null(NullType::Null)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null(NullType::Null));

    // 测试 head(NULL)
    let result = registry.execute("head", &[Value::Null(NullType::Null)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null(NullType::Null));

    // 测试 size(NULL)
    let result = registry.execute("size", &[Value::Null(NullType::Null)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null(NullType::Null));

    // 测试 nodes(NULL)
    let result = registry.execute("nodes", &[Value::Null(NullType::Null)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null(NullType::Null));

    // 测试 hash(NULL)
    let result = registry.execute("hash", &[Value::Null(NullType::Null)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null(NullType::Null));
}

// ==================== 边界情况测试 ====================

#[test]
fn test_empty_list_operations() {
    let registry = FunctionRegistry::new();
    let empty_list = Value::List(List { values: vec![] });

    // head(空列表) 应该返回 NULL
    let result = registry.execute("head", &[empty_list.clone()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null(NullType::Null));

    // last(空列表) 应该返回 NULL
    let result = registry.execute("last", &[empty_list.clone()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null(NullType::Null));

    // tail(空列表) 应该返回空列表
    let result = registry.execute("tail", &[empty_list.clone()]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert!(list.values.is_empty());
    } else {
        panic!("期望返回列表类型");
    }

    // size(空列表) 应该返回 0
    let result = registry.execute("size", &[empty_list]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(0));
}

#[test]
fn test_empty_path() {
    let registry = FunctionRegistry::new();
    let v1 = create_test_vertex(1, vec![("Person", HashMap::new())]);
    let empty_path = Path::new(v1);

    // nodes(空路径) 应该返回包含起点的列表
    let result = registry.execute("nodes", &[Value::Path(empty_path.clone())]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert_eq!(list.values.len(), 1);
    } else {
        panic!("期望返回列表类型");
    }

    // relationships(空路径) 应该返回空列表
    let result = registry.execute("relationships", &[Value::Path(empty_path)]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert!(list.values.is_empty());
    } else {
        panic!("期望返回列表类型");
    }
}

#[test]
fn test_single_element_list() {
    let registry = FunctionRegistry::new();
    let single_list = Value::List(List {
        values: vec![Value::Int(42)],
    });

    let result = registry.execute("head", &[single_list.clone()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));

    let result = registry.execute("last", &[single_list.clone()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Int(42));

    let result = registry.execute("tail", &[single_list]);
    assert!(result.is_ok());
    
    if let Value::List(list) = result.unwrap() {
        assert!(list.values.is_empty());
    } else {
        panic!("期望返回列表类型");
    }
}

// ==================== 函数存在性测试 ====================

#[test]
fn test_all_functions_registered() {
    let registry = FunctionRegistry::new();

    // 图相关函数
    assert!(registry.contains("id"));
    assert!(registry.contains("tags"));
    assert!(registry.contains("labels"));
    assert!(registry.contains("properties"));
    assert!(registry.contains("type"));
    assert!(registry.contains("src"));
    assert!(registry.contains("dst"));
    assert!(registry.contains("rank"));

    // 容器操作函数
    assert!(registry.contains("head"));
    assert!(registry.contains("last"));
    assert!(registry.contains("tail"));
    assert!(registry.contains("size"));
    assert!(registry.contains("range"));
    assert!(registry.contains("keys"));

    // 路径函数
    assert!(registry.contains("nodes"));
    assert!(registry.contains("relationships"));

    // 数学函数
    assert!(registry.contains("bit_and"));
    assert!(registry.contains("bit_or"));
    assert!(registry.contains("bit_xor"));
    assert!(registry.contains("asin"));
    assert!(registry.contains("acos"));
    assert!(registry.contains("atan"));
    assert!(registry.contains("cbrt"));
    assert!(registry.contains("hypot"));

    // 字符串函数
    assert!(registry.contains("split"));

    // 实用函数
    assert!(registry.contains("coalesce"));
    assert!(registry.contains("hash"));
}
