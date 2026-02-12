use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::core::error::{DBError, DBResult, StorageError};
use crate::core::{Edge, Value};
use crate::query::executor::base::EdgeDirection;
use crate::storage::StorageClient;
use crate::utils::safe_lock;

/// 获取邻居节点
///
/// 自环边（A->A）在图遍历中可能导致结果膨胀或无限循环。
/// 此函数默认对自环边进行去重，通过跟踪已处理的（边类型, ranking）组合
/// 确保相同类型和ranking的自环边只返回一次。
///
/// # 参数
/// - `storage`: 存储客户端
/// - `node_id`: 当前节点ID
/// - `edge_direction`: 边方向
/// - `edge_types`: 边类型过滤
///
/// # 返回
/// 邻居节点列表
///
/// # 示例
/// ```
/// let neighbors = get_neighbors(
///     &storage,
///     &node_id,
///     EdgeDirection::Out,
///     &Some(vec!["follow".to_string()]),
/// )?;
/// ```
pub fn get_neighbors<S: StorageClient>(
    storage: &Arc<Mutex<S>>,
    node_id: &Value,
    edge_direction: EdgeDirection,
    edge_types: &Option<Vec<String>>,
) -> DBResult<Vec<Value>> {
    let storage_guard = safe_lock(storage)
        .expect("Storage lock should not be poisoned");

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

    // 自环边去重：使用 (edge_type, ranking) 作为key
    let mut seen_self_loops: HashSet<(String, i64)> = HashSet::new();

    let neighbors: Vec<Value> = filtered_edges
        .into_iter()
        .filter_map(|edge| {
            // 检查是否是自环边
            let is_self_loop = *edge.src == *edge.dst;

            if is_self_loop {
                let key = (edge.edge_type.clone(), edge.ranking);
                if !seen_self_loops.insert(key) {
                    return None; // 重复的自环边，跳过
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

/// 获取邻居节点和边
///
/// 默认对自环边进行去重，确保相同类型和ranking的自环边只返回一次。
///
/// # 参数
/// - `storage`: 存储客户端
/// - `node_id`: 当前节点ID
/// - `edge_direction`: 边方向
/// - `edge_types`: 边类型过滤
///
/// # 返回
/// (邻居节点, 边) 元组列表
pub fn get_neighbors_with_edges<S: StorageClient>(
    storage: &Arc<Mutex<S>>,
    node_id: &Value,
    edge_direction: EdgeDirection,
    edge_types: &Option<Vec<String>>,
) -> DBResult<Vec<(Value, Edge)>> {
    let storage_guard = safe_lock(storage)
        .expect("Storage lock should not be poisoned");

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

    // 自环边去重
    let mut seen_self_loops: HashSet<(String, i64)> = HashSet::new();

    let neighbors_with_edges: Vec<(Value, Edge)> = filtered_edges
        .into_iter()
        .filter_map(|edge| {
            // 检查是否是自环边
            let is_self_loop = *edge.src == *edge.dst;

            if is_self_loop {
                let key = (edge.edge_type.clone(), edge.ranking);
                if !seen_self_loops.insert(key) {
                    return None; // 重复的自环边，跳过
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
