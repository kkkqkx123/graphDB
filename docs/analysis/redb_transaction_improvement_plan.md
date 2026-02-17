# redb 事务管理长期改进方案

## 概述

本文档详细描述了如何将当前基于redb的存储层从**单操作事务模式**改进为**应用层事务管理模式**，以实现跨多个操作的ACID保证。

## 当前架构分析

### 现有问题

```rust
// 当前实现：每个操作独立事务
impl StorageClient for RedbStorage {
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let mut writer = self.writer.lock();
        let id = writer.insert_vertex(space, vertex.clone())?;  // 事务1
        
        // 风险：如果以下操作失败，数据已写入但索引未更新
        self.index_data_manager.update_vertex_indexes(space, &id, &index_props)?;  // 事务2
    }
}
```

**主要问题**：
1. 数据操作与索引更新分离，存在不一致风险
2. 级联删除（如delete_vertex_with_edges）非原子性执行
3. 无法执行跨多个操作的事务
4. 未利用redb的高级特性（保存点、两阶段提交）

## 改进方案

### 阶段一：统一事务管理器（高优先级）

#### 1.1 设计目标

- 将多个存储操作包裹在单一事务中
- 确保数据与索引的一致性
- 向后兼容现有API

#### 1.2 架构设计

```rust
/// 事务管理器
pub struct TransactionManager {
    db: Arc<Database>,
    active_transactions: Arc<Mutex<HashMap<TransactionId, ActiveTransaction>>>,
}

/// 活跃事务
pub struct ActiveTransaction {
    id: TransactionId,
    write_txn: WriteTransaction,
    savepoints: Vec<Savepoint>,
    durability: Durability,
    two_phase: bool,
}

/// 事务ID
pub type TransactionId = u64;

/// 事务选项
pub struct TransactionOptions {
    pub durability: Durability,
    pub two_phase_commit: bool,
    pub read_only: bool,
}
```

#### 1.3 核心API设计

```rust
impl TransactionManager {
    /// 开始新事务
    pub fn begin_transaction(&self, options: TransactionOptions) -> Result<TransactionId, StorageError>;
    
    /// 提交事务
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), StorageError>;
    
    /// 中止事务
    pub fn abort_transaction(&self, txn_id: TransactionId) -> Result<(), StorageError>;
    
    /// 获取事务引用
    pub fn get_transaction(&self, txn_id: TransactionId) -> Result<ActiveTransactionRef, StorageError>;
}

/// 支持事务的存储客户端
pub trait TransactionalStorageClient: StorageClient {
    /// 在指定事务中插入顶点
    fn insert_vertex_in_transaction(
        &mut self, 
        txn_id: TransactionId, 
        space: &str, 
        vertex: Vertex
    ) -> Result<Value, StorageError>;
    
    /// 在指定事务中更新索引
    fn update_indexes_in_transaction(
        &mut self,
        txn_id: TransactionId,
        space: &str,
        vertex_id: &Value,
        indexes: &[Index],
    ) -> Result<(), StorageError>;
    
    /// 批量操作（在同一事务中）
    fn execute_batch_in_transaction<F>(
        &mut self,
        txn_id: TransactionId,
        operations: F,
    ) -> Result<(), StorageError>
    where F: FnOnce(&mut dyn TransactionalStorageClient) -> Result<(), StorageError>;
}
```

#### 1.4 实现示例

```rust
/// 带事务的顶点插入
fn insert_vertex_with_indexes(
    &mut self, 
    space: &str, 
    vertex: Vertex
) -> Result<Value, StorageError> {
    let txn_id = self.transaction_manager.begin_transaction(TransactionOptions {
        durability: Durability::Immediate,
        two_phase_commit: true,
        read_only: false,
    })?;
    
    let result = self.execute_in_transaction(txn_id, |client| {
        // 1. 插入顶点
        let id = client.insert_vertex_in_transaction(txn_id, space, vertex.clone())?;
        
        // 2. 更新索引（在同一事务中）
        for tag in &vertex.tags {
            let indexes = client.get_tag_indexes(space, &tag.name)?;
            client.update_indexes_in_transaction(txn_id, space, &id, &indexes)?;
        }
        
        Ok(id)
    });
    
    match result {
        Ok(id) => {
            self.transaction_manager.commit_transaction(txn_id)?;
            Ok(id)
        }
        Err(e) => {
            self.transaction_manager.abort_transaction(txn_id)?;
            Err(e)
        }
    }
}
```

