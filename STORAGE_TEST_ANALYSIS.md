# GraphDB Storage 包集成测试覆盖情况分析

## 执行摘要

目前 storage 包的集成测试覆盖主要集中在 **上层 API 层**（GraphStorage）和部分 **事务层** 的操作，但在以下方面存在明显的测试覆盖缺陷：

- **Property 属性操作** 测试严重不足
- **Edge 边的高级功能** 测试覆盖不完整（如多边、不同CSR策略等）
- **Vertex 顶点的复杂场景** 缺少测试
- **存储编码、压缩层** 几乎无集成测试
- **MVCC、事务隔离** 缺少端到端集成测试

---

## 现有测试统计

### 测试文件清单

| 文件 | 行数 | 主要内容 |
|------|------|--------|
| `engine/graph_storage/tests.rs` | 1,318 | **主要集成测试** - GraphStorage API、Schema、Vertex、Edge、User、Admin 操作 |
| `engine/transaction/ops_test.rs` | 538 | **事务操作测试** - 底层 Vertex/Edge 插入、删除操作 |
| `engine/sync_wrapper/tests.rs` | 86 | **同步层测试** - 指标记录、同步管理 |
| `engine/data_store_test.rs` | 180 | **数据存储测试** - 数据存储的基础操作 |
| **总计** | **2,122** | |

### 测试覆盖的功能范围

#### ✅ **已覆盖的功能**

**Schema 操作（完整覆盖）**
- 创建/列出/删除 Space
- 创建/列出/删除/修改 Tag（Vertex Label）
- 创建/列出/删除/修改 EdgeType
- 按照 Space 隔离 Schema
- Schema 的 WAL 恢复和持久化

**Vertex 基础操作（覆盖 70%）**
- ✅ 插入单个顶点
- ✅ 获取顶点
- ✅ 删除顶点
- ✅ 批量插入顶点（带回滚）
- ✅ 扫描所有顶点
- ✅ 按 Tag 扫描顶点
- ✅ 更新顶点（含索引更新）
- ✅ 顶点属性基本操作

**Edge 基础操作（覆盖 60%）**
- ✅ 插入单条边
- ✅ 获取单条边
- ✅ 删除边
- ✅ 批量插入边（带回滚）
- ✅ 按类型扫描边
- ✅ 获取顶点的出/入边
- ✅ String ID 顶点的边操作

**Index 操作（覆盖 40%）**
- ✅ 创建/删除 Tag 索引
- ✅ 索引查询（lookup_index）
- ✅ 索引与顶点更新的同步

**User/Auth 操作（覆盖 50%）**
- ✅ 创建/删除用户
- ✅ 授予/回收角色
- ✅ 用户数据持久化和恢复

**Persistence 操作（覆盖 60%）**
- ✅ 快照创建/验证/清理
- ✅ Schema 的 WAL 恢复
- ✅ Flush 操作
- ✅ Checkpoint 创建

**Admin 操作（覆盖 50%）**
- ✅ 存储统计
- ✅ 数据库路径获取

#### ❌ **缺失或不足的测试**

**Property 属性操作（覆盖 < 10%）**
- ❌ Property 更新操作（单独修改某个属性）
- ❌ Property 删除操作
- ❌ 多类型属性值的完整测试
- ❌ Property 索引创建和查询
- ❌ Property 溢出处理（>256字节）
- ❌ Property 编码选择和压缩
- ❌ Property 的事务隔离和并发性
- ❌ Property 值的类型转换

**Edge 高级功能（覆盖 < 30%）**
- ❌ **多边功能** - 同一 src/dst 对的多条边（不同 rank）
- ❌ **自环边** - 顶点指向自身的边
- ❌ **并发边插入** - 多个线程同时插入
- ❌ **大量边操作** - 单顶点数千条边的性能测试
- ❌ Edge 属性更新（不删除不新增）
- ❌ Edge 属性的事务隔离
- ❌ 边的 CSR 压缩策略（Single vs Multiple）
- ❌ EdgeOffset 溢出处理
- ❌ 大批量边删除

