use crate::core::StorageError;
use crate::index::Index;
use bincode;

pub fn index_to_bytes(index: &Index) -> Result<Vec<u8>, StorageError> {
    bincode::encode_to_vec(index, bincode::config::standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn index_from_bytes(bytes: &[u8]) -> Result<Index, StorageError> {
    let (index, _): (Index, usize) =
        bincode::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(index)
}

pub fn index_id_to_bytes(index_id: &i32) -> Result<Vec<u8>, StorageError> {
    bincode::encode_to_vec(index_id, bincode::config::standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn index_id_from_bytes(bytes: &[u8]) -> Result<i32, StorageError> {
    let (index_id, _): (i32, usize) =
        bincode::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(index_id)
}
