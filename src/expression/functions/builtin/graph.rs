//! 图相关函数实现
//!
//! 提供顶点和边的操作函数，包括 id, tags, labels, properties, type, src, dst, rank

use crate::core::error::ExpressionError;
use crate::core::value::dataset::List;
use crate::core::Value;
use crate::expression::functions::registry::FunctionRegistry;
use crate::expression::functions::signature::FunctionSignature;
use crate::expression::functions::signature::ValueType;

/// 注册所有图相关函数
pub fn register_all(registry: &mut FunctionRegistry) {
    register_id(registry);
    register_tags(registry);
    register_properties(registry);
    register_edge_type(registry);
    register_src(registry);
    register_dst(registry);
    register_rank(registry);
}

fn register_id(registry: &mut FunctionRegistry) {
    registry.register(
        "id",
        FunctionSignature::new(
            "id",
            vec![ValueType::Null],
            ValueType::Null,
            1,
            1,
            true,
            "获取顶点ID",
        ),
        |args| {
            match &args[0] {
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Ok(Value::Null(crate::core::value::NullType::Null)),
            }
        },
    );

    registry.register(
        "id",
        FunctionSignature::new(
            "id",
            vec![ValueType::Vertex],
            ValueType::Any,
            1,
            1,
            true,
            "获取顶点ID",
        ),
        |args| {
            match &args[0] {
                Value::Vertex(v) => Ok((*v.vid).clone()),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("id函数需要顶点类型")),
            }
        },
    );
}

fn register_tags(registry: &mut FunctionRegistry) {
    for name in ["tags", "labels"] {
        registry.register(
            name,
            FunctionSignature::new(
                name,
                vec![ValueType::Null],
                ValueType::Null,
                1,
                1,
                true,
                "获取顶点标签列表",
            ),
            |args| {
                match &args[0] {
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Ok(Value::Null(crate::core::value::NullType::Null)),
                }
            },
        );

        registry.register(
            name,
            FunctionSignature::new(
                name,
                vec![ValueType::Vertex],
                ValueType::List,
                1,
                1,
                true,
                "获取顶点标签列表",
            ),
            |args| {
                match &args[0] {
                    Value::Vertex(v) => {
                        let tags: Vec<Value> = v
                            .tags
                            .iter()
                            .map(|tag| Value::String(tag.name.clone()))
                            .collect();
                        Ok(Value::List(List { values: tags }))
                    }
                    Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                    _ => Err(ExpressionError::type_error("tags函数需要顶点类型")),
                }
            },
        );
    }
}

fn register_properties(registry: &mut FunctionRegistry) {
    registry.register(
        "properties",
        FunctionSignature::new(
            "properties",
            vec![ValueType::Vertex],
            ValueType::Map,
            1,
            1,
            true,
            "获取顶点属性映射",
        ),
        |args| {
            match &args[0] {
                Value::Vertex(v) => {
                    let mut props = std::collections::HashMap::new();
                    for tag in &v.tags {
                        props.extend(tag.properties.clone());
                    }
                    props.extend(v.properties.clone());
                    Ok(Value::Map(props))
                }
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error(
                    "properties函数需要顶点类型",
                )),
            }
        },
    );

    registry.register(
        "properties",
        FunctionSignature::new(
            "properties",
            vec![ValueType::Edge],
            ValueType::Map,
            1,
            1,
            true,
            "获取边属性映射",
        ),
        |args| {
            match &args[0] {
                Value::Edge(e) => Ok(Value::Map(e.props.clone())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("properties函数需要边类型")),
            }
        },
    );

    registry.register(
        "properties",
        FunctionSignature::new(
            "properties",
            vec![ValueType::Map],
            ValueType::Map,
            1,
            1,
            true,
            "获取映射属性",
        ),
        |args| {
            match &args[0] {
                Value::Map(m) => Ok(Value::Map(m.clone())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("properties函数需要映射类型")),
            }
        },
    );
}

fn register_edge_type(registry: &mut FunctionRegistry) {
    registry.register(
        "type",
        FunctionSignature::new(
            "type",
            vec![ValueType::Null],
            ValueType::Null,
            1,
            1,
            true,
            "获取边类型",
        ),
        |args| {
            match &args[0] {
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Ok(Value::Null(crate::core::value::NullType::Null)),
            }
        },
    );

    registry.register(
        "type",
        FunctionSignature::new(
            "type",
            vec![ValueType::Edge],
            ValueType::String,
            1,
            1,
            true,
            "获取边类型",
        ),
        |args| {
            match &args[0] {
                Value::Edge(e) => Ok(Value::String(e.edge_type.clone())),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("type函数需要边类型")),
            }
        },
    );
}

fn register_src(registry: &mut FunctionRegistry) {
    registry.register(
        "src",
        FunctionSignature::new(
            "src",
            vec![ValueType::Edge],
            ValueType::Any,
            1,
            1,
            true,
            "获取边起点",
        ),
        |args| {
            match &args[0] {
                Value::Edge(e) => Ok((*e.src).clone()),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("src函数需要边类型")),
            }
        },
    );
}

