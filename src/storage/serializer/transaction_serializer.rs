use crate::core::StorageError;
use crate::storage::transaction::{LogRecord, LogType};
use bincode;

pub fn log_record_to_bytes(record: &LogRecord) -> Result<Vec<u8>, StorageError> {
    bincode::serde::encode_to_vec(record, bincode::config::standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn log_record_from_bytes(bytes: &[u8]) -> Result<LogRecord, StorageError> {
    let (record, _): (LogRecord, usize) =
        bincode::serde::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(record)
}

pub fn log_type_to_bytes(log_type: &LogType) -> Result<Vec<u8>, StorageError> {
    bincode::encode_to_vec(log_type, bincode::config::standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn log_type_from_bytes(bytes: &[u8]) -> Result<LogType, StorageError> {
    let (log_type, _): (LogType, usize) =
        bincode::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(log_type)
}
