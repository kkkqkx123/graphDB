# Cache 模块缺陷分析与重构方案

基于 `crates/graphdb-storage/src/storage/cache/` 代码审计。

---

## 一、现存功能缺陷

### 1.1 批次操作不校验时间戳且跳过事务快照

**涉及**：`record_cache.rs:425-474`

```rust
// get_vertices_batch 无 query_ts，直接返回缓存的顶点数据
pub fn get_vertices_batch(&self, keys: &[VertexCacheKey]) -> BatchGetResult<CachedVertex>
```

- 无法过滤 `cached_at_ts > query_ts` 的过期条目
- `insert_vertices_batch` 不记录 `transaction_snapshot`，回滚不恢复
- `get_vertices_batch` 不读取 `transaction_snapshot`，但该为读操作可豁免

### 1.2 按 label 失效不记录事务快照

**涉及**：`record_cache.rs:305-317`

```rust
pub fn invalidate_vertices_by_label(&self, label_id: u32) {
    self.vertex_cache.invalidate_entries_if(...)  // Moka 不返回被移除的条目
```

- 事务内调用后回滚无法恢复被失效的条目
- `handle_memory_pressure(Warning)` 同理

### 1.3 `rollback_transaction` 在锁外回放

**涉及**：`record_cache.rs:600-620`

```rust
pub fn rollback_transaction(&self) {
    let snapshot = self.transaction_snapshot.lock().take();  // 释放锁
    for entry in snapshot.into_entries() {
        self.vertex_cache.insert(key, old_value);  // 竞态
```

- 锁释放后回放期间其他线程可并发修改缓存
- 回放可能覆盖并发写入，或被并发写入覆盖

### 1.4 逐出回调死锁风险

**涉及**：`record_cache.rs:135-141`

```rust
.eviction_listener(move |_key, _value, cause| {
    if let Some(ref callback) = *eviction_callback.lock() {  // Moka 内部锁内再上锁
```

- Moka 在持有内部分段锁时调用回调
- 若回调实现试图写回同一缓存 → 死锁

### 1.5 `TransactionCacheSnapshot` 记录全中间态

每次 `insert`/`remove` 都记录 snapshot entry。同一 key 修改 N 次产生 N 条冗余记录。

---

## 二、Moka 0.12 限制导致无法实现的代码路径

以下代码调用了 Moka 不支持的接口，属于**功能性误导**——编译通过但不生效。

### 2.1 动态容量调整

Moka `sync::Cache::builder().max_capacity(N)` 在 `build()` 后不可变。

| 方法 | 实际行为 | 预期行为 |
|------|---------|---------|
| `set_max_memory(usize)` | 只改 `config.max_memory` | Moka 应动态缩小/扩大容量 |
| `set_memory_ratio(u32,u32)` | 只改 `config.memory_ratio` | Moka 应重分配两个子缓存容量 |
| `update_config(RecordCacheConfig)` | 只改 `config` 字段 | Moka 应应用新配置 |
| `handle_memory_pressure(Warning)` | `current_max_memory` 缩小，但 Moka 仍按原始 `max_capacity` 驱逐 | Moka 应动态缩小容量减少内存占用 |

**结论**：`current_max_memory` 与 Moka 实际容量完全脱节。`utilization()`, `max_memory()`, `stats().max_memory` 均不可信。

### 2.2 按 Label 批量失效前枚举条目

`invalidate_vertices_by_label` 需要将操作记录到事务快照，但 Moka 的 `invalidate_entries_if` 不返回被移除的条目列表，也没有 `iter()` 方法可供扫描。

即使 `support_invalidation_closures()` 开启，eviction listener 只收到键值对的**引用**，无法在调用 `invalidate_entries_if` 之前知道哪些条目会被移除。

**结论**：按 label 失效的事务保护不可能正确实现。

### 2.3 智能内存压力驱逐

`handle_memory_pressure(Warning)` 使用计数器保留前 N 条——本质是**随机驱逐**而非 LRU/TinyLFU。因为 Moka 不暴露条目的访问频率/热度和迭代顺序，无法实现"保留热点条目"的智能驱逐。

**结论**：内存压力 Warning 级别的驱逐策略只能退化为随机清除。

### 2.4 有意义的碎片估算

`stats().memory_fragmentation_estimate` 基于 `avg_entry_size / baseline`，但：
- Moka 不提供内部 slot 占用信息
- `weighted_size()` 已经是用户自定义的 weigher 总和，不含 Moka 内部开销
- 无法区分"条目本身大"和"哈希表碎片"

