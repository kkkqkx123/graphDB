# GraphDB 事务功能实现方案

## 概述

本文档详细描述如何将 GraphDB 的存储层与 TransactionManager 集成，并实现保存点（Savepoint）和两阶段提交（2PC）功能。

---

## 一、存储层集成方案

### 1.1 当前架构问题

当前存储层直接调用 redb API，存在以下问题：

```rust
// 当前模式：每个操作独立事务
impl VertexWriter for RedbWriter {
    fn insert_vertex(&mut self, _space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let write_txn = self.db.begin_write()?;  // 创建新事务
        // ... 执行操作
        write_txn.commit()?;  // 立即提交
        Ok(id)
    }
}
```

**问题**：
- 无法将多个操作组合成原子事务
- TransactionManager 被闲置
- 无法实现跨操作的一致性保证

### 1.2 目标架构

```
┌─────────────────────────────────────────────────────────────────┐
│                        应用层 (API/查询引擎)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   查询执行    │  │   DML操作    │  │   事务控制   │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
└─────────┼─────────────────┼─────────────────┼──────────────────┘
          │                 │                 │
          └─────────────────┼─────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────┐
│                    事务管理层 (TransactionManager)               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  begin_transaction() / commit() / abort() / savepoint() │   │
│  └─────────────────────────┬───────────────────────────────┘   │
└────────────────────────────┼────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                    存储层 (Storage Layer)                        │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  TransactionalStorage: StorageClient + TransactionAware │   │
│  │  - 使用 TransactionContext 中的 redb 事务               │   │
│  │  - 支持延迟提交和批量操作                                │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

### 1.3 集成方案设计

#### 1.3.1 方案一：事务上下文注入（推荐）

**核心思想**：存储操作接收可选的事务上下文，有上下文时使用上下文中的事务，无上下文时创建独立事务。

```rust
/// 事务感知存储 trait
pub trait TransactionalStorage {
    /// 在指定事务上下文中执行操作
    fn with_transaction<F, R>(
        &self,
        txn_id: Option<TransactionId>,
        operation: F,
    ) -> Result<R, StorageError>
    where
        F: FnOnce(&dyn StorageClient) -> Result<R, StorageError>;
}

/// 修改后的 RedbWriter
pub struct RedbWriter {
    db: Arc<Database>,
    /// 当前绑定的事务上下文（可选）
    txn_context: Option<Arc<TransactionContext>>,
}

impl RedbWriter {
    /// 绑定到事务上下文
    pub fn bind_transaction(&mut self, context: Arc<TransactionContext>) {
        self.txn_context = Some(context);
    }
    
    /// 解绑事务上下文
    pub fn unbind_transaction(&mut self) {
        self.txn_context = None;
    }
    
    /// 获取或创建写事务
    fn get_write_txn(&self) -> Result<WriteTxnRef, StorageError> {
        match &self.txn_context {
            Some(ctx) => {
                // 使用事务上下文中的 redb 写事务
                let guard = ctx.write_txn.lock();
                guard.as_ref()
                    .map(|txn| WriteTxnRef::Borrowed(txn))
                    .ok_or(StorageError::DbError("无可用写事务".to_string()))
            }
            None => {
                // 创建新的独立事务
                let txn = self.db.begin_write()
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                Ok(WriteTxnRef::Owned(txn))
            }
        }
    }
}

/// 写事务引用（借用或拥有）
pub enum WriteTxnRef<'a> {
    Borrowed(&'a redb::WriteTransaction),
    Owned(redb::WriteTransaction),
}

impl<'a> WriteTxnRef<'a> {
    /// 如果不是绑定的事务，则提交
    pub fn commit_if_owned(self) -> Result<(), StorageError> {
        match self {
            WriteTxnRef::Owned(txn) => {
                txn.commit().map_err(|e| StorageError::DbError(e.to_string()))
            }
            WriteTxnRef::Borrowed(_) => Ok(()),  // 绑定的事务由 TransactionManager 提交
        }
    }
}
```

**修改后的写入操作**：

```rust
impl VertexWriter for RedbWriter {
    fn insert_vertex(&mut self, _space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let id = match vertex.vid() {
            Value::Int(0) | Value::Null(_) => Value::Int(generate_id() as i64),
            _ => vertex.vid().clone(),
        };
        
        let vertex_with_id = Vertex::new(id.clone(), vertex.tags);
        let vertex_bytes = vertex_to_bytes(&vertex_with_id)?;
        let id_bytes = value_to_bytes(&id)?;

        // 获取写事务（可能是绑定的或新建的）
        let write_txn = self.get_write_txn()?;
        
        {
            let mut table = write_txn.open_table(NODES_TABLE)?;
            table.insert(ByteKey(id_bytes), ByteKey(vertex_bytes))?;
        }
        
        // 如果是独立事务则提交，绑定事务则不提交
        write_txn.commit_if_owned()?;

        Ok(id)
    }
}
```

#### 1.3.2 方案二：存储操作队列（批量优化）

**核心思想**：在事务中累积操作，提交时批量执行。

```rust
/// 存储操作队列
pub struct StorageOperationQueue {
    operations: Vec<StorageOperation>,
}

