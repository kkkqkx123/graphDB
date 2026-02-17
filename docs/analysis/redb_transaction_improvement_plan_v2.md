# redb 事务管理长期改进方案 V2

> 基于PostgreSQL、TiDB、Apache Ignite事务管理最佳实践的调整方案

## 概述

本文档基于主流数据库（PostgreSQL、TiDB、Apache Ignite）的事务管理实现经验，重新设计了GraphDB的事务管理架构。

## 业界最佳实践分析

### 1. PostgreSQL 事务管理特点

- **显式事务控制**：`BEGIN/COMMIT/ROLLBACK`语法
- **WAL（Write-Ahead Logging）**：确保崩溃恢复
- **多隔离级别**：`READ COMMITTED`（默认）、`REPEATABLE READ`、`SERIALIZABLE`
- **保存点（Savepoint）**：支持部分回滚
- **PL/pgSQL过程式事务**：过程内可控制事务提交/回滚

### 2. TiDB 分布式事务特点

- **乐观事务模型**：减少锁竞争，提交时检测冲突
- **2PC（两阶段提交）**：`Prewrite` + `Commit`阶段
- **Percolator协议**：基于时间戳的MVCC实现
- **Primary Key选择**：选择一个key作为主锁
- **异步清理**：提交后异步清理锁

### 3. Apache Ignite 事务管理特点

- **显式事务API**：`begin()`、`commit()`、`rollback()`
- **锁管理器（LockManager）**：堆内/堆外锁管理
- **事务状态追踪**：`Active`、`Prepared`、`Committed`、`Aborted`
- **自动事务包装**：无显式事务时自动包裹

## 改进架构设计

### 核心设计原则

1. **分层架构**：事务管理器 → 事务上下文 → 存储操作
2. **显式优先**：优先支持显式事务控制，自动事务作为默认行为
3. **状态机驱动**：事务状态转换清晰可控
4. **资源隔离**：读写分离，MVCC支持

### 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    查询层 (Query Layer)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ Cypher Parser│  │ ExecutionPlan│  │ Transaction Node │  │
│  └──────┬───────┘  └──────┬───────┘  └────────┬─────────┘  │
└─────────┼─────────────────┼───────────────────┼────────────┘
          │                 │                   │
          ▼                 ▼                   ▼
┌─────────────────────────────────────────────────────────────┐
│                 事务管理层 (Transaction Layer)                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              TransactionManager                     │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │   │
│  │  │ Transaction │  │ Savepoint   │  │ 2PC Manager │ │   │
│  │  │   State     │  │   Manager   │  │             │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘ │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────┐
│                  存储层 (Storage Layer)                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ RedbStorage  │  │ Transactional│  │ IndexDataManager │  │
│  │              │  │   Storage    │  │                  │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## 阶段一：基础事务管理器（核心）

### 1.1 事务状态机

```rust
/// 事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// 活跃状态，可执行读写操作
    Active,
    /// 已准备（2PC阶段1完成）
    Prepared,
    /// 提交中
    Committing,
    /// 已提交
    Committed,
    /// 中止中
    Aborting,
    /// 已中止
    Aborted,
}

impl TransactionState {
    /// 检查是否可以执行操作
    pub fn can_execute(&self) -> bool {
        matches!(self, TransactionState::Active)
    }
    
    /// 检查是否可以提交
    pub fn can_commit(&self) -> bool {
        matches!(self, TransactionState::Active | TransactionState::Prepared)
    }
    
    /// 检查是否可以中止
    pub fn can_abort(&self) -> bool {
        matches!(self, TransactionState::Active | TransactionState::Prepared)
    }
}
```

### 1.2 事务上下文

