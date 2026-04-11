# 事务性索引同步重构方案

## 概述

本文档提出 GraphDB 事务性索引同步的重构方案，基于对 PostgreSQL、MongoDB、Neo4j 等数据库的调研结果，设计适合当前项目的实现方案。

## 1. 现状分析

### 1.1 当前架构

```
存储层 (Storage)
    ↓ (数据变更)
on_vertex_change(space_id, tag_name, vertex_id, properties, change_type)
    ↓
SyncManager
    ↓
SyncCoordinator
    ↓
on_change(ctx: ChangeContext)
    ↓
processor.add(operation)  // 立即执行，无事务缓冲
```

### 1.2 存在的问题

1. **缺少事务上下文**：
   - `on_vertex_change` 不知道当前事务 ID
   - 无法区分已提交和未提交的变更
   - 回滚时无法撤销索引更新

2. **即时同步模式**：
   - 变更直接发送给 processor
   - 没有缓冲机制
   - 不支持原子性提交

3. **两阶段提交不完整**：
   - `prepare_transaction` 没有实际作用
   - `commit_transaction` 只是清理缓冲区（但缓冲区为空）
   - `rollback_transaction` 无法回滚已应用的变更

4. **API 设计不一致**：
   - `prepare_transaction` 曾经需要 `ChangeContext` 参数
   - 调用方无法提供该上下文
   - 设计意图与实际使用不匹配

### 1.3 已完成的修复

- ✅ 移除 `prepare_transaction` 的 `ChangeContext` 参数
- ✅ 保留两阶段提交的调用点
- ✅ 编译通过（无错误）

## 2. 重构目标

### 2.1 功能目标

1. **事务性**：
   - 索引更新与数据更新原子性
   - 提交前对外不可见
   - 回滚时撤销所有变更

2. **隔离性**：
   - 支持 Read Committed（默认）
   - 可选支持 Snapshot Isolation

3. **持久性**：
   - 提交后索引变更持久化
   - 崩溃后可恢复

4. **性能**：
   - 小事务：低延迟
   - 大事务：批量优化
   - 可配置异步模式

### 2.2 非功能目标

1. **最小侵入**：尽量复用现有代码
2. **向后兼容**：保留现有 API
3. **可测试性**：完善的单元测试
4. **可维护性**：清晰的代码结构

## 3. 重构方案

### 3.1 总体设计

采用**操作日志缓冲 + 提交时应用**策略：

```
存储层 (Storage)
    ↓ (带事务 ID)
on_vertex_insert(txn_id, space_id, vertex)
    ↓
SyncManager
    ↓
SyncCoordinator
    ↓
buffer_operation(txn_id, ctx)  // 缓冲到事务缓冲区
    ↓
[事务提交]
    ↓
commit_transaction(txn_id)
    ↓
apply_buffered_operations(txn_id)  // 应用所有缓冲操作
    ↓
processor.add_batch(operations)
```

### 3.2 核心数据结构

#### 3.2.1 事务缓冲区

```rust
// src/sync/coordinator/types.rs

/// 索引操作类型
#[derive(Debug, Clone)]
pub enum IndexOperation {
    Insert {
        key: String,
        data: IndexData,
        payload: HashMap<String, Value>,
    },
    Update {
        key: String,
        old_data: IndexData,
        new_data: IndexData,
        payload: HashMap<String, Value>,
    },
    Delete {
        key: String,
    },
}

/// 索引数据类型
#[derive(Debug, Clone)]
pub enum IndexData {
    Fulltext(String),
    Vector(Vec<f32>),
}

/// 事务缓冲
#[derive(Debug, Default)]
pub struct TransactionBuffer {
    txn_id: TransactionId,
    operations: Vec<IndexOperation>,
    state: TransactionState,
}

/// 事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Active,
    Prepared,
    Committed,
    Aborted,
}

/// 同步协调器扩展
pub struct SyncCoordinator {
    // ... 现有字段

    /// 事务缓冲区
    transaction_buffers: DashMap<TransactionId, Arc<Mutex<TransactionBuffer>>>,

    /// 配置
    config: BatchConfig,
}
```

