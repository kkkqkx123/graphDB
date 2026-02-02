use crate::core::types::{SpaceInfo, TagInfo, EdgeTypeInfo};
use crate::index::Index;
use serde_json;
use crate::core::StorageError;

pub fn space_to_bytes(space: &SpaceInfo) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(space).map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn space_from_bytes(bytes: &[u8]) -> Result<SpaceInfo, StorageError> {
    serde_json::from_slice(bytes).map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn tag_to_bytes(tag: &TagInfo) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(tag).map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn tag_from_bytes(bytes: &[u8]) -> Result<TagInfo, StorageError> {
    serde_json::from_slice(bytes).map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn edge_type_to_bytes(edge_type: &EdgeTypeInfo) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(edge_type).map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn edge_type_from_bytes(bytes: &[u8]) -> Result<EdgeTypeInfo, StorageError> {
    serde_json::from_slice(bytes).map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn index_to_bytes(index: &Index) -> Result<Vec<u8>, StorageError> {
    serde_json::to_vec(index).map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn index_from_bytes(bytes: &[u8]) -> Result<Index, StorageError> {
    serde_json::from_slice(bytes).map_err(|e| StorageError::SerializeError(e.to_string()))
}
