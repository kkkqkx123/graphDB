//! 序列化格式处理模块
//!
//! 提供不同格式（JSON、Binary、MessagePack、CBOR）的序列化和反序列化功能

use crate::error::Result;
use crate::serialize::types::{IndexExportData, SerializeFormat};

/// 将数据序列化为字节数组（根据格式选择序列化方式）
pub fn serialize_to_bytes(data: &IndexExportData, format: &SerializeFormat) -> Result<Vec<u8>> {
    let serialized = match format {
        SerializeFormat::Json => serde_json::to_vec(data)?,
        SerializeFormat::Binary | SerializeFormat::MessagePack => bincode::serialize(data)?,
        SerializeFormat::Cbor => {
            let mut buf = Vec::new();
            ciborium::into_writer(data, &mut buf).map_err(|e| {
                crate::error::InversearchError::Serialization(format!(
                    "CBOR serialization error: {:?}",
                    e
                ))
            })?;
            buf
        }
    };
    Ok(serialized)
}

/// 从字节数组反序列化数据（根据格式选择反序列化方式）
pub fn deserialize_from_bytes(bytes: &[u8], format: &SerializeFormat) -> Result<IndexExportData> {
    let data: IndexExportData = match format {
        SerializeFormat::Json => serde_json::from_slice(bytes)?,
        SerializeFormat::Binary | SerializeFormat::MessagePack => bincode::deserialize(bytes)?,
        SerializeFormat::Cbor => ciborium::from_reader(bytes).map_err(|e| {
            crate::error::InversearchError::Deserialization(format!(
                "CBOR deserialization error: {:?}",
                e
            ))
        })?,
    };
    Ok(data)
}

/// 将数据序列化为 JSON 字符串
pub fn to_json_string(data: &IndexExportData) -> Result<String> {
    Ok(serde_json::to_string_pretty(data)?)
}

/// 从 JSON 字符串反序列化数据
pub fn from_json_str(json_str: &str) -> Result<IndexExportData> {
    Ok(serde_json::from_str(json_str)?)
}
