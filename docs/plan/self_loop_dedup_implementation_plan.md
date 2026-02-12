# 自环边去重功能实现方案

## 文档信息
- **创建日期**: 2026-02-12
- **功能**: 图遍历中的自环边去重
- **优先级**: 高（数据正确性问题）

---

## 一、问题描述

### 1.1 什么是自环边
自环边（Self-loop Edge）是指起点和终点相同的边，即 A → A。

### 1.2 为什么需要去重
在图遍历中，自环边会导致以下问题：
1. **结果膨胀**: 同一个节点会被重复访问多次
2. **无限循环**: 在路径查询中可能导致死循环
3. **数据错误**: 路径计数、邻居统计等结果不准确

### 1.3 nebula-graph的处理方式
nebula-graph在 `GetNeighborsNode` 中通过 `visitedSelfReflectiveEdges_` 集合实现去重：
```cpp
bool isDuplicatedSelfReflectiveEdge(const folly::StringPiece& key) {
    folly::StringPiece srcID = NebulaKeyUtils::getSrcId(context_->vIdLen(), key);
    folly::StringPiece dstID = NebulaKeyUtils::getDstId(context_->vIdLen(), key);
    if (srcID == dstID) {
        // 自环边去重逻辑
        std::string rank = std::to_string(NebulaKeyUtils::getRank(context_->vIdLen(), key));
        auto edgeType = NebulaKeyUtils::getEdgeType(context_->vIdLen(), key);
        edgeType = edgeType > 0 ? edgeType : -edgeType;
        std::string type = std::to_string(edgeType);
        std::string localKey = type + rank + srcID.str();
        if (!visitedSelfReflectiveEdges_.insert(localKey).second) {
            return true; // 重复的自环边
        }
    }
    return false;
}
```

---

## 二、实现方案

### 方案1：在工具函数层添加去重（推荐）

**修改文件**: `src/query/executor/data_processing/graph_traversal/traversal_utils.rs`

#### 2.1.1 添加新的去重函数

