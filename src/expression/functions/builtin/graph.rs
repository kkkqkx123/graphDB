//! 图相关函数实现
//!
//! 提供顶点和边的操作函数，包括 id, tags, labels, properties, type, src, dst, rank

use crate::core::error::ExpressionError;
use crate::core::value::dataset::List;
use crate::core::value::NullType;
use crate::core::Value;
use crate::core::vertex_edge_path::Vertex;

/// 图函数枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphFunction {
    Id,
    Tags,
    Labels,
    Properties,
    EdgeType,
    Src,
    Dst,
    Rank,
    StartNode,
    EndNode,
}

impl GraphFunction {
    /// 获取函数名称
    pub fn name(&self) -> &str {
        match self {
            Self::Id => "id",
            Self::Tags => "tags",
            Self::Labels => "labels",
            Self::Properties => "properties",
            Self::EdgeType => "type",
            Self::Src => "src",
            Self::Dst => "dst",
            Self::Rank => "rank",
            Self::StartNode => "startnode",
            Self::EndNode => "endnode",
        }
    }

    /// 获取参数数量
    pub fn arity(&self) -> usize {
        match self {
            Self::Id => 1,
            Self::Tags => 1,
            Self::Labels => 1,
            Self::Properties => 1,
            Self::EdgeType => 1,
            Self::Src => 1,
            Self::Dst => 1,
            Self::Rank => 1,
            Self::StartNode => 1,
            Self::EndNode => 1,
        }
    }

    /// 是否为可变参数函数
    pub fn is_variadic(&self) -> bool {
        false
    }

    /// 获取函数描述
    pub fn description(&self) -> &str {
        match self {
            Self::Id => "获取顶点的ID",
            Self::Tags => "获取顶点的所有标签",
            Self::Labels => "获取顶点的所有标签（别名）",
            Self::Properties => "获取顶点或边的所有属性",
            Self::EdgeType => "获取边的类型",
            Self::Src => "获取边的起始顶点ID",
            Self::Dst => "获取边的目标顶点ID",
            Self::Rank => "获取边的rank值",
            Self::StartNode => "获取边的起始顶点",
            Self::EndNode => "获取边的目标顶点",
        }
    }

    pub fn execute(&self, args: &[Value]) -> Result<Value, ExpressionError> {
        match self {
            Self::Id => execute_id(args),
            Self::Tags => execute_tags(args),
            Self::Labels => execute_labels(args),
            Self::Properties => execute_properties(args),
            Self::EdgeType => execute_edge_type(args),
            Self::Src => execute_src(args),
            Self::Dst => execute_dst(args),
            Self::Rank => execute_rank(args),
            Self::StartNode => execute_startnode(args),
            Self::EndNode => execute_endnode(args),
        }
    }
}

fn execute_id(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("id函数需要1个参数"));
    }
    match &args[0] {
        Value::Vertex(v) => Ok((*v.vid).clone()),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("id函数需要顶点类型")),
    }
}

fn execute_tags(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("tags函数需要1个参数"));
    }
    match &args[0] {
        Value::Vertex(v) => {
            let tags: Vec<Value> = v
                .tags
                .iter()
                .map(|tag| Value::String(tag.name.clone()))
                .collect();
            Ok(Value::List(List { values: tags }))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("tags函数需要顶点类型")),
    }
}

fn execute_labels(args: &[Value]) -> Result<Value, ExpressionError> {
    execute_tags(args)
}

fn execute_properties(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("properties函数需要1个参数"));
    }
    match &args[0] {
        Value::Vertex(v) => {
            let mut props = std::collections::HashMap::new();
            for tag in &v.tags {
                props.extend(tag.properties.clone());
            }
            props.extend(v.properties.clone());
            Ok(Value::Map(props))
        }
        Value::Edge(e) => Ok(Value::Map(e.props.clone())),
        Value::Map(m) => Ok(Value::Map(m.clone())),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error(
            "properties函数需要顶点、边或映射类型",
        )),
    }
}

fn execute_edge_type(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("type函数需要1个参数"));
    }
    match &args[0] {
        Value::Edge(e) => Ok(Value::String(e.edge_type.clone())),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("type函数需要边类型")),
    }
}