pub enum StorageOperation {
    InsertVertex { space: String, vertex: Vertex },
    UpdateVertex { space: String, vertex: Vertex },
    DeleteVertex { space: String, id: Value },
    InsertEdge { space: String, edge: Edge },
    DeleteEdge { space: String, src: Value, dst: Value, edge_type: String },
}

/// 事务上下文扩展
pub struct TransactionContext {
    // ... 原有字段
    operation_queue: Mutex<StorageOperationQueue>,
}

impl TransactionContext {
    /// 添加操作到队列
    pub fn queue_operation(&self, op: StorageOperation) {
        self.operation_queue.lock().operations.push(op);
    }
    
    /// 执行所有队列中的操作
    pub fn flush_operations(&self, storage: &RedbStorage) -> Result<(), StorageError> {
        let queue = std::mem::take(&mut *self.operation_queue.lock());
        
        // 获取写事务
        let write_txn = self.write_txn.lock();
        let txn = write_txn.as_ref()
            .ok_or(StorageError::DbError("无可用写事务".to_string()))?;
        
        for op in queue.operations {
            match op {
                StorageOperation::InsertVertex { space, vertex } => {
                    // 直接操作 redb 表
                }
                // ... 其他操作
            }
        }
        
        Ok(())
    }
}
```

### 1.4 集成实施步骤

#### 步骤 1：修改 TransactionContext

```rust
// src/transaction/context.rs

impl TransactionContext {
    /// 获取 redb 写事务引用（供存储层使用）
    pub fn with_write_txn<F, R>(&self, f: F) -> Result<R, TransactionError>
    where
        F: FnOnce(&redb::WriteTransaction) -> Result<R, StorageError>,
    {
        if self.read_only {
            return Err(TransactionError::ReadOnlyTransaction);
        }
        
        let guard = self.write_txn.lock();
        let txn = guard.as_ref()
            .ok_or(TransactionError::Internal("写事务不可用".to_string()))?;
        
        f(txn).map_err(|e| TransactionError::Internal(e.to_string()))
    }
    
    /// 获取 redb 读事务引用
    pub fn with_read_txn<F, R>(&self, f: F) -> Result<R, TransactionError>
    where
        F: FnOnce(&redb::ReadTransaction) -> Result<R, StorageError>,
    {
        match &self.read_txn {
            Some(txn) => f(txn).map_err(|e| TransactionError::Internal(e.to_string())),
            None => {
                // 读写事务也可以读
                let guard = self.write_txn.lock();
                let txn = guard.as_ref()
                    .ok_or(TransactionError::Internal("事务不可用".to_string()))?;
                f(txn).map_err(|e| TransactionError::Internal(e.to_string()))
            }
        }
    }
}
```

#### 步骤 2：创建事务感知存储包装器

```rust
// src/storage/transactional_storage.rs

pub struct TransactionalStorage {
    inner: RedbStorage,
    txn_manager: Arc<TransactionManager>,
}

impl TransactionalStorage {
    /// 在事务中执行多个操作
    pub fn execute_in_transaction<F, R>(
        &self,
        options: TransactionOptions,
        operations: F,
    ) -> Result<R, StorageError>
    where
        F: FnOnce(&mut TransactionalStorageClient) -> Result<R, StorageError>,
    {
        // 开始事务
        let txn_id = self.txn_manager.begin_transaction(options)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        // 创建事务客户端
        let mut client = TransactionalStorageClient::new(&self.inner, &self.txn_manager, txn_id);
        
        // 执行操作
        match operations(&mut client) {
            Ok(result) => {
                // 提交事务
                self.txn_manager.commit_transaction(txn_id)
                    .map_err(|e| StorageError::DbError(e.to_string()))?;
                Ok(result)
            }
            Err(e) => {
                // 中止事务
                let _ = self.txn_manager.abort_transaction(txn_id);
                Err(e)
            }
        }
    }
}

/// 事务中的存储客户端
pub struct TransactionalStorageClient<'a> {
    storage: &'a RedbStorage,
    txn_manager: &'a TransactionManager,
    txn_id: TransactionId,
}

impl<'a> StorageClient for TransactionalStorageClient<'a> {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let ctx = self.txn_manager.get_context(self.txn_id)
            .map_err(|e| StorageError::DbError(e.to_string()))?;
        