fn register_dst(registry: &mut FunctionRegistry) {
    registry.register(
        "dst",
        FunctionSignature::new(
            "dst",
            vec![ValueType::Edge],
            ValueType::Any,
            1,
            1,
            true,
            "获取边终点",
        ),
        |args| {
            match &args[0] {
                Value::Edge(e) => Ok((*e.dst).clone()),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("dst函数需要边类型")),
            }
        },
    );
}

fn register_rank(registry: &mut FunctionRegistry) {
    registry.register(
        "rank",
        FunctionSignature::new(
            "rank",
            vec![ValueType::Edge],
            ValueType::Int,
            1,
            1,
            true,
            "获取边rank",
        ),
        |args| {
            match &args[0] {
                Value::Edge(e) => Ok(Value::Int(e.ranking)),
                Value::Null(_) => Ok(Value::Null(crate::core::value::NullType::Null)),
                _ => Err(ExpressionError::type_error("rank函数需要边类型")),
            }
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vertex_edge_path::{Edge, Tag, Vertex};
    use std::collections::HashMap;

    fn create_test_registry() -> FunctionRegistry {
        let mut registry = FunctionRegistry::new();
        register_all(&mut registry);
        registry
    }

    fn create_test_vertex() -> Vertex {
        let tag1 = Tag::new(
            "person".to_string(),
            HashMap::from([
                ("name".to_string(), Value::String("Alice".to_string())),
                ("age".to_string(), Value::Int(30)),
            ]),
        );
        let tag2 = Tag::new(
            "employee".to_string(),
            HashMap::from([("dept".to_string(), Value::String("Engineering".to_string()))]),
        );
        Vertex::new(Value::Int(1), vec![tag1, tag2])
    }

    fn create_test_edge() -> Edge {
        Edge::new(
            Value::Int(1),
            Value::Int(2),
            "knows".to_string(),
            0,
            HashMap::from([("since".to_string(), Value::Int(2020))]),
        )
    }

    #[test]
    fn test_id_function() {
        let registry = create_test_registry();
        let vertex = create_test_vertex();
        let result = registry.execute("id", &[Value::Vertex(Box::new(vertex))]).expect("id函数执行应该成功");
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_tags_function() {
        let registry = create_test_registry();
        let vertex = create_test_vertex();
        let result = registry.execute("tags", &[Value::Vertex(Box::new(vertex))]).expect("tags函数执行应该成功");
        if let Value::List(tags) = result {
            assert_eq!(tags.values.len(), 2);
        } else {
            panic!("tags函数应该返回列表");
        }
    }

    #[test]
    fn test_labels_function() {
        let registry = create_test_registry();
        let vertex = create_test_vertex();
        let result = registry
            .execute("labels", &[Value::Vertex(Box::new(vertex))])
            .expect("labels函数执行应该成功");
        if let Value::List(labels) = result {
            assert_eq!(labels.values.len(), 2);
        } else {
            panic!("labels函数应该返回列表");
        }
    }

    #[test]
    fn test_properties_vertex() {
        let registry = create_test_registry();
        let vertex = create_test_vertex();
        let result = registry
            .execute("properties", &[Value::Vertex(Box::new(vertex))])
            .expect("properties函数执行应该成功");
        if let Value::Map(props) = result {
            assert_eq!(props.len(), 3);
            assert!(props.contains_key("name"));
            assert!(props.contains_key("age"));
            assert!(props.contains_key("dept"));
        } else {
            panic!("properties函数应该返回映射");
        }
    }

    #[test]
    fn test_properties_edge() {
        let registry = create_test_registry();
        let edge = create_test_edge();
        let result = registry
            .execute("properties", &[Value::Edge(edge)])
            .expect("properties函数执行应该成功");
        if let Value::Map(props) = result {
            assert_eq!(props.len(), 1);
            assert!(props.contains_key("since"));
        } else {
            panic!("properties函数应该返回映射");
        }
    }

    #[test]
    fn test_type_function() {
        let registry = create_test_registry();
        let edge = create_test_edge();
        let result = registry
            .execute("type", &[Value::Edge(edge)])
            .expect("type函数执行应该成功");
        assert_eq!(result, Value::String("knows".to_string()));
    }

    #[test]
    fn test_src_function() {
        let registry = create_test_registry();
        let edge = create_test_edge();
        let result = registry.execute("src", &[Value::Edge(edge)]).expect("src函数执行应该成功");
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_dst_function() {
        let registry = create_test_registry();
        let edge = create_test_edge();
        let result = registry.execute("dst", &[Value::Edge(edge)]).expect("dst函数执行应该成功");
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_rank_function() {
        let registry = create_test_registry();
        let edge = create_test_edge();
        let result = registry
            .execute("rank", &[Value::Edge(edge)])
            .expect("rank函数执行应该成功");
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_null_handling() {
        let registry = create_test_registry();
        let null_value = Value::Null(crate::core::value::NullType::Null);

        assert_eq!(
            registry.execute("id", &[null_value.clone()]).expect("id函数应该处理NULL"),
            Value::Null(crate::core::value::NullType::Null)
        );
        assert_eq!(
            registry.execute("tags", &[null_value.clone()]).expect("tags函数应该处理NULL"),
            Value::Null(crate::core::value::NullType::Null)
        );
        assert_eq!(
            registry.execute("type", &[null_value.clone()]).expect("type函数应该处理NULL"),
            Value::Null(crate::core::value::NullType::Null)
        );
    }
}