### 阶段二：保存点与部分回滚（中优先级）

#### 2.1 设计目标

- 在复杂事务中设置恢复点
- 实现部分回滚，而非全部回滚
- 支持嵌套保存点

#### 2.2 保存点API

```rust
/// 保存点管理
pub trait SavepointManager {
    /// 创建保存点
    fn create_savepoint(&mut self, txn_id: TransactionId, name: &str) -> Result<SavepointId, StorageError>;
    
    /// 回滚到保存点
    fn rollback_to_savepoint(&mut self, txn_id: TransactionId, savepoint_id: SavepointId) -> Result<(), StorageError>;
    
    /// 释放保存点
    fn release_savepoint(&mut self, txn_id: TransactionId, savepoint_id: SavepointId) -> Result<(), StorageError>;
    
    /// 获取所有保存点
    fn list_savepoints(&self, txn_id: TransactionId) -> Result<Vec<SavepointInfo>, StorageError>;
}

/// 保存点信息
pub struct SavepointInfo {
    pub id: SavepointId,
    pub name: String,
    pub created_at: Instant,
    pub operation_count: u64,
}
```

#### 2.3 使用场景

```rust
/// 复杂图操作：创建顶点及其关联边
fn create_vertex_with_relationships(
    &mut self,
    space: &str,
    vertex: Vertex,
    edges: Vec<Edge>,
) -> Result<Value, StorageError> {
    let txn_id = self.begin_transaction(TransactionOptions::default())?;
    
    // 创建初始保存点
    let sp1 = self.create_savepoint(txn_id, "before_vertex")?;
    
    let result = (|| {
        // 1. 插入顶点
        let vertex_id = self.insert_vertex_in_transaction(txn_id, space, vertex)?;
        
        // 2. 创建保存点（顶点已创建）
        let sp2 = self.create_savepoint(txn_id, "after_vertex")?;
        
        // 3. 尝试插入边
        for (i, edge) in edges.iter().enumerate() {
            if let Err(e) = self.insert_edge_in_transaction(txn_id, space, edge.clone()) {
                // 如果边插入失败，回滚到顶点创建后的状态
                // 但保留已创建的顶点
                self.rollback_to_savepoint(txn_id, sp2)?;
                
                // 记录部分失败，继续处理其他边
                log::warn!("边 {} 插入失败: {}", i, e);
                continue;
            }
        }
        
        // 4. 更新索引
        if let Err(e) = self.update_indexes_in_transaction(txn_id, space, &vertex_id) {
            // 索引更新失败，回滚到初始状态
            self.rollback_to_savepoint(txn_id, sp1)?;
            return Err(e);
        }
        
        Ok(vertex_id)
    })();
    
    match result {
        Ok(id) => {
            self.commit_transaction(txn_id)?;
            Ok(id)
        }
        Err(e) => {
            self.abort_transaction(txn_id)?;
            Err(e)
        }
    }
}
```

#### 2.4 基于redb的实现

```rust
impl ActiveTransaction {
    /// 创建保存点（基于redb的ephemeral_savepoint）
    pub fn create_savepoint(&mut self, name: &str) -> Result<SavepointId, StorageError> {
        let savepoint = self.write_txn.ephemeral_savepoint()
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;
        
        let id = self.next_savepoint_id();
        self.savepoints.push(Savepoint {
            id,
            name: name.to_string(),
            inner: savepoint,
        });
        
        Ok(id)
    }
    
    /// 回滚到保存点
    pub fn rollback_to_savepoint(&mut self, savepoint_id: SavepointId) -> Result<(), StorageError> {
        let savepoint = self.savepoints.iter()
            .find(|sp| sp.id == savepoint_id)
            .ok_or_else(|| StorageError::TransactionNotFound(savepoint_id))?;
        
        self.write_txn.restore_savepoint(&savepoint.inner)
            .map_err(|e| StorageError::TransactionError(e.to_string()))?;
        
        // 移除该保存点之后的所有保存点
        self.savepoints.retain(|sp| sp.id <= savepoint_id);
        
        Ok(())
    }
}
```

### 阶段三：两阶段提交集成（中优先级）

#### 3.1 设计目标

- 提高关键操作的崩溃安全性
- 防止提交过程中的数据损坏
- 支持事务恢复

#### 3.2 两阶段提交流程

