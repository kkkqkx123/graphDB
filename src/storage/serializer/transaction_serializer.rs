use crate::storage::transaction::{LogRecord, LogType};
use bincode::{decode_from_slice, encode_to_vec, config::standard};
use crate::core::StorageError as CoreStorageError;

pub fn log_record_to_bytes(record: &LogRecord) -> Result<Vec<u8>, CoreStorageError> {
    encode_to_vec(record, standard())
        .map_err(|e| CoreStorageError::SerializeError(e.to_string()))
}

pub fn log_record_from_bytes(bytes: &[u8]) -> Result<LogRecord, CoreStorageError> {
    let (record, _): (LogRecord, usize) = decode_from_slice(bytes, standard())
        .map_err(|e| CoreStorageError::SerializeError(e.to_string()))?;
    Ok(record)
}

pub fn log_type_to_bytes(log_type: &LogType) -> Result<Vec<u8>, CoreStorageError> {
    encode_to_vec(log_type, standard())
        .map_err(|e| CoreStorageError::SerializeError(e.to_string()))
}

pub fn log_type_from_bytes(bytes: &[u8]) -> Result<LogType, CoreStorageError> {
    let (log_type, _): (LogType, usize) = decode_from_slice(bytes, standard())
        .map_err(|e| CoreStorageError::SerializeError(e.to_string()))?;
    Ok(log_type)
}