#### 3.2.2 变更上下文（保留）

```rust
// src/sync/coordinator/types.rs (已有)

#[derive(Debug, Clone)]
pub struct ChangeContext {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub index_type: IndexType,
    pub change_type: ChangeType,
    pub vertex_id: String,
    pub data: ChangeData,
}
```

### 3.3 API 设计

#### 3.3.1 存储层 API 变更

```rust
// src/storage/event_storage.rs

// 修改前
pub fn insert_vertex(&self, space: &str, vertex: &Vertex) -> Result<()> {
    // ...
    if let Some(ref sync) = self.sync_manager {
        tokio::spawn(async move {
            let _ = sync.on_vertex_change(...).await;
        });
    }
}

// 修改后
pub fn insert_vertex(&self, space: &str, vertex: &Vertex, txn_id: Option<TransactionId>) -> Result<()> {
    // ...
    if let Some(ref sync) = self.sync_manager {
        // 传递事务 ID（如果有）
        if let Some(txn_id) = txn_id {
            // 同步调用，缓冲到事务
            let _ = sync.on_vertex_insert(txn_id, space_id, vertex);
        } else {
            // 无事务，异步立即执行
            let sync = sync.clone();
            let vertex = vertex.clone();
            tokio::spawn(async move {
                let _ = sync.on_vertex_change_auto(...).await;
            });
        }
    }
}
```

#### 3.3.2 SyncManager API

```rust
// src/sync/manager.rs

impl SyncManager {
    /// 带事务的顶点插入
    pub fn on_vertex_insert(
        &self,
        txn_id: TransactionId,
        space_id: u64,
        vertex: &Vertex,
    ) -> Result<(), SyncError> {
        // 创建变更上下文
        let ctx = self.create_context(space_id, vertex, ChangeType::Insert)?;

        // 缓冲操作
        self.sync_coordinator.buffer_operation(txn_id, ctx)?;

        Ok(())
    }

    /// 无事务的顶点变更（立即执行）
    pub async fn on_vertex_change_auto(
        &self,
        space_id: u64,
        tag_name: &str,
        vertex_id: &Value,
        properties: &[(String, Value)],
        change_type: ChangeType,
    ) -> Result<(), SyncError> {
        // 立即执行，不缓冲
        self.sync_coordinator
            .on_vertex_change(space_id, tag_name, vertex_id, properties, change_type)
            .await?;
        Ok(())
    }

    /// 两阶段提交 - Prepare
    pub async fn prepare_transaction(&self, txn_id: TransactionId) -> Result<(), SyncError> {
        self.sync_coordinator.prepare_transaction(txn_id).await?;
        Ok(())
    }

    /// 两阶段提交 - Commit
    pub async fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), SyncError> {
        self.sync_coordinator.commit_transaction(txn_id).await?;
        Ok(())
    }

    /// 两阶段提交 - Rollback
    pub async fn rollback_transaction(&self, txn_id: TransactionId) -> Result<(), SyncError> {
        self.sync_coordinator.rollback_transaction(txn_id).await?;
        Ok(())
    }
}
```

#### 3.3.3 SyncCoordinator 实现

