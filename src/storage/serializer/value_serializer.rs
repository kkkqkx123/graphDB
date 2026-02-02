use crate::core::Value;
use bincode;
use crate::core::StorageError;

pub fn value_to_bytes(value: &Value) -> Result<Vec<u8>, StorageError> {
    bincode::encode_to_vec(value, bincode::config::standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn value_from_bytes(bytes: &[u8]) -> Result<Value, StorageError> {
    let (value, _): (Value, usize) =
        bincode::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(value)
}

pub fn generate_id() -> Value {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64;
    Value::Int(id as i64)
}