**Vertex 高级功能（覆盖 < 40%）**
- ❌ **多标签顶点** - 单个顶点多个 Tag
- ❌ **顶点 Label 修改** - 添加/移除 Label
- ❌ **顶点大属性** - 超大属性值的处理
- ❌ **顶点并发操作** - 并发插入/更新
- ❌ **顶点 MVCC 可见性** - 事务隔离下的读可见性
- ❌ Vertex 的列式存储压缩效果
- ❌ 顶点的增量更新
- ❌ 按属性范围扫描顶点（如 age > 25）

**编码和压缩（覆盖 0%）**
- ❌ Dictionary 编码效果测试
- ❌ RLE（Run-Length Encoding）效果
- ❌ BitPacking 编码
- ❌ FSST 字符串压缩
- ❌ ALP 浮点压缩
- ❌ CompressionSelector 的自动选择
- ❌ 压缩比验证
- ❌ 压缩后的查询正确性

**MVCC 和事务隔离（覆盖 < 20%）**
- ❌ 并发读写的隔离级别验证
- ❌ 脏读防护
- ❌ 写冲突检测
- ❌ 版本链的长度和垃圾回收
- ❌ MVCC 时间戳的正确性
- ❌ 快照隔离
- ❌ 长事务下的版本可见性

**索引高级功能（覆盖 < 20%）**
- ❌ 多字段索引
- ❌ 唯一索引约束
- ❌ 部分索引（Partial Index）
- ❌ 索引的并发更新
- ❌ 索引的增量更新
- ❌ 索引范围查询
- ❌ 索引的持久化和恢复
- ❌ 索引的空间占用

**存储持久化（覆盖 < 40%）**
- ❌ WAL 的对数增长和 compaction
- ❌ Checkpoint 的增量性
- ❌ 多个 Checkpoint 的管理
- ❌ 持久化的原子性保证
- ❌ 磁盘故障恢复场景
- ❌ 大规模数据的持久化
- ❌ 存储空间的重用（回收）

**缓存和性能（覆盖 < 10%）**
- ❌ RecordCache 的有效性
- ❌ 缓存命中率
- ❌ 缓存驱逐策略
- ❌ 缓存容量限制

**容错和异常（覆盖 < 50%）**
- ❌ 内存不足场景
- ❌ 磁盘空间不足
- ❌ 并发冲突异常处理
- ❌ 损坏数据恢复
- ❌ 部分失败的回滚

---

## 功能调用链分析

### Vertex 操作的完整调用链

```
insert_vertex(space, vertex)
  ├─ validate_space_and_schema
  ├─ get_vertex_table_mut(space_id, tag_id)
  ├─ VertexTable::insert_by_i64() 或 insert()
  │  ├─ id_indexer::add() - 外部ID到内部ID映射
  │  ├─ column_store::add_row() - 列式存储每个属性
  │  │  ├─ Column::add()
  │  │  │  ├─ encoder::encode() - 自动选择编码
  │  │  │  ├─ overflow_buffer::write() - 处理大值
  │  │  │  └─ row_group::add() - 行组管理
  │  │  └─ 返回内部行ID
  │  ├─ vertex_timestamp::set_timestamp() - 设置MVCC时间戳
  │  └─ notify_indices() - 索引更新
  ├─ TransactionOps::add_vertex()
  ├─ WAL::append() - 写入日志
  └─ return external_vertex_id
```

**缺失的链路测试**：
- ❌ encoder 的自动选择验证
- ❌ column_store 的压缩效果
- ❌ overflow_buffer 的大值处理
- ❌ row_group 的跨越边界场景
- ❌ 并发插入时的 timestamp 分配

### Edge 操作的完整调用链

```
insert_edge(space, edge)
  ├─ validate_space_and_schema
  ├─ resolve src/dst 顶点的内部ID
  ├─ get_edge_table_mut(space_id, src_tag, dst_tag, edge_type)
  ├─ EdgeTable::insert()
  │  ├─ CSR操作（outgoing edges）
  │  │  ├─ MutableCsr::add() 或 SingleMutableCsr::add()
  │  │  │  └─ 根据edge count自动选择策略
  │  │  └─ offset处理和CSR段管理
  │  ├─ CSR操作（incoming edges）
  │  │  └─ 同上
  │  ├─ PropertyTable::add_row()
  │  │  ├─ encoder::encode() - 边属性编码
  │  │  └─ Column::add() - 列存
  │  └─ notify_indices() - 索引更新
  ├─ TransactionOps::add_edge()
  ├─ WAL::append()
  └─ return edge_id
```