```rust
/// 事务上下文
pub struct TransactionContext {
    /// 事务ID
    pub id: TransactionId,
    /// 当前状态
    state: AtomicCell<TransactionState>,
    /// 开始时间戳
    pub start_ts: Instant,
    /// 超时时间
    timeout: Duration,
    /// redb写事务
    write_txn: Option<redb::WriteTransaction>,
    /// 读事务（用于MVCC）
    read_txn: Option<redb::ReadTransaction>,
    /// 已修改的表集合（用于冲突检测）
    modified_tables: HashSet<String>,
    /// 操作日志（用于恢复）
    operation_log: Vec<OperationLog>,
}

impl TransactionContext {
    /// 检查事务是否超时
    pub fn is_expired(&self) -> bool {
        self.start_ts.elapsed() > self.timeout
    }
    
    /// 状态转换
    pub fn transition_to(&self, new_state: TransactionState) -> Result<(), TransactionError> {
        let current = self.state.load();
        
        // 验证状态转换是否合法
        match (current, new_state) {
            (TransactionState::Active, TransactionState::Prepared) => Ok(()),
            (TransactionState::Active, TransactionState::Aborting) => Ok(()),
            (TransactionState::Prepared, TransactionState::Committing) => Ok(()),
            (TransactionState::Prepared, TransactionState::Aborting) => Ok(()),
            (TransactionState::Committing, TransactionState::Committed) => Ok(()),
            (TransactionState::Aborting, TransactionState::Aborted) => Ok(()),
            _ => Err(TransactionError::InvalidStateTransition { from: current, to: new_state }),
        }?;
        
        self.state.store(new_state);
        Ok(())
    }
}
```

### 1.3 事务管理器

```rust
/// 事务管理器配置
pub struct TransactionManagerConfig {
    /// 默认事务超时时间
    pub default_timeout: Duration,
    /// 最大并发事务数
    pub max_concurrent_transactions: usize,
    /// 是否启用2PC
    pub enable_2pc: bool,
    /// 是否启用乐观锁
    pub enable_optimistic_locking: bool,
    /// 死锁检测间隔
    pub deadlock_detection_interval: Duration,
}

impl Default for TransactionManagerConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            max_concurrent_transactions: 1000,
            enable_2pc: false,
            enable_optimistic_locking: true,
            deadlock_detection_interval: Duration::from_secs(5),
        }
    }
}

/// 事务管理器
pub struct TransactionManager {
    db: Arc<Database>,
    config: TransactionManagerConfig,
    /// 活跃事务表
    active_transactions: Arc<RwLock<HashMap<TransactionId, Arc<TransactionContext>>>>,
    /// 事务ID生成器
    id_generator: AtomicU64,
    /// 锁管理器
    lock_manager: Arc<LockManager>,
    /// 2PC管理器（可选）
    two_phase_manager: Option<Arc<TwoPhaseCommitManager>>,
}

impl TransactionManager {
    /// 创建新的事务管理器
    pub fn new(db: Arc<Database>, config: TransactionManagerConfig) -> Self {
        let lock_manager = Arc::new(LockManager::new());
        let two_phase_manager = if config.enable_2pc {
            Some(Arc::new(TwoPhaseCommitManager::new(db.clone())))
        } else {
            None
        };
        
        Self {
            db,
            config,
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
            id_generator: AtomicU64::new(1),
            lock_manager,
            two_phase_manager,
        }
    }
    
    /// 开始新事务
    pub fn begin_transaction(&self, options: TransactionOptions) -> Result<TransactionId, TransactionError> {
        // 检查并发事务数限制
        let active_count = self.active_transactions.read().len();
        if active_count >= self.config.max_concurrent_transactions {
            return Err(TransactionError::TooManyTransactions);
        }
        
        let txn_id = self.id_generator.fetch_add(1, Ordering::SeqCst);
        let timeout = options.timeout.unwrap_or(self.config.default_timeout);
        
        // 创建redb事务
        let (write_txn, read_txn) = if options.read_only {
            let read_txn = self.db.begin_read()
                .map_err(|e| TransactionError::BeginFailed(e.to_string()))?;
            (None, Some(read_txn))
        } else {
            let write_txn = self.db.begin_write()
                .map_err(|e| TransactionError::BeginFailed(e.to_string()))?;
            (Some(write_txn), None)
        };
        
        let context = Arc::new(TransactionContext {
            id: txn_id,
            state: AtomicCell::new(TransactionState::Active),
            start_ts: Instant::now(),
            timeout,
            write_txn: write_txn.map(Mutex::new),
            read_txn,
            modified_tables: HashSet::new(),
            operation_log: Vec::new(),
        });
        
        self.active_transactions.write().insert(txn_id, context);
        
        Ok(txn_id)
    }
    
    /// 获取事务上下文
    pub fn get_context(&self, txn_id: TransactionId) -> Result<Arc<TransactionContext>, TransactionError> {
        self.active_transactions
            .read()
            .get(&txn_id)
            .cloned()
            .ok_or(TransactionError::TransactionNotFound(txn_id))
    }
    
    /// 提交事务
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        let context = self.get_context(txn_id)?;
        
        // 检查状态
        if !context.state.load().can_commit() {
            return Err(TransactionError::InvalidStateForCommit(context.state.load()));
        }
        
        // 检查超时
        if context.is_expired() {
            self.abort_transaction(txn_id)?;
            return Err(TransactionError::TransactionTimeout);
        }
        
        // 执行提交
        if let Some(ref two_phase) = self.two_phase_manager {
            // 使用2PC提交
            two_phase.prepare(txn_id)?;
            two_phase.commit_prepared(txn_id)?;
        } else {
            // 普通提交
            context.transition_to(TransactionState::Committing)?;
            
            if let Some(ref write_txn) = context.write_txn {
                let txn = write_txn.lock();
                txn.commit()
                    .map_err(|e| TransactionError::CommitFailed(e.to_string()))?;
            }
            
            context.transition_to(TransactionState::Committed)?;
        }
        
        // 清理
        self.active_transactions.write().remove(&txn_id);
        
        Ok(())
    }
    
    /// 中止事务
    pub fn abort_transaction(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        let context = self.get_context(txn_id)?;
        
        if !context.state.load().can_abort() {
            return Err(TransactionError::InvalidStateForAbort(context.state.load()));
        }
        
        context.transition_to(TransactionState::Aborting)?;
        
        // redb事务在Drop时会自动回滚
        drop(context);
        
        self.active_transactions.write().remove(&txn_id);
        
        Ok(())
    }
    
    /// 清理过期事务（后台任务）
    pub fn cleanup_expired_transactions(&self) {
        let expired: Vec<TransactionId> = self.active_transactions
            .read()
            .iter()
            .filter(|(_, ctx)| ctx.is_expired())
            .map(|(id, _)| *id)
            .collect();
        
        for txn_id in expired {
            let _ = self.abort_transaction(txn_id);
        }
    }
}
```

