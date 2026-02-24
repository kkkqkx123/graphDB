//! 路径相关函数实现
//!
//! 提供路径操作函数，包括 nodes, relationships

use crate::core::error::ExpressionError;
use crate::core::value::dataset::List;
use crate::core::value::NullType;
use crate::core::Value;

/// 路径函数枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathFunction {
    Nodes,
    Relationships,
}

impl PathFunction {
    /// 获取函数名称
    pub fn name(&self) -> &str {
        match self {
            Self::Nodes => "nodes",
            Self::Relationships => "relationships",
        }
    }

    /// 获取参数数量
    pub fn arity(&self) -> usize {
        1
    }

    /// 是否为可变参数函数
    pub fn is_variadic(&self) -> bool {
        false
    }

    /// 获取函数描述
    pub fn description(&self) -> &str {
        match self {
            Self::Nodes => "获取路径中的所有顶点",
            Self::Relationships => "获取路径中的所有边",
        }
    }

    pub fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        match self {
            Self::Nodes => execute_nodes(args),
            Self::Relationships => execute_relationships(args),
        }
    }
}

fn execute_nodes(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("nodes函数需要1个参数"));
    }
    match &args[0] {
        Value::Path(path) => {
            let mut result = vec![Value::Vertex(Box::new((*path.src).clone()))];
            for step in &path.steps {
                result.push(Value::Vertex(Box::new((*step.dst).clone())));
            }
            Ok(Value::List(List { values: result }))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("nodes函数需要路径类型")),
    }
}

fn execute_relationships(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("relationships函数需要1个参数"));
    }
    match &args[0] {
        Value::Path(path) => {
            let result: Vec<Value> = path
                .steps
                .iter()
                .map(|step| Value::Edge((*step.edge).clone()))
                .collect();
            Ok(Value::List(List { values: result }))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("relationships函数需要路径类型")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vertex_edge_path::{Edge, Path, Step, Tag, Vertex};
    use std::collections::HashMap;

    fn create_test_vertex_with_id(id: i64) -> Vertex {
        Vertex::new(Value::Int(id), vec![Tag::new("person".to_string(), HashMap::new())])
    }

    fn create_test_path() -> Path {
        let v1 = create_test_vertex_with_id(1);
        let v2 = create_test_vertex_with_id(2);
        let v3 = create_test_vertex_with_id(3);

        let e1 = Edge::new(
            Value::Int(1),
            Value::Int(2),
            "knows".to_string(),
            0,
            HashMap::new(),
        );
        let e2 = Edge::new(
            Value::Int(2),
            Value::Int(3),
            "follows".to_string(),
            0,
            HashMap::new(),
        );

        let mut path = Path::new(v1);
        path.add_step(Step {
            dst: Box::new(v2),
            edge: Box::new(e1),
        });
        path.add_step(Step {
            dst: Box::new(v3),
            edge: Box::new(e2),
        });
        path
    }

    #[test]
    fn test_nodes_function() {
        let path = create_test_path();
        let result = PathFunction::Nodes
            .execute(&[Value::Path(path)])
            .expect("nodes函数执行应该成功");

        if let Value::List(nodes) = result {
            assert_eq!(nodes.values.len(), 3);
            if let Value::Vertex(v) = &nodes.values[0] {
                assert_eq!(*v.vid, Value::Int(1));
            } else {
                panic!("第一个节点应该是顶点");
            }
            if let Value::Vertex(v) = &nodes.values[1] {
                assert_eq!(*v.vid, Value::Int(2));
            } else {
                panic!("第二个节点应该是顶点");
            }
            if let Value::Vertex(v) = &nodes.values[2] {
                assert_eq!(*v.vid, Value::Int(3));
            } else {
                panic!("第三个节点应该是顶点");
            }
        } else {
            panic!("nodes函数应该返回列表");
        }
    }

    #[test]
    fn test_relationships_function() {
        let path = create_test_path();
        let result = PathFunction::Relationships
            .execute(&[Value::Path(path)])
            .expect("relationships函数执行应该成功");

        if let Value::List(edges) = result {
            assert_eq!(edges.values.len(), 2);
            if let Value::Edge(e) = &edges.values[0] {
                assert_eq!(e.edge_type, "knows");
            } else {
                panic!("第一个元素应该是边");
            }
            if let Value::Edge(e) = &edges.values[1] {
                assert_eq!(e.edge_type, "follows");
            } else {
                panic!("第二个元素应该是边");
            }
        } else {
            panic!("relationships函数应该返回列表");
        }
    }

    #[test]
    fn test_nodes_empty_path() {
        let v1 = create_test_vertex_with_id(1);
        let path = Path::new(v1);
        let result = PathFunction::Nodes
            .execute(&[Value::Path(path)])
            .expect("nodes函数执行应该成功");

        if let Value::List(nodes) = result {
            assert_eq!(nodes.values.len(), 1);
        } else {
            panic!("nodes函数应该返回列表");
        }
    }

    #[test]
    fn test_relationships_empty_path() {
        let v1 = create_test_vertex_with_id(1);
        let path = Path::new(v1);
        let result = PathFunction::Relationships
            .execute(&[Value::Path(path)])
            .expect("relationships函数执行应该成功");

        if let Value::List(edges) = result {
            assert_eq!(edges.values.len(), 0);
        } else {
            panic!("relationships函数应该返回列表");
        }
    }

    #[test]
    fn test_null_handling() {
        let null_value = Value::Null(NullType::Null);

        assert_eq!(
            PathFunction::Nodes
                .execute(&[null_value.clone()])
                .expect("nodes函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
        assert_eq!(
            PathFunction::Relationships
                .execute(&[null_value.clone()])
                .expect("relationships函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
    }
}
