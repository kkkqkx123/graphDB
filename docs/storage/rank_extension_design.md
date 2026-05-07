# Rank 扩展表设计方案

## 1. 背景与动机

### 1.1 Rank 的作用

在图数据库中，**rank** 用于区分同一对顶点间的多条**同类型**边。

**示例场景**：

```
Alice -> Bob (朋友关系, rank=0, 属性: since=2020)
Alice -> Bob (朋友关系, rank=1, 属性: since=2021)  // 同一类型，不同 rank
Alice -> Bob (同事关系, rank=0, 属性: company="Google")  // 不同类型，不需要 rank
```

没有 rank 的话，`(Alice, Bob, 朋友关系)` 只能存储一条边。有了 rank，可以存储多条。

### 1.2 当前实现状态

**CSR 结构**（`src/storage/edge/mod.rs`）：

```rust
pub struct Nbr {
    pub neighbor: VertexId,      // 目标顶点
    pub edge_id: EdgeId,         // 边 ID
    pub prop_offset: u32,        // 属性偏移
    pub timestamp: Timestamp,    // MVCC 时间戳
}
```

**问题**：`Nbr` 结构中**没有 rank 字段**。当前 CSR 使用 `(src, dst)` 作为边的唯一标识，不支持同一对顶点间的多条同类型边。

**Edge 结构**（`src/core/vertex_edge_path.rs`）：

```rust
pub struct Edge {
    pub src: Box<Value>,
    pub dst: Box<Value>,
    pub edge_type: String,
    pub ranking: i64,    // <-- rank 字段存在
    pub id: i64,
    pub props: HashMap<String, Value>,
}
```

### 1.3 设计约束

1. **CSR 性能优势**：CSR 的核心优势是紧凑存储和缓存友好性，添加 rank 会破坏这些优势
2. **可选功能**：大多数场景不需要 rank，不应为不使用的功能付出代价
3. **向后兼容**：现有 API 已保留 rank 参数，需要保持兼容

## 2. 设计方案：Rank 扩展表

### 2.1 核心思想

将 rank 作为**可选扩展功能**，通过独立的扩展表存储，而不是修改 CSR 核心结构。

**优势**：

- **不需要 rank 的场景**：CSR 保持紧凑结构，零额外开销
- **需要 rank 的场景**：通过扩展表查询，付出额外查找代价
- **向后兼容**：现有 API 保持不变，rank=0 时走 CSR 快速路径

### 2.2 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                      Edge Storage Layer                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────┐    ┌──────────────────────────────┐   │
│  │   CSR (主存储)    │    │   Rank Extension Table       │   │
│  │                  │    │   (可选扩展)                  │   │
│  │  - 紧凑存储       │    │                              │   │
│  │  - 缓存友好       │◄──►│  - HashMap<(src,dst,edge),   │   │
│  │  - 快速遍历       │    │              Vec<RankEntry>> │   │
│  │  - 无 rank 开销   │    │                              │   │
│  └──────────────────┘    │  struct RankEntry {          │   │
│                          │    rank: i64,                │   │
│                          │    edge_id: EdgeId,          │   │
│                          │    prop_offset: u32,         │   │
│                          │    timestamp: Timestamp,     │   │
│                          │  }                           │   │
│                          └──────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### 2.3 数据结构

```rust
/// Rank 扩展表条目
#[derive(Debug, Clone)]
pub struct RankEntry {
    pub rank: i64,
    pub edge_id: EdgeId,
    pub prop_offset: u32,
    pub timestamp: Timestamp,
}

/// Rank 扩展表
/// Key: (src_internal_id, dst_internal_id, edge_label_id)
/// Value: Vec<RankEntry> (按 rank 排序)
#[derive(Debug, Default)]
pub struct RankExtensionTable {
    entries: DashMap<(VertexId, VertexId, LabelId), Vec<RankEntry>>,
}

impl RankExtensionTable {
    /// 插入 rank 条目
    pub fn insert(
        &self,
        src: VertexId,
        dst: VertexId,
        edge_label: LabelId,
        rank: i64,
        edge_id: EdgeId,
        prop_offset: u32,
        ts: Timestamp,
    ) {
        let key = (src, dst, edge_label);
        let mut entries = self.entries.entry(key).or_insert_with(Vec::new);

        // 保持按 rank 排序
        let pos = entries.partition_point(|e| e.rank < rank);
        entries.insert(pos, RankEntry {
            rank,
            edge_id,
            prop_offset,
            timestamp: ts,
        });
    }

    /// 获取指定 rank 的条目
    pub fn get(
        &self,
        src: VertexId,
        dst: VertexId,
        edge_label: LabelId,
        rank: i64,
        ts: Timestamp,
    ) -> Option<&RankEntry> {
        let key = (src, dst, edge_label);
        self.entries
            .get(&key)
            .and_then(|entries| {
                entries.iter().find(|e| e.rank == rank && e.timestamp <= ts)
            })
    }

    /// 获取所有 rank 条目
    pub fn get_all(
        &self,
        src: VertexId,
        dst: VertexId,
        edge_label: LabelId,
        ts: Timestamp,
    ) -> Vec<&RankEntry> {
        let key = (src, dst, edge_label);
        self.entries
            .get(&key)
            .map(|entries| {
                entries.iter().filter(|e| e.timestamp <= ts).collect()
            })
            .unwrap_or_default()
    }

    /// 删除指定 rank 的条目
    pub fn remove(
        &self,
        src: VertexId,
        dst: VertexId,
        edge_label: LabelId,
        rank: i64,
    ) -> bool {
        let key = (src, dst, edge_label);
        if let Some(mut entries) = self.entries.get_mut(&key) {
            let len = entries.len();
            entries.retain(|e| e.rank != rank);
            return entries.len() < len;
        }
        false
    }

    /// 删除顶点相关的所有 rank 条目
    pub fn remove_vertex(&self, vertex: VertexId) {
        self.entries.retain(|key, _| {
            let (src, dst, _) = *key;
            src != vertex && dst != vertex
        });
    }
}
```