        // 使用事务上下文中的 redb 事务执行操作
        ctx.with_write_txn(|txn| {
            let mut table = txn.open_table(NODES_TABLE)
                .map_err(|e| StorageError::DbError(e.to_string()))?;
            // ... 执行插入
            Ok(id)
        }).map_err(|e| StorageError::DbError(e.to_string()))
    }
    
    // ... 其他方法
}
```

#### 步骤 3：修改 RedbStorage 支持事务绑定

```rust
// src/storage/redb_storage.rs

impl RedbStorage {
    /// 创建支持事务的存储实例
    pub fn new_with_transaction_manager(
        db: Arc<Database>,
        txn_manager: Arc<TransactionManager>,
    ) -> Result<Self, StorageError> {
        // ... 初始化
    }
    
    /// 获取事务管理器
    pub fn transaction_manager(&self) -> Option<Arc<TransactionManager>> {
        self.txn_manager.clone()
    }
}
```

---

## 二、保存点（Savepoint）实现方案

### 2.1 保存点概念

保存点允许在事务内部设置恢复点，可以回滚到指定保存点而不中止整个事务。

```sql
BEGIN;
UPDATE accounts SET balance = balance - 100 WHERE name = 'Alice';
SAVEPOINT my_savepoint;
UPDATE accounts SET balance = balance + 100 WHERE name = 'Bob';
-- 发现错误
ROLLBACK TO my_savepoint;
-- 继续执行其他操作
UPDATE accounts SET balance = balance + 100 WHERE name = 'Wally';
COMMIT;
```

### 2.2 实现方案

#### 2.2.1 保存点管理器

```rust
// src/transaction/savepoint.rs

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// 保存点ID
pub type SavepointId = u64;

/// 保存点
#[derive(Debug, Clone)]
pub struct Savepoint {
    pub id: SavepointId,
    pub name: Option<String>,
    /// 操作日志索引（用于回滚）
    pub operation_log_index: usize,
    /// 已修改的表快照
    pub modified_tables: HashSet<String>,
    /// 创建时间
    pub created_at: Instant,
}

/// 保存点管理器
pub struct SavepointManager {
    savepoints: RwLock<HashMap<SavepointId, Savepoint>>,
    id_generator: AtomicU64,
}

impl SavepointManager {
    pub fn new() -> Self {
        Self {
            savepoints: RwLock::new(HashMap::new()),
            id_generator: AtomicU64::new(1),
        }
    }
    
    /// 创建保存点
    pub fn create_savepoint(
        &self,
        name: Option<String>,
        operation_log_index: usize,
        modified_tables: HashSet<String>,
    ) -> SavepointId {
        let id = self.id_generator.fetch_add(1, Ordering::SeqCst);
        let savepoint = Savepoint {
            id,
            name,
            operation_log_index,
            modified_tables,
            created_at: Instant::now(),
        };
        self.savepoints.write().insert(id, savepoint);
        id
    }
    
    /// 获取保存点
    pub fn get_savepoint(&self, id: SavepointId) -> Option<Savepoint> {
        self.savepoints.read().get(&id).cloned()
    }
    
    /// 删除保存点及之后的所有保存点
    pub fn remove_savepoints_from(&self, from_id: SavepointId) {
        let mut guard = self.savepoints.write();
        // 需要按创建顺序删除，这里简化处理
        guard.retain(|id, _| *id < from_id);
    }
    
    /// 获取所有保存点（按创建顺序）
    pub fn list_savepoints(&self) -> Vec<Savepoint> {
        let guard = self.savepoints.read();
        let mut savepoints: Vec<_> = guard.values().cloned().collect();
        savepoints.sort_by_key(|s| s.created_at);
        savepoints
    }
}
```

#### 2.2.2 扩展 TransactionContext 支持保存点

```rust
// src/transaction/context.rs

pub struct TransactionContext {
    // ... 原有字段
    
    /// 保存点管理器
    savepoint_manager: SavepointManager,
    
    /// 操作日志（用于回滚）
    operation_log: Mutex<Vec<OperationLog>>,
}

impl TransactionContext {
    /// 创建保存点
    pub fn create_savepoint(&self, name: Option<String>) -> Result<SavepointId, TransactionError> {
        if self.read_only {
            return Err(TransactionError::ReadOnlyTransaction);
        }
        
        let state = self.state.load();
        if !state.can_execute() {
            return Err(TransactionError::InvalidStateForCommit(state));
        }
        
        let operation_log_index = self.operation_log.lock().len();
        let modified_tables = self.modified_tables.lock().clone();
        
        let id = self.savepoint_manager.create_savepoint(
            name,
            operation_log_index,
            modified_tables,
        );
        
        Ok(id)
    }
    
