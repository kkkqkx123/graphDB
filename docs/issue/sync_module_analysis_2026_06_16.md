# Sync 模块实现与问题分析

本文分析 `crates/graphdb-sync` 当前的实现方式，并检查其在事务、一致性、批处理和向量同步上的逻辑风险。

## 1. 当前实现概览

### 1.1 模块组成

`sync` 模块当前由以下子系统构成：

- `SyncManager`：统一对外入口，负责把存储层写入转换为全文或向量同步请求。
- `SyncCoordinator`：全文同步协调器，负责缓冲、批处理、重试、死信队列。
- `FulltextBatchProcessor`：对单个 `(space, tag, field)` 的全文索引执行批量写入和提交。
- `VectorSyncCoordinator`：向量索引同步协调器，负责向量写入、事务缓冲和提交。
- `TransactionBatchBuffer`：按事务缓存同步操作，支持按序号截断。
- `DeadLetterQueue`：保存重试失败的操作，供后续恢复。
- `CircuitBreaker` / `Retry`：用于远端服务失败保护与重试。

### 1.2 事务同步主流程

当前事务写入链路大致是：

1. 存储层写入成功。
2. `SyncWrapper` 根据当前是否处于事务中，调用 `SyncManager::on_vertex_change_with_txn()`、`on_edge_insert()` 等接口。
3. `SyncManager` 为每个事务递增序号，并把操作转交给 `SyncCoordinator` / `VectorSyncCoordinator` 缓存。
4. `TransactionManager::commit_transaction()` 在事务提交时调用 `sync_manager.commit_transaction_sync(txn_id)`。
5. `SyncCoordinator::commit_transaction()` 将缓冲操作写入全文引擎。
6. `VectorSyncCoordinator::commit_transaction()` 将缓冲向量更新写入向量引擎。

### 1.3 非事务同步主流程

非事务写入则直接走：

1. `SyncWrapper` 识别当前没有事务上下文。
2. 调用 `SyncManager::on_*_direct_sync()`。
3. `SyncManager` 内部通过 `execute_sync()` 阻塞执行异步同步。
4. `SyncCoordinator::on_change()` 或 `VectorSyncCoordinator::on_vector_change()` 立即写入外部索引。

## 2. 现有设计中的问题

### 问题 1：事务提交顺序不安全，存储与同步不是原子提交

**位置：**

- [`crates/graphdb-transaction/src/transaction/manager.rs`](../../crates/graphdb-transaction/src/transaction/manager.rs)
- [`crates/graphdb-sync/src/sync/manager.rs`](../../crates/graphdb-sync/src/sync/manager.rs)

**现象：**

`TransactionManager::commit_transaction()` 先把事务从活动表移除，再把事务状态改为 `Committed`，随后才调用 `sync_manager.commit_transaction_sync(txn_id)`。

**风险：**

- 一旦同步失败，事务在事务层面已经提交，存储层的可见性也已经放开。
- 这时同步层失败无法回滚，最终状态会变成“存储已提交，但索引未提交或只提交了一部分”。
- 这与注释中暗示的 2PC 语义不一致，实际上只是“先提交主事务，再做副作用同步”。

**结论：**

这是一个高风险一致性问题。当前实现无法保证存储与全文/向量索引的原子性。

**建议：**

- 若要维持强一致，应先完成 sync prepare/commit，再把事务标记为 committed。
- 若接受最终一致，也要明确文档边界，并补充失败补偿或重建流程。

---

### 问题 2：`SyncManager::commit_transaction()` 内部不是原子提交，全文与向量之间存在部分成功

**位置：**

- [`crates/graphdb-sync/src/sync/manager.rs`](../../crates/graphdb-sync/src/sync/manager.rs)

**现象：**

`commit_transaction()` 中先提交全文协调器，再提交向量协调器：

- 全文成功后继续向量提交。
- 若向量提交失败，全文部分已经完成，没有回滚逻辑。

**风险：**

- 一个事务内不同索引类型可能出现部分成功、部分失败。
- 对外看是一次事务提交，但内部结果并不一致。

**结论：**

这会破坏“一个事务对应一个一致索引状态”的设计目标。

**建议：**

- 引入真正的准备阶段和提交阶段。
- 或者至少把失败后补偿与重建流程做成显式、可观测的机制。

---

### 问题 3：`FulltextBatchProcessor::execute_batch()` 先清空缓冲再执行写入，失败时会丢数据

**位置：**

- [`crates/graphdb-sync/src/sync/batch/processor.rs`](../../crates/graphdb-sync/src/sync/batch/processor.rs)

**现象：**

`execute_batch()` 先调用 `self.buffer.drain_all(key)`，然后才执行 `delete_batch()`、`index_batch()` 和 `commit()`。

**风险：**

- 一旦后续任一步失败，缓冲中的操作已经被移除。
- 背景任务调用 `commit_timeout()` 时，错误只会被日志记录，不会自动重试，也不会回填缓冲。
- 这意味着全文同步在批处理路径上存在真实的数据丢失风险。