**缺失的链路测试**：
- ❌ CSR 策略自动切换（Single → Multiple）
- ❌ offset 溢出处理
- ❌ 多边（rank != 0）的完整生命周期
- ❌ 自环边的 CSR 处理
- ❌ Edge 属性编码压缩

### Property 操作的完整调用链

```
Vertex Properties:
  insert_vertex() 
    → column_store::add_row()
      → Column::add(property_bytes)
        ├─ CompressionSelector::select() - 选择编码
        ├─ encoder.encode()
        │  ├─ Dictionary - 字典编码
        │  ├─ RLE - 行程长度编码
        │  ├─ BitPacking - 位封装
        │  ├─ FSST - 字符串压缩
        │  └─ ALP - 浮点压缩
        └─ ColumnData::append()

Edge Properties:
  insert_edge()
    → property_table::add_row()
      → Column::add(property_bytes)  [同上]
      → 处理属性溢出（>256字节）

Property Update:
  update_vertex/edge
    → column_store::update_row() / property_table::update_row()
      → 重新编码 + 版本链管理
```

**缺失的链路测试**：
- ❌ 各编码策略的编解码正确性
- ❌ CompressionSelector 的决策逻辑
- ❌ 属性值类型转换
- ❌ 溢出缓冲区的写入/读取
- ❌ 压缩效率的实际测量

---

## 关键测试覆盖缺陷细化

### 1. **Property 属性操作 - 严重缺陷** ⚠️⚠️⚠️

**现状**：
- 仅在 Vertex/Edge 插入时作为辅助测试
- 无独立的属性修改/删除测试
- 无属性编码效果验证

**应添加的测试**：

```rust
#[test]
fn test_property_update_single_field() {
    // 更新顶点的单个属性，其他属性不变
    // 验证列式存储的增量更新
}

#[test]
fn test_property_delete_attribute() {
    // 删除顶点/边的某个属性
    // 验证null值表示
}

#[test]
fn test_property_overflow_buffer_handling() {
    // 属性值 > 256字节
    // 验证溢出缓冲区的写入和检索
}

#[test]
fn test_edge_property_update_preserves_adjacency() {
    // 更新边属性不应影响CSR邻接信息
}

#[test]
fn test_property_encoding_compression_correctness() {
    // 验证Dictionary/RLE/BitPacking/FSST/ALP的编解码正确性
}

#[test]
fn test_compression_selector_chooses_optimal_encoding() {
    // 验证CompressionSelector的决策合理性
}

#[test]
fn test_property_batch_update_atomicity() {
    // 批量更新多个顶点的属性
    // 验证原子性或失败回滚
}

#[test]
fn test_property_index_update_on_change() {
    // 更新有索引的属性
    // 验证索引也被更新
}
```

### 2. **Edge 高级功能 - 覆盖不足** ⚠️⚠️

**现状**：
- 仅测试单边（rank=0）
- 未测试多边（同一src/dst的多条边）
- 未测试自环

**应添加的测试**：

```rust
#[test]
fn test_multi_edges_same_src_dst_different_rank() {
    // 创建(1,2,"KNOWS",rank=0), (1,2,"KNOWS",rank=1), ...
    // 验证get_node_edges包含所有rank
    // 验证CSR策略从SingleMutableCsr切换到MutableCsr
}

#[test]
fn test_self_loop_edge() {
    // 创建顶点1到自身的边
    // 验证出/入边都包含该边
    // 验证CSR的自环处理
}

#[test]
fn test_edge_csr_strategy_switch_at_threshold() {
    // 逐步添加边，观察CSR策略何时从Single切换到Multiple
}

#[test]
fn test_edge_property_update_without_src_dst_change() {
    // 更新边属性（如weight），不修改src/dst
    // 验证CSR邻接不变，PropertyTable更新
}

#[test]
fn test_massive_edges_single_vertex() {
    // 单个顶点有千级边
    // 验证CSR的内存效率和性能
}

#[test]
fn test_delete_middle_edge_in_adjacency() {
    // 顶点有[e1, e2, e3]，删除e2
    // 验证CSR重建的正确性
}

#[test]
fn test_edge_traversal_consistency_after_concurrent_adds() {
    // 并发添加边，验证遍历的一致性
}

#[test]
fn test_edge_offset_overflow_handling() {
    // 创建大量边使offset接近溢出
    // 验证处理是否正确或是否有限制
}
```

