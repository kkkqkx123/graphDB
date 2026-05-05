//! Index Key Types and Constants
//!
//! This module defines the core types and constants used for index key encoding.

use crate::core::{StorageError, Value};
use oxicode::{decode_from_slice, encode_to_vec};

/// Byte key wrapper for index keys
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ByteKey(pub Vec<u8>);

impl AsRef<[u8]> for ByteKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for ByteKey {
    fn from(v: Vec<u8>) -> Self {
        ByteKey(v)
    }
}

impl From<ByteKey> for Vec<u8> {
    fn from(key: ByteKey) -> Self {
        key.0
    }
}

impl Default for ByteKey {
    fn default() -> Self {
        ByteKey(Vec::new())
    }
}

pub type IndexKey = Vec<u8>;

pub const KEY_TYPE_VERTEX_REVERSE: u8 = 0x01;
pub const KEY_TYPE_EDGE_REVERSE: u8 = 0x02;
pub const KEY_TYPE_VERTEX_FORWARD: u8 = 0x03;
pub const KEY_TYPE_EDGE_FORWARD: u8 = 0x04;

pub fn serialize_value(value: &Value) -> Result<Vec<u8>, StorageError> {
    encode_to_vec(value).map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn deserialize_value(data: &[u8]) -> Result<Value, StorageError> {
    decode_from_slice(data)
        .map(|(v, _)| v)
        .map_err(|e| StorageError::DeserializeError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_value() {
        let value = Value::String("test".to_string());
        let bytes = serialize_value(&value).expect("serialize_value should succeed");
        let decoded = deserialize_value(&bytes).expect("deserialize_value should succeed");
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_byte_key_from_vec() {
        let vec = vec![1, 2, 3, 4];
        let key: ByteKey = vec.clone().into();
        assert_eq!(key.0, vec);
    }

    #[test]
    fn test_byte_key_as_ref() {
        let key = ByteKey(vec![1, 2, 3]);
        assert_eq!(key.as_ref(), &[1, 2, 3]);
    }
}
