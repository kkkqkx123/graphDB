# Vertex & Edge 存储模块分析与修复计划

## 概述

本文档对 `src/storage/vertex` 和 `src/storage/edge` 两个目录的实现进行深入分析，识别现有问题，并制定分阶段修复计划。

---

## 一、Vertex 存储模块 (`src/storage/vertex`)

### 1.1 组件结构

| 组件 | 文件 | 职责 |
|------|------|------|
| `VertexTable` | vertex_table.rs | 顶点表主存储，组合 ID 索引、列存储、MVCC 时间戳 |
| `IdIndexer<K>` | id_indexer.rs | 外部 ID (String) → 内部 ID (u32) 双向映射，FreeList 复用 |
| `ColumnStore` | column_store.rs | 列式属性存储，支持定长/变长类型和可空列 |
| `VertexTimestamp` | vertex_timestamp.rs | MVCC 时间戳追踪（start_ts / end_ts / deleted 向量） |
| encoding/ | encoding/mod.rs | 压缩编码：Dictionary, RLE, BitPacking, Varint, FSST, ALP, Lazy |

### 1.2 核心数据流

```
外部 ID (String) → IdIndexer → 内部 ID (u32) → ColumnStore (属性) + VertexTimestamp (MVCC)
```

### 1.3 使用方式

- `SchemaOps` 持有 `HashMap<LabelId, VertexTable>`，通过 `PropertyGraph` 门面调用
- `TransactionOps` - WAL 回放
- `QueryOps` - 顶点扫描

---

## 二、Edge 存储模块 (`src/storage/edge`)

### 2.1 组件结构

| 组件 | 文件 | 职责 |
|------|------|------|
| `EdgeTable` | edge_table.rs | 边表主存储，组合出/入 CSR、入 CSR、属性表、Edge ID 映射 |
| `MutableCsr` | mutable_csr.rs | 多边 CSR 实现（每个顶点多条边） |
| `SingleMutableCsr` | single_mutable_csr.rs | 单边 CSR（每个顶点最多一条边，O(1) 访问） |
| `MutableCsrVariant` | mutable_csr_variant.rs | 枚举包装器，运行时策略选择 |
| `PropertyTable` | property_table.rs | 边的列式属性存储 + Overflow Store |
| csr_trait.rs | csr_trait.rs | 统一 trait 定义 |
| `Csr` | csr.rs | 不可变 CSR（读优化场景） |

### 2.2 核心数据流

```
(src, dst) → MutableCsrVariant (out/in) → Nbr {neighbor, edge_id, prop_offset, timestamp}
                                                      ↓
                                               PropertyTable (属性)
```

### 2.3 使用方式

- `EdgeOps` 持有 `HashMap<(LabelId, LabelId, LabelId), EdgeTable>`
- 经 `PropertyGraph` 门面调用
- 边迭代器、批操作

---

## 三、现有实现的问题

### 🔴 P1: `MutableCsr` 中的 SpinLock 冗余

