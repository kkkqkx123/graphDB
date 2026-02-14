//! 存储层测试辅助模块
//!
//! 提供存储层测试的辅助函数

use graphdb::core::types::{SpaceInfo, TagInfo, EdgeTypeInfo, PropertyDef};
use graphdb::core::DataType;
use graphdb::core::Value;
use graphdb::index::{Index, IndexType, IndexField};

/// 创建测试图空间信息
pub fn create_test_space(name: &str) -> SpaceInfo {
    SpaceInfo::new(name.to_string())
        .with_vid_type(DataType::Int64)
        .with_comment(Some("测试空间".to_string()))
}

/// 创建标签信息
pub fn create_tag_info(name: &str, properties: Vec<(&str, DataType)>) -> TagInfo {
    let props = properties
        .into_iter()
        .map(|(name, data_type)| PropertyDef::new(name.to_string(), data_type))
        .collect();

    TagInfo::new(name.to_string())
        .with_properties(props)
}

/// 创建边类型信息
pub fn create_edge_type_info(name: &str, properties: Vec<(&str, DataType)>) -> EdgeTypeInfo {
    let props = properties
        .into_iter()
        .map(|(name, data_type)| PropertyDef::new(name.to_string(), data_type))
        .collect();

    EdgeTypeInfo::new(name.to_string())
        .with_properties(props)
}

/// 创建 Person 标签信息（常用测试标签）
pub fn person_tag_info() -> TagInfo {
    create_tag_info(
        "Person",
        vec![
            ("name", DataType::String),
            ("age", DataType::Int64),
        ],
    )
}

/// 创建 KNOWS 边类型信息（常用测试边类型）
pub fn knows_edge_type_info() -> EdgeTypeInfo {
    create_edge_type_info("KNOWS", vec![("since", DataType::Date)])
}

/// 创建测试 Tag 索引
pub fn create_tag_index(id: i32, name: &str, tag_name: &str, properties: Vec<&str>) -> Index {
    let fields = properties
        .iter()
        .map(|prop| IndexField::new(prop.to_string(), Value::String("string".to_string()), false))
        .collect();

    Index::new(
        id,
        name.to_string(),
        0,
        tag_name.to_string(),
        fields,
        properties.iter().map(|s| s.to_string()).collect(),
        IndexType::TagIndex,
        false,
    )
}

/// 创建测试 Edge 索引
pub fn create_edge_index(id: i32, name: &str, edge_type: &str, properties: Vec<&str>) -> Index {
    let fields = properties
        .iter()
        .map(|prop| IndexField::new(prop.to_string(), Value::String("string".to_string()), false))
        .collect();

    Index::new(
        id,
        name.to_string(),
        0,
        edge_type.to_string(),
        fields,
        properties.iter().map(|s| s.to_string()).collect(),
        IndexType::EdgeIndex,
        false,
    )
}

/// 创建唯一索引
pub fn create_unique_tag_index(id: i32, name: &str, tag_name: &str, property: &str) -> Index {
    let field = IndexField::new(property.to_string(), Value::String("string".to_string()), false);

    Index::new(
        id,
        name.to_string(),
        0,
        tag_name.to_string(),
        vec![field],
        vec![property.to_string()],
        IndexType::TagIndex,
        true,
    )
}
