//! Index Key Builder
//!
//! This module provides functions for building index keys.

use crate::core::{StorageError, Value};

use super::key_types::{
    serialize_value, ByteKey, KEY_TYPE_EDGE_FORWARD, KEY_TYPE_EDGE_REVERSE,
    KEY_TYPE_VERTEX_FORWARD, KEY_TYPE_VERTEX_REVERSE,
};

pub struct KeyBuilder;

impl KeyBuilder {
    // ========================================================================
    // Vertex Forward Index Keys
    // ========================================================================

    pub fn build_vertex_index_key(
        space_id: u64,
        index_name: &str,
        prop_value: &Value,
        vertex_id: &Value,
    ) -> Result<ByteKey, StorageError> {
        let prop_value_bytes = serialize_value(prop_value)?;
        let vertex_id_bytes = serialize_value(vertex_id)?;

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

    pub fn build_vertex_index_prefix(space_id: u64, index_name: &str) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        ByteKey(key)
    }

    pub fn build_vertex_index_key_native(
        space_id: u64,
        index_name: &str,
        prop_value: &Value,
        vertex_id: u64,
    ) -> Result<ByteKey, StorageError> {
        let prop_value_bytes = serialize_value(prop_value)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(prop_value_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&prop_value_bytes);
        key.extend_from_slice(&vertex_id.to_le_bytes());

        Ok(ByteKey(key))
    }

    // ========================================================================
    // Vertex Reverse Index Keys
    // ========================================================================

    pub fn build_vertex_reverse_key(
        space_id: u64,
        index_name: &str,
        vertex_id: &Value,
    ) -> Result<ByteKey, StorageError> {
        let vertex_id_bytes = serialize_value(vertex_id)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_REVERSE);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(vertex_id_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&vertex_id_bytes);