```rust
// src/sync/coordinator/coordinator.rs

impl SyncCoordinator {
    /// 缓冲索引操作
    pub fn buffer_operation(
        &self,
        txn_id: TransactionId,
        ctx: ChangeContext,
    ) -> Result<(), SyncCoordinatorError> {
        // 创建索引操作
        let operation = self.create_operation(&ctx)?;

        // 获取或创建事务缓冲区
        let buffer = self
            .transaction_buffers
            .entry(txn_id)
            .or_insert_with(|| Arc::new(Mutex::new(TransactionBuffer {
                txn_id,
                operations: Vec::new(),
                state: TransactionState::Active,
            })))
            .clone();

        // 添加操作到缓冲区
        let mut buf = buffer.lock().unwrap();
        buf.operations.push(operation);

        Ok(())
    }

    /// Prepare 阶段：验证所有操作
    pub async fn prepare_transaction(
        &self,
        txn_id: TransactionId,
    ) -> Result<(), SyncCoordinatorError> {
        let buffer = self
            .transaction_buffers
            .get(&txn_id)
            .ok_or_else(|| SyncCoordinatorError::TransactionNotFound(txn_id))?;

        let mut buf = buffer.lock().unwrap();

        // 验证所有操作可执行
        for operation in &buf.operations {
            self.validate_operation(operation)?;
        }

        // 标记为 Prepared 状态
        buf.state = TransactionState::Prepared;

        log::debug!("Transaction {:?} prepared with {} operations", txn_id, buf.operations.len());

        Ok(())
    }

    /// Commit 阶段：应用所有操作
    pub async fn commit_transaction(
        &self,
        txn_id: TransactionId,
    ) -> Result<(), SyncCoordinatorError> {
        let buffer = self
            .transaction_buffers
            .remove(&txn_id)
            .ok_or_else(|| SyncCoordinatorError::TransactionNotFound(txn_id))?;

        let (_, buffer) = buffer;
        let buf = buffer.lock().unwrap();

        if buf.state != TransactionState::Prepared {
            return Err(SyncCoordinatorError::InvalidTransactionState {
                txn_id,
                expected: TransactionState::Prepared,
                actual: buf.state,
            });
        }

        // 批量应用所有操作
        if !buf.operations.is_empty() {
            // 按索引类型分组
            let mut fulltext_ops = Vec::new();
            let mut vector_ops = Vec::new();

            for op in &buf.operations {
                match op {
                    IndexOperation::Insert { data, .. } | IndexOperation::Update { new_data: data, .. } => {
                        match data {
                            IndexData::Fulltext(_) => fulltext_ops.push(op.clone()),
                            IndexData::Vector(_) => vector_ops.push(op.clone()),
                        }
                    }
                    IndexOperation::Delete { .. } => {
                        // Delete 操作需要特殊处理
                        fulltext_ops.push(op.clone());
                    }
                }
            }

            // 批量应用
            if !fulltext_ops.is_empty() {
                self.apply_fulltext_batch(fulltext_ops).await?;
            }

            if !vector_ops.is_empty() {
                self.apply_vector_batch(vector_ops).await?;
            }
        }

        // 标记为 Committed
        log::debug!("Transaction {:?} committed", txn_id);

        Ok(())
    }

    /// Rollback 阶段：丢弃缓冲区
    pub async fn rollback_transaction(
        &self,
        txn_id: TransactionId,
    ) -> Result<(), SyncCoordinatorError> {
        let buffer = self
            .transaction_buffers
            .remove(&txn_id)
            .ok_or_else(|| SyncCoordinatorError::TransactionNotFound(txn_id))?;

        let buf = buffer.lock().unwrap();
        log::debug!("Transaction {:?} rolled back ({} operations discarded)", txn_id, buf.operations.len());

        Ok(())
    }

    /// 验证操作（可扩展）
    fn validate_operation(&self, operation: &IndexOperation) -> Result<(), SyncCoordinatorError> {
        // 基本验证：操作不为空
        match operation {
            IndexOperation::Insert { key, data, .. } => {
                if key.is_empty() {
                    return Err(SyncCoordinatorError::InvalidOperation("Insert key cannot be empty".to_string()));
                }
            }
            IndexOperation::Update { key, .. } => {
                if key.is_empty() {
                    return Err(SyncCoordinatorError::InvalidOperation("Update key cannot be empty".to_string()));
                }
            }
            IndexOperation::Delete { key } => {
                if key.is_empty() {
                    return Err(SyncCoordinatorError::InvalidOperation("Delete key cannot be empty".to_string()));
                }
            }
        }

        Ok(())
    }

    /// 批量应用全文索引操作
    async fn apply_fulltext_batch(
        &self,
        operations: Vec<IndexOperation>,
    ) -> Result<(), SyncCoordinatorError> {
        // 按 processor 分组
        let mut ops_by_processor: HashMap<(u64, String, String), Vec<IndexOperation>> = HashMap::new();

        for op in &operations {
            // 提取 key 中的 space_id, tag_name, field_name
            // 这里需要解析 key 或从 operation 中提取
            // 简化处理：假设 operation 包含这些信息
            let processor_key = self.extract_processor_key(op)?;
            ops_by_processor.entry(processor_key).or_default().push(op.clone());
        }

        // 批量应用
        for ((space_id, tag_name, field_name), ops) in ops_by_processor {
            if let Some(processor) = self.get_or_create_fulltext_processor(space_id, &tag_name, &field_name) {
                processor.add_batch(ops).await?;
            }
        }

        Ok(())
    }

    /// 批量应用向量索引操作
    async fn apply_vector_batch(
        &self,
        operations: Vec<IndexOperation>,
    ) -> Result<(), SyncCoordinatorError> {
        // 类似 fulltext_batch 的实现
        // ...
        Ok(())
    }
}
```