    /// 回滚到保存点
    pub fn rollback_to_savepoint(&self, savepoint_id: SavepointId) -> Result<(), TransactionError> {
        if self.read_only {
            return Err(TransactionError::ReadOnlyTransaction);
        }
        
        let savepoint = self.savepoint_manager.get_savepoint(savepoint_id)
            .ok_or(TransactionError::SavepointNotFound(savepoint_id))?;
        
        // 1. 截断操作日志
        self.truncate_operation_log(savepoint.operation_log_index);
        
        // 2. 恢复已修改的表集合
        *self.modified_tables.lock() = savepoint.modified_tables.clone();
        
        // 3. 删除该保存点之后的所有保存点
        self.savepoint_manager.remove_savepoints_from(savepoint_id);
        
        // 4. 使用 redb 的保存点功能回滚
        self.with_write_txn(|txn| {
            // redb 支持 savepoint 回滚
            // txn.restore_savepoint(&savepoint_id)?;
            Ok(())
        })?;
        
        Ok(())
    }
    
    /// 释放保存点
    pub fn release_savepoint(&self, savepoint_id: SavepointId) -> Result<(), TransactionError> {
        self.savepoint_manager.remove_savepoints_from(savepoint_id + 1);
        Ok(())
    }
}
```

#### 2.2.3 操作日志设计

```rust
// src/transaction/types.rs

/// 操作日志条目（用于回滚）
#[derive(Debug, Clone)]
pub enum OperationLog {
    /// 插入顶点
    InsertVertex {
        space: String,
        vertex_id: Vec<u8>,
        /// 插入前的值（如果有）
        previous_value: Option<Vec<u8>>,
    },
    /// 更新顶点
    UpdateVertex {
        space: String,
        vertex_id: Vec<u8>,
        previous_value: Vec<u8>,
    },
    /// 删除顶点
    DeleteVertex {
        space: String,
        vertex_id: Vec<u8>,
        deleted_value: Vec<u8>,
    },
    /// 插入边
    InsertEdge {
        space: String,
        edge_key: Vec<u8>,
        previous_value: Option<Vec<u8>>,
    },
    /// 更新边
    UpdateEdge {
        space: String,
        edge_key: Vec<u8>,
        previous_value: Vec<u8>,
    },
    /// 删除边
    DeleteEdge {
        space: String,
        edge_key: Vec<u8>,
        deleted_value: Vec<u8>,
    },
    /// Schema 变更
    SchemaChange {
        change_type: SchemaChangeType,
        space: String,
        entity_name: String,
    },
}

#[derive(Debug, Clone)]
pub enum SchemaChangeType {
    CreateTag,
    AlterTag,
    DropTag,
    CreateEdgeType,
    AlterEdgeType,
    DropEdgeType,
    CreateIndex,
    DropIndex,
}
```

#### 2.2.4 TransactionManager 扩展

```rust
// src/transaction/manager.rs

impl TransactionManager {
    /// 创建保存点
    pub fn create_savepoint(
        &self,
        txn_id: TransactionId,
        name: Option<String>,
    ) -> Result<SavepointId, TransactionError> {
        let context = self.get_context(txn_id)?;
        context.create_savepoint(name)
    }
    
    /// 回滚到保存点
    pub fn rollback_to_savepoint(
        &self,
        txn_id: TransactionId,
        savepoint_id: SavepointId,
    ) -> Result<(), TransactionError> {
        let context = self.get_context(txn_id)?;
        context.rollback_to_savepoint(savepoint_id)
    }
    
    /// 释放保存点
    pub fn release_savepoint(
        &self,
        txn_id: TransactionId,
        savepoint_id: SavepointId,
    ) -> Result<(), TransactionError> {
        let context = self.get_context(txn_id)?;
        context.release_savepoint(savepoint_id)
    }
    
    /// 列出事务的所有保存点
    pub fn list_savepoints(&self, txn_id: TransactionId) -> Result<Vec<SavepointInfo>, TransactionError> {
        let context = self.get_context(txn_id)?;
        Ok(context.savepoint_manager.list_savepoints()
            .into_iter()
            .map(|s| SavepointInfo {
                id: s.id,
                name: s.name,
                created_at: s.created_at,
            })
            .collect())
    }
}

