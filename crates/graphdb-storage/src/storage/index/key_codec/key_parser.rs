//! Index Key Parser
//!
//! This module provides functions for parsing index keys.

use crate::core::{StorageError, Value};

use super::key_types::deserialize_value;

pub struct KeyParser;

impl KeyParser {
    // ========================================================================
    // Vertex Forward Index Key Parsing
    // ========================================================================

    pub fn parse_vertex_id_from_key(key_bytes: &[u8]) -> Result<Value, StorageError> {
        let mut pos = 9;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::db_error("Invalid key: too short".to_string()));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + index_name_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::db_error(
                "Invalid key: missing prop_value_len".to_string(),
            ));
        }
        let prop_value_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + prop_value_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::db_error(
                "Invalid key: missing vertex_id_len".to_string(),
            ));
        }
        let vertex_id_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + vertex_id_len {
            return Err(StorageError::db_error(
                "Invalid key: vertex_id exceeds key length".to_string(),
            ));
        }
        let vertex_id_bytes = &key_bytes[pos..pos + vertex_id_len];
        deserialize_value(vertex_id_bytes)
    }

    // ========================================================================
    // Vertex Reverse Index Key Parsing
    // ========================================================================

    pub fn parse_vertex_reverse_key_v2(
        key_bytes: &[u8],
    ) -> Result<(Vec<u8>, String), StorageError> {
        if key_bytes.len() < 9 {
            return Err(StorageError::db_error(
                "Invalid reverse key v2: too short".to_string(),
            ));
        }

        let mut pos = 9;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::db_error(
                "Invalid reverse key v2: missing vertex_id_len".to_string(),
            ));
        }
        let vertex_id_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + vertex_id_len {
            return Err(StorageError::db_error(
                "Invalid reverse key v2: vertex_id exceeds key length".to_string(),
            ));
        }
        let vertex_id_bytes = key_bytes[pos..pos + vertex_id_len].to_vec();
        pos += vertex_id_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::db_error(
                "Invalid reverse key v2: missing index_name_len".to_string(),
            ));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + index_name_len {
            return Err(StorageError::db_error(
                "Invalid reverse key v2: index_name exceeds key length".to_string(),
            ));
        }
        let index_name = String::from_utf8(key_bytes[pos..pos + index_name_len].to_vec())
            .map_err(|e| StorageError::db_error(format!("Invalid index_name encoding: {}", e)))?;

        Ok((vertex_id_bytes, index_name))
    }

    // ========================================================================
    // Edge Reverse Index Key Parsing
    // ========================================================================

    pub fn parse_edge_reverse_key_v2(
        key_bytes: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>, String), StorageError> {
        if key_bytes.len() < 9 {
            return Err(StorageError::db_error(
                "Invalid edge reverse key v2: too short".to_string(),
            ));
        }

        let mut pos = 9;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::db_error(
                "Invalid edge reverse key v2: missing src_len".to_string(),
            ));
        }
        let src_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + src_len {
            return Err(StorageError::db_error(
                "Invalid edge reverse key v2: src exceeds key length".to_string(),
            ));
        }
        let src_bytes = key_bytes[pos..pos + src_len].to_vec();
        pos += src_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::db_error(
                "Invalid edge reverse key v2: missing dst_len".to_string(),
            ));
        }
        let dst_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + dst_len {
            return Err(StorageError::db_error(
                "Invalid edge reverse key v2: dst exceeds key length".to_string(),
            ));
        }
        let dst_bytes = key_bytes[pos..pos + dst_len].to_vec();
        pos += dst_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::db_error(
                "Invalid edge reverse key v2: missing index_name_len".to_string(),
            ));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + index_name_len {
            return Err(StorageError::db_error(
                "Invalid edge reverse key v2: index_name exceeds key length".to_string(),
            ));
        }
        let index_name = String::from_utf8(key_bytes[pos..pos + index_name_len].to_vec())
            .map_err(|e| StorageError::db_error(format!("Invalid index_name encoding: {}", e)))?;

        Ok((src_bytes, dst_bytes, index_name))
    }
}

#[cfg(test)]
mod tests {
    use super::super::key_types::serialize_value;
    use super::*;
    use crate::core::Value;
    use crate::storage::index::key_codec::key_builder::KeyBuilder;

    #[test]
    fn test_parse_vertex_id_from_key() {
        let space_id = 1u64;
        let index_name = "idx_test";
        let prop_value = Value::String("test_value".to_string());
        let vertex_id = Value::Int(123);

        let key = KeyBuilder::build_vertex_index_key(space_id, index_name, &prop_value, &vertex_id)
            .expect("build_vertex_index_key should succeed");

        let parsed_vid = KeyParser::parse_vertex_id_from_key(&key.0)
            .expect("parse_vertex_id_from_key should succeed");
        assert_eq!(parsed_vid, vertex_id);
    }

    #[test]
    fn test_parse_vertex_reverse_key_v2() {
        let space_id = 1u64;
        let vertex_id = Value::Int(456);
        let index_name = "idx_test";

        let key = KeyBuilder::build_vertex_reverse_key_v2(space_id, &vertex_id, index_name)
            .expect("build_vertex_reverse_key_v2 should succeed");

        let (parsed_vid_bytes, parsed_name) = KeyParser::parse_vertex_reverse_key_v2(&key.0)
            .expect("parse_vertex_reverse_key_v2 should succeed");
        assert_eq!(parsed_name, index_name);

        let vertex_id_bytes = serialize_value(&vertex_id).expect("serialize_value should succeed");
        assert_eq!(parsed_vid_bytes, vertex_id_bytes);
    }

    #[test]
    fn test_parse_edge_reverse_key_v2() {
        let space_id = 1u64;
        let src = Value::Int(100);
        let dst = Value::Int(200);
        let index_name = "edge_idx";

        let key = KeyBuilder::build_edge_reverse_key_v2(space_id, &src, &dst, index_name)
            .expect("build_edge_reverse_key_v2 should succeed");

        let (parsed_src_bytes, parsed_dst_bytes, parsed_name) =
            KeyParser::parse_edge_reverse_key_v2(&key.0)
                .expect("parse_edge_reverse_key_v2 should succeed");
        assert_eq!(parsed_name, index_name);

        let src_bytes = serialize_value(&src).expect("serialize_value should succeed");
        let dst_bytes = serialize_value(&dst).expect("serialize_value should succeed");
        assert_eq!(parsed_src_bytes, src_bytes);
        assert_eq!(parsed_dst_bytes, dst_bytes);
    }
}