```rust
/// 两阶段提交管理器
pub struct TwoPhaseCommitManager {
    db: Arc<Database>,
    prepared_transactions: Arc<Mutex<HashMap<TransactionId, PreparedTransaction>>>,
}

/// 已准备的事务
pub struct PreparedTransaction {
    pub txn_id: TransactionId,
    pub start_time: Instant,
    pub operations: Vec<OperationLog>,
    pub status: TransactionStatus,
}

/// 事务状态
pub enum TransactionStatus {
    Active,
    Prepared,      // 阶段1：准备完成
    Committed,     // 阶段2：已提交
    Aborted,       // 已中止
    Recovering,    // 恢复中
}
```

#### 3.3 实现细节

```rust
impl TwoPhaseCommitManager {
    /// 阶段1：准备提交
    pub fn prepare(&self, txn_id: TransactionId) -> Result<(), StorageError> {
        let mut prepared = self.prepared_transactions.lock()?;
        
        // 记录准备状态到持久存储
        let prepared_txn = PreparedTransaction {
            txn_id,
            start_time: Instant::now(),
            operations: self.get_operation_log(txn_id)?,
            status: TransactionStatus::Prepared,
        };
        
        prepared.insert(txn_id, prepared_txn);
        
        // 写入准备记录到redb
        self.write_prepare_record(txn_id)?;
        
        Ok(())
    }
    
    /// 阶段2：提交
    pub fn commit_prepared(&self, txn_id: TransactionId) -> Result<(), StorageError> {
        let mut prepared = self.prepared_transactions.lock()?;
        
        let txn = prepared.get_mut(&txn_id)
            .ok_or_else(|| StorageError::TransactionNotFound(txn_id))?;
        
        // 执行实际提交
        self.execute_commit(txn_id)?;
        
        txn.status = TransactionStatus::Committed;
        
        // 清理准备记录
        self.remove_prepare_record(txn_id)?;
        prepared.remove(&txn_id);
        
        Ok(())
    }
    
    /// 恢复未完成的准备事务
    pub fn recover(&self) -> Result<Vec<TransactionId>, StorageError> {
        let prepared_records = self.read_prepare_records()?;
        let mut recovered = Vec::new();
        
        for record in prepared_records {
            match record.status {
                TransactionStatus::Prepared => {
                    // 准备完成但未提交，需要提交
                    self.commit_prepared(record.txn_id)?;
                    recovered.push(record.txn_id);
                }
                TransactionStatus::Recovering => {
                    // 恢复中的事务，检查状态后决定提交或中止
                    if self.is_operations_complete(record.txn_id)? {
                        self.commit_prepared(record.txn_id)?;
                    } else {
                        self.abort_prepared(record.txn_id)?;
                    }
                    recovered.push(record.txn_id);
                }
                _ => {}
            }
        }
        
        Ok(recovered)
    }
}
```

#### 3.4 与redb集成

```rust
impl TransactionManager {
    /// 启用两阶段提交的事务
    pub fn begin_transaction_with_2pc(&self, options: TransactionOptions) -> Result<TransactionId, StorageError> {
        let mut txn_options = options;
        txn_options.two_phase_commit = true;
        
        let txn_id = self.begin_transaction(txn_options)?;
        
        // 设置redb的两阶段提交
        if let Ok(mut txn) = self.get_transaction(txn_id) {
            txn.write_txn.set_two_phase_commit(true);
        }
        
        Ok(txn_id)
    }
    
    /// 提交事务（自动使用两阶段提交）
    pub fn commit_transaction(&self, txn_id: TransactionId) -> Result<(), StorageError> {
        let txn = self.get_transaction(txn_id)?;
        
        if txn.two_phase {
            // 阶段1：准备
            self.two_phase_manager.prepare(txn_id)?;
            
            // 阶段2：提交
            self.two_phase_manager.commit_prepared(txn_id)?;
        } else {
            // 普通提交
            txn.write_txn.commit()
                .map_err(|e| StorageError::TransactionError(e.to_string()))?;
        }
        
        self.active_transactions.lock()?.remove(&txn_id);
        
        Ok(())
    }
}
```

### 阶段四：查询层集成（低优先级）

#### 4.1 目标

- 支持Cypher/MATCH语句中的事务控制
- 实现自动事务边界管理
- 提供显式事务语法（如BEGIN/COMMIT）

#### 4.2 语法扩展