```rust
use std::collections::HashSet;

/// 获取邻居节点，支持自环边去重
/// 
/// # 参数
/// - `storage`: 存储客户端
/// - `node_id`: 当前节点ID
/// - `edge_direction`: 边方向
/// - `edge_types`: 边类型过滤
/// - `dedup_self_loop`: 是否对自环边去重（A->A 的边只返回一次）
/// 
/// # 返回
/// 邻居节点列表
/// 
/// # 示例
/// ```
/// let neighbors = get_neighbors_dedup(
///     &storage,
///     &node_id,
///     EdgeDirection::Out,
///     &Some(vec!["follow".to_string()]),
///     true, // 启用自环边去重
/// )?;
/// ```
pub fn get_neighbors_dedup<S: StorageClient>(
    storage: &Arc<Mutex<S>>,
    node_id: &Value,
    edge_direction: EdgeDirection,
    edge_types: &Option<Vec<String>>,
    dedup_self_loop: bool,
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
            
            if is_self_loop && dedup_self_loop {
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
```

#### 2.1.2 添加带边的去重函数

```rust
/// 获取邻居节点和边，支持自环边去重
/// 
/// # 参数
/// - `storage`: 存储客户端
/// - `node_id`: 当前节点ID
/// - `edge_direction`: 边方向
/// - `edge_types`: 边类型过滤
/// - `dedup_self_loop`: 是否对自环边去重
/// 
/// # 返回
/// (邻居节点, 边) 元组列表
pub fn get_neighbors_with_edges_dedup<S: StorageClient>(
    storage: &Arc<Mutex<S>>,
    node_id: &Value,
    edge_direction: EdgeDirection,
    edge_types: &Option<Vec<String>>,
    dedup_self_loop: bool,
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
            
            if is_self_loop && dedup_self_loop {
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
```

#### 2.1.3 保留原函数作为兼容接口

```rust
/// 获取邻居节点（保留原函数作为兼容接口，不去重）
/// 
/// # 注意
/// 此函数不对自环边去重，如果需要去重，请使用 `get_neighbors_dedup`
pub fn get_neighbors<S: StorageClient>(
    storage: &Arc<Mutex<S>>,
    node_id: &Value,
    edge_direction: EdgeDirection,
    edge_types: &Option<Vec<String>>,
) -> DBResult<Vec<Value>> {
    get_neighbors_dedup(storage, node_id, edge_direction, edge_types, false)
}

/// 获取邻居节点和边（保留原函数作为兼容接口，不去重）
pub fn get_neighbors_with_edges<S: StorageClient>(
    storage: &Arc<Mutex<S>>,
    node_id: &Value,
    edge_direction: EdgeDirection,
    edge_types: &Option<Vec<String>>,
) -> DBResult<Vec<(Value, Edge)>> {
    get_neighbors_with_edges_dedup(storage, node_id, edge_direction, edge_types, false)
}
```

---

### 方案2：在ExpandExecutor中添加去重

**修改文件**: `src/query/executor/data_processing/graph_traversal/expand.rs`

#### 2.2.1 添加去重相关字段

```rust
pub struct ExpandExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    pub edge_direction: EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_depth: Option<usize>,
    pub step_limits: Option<Vec<usize>>,
    pub sample: bool,
    pub sample_limit: Option<usize>,
    input_executor: Option<Box<ExecutorEnum<S>>>,
    pub visited_nodes: HashSet<Value>,
    adjacency_cache: HashMap<Value, Vec<Value>>,
    current_step: usize,
    
    // 新增字段
    /// 是否启用自环边去重
    dedup_self_loop: bool,
    /// 自环边去重集合：key = (node_id, edge_type, ranking)
    self_loop_dedup_set: HashSet<(Value, String, i64)>,
}
```

#### 2.2.2 添加配置方法

```rust
impl<S: StorageClient> ExpandExecutor<S> {
    /// 设置是否启用自环边去重
    /// 
    /// # 示例
    /// ```
    /// let executor = ExpandExecutor::new(...)
    ///     .with_dedup_self_loop(true);
    /// ```
    pub fn with_dedup_self_loop(mut self, enable: bool) -> Self {
        self.dedup_self_loop = enable;
        self
    }
    
    // ... 其他方法
}
```

#### 2.2.3 修改扩展逻辑

```rust
impl<S: StorageClient> ExpandExecutor<S> {
    fn expand_step(&mut self, input_nodes: Vec<Value>) -> Result<Vec<Value>, QueryError> {
        let mut expanded_nodes = Vec::new();

        for node_id in input_nodes {
            if self.visited_nodes.contains(&node_id) {
                continue;
            }
            self.visited_nodes.insert(node_id.clone());

            // 使用新的带去重的邻居获取方法
            let neighbors = self.get_neighbors_with_dedup(&node_id)?;

            // 缓存邻接关系
            self.adjacency_cache
                .insert(node_id.clone(), neighbors.clone());

            for neighbor in neighbors {
                if !self.visited_nodes.contains(&neighbor) {
                    expanded_nodes.push(neighbor);
                }
            }
        }

        Ok(expanded_nodes)
    }

    /// 获取邻居节点（带自环边去重）
    fn get_neighbors_with_dedup(&self, node_id: &Value) -> Result<Vec<Value>, QueryError> {
        let storage = self.base.get_storage().clone();
        
        let edges = safe_lock(&*storage)
            .map_err(|e| QueryError::StorageError(e.to_string()))?
            .get_node_edges("default", node_id, self.edge_direction)
            .map_err(|e| QueryError::StorageError(e.to_string()))?;

        let mut seen_self_loops: HashSet<(String, i64)> = HashSet::new();
        let mut neighbors = Vec::new();

        for edge in edges {
            // 过滤边类型
            if let Some(ref types) = self.edge_types {
                if !types.contains(&edge.edge_type) {
                    continue;
                }
            }

            // 自环边去重检查
            let is_self_loop = *edge.src == *edge.dst;
            if is_self_loop && self.dedup_self_loop {
                let key = (edge.edge_type.clone(), edge.ranking);
                if !seen_self_loops.insert(key) {
                    continue;
                }
            }

            // 根据方向确定邻居
            let neighbor = match self.edge_direction {
                EdgeDirection::Out if *edge.src == *node_id => (*edge.dst).clone(),
                EdgeDirection::In if *edge.dst == *node_id => (*edge.src).clone(),
                EdgeDirection::Both => {
                    if *edge.src == *node_id {
                        (*edge.dst).clone()
                    } else {
                        (*edge.src).clone()
                    }
                }
                _ => continue,
            };

            neighbors.push(neighbor);
        }

        Ok(neighbors)
    }
}
```

---

## 三、方案对比

| 维度 | 方案1（工具函数层） | 方案2（ExpandExecutor） |
|------|-------------------|------------------------|
| **复用性** | 高，所有遍历执行器可用 | 低，仅ExpandExecutor可用 |
| **侵入性** | 低，不修改执行器结构 | 高，需要修改执行器字段和方法 |
| **向后兼容** | 好，保留原函数 | 需要修改构造函数 |
| **实现复杂度** | 低 | 中 |
| **维护成本** | 低 | 中 |

**推荐方案**: 方案1（工具函数层）

---

## 四、实施步骤

### 步骤1：修改 traversal_utils.rs
1. 添加 `get_neighbors_dedup` 函数
2. 添加 `get_neighbors_with_edges_dedup` 函数
3. 修改原函数为兼容接口

### 步骤2：更新使用方
1. 在 `ExpandExecutor` 中使用新的去重函数
2. 在 `AllPathsExecutor` 中使用新的去重函数
3. 在 `ShortestPathExecutor` 中使用新的去重函数

### 步骤3：添加测试
1. 单元测试：测试自环边去重逻辑
2. 集成测试：测试图遍历中的自环边处理

---

## 五、测试用例

### 5.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;
    use crate::core::vertex_edge_path::{Edge, Vertex, Tag};
    use std::collections::HashMap;

    #[test]
    fn test_self_loop_dedup() {
        // 创建测试数据
        let node_a = Value::Int(1);
        
        // 创建两条相同的自环边（相同类型和ranking）
        let edge1 = create_test_edge(1, 1, "self_loop", 100);
        let edge2 = create_test_edge(1, 1, "self_loop", 100); // 重复
        let edge3 = create_test_edge(1, 1, "self_loop", 200); // 不同ranking
        
        // 测试去重
        let mut seen = HashSet::new();
        assert!(should_include_edge(&edge1, &mut seen, true));
        assert!(!should_include_edge(&edge2, &mut seen, true)); // 被去重
        assert!(should_include_edge(&edge3, &mut seen, true)); // 不同ranking，保留
    }

    fn create_test_edge(src: i64, dst: i64, edge_type: &str, ranking: i64) -> Edge {
        Edge::new(
            Value::Int(src),
            Value::Int(dst),
            edge_type.to_string(),
            ranking,
            HashMap::new(),
        )
    }

    fn should_include_edge(edge: &Edge, seen: &mut HashSet<(String, i64)>, dedup: bool) -> bool {
        let is_self_loop = *edge.src == *edge.dst;
        if is_self_loop && dedup {
            let key = (edge.edge_type.clone(), edge.ranking);
            seen.insert(key)
        } else {
            true
        }
    }
}
```

### 5.2 集成测试

```rust
#[test]
fn test_expand_with_self_loop_dedup() {
    // 测试ExpandExecutor的自环边去重功能
    // 1. 创建包含自环边的图
    // 2. 执行扩展查询
    // 3. 验证结果中自环边只出现一次
}

#[test]
fn test_all_paths_with_self_loop() {
    // 测试AllPathsExecutor处理自环边
    // 验证no_loop选项与自环边去重的协同工作
}
```

---

## 六、注意事项

### 6.1 去重键的选择
去重键使用 `(edge_type, ranking)` 组合，因为：
1. 相同类型和ranking的自环边被视为重复
2. 不同ranking的自环边被视为不同的边（保留）
3. 与nebula-graph的实现保持一致

### 6.2 性能考虑
1. 使用 `HashSet` 进行O(1)的去重检查
2. 仅在启用去重时创建 `HashSet`，避免不必要的内存开销
3. 对于无自环边的场景，性能影响极小

### 6.3 向后兼容
1. 保留原函数作为默认不去重的兼容接口
2. 新的去重功能通过新函数或参数启用
3. 现有代码无需修改即可继续工作

---

## 七、相关文档
- `docs/plan/adjacency_architecture_analysis.md` - 架构分析文档
- `docs/adjacency_analysis.md` - 原始邻接分析
- `src/query/executor/data_processing/graph_traversal/traversal_utils.rs` - 工具函数实现