        Ok(ByteKey(key))
    }

    pub fn build_vertex_reverse_prefix(space_id: u64) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_REVERSE);
        ByteKey(key)
    }

    pub fn build_vertex_reverse_key_v2(
        space_id: u64,
        vertex_id: &Value,
        index_name: &str,
    ) -> Result<ByteKey, StorageError> {
        let vertex_id_bytes = serialize_value(vertex_id)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_REVERSE);
        key.extend_from_slice(&(vertex_id_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&vertex_id_bytes);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());

        Ok(ByteKey(key))
    }

    pub fn build_vertex_reverse_prefix_v2(
        space_id: u64,
        vertex_id: &Value,
    ) -> Result<ByteKey, StorageError> {
        let vertex_id_bytes = serialize_value(vertex_id)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_REVERSE);
        key.extend_from_slice(&(vertex_id_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&vertex_id_bytes);

        Ok(ByteKey(key))
    }

    pub fn build_vertex_reverse_key_native(
        space_id: u64,
        vertex_id: u64,
        index_name: &str,
    ) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_REVERSE);
        key.extend_from_slice(&vertex_id.to_le_bytes());
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());

        ByteKey(key)
    }

    pub fn build_vertex_reverse_prefix_native(space_id: u64, vertex_id: u64) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_REVERSE);
        key.extend_from_slice(&vertex_id.to_le_bytes());

        ByteKey(key)
    }

    // ========================================================================
    // Edge Forward Index Keys
    // ========================================================================

    pub fn build_edge_index_key(
        space_id: u64,
        index_name: &str,
        prop_value: &Value,
        src: &Value,
        dst: &Value,
    ) -> Result<ByteKey, StorageError> {
        let prop_value_bytes = serialize_value(prop_value)?;
        let src_bytes = serialize_value(src)?;
        let dst_bytes = serialize_value(dst)?;

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

    pub fn build_edge_index_prefix(space_id: u64, index_name: &str) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        ByteKey(key)
    }

    pub fn build_edge_index_key_native(
        space_id: u64,
        index_name: &str,
        prop_value: &Value,
        src: u64,
        dst: u64,
    ) -> Result<ByteKey, StorageError> {
        let prop_value_bytes = serialize_value(prop_value)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(prop_value_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&prop_value_bytes);
        key.extend_from_slice(&src.to_le_bytes());
        key.extend_from_slice(&dst.to_le_bytes());

        Ok(ByteKey(key))
    }

    // ========================================================================
    // Edge Reverse Index Keys
    // ========================================================================

    pub fn build_edge_reverse_key(
        space_id: u64,
        index_name: &str,
        src: &Value,
    ) -> Result<ByteKey, StorageError> {
        let src_bytes = serialize_value(src)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(src_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&src_bytes);

        Ok(ByteKey(key))
    }

    pub fn build_edge_reverse_prefix(space_id: u64) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        ByteKey(key)
    }

    pub fn build_edge_reverse_key_v2(
        space_id: u64,
        src: &Value,
        dst: &Value,
        index_name: &str,
    ) -> Result<ByteKey, StorageError> {
        let src_bytes = serialize_value(src)?;
        let dst_bytes = serialize_value(dst)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        key.extend_from_slice(&(src_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&src_bytes);
        key.extend_from_slice(&(dst_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&dst_bytes);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());

        Ok(ByteKey(key))
    }

    pub fn build_edge_reverse_prefix_v2(
        space_id: u64,
        src: &Value,
    ) -> Result<ByteKey, StorageError> {
        let src_bytes = serialize_value(src)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        key.extend_from_slice(&(src_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&src_bytes);

        Ok(ByteKey(key))
    }

    pub fn build_edge_reverse_prefix_v2_with_dst(
        space_id: u64,
        src: &Value,
        dst: &Value,
    ) -> Result<ByteKey, StorageError> {
        let src_bytes = serialize_value(src)?;
        let dst_bytes = serialize_value(dst)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        key.extend_from_slice(&(src_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&src_bytes);
        key.extend_from_slice(&(dst_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&dst_bytes);

        Ok(ByteKey(key))
    }

    pub fn build_edge_reverse_key_native(
        space_id: u64,
        src: u64,
        dst: u64,
        index_name: &str,
    ) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        key.extend_from_slice(&src.to_le_bytes());
        key.extend_from_slice(&dst.to_le_bytes());
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());

        ByteKey(key)
    }

    pub fn build_edge_reverse_prefix_native(space_id: u64, src: u64) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        key.extend_from_slice(&src.to_le_bytes());

        ByteKey(key)
    }

    pub fn build_edge_reverse_prefix_native_with_dst(space_id: u64, src: u64, dst: u64) -> ByteKey {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_REVERSE);
        key.extend_from_slice(&src.to_le_bytes());
        key.extend_from_slice(&dst.to_le_bytes());

        ByteKey(key)
    }

    // ========================================================================
    // Composite Index Keys
    // ========================================================================

    pub fn build_composite_vertex_index_key(
        space_id: u64,
        index_name: &str,
        field_values: &[Value],
        vertex_id: &Value,
    ) -> Result<ByteKey, StorageError> {
        let vertex_id_bytes = serialize_value(vertex_id)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_VERTEX_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(field_values.len() as u32).to_le_bytes());

        for value in field_values {
            let value_bytes = serialize_value(value)?;
            key.extend_from_slice(&(value_bytes.len() as u32).to_le_bytes());
            key.extend_from_slice(&value_bytes);
        }

        key.extend_from_slice(&(vertex_id_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&vertex_id_bytes);

        Ok(ByteKey(key))
    }

    pub fn build_composite_edge_index_key(
        space_id: u64,
        index_name: &str,
        field_values: &[Value],
        src: &Value,
        dst: &Value,
    ) -> Result<ByteKey, StorageError> {
        let src_bytes = serialize_value(src)?;
        let dst_bytes = serialize_value(dst)?;

        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_EDGE_FORWARD);
        key.extend_from_slice(&(index_name.len() as u32).to_le_bytes());
        key.extend_from_slice(index_name.as_bytes());
        key.extend_from_slice(&(field_values.len() as u32).to_le_bytes());

        for value in field_values {
            let value_bytes = serialize_value(value)?;
            key.extend_from_slice(&(value_bytes.len() as u32).to_le_bytes());
            key.extend_from_slice(&value_bytes);
        }

        key.extend_from_slice(&(src_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&src_bytes);
        key.extend_from_slice(&(dst_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&dst_bytes);

        Ok(ByteKey(key))
    }

    // ========================================================================
    // Range Query Helpers
    // ========================================================================

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_build_vertex_index_key() {
        let space_id = 1u64;
        let index_name = "idx_test";
        let prop_value = Value::String("test_value".to_string());
        let vertex_id = Value::Int(123);

        let key = KeyBuilder::build_vertex_index_key(space_id, index_name, &prop_value, &vertex_id)
            .expect("build_vertex_index_key should succeed");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_VERTEX_FORWARD);
    }

    #[test]
    fn test_build_vertex_reverse_key_v2() {
        let space_id = 1u64;
        let vertex_id = Value::Int(456);
        let index_name = "idx_test";

        let key = KeyBuilder::build_vertex_reverse_key_v2(space_id, &vertex_id, index_name)
            .expect("build_vertex_reverse_key_v2 should succeed");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_VERTEX_REVERSE);
    }

    #[test]
    fn test_build_edge_index_key() {
        let space_id = 1u64;
        let index_name = "edge_idx";
        let prop_value = Value::String("edge_prop".to_string());
        let src = Value::Int(100);
        let dst = Value::Int(200);

        let key = KeyBuilder::build_edge_index_key(space_id, index_name, &prop_value, &src, &dst)
            .expect("build_edge_index_key should succeed");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_EDGE_FORWARD);
    }

    #[test]
    fn test_build_edge_reverse_key_v2() {
        let space_id = 1u64;
        let src = Value::Int(100);
        let dst = Value::Int(200);
        let index_name = "edge_idx";

        let key = KeyBuilder::build_edge_reverse_key_v2(space_id, &src, &dst, index_name)
            .expect("build_edge_reverse_key_v2 should succeed");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_EDGE_REVERSE);
    }

    #[test]
    fn test_build_range_end() {
        let prefix = ByteKey(vec![1, 2, 3]);
        let end = KeyBuilder::build_range_end(&prefix);
        assert_eq!(end.0, vec![1, 2, 4]);
    }

    #[test]
    fn test_build_range_end_overflow() {
        let prefix = ByteKey(vec![1, 255, 255]);
        let end = KeyBuilder::build_range_end(&prefix);
        assert_eq!(end.0, vec![2, 0, 0]);
    }

    #[test]
    fn test_build_vertex_index_key_native() {
        let space_id = 1u64;
        let index_name = "idx_test";
        let prop_value = Value::String("test_value".to_string());
        let vertex_id = 123u64;

        let key =
            KeyBuilder::build_vertex_index_key_native(space_id, index_name, &prop_value, vertex_id)
                .expect("build_vertex_index_key_native should succeed");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_VERTEX_FORWARD);
    }

    #[test]
    fn test_build_edge_index_key_native() {
        let space_id = 1u64;
        let index_name = "edge_idx";
        let prop_value = Value::String("edge_prop".to_string());
        let src = 100u64;
        let dst = 200u64;

        let key =
            KeyBuilder::build_edge_index_key_native(space_id, index_name, &prop_value, src, dst)
                .expect("build_edge_index_key_native should succeed");

        assert!(key.0.len() > 9);
        assert_eq!(key.0[8], KEY_TYPE_EDGE_FORWARD);
    }
}
