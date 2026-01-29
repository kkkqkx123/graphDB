use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult, StorageError};
use crate::core::{Edge, Value};
use crate::query::executor::base::EdgeDirection;
use crate::storage::StorageClient;
use crate::utils::safe_lock;

pub async fn get_neighbors<S: StorageClient>(
    storage: &Arc<Mutex<S>>,
    node_id: &Value,
    edge_direction: EdgeDirection,
    edge_types: &Option<Vec<String>>,
) -> DBResult<Vec<Value>> {
    let storage_guard = safe_lock(storage)
        .expect("Storage lock should not be poisoned");

    let edges = storage_guard
        .get_node_edges(node_id, EdgeDirection::Both)
        .map_err(|e| DBError::Storage(StorageError::DbError(e.to_string())))?;

    let filtered_edges: Vec<Edge> = if let Some(ref edge_types) = edge_types {
        edges
            .into_iter()
            .filter(|edge| edge_types.contains(&edge.edge_type))
            .collect()
    } else {
        edges
    };

    let neighbors: Vec<Value> = filtered_edges
        .into_iter()
        .filter_map(|edge| match edge_direction {
            EdgeDirection::In => {
                if *edge.dst == *node_id {
                    Some((*edge.src).clone())
                } else {
                    None
                }
            }
            EdgeDirection::Out => {
                if *edge.src == *node_id {
                    Some((*edge.dst).clone())
                } else {
                    None
                }
            }
            EdgeDirection::Both => {
                if *edge.src == *node_id {
                    Some((*edge.dst).clone())
                } else if *edge.dst == *node_id {
                    Some((*edge.src).clone())
                } else {
                    None
                }
            }
        })
        .collect();

    Ok(neighbors)
}

pub async fn get_neighbors_with_edges<S: StorageClient>(
    storage: &Arc<Mutex<S>>,
    node_id: &Value,
    edge_direction: EdgeDirection,
    edge_types: &Option<Vec<String>>,
) -> DBResult<Vec<(Value, Edge)>> {
    let storage_guard = safe_lock(storage)
        .expect("Storage lock should not be poisoned");

    let edges = storage_guard
        .get_node_edges(node_id, EdgeDirection::Both)
        .map_err(|e| DBError::Storage(StorageError::DbError(e.to_string())))?;

    let filtered_edges: Vec<Edge> = if let Some(ref edge_types) = edge_types {
        edges
            .into_iter()
            .filter(|edge| edge_types.contains(&edge.edge_type))
            .collect()
    } else {
        edges
    };

    let neighbors_with_edges: Vec<(Value, Edge)> = filtered_edges
        .into_iter()
        .filter_map(|edge| match edge_direction {
            EdgeDirection::In => {
                if *edge.dst == *node_id {
                    Some(((*edge.src).clone(), edge))
                } else {
                    None
                }
            }
            EdgeDirection::Out => {
                if *edge.src == *node_id {
                    Some(((*edge.dst).clone(), edge))
                } else {
                    None
                }
            }
            EdgeDirection::Both => {
                if *edge.src == *node_id {
                    Some(((*edge.dst).clone(), edge))
                } else if *edge.dst == *node_id {
                    Some(((*edge.src).clone(), edge))
                } else {
                    None
                }
            }
        })
        .collect();

    Ok(neighbors_with_edges)
}