**位置**: [mutable_csr.rs](file:///d:/项目/database/graphDB/src/storage/edge/mutable_csr.rs)

**问题描述**:
- `SpinLock`、`SpinLockGuard` 已完整实现，每个顶点预分配一个 SpinLock
- 尽管写方法（`insert_edge`, `delete_edge` 等）使用了 `SpinLockGuard`，但所有 mutable 方法已经拥有 `&mut self`，Rust 的借用检查器已经保证了独占访问
- 读方法（`edges_of`, `has_edge`, `get_edge` 等）**不使用** SpinLock，所以没有提供锁保护
- 上层的 `RwLock<EdgeOps>` 已经提供了并发控制已提供安全保障
- SpinLock 增加了复杂性和内存开销（`vertex_capacity` × 每个 SpinLock），无实际收益

**修复方案**: 移除 `SpinLock`, `SpinLockGuard` 及所有相关调用

---

### 🔴 P2: release 构建中的静默数据不一致

**位置**: [edge_table.rs:L590](file:///d:/项目/database/graphDB/src/storage/edge/edge_table.rs#L590)

**问题描述**:
```rust
debug_assert_eq!(nbr.prop_offset, ie_nbr.prop_offset,
    "out_csr and in_csr should share the same prop_offset");
```
`debug_assert_eq!` 仅在 debug 构建生效。如果 out_csr 和 in_csr 的 `prop_offset` 在 release 构建中分歧，只会更新 out 边属性而跳过 in 边，导致双向边属性不一致。

**修复方案**: 将 `debug_assert_eq!` 改为 `assert_eq!`，确保在所有构建中验证

---

### 🔴 P3: `from_strategy(None)` 引发 panic

**位置**: [mutable_csr_variant.rs:L40](file:///d:/项目/database/graphDB/src/storage/edge/mutable_csr_variant.rs#L40)

**问题描述**:
```rust
EdgeStrategy::None => {
    panic!("Cannot create MutableCsrVariant with EdgeStrategy::None")
}
```
`EdgeStrategy::None` 导致直接崩溃，应返回 `Result` 而不是 panic。

**修复方案**: 将 `from_strategy` 改为返回 `StorageResult<Self>`，错误路径返回 `Err`

---

### 🔴 P4: `flush_id_indexer` 不持久化内部 ID 映射

**位置**: [vertex_table.rs:L499](file:///d:/项目/database/graphDB/src/storage/vertex/vertex_table.rs#L499)

**问题描述**:
- 持久化时仅按顺序写入 key 字符串
- 在 `load_id_indexer` 中重新 `insert` key，可能获得**不同的内部 ID**
- 如果系统中有其他组件（索引、事务日志）引用旧内部 ID，Flush/Load 后将出现数据损坏

**修复方案**: 同时持久化 key→internal_id 的映射关系，load 时精确恢复

---

### 🟠 P5: `active_vertices` 死代码

**位置**: [edge_table.rs:L53](file:///d:/项目/database/graphDB/src/storage/edge/edge_table.rs#L53)

**问题描述**:
- `active_vertices: HashSet<VertexId>` 声明后，在 `insert_edge`、`delete_edge`、`revert_delete_edge` 等方法中有更新，但**没有任何方法读取它**
- 增加了维护成本和内存消耗（同时 persist/load 也有序列化代码）

**修复方案**: 移除 `active_vertices` 及其所有相关代码

---

### 🟠 P6: `revert_remove` 忽略 ts 参数

**位置**: [vertex_timestamp.rs:L45](file:///d:/项目/database/graphDB/src/storage/vertex/vertex_timestamp.rs#L45)

**问题描述**:
```rust
pub fn revert_remove(&mut self, index: u32, _ts: Timestamp) {
```
`_ts` 被忽略，不做 `ts >= end_ts` 的验证。事务回滚时用旧时间戳恢复已在新时间戳删除的顶点，破坏 MVCC 语义。

**修复方案**: 验证 `ts >= end_ts`，不满足条件时不做恢复并返回成功标志

---

### 🟠 P7: `batch_update` 静默忽略错误

**位置**: [vertex_table.rs:L344](file:///d:/项目/database/graphDB/src/storage/vertex/vertex_table.rs#L344)

**问题描述**:
```rust
let _ = self.columns.set_property(...);
```
使用 `let _ =` 丢弃错误返回值。如果属性类型不匹配或列不存在，`batch_update` 仍报告成功。

**修复方案**: 收集错误信息并返回可选的错误集合

---

### 🟠 P8: 重复边检测仅限于 Single 策略

**位置**: [edge_table.rs:L149-L170](file:///d:/项目/database/graphDB/src/storage/edge/edge_table.rs#L149)

**问题描述**:
- `insert_edge` 仅在 `EdgeStrategy::Single` 时检查 `has_edge` 并拒绝重复
- `Multiple` 策略不做校验，允许无限重复的 (src, dst) 边
- 相关方法（如 `get_edge`, `delete_edge`）只返回/删除第一个匹配

**修复方案**: 在 `Multiple` 策略下，如果检测到完全相同的 (src, dst) 边（active），也应返回错误或定义明确的去重策略

---

### 🔵 P9: EdgeTable 不维护顶点引用完整性

**原因**: 属于上层 `EdgeOps` 的职责，当前架构设计如此；EdgeTable 作为底层存储不需要验证顶点存在性。

**处理**: 保留现状，不做修改。

---

### 🔵 P10: `compact` 重建效率

**位置**: [vertex_table.rs:L396-L425](file:///d:/项目/database/graphDB/src/storage/vertex/vertex_table.rs#L396)

**问题描述**:
- `remap_columns` 创建全新 `ColumnStore` 并全量复制数据
- 迭代 `HashMap<u32, u32>` 顺序不确定，大数据集内存消耗巨大

**修复方案**: 采用原地重排策略，减少内存分配

---

### 🔵 P11: FreeList 双重清零

**原因**: `clear_row` 在 delete 和 reuse 时各调用一次，属于轻微冗余，不影响正确性。

**处理**: 记录为潜在优化点，当前不做修改。

---

### 🔵 P12: 缺 Rank 支持

**原因**: `EdgeRecord` 和 `EdgeSchema` 中已有 `ranking` 字段但硬编码为 0，结构化存在但功能未实现。需要在 schema 层面完整支持。

**处理**: 移除未使用的 `ranking` 字段。

---

### 🔵 P13: 缺内部并发控制

**原因**: 当前依赖于外部 `RwLock<SchemaOps>` / `RwLock<EdgeOps>`。`EdgeOps>`，架构设计如此，非当前阶段问题。

**处理**: 保留现状。

---

### 🔵 P14: VertexId 字节长度用 u8 编码

**位置**: [edge_table.rs:L759](file:///d:/项目/database/graphDB/src/storage/edge/edge_table.rs#L759)

**问题描述**:
```rust
file.write_all(&(bytes.len() as u8).to_le_bytes())?;
```
使用 `u8` 存储字节长度，当 `VertexId.as_bytes()` 返回超过 255 字节时截断。

**修复方案**: 使用 u32 代替 u8

---

## 四、分阶段修复计划

### 第一阶段（高优先级）
1. P1: 移除冗余 SpinLock
2. P2: debug_assert_eq → assert_eq
3. P3: from_strategy 返回 Result
4. P4: flush_id_indexer 持久化内部 ID
5. P14: VertexId 字节长度用 u32

### 第二阶段（中优先级）
6. P5: 移除 unused active_vertices
7. P6: revert_remove 验证 ts
8. P7: batch_update 收集错误
9. P8: Multiple 策略重复检测

### 第三阶段（低优先级）
10. P10: compact 原地重排优化
11. P12: 移除未使用的 ranking 字段