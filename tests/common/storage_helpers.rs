//! Storage Layer Testing Assistance Module
//!
//! Provide auxiliary functions for storage layer testing

#![allow(dead_code)]

use graphdb::core::types::{EdgeTypeInfo, PropertyDef, SpaceInfo, TagInfo};
use graphdb::core::DataType;
use graphdb::storage::RedbStorage;
use parking_lot::{Mutex, MutexGuard};
use std::sync::Arc;

/// Create test image space information
pub fn create_test_space(name: &str) -> SpaceInfo {
    SpaceInfo::new(name.to_string())
        .with_vid_type(DataType::Int64)
        .with_comment(Some("测试空间".to_string()))
}

/// Create tag information
pub fn create_tag_info(name: &str, properties: Vec<(&str, DataType)>) -> TagInfo {
    let props = properties
        .into_iter()
        .map(|(name, data_type)| PropertyDef::new(name.to_string(), data_type))
        .collect();

    TagInfo::new(name.to_string()).with_properties(props)
}

/// Create edge type information.
pub fn create_edge_type_info(name: &str, properties: Vec<(&str, DataType)>) -> EdgeTypeInfo {
    let props = properties
        .into_iter()
        .map(|(name, data_type)| PropertyDef::new(name.to_string(), data_type))
        .collect();

    EdgeTypeInfo::new(name.to_string()).with_properties(props)
}

/// Create Person tag information (commonly used for testing purposes)
pub fn person_tag_info() -> TagInfo {
    create_tag_info(
        "Person",
        vec![("name", DataType::String), ("age", DataType::Int64)],
    )
}

/// Create KNOWS edge type information (commonly used test edge types)
pub fn knows_edge_type_info() -> EdgeTypeInfo {
    create_edge_type_info("KNOWS", vec![("since", DataType::Date)])
}

/// Helper function to get storage guard from Arc<Mutex<RedbStorage>>
pub fn get_storage(
    storage: &Arc<Mutex<RedbStorage>>,
) -> MutexGuard<'_, RedbStorage> {
    storage.lock()
}