### 3. **Vertex 多标签和并发 - 缺失** ⚠️

**现状**：
- 仅单标签顶点测试
- 无并发插入的隔离性测试

**应添加的测试**：

```rust
#[test]
fn test_multi_tag_vertex() {
    // 创建同时有"Person"和"Employee"标签的顶点
    // 验证属性来自两个tag的schema
    // 验证更新时的一致性
}

#[test]
fn test_concurrent_vertex_inserts_isolation() {
    // 多线程并发插入
    // 验证MVCC时间戳的递增分配
    // 验证各线程间的可见性
}

#[test]
fn test_vertex_mvcc_timestamp_visibility() {
    // T1: 插入顶点v1 at ts=100
    // T2: 读取v1 at ts=90 (应不可见)
    // T3: 读取v1 at ts=100 (应可见)
}

#[test]
fn test_vertex_column_compression_effectiveness() {
    // 插入相同属性值的多个顶点
    // 验证Dictionary编码的压缩率
}

#[test]
fn test_vertex_large_property_storage() {
    // 属性值为1MB字符串
    // 验证overflow buffer的处理
}
```

### 4. **编码和压缩 - 完全缺失** ⚠️⚠️⚠️

**现状**：
- 0% 覆盖
- 无单元测试集成到存储层

**应添加的测试**：

```rust
#[test]
fn test_dictionary_encoding_low_cardinality() {
    // 100个顶点，属性值仅5种
    // 验证Dictionary编码能压缩至字典+索引
}

#[test]
fn test_rle_encoding_sequential_duplicates() {
    // 1000个顶点，属性值为(A,A,A,B,B,B,C,C,...)
    // 验证RLE的压缩比
}

#[test]
fn test_bitpacking_integer_range() {
    // 顶点ID在[0, 1000)范围
    // 验证BitPacking能用10位表示
}

#[test]
fn test_fsst_string_compression() {
    // 边属性为长字符串集合
    // 验证FSST压缩效果
}

#[test]
fn test_alp_float_compression() {
    // Float64属性接近某个基数
    // 验证ALP压缩
}

#[test]
fn test_compression_selector_automatic_strategy() {
    // 插入属性，让selector自动选择最优编码
    // 验证选择的合理性
}

#[test]
fn test_encoded_data_decompression_accuracy() {
    // 压缩后的数据读取应完全还原
}

#[test]
fn test_mixed_encoding_in_same_column() {
    // 某列的行组采用不同编码
    // 验证读取的正确性
}
```

### 5. **MVCC 和事务隔离 - 严重不足** ⚠️⚠️⚠️

**现状**：
- 仅在恢复测试中涉及timestamp
- 无隔离级别验证
- 无并发冲突测试

**应添加的测试**：

```rust
#[test]
fn test_serializable_isolation() {
    // T1 @ ts=100: Insert v1
    // T2 @ ts=101: Read v1 (应可见)
    // T3 @ ts=99:  Read v1 (应不可见)
}

#[test]
fn test_dirty_read_prevention() {
    // T1: Insert v1 (not committed)
    // T2: Try to read v1 (should not see)
}

#[test]
fn test_write_conflict_detection() {
    // T1 @ ts=100: Update v1
    // T2 @ ts=100: Update v1 (should conflict)
}

#[test]
fn test_version_chain_garbage_collection() {
    // 反复更新顶点100次
    // 验证旧版本被垃圾回收
    // 验证最新版本可读
}

#[test]
fn test_concurrent_read_write_stability() {
    // 多线程读写
    // 验证快照一致性
}

#[test]
fn test_mvcc_timestamp_monotonic_increase() {
    // 多个事务的timestamp应严格递增
}

#[test]
fn test_long_running_transaction_visibility() {
    // T1: Start at ts=100, read v1
    // T2 @ ts=101: Update v1
    // T1 @ ts=100: Read v1 again (should still see old version)
}
```