/// 保存点信息
#[derive(Debug, Clone)]
pub struct SavepointInfo {
    pub id: SavepointId,
    pub name: Option<String>,
    pub created_at: Instant,
}
```

---

## 三、两阶段提交（2PC）实现方案

### 3.1 2PC 概念

两阶段提交是一种分布式事务协议，确保跨多个资源管理器的事务原子性。

**阶段一（准备阶段）**：
1. 协调者询问所有参与者是否可以提交
2. 参与者执行本地事务并记录准备日志
3. 参与者回复 Yes/No

**阶段二（提交阶段）**：
- 如果所有参与者回复 Yes：
  1. 协调者记录提交日志
  2. 协调者通知所有参与者提交
  3. 参与者提交本地事务
- 如果有参与者回复 No：
  1. 协调者记录回滚日志
  2. 协调者通知所有参与者回滚
  3. 参与者回滚本地事务

### 3.2 单机版 2PC 实现

对于 GraphDB（单节点），2PC 主要用于：
1. 跨多个存储引擎的事务
2. 与外部系统的分布式事务集成
3. 提供更严格的持久性保证

```rust
// src/transaction/two_phase.rs

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// 2PC 事务ID
pub type Xid = String;

/// 2PC 事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwoPhaseState {
    /// 活跃状态
    Active,
    /// 准备中
    Preparing,
    /// 已准备（等待提交/回滚）
    Prepared,
    /// 提交中
    Committing,
    /// 已提交
    Committed,
    /// 回滚中
    RollingBack,
    /// 已回滚
    RolledBack,
}

/// 2PC 事务上下文
pub struct TwoPhaseTransaction {
    pub xid: Xid,
    pub state: AtomicCell<TwoPhaseState>,
    pub participants: Vec<Participant>,
    pub created_at: Instant,
    pub timeout: Duration,
    /// 准备日志位置（用于恢复）
    pub prepare_log_position: Option<u64>,
}

/// 参与者
pub struct Participant {
    pub id: String,
    pub resource_manager: Arc<dyn ResourceManager>,
    pub vote: Option<Vote>,
}

/// 投票结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vote {
    Yes,
    No,
}

/// 资源管理器 trait（参与者需要实现）
pub trait ResourceManager: Send + Sync {
    /// 准备
    fn prepare(&self, xid: &Xid) -> Result<Vote, TransactionError>;
    
    /// 提交
    fn commit(&self, xid: &Xid) -> Result<(), TransactionError>;
    
    /// 回滚
    fn rollback(&self, xid: &Xid) -> Result<(), TransactionError>;
    
    /// 忘记（清理）
    fn forget(&self, xid: &Xid) -> Result<(), TransactionError>;
}
```

### 3.3 2PC 协调者实现

```rust
// src/transaction/two_phase.rs

/// 2PC 协调者
pub struct TwoPhaseCoordinator {
    /// 活跃的事务
    transactions: RwLock<HashMap<Xid, Arc<TwoPhaseTransaction>>>,
    /// 已准备的事务（用于恢复）
    prepared_txns: RwLock<HashMap<Xid, PreparedRecord>>,
    /// 日志管理器
    log_manager: Arc<LogManager>,
}

impl TwoPhaseCoordinator {
    /// 开始 2PC 事务
    pub fn begin_transaction(
        &self,
        xid: Xid,
        participants: Vec<Arc<dyn ResourceManager>>,
    ) -> Result<Arc<TwoPhaseTransaction>, TransactionError> {
        let txn = Arc::new(TwoPhaseTransaction {
            xid: xid.clone(),
            state: AtomicCell::new(TwoPhaseState::Active),
            participants: participants.into_iter()
                .enumerate()
                .map(|(i, rm)| Participant {
                    id: format!("participant_{}", i),
                    resource_manager: rm,
                    vote: None,
                })
                .collect(),
            created_at: Instant::now(),
            timeout: Duration::from_secs(30),
            prepare_log_position: None,
        });
        
        self.transactions.write().insert(xid, txn.clone());
        Ok(txn)
    }
    
    /// 阶段一：准备
    pub async fn prepare(&self, xid: &Xid) -> Result<bool, TransactionError> {
        let txn = self.get_transaction(xid)?;
        txn.state.store(TwoPhaseState::Preparing);
        
        // 记录准备日志（用于恢复）
        let log_pos = self.log_manager.append_prepare_record(xid)?;
        txn.prepare_log_position = Some(log_pos);
        
        // 询问所有参与者
        let mut all_yes = true;
        for participant in &txn.participants {
            match participant.resource_manager.prepare(xid) {
                Ok(Vote::Yes) => {
                    // 记录投票
                }
                Ok(Vote::No) | Err(_) => {
                    all_yes = false;
                    break;
                }
            }
        }
        
        if all_yes {
            txn.state.store(TwoPhaseState::Prepared);
            // 记录已准备状态
            self.prepared_txns.write().insert(xid.clone(), PreparedRecord {
                xid: xid.clone(),
                participants: txn.participants.iter()
                    .map(|p| p.id.clone())
                    .collect(),
                prepared_at: Instant::now(),
            });
        }
        
        Ok(all_yes)
    }
    