**结论**：碎片估算在 Moka API 层面不可能准确。

---

## 三、需清理的误导性代码

以下代码**不可以修复**，只能删除或标记为静默无操作：

```rust
// 1. set_memory_pressure_config(&mut self)  -- 与 Arc<RecordCache> 不兼容
pub fn set_memory_pressure_config(&mut self, config: MemoryPressureConfig) {
    self.memory_pressure_config = config;  // 直接赋值无锁
}

// 2. set_max_memory / set_memory_ratio / update_config
//    → 只改 config 不对 Moka 生效。建议删除或改为返回 Result 说明不支持运行时变更。

// 3. handle_memory_pressure(Critical) 的 self.current_max_memory.store(0)
//    → clear() 后 Moka 无容量限制，新 insert 会按原始 max_capacity 重新填充

// 4. restore_memory() 的 self.config.lock().max_memory = self.original_max_memory
//    → 同样不对 Moka 实际容量生效

// 5. stats().memory_fragmentation_estimate
//    → 永远不可信，建议移除该字段
```

---

## 四、是否放弃 Moka？

### 4.1 Moka 做对了什么

| 需求 | Moka 支持 |
|------|----------|
| 并发 O(1) get/insert/remove | ✓ `sync::Cache` |
| 权重驱逐（变长条目） | ✓ `weigher` + `max_capacity` |
| TTL/TTI 过期 | ✓ `time_to_live` / `time_to_idle` |
| 高并发读 | ✓ 内部分段锁，读不互斥 |
| TinyLFU 驱逐策略 | ✓ 比 LRU 抗扫描 |

### 4.2 Moka 做不到什么

| 需求 | 是否必需 | 替代方案 |
|------|---------|---------|
| 动态容量调整 | **是**——内存压力响应 | Builder 模式固定，无法运行时变更 |
| 条目迭代 | **是**——按 label 失效前枚举 | Moka 无 `iter()` |
| 失效回调返回条目 | **是**——事务快照 | `invalidate_entries_if` 不提供回传 |
| 安全逐出通知 | 否——通知可延迟 | 改用 channel 避免死锁 |
| 访问频率查询 | 否——不依赖 | 无 |

### 4.3 结论：保持 Moka，清理伪实现

**不建议现在放弃 Moka**。理由：

1. **核心功能正常**：`get_vertex`, `insert_vertex`, `get_id_index`, `insert_id_index` 的单条目操作均正确工作
2. **实现成本高**：从头实现一个并发哈希表 + TinyLFU 驱逐 + TTL/TTI 至少 2-3 周
3. **替代路径清晰**：将当前 Moka 无法实现的功能移出到上层存储层

### 4.4 过渡方案

```
┌─────────────────────────────────────────┐
│               存储层事务                 │
│  (处理 按label失效 + 缓存失效回滚)       │
├─────────────────────────────────────────┤
│              CacheManager               │
│  (封装 RecordCache，添加调用方驱动快照)   │
├─────────────────────────────────────────┤
│              RecordCache                │
│  (纯 KV 缓存：get/insert/remove)         │
│  Moka sync::Cache 做后端                │
└─────────────────────────────────────────┘
```

具体变更：

1. **删除** `transaction_snapshot` 相关代码（`begin_transaction`, `commit_transaction`, `rollback_transaction`, `invalidate_batch` 的快照逻辑）
2. **删除** `set_max_memory`, `set_memory_ratio`, `update_config`, `handle_memory_pressure`, `restore_memory`
3. **删除** `memory_fragmentation_estimate` 相关统计
4. **删除** `set_memory_pressure_config`
5. 将事务级缓存快照**上移到** `CacheManager`，由存储层驱动
6. `CacheManager` 在失效前自行枚举 key（通过 `core_ops` 的 label 查询）并记录快照

Moka 退化为纯 KV 存储，所有"策略性"行为（容量管理、驱逐策略）由上层控制。

### 4.5 未来替换 Moka 的时机

当以下任一条件满足时，应替换 Moka：
- 需要运行时动态调整缓存容量以响操作系统内存压力
- 需要按 label 批量迭代缓存条目
- 需要将缓存与底层存储的 CSR 格式共享内存（mmap-backed cache）

届时建议实现一个**轻量 LRU** + 分段锁的简单缓存，而不是复制 TinyLFU：
- 对于图数据库的点查场景，LRU 已足够
- 可支持动态容量和迭代
- 实现量约 500-800 行