## 阶段二：保存点管理器

### 2.1 保存点设计

```rust
/// 保存点
pub struct Savepoint {
    pub id: SavepointId,
    pub name: String,
    pub created_at: Instant,
    /// redb保存点
    inner: redb::Savepoint,
    /// 保存点时的操作日志位置
    operation_log_index: usize,
}

/// 保存点管理器
pub struct SavepointManager {
    savepoints: RwLock<HashMap<TransactionId, Vec<Savepoint>>>,
    id_generator: AtomicU64,
}

impl SavepointManager {
    /// 创建保存点
    pub fn create_savepoint(
        &self,
        txn_id: TransactionId,
        name: &str,
        write_txn: &redb::WriteTransaction,
        operation_log_len: usize,
    ) -> Result<SavepointId, TransactionError> {
        let savepoint_id = self.id_generator.fetch_add(1, Ordering::SeqCst);
        
        let savepoint = Savepoint {
            id: savepoint_id,
            name: name.to_string(),
            created_at: Instant::now(),
            inner: write_txn.ephemeral_savepoint()
                .map_err(|e| TransactionError::SavepointFailed(e.to_string()))?,
            operation_log_index: operation_log_len,
        };
        
        self.savepoints
            .write()
            .entry(txn_id)
            .or_insert_with(Vec::new)
            .push(savepoint);
        
        Ok(savepoint_id)
    }
    
    /// 回滚到保存点
    pub fn rollback_to_savepoint(
        &self,
        txn_id: TransactionId,
        savepoint_id: SavepointId,
        write_txn: &redb::WriteTransaction,
    ) -> Result<usize, TransactionError> {
        let mut savepoints = self.savepoints.write();
        let txn_savepoints = savepoints
            .get_mut(&txn_id)
            .ok_or(TransactionError::NoSavepointsInTransaction)?;
        
        // 找到目标保存点
        let target_index = txn_savepoints
            .iter()
            .position(|sp| sp.id == savepoint_id)
            .ok_or(TransactionError::SavepointNotFound(savepoint_id))?;
        
        let target = &txn_savepoints[target_index];
        
        // 执行回滚
        write_txn.restore_savepoint(&target.inner)
            .map_err(|e| TransactionError::RollbackFailed(e.to_string()))?;
        
        // 移除该保存点之后的所有保存点
        let removed_count = txn_savepoints.len() - target_index - 1;
        txn_savepoints.truncate(target_index + 1);
        
        Ok(removed_count)
    }
    
    /// 释放保存点
    pub fn release_savepoint(
        &self,
        txn_id: TransactionId,
        savepoint_id: SavepointId,
    ) -> Result<(), TransactionError> {
        let mut savepoints = self.savepoints.write();
        let txn_savepoints = savepoints
            .get_mut(&txn_id)
            .ok_or(TransactionError::NoSavepointsInTransaction)?;
        
        let index = txn_savepoints
            .iter()
            .position(|sp| sp.id == savepoint_id)
            .ok_or(TransactionError::SavepointNotFound(savepoint_id))?;
        
        txn_savepoints.remove(index);
        
        Ok(())
    }
    
    /// 事务结束时清理所有保存点
    pub fn cleanup_transaction(&self, txn_id: TransactionId) {
        self.savepoints.write().remove(&txn_id);
    }
}
```