### 3.4 事务管理器集成

```rust
// src/transaction/manager.rs

impl<S: StorageClient + Clone + 'static> TransactionManager<S> {
    async fn commit_transaction_internal(
        &self,
        txn_id: TransactionId,
        context: TransactionContext,
    ) -> Result<(), TransactionError> {
        // Phase 1: Prepare (if two-phase commit is enabled)
        if context.is_two_phase_enabled() {
            if let Some(ref sync_manager) = self.sync_manager {
                // 1. Prepare index sync
                sync_manager
                    .prepare_transaction(txn_id)
                    .await
                    .map_err(|e| TransactionError::SyncFailed(e.to_string()))?;

                log::debug!("Index sync prepared for transaction {:?}", txn_id);
            }
        }

        // Phase 2: Commit storage transaction
        if !context.read_only {
            let mut write_txn = context.take_write_txn()?;
            let durability: redb::Durability = context.durability.into();
            write_txn.set_durability(durability);

            write_txn
                .commit()
                .map_err(|e| TransactionError::CommitFailed(e.to_string()))?;
        }

        context.transition_to(TransactionState::Committed)?;

        // Phase 3: Confirm index sync
        if context.is_two_phase_enabled() {
            if let Some(ref sync_manager) = self.sync_manager {
                sync_manager
                    .commit_transaction(txn_id)
                    .await
                    .map_err(|e| {
                        log::error!("Index sync commit failed for transaction {:?}: {}", txn_id, e);
                        // 注意：存储已提交，只能记录错误
                        TransactionError::SyncFailed(e.to_string())
                    })?;

                log::debug!("Index sync committed for transaction {:?}", txn_id);
            }
        }

        // 清理事务上下文
        self.active_transactions.remove(&txn_id);

        Ok(())
    }

    async fn rollback_transaction_internal(
        &self,
        txn_id: TransactionId,
        context: TransactionContext,
    ) -> Result<(), TransactionError> {
        // 回滚索引同步
        if let Some(ref sync_manager) = self.sync_manager {
            if let Err(e) = sync_manager.rollback_transaction(txn_id).await {
                log::warn!("Index sync rollback failed for transaction {:?}: {}", txn_id, e);
                // 继续执行，不阻断回滚
            }
        }

        // 回滚存储事务
        if !context.read_only {
            let write_txn = context.take_write_txn()?;
            write_txn
                .abort()
                .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;
        }

        context.transition_to(TransactionState::Aborted)?;
        self.active_transactions.remove(&txn_id);

        Ok(())
    }
}
```

### 3.5 存储层集成

```rust
// src/storage/event_storage.rs

impl<S: StorageClient + Clone + 'static> EventStorage<S> {
    pub fn insert_vertex(&self, space: &str, vertex: &Vertex) -> Result<()> {
        // 获取当前事务 ID（如果有）
        let txn_id = self.get_current_txn_id();

        // 插入数据
        // ...

        // 通知同步系统
        if let Some(ref sync_manager) = self.sync_manager {
            let space_id = self.inner.get_space_id(space)?;

            if let Some(txn_id) = txn_id {
                // 有事务：同步缓冲
                sync_manager.on_vertex_insert(txn_id, space_id, vertex)?;
            } else {
                // 无事务：异步立即执行
                let sync_manager = sync_manager.clone();
                let vertex = vertex.clone();
                tokio::spawn(async move {
                    let _ = sync_manager.on_vertex_change_auto(...).await;
                });
            }
        }

        Ok(())
    }

    /// 获取当前事务 ID
    fn get_current_txn_id(&self) -> Option<TransactionId> {
        // 从线程局部存储或上下文中获取
        // 实现方式取决于事务管理器的设计
        crate::transaction::context::get_current_txn_id()
    }
}
```

