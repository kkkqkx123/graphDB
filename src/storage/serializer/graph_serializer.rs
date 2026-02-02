use crate::core::{Vertex, Edge};
use bincode;
use crate::core::StorageError;

pub fn vertex_to_bytes(vertex: &Vertex) -> Result<Vec<u8>, StorageError> {
    bincode::encode_to_vec(vertex, bincode::config::standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn vertex_from_bytes(bytes: &[u8]) -> Result<Vertex, StorageError> {
    let (vertex, _): (Vertex, usize) =
        bincode::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(vertex)
}

pub fn edge_to_bytes(edge: &Edge) -> Result<Vec<u8>, StorageError> {
    bincode::encode_to_vec(edge, bincode::config::standard())
        .map_err(|e| StorageError::SerializeError(e.to_string()))
}

pub fn edge_from_bytes(bytes: &[u8]) -> Result<Edge, StorageError> {
    let (edge, _): (Edge, usize) =
        bincode::decode_from_slice(bytes, bincode::config::standard())
            .map_err(|e| StorageError::SerializeError(e.to_string()))?;
    Ok(edge)
}
