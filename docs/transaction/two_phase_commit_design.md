# 两阶段提交（2PC）事务与索引同步设计文档

## 📋 目录

- [概述](#概述)
- [问题背景](#问题背景)
- [设计目标](#设计目标)
- [架构设计](#架构设计)
- [核心组件](#核心组件)
- [实现方案](#实现方案)
- [API 设计](#api 设计)
- [使用示例](#使用示例)
- [性能考虑](#性能考虑)
- [测试策略](#测试策略)

---

## 概述

本文档描述了 GraphDB 中实现**两阶段提交**（Two-Phase Commit, 2PC）机制的设计方案，用于保证主数据存储与全文/向量索引之间的**强一致性**。

### 核心特性

- ✅ **原子性**：数据与索引要么都提交成功，要么都失败
- ✅ **FailClosed 支持**：索引同步失败时真正回滚事务
- ✅ **事务隔离**：索引更新在事务提交前对其它事务不可见
- ✅ **异步优化**：支持异步模式以提升性能

---

## 问题背景

### 当前架构缺陷

在现有实现中，事务提交流程如下：

```
1. 提交 redb 事务 (write_txn.commit())
2. 触发索引同步 (sync_manager.force_commit())
3. 索引同步失败 → 返回错误，但数据已提交 ❌
```

**关键问题**：

- redb 事务已提交，无法回滚
- FailClosed 策略仅返回错误，无法真正保证一致性
- 数据与索引可能出现不一致

### 需求分析

1. **强一致性场景**：金融、医疗等需要数据与索引严格一致
2. **可配置策略**：支持 FailOpen/FailClosed 配置
3. **性能平衡**：在一致性和性能之间提供灵活选择

---

## 设计目标

### 功能性目标

1. ✅ 实现真正的 FailClosed 语义
2. ✅ 支持事务内索引更新缓冲
3. ✅ 提供 prepare-commit-abort 三阶段 API
4. ✅ 保持向后兼容性

### 非功能性目标

1. 📈 性能开销可控（同步模式 < 50ms, 异步模式 < 5ms）
2. 🔒 线程安全，支持高并发
3. 🛡️ 故障恢复能力（系统崩溃后能清理残留）
4. 📊 可观测性（指标、日志、追踪）

---

## 架构设计

### 两阶段提交流程

```
┌─────────────────────────────────────────────────────────────┐
│                   Transaction Commit                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Phase 1: Prepare (准备阶段)                                 │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 1. 收集事务内所有索引更新                            │   │
│  │ 2. 异步执行索引同步（不提交）                        │   │
│  │ 3. 等待同步完成                                      │   │
│  │ 4. 如果失败 → Abort Transaction                      │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  Phase 2: Commit (提交阶段)                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ 1. 提交 redb 事务                                     │   │
│  │ 2. 确认提交索引更新                                  │   │
│  │ 3. 清理临时资源                                      │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 状态机

```
Transaction State:
  Active → Preparing → (PreparingFailed | PreparingSucceeded)
  PreparingSucceeded → Committing → (Committed | CommitFailed)
  PreparingFailed → Aborting → Aborted

Sync Handle State:
  Created → Syncing → (Synced | SyncFailed)
  Synced → Confirmed → Completed
  Synced → Cancelled → Aborted
```

---

## 核心组件

### 1. PendingIndexUpdate（待处理索引更新）

```rust
/// 待处理的索引更新操作
#[derive(Debug, Clone)]
pub struct PendingIndexUpdate {
    /// 所属事务 ID
    pub txn_id: TransactionId,
    /// 空间 ID
    pub space_id: u64,
    /// Tag 名称
    pub tag_name: String,
    /// 字段名称
    pub field_name: String,
    /// 文档 ID
    pub doc_id: String,
    /// 更新内容（None 表示删除）
    pub content: Option<String>,
    /// 变更类型
    pub change_type: ChangeType,
    /// 创建时间戳
    pub created_at: Instant,
}
```

### 2. SyncHandle（同步句柄）

```rust
/// 同步操作句柄，用于跟踪和控制索引同步
pub struct SyncHandle {
    /// 事务 ID
    txn_id: TransactionId,
    /// 待处理的索引更新列表
    pending_updates: Vec<PendingIndexUpdate>,
    /// 同步结果通道
    completion_tx: oneshot::Sender<Result<(), SyncError>>,
    completion_rx: Option<oneshot::Receiver<Result<(), SyncError>>>,
    /// 状态
    state: AtomicCell<SyncHandleState>,
    /// 创建时间
    created_at: Instant,
}

impl SyncHandle {
    /// 等待同步完成（阻塞）
    pub fn wait_for_completion(&self) -> Result<(), SyncError> {
        futures::executor::block_on(
            self.completion_rx.clone().unwrap()
        ).map_err(|_| SyncError::Internal("Channel closed".to_string()))?
    }

    /// 确认提交
    pub fn commit(self) -> Result<(), SyncError> {
        // 标记为已确认，通知后台完成提交
    }

    /// 取消提交
    pub fn abort(self) -> Result<(), SyncError> {
        // 取消索引更新，清理资源
    }
}
```

### 3. SyncHandleState

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncHandleState {
    /// 已创建，等待同步
    Created,
    /// 同步中
    Syncing,
    /// 同步完成，等待确认
    Synced,
    /// 同步失败
    SyncFailed,
    /// 已确认提交
    Confirmed,
    /// 已取消
    Cancelled,
}
```

### 4. IndexUpdateBuffer（索引更新缓冲区）

```rust
/// 事务内索引更新缓冲区
pub struct IndexUpdateBuffer {
    /// 按事务 ID 组织的待处理更新
    buffers: DashMap<TransactionId, Vec<PendingIndexUpdate>>,
    /// 同步中的句柄
    syncing_handles: DashMap<TransactionId, Arc<SyncHandle>>,
    /// 配置
    config: IndexBufferConfig,
}

pub struct IndexBufferConfig {
    /// 最大缓冲大小
    pub max_buffer_size: usize,
    /// 超时时间
    pub timeout: Duration,
    /// 同步模式
    pub sync_mode: SyncMode,
}
```

---

## 实现方案

### 修改清单

#### 1. 新增文件

```
src/transaction/
  ├── sync_handle.rs          # SyncHandle 和相关类型
  └── index_buffer.rs         # IndexUpdateBuffer

src/sync/
  ├── two_phase.rs            # 两阶段提交核心逻辑
  └── pending_update.rs       # PendingIndexUpdate 定义
```

#### 2. 修改文件

```
src/transaction/
  ├── manager.rs              # 实现两阶段提交逻辑
  ├── context.rs              # 添加索引更新缓冲支持
  └── types.rs                # 新增错误类型和状态

src/sync/
  ├── manager.rs              # 新增 prepare/commit/abort API
  └── batch.rs                # 支持待处理更新

src/storage/
  └── event_storage.rs        # 修改为事务内缓冲
```

### 详细实现

#### 步骤 1：修改 TransactionContext

```rust
pub struct TransactionContext {
    // ... 现有字段

    /// 待处理的索引更新（事务内缓冲）
    pending_index_updates: RwLock<Vec<PendingIndexUpdate>>,
    /// 同步句柄
    sync_handle: Mutex<Option<Arc<SyncHandle>>>,
    /// 是否启用两阶段提交
    two_phase_enabled: bool,
}

impl TransactionContext {
    /// 添加待处理索引更新
    pub fn add_pending_index_update(&self, update: PendingIndexUpdate) {
        if self.two_phase_enabled {
            self.pending_index_updates.write().push(update);
        } else {
            // 旧模式：立即异步同步
            self.immediate_async_sync(update);
        }
    }

    /// 获取所有待处理更新
    pub fn take_pending_updates(&self) -> Vec<PendingIndexUpdate> {
        std::mem::take(&mut *self.pending_index_updates.write())
    }

    /// 设置同步句柄
    pub fn set_sync_handle(&self, handle: Arc<SyncHandle>) {
        *self.sync_handle.lock() = Some(handle);
    }
}
```

#### 步骤 2：修改 SyncManager

```rust
impl SyncManager {
    /// 阶段 1：准备提交
    pub fn prepare_commit(&self, txn_id: TransactionId) -> Result<Arc<SyncHandle>, SyncError> {
        // 1. 获取待处理的索引更新
        let pending_updates = self.get_pending_updates(txn_id)?;

        // 2. 创建同步句柄
        let (tx, rx) = oneshot::channel();
        let handle = Arc::new(SyncHandle::new(txn_id, pending_updates.clone(), tx, rx));

        // 3. 异步执行索引同步
        let handle_clone = handle.clone();
        let sync_manager = self.clone();

        tokio::spawn(async move {
            handle_clone.set_state(SyncHandleState::Syncing);

            // 执行实际的索引同步
            let result = sync_manager.execute_pending_updates(&handle_clone.pending_updates).await;

            match result {
                Ok(()) => {
                    handle_clone.set_state(SyncHandleState::Synced);
                    let _ = handle_clone.completion_tx.send(Ok(()));
                }
                Err(e) => {
                    handle_clone.set_state(SyncHandleState::SyncFailed);
                    let _ = handle_clone.completion_tx.send(Err(e));
                }
            }
        });

        Ok(handle)
    }

    /// 阶段 2：确认提交
    pub fn commit_sync(&self, handle: Arc<SyncHandle>) -> Result<(), SyncError> {
        // 验证状态
        if handle.state() != SyncHandleState::Synced {
            return Err(SyncError::InvalidState);
        }

        // 标记为已确认（实际索引已在 prepare 阶段同步完成）
        handle.set_state(SyncHandleState::Confirmed);

        // 清理资源
        self.cleanup_handle(handle);

        Ok(())
    }

    /// 阶段 2：取消提交
    pub fn abort_sync(&self, handle: Arc<SyncHandle>) -> Result<(), SyncError> {
        // 标记为已取消
        handle.set_state(SyncHandleState::Cancelled);

        // 如果是 FailClosed 模式，需要回滚已同步的索引
        if self.config.failure_policy == SyncFailurePolicy::FailClosed {
            self.rollback_pending_updates(&handle.pending_updates)?;
        }

        // 清理资源
        self.cleanup_handle(handle);

        Ok(())
    }
}
```

#### 步骤 3：修改 TransactionManager.commit_transaction

```rust
pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
    let context = self.get_context(txn_id)?;

    // 检查是否可以提交
    if !context.state().can_commit() {
        return Err(TransactionError::InvalidStateForCommit(context.state()));
    }

    // Phase 1: Prepare
    if context.two_phase_enabled {
        if let Some(ref sync_manager) = self.sync_manager {
            // 1. 准备索引同步
            let sync_handle = sync_manager.prepare_commit(txn_id)
                .map_err(|e| TransactionError::SyncFailed(e.to_string()))?;

            context.set_sync_handle(sync_handle.clone());

            // 2. 等待同步完成
            match sync_handle.wait_for_completion() {
                Ok(()) => {
                    // 同步成功，继续提交
                }
                Err(e) => {
                    // 同步失败，根据策略处理
                    match sync_manager.buffer().config().failure_policy {
                        SyncFailurePolicy::FailClosed => {
                            // 回滚事务
                            self.abort_transaction_internal(context.clone())?;
                            return Err(TransactionError::SyncFailed(e.to_string()));
                        }
                        SyncFailurePolicy::FailOpen => {
                            // 降级为 FailOpen，继续提交
                            log::warn!("Index sync failed, proceeding with fail_open: {}", e);
                        }
                    }
                }
            }
        }
    }

    // Phase 2: Commit
    context.transition_to(TransactionState::Committing)?;

    // 提交 redb 事务
    if !context.read_only {
        let mut write_txn = context.take_write_txn()?;
        let durability: redb::Durability = context.durability.into();
        write_txn.set_durability(durability);

        write_txn
            .commit()
            .map_err(|e| TransactionError::CommitFailed(e.to_string()))?;
    }

    context.transition_to(TransactionState::Committed)?;

    // 确认索引提交
    if context.two_phase_enabled {
        if let Some(ref sync_manager) = self.sync_manager {
            if let Some(handle) = context.sync_handle.lock().take() {
                if let Err(e) = sync_manager.commit_sync(handle) {
                    log::error!("Failed to confirm index sync: {}", e);
                    // 注意：此时 redb 已提交，只能记录错误
                }
            }
        }
    }

    // 清理
    self.stats.decrement_active();
    self.stats.increment_committed();

    Ok(())
}
```

#### 步骤 4：修改 SyncStorage

```rust
impl<S: StorageClient> StorageClient for SyncStorage<S> {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let result = self.inner.insert_vertex(space, vertex.clone())?;

        if self.enabled {
            if let Some(ref sync_manager) = self.sync_manager {
                let space_id = self.inner.get_space_id(space)?;

                // 获取当前事务 ID（从线程本地存储或上下文）
                if let Some(txn_id) = get_current_txn_id() {
                    // 创建待处理更新
                    let update = PendingIndexUpdate {
                        txn_id,
                        space_id,
                        tag_name: vertex.tags.first().map(|t| t.name.clone()).unwrap_or_default(),
                        field_name: String::new(), // 需要从属性中提取
                        doc_id: vertex.vid.to_string(),
                        content: extract_text_content(&vertex),
                        change_type: ChangeType::Insert,
                        created_at: Instant::now(),
                    };

                    // 缓冲到事务上下文
                    sync_manager.buffer_update(txn_id, update)?;
                } else {
                    // 不在事务中，立即异步同步（旧模式）
                    self.immediate_async_sync(vertex);
                }
            }
        }

        Ok(result)
    }
}
```

---

## API 设计

### SyncManager API

```rust
impl SyncManager {
    /// 准备提交（阶段 1）
    pub fn prepare_commit(&self, txn_id: TransactionId) -> Result<Arc<SyncHandle>, SyncError>;

    /// 确认提交（阶段 2）
    pub fn commit_sync(&self, handle: Arc<SyncHandle>) -> Result<(), SyncError>;

    /// 取消提交（阶段 2）
    pub fn abort_sync(&self, handle: Arc<SyncHandle>) -> Result<(), SyncError>;

    /// 缓冲索引更新（事务内）
    pub fn buffer_update(&self, txn_id: TransactionId, update: PendingIndexUpdate) -> Result<(), SyncError>;

    /// 获取待处理更新
    pub fn get_pending_updates(&self, txn_id: TransactionId) -> Result<Vec<PendingIndexUpdate>, SyncError>;
}
```

### TransactionContext API

```rust
impl TransactionContext {
    /// 添加待处理索引更新
    pub fn add_pending_index_update(&self, update: PendingIndexUpdate);

    /// 获取并清除待处理更新
    pub fn take_pending_updates(&self) -> Vec<PendingIndexUpdate>;

    /// 设置同步句柄
    pub fn set_sync_handle(&self, handle: Arc<SyncHandle>);

    /// 启用两阶段提交
    pub fn enable_two_phase_commit(&mut self);
}
```

### TransactionOptions 扩展

```rust
pub struct TransactionOptions {
    // ... 现有字段

    /// 是否启用两阶段提交
    pub two_phase_commit: bool,
    /// 同步超时时间
    pub sync_timeout: Option<Duration>,
}

impl TransactionOptions {
    pub fn with_two_phase_commit(mut self, enabled: bool) -> Self {
        self.two_phase_commit = enabled;
        self
    }

    pub fn with_sync_timeout(mut self, timeout: Duration) -> Self {
        self.sync_timeout = Some(timeout);
        self
    }
}
```

---

## 使用示例

### 示例 1：使用 FailClosed 策略

```rust
use graphdb::{GraphDatabase, SyncFailurePolicy, TransactionOptions};

let db = GraphDatabase::open("my_db")?;
let session = db.session()?;

// 配置事务选项：启用两阶段提交 + FailClosed
let txn_options = TransactionOptions::default()
    .with_two_phase_commit(true)
    .with_sync_timeout(Duration::from_secs(5));

let txn_id = session.begin_transaction_with_config(txn_options)?;

// 执行数据操作
session.execute("INSERT VERTEX user(name) VALUES \"1\":(\"Alice\")")?;
session.execute("INSERT VERTEX user(name) VALUES \"2\":(\"Bob\")")?;

// 提交事务
// 如果索引同步失败，事务会自动回滚
match session.commit_transaction(txn_id) {
    Ok(()) => println!("Transaction committed successfully"),
    Err(e) if e.is_sync_failed() => {
        println!("Index sync failed, transaction rolled back");
        // 数据不会提交，保持一致性
    }
    Err(e) => return Err(e),
}
```

### 示例 2：降级为 FailOpen

```rust
// 配置为 FailOpen：索引同步失败不影响事务提交
let sync_config = SyncConfig {
    mode: SyncMode::Async,
    failure_policy: SyncFailurePolicy::FailOpen,
    ..Default::default()
};

// 事务会正常提交，即使索引同步失败
session.commit_transaction(txn_id)?;
// 索引会在后台重试同步
```

---

## 性能考虑

### 延迟分析

| 模式                          | 延迟影响 | 适用场景         |
| ----------------------------- | -------- | ---------------- |
| **两阶段提交（同步）**        | +10-50ms | 强一致性要求     |
| **两阶段提交（异步）**        | +1-5ms   | 平衡一致性和性能 |
| **旧模式（Fire-and-Forget）** | 0ms      | 高性能场景       |

### 优化策略

1. **批量同步**：多个索引更新合并为一次批量操作
2. **并行同步**：全文索引和向量索引并行处理
3. **超时控制**：避免长时间等待
4. **异步模式**：提供异步选项以提升性能

---

## 测试策略

### 单元测试

```rust
#[test]
fn test_two_phase_commit_success() {
    // 测试正常的两阶段提交流程
}

#[test]
fn test_fail_closed_policy() {
    // 测试 FailClosed 策略下索引同步失败的回滚
}

#[test]
fn test_fail_open_policy() {
    // 测试 FailOpen 策略下索引同步失败仍提交
}
```

### 集成测试

```rust
#[test]
fn test_transaction_with_fulltext_index() {
    // 测试事务与全文索引的一致性
}

#[test]
fn test_transaction_with_vector_index() {
    // 测试事务与向量索引的一致性
}

#[test]
fn test_concurrent_transactions() {
    // 测试并发事务的隔离性
}
```

### 故障恢复测试

```rust
#[test]
fn test_system_crash_recovery() {
    // 测试系统崩溃后的资源清理
}

#[test]
fn test_sync_handle_timeout() {
    // 测试同步超时的处理
}
```

---

## 迁移指南

### 向后兼容性

- ✅ 默认不启用两阶段提交（保持旧行为）
- ✅ 通过 `TransactionOptions` 显式启用
- ✅ 现有代码无需修改

### 迁移步骤

1. **阶段 1**：部署新代码，默认使用旧模式
2. **阶段 2**：对关键业务启用两阶段提交
3. **阶段 3**：验证稳定性后，推广到所有业务

---

## 总结

两阶段提交机制为 GraphDB 提供了：

1. ✅ **真正的 FailClosed 语义**：索引同步失败时回滚事务
2. ✅ **灵活的配置选项**：支持 FailOpen/FailClosed
3. ✅ **强一致性保证**：数据与索引原子性提交
4. ✅ **向后兼容**：不影响现有代码

通过本设计，GraphDB 能够满足金融、医疗等对一致性要求极高的场景需求。