    /// 阶段二：提交
    pub async fn commit(&self, xid: &Xid) -> Result<(), TransactionError> {
        let txn = self.get_transaction(xid)?;
        
        if txn.state.load() != TwoPhaseState::Prepared {
            return Err(TransactionError::TransactionNotPrepared(0)); // 需要转换错误类型
        }
        
        txn.state.store(TwoPhaseState::Committing);
        
        // 记录提交日志
        self.log_manager.append_commit_record(xid)?;
        
        // 通知所有参与者提交
        for participant in &txn.participants {
            if let Err(e) = participant.resource_manager.commit(xid) {
                // 记录错误，继续通知其他参与者
                log::error!("Failed to commit participant {}: {}", participant.id, e);
            }
        }
        
        txn.state.store(TwoPhaseState::Committed);
        
        // 清理
        self.transactions.write().remove(xid);
        self.prepared_txns.write().remove(xid);
        self.log_manager.append_end_record(xid)?;
        
        Ok(())
    }
    
    /// 阶段二：回滚
    pub async fn rollback(&self, xid: &Xid) -> Result<(), TransactionError> {
        let txn = self.get_transaction(xid)?;
        
        txn.state.store(TwoPhaseState::RollingBack);
        
        // 记录回滚日志
        self.log_manager.append_rollback_record(xid)?;
        
        // 通知所有参与者回滚
        for participant in &txn.participants {
            if let Err(e) = participant.resource_manager.rollback(xid) {
                log::error!("Failed to rollback participant {}: {}", participant.id, e);
            }
        }
        
        txn.state.store(TwoPhaseState::RolledBack);
        
        // 清理
        self.transactions.write().remove(xid);
        self.prepared_txns.write().remove(xid);
        
        Ok(())
    }
    
    /// 恢复（崩溃后）
    pub fn recover(&self) -> Result<(), TransactionError> {
        // 1. 读取日志
        let logs = self.log_manager.read_logs()?;
        
        // 2. 找到所有已准备但未完成的事务
        let mut prepared_txns: HashMap<Xid, Vec<LogRecord>> = HashMap::new();
        
        for log in logs {
            match log {
                LogRecord::Prepare { xid } => {
                    prepared_txns.entry(xid).or_default().push(log);
                }
                LogRecord::Commit { xid } => {
                    // 需要提交的事务
                    if let Some(_) = prepared_txns.remove(&xid) {
                        // 重新提交
                        self.recommit(&xid)?;
                    }
                }
                LogRecord::Rollback { xid } => {
                    prepared_txns.remove(&xid);
                }
                LogRecord::End { xid } => {
                    prepared_txns.remove(&xid);
                }
            }
        }
        
        // 3. 对于没有 Commit/Rollback 记录的已准备事务，需要询问参与者
        for (xid, _) in prepared_txns {
            // 启发式决策或人工干预
            log::warn!("Heuristic decision needed for transaction {}", xid);
        }
        
        Ok(())
    }
    
    fn get_transaction(&self, xid: &Xid) -> Result<Arc<TwoPhaseTransaction>, TransactionError> {
        self.transactions.read().get(xid)
            .cloned()
            .ok_or(TransactionError::TransactionNotFound(0))
    }
}
```

### 3.4 redb 作为参与者的实现

```rust
// src/storage/redb_2pc_participant.rs

use crate::transaction::two_phase::{ResourceManager, Vote, Xid};
use crate::transaction::TransactionError;
use redb::Database;
use std::sync::Arc;

/// redb 作为 2PC 参与者
pub struct Redb2PCParticipant {
    db: Arc<Database>,
    /// 已准备的事务（xid -> WriteTransaction）
    prepared_txns: RwLock<HashMap<Xid, redb::WriteTransaction>>,
}

impl ResourceManager for Redb2PCParticipant {
    fn prepare(&self, xid: &Xid) -> Result<Vote, TransactionError> {
        // 1. 开始 redb 事务
        let write_txn = self.db.begin_write()
            .map_err(|e| TransactionError::Internal(e.to_string()))?;
        
        // 2. 设置两阶段提交
        write_txn.set_two_phase_commit(true);
        
        // 3. 存储事务（等待提交/回滚）
        self.prepared_txns.write().insert(xid.to_string(), write_txn);
        
        Ok(Vote::Yes)
    }
    
    fn commit(&self, xid: &Xid) -> Result<(), TransactionError> {
        let mut guard = self.prepared_txns.write();
        let write_txn = guard.remove(xid)
            .ok_or(TransactionError::Internal("Transaction not prepared".to_string()))?;
        
        // 提交 redb 事务
        write_txn.commit()
            .map_err(|e| TransactionError::CommitFailed(e.to_string()))?;
        
        Ok(())
    }
    
