//! 图结构类型定义
//!
//! 包含图数据库中图结构相关的类型定义

use crate::core::DataType;

/// 边的方向类型
///
/// 用于表示边的遍历方向，支持出边、入边和双向遍历
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeDirection {
    /// 出边：从源节点指向目标节点
    Out,
    /// 入边：从目标节点指向源节点
    In,
    /// 双向：同时包含出边和入边
    Both,
}

impl EdgeDirection {
    /// 判断是否包含出边
    pub fn is_outgoing(&self) -> bool {
        matches!(self, EdgeDirection::Out | EdgeDirection::Both)
    }

    /// 判断是否包含入边
    pub fn is_incoming(&self) -> bool {
        matches!(self, EdgeDirection::In | EdgeDirection::Both)
    }

    /// 获取反向方向
    pub fn reverse(&self) -> Self {
        match self {
            EdgeDirection::Out => EdgeDirection::In,
            EdgeDirection::In => EdgeDirection::Out,
            EdgeDirection::Both => EdgeDirection::Both,
        }
    }
}

impl From<&str> for EdgeDirection {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "out" | "outgoing" => EdgeDirection::Out,
            "in" | "incoming" => EdgeDirection::In,
            "both" | "bidirectional" => EdgeDirection::Both,
            _ => EdgeDirection::Both,
        }
    }
}

impl From<String> for EdgeDirection {
    fn from(s: String) -> Self {
        EdgeDirection::from(s.as_str())
    }
}

/// 顶点类型定义
#[derive(Debug, Clone, PartialEq)]
pub struct VertexType {
    pub tag_id: Option<i32>,
    pub tag_name: String,
    pub properties: Vec<PropertyType>,
}

/// 属性类型定义
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyType {
    pub name: String,
    pub type_def: DataType,
    pub is_nullable: bool,
}

/// 边类型定义
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeTypeInfo {
    pub edge_type: i32,
    pub edge_name: String,
    pub src_tag: String,
    pub dst_tag: String,
    pub properties: Vec<PropertyType>,
    pub rank_enabled: bool,
}

/// 路径类型定义
#[derive(Debug, Clone, PartialEq)]
pub enum PathType {
    SimplePath,
    AllPaths,
    ShortestPath,
    NonWeightedShortestPath,
    WeightedShortestPath,
}

/// 路径信息
#[derive(Debug, Clone, PartialEq)]
pub struct PathInfo {
    pub path_type: PathType,
    pub steps: Option<(i32, i32)>,
    pub node_types: Vec<VertexType>,
    pub edge_types: Vec<EdgeTypeInfo>,
}

/// 图结构类型推导器
pub struct GraphTypeInference;

impl GraphTypeInference {
    pub fn new() -> Self {
        Self
    }

    /// 推导顶点类型
    pub fn deduce_vertex_type(&self, tag_name: &str, tag_id: Option<i32>) -> VertexType {
        VertexType {
            tag_id,
            tag_name: tag_name.to_string(),
            properties: Vec::new(),
        }
    }

    /// 推导边类型
    pub fn deduce_edge_type(&self, edge_name: &str, edge_type: i32) -> EdgeTypeInfo {
        EdgeTypeInfo {
            edge_type,
            edge_name: edge_name.to_string(),
            src_tag: String::new(),
            dst_tag: String::new(),
            properties: Vec::new(),
            rank_enabled: true,
        }
    }

    /// 推导路径类型
    pub fn deduce_path_type(&self, path_type: PathType, steps: Option<(i32, i32)>) -> PathInfo {
        PathInfo {
            path_type,
            steps,
            node_types: Vec::new(),
            edge_types: Vec::new(),
        }
    }

    /// 推导属性类型
    pub fn deduce_property_type(&self, prop_name: &str, _object_type: &str) -> Option<DataType> {
        match prop_name.to_lowercase().as_str() {
            "id" => Some(DataType::Int),
            "name" | "title" | "desc" | "description" => Some(DataType::String),
            "age" | "count" | "size" | "year" | "month" | "day" | 
            "hour" | "minute" | "second" => Some(DataType::Int),
            "price" | "score" | "rate" | "ratio" | "percent" | 
            "weight" | "height" | "width" | "length" => Some(DataType::Float),
            "created_at" | "updated_at" | "birthday" | "date" | "time" | "datetime" => {
                Some(DataType::DateTime)
            }
            "active" | "enabled" | "visible" | "valid" | "exists" => Some(DataType::Bool),
            "tags" | "labels" | "categories" => Some(DataType::List),
            "properties" | "attrs" | "attributes" => Some(DataType::Map),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_type_inference_creation() {
        let _inference = GraphTypeInference::new();
        assert!(true);
    }

    #[test]
    fn test_deduce_vertex_type() {
        let inference = GraphTypeInference::new();
        
        let vertex_type = inference.deduce_vertex_type("person", Some(1));
        assert_eq!(vertex_type.tag_name, "person");
        assert_eq!(vertex_type.tag_id, Some(1));
        assert!(vertex_type.properties.is_empty());
    }

    #[test]
    fn test_deduce_edge_type() {
        let inference = GraphTypeInference::new();
        
        let edge_type = inference.deduce_edge_type("knows", 2);
        assert_eq!(edge_type.edge_name, "knows");
        assert_eq!(edge_type.edge_type, 2);
        assert!(edge_type.rank_enabled);
        assert!(edge_type.properties.is_empty());
    }

    #[test]
    fn test_deduce_path_type() {
        let inference = GraphTypeInference::new();
        
        let path_info = inference.deduce_path_type(PathType::ShortestPath, Some((1, 3)));
        assert_eq!(path_info.path_type, PathType::ShortestPath);
        assert_eq!(path_info.steps, Some((1, 3)));
        assert!(path_info.node_types.is_empty());
        assert!(path_info.edge_types.is_empty());
    }

    #[test]
    fn test_deduce_property_type() {
        let inference = GraphTypeInference::new();
        
        assert_eq!(inference.deduce_property_type("id", "person"), Some(DataType::Int));
        assert_eq!(inference.deduce_property_type("name", "person"), Some(DataType::String));
        assert_eq!(inference.deduce_property_type("age", "person"), Some(DataType::Int));
        assert_eq!(inference.deduce_property_type("price", "product"), Some(DataType::Float));
        assert_eq!(inference.deduce_property_type("created_at", "person"), Some(DataType::DateTime));
        assert_eq!(inference.deduce_property_type("active", "person"), Some(DataType::Bool));
        assert_eq!(inference.deduce_property_type("tags", "person"), Some(DataType::List));
        assert_eq!(inference.deduce_property_type("properties", "person"), Some(DataType::Map));
        assert_eq!(inference.deduce_property_type("unknown", "person"), None);
    }
}