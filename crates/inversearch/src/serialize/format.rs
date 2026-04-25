//! Serialized Format Processing Module
//!
//! Provide serialization and deserialization functions in different formats (JSON, Binary, MessagePack, CBOR)

use crate::error::Result;
use crate::serialize::types::{IndexExportData, SerializeFormat};
use oxicode::config::standard;
use oxicode::serde::{decode_from_slice, encode_to_vec};

/// Serialize data into byte arrays (choose serialization method based on format)
pub fn serialize_to_bytes(data: &IndexExportData, format: &SerializeFormat) -> Result<Vec<u8>> {
    let serialized = match format {
        SerializeFormat::Json => serde_json::to_vec(data)?,
        SerializeFormat::Binary | SerializeFormat::MessagePack => encode_to_vec(data, standard())?,
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

/// Deserialize data from byte arrays (choose deserialization method based on format)
pub fn deserialize_from_bytes(bytes: &[u8], format: &SerializeFormat) -> Result<IndexExportData> {
    let data: IndexExportData = match format {
        SerializeFormat::Json => serde_json::from_slice(bytes)?,
        SerializeFormat::Binary | SerializeFormat::MessagePack => {
            let (data, _) = decode_from_slice::<IndexExportData, _>(bytes, standard())?;
            data
        }
        SerializeFormat::Cbor => ciborium::from_reader(bytes).map_err(|e| {
            crate::error::InversearchError::Deserialization(format!(
                "CBOR deserialization error: {:?}",
                e
            ))
        })?,
    };
    Ok(data)
}

/// Serialize data to a JSON string
pub fn to_json_string(data: &IndexExportData) -> Result<String> {
    Ok(serde_json::to_string_pretty(data)?)
}

/// Deserializing data from a JSON string
pub fn from_json_str(json_str: &str) -> Result<IndexExportData> {
    Ok(serde_json::from_str(json_str)?)
}
