use crate::core::types::{SpaceInfo, TagInfo, EdgeTypeInfo};
use crate::index::Index;
use bincode::{decode_from_slice, encode_to_vec, config::standard};
use crate::core::StorageError;

pub fn space_to_bytes(space: &SpaceInfo) -> Result<Vec<u8>, StorageError> {
    encode_to_vec(space, standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn space_from_bytes(bytes: &[u8]) -> Result<SpaceInfo, StorageError> {
    let (space, _): (SpaceInfo, usize) = decode_from_slice(bytes, standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(space)
}

pub fn tag_to_bytes(tag: &TagInfo) -> Result<Vec<u8>, StorageError> {
    encode_to_vec(tag, standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn tag_from_bytes(bytes: &[u8]) -> Result<TagInfo, StorageError> {
    let (tag, _): (TagInfo, usize) = decode_from_slice(bytes, standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(tag)
}

pub fn edge_type_to_bytes(edge_type: &EdgeTypeInfo) -> Result<Vec<u8>, StorageError> {
    encode_to_vec(edge_type, standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn edge_type_from_bytes(bytes: &[u8]) -> Result<EdgeTypeInfo, StorageError> {
    let (edge_type, _): (EdgeTypeInfo, usize) = decode_from_slice(bytes, standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(edge_type)
}

pub fn index_to_bytes(index: &Index) -> Result<Vec<u8>, StorageError> {
    encode_to_vec(index, standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn index_from_bytes(bytes: &[u8]) -> Result<Index, StorageError> {
    let (index, _): (Index, usize) = decode_from_slice(bytes, standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(index)
}