### 6. **索引高级功能 - 覆盖不足** ⚠️

**现状**：
- 仅单字段索引
- 无唯一索引约束
- 无范围查询

**应添加的测试**：

```rust
#[test]
fn test_multi_field_composite_index() {
    // 创建(Person.name, Person.age)的复合索引
    // 验证查询(name="Alice", age=30)的效率
}

#[test]
fn test_unique_index_constraint() {
    // 创建唯一索引on Person.email
    // 插入duplicate应失败
}

#[test]
fn test_partial_index_with_condition() {
    // 创建where age > 18的部分索引
    // 验证age<=18的值不在索引中
}

#[test]
fn test_index_range_query() {
    // 查询age in [25, 35)
    // 验证索引加速
}

#[test]
fn test_concurrent_index_update() {
    // 多线程并发更新被索引的属性
    // 验证索引一致性
}

#[test]
fn test_index_persistence_and_recovery() {
    // 创建索引，flush，restart
    // 验证索引恢复
}
```

### 7. **持久化和恢复 - 覆盖不足** ⚠️

**现状**：
- 基础WAL恢复有测试
- 无大规模数据恢复
- 无增量checkpoint

**应添加的测试**：

```rust
#[test]
fn test_wal_compaction_and_cleanup() {
    // 插入1000个顶点/边
    // 触发checkpoint
    // 验证WAL被压缩/清理
}

#[test]
fn test_incremental_checkpoint() {
    // 创建checkpoint1，修改少量数据，创建checkpoint2
    // 验证checkpoint2的增量性（不是完整copy）
}

#[test]
fn test_multiple_checkpoints_management() {
    // 创建多个checkpoint
    // 验证可选择从任一checkpoint恢复
}

#[test]
fn test_recovery_from_corrupted_checkpoint() {
    // checkpoint损坏，验证能否从WAL恢复
}

#[test]
fn test_large_scale_persistence() {
    // 100万顶点+1000万边
    // 验证持久化和恢复的正确性
}

#[test]
fn test_atomic_checkpoint_creation() {
    // checkpoint创建中断
    // 验证不会留下半成品状态
}
```

### 8. **容错和边界 - 几乎缺失** ⚠️⚠️

**现状**：
- 无内存不足、磁盘不足的模拟
- 无部分失败的回滚

**应添加的测试**：

```rust
#[test]
fn test_out_of_memory_handling() {
    // 模拟内存分配失败
    // 验证graceful处理或panic
}

#[test]
fn test_disk_space_exhaustion() {
    // flush时磁盘满
    // 验证错误处理
}

#[test]
fn test_batch_operation_partial_failure_rollback() {
    // batch_insert_vertices: v1成功，v2失败
    // 验证v1也被回滚
}

#[test]
fn test_corrupted_storage_recovery() {
    // 修改storage文件的某个字节
    // 验证能否检测和恢复
}

#[test]
fn test_offset_id_overflow() {
    // 创建2^32个顶点/边
    // 验证是否有溢出检测或自动扩展
}
```

---

## 测试改进优先级和实施建议

### 优先级 P0（必须）- 基础功能完整性

| # | 功能 | 建议测试数 | 预期工作量 |
|----|------|--------|--------|
| 1 | Property 属性的增删改 | 8 | 高 |
| 2 | Edge 多边和自环 | 7 | 中 |
| 3 | MVCC 隔离性 | 7 | 高 |
| 4 | 编码压缩的端到端 | 8 | 高 |

### 优先级 P1（重要）- 正确性和性能验证

| # | 功能 | 建议测试数 | 预期工作量 |
|----|------|--------|--------|
| 5 | Vertex 多标签和并发 | 5 | 中 |
| 6 | 索引高级功能 | 6 | 中 |
| 7 | 持久化 compaction | 6 | 中 |

### 优先级 P2（改进）- 容错和边界

