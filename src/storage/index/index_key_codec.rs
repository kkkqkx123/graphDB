//! Index key encoding/decoding module
//!
//! Provide functions for the construction, parsing, and serialization of index keys.
//! Support for both forward and reverse indexing key formats for vertices and edges.

use crate::core::{StorageError, Value};
use crate::storage::redb_types::ByteKey;
use bincode::{config::standard, decode_from_slice, encode_to_vec};

/// Index key type identifier
pub const KEY_TYPE_VERTEX_REVERSE: u8 = 0x01;
pub const KEY_TYPE_EDGE_REVERSE: u8 = 0x02;
pub const KEY_TYPE_VERTEX_FORWARD: u8 = 0x03;
pub const KEY_TYPE_EDGE_FORWARD: u8 = 0x04;

/// Index key encoder/decoder
pub struct IndexKeyCodec;

impl IndexKeyCodec {
    /// Serialized value
    pub fn serialize_value(value: &Value) -> Result<Vec<u8>, StorageError> {
        encode_to_vec(value, standard()).map_err(|e| StorageError::SerializeError(e.to_string()))
    }

    /// Deserialized value
    pub fn deserialize_value(data: &[u8]) -> Result<Value, StorageError> {
        decode_from_slice(data, standard())
            .map(|(v, _)| v)
            .map_err(|e| StorageError::DeserializeError(e.to_string()))
    }

    /// Constructing a vertex forward index key
    /// Format: [space_id: u64] [type: u8=0x03] [index_name_len: u32] [index_name] [prop_value_len: u32] [prop_value] [vertex_id_len: u32] [vertex_id]
    pub fn build_vertex_index_key(
        space_id: u64,
        index_name: &str,
        prop_value: &Value,
        vertex_id: &Value,
    ) -> Result<ByteKey, StorageError> {
        let prop_value_bytes = Self::serialize_value(prop_value)?;
        let vertex_id_bytes = Self::serialize_value(vertex_id)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(prop_value_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&prop_value_bytes);
        key.extend_from_slice(&(vertex_id_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&vertex_id_bytes);

        Ok(ByteKey(key))
    }

    /// Constructing a prefix for the vertex forward index keys (used for range queries)
    pub fn build_vertex_index_prefix(space_id: u64, index_name: &str) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        ByteKey(key)
    }