**结论：**

这是一个明确的逻辑缺陷，不只是性能问题。

**建议：**

- 先保留缓冲，确认引擎写入成功后再删除。
- 或者把失败操作重新入队，至少保留重试能力。

---

### 问题 4：`get_or_create_fulltext_processor()` 的创建过程不是原子操作，存在并发重复创建风险

**位置：**

- [`crates/graphdb-sync/src/sync/coordinator/coordinator.rs`](../../crates/graphdb-sync/src/sync/coordinator/coordinator.rs)

**现象：**

该函数先 `get()`，再创建 processor，最后 `insert()`。

**风险：**

- 并发请求同一个 `(space, tag, field)` 时，多个线程可能同时 miss。
- 结果是重复创建多个 `FulltextBatchProcessor`，并可能重复启动后台任务。
- `DashMap::insert()` 只能保证最终 map 中保留一个值，但无法避免中间的重复构造和重复 spawn。

**结论：**

这是一个典型的竞态条件，尤其在高并发写入和索引首次创建时容易触发。

**建议：**

- 改为基于 entry API 的原子初始化。
- 或者对单个索引键引入更明确的初始化锁。

---

### 问题 5：向量事务缓冲 API 的语义与命名不一致，且有潜在错误实现

**位置：**

- [`crates/graphdb-sync/src/sync/vector_sync.rs`](../../crates/graphdb-sync/src/sync/vector_sync.rs)

**现象：**

`VectorTransactionBuffer::take_updates_after_sequence()` 的实现是：

- 先 `retain(|update| update.sequence <= sequence)`。
- 然后返回当前缓冲的 clone。

这实际上是“保留序号不超过指定值的前缀”，而不是“取出 sequence 之后的更新”。

**风险：**

- 如果未来调用者按名字理解为“取出之后的操作”，会得到完全相反的结果。
- 这会直接影响 savepoint rollback 或局部回滚语义。

**结论：**

当前实现至少存在 API 语义错误；即使现在还没被调用，也属于隐患。

**建议：**

- 统一命名和行为。
- 如果该函数表示“truncate 到某个序号”，建议重命名得更明确。

---

### 问题 6：向量索引按 `space` 共用一个物理 collection，隔离粒度过粗

**位置：**

- [`crates/graphdb-sync/src/sync/vector_sync.rs`](../../crates/graphdb-sync/src/sync/vector_sync.rs)

**现象：**

`VectorIndexLocation::to_collection_name()` 只按 `space_id` 生成 collection 名：

- 同一个 space 下不同 tag / field 共享同一个物理 collection。
- 用 `group_id` 做逻辑隔离和搜索过滤。

**风险：**

- 不同字段无法使用不同的向量维度或距离度量。
- `create_vector_index()` 会因为 collection 级配置冲突拒绝创建新索引。
- `on_vertex_deleted()` 只按 `vertex_id` 删除，会把整个 space 内同 ID 的所有向量一起清掉，隔离粒度依赖 payload 约束而不是物理结构。

**结论：**

这是一个明显的架构约束，不一定是 bug，但它会限制模型扩展，并让删除/重建语义变得粗糙。

**建议：**

- 明确这是“space 级共享 collection”策略，并在文档中写清限制。
- 如果后续需要更高隔离度，应改成“tag/field 级 collection”或引入更细的物理分区方案。

---

### 问题 7：清理路径对同步失败是 best-effort，可能留下未清理的同步状态

**位置：**

- [`crates/graphdb-transaction/src/transaction/cleaner.rs`](../../crates/graphdb-transaction/src/transaction/cleaner.rs)

**现象：**

过期事务清理时，如果 `sync_manager.rollback_transaction_sync(txn_id)` 失败，只会记录日志，然后继续把事务清掉。

**风险：**

- 事务已从活动表移除，但同步缓冲可能还残留。
- 这会制造“事务不存在，但 sync 侧还有残留状态”的脏数据。

**结论：**

这在资源清理路径上可以接受为 best-effort，但从一致性角度看并不安全。

**建议：**

- 至少把 rollback 失败计入可观测指标。
- 如果这是关键路径，建议把失败升级为显式告警或恢复任务。

## 3. 总结

当前 `sync` 模块的整体设计方向是清晰的：

- 存储层通过 `SyncWrapper` 统一接入。
- `SyncManager` 负责事务序号和统一调度。
- `SyncCoordinator` 与 `VectorSyncCoordinator` 分别处理全文和向量同步。

但现有实现仍然有几个核心问题：

1. 事务提交不是原子提交。
2. 全文批处理在失败时可能丢失缓冲数据。
3. 并发创建 processor 存在竞态。
4. 向量事务缓冲 API 的语义不清晰，容易误用。
5. 向量索引物理隔离粒度过粗，限制后续扩展。

如果后续要把 sync 作为“事务一致性的一部分”，建议优先修正前 3 项。
