# 事务性索引同步实现研究

## 概述

本文档调研了主流数据库系统如何实现事务性索引同步，包括关系型数据库（PostgreSQL、MySQL）、文档数据库（MongoDB）和图数据库（Neo4j、AllegroGraph）的实现方案。

## 1. 关系型数据库实现

### 1.1 PostgreSQL

#### 核心机制

**MVCC（多版本并发控制）+ WAL（预写日志）**

PostgreSQL 使用 MVCC 和 WAL 的组合来保证事务性和索引一致性：

1. **MVCC 实现**：
   - 每行数据维护多个版本（通过 `xmin`/`xmax` 事务 ID）
   - 索引指向元组版本链的"头部"
   - 读操作访问可见的旧版本，写操作创建新版本
   - 读写不互斥，提高并发性能

2. **WAL 机制**：
   ```
   提交流程：
   1. 将变更写入 WAL（追加到日志文件）
   2. 强制 WAL 落盘（fsync）
   3. 在 pg_xact 中标记事务为 COMMITTED
   4. 数据页异步写入（后台刷新）
   ```

3. **索引维护策略**：
   - **立即更新**：INSERT/UPDATE/DELETE 时同步更新所有相关索引
   - **索引记录版本**：索引条目包含事务可见性信息
   - **清理机制**：通过 VACUUM 清理死元组和索引条目

#### 关键设计决策

- **原子性保证**：WAL 确保索引更新与数据更新原子性
- **回滚策略**：通过 undo 信息恢复索引状态
- **隔离级别**：支持 Read Committed、Repeatable Read、Serializable

**参考资料**：
- PostgreSQL WAL: https://www.postgresql.fastware.com/pzone/2025-01-postgresql-wal-the-backbone-of-reliable-transaction-logging-and-replication
- MVCC 实现：https://db.in.tum.de/~muehlbau/papers/mvcc.pdf

### 1.2 MySQL (InnoDB)

#### 核心机制

**Undo Log + Redo Log + 自适应哈希索引**

1. **Undo Log**：
   - 存储在单独的表空间
   - 用于事务回滚和 MVCC 版本构建
   - 支持一致性读（consistent read）

2. **Redo Log**：
   - 预写日志，保证持久性
   - 崩溃恢复时重放已提交事务

3. **Change Buffer**：
   - 延迟更新二级索引
   - 批量应用变更，减少随机 I/O
   - 提高写性能

4. **索引更新策略**：
   ```
   INSERT 流程：
   1. 写入数据行（主键索引）
   2. 记录 redo log
   3. 更新二级索引（或使用 change buffer 延迟）
   4. 提交时刷新 redo log
   ```

## 2. 文档数据库实现

### 2.1 MongoDB

#### 核心机制

**多文档事务 + Write Concern**

MongoDB 4.0+ 支持多文档 ACID 事务：

1. **事务模型**：
   ```javascript
   session.startTransaction({
       readConcern: { level: "local" },
       writeConcern: { w: "majority" }
   });
   
   try {
       // 更新多个集合/文档
       collection1.updateOne(session, ...);
       collection2.insertOne(session, ...);
       session.commitTransaction();
   } catch (error) {
       session.abortTransaction();
   }
   ```

2. **索引一致性保证**：
   - 事务内的所有索引更新对外不可见，直到提交
   - 提交时原子性更新所有索引
   - 回滚时丢弃所有未提交的索引变更

3. **实现特点**：
   - 使用 WiredTiger 存储引擎
   - 基于 MVCC 的并发控制
   - 支持 snapshot isolation

**参考资料**：
- MongoDB 事务文档：https://github.com/mongodb/docs/blob/main/content/manual/manual/source/core/transactions.txt

## 3. 图数据库实现

### 3.1 Neo4j

#### 核心机制

**行级锁 + 事务日志**

1. **事务管理**：
   ```cypher
   // 自动事务（推荐）
   MATCH (n:Person) RETURN n;
   
   // 显式事务
   BEGIN TRANSACTION;
   CREATE (n:Person {name: 'Alice'});
   COMMIT;
   ```

2. **索引实现**：
   - **Range Index**：B+ 树结构，支持范围查询
   - **Fulltext Index**：独立的全文索引，异步更新
   - **Vector Index**：用于向量相似度搜索

3. **一致性保证**：
   - 索引更新在事务内立即可见
   - 提交前不持久化
   - 支持 ACID 属性