## 阶段三：两阶段提交（2PC）

### 3.1 2PC管理器

```rust
/// 2PC事务记录（持久化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoPhaseRecord {
    pub txn_id: TransactionId,
    pub state: TwoPhaseState,
    pub participants: Vec<Participant>,
    pub created_at: SystemTime,
    pub prepared_at: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwoPhaseState {
    Active,
    Preparing,
    Prepared,
    Committing,
    Committed,
    Aborting,
    Aborted,
}

/// 参与者（用于未来扩展分布式事务）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: String,
    pub address: String,
    pub state: ParticipantState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticipantState {
    Active,
    Prepared,
    Committed,
    Aborted,
}

/// 2PC管理器
pub struct TwoPhaseCommitManager {
    db: Arc<Database>,
    /// 准备中的事务
    prepared_transactions: RwLock<HashMap<TransactionId, TwoPhaseRecord>>,
    /// 持久化表定义
    _2pc_table: TableDefinition<u64, &[u8]>,
}

impl TwoPhaseCommitManager {
    pub fn new(db: Arc<Database>) -> Self {
        const TWO_PHASE_TABLE: TableDefinition<u64, &[u8]> = 
            TableDefinition::new("__2pc_records");
        
        Self {
            db,
            prepared_transactions: RwLock::new(HashMap::new()),
            _2pc_table: TWO_PHASE_TABLE,
        }
    }
    
    /// 阶段1：准备
    pub fn prepare(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        let record = TwoPhaseRecord {
            txn_id,
            state: TwoPhaseState::Preparing,
            participants: vec![], // 单机版暂无分布式参与者
            created_at: SystemTime::now(),
            prepared_at: None,
        };
        
        // 持久化准备记录
        self.persist_record(&record)?;
        
        // 执行本地准备
        // 在redb中，准备就是确保所有操作都已写入事务
        
        let mut record = record;
        record.state = TwoPhaseState::Prepared;
        record.prepared_at = Some(SystemTime::now());
        
        self.persist_record(&record)?;
        self.prepared_transactions.write().insert(txn_id, record);
        
        Ok(())
    }
    
    /// 阶段2：提交
    pub fn commit_prepared(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        let mut record = self.prepared_transactions
            .write()
            .remove(&txn_id)
            .ok_or(TransactionError::TransactionNotPrepared)?;
        
        record.state = TwoPhaseState::Committing;
        self.persist_record(&record)?;
        
        // 实际提交由TransactionManager执行
        
        record.state = TwoPhaseState::Committed;
        self.persist_record(&record)?;
        
        // 清理记录（异步）
        self.async_cleanup_record(txn_id);
        
        Ok(())
    }
    
    /// 中止已准备的事务
    pub fn abort_prepared(&self, txn_id: TransactionId) -> Result<(), TransactionError> {
        let mut record = self.prepared_transactions
            .write()
            .remove(&txn_id)
            .ok_or(TransactionError::TransactionNotPrepared)?;
        
        record.state = TwoPhaseState::Aborting;
        self.persist_record(&record)?;
        
        record.state = TwoPhaseState::Aborted;
        self.persist_record(&record)?;
        
        self.async_cleanup_record(txn_id);
        
        Ok(())
    }
    
    /// 恢复未完成的2PC事务（启动时调用）
    pub fn recover(&self) -> Result<Vec<TransactionId>, TransactionError> {
        let read_txn = self.db.begin_read()
            .map_err(|e| TransactionError::RecoveryFailed(e.to_string()))?;
        
        let table = read_txn.open_table(self._2pc_table)
            .map_err(|e| TransactionError::RecoveryFailed(e.to_string()))?;
        
        let mut recovered = Vec::new();
        
        for result in table.iter()
            .map_err(|e| TransactionError::RecoveryFailed(e.to_string()))? {
            let (_, value) = result
                .map_err(|e| TransactionError::RecoveryFailed(e.to_string()))?;
            
            let record: TwoPhaseRecord = bincode::deserialize(value.value())
                .map_err(|e| TransactionError::RecoveryFailed(e.to_string()))?;
            
            match record.state {
                TwoPhaseState::Prepared => {
                    // 准备完成但未提交，需要提交
                    self.commit_prepared(record.txn_id)?;
                    recovered.push(record.txn_id);
                }
                TwoPhaseState::Committing => {
                    // 提交中崩溃，继续提交
                    self.commit_prepared(record.txn_id)?;
                    recovered.push(record.txn_id);
                }
                TwoPhaseState::Aborting => {
                    // 中止中崩溃，继续中止
                    self.abort_prepared(record.txn_id)?;
                    recovered.push(record.txn_id);
                }
                _ => {}
            }
        }
        
        Ok(recovered)
    }
    
    fn persist_record(&self, record: &TwoPhaseRecord) -> Result<(), TransactionError> {
        let write_txn = self.db.begin_write()
            .map_err(|e| TransactionError::PersistenceFailed(e.to_string()))?;
        
        {
            let mut table = write_txn.open_table(self._2pc_table)
                .map_err(|e| TransactionError::PersistenceFailed(e.to_string()))?;
            
            let bytes = bincode::serialize(record)
                .map_err(|e| TransactionError::SerializationFailed(e.to_string()))?;
            
            table.insert(&record.txn_id, &bytes.as_slice())
                .map_err(|e| TransactionError::PersistenceFailed(e.to_string()))?;
        }
        
        write_txn.commit()
            .map_err(|e| TransactionError::PersistenceFailed(e.to_string()))?;
        
        Ok(())
    }
    
    fn async_cleanup_record(&self, txn_id: TransactionId) {
        // 异步删除已完成的2PC记录
        let db = self.db.clone();
        let table_def = self._2pc_table;
        
        std::thread::spawn(move || {
            if let Ok(write_txn) = db.begin_write() {
                if let Ok(mut table) = write_txn.open_table(table_def) {
                    let _ = table.remove(&txn_id);
                }
                let _ = write_txn.commit();
            }
        });
    }
}
```