    fn rollback(&self, xid: &Xid) -> Result<(), TransactionError> {
        let mut guard = self.prepared_txns.write();
        let write_txn = guard.remove(xid)
            .ok_or(TransactionError::Internal("Transaction not prepared".to_string()))?;
        
        // redb 事务在 Drop 时自动回滚
        drop(write_txn);
        
        Ok(())
    }
    
    fn forget(&self, xid: &Xid) -> Result<(), TransactionError> {
        self.prepared_txns.write().remove(xid);
        Ok(())
    }
}
```

### 3.5 与 TransactionManager 集成

```rust
// src/transaction/manager.rs

pub struct TransactionManager {
    // ... 原有字段
    
    /// 2PC 协调者（可选）
    two_phase_coordinator: Option<Arc<TwoPhaseCoordinator>>,
    
    /// 是否启用 2PC
    enable_2pc: bool,
}

impl TransactionManager {
    /// 使用 2PC 开始事务
    pub fn begin_2pc_transaction(
        &self,
        options: TransactionOptions,
        xid: Xid,
        participants: Vec<Arc<dyn ResourceManager>>,
    ) -> Result<TransactionId, TransactionError> {
        if !self.enable_2pc {
            return Err(TransactionError::Internal("2PC not enabled".to_string()));
        }
        
        let coordinator = self.two_phase_coordinator.as_ref()
            .ok_or(TransactionError::Internal("2PC coordinator not available".to_string()))?;
        
        // 开始 2PC 事务
        let two_phase_txn = coordinator.begin_transaction(xid, participants)?;
        
        // 同时创建普通事务上下文
        let txn_id = self.begin_transaction(options)?;
        
        // 关联 2PC 事务
        if let Ok(ctx) = self.get_context(txn_id) {
            ctx.set_two_phase_transaction(two_phase_txn);
        }
        
        Ok(txn_id)
    }
    
    /// 准备（阶段一）
    pub async fn prepare_transaction(&self, txn_id: TransactionId) -> Result<bool, TransactionError> {
        let ctx = self.get_context(txn_id)?;
        
        if let Some(two_phase_txn) = ctx.two_phase_transaction() {
            let coordinator = self.two_phase_coordinator.as_ref()
                .ok_or(TransactionError::Internal("2PC coordinator not available".to_string()))?;
            
            coordinator.prepare(&two_phase_txn.xid).await
        } else {
            Err(TransactionError::Internal("Not a 2PC transaction".to_string()))
        }
    }
    
    /// 提交 2PC 事务（阶段二）
    pub async fn commit_2pc_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        let ctx = self.get_context(txn_id)?;
        
        if let Some(two_phase_txn) = ctx.two_phase_transaction() {
            let coordinator = self.two_phase_coordinator.as_ref()
                .ok_or(TransactionError::Internal("2PC coordinator not available".to_string()))?;
            
            // 阶段二：提交
            coordinator.commit(&two_phase_txn.xid).await?;
            
            // 提交本地事务
            self.commit_transaction(txn_id)
        } else {
            Err(TransactionError::Internal("Not a 2PC transaction".to_string()))
        }
    }
}
```

---

## 四、实施路线图

### 阶段一：存储层集成（优先级：高）

**目标**：使 TransactionManager 能够管理存储操作

**任务**：
1. 修改 `TransactionContext`，添加 `with_write_txn` 和 `with_read_txn` 方法
2. 创建 `TransactionalStorage` 包装器
3. 修改 `RedbWriter` 支持事务上下文注入
4. 添加存储层测试用例

**时间估计**：3-5 天

### 阶段二：保存点实现（优先级：中）

**目标**：支持事务内的部分回滚

**任务**：
1. 实现 `SavepointManager`
2. 扩展 `TransactionContext` 支持保存点
3. 设计并实现操作日志
4. 添加保存点相关 API
5. 编写测试用例

**时间估计**：5-7 天

### 阶段三：两阶段提交（优先级：低）

**目标**：支持分布式事务和更严格的持久性保证

**任务**：
1. 实现 `TwoPhaseCoordinator`
2. 实现 `ResourceManager` trait
3. 实现 `Redb2PCParticipant`
4. 添加日志管理器
5. 实现恢复逻辑
6. 编写测试用例

**时间估计**：7-10 天

### 阶段四：集成测试与优化（优先级：中）

**任务**：
1. 端到端事务测试
2. 性能基准测试
3. 崩溃恢复测试
4. 文档完善

**时间估计**：3-5 天

---

## 五、关键设计决策

### 5.1 事务传播策略

| 策略 | 优点 | 缺点 | 适用场景 |
|------|------|------|----------|
| **显式传递** | 清晰可控 | 代码侵入性强 | 核心业务逻辑 |
| **线程本地存储** | 对业务代码透明 | 调试困难 | 遗留系统集成 |
| **参数注入** | 灵活性高 | 需要修改接口 | 新开发功能 |

**建议**：采用**显式传递**策略，通过 `TransactionalStorageClient` 明确事务边界。

### 5.2 保存点实现方式

| 方式 | 优点 | 缺点 |
|------|------|------|
| **操作日志回滚** | 精确控制 | 实现复杂 |
| **redb savepoint** | 简单可靠 | 依赖存储引擎 |
| **混合方案** | 兼顾两者 | 维护成本高 |

**建议**：采用**混合方案**：
- 使用 redb 的 savepoint 进行底层数据回滚
- 使用操作日志进行高层语义回滚

### 5.3 2PC 日志存储

| 存储方式 | 优点 | 缺点 |
|----------|------|------|
| **WAL 日志** | 与存储集成 | 耦合度高 |
| **独立日志文件** | 灵活可控 | 需要额外管理 |
| **数据库表** | 易于查询 | 有循环依赖风险 |

**建议**：采用**独立日志文件**，使用追加写模式保证性能。

---

## 六、风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 性能下降 | 高 | 延迟事务创建、批量提交、连接池 |
| 死锁 | 高 | 超时检测、锁排序、死锁检测算法 |
| 数据不一致 | 极高 | 充分测试、渐进式部署、回滚机制 |
| 内存泄漏 | 中 | 事务超时清理、资源监控 |
| 复杂度增加 | 中 | 模块化设计、清晰接口、完善文档 |

---

## 七、示例代码

### 7.1 基本事务使用

```rust
use graphdb::transaction::{TransactionManager, TransactionOptions};
use graphdb::storage::TransactionalStorage;