| # | 功能 | 建议测试数 | 预期工作量 |
|----|------|--------|--------|
| 8 | 缓存有效性 | 4 | 低 |
| 9 | 容错和恢复 | 6 | 高 |

---

## 实施建议

### 1. **建立分层测试体系**

```
tests/storage/
  ├── property_operations/      # Property增删改、编码压缩
  │   ├── update_test.rs
  │   ├── encoding_test.rs
  │   └── compression_test.rs
  ├── edge_advanced/           # 多边、自环、并发
  │   ├── multi_edge_test.rs
  │   ├── self_loop_test.rs
  │   └── concurrent_test.rs
  ├── mvcc_isolation/          # MVCC和事务隔离
  │   ├── isolation_test.rs
  │   ├── version_test.rs
  │   └── timestamp_test.rs
  ├── index_advanced/          # 索引高级功能
  │   ├── composite_index_test.rs
  │   ├── unique_constraint_test.rs
  │   └── range_query_test.rs
  ├── persistence/             # 持久化和恢复
  │   ├── compaction_test.rs
  │   ├── recovery_large_scale_test.rs
  │   └── atomic_test.rs
  └── fault_tolerance/         # 容错
      ├── out_of_memory_test.rs
      └── corruption_recovery_test.rs
```

### 2. **增强现有测试框架**

现有 `common/mod.rs` 中的辅助函数应扩展：

```rust
// 现有
fn create_test_storage() -> GraphStorage
fn setup_space(storage: &mut GraphStorage) -> u64
fn setup_person_tag(storage: &mut GraphStorage) -> u32

// 建议补充
fn setup_multi_tag_space() -> GraphStorage
fn create_high_cardinality_vertices() -> Vec<Vertex>
fn create_edge_with_various_ranks() -> Vec<Edge>
fn setup_index_on_property() -> (GraphStorage, String)
fn simulate_concurrent_operations() -> JoinHandles
fn assert_mvcc_visibility(storage, vertex, at_ts, expected_visible)
fn assert_compression_effective(column_data, compression_ratio_min)
```

### 3. **设计综合集成测试**

用于验证跨功能的端到端场景：

```rust
#[test]
fn end_to_end_social_network_scenario() {
    // 创建Person标签，KNOWS边类型
    // 插入1000个顶点，10000条边
    // 创建(name, age)索引
    // 并发查询和更新
    // 验证属性编码压缩
    // 触发checkpoint和恢复
    // 验证MVCC隔离
}
```

### 4. **性能基准测试**

为以下功能建立基准：

```rust
#[bench]
fn bench_property_encoding_throughput()
fn bench_edge_csr_insertion_rate()
fn bench_index_lookup_performance()
fn bench_mvcc_version_chain_length()
fn bench_compression_ratio_by_type()
```

### 5. **覆盖率目标**

当前整体覆盖率估计为 **45%**，目标：

| 组件 | 当前 | 目标 | 优先级 |
|------|------|------|--------|
| Vertex基础 | 70% | 95% | P0 |
| Vertex高级 | 20% | 80% | P1 |
| Edge基础 | 60% | 95% | P0 |
| Edge高级 | 20% | 85% | P0 |
| Property | 10% | 90% | P0 |
| Index | 40% | 85% | P1 |
| Persistence | 60% | 90% | P1 |
| MVCC | 15% | 80% | P0 |
| Encoding | 0% | 95% | P0 |
| Fault Tolerance | 20% | 70% | P2 |

---

## 总结

GraphDB storage 包现有测试集中在 **API 层面的正确性验证**，对 **内层机制** 的覆盖严重不足。

### 主要缺陷：
1. ❌ Property 属性操作几乎无测试
2. ❌ Edge 多边、自环无覆盖
3. ❌ 编码压缩完全无集成测试
4. ❌ MVCC 隔离性验证严重不足
5. ❌ 并发场景缺失

### 建议：
- **优先完善** Property、Edge高级、MVCC、编码压缩的测试（P0）
- **建立分层测试结构**，区分功能、集成、性能测试
- **增强测试工具库**，支持并发、MVCC可见性、压缩效果验证
- **设计综合场景**，验证跨层功能的协作正确性

预期补充 **50-70 个新的集成测试**，可将覆盖率提升至 **70-75%**。