## 阶段四：与StorageClient集成

### 4.1 事务性感知存储客户端

```rust
/// 支持事务的存储客户端
pub struct TransactionalStorageClient {
    inner: RedbStorage,
    transaction_manager: Arc<TransactionManager>,
}

impl TransactionalStorageClient {
    pub fn new(inner: RedbStorage, config: TransactionManagerConfig) -> Self {
        let transaction_manager = Arc::new(TransactionManager::new(
            inner.get_db().clone(),
            config,
        ));
        
        Self {
            inner,
            transaction_manager,
        }
    }
    
    /// 开始事务
    pub fn begin_transaction(&self, options: TransactionOptions) -> Result<TransactionId, StorageError> {
        self.transaction_manager
            .begin_transaction(options)
            .map_err(|e| StorageError::TransactionError(e.to_string()))
    }
    
    /// 提交事务
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), StorageError> {
        self.transaction_manager
            .commit_transaction(txn_id)
            .map_err(|e| StorageError::TransactionError(e.to_string()))
    }
    
    /// 中止事务
    pub fn abort_transaction(&self, txn_id: TransactionId) -> Result<(), StorageError> {
        self.transaction_manager
            .abort_transaction(txn_id)
            .map_err(|e| StorageError::TransactionError(e.to_string()))
    }
    
    /// 在事务中插入顶点
    pub fn insert_vertex_in_transaction(
        &mut self,
        txn_id: TransactionId,
        space: &str,
        vertex: Vertex,
    ) -> Result<Value, StorageError> {
        let context = self.transaction_manager
            .get_context(txn_id)
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;
        
        // 获取写事务
        let write_txn = context.write_txn
            .as_ref()
            .ok_or(StorageError::TransactionError("Read-only transaction".to_string()))?
            .lock();
        
        // 执行插入操作
        let id = self.insert_vertex_with_txn(&write_txn, space, vertex)?;
        
        // 记录操作日志
        drop(write_txn);
        
        Ok(id)
    }
    
    /// 带事务的完整操作（数据+索引）
    pub fn insert_vertex_with_indexes_in_transaction(
        &mut self,
        txn_id: TransactionId,
        space: &str,
        vertex: Vertex,
    ) -> Result<Value, StorageError> {
        let context = self.transaction_manager
            .get_context(txn_id)
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;
        
        let write_txn = context.write_txn
            .as_ref()
            .ok_or(StorageError::TransactionError("Read-only transaction".to_string()))?
            .lock();
        
        // 1. 插入顶点
        let id = self.insert_vertex_with_txn(&write_txn, space, vertex.clone())?;
        
        // 2. 更新索引（在同一事务中）
        for tag in &vertex.tags {
            let indexes = self.inner.index_metadata_manager.list_tag_indexes(space)?;
            
            for index in indexes {
                if index.schema_name == tag.name {
                    let mut index_props = Vec::new();
                    for field in &index.fields {
                        if let Some(value) = tag.properties.get(&field.name) {
                            index_props.push((field.name.clone(), value.clone()));
                        }
                    }
                    
                    if !index_props.is_empty() {
                        self.update_index_with_txn(
                            &write_txn,
                            space,
                            &id,
                            &index.name,
                            &index_props,
                        )?;
                    }
                }
            }
        }
        
        Ok(id)
    }
    
    /// 创建保存点
    pub fn create_savepoint(&self, txn_id: TransactionId, name: &str) -> Result<SavepointId, StorageError> {
        let context = self.transaction_manager
            .get_context(txn_id)
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;
        
        let write_txn = context.write_txn
            .as_ref()
            .ok_or(StorageError::TransactionError("Read-only transaction".to_string()))?
            .lock();
        
        self.transaction_manager
            .savepoint_manager
            .create_savepoint(txn_id, name, &write_txn, context.operation_log.len())
            .map_err(|e| StorageError::TransactionError(e.to_string()))
    }
    
    /// 回滚到保存点
    pub fn rollback_to_savepoint(&self, txn_id: TransactionId, savepoint_id: SavepointId) -> Result<(), StorageError> {
        let context = self.transaction_manager
            .get_context(txn_id)
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;
        
        let write_txn = context.write_txn
            .as_ref()
            .ok_or(StorageError::TransactionError("Read-only transaction".to_string()))?
            .lock();
        
        self.transaction_manager
            .savepoint_manager
            .rollback_to_savepoint(txn_id, savepoint_id, &write_txn)
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;
        
        Ok(())
    }
}
```