## 4. 实现步骤

### 阶段一：基础设施（1-2 天）

1. **定义核心数据结构**
   - [ ] `IndexOperation` 枚举
   - [ ] `TransactionBuffer` 结构
   - [ ] `TransactionState` 枚举
   - [ ] 错误类型扩展

2. **扩展 SyncCoordinator**
   - [ ] 添加 `transaction_buffers` 字段
   - [ ] 实现 `buffer_operation` 方法
   - [ ] 实现 `validate_operation` 方法

### 阶段二：两阶段提交（2-3 天）

3. **实现 Prepare/Commit/Rollback**
   - [ ] `prepare_transaction`：验证操作
   - [ ] `commit_transaction`：应用缓冲操作
   - [ ] `rollback_transaction`：丢弃缓冲区
   - [ ] 批量应用优化（`apply_fulltext_batch`, `apply_vector_batch`）

4. **集成到 TransactionManager**
   - [ ] 修改 `commit_transaction_internal`
   - [ ] 修改 `rollback_transaction_internal`
   - [ ] 添加单元测试

### 阶段三：存储层集成（2-3 天）

5. **修改存储层 API**
   - [ ] 添加 `txn_id` 参数到数据变更方法
   - [ ] 实现 `get_current_txn_id`
   - [ ] 区分有事务/无事务的处理逻辑

6. **集成测试**
   - [ ] 事务提交测试
   - [ ] 事务回滚测试
   - [ ] 并发事务测试

### 阶段四：优化与完善（2-3 天）

7. **性能优化**
   - [ ] 批量操作优化
   - [ ] 内存管理
   - [ ] 锁优化（考虑使用 `parking_lot::Mutex`）

8. **错误处理与恢复**
   - [ ] 提交失败的补偿机制
   - [ ] 崩溃恢复逻辑（可选）
   - [ ] 日志记录完善

9. **文档与测试**
   - [ ] API 文档
   - [ ] 集成测试
   - [ ] 性能基准测试

## 5. 测试计划

### 5.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buffer_operation() {
        // 测试缓冲操作
    }

    #[tokio::test]
    async fn test_prepare_transaction() {
        // 测试 prepare 阶段
    }

    #[tokio::test]
    async fn test_commit_transaction() {
        // 测试 commit 阶段
    }

    #[tokio::test]
    async fn test_rollback_transaction() {
        // 测试 rollback 阶段
    }

    #[tokio::test]
    async fn test_transactional_index_update() {
        // 完整的事务性索引更新测试
        // 1. 开始事务
        // 2. 插入顶点（应缓冲）
        // 3. 提交事务（应应用缓冲）
        // 4. 验证索引已更新
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        // 回滚测试
        // 1. 开始事务
        // 2. 插入顶点（应缓冲）
        // 3. 回滚事务（应丢弃缓冲）
        // 4. 验证索引未更新
    }
}
```

### 5.2 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_concurrent_transactions() {
        // 并发事务测试
    }

    #[tokio::test]
    async fn test_mixed_read_write() {
        // 混合读写测试
    }

    #[tokio::test]
    async fn test_large_transaction() {
        // 大事务性能测试
    }
}
```

## 6. 性能考虑

### 6.1 内存管理

- **缓冲区大小限制**：每个事务缓冲区限制为 1000 个操作
- **内存预警**：超过阈值时拒绝新事务
- **批量提交**：大事务自动分批提交

### 6.2 锁优化

- **细粒度锁**：每个事务独立的锁，减少竞争
- **读写锁**：考虑使用 `RwLock` 优化读多写少场景
- **无锁缓冲**：使用 `crossbeam` 等无锁数据结构（可选）

