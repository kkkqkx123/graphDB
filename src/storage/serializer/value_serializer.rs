use crate::core::Value;
use bincode::{decode_from_slice, encode_to_vec, config::standard};
use crate::core::StorageError;

pub fn value_to_bytes(value: &Value) -> Result<Vec<u8>, StorageError> {
    encode_to_vec(value, standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn value_from_bytes(bytes: &[u8]) -> Result<Value, StorageError> {
    let (value, _): (Value, usize) = decode_from_slice(bytes, standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(value)
}