### 2.4 边操作逻辑

#### 2.4.1 插入边

```rust
fn insert_edge(
    &mut self,
    src: VertexId,
    dst: VertexId,
    edge_label: LabelId,
    rank: i64,
    properties: &[(String, Value)],
    ts: Timestamp,
) -> StorageResult<EdgeId> {
    if rank == 0 {
        // 快速路径：使用 CSR 存储
        self.csr.insert_edge(src, dst, properties, ts)
    } else {
        // 扩展路径：使用 Rank 扩展表
        let edge_id = self.allocate_edge_id();
        let prop_offset = self.property_table.insert(properties, ts)?;

        self.rank_table.insert(
            src, dst, edge_label, rank,
            edge_id, prop_offset, ts,
        );

        Ok(edge_id)
    }
}
```

#### 2.4.2 获取边

```rust
fn get_edge(
    &self,
    src: VertexId,
    dst: VertexId,
    edge_label: LabelId,
    rank: i64,
    ts: Timestamp,
) -> Option<EdgeRecord> {
    if rank == 0 {
        // 快速路径：从 CSR 获取
        self.csr.get_edge(src, dst, ts)
    } else {
        // 扩展路径：从 Rank 扩展表获取
        self.rank_table.get(src, dst, edge_label, rank, ts)
            .map(|entry| EdgeRecord {
                edge_id: entry.edge_id,
                src_vid: src,
                dst_vid: dst,
                properties: self.property_table.get(entry.prop_offset, ts),
            })
    }
}
```

#### 2.4.3 获取顶点的所有边

```rust
fn get_node_edges(
    &self,
    vertex: VertexId,
    direction: EdgeDirection,
    ts: Timestamp,
) -> Vec<EdgeRecord> {
    let mut edges = Vec::new();

    // 1. 从 CSR 获取 rank=0 的边
    edges.extend(self.csr.edges_of(vertex, direction, ts));

    // 2. 从 Rank 扩展表获取 rank!=0 的边
    // 需要扫描所有包含该顶点的 rank 条目
    for (key, entries) in self.rank_table.iter() {
        let (src, dst, edge_label) = *key;
        if direction == EdgeDirection::Out && src == vertex {
            for entry in entries.iter().filter(|e| e.timestamp <= ts) {
                edges.push(EdgeRecord {
                    edge_id: entry.edge_id,
                    src_vid: src,
                    dst_vid: dst,
                    properties: self.property_table.get(entry.prop_offset, ts),
                });
            }
        } else if direction == EdgeDirection::In && dst == vertex {
            for entry in entries.iter().filter(|e| e.timestamp <= ts) {
                edges.push(EdgeRecord {
                    edge_id: entry.edge_id,
                    src_vid: src,
                    dst_vid: dst,
                    properties: self.property_table.get(entry.prop_offset, ts),
                });
            }
        }
    }

    edges
}
```

### 2.5 Schema 级别支持

在 EdgeTypeInfo 中添加 rank 支持标记：

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode)]
pub struct EdgeTypeInfo {
    pub edge_type_id: i32,
    pub edge_type_name: String,
    pub properties: Vec<PropertyDef>,
    pub comment: Option<String>,
    pub ttl_duration: Option<i64>,
    pub ttl_col: Option<String>,
    pub rank_enabled: bool,  // 新增：是否启用 rank
}
```

创建边类型时指定是否启用 rank：

```sql
-- 不启用 rank（默认）
CREATE EDGE friend (since INT)