4. **批量处理**：
   ```cypher
   LOAD CSV FROM 'file.csv' AS line
   CREATE (:Person {name: line[1]})
   IN TRANSACTIONS OF 10 ROWS  // 每 10 行提交一次
   ```

**参考资料**：
- Neo4j 索引：https://neo4j.com/docs/cypher-cheat-sheet/current
- 事务管理：https://medium.com/@n.peiris97/transaction-management-in-graph-databases-with-neo4j-8979021d5d21

### 3.2 AllegroGraph

#### 核心机制

**Snapshot Isolation + 事务日志**

1. **隔离模型**：
   - 每个事务看到数据库的快照
   - 提交时检测元数据冲突
   - 不支持三元组级锁

2. **提交流程**：
   ```
   1. 验证元数据一致性
   2. 写入事务日志
   3. 定期 checkpoint
   4. 更新可见性
   ```

3. **索引同步**：
   - 自由文本索引：异步更新
   - 其他索引：提交时同步更新

**参考资料**：
- AllegroGraph 文档：https://franz.com/agraph/support/documentation/agraph-introduction.html

## 4. 实现模式总结

### 4.1 索引更新时机

| 策略 | 优点 | 缺点 | 适用场景 |
|------|------|------|----------|
| **立即更新** | 实现简单，查询立即可见 | 写性能低，事务内可见性复杂 | 读多写少，小事务 |
| **延迟更新（提交时）** | 支持原子性，回滚简单 | 需要缓冲机制 | 通用场景 |
| **异步更新** | 写性能最高 | 可能丢失更新，需要补偿 | 可接受最终一致性 |

### 4.2 事务缓冲机制

#### 方案 A：操作日志缓冲（推荐）

```rust
struct TransactionBuffer {
    txn_id: TransactionId,
    operations: Vec<IndexOperation>,  // 缓冲的操作序列
}

enum IndexOperation {
    Insert { key: Key, value: Value },
    Delete { key: Key },
    Update { key: Key, old: Value, new: Value },
}
```

**流程**：
1. 数据变更时，创建 `IndexOperation` 并缓冲
2. `prepare_transaction`：验证所有操作可执行
3. `commit_transaction`：应用所有缓冲操作到索引
4. `rollback_transaction`：丢弃缓冲区

**优点**：
- 支持原子性提交
- 回滚简单（丢弃缓冲区）
- 可批量优化

**缺点**：
- 内存开销
- 需要额外的验证逻辑

#### 方案 B：版本链（MVCC）

```rust
struct IndexEntry {
    key: Key,
    versions: Vec<VersionedValue>,
}

struct VersionedValue {
    value: Value,
    txn_id: TransactionId,
    visible_from: Timestamp,
    visible_to: Option<Timestamp>,  // None 表示当前版本
}
```

**流程**：
1. 更新时创建新版本（不覆盖旧版本）
2. 查询时根据事务 ID 选择可见版本
3. 提交时更新版本可见性
4. 回滚时标记新版本为不可见

**优点**：
- 读写不互斥
- 支持快照隔离
- 历史版本可追溯

**缺点**：
- 实现复杂
- 需要垃圾回收机制
- 存储开销大

#### 方案 C：Write-Ahead Log

```rust
struct WAL {
    log_file: File,
    entries: Vec<WALEntry>,
}

struct WALEntry {
    lsn: LSN,  // Log Sequence Number
    txn_id: TransactionId,
    operation: IndexOperation,
    prev_lsn: Option<LSN>,
}
```

**流程**：
1. 所有变更先追加到 WAL
2. 强制 WAL 落盘（fsync）
3. 应用变更到索引
4. 提交时写入 COMMIT 记录
5. 回滚时根据 WAL 反向操作

**优点**：
- 持久性保证
- 支持崩溃恢复
- 可实现 point-in-time recovery

**缺点**：
- I/O 开销
- 实现复杂度高

### 4.3 推荐实现策略

基于调研结果，针对 GraphDB 项目的特点，推荐以下实现策略：

#### 短期方案（立即实现）

**操作日志缓冲 + 提交时应用**