// 创建事务管理器
let txn_manager = Arc::new(TransactionManager::new(db.clone(), Default::default()));
let storage = TransactionalStorage::new(redb_storage, txn_manager);

// 执行事务
let result = storage.execute_in_transaction(
    TransactionOptions::new().with_durability(DurabilityLevel::Immediate),
    |client| {
        // 插入顶点
        let vertex = Vertex::new(Value::Int(1), vec![tag]);
        let id = client.insert_vertex("my_space", vertex)?;
        
        // 插入边
        let edge = Edge::new(id.clone(), Value::Int(2), "KNOWS".to_string());
        client.insert_edge("my_space", edge)?;
        
        Ok(id)
    },
)?;
```

### 7.2 保存点使用

```rust
// 开始事务
let txn_id = manager.begin_transaction(TransactionOptions::default())?;

// 执行操作
let ctx = manager.get_context(txn_id)?;
ctx.with_write_txn(|txn| {
    // ... 插入数据
    Ok(())
})?;

// 创建保存点
let sp1 = manager.create_savepoint(txn_id, Some("after_insert".to_string()))?;

// 执行更多操作
ctx.with_write_txn(|txn| {
    // ... 更新数据
    Ok(())
})?;

// 回滚到保存点
manager.rollback_to_savepoint(txn_id, sp1)?;

// 继续执行其他操作
ctx.with_write_txn(|txn| {
    // ... 其他操作
    Ok(())
})?;

// 提交事务
manager.commit_transaction(txn_id)?;
```

### 7.3 两阶段提交使用

```rust
use graphdb::transaction::two_phase::{TwoPhaseCoordinator, Redb2PCParticipant};

// 创建参与者
let redb_participant = Arc::new(Redb2PCParticipant::new(db.clone()));
let other_participant = Arc::new(OtherResourceManager::new());

// 开始 2PC 事务
let txn_id = manager.begin_2pc_transaction(
    TransactionOptions::default(),
    "txn_12345".to_string(),
    vec![redb_participant, other_participant],
)?;

// 执行操作
// ...

// 阶段一：准备
let can_commit = manager.prepare_transaction(txn_id).await?;

if can_commit {
    // 阶段二：提交
    manager.commit_2pc_transaction(txn_id).await?;
} else {
    // 回滚
    manager.abort_transaction(txn_id)?;
}
```

---

## 八、总结

本文档提供了 GraphDB 事务功能完整的实现方案：

1. **存储层集成**：通过事务上下文注入，实现 TransactionManager 与存储层的无缝集成
2. **保存点**：基于操作日志和 redb savepoint 的混合方案，支持事务内部分回滚
3. **两阶段提交**：完整的 2PC 实现，支持分布式事务和严格持久性保证

实施建议：
- 优先完成存储层集成，这是其他功能的基础
- 保存点可作为独立功能迭代开发
- 2PC 适用于需要与外部系统协调的场景，可延后实现

---

*文档版本: 1.0*  
*更新日期: 2026-02-17*  
*基于代码版本: GraphDB 事务模块 v1.0.0*