fn execute_src(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("src函数需要1个参数"));
    }
    match &args[0] {
        Value::Edge(e) => Ok((*e.src).clone()),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("src函数需要边类型")),
    }
}

fn execute_dst(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("dst函数需要1个参数"));
    }
    match &args[0] {
        Value::Edge(e) => Ok((*e.dst).clone()),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("dst函数需要边类型")),
    }
}

fn execute_rank(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("rank函数需要1个参数"));
    }
    match &args[0] {
        Value::Edge(e) => Ok(Value::Int(e.ranking)),
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("rank函数需要边类型")),
    }
}

fn execute_startnode(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("startnode函数需要1个参数"));
    }
    match &args[0] {
        Value::Edge(e) => {
            let vertex = Vertex::new((*e.src).clone(), vec![]);
            Ok(Value::Vertex(Box::new(vertex)))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("startnode函数需要边类型")),
    }
}

fn execute_endnode(args: &[Value]) -> Result<Value, ExpressionError> {
    if args.len() != 1 {
        return Err(ExpressionError::type_error("endnode函数需要1个参数"));
    }
    match &args[0] {
        Value::Edge(e) => {
            let vertex = Vertex::new((*e.dst).clone(), vec![]);
            Ok(Value::Vertex(Box::new(vertex)))
        }
        Value::Null(_) => Ok(Value::Null(NullType::Null)),
        _ => Err(ExpressionError::type_error("endnode函数需要边类型")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vertex_edge_path::{Edge, Tag};
    use std::collections::HashMap;

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
        let vertex = create_test_vertex();
        let result = GraphFunction::Id
            .execute(&[Value::Vertex(Box::new(vertex))])
            .expect("id函数执行应该成功");
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_tags_function() {
        let vertex = create_test_vertex();
        let result = GraphFunction::Tags
            .execute(&[Value::Vertex(Box::new(vertex))])
            .expect("tags函数执行应该成功");
        if let Value::List(tags) = result {
            assert_eq!(tags.values.len(), 2);
        } else {
            panic!("tags函数应该返回列表");
        }
    }

    #[test]
    fn test_properties_vertex() {
        let vertex = create_test_vertex();
        let result = GraphFunction::Properties
            .execute(&[Value::Vertex(Box::new(vertex))])
            .expect("properties函数执行应该成功");
        if let Value::Map(props) = result {
            assert!(props.contains_key("name"));
            assert!(props.contains_key("age"));
            assert!(props.contains_key("dept"));
        } else {
            panic!("properties函数应该返回映射");
        }
    }

    #[test]
    fn test_type_function() {
        let edge = create_test_edge();
        let result = GraphFunction::EdgeType
            .execute(&[Value::Edge(edge)])
            .expect("type函数执行应该成功");
        assert_eq!(result, Value::String("knows".to_string()));
    }

    #[test]
    fn test_src_function() {
        let edge = create_test_edge();
        let result = GraphFunction::Src
            .execute(&[Value::Edge(edge)])
            .expect("src函数执行应该成功");
        assert_eq!(result, Value::Int(1));
    }

    #[test]
    fn test_dst_function() {
        let edge = create_test_edge();
        let result = GraphFunction::Dst
            .execute(&[Value::Edge(edge)])
            .expect("dst函数执行应该成功");
        assert_eq!(result, Value::Int(2));
    }

    #[test]
    fn test_rank_function() {
        let edge = create_test_edge();
        let result = GraphFunction::Rank
            .execute(&[Value::Edge(edge)])
            .expect("rank函数执行应该成功");
        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_null_handling() {
        let null_value = Value::Null(NullType::Null);

        assert_eq!(
            GraphFunction::Id.execute(&[null_value.clone()]).expect("id函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
        assert_eq!(
            GraphFunction::Tags.execute(&[null_value.clone()]).expect("tags函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
        assert_eq!(
            GraphFunction::Properties
                .execute(&[null_value.clone()])
                .expect("properties函数应该处理NULL"),
            Value::Null(NullType::Null)
        );
    }
}
