use std::collections::HashSet;
use std::sync::Arc;

use crate::core::error::{DBError, DBResult, StorageError};
use crate::core::{Edge, Value};
use crate::query::executor::base::EdgeDirection;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// Obtaining neighbor nodes
///
/// Self-loop edges (A->A) can lead to an inflated result or an infinite loop during graph traversal.
/// By default, this function removes duplicates from self-loop edges by tracking the combinations of edge types and rankings that have already been processed.
/// Ensure that self-loop edges of the same type and ranking are only returned once.
///
/// # Parameters
/// “storage”: The storage component on the client side.
/// `node_id`: The ID of the current node.
/// `edge_direction`: Direction of the edge
/// `edge_types`: Filter by edge type
/// `allow_loop`: Whether self-loop edges are allowed (default is `false`, which means duplicate self-loop edges are removed).
///
/// # Return
/// List of neighbor nodes
///
/// # Example
/// ```
/// let neighbors = get_neighbors(
///     &storage,
///     &node_id,
///     EdgeDirection::Out,
///     &Some(vec!["follow".to_string()]),
///     false,
/// )?;
/// ```
pub fn get_neighbors<S: StorageClient>(
    storage: &Arc<Mutex<S>>,
    node_id: &Value,
    edge_direction: EdgeDirection,
    edge_types: &Option<Vec<String>>,
    allow_loop: bool,
) -> DBResult<Vec<Value>> {
    let storage_guard = storage.lock();

    let edges = storage_guard
        .get_node_edges("default", node_id, EdgeDirection::Both)
        .map_err(|e| DBError::Storage(StorageError::DbError(e.to_string())))?;

    let filtered_edges: Vec<Edge> = if let Some(ref edge_types) = edge_types {
        edges
            .into_iter()
            .filter(|edge| edge_types.contains(&edge.edge_type))
            .collect()
    } else {
        edges
    };

    // Remove duplicates from self-loop edges: Use (edge_type, ranking) as the key.
    let mut seen_self_loops: HashSet<(String, i64)> = HashSet::new();

    let neighbors: Vec<Value> = filtered_edges
        .into_iter()
        .filter_map(|edge| {
            // Check whether it is a self-loop edge.
            let is_self_loop = *edge.src == *edge.dst;

            // If self-loop edges are not allowed, then deduplication should be performed.
            if is_self_loop && !allow_loop {
                let key = (edge.edge_type.clone(), edge.ranking);
                if !seen_self_loops.insert(key) {
                    return None; // Duplicate self-loop edges should be skipped.
                }
            }

            match edge_direction {
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
            }
        })
        .collect();

    Ok(neighbors)
}

/// Obtaining neighbor nodes and edges
///
/// By default, duplicate self-loop edges are removed to ensure that self-loop edges of the same type and ranking are only returned once.
///
/// # 参数
/// - `storage`: 存储客户端
/// - `node_id`: 当前节点ID
/// - `edge_direction`: 边方向
/// - `edge_types`: 边类型过滤
/// - `allow_loop`: 是否允许自环边（默认false，即去重自环边）
///
/// # 返回
/// List of tuples representing neighbor nodes and edges
pub fn get_neighbors_with_edges<S: StorageClient>(
    storage: &Arc<Mutex<S>>,
    node_id: &Value,
    edge_direction: EdgeDirection,
    edge_types: &Option<Vec<String>>,
    allow_loop: bool,
) -> DBResult<Vec<(Value, Edge)>> {
    let storage_guard = storage.lock();

    let edges = storage_guard
        .get_node_edges("default", node_id, EdgeDirection::Both)
        .map_err(|e| DBError::Storage(StorageError::DbError(e.to_string())))?;

    let filtered_edges: Vec<Edge> = if let Some(ref edge_types) = edge_types {
        edges
            .into_iter()
            .filter(|edge| edge_types.contains(&edge.edge_type))
            .collect()
    } else {
        edges
    };

    // Remove duplicates from the self-loop edges.
    let mut seen_self_loops: HashSet<(String, i64)> = HashSet::new();

    let neighbors_with_edges: Vec<(Value, Edge)> = filtered_edges
        .into_iter()
        .filter_map(|edge| {
            // Check whether it is a self-loop edge.
            let is_self_loop = *edge.src == *edge.dst;

            // If self-loop edges are not allowed, then deduplication should be performed.
            if is_self_loop && !allow_loop {
                let key = (edge.edge_type.clone(), edge.ranking);
                if !seen_self_loops.insert(key) {
                    return None; // Duplicate self-loop edges should be skipped.
                }
            }

            match edge_direction {
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
            }
        })
        .collect();

    Ok(neighbors_with_edges)
}