    /// Parse the `vertex_id` from the vertex index key in a forward direction, starting from the top.
    pub fn parse_vertex_id_from_key(key_bytes: &[u8]) -> Result<Value, StorageError> {
        let mut pos = 9;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError("Invalid key: too short".to_string()));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + index_name_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid key: missing prop_value_len".to_string(),
            ));
        }
        let prop_value_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + prop_value_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid key: missing vertex_id_len".to_string(),
            ));
        }
        let vertex_id_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + vertex_id_len {
            return Err(StorageError::DbError(
                "Invalid key: vertex_id exceeds key length".to_string(),
            ));
        }
        let vertex_id_bytes = &key_bytes[pos..pos + vertex_id_len];
        Self::deserialize_value(vertex_id_bytes)
    }

    /// Constructing a reverse index key for vertices
    /// Format: [space_id: u64] [type: u8=0x01] [index_name_len: u32] [index_name] [vertex_id_len: u32] [vertex_id]
    pub fn build_vertex_reverse_key(
        space_id: u64,
        index_name: &str,
        vertex_id: &Value,
    ) -> Result<ByteKey, StorageError> {
        let vertex_id_bytes = Self::serialize_value(vertex_id)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_REVERSE);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(vertex_id_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&vertex_id_bytes);

        Ok(ByteKey(key))
    }

    /// Constructing a prefix for the vertex reverse index key
    pub fn build_vertex_reverse_prefix(space_id: u64) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_REVERSE);
        ByteKey(key)
    }

    /// Analyzing the reverse index keys of the vertices
    pub fn parse_vertex_reverse_key(key_bytes: &[u8]) -> Result<(String, Vec<u8>), StorageError> {
        if key_bytes.len() < 9 {
            return Err(StorageError::DbError(
                "Invalid reverse key: too short".to_string(),
            ));
        }

        let mut pos = 9;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid reverse key: missing index_name_len".to_string(),
            ));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + index_name_len {
            return Err(StorageError::DbError(
                "Invalid reverse key: index_name exceeds key length".to_string(),
            ));
        }
        let index_name = String::from_utf8(key_bytes[pos..pos + index_name_len].to_vec())
            .map_err(|e| StorageError::DbError(format!("Invalid index_name encoding: {}", e)))?;
        pos += index_name_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid reverse key: missing vertex_id_len".to_string(),
            ));
        }
        let vertex_id_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + vertex_id_len {
            return Err(StorageError::DbError(
                "Invalid reverse key: vertex_id exceeds key length".to_string(),
            ));
        }
        let vertex_id_bytes = key_bytes[pos..pos + vertex_id_len].to_vec();

        Ok((index_name, vertex_id_bytes))
    }

    /// Constructing a forward index key for edges
    /// Format: [space_id: u64] [type: u8=0x04] [index_name_len: u32] [index_name] [prop_value_len: u32] [prop_value] [src_len: u32] [src] [dst_len: u32] [dst]
    pub fn build_edge_index_key(
        space_id: u64,
        index_name: &str,
        prop_value: &Value,
        src: &Value,
        dst: &Value,
    ) -> Result<ByteKey, StorageError> {
        let prop_value_bytes = Self::serialize_value(prop_value)?;
        let src_bytes = Self::serialize_value(src)?;
        let dst_bytes = Self::serialize_value(dst)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(prop_value_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&prop_value_bytes);
        key.extend_from_slice(&(src_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&src_bytes);
        key.extend_from_slice(&(dst_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&dst_bytes);

        Ok(ByteKey(key))
    }

    /// Constructing a prefix for the edge forward index key
    pub fn build_edge_index_prefix(space_id: u64, index_name: &str) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        ByteKey(key)
    }

    /// Constructing a reverse index key for edges
    /// Format: [space_id: u64] [type: u8=0x02] [index_name_len: u32] [index_name] [src_len: u32] [src]
    pub fn build_edge_reverse_key(
        space_id: u64,
        index_name: &str,
        src: &Value,
    ) -> Result<ByteKey, StorageError> {
        let src_bytes = Self::serialize_value(src)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(src_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&src_bytes);

        Ok(ByteKey(key))
    }

    /// Constructing a prefix for the reverse index key of an edge
    pub fn build_edge_reverse_prefix(space_id: u64) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        ByteKey(key)
    }

    /// The end key for constructing a range query (the value that follows the prefix)
    pub fn build_range_end(prefix: &ByteKey) -> ByteKey {
        let mut end = prefix.0.clone();
        for i in (0..end.len()).rev() {
            if end[i] == 255 {
                end[i] = 0;
            } else {
                end[i] += 1;
                break;
            }
        }
        ByteKey(end)
    }

    /// Analyzing the reverse index keys on the edges
    pub fn parse_edge_reverse_key(key_bytes: &[u8]) -> Result<(String, Vec<u8>), StorageError> {
        if key_bytes.len() < 9 {
            return Err(StorageError::DbError(
                "Invalid edge reverse key: too short".to_string(),
            ));
        }

        let mut pos = 9;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid edge reverse key: missing index_name_len".to_string(),
            ));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + index_name_len {
            return Err(StorageError::DbError(
                "Invalid edge reverse key: index_name exceeds key length".to_string(),
            ));
        }
        let index_name = String::from_utf8(key_bytes[pos..pos + index_name_len].to_vec())
            .map_err(|e| StorageError::DbError(format!("Invalid index_name encoding: {}", e)))?;
        pos += index_name_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid edge reverse key: missing src_len".to_string(),
            ));
        }
        let src_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + src_len {
            return Err(StorageError::DbError(
                "Invalid edge reverse key: src exceeds key length".to_string(),
            ));
        }
        let src_bytes = key_bytes[pos..pos + src_len].to_vec();

        Ok((index_name, src_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_serialize_deserialize_value() {
        let value = Value::String("test".to_string());
        let bytes = IndexKeyCodec::serialize_value(&value).expect("serialize_value should succeed");
        let decoded =
            IndexKeyCodec::deserialize_value(&bytes).expect("deserialize_value should succeed");
        assert_eq!(value, decoded);
    }

    #[test]
    fn test_build_vertex_index_key() {
        let space_id = 1u64;
        let index_name = "idx_test";
        let prop_value = Value::String("test_value".to_string());
        let vertex_id = Value::Int(123);

        let key =
            IndexKeyCodec::build_vertex_index_key(space_id, index_name, &prop_value, &vertex_id)
                .expect("build_vertex_index_key should succeed");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_VERTEX_FORWARD);

        let parsed_vid = IndexKeyCodec::parse_vertex_id_from_key(&key.0)
            .expect("parse_vertex_id_from_key should succeed");
        assert_eq!(parsed_vid, vertex_id);
    }

    #[test]
    fn test_build_vertex_reverse_key() {
        let space_id = 1u64;
        let index_name = "idx_test";
        let vertex_id = Value::Int(456);

        let key = IndexKeyCodec::build_vertex_reverse_key(space_id, index_name, &vertex_id)
            .expect("build_vertex_reverse_key should succeed");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_VERTEX_REVERSE);

        let (parsed_name, parsed_vid_bytes) = IndexKeyCodec::parse_vertex_reverse_key(&key.0)
            .expect("parse_vertex_reverse_key should succeed");
        assert_eq!(parsed_name, index_name);
        let parsed_vid = IndexKeyCodec::deserialize_value(&parsed_vid_bytes)
            .expect("deserialize_value should succeed");
        assert_eq!(parsed_vid, vertex_id);
    }

    #[test]
    fn test_build_edge_index_key() {
        let space_id = 1u64;
        let index_name = "idx_edge_test";
        let prop_value = Value::String("edge_prop".to_string());
        let src = Value::Int(100);
        let dst = Value::Int(200);

        let key =
            IndexKeyCodec::build_edge_index_key(space_id, index_name, &prop_value, &src, &dst)
                .expect("build_edge_index_key should succeed");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_EDGE_FORWARD);
    }

    #[test]
    fn test_build_edge_reverse_key() {
        let space_id = 1u64;
        let index_name = "idx_edge_test";
        let src = Value::Int(300);

        let key = IndexKeyCodec::build_edge_reverse_key(space_id, index_name, &src)
            .expect("build_edge_reverse_key should succeed");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_EDGE_REVERSE);

        let (parsed_name, parsed_src_bytes) = IndexKeyCodec::parse_edge_reverse_key(&key.0)
            .expect("parse_edge_reverse_key should succeed");
        assert_eq!(parsed_name, index_name);
        let parsed_src = IndexKeyCodec::deserialize_value(&parsed_src_bytes)
            .expect("deserialize_value should succeed");
        assert_eq!(parsed_src, src);
    }

    #[test]
    fn test_build_range_end() {
        let prefix = ByteKey(vec![1, 2, 3]);
        let end = IndexKeyCodec::build_range_end(&prefix);
        assert_eq!(end.0, vec![1, 2, 4]);

        let prefix_max = ByteKey(vec![1, 2, 255]);
        let end_max = IndexKeyCodec::build_range_end(&prefix_max);
        assert_eq!(end_max.0, vec![1, 3, 0]);
    }
}