```rust
// 1. 修改存储层，传递事务 ID
pub fn insert_vertex(
    &self,
    txn_id: TransactionId,  // 新增参数
    space: &str,
    vertex: &Vertex,
) -> Result<()> {
    // ... 插入数据
    
    // 通知同步系统（带事务 ID）
    if let Some(ref sync) = self.sync_manager {
        sync.on_vertex_insert(txn_id, space_id, vertex)?;
    }
    
    Ok(())
}

// 2. 同步系统缓冲操作
pub async fn on_vertex_insert(
    &self,
    txn_id: TransactionId,
    space_id: u64,
    vertex: &Vertex,
) -> Result<(), SyncError> {
    // 创建变更上下文
    let ctx = self.create_context(space_id, vertex, ChangeType::Insert)?;
    
    // 缓冲到事务缓冲区
    self.buffer_operation(txn_id, ctx).await?;
    
    Ok(())
}

// 3. 两阶段提交
pub async fn prepare_transaction(&self, txn_id: TransactionId) -> Result<()> {
    // 验证缓冲区所有操作可执行
    let buffer = self.get_buffer(txn_id)?;
    for op in &buffer.operations {
        self.validate_operation(op)?;
    }
    Ok(())
}

pub async fn commit_transaction(&self, txn_id: TransactionId) -> Result<()> {
    // 应用所有缓冲操作
    let buffer = self.take_buffer(txn_id)?;
    for op in buffer.operations {
        self.apply_operation(op).await?;
    }
    Ok(())
}

pub async fn rollback_transaction(&self, txn_id: TransactionId) -> Result<()> {
    // 丢弃缓冲区
    self.remove_buffer(txn_id)?;
    Ok(())
}
```

#### 长期方案（可选）

**MVCC + WAL 组合**

- 实现索引级别的 MVCC，支持快照隔离
- 添加 WAL 保证持久性和崩溃恢复
- 支持更高级的隔离级别（Serializable）

## 5. 关键设计决策

### 5.1 事务上下文传递

**问题**：当前架构中，存储层调用 `on_vertex_change` 时没有事务 ID

**解决方案**：
1. 修改存储层 API，接受 `txn_id` 参数
2. 在存储操作中获取当前事务 ID（从线程局部存储或参数传递）
3. 使用 `tokio::task_local!` 在异步上下文中传递事务 ID

### 5.2 缓冲 vs 立即更新

**决策因素**：
- 事务大小：大事务适合缓冲，小事务可立即更新
- 隔离级别要求：Snapshot Isolation 需要 MVCC
- 性能要求：异步更新性能最高但不保证原子性

**推荐**：
- 默认使用缓冲策略（支持原子性）
- 提供配置选项支持异步模式（性能优先）

### 5.3 错误处理

**提交失败**：
- 如果索引更新失败，需要回滚已应用的变更
- 使用补偿事务（compensating transaction）
- 或标记索引为不一致，后台修复

**回滚策略**：
- 简单丢弃缓冲区（缓冲策略）
- 应用 undo 操作（MVCC 策略）

## 6. 参考资料

### 6.1 论文与学术资源

1. "Fast Serializable Multi-Version Concurrency Control for Main-Memory Database Systems" (TUM)
   - https://db.in.tum.de/~muehlbau/papers/mvcc.pdf

2. "Multiversion Concurrency Control" (CMU 15-445)
   - https://15445.courses.cs.cmu.edu/spring2025/notes/19-multiversioning.pdf

### 6.2 技术博客

1. "The Write-Ahead Log: A Foundation for Reliability"
   - https://www.architecture-weekly.com/p/the-write-ahead-log-a-foundation

2. "PostgreSQL WAL: The backbone of reliable transaction logging"
   - https://www.postgresql.fastware.com/pzone/2025-01-postgresql-wal-the-backbone-of-reliable-transaction-logging-and-replication

3. "Multiversion Concurrency Control (MVCC): A Practical Deep Dive"
   - https://celerdata.com/glossary/multiversion-concurrency-control

### 6.3 官方文档

1. PostgreSQL Transactions: https://www.postgresql.org/docs/current/tutorial-transactions.html
2. MongoDB Transactions: https://github.com/mongodb/docs
3. Neo4j Indexes: https://neo4j.com/docs/cypher-cheat-sheet/current
4. AllegroGraph: https://franz.com/agraph/support/documentation/

## 7. 总结

通过对主流数据库的调研，我们得出以下结论：

1. **MVCC + WAL** 是最成熟的组合，但实现复杂度高
2. **操作日志缓冲** 是简单有效的方案，适合当前项目
3. **事务上下文传递** 是关键，需要修改存储层 API
4. **可配置策略** 允许用户在一致性和性能间权衡

下一步应基于"短期方案"实现基本的缓冲机制，然后根据实际需求考虑是否引入 MVCC 和 WAL。
