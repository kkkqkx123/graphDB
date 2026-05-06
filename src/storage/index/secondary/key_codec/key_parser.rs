//! Index Key Parser
//!
//! This module provides functions for parsing index keys.

use crate::core::{StorageError, Value};

use super::key_types::{deserialize_value, serialize_value};

pub struct KeyParser;

impl KeyParser {
    // ========================================================================
    // Vertex Forward Index Key Parsing
    // ========================================================================

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
        deserialize_value(vertex_id_bytes)
    }

    pub fn parse_vertex_id_from_key_native(key_bytes: &[u8]) -> Result<u64, StorageError> {
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

        if key_bytes.len() < pos + 8 {
            return Err(StorageError::DbError(
                "Invalid key: vertex_id exceeds key length".to_string(),
            ));
        }
        let vertex_id = u64::from_le_bytes(key_bytes[pos..pos + 8].try_into().unwrap_or([0; 8]));

        Ok(vertex_id)
    }

    // ========================================================================
    // Vertex Reverse Index Key Parsing
    // ========================================================================

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

    pub fn parse_vertex_reverse_key_v2(
        key_bytes: &[u8],
    ) -> Result<(Vec<u8>, String), StorageError> {
        if key_bytes.len() < 9 {
            return Err(StorageError::DbError(
                "Invalid reverse key v2: too short".to_string(),
            ));
        }

        let mut pos = 9;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid reverse key v2: missing vertex_id_len".to_string(),
            ));
        }
        let vertex_id_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + vertex_id_len {
            return Err(StorageError::DbError(
                "Invalid reverse key v2: vertex_id exceeds key length".to_string(),
            ));
        }
        let vertex_id_bytes = key_bytes[pos..pos + vertex_id_len].to_vec();
        pos += vertex_id_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid reverse key v2: missing index_name_len".to_string(),
            ));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + index_name_len {
            return Err(StorageError::DbError(
                "Invalid reverse key v2: index_name exceeds key length".to_string(),
            ));
        }
        let index_name = String::from_utf8(key_bytes[pos..pos + index_name_len].to_vec())
            .map_err(|e| StorageError::DbError(format!("Invalid index_name encoding: {}", e)))?;

        Ok((vertex_id_bytes, index_name))
    }

    pub fn parse_vertex_reverse_key_native(
        key_bytes: &[u8],
    ) -> Result<(u64, String), StorageError> {
        if key_bytes.len() < 17 {
            return Err(StorageError::DbError(
                "Invalid reverse key native: too short".to_string(),
            ));
        }

        let vertex_id = u64::from_le_bytes(key_bytes[9..17].try_into().unwrap_or([0; 8]));

        let mut pos = 17;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid reverse key native: missing index_name_len".to_string(),
            ));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + index_name_len {
            return Err(StorageError::DbError(
                "Invalid reverse key native: index_name exceeds key length".to_string(),
            ));
        }
        let index_name = String::from_utf8(key_bytes[pos..pos + index_name_len].to_vec())
            .map_err(|e| StorageError::DbError(format!("Invalid index_name encoding: {}", e)))?;

        Ok((vertex_id, index_name))
    }

    // ========================================================================
    // Edge Forward Index Key Parsing
    // ========================================================================

    pub fn parse_edge_ids_from_key_native(key_bytes: &[u8]) -> Result<(u64, u64), StorageError> {
        let mut pos = 9;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid edge key: too short".to_string(),
            ));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + index_name_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid edge key: missing prop_value_len".to_string(),
            ));
        }
        let prop_value_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + prop_value_len;

        if key_bytes.len() < pos + 16 {
            return Err(StorageError::DbError(
                "Invalid edge key: src/dst exceeds key length".to_string(),
            ));
        }
        let src = u64::from_le_bytes(key_bytes[pos..pos + 8].try_into().unwrap_or([0; 8]));
        let dst = u64::from_le_bytes(key_bytes[pos + 8..pos + 16].try_into().unwrap_or([0; 8]));

        Ok((src, dst))
    }

    // ========================================================================
    // Edge Reverse Index Key Parsing
    // ========================================================================

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

    pub fn parse_edge_reverse_key_v2(
        key_bytes: &[u8],
    ) -> Result<(Vec<u8>, Vec<u8>, String), StorageError> {
        if key_bytes.len() < 9 {
            return Err(StorageError::DbError(
                "Invalid edge reverse key v2: too short".to_string(),
            ));
        }

        let mut pos = 9;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid edge reverse key v2: missing src_len".to_string(),
            ));
        }
        let src_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + src_len {
            return Err(StorageError::DbError(
                "Invalid edge reverse key v2: src exceeds key length".to_string(),
            ));
        }
        let src_bytes = key_bytes[pos..pos + src_len].to_vec();
        pos += src_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid edge reverse key v2: missing dst_len".to_string(),
            ));
        }
        let dst_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + dst_len {
            return Err(StorageError::DbError(
                "Invalid edge reverse key v2: dst exceeds key length".to_string(),
            ));
        }
        let dst_bytes = key_bytes[pos..pos + dst_len].to_vec();
        pos += dst_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid edge reverse key v2: missing index_name_len".to_string(),
            ));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + index_name_len {
            return Err(StorageError::DbError(
                "Invalid edge reverse key v2: index_name exceeds key length".to_string(),
            ));
        }
        let index_name = String::from_utf8(key_bytes[pos..pos + index_name_len].to_vec())
            .map_err(|e| StorageError::DbError(format!("Invalid index_name encoding: {}", e)))?;

        Ok((src_bytes, dst_bytes, index_name))
    }

    pub fn parse_edge_reverse_key_native(
        key_bytes: &[u8],
    ) -> Result<(u64, u64, String), StorageError> {
        if key_bytes.len() < 25 {
            return Err(StorageError::DbError(
                "Invalid edge reverse key native: too short".to_string(),
            ));
        }

        let src = u64::from_le_bytes(key_bytes[9..17].try_into().unwrap_or([0; 8]));
        let dst = u64::from_le_bytes(key_bytes[17..25].try_into().unwrap_or([0; 8]));

        let mut pos = 25;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid edge reverse key native: missing index_name_len".to_string(),
            ));
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + index_name_len {
            return Err(StorageError::DbError(
                "Invalid edge reverse key native: index_name exceeds key length".to_string(),
            ));
        }
        let index_name = String::from_utf8(key_bytes[pos..pos + index_name_len].to_vec())
            .map_err(|e| StorageError::DbError(format!("Invalid index_name encoding: {}", e)))?;

        Ok((src, dst, index_name))
    }

    // ========================================================================
    // Composite Index Key Parsing
    // ========================================================================

    pub fn parse_composite_vertex_index_key(
        key_bytes: &[u8],
    ) -> Result<(Vec<Value>, Value), StorageError> {
        if key_bytes.len() < 9 {
            return Err(StorageError::DbError(
                "Invalid composite vertex key: too short".to_string(),
            ));
        }

        let mut pos = 9;

        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + index_name_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid composite vertex key: missing field_count".to_string(),
            ));
        }
        let field_count =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        let mut field_values = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            if key_bytes.len() < pos + 4 {
                return Err(StorageError::DbError(
                    "Invalid composite vertex key: missing field_len".to_string(),
                ));
            }
            let field_len =
                u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
            pos += 4;

            if key_bytes.len() < pos + field_len {
                return Err(StorageError::DbError(
                    "Invalid composite vertex key: field exceeds key length".to_string(),
                ));
            }
            let field_bytes = &key_bytes[pos..pos + field_len];
            let value = deserialize_value(field_bytes)?;
            field_values.push(value);
            pos += field_len;
        }

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid composite vertex key: missing vertex_id_len".to_string(),
            ));
        }
        let vertex_id_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + vertex_id_len {
            return Err(StorageError::DbError(
                "Invalid composite vertex key: vertex_id exceeds key length".to_string(),
            ));
        }
        let vertex_id_bytes = &key_bytes[pos..pos + vertex_id_len];
        let vertex_id = deserialize_value(vertex_id_bytes)?;

        Ok((field_values, vertex_id))
    }

    pub fn parse_composite_edge_index_key(
        key_bytes: &[u8],
    ) -> Result<(Vec<Value>, Value, Value), StorageError> {
        if key_bytes.len() < 9 {
            return Err(StorageError::DbError(
                "Invalid composite edge key: too short".to_string(),
            ));
        }

        let mut pos = 9;

        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + index_name_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid composite edge key: missing field_count".to_string(),
            ));
        }
        let field_count =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        let mut field_values = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            if key_bytes.len() < pos + 4 {
                return Err(StorageError::DbError(
                    "Invalid composite edge key: missing field_len".to_string(),
                ));
            }
            let field_len =
                u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
            pos += 4;

            if key_bytes.len() < pos + field_len {
                return Err(StorageError::DbError(
                    "Invalid composite edge key: field exceeds key length".to_string(),
                ));
            }
            let field_bytes = &key_bytes[pos..pos + field_len];
            let value = deserialize_value(field_bytes)?;
            field_values.push(value);
            pos += field_len;
        }

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid composite edge key: missing src_len".to_string(),
            ));
        }
        let src_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + src_len {
            return Err(StorageError::DbError(
                "Invalid composite edge key: src exceeds key length".to_string(),
            ));
        }
        let src_bytes = &key_bytes[pos..pos + src_len];
        let src = deserialize_value(src_bytes)?;
        pos += src_len;

        if key_bytes.len() < pos + 4 {
            return Err(StorageError::DbError(
                "Invalid composite edge key: missing dst_len".to_string(),
            ));
        }
        let dst_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4;

        if key_bytes.len() < pos + dst_len {
            return Err(StorageError::DbError(
                "Invalid composite edge key: dst exceeds key length".to_string(),
            ));
        }
        let dst_bytes = &key_bytes[pos..pos + dst_len];
        let dst = deserialize_value(dst_bytes)?;

        Ok((field_values, src, dst))
    }

    pub fn is_composite_key(key_bytes: &[u8]) -> bool {
        if key_bytes.len() < 9 {
            return false;
        }

        let mut pos = 9;

        if key_bytes.len() < pos + 4 {
            return false;
        }
        let index_name_len =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;
        pos += 4 + index_name_len;

        if key_bytes.len() < pos + 4 {
            return false;
        }
        let field_count =
            u32::from_le_bytes(key_bytes[pos..pos + 4].try_into().unwrap_or([0; 4])) as usize;

        field_count > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::storage::index::secondary::key_codec::key_builder::KeyBuilder;

    #[test]
    fn test_parse_vertex_id_from_key() {
        let space_id = 1u64;
        let index_name = "idx_test";
        let prop_value = Value::String("test_value".to_string());
        let vertex_id = Value::Int(123);

        let key =
            KeyBuilder::build_vertex_index_key(space_id, index_name, &prop_value, &vertex_id)
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

    #[test]
    fn test_parse_vertex_id_from_key_native() {
        let space_id = 1u64;
        let index_name = "idx_test";
        let prop_value = Value::String("test_value".to_string());
        let vertex_id = 123u64;

        let key = KeyBuilder::build_vertex_index_key_native(space_id, index_name, &prop_value, vertex_id)
            .expect("build_vertex_index_key_native should succeed");

        let parsed_vid = KeyParser::parse_vertex_id_from_key_native(&key.0)
            .expect("parse_vertex_id_from_key_native should succeed");
        assert_eq!(parsed_vid, vertex_id);
    }

    #[test]
    fn test_parse_edge_ids_from_key_native() {
        let space_id = 1u64;
        let index_name = "edge_idx";
        let prop_value = Value::String("edge_prop".to_string());
        let src = 100u64;
        let dst = 200u64;

        let key = KeyBuilder::build_edge_index_key_native(space_id, index_name, &prop_value, src, dst)
            .expect("build_edge_index_key_native should succeed");

        let (parsed_src, parsed_dst) = KeyParser::parse_edge_ids_from_key_native(&key.0)
            .expect("parse_edge_ids_from_key_native should succeed");
        assert_eq!(parsed_src, src);
        assert_eq!(parsed_dst, dst);
    }
}