```cypher
// 显式事务
BEGIN TRANSACTION
MATCH (n:Person {name: 'Alice'})
SET n.age = 30
CREATE (n)-[:KNOWS]->(m:Person {name: 'Bob'})
COMMIT

// 带保存点的事务
BEGIN TRANSACTION
CREATE (n:Person {name: 'Charlie'})
SAVEPOINT after_person_created
CREATE (n)-[:WORKS_AT]->(c:Company {name: 'Acme'})
ROLLBACK TO SAVEPOINT after_person_created  // 保留Person，删除Company和关系
COMMIT
```

#### 4.3 执行计划集成

```rust
/// 事务执行节点
pub struct TransactionNode {
    pub operations: Vec<ExecutionNode>,
    pub savepoints: Vec<SavepointDefinition>,
    pub durability: Durability,
    pub two_phase: bool,
}

impl ExecutionNode for TransactionNode {
    fn execute(&self, ctx: &mut ExecutionContext) -> Result<DataSet, ExecutionError> {
        let txn_id = ctx.storage.begin_transaction(TransactionOptions {
            durability: self.durability,
            two_phase_commit: self.two_phase,
            read_only: false,
        })?;
        
        // 设置事务上下文
        ctx.set_active_transaction(txn_id);
        
        let result = (|| {
            for (i, op) in self.operations.iter().enumerate() {
                // 检查是否需要创建保存点
                if let Some(sp_def) = self.savepoints.iter().find(|sp| sp.after_operation == i) {
                    ctx.storage.create_savepoint(txn_id, &sp_def.name)?;
                }
                
                // 执行操作
                op.execute(ctx)?;
            }
            
            Ok(DataSet::empty())
        })();
        
        match result {
            Ok(data) => {
                ctx.storage.commit_transaction(txn_id)?;
                ctx.clear_active_transaction();
                Ok(data)
            }
            Err(e) => {
                ctx.storage.abort_transaction(txn_id)?;
                ctx.clear_active_transaction();
                Err(e)
            }
        }
    }
}
```

## 实施路线图

### 第一阶段（4-6周）：基础事务管理器

1. **第1-2周**：设计并实现TransactionManager核心结构
   - 事务生命周期管理
   - 事务ID生成与追踪
   - 与现有StorageClient集成

2. **第3-4周**：实现事务性操作API
   - insert_vertex_in_transaction
   - update_indexes_in_transaction
   - delete_vertex_in_transaction

3. **第5-6周**：集成测试与性能优化
   - 单元测试覆盖
   - 与现有API的兼容性测试
   - 性能基准测试

### 第二阶段（3-4周）：保存点机制

1. **第1-2周**：实现SavepointManager
   - 基于redb的ephemeral_savepoint封装
   - 保存点栈管理
   - 部分回滚逻辑

2. **第3-4周**：集成到StorageClient
   - 高级API设计（create_vertex_with_relationships）
   - 错误处理与恢复
   - 文档与示例

### 第三阶段（3-4周）：两阶段提交

1. **第1-2周**：实现TwoPhaseCommitManager
   - 准备记录持久化
   - 提交协议实现
   - 恢复机制

2. **第3-4周**：集成与测试
   - 与TransactionManager集成
   - 崩溃恢复测试
   - 性能影响评估

### 第四阶段（2-3周）：查询层集成

1. **第1-2周**：语法解析与执行计划
   - Cypher事务语法扩展
   - 执行计划节点实现

2. **第3周**：集成测试
   - 端到端事务测试
   - 并发测试

## 风险评估与缓解策略

| 风险 | 影响 | 缓解策略 |
|------|------|----------|
| 性能下降 | 高 | 保持默认快速路径，事务管理器仅在高一致性需求时启用 |
| 死锁 | 中 | 实现超时机制，提供死锁检测 |
| 内存泄漏 | 中 | 事务超时自动中止，定期清理孤儿事务 |
| 向后兼容性 | 低 | 保留现有API，新功能通过新trait暴露 |

## 总结

本改进方案将分阶段实现：

1. **基础事务管理器**：解决数据与索引不一致问题
2. **保存点机制**：支持复杂操作的部分回滚
3. **两阶段提交**：提高关键操作的崩溃安全性
4. **查询层集成**：提供用户友好的事务控制语法

通过逐步实施，可以在保证系统稳定性的同时，显著提升GraphDB的事务管理能力。