### 6.3 批量优化

```rust
// 按索引类型和 processor 分组
let mut grouped_ops: HashMap<ProcessorKey, Vec<IndexOperation>> = HashMap::new();
for op in operations {
    let key = extract_processor_key(&op);
    grouped_ops.entry(key).or_default().push(op);
}

// 批量应用
for (key, ops) in grouped_ops {
    processor.add_batch(ops).await?;
}
```

## 7. 向后兼容性

### 7.1 API 兼容

- 保留现有的 `on_vertex_change` 方法（标记为 deprecated）
- 新增 `on_vertex_insert` 等方法
- 渐进式迁移

### 7.2 配置选项

```rust
// src/config.rs

#[derive(Debug, Clone)]
pub struct SyncConfig {
    // ... 现有字段

    /// 同步模式
    #[serde(default)]
    pub mode: SyncMode,

    /// 是否启用事务性索引同步（默认 true）
    #[serde(default = "default_true")]
    pub transactional: bool,

    /// 每个事务的最大缓冲操作数
    #[serde(default = "default_max_buffer_size")]
    pub max_buffer_size: usize,
}

fn default_true() -> bool {
    true
}

fn default_max_buffer_size() -> usize {
    1000
}
```

## 8. 风险评估

### 8.1 技术风险

| 风险       | 影响 | 概率 | 缓解措施                 |
| ---------- | ---- | ---- | ------------------------ |
| 性能下降   | 中   | 中   | 批量优化，可配置异步模式 |
| 内存溢出   | 高   | 低   | 缓冲区大小限制，内存预警 |
| 死锁       | 中   | 低   | 细粒度锁，锁顺序约定     |
| 数据不一致 | 高   | 低   | 完善的测试，错误恢复机制 |

### 8.2 项目风险

| 风险         | 影响 | 概率 | 缓解措施                 |
| ------------ | ---- | ---- | ------------------------ |
| 工期延误     | 中   | 中   | 分阶段实现，优先核心功能 |
| 代码复杂度   | 中   | 高   | 详细文档，代码审查       |
| 测试覆盖不足 | 高   | 中   | 强制测试覆盖率要求       |

## 9. 成功标准

### 9.1 功能标准

- [ ] 事务提交时索引更新原子性
- [ ] 事务回滚时索引不更新
- [ ] 并发事务正确隔离
- [ ] 所有单元测试通过
- [ ] 所有集成测试通过

### 9.2 性能标准

- [ ] 小事务（<10 操作）延迟增加 < 10%
- [ ] 大事务（>100 操作）吞吐量提升 > 20%（批量优化）
- [ ] 内存使用增长 < 20%

### 9.3 代码质量标准

- [ ] 代码审查通过
- [ ] 测试覆盖率 > 80%
- [ ] 文档完整
- [ ] 无 clippy 警告

## 10. 总结

本重构方案基于对主流数据库的调研，采用**操作日志缓冲 + 提交时应用**的策略，实现事务性索引同步。方案特点：

1. **简单有效**：相比 MVCC 和 WAL，实现复杂度低
2. **原子性保证**：两阶段提交确保一致性
3. **性能可控**：批量优化减少开销
4. **向后兼容**：保留现有 API，渐进式迁移

建议按阶段实施，优先完成核心功能，然后逐步优化和完善。

## 附录

### A. 关键代码位置

- 数据结构：`src/sync/coordinator/types.rs`
- 协调器：`src/sync/coordinator/coordinator.rs`
- 管理器：`src/sync/manager.rs`
- 事务管理器：`src/transaction/manager.rs`
- 存储层：`src/storage/event_storage.rs`

### B. 相关文件

- 调研报告：`docs/archive/transactional_index_sync_research.md`

### C. 参考实现

- PostgreSQL MVCC: https://www.postgresql.org/docs/current/mvcc.html
- MongoDB Transactions: https://www.mongodb.com/docs/manual/core/transactions/
- Neo4j Indexes: https://neo4j.com/docs/cypher-manual/current/indexes/