-- 启用 rank
CREATE EDGE friend (since INT) WITH RANK
```

## 3. 性能分析

### 3.1 不需要 rank 的场景（rank=0）

| 操作     | CSR 性能      | 扩展表方案性能 | 差异   |
| -------- | ------------- | -------------- | ------ |
| 插入边   | O(1)          | O(1)           | 无差异 |
| 获取边   | O(log N)      | O(log N)       | 无差异 |
| 遍历边   | O(1) 缓存友好 | O(1) 缓存友好  | 无差异 |
| 内存占用 | 紧凑          | 紧凑           | 无差异 |

### 3.2 需要 rank 的场景（rank!=0）

| 操作     | 修改 CSR 方案       | 扩展表方案         | 差异       |
| -------- | ------------------- | ------------------ | ---------- |
| 插入边   | O(log N)            | O(log N)           | 相似       |
| 获取边   | O(log N)            | O(log N)           | 相似       |
| 遍历边   | O(N) 缓存友好性下降 | O(N) 额外查找      | 扩展表略慢 |
| 内存占用 | 每条边 +8 字节      | 仅 rank 边额外存储 | 扩展表更优 |

### 3.3 内存对比

**修改 CSR 方案**（每条边增加 8 字节 rank 字段）：

```
1 亿条边 × 8 字节 = 800 MB 额外内存
```

**扩展表方案**（仅 rank 边额外存储）：

```
假设 1% 的边使用 rank：
100 万条边 × (8 + 16) 字节 = 24 MB 额外内存
```

## 4. 实现计划

### 4.1 阶段 1：基础结构

- [ ] 创建 `RankExtensionTable` 结构
- [ ] 实现基本的 insert/get/remove 操作
- [ ] 添加单元测试

### 4.2 阶段 2：集成到 EdgeTable

- [ ] 在 `EdgeTable` 中添加 `rank_table` 字段
- [ ] 修改 `insert_edge` 方法支持 rank
- [ ] 修改 `get_edge` 方法支持 rank
- [ ] 修改 `edges_of` 方法合并 CSR 和 rank 结果

### 4.3 阶段 3：集成到 PropertyGraph

- [ ] 修改 `PropertyGraph::insert_edge` 传递 rank
- [ ] 修改 `PropertyGraph::get_edge` 传递 rank
- [ ] 修改 `PropertyGraph::delete_edge` 处理 rank

### 4.4 阶段 4：Schema 支持

- [ ] 在 `EdgeTypeInfo` 中添加 `rank_enabled` 字段
- [ ] 修改 DDL 解析支持 `WITH RANK` 语法
- [ ] 在创建边类型时设置 rank 标记

### 4.5 阶段 5：API 层集成

- [ ] 修改 `StorageClient::insert_edge` 传递 rank
- [ ] 修改 `StorageClient::get_edge` 传递 rank
- [ ] 修改 `StorageClient::delete_edge` 传递 rank
- [ ] 确保 `GraphStorage` 正确委托给 PropertyGraph

### 4.6 阶段 6：持久化

- [ ] 实现 `RankExtensionTable` 的序列化/反序列化
- [ ] 集成到现有的 flush/load 流程
- [ ] 添加 MVCC 支持（时间戳过滤）

## 5. 替代方案对比

### 5.1 方案 A：修改 CSR 结构（不推荐）

**优点**：

- 实现简单，直接添加 rank 字段
- 查询路径统一

**缺点**：

- 破坏 CSR 紧凑存储优势
- 每条边增加 8 字节内存开销
- 缓存友好性下降
- 为不使用的功能付出代价

### 5.2 方案 B：Rank 扩展表（推荐）

**优点**：

- CSR 保持紧凑，零额外开销
- 仅 rank 边付出额外代价
- 向后兼容，rank=0 走快速路径
- 可以独立优化 rank 查询

**缺点**：

- 实现复杂度略高
- rank 查询需要额外查找
- 遍历边时需要合并两个数据源

### 5.3 方案 C：完全独立的存储（不推荐）

将 rank 边完全存储在独立的结构中（如 BTreeMap）。

**优点**：

- 完全隔离，互不影响

**缺点**：

- 查询逻辑复杂
- 内存碎片化
- 遍历性能差

## 6. 总结

**推荐方案 B：Rank 扩展表**

核心理由：

1. **零开销原则**：不使用的功能不应付出代价
2. **向后兼容**：现有 API 和数据结构保持不变
3. **可扩展性**：可以独立优化 rank 存储和查询
4. **性能平衡**：在常见场景（rank=0）保持 CSR 性能优势

实现优先级：

- **低优先级**：大多数场景不需要 rank
- **高价值**：为需要多边的场景提供支持
- **低风险**：不影响现有功能