## 实施路线图（调整后）

### 阶段一：基础事务管理器（3-4周）

**第1周**：核心数据结构
- `TransactionState` 状态机
- `TransactionContext` 上下文
- `TransactionOptions` 配置

**第2周**：事务管理器实现
- `TransactionManager::begin_transaction`
- `TransactionManager::commit_transaction`
- `TransactionManager::abort_transaction`

**第3周**：集成与测试
- 与 `RedbStorage` 集成
- 单元测试
- 并发测试

**第4周**：性能优化与文档
- 性能基准测试
- 文档编写

### 阶段二：保存点机制（2周）

**第1周**：保存点管理器
- `SavepointManager` 实现
- 与 `TransactionManager` 集成

**第2周**：测试与文档
- 保存点测试用例
- 部分回滚场景测试

### 阶段三：两阶段提交（2周）

**第1周**：2PC管理器
- `TwoPhaseCommitManager` 实现
- 持久化机制

**第2周**：恢复机制与测试
- 启动恢复逻辑
- 崩溃恢复测试

### 阶段四：查询层集成（2周）

**第1周**：执行计划节点
- `TransactionNode` 实现
- 保存点语法支持

**第2周**：端到端测试
- 完整事务流程测试
- 性能回归测试

## 总结

本方案参考了PostgreSQL的显式事务控制、TiDB的2PC实现和Apache Ignite的事务管理API，设计了适合GraphDB的分层事务管理架构：

1. **状态机驱动**：清晰的事务状态转换
2. **分层设计**：事务管理器 → 事务上下文 → 存储操作
3. **可扩展性**：支持未来分布式事务扩展
4. **向后兼容**：保留现有API，新功能通过新trait暴露

现在开始依次实现各个阶段。
