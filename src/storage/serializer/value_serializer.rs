use crate::core::Value;
use bincode;
use crate::core::StorageError;

pub fn value_to_bytes(value: &Value) -> Result<Vec<u8>, StorageError> {
    bincode::encode_to_vec(value, bincode::config::standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}
