# redb ACID与事务管理分析

## 概述

redb是一个用纯Rust编写的简单、可移植、高性能、ACID兼容的嵌入式键值存储。它提供了零拷贝、线程安全的B树API，完全支持ACID事务和MVCC（多版本并发控制）。

## ACID特性

### 1. 原子性（Atomicity）

redb通过以下机制保证原子性：

- **事务边界**：所有操作都在`WriteTransaction`中进行，事务要么完全提交，要么完全回滚
- **自动回滚**：`WriteTransaction`实现了`Drop` trait，如果事务未完成且线程未panic，会自动调用`abort_inner()`回滚事务
- **两阶段提交**：支持`set_two_phase_commit(true)`启用两阶段提交，进一步增强原子性保证

```rust
impl Drop for WriteTransaction {
    fn drop(&mut self) {
        if !self.completed && !thread::panicking() && !self.mem.storage_failure() {
            if let Err(error) = self.abort_inner() {
                warn!("Failure automatically aborting transaction: {error}");
            }
        }
    }
}
```

### 2. 一致性（Consistency）

- **B树结构**：基于`BTreeMap`的API确保数据始终处于一致的状态
- **事务隔离**：读事务看到的数据在整个事务期间保持一致性快照
- **约束检查**：在事务提交前进行I/O错误检查，确保事务可提交性

### 3. 隔离性（Isolation）

redb采用**MVCC（多版本并发控制）**实现事务隔离：

#### 3.1 读写分离

- **读事务（ReadTransaction）**：只读访问，获取数据的一致性快照
- **写事务（WriteTransaction）**：支持读写操作，可修改数据库

```rust
// 开始读事务
fn begin_read(&self) -> Result<ReadTransaction, TransactionError>;

// 开始写事务
pub fn begin_write(&self) -> Result<WriteTransaction, TransactionError> {
    self.mem.check_io_errors()?; // 提前检查I/O错误
    let guard = TransactionGuard::new_write(...);
    WriteTransaction::new(guard, self.transaction_tracker.clone(), self.mem.clone())
}
```

#### 3.2 并发控制特性

- **读者不阻塞写者**：多个读事务可以并发执行，不会被写事务阻塞
- **写者不阻塞读者**：写事务进行时，读事务仍可以访问之前版本的数据
- **单写者模型**：同一时间只允许一个写事务（通过`TransactionGuard`控制）

#### 3.3 事务追踪器（TransactionTracker）

```rust
let id = self.transaction_tracker.register_read_transaction(&self.mem)?;
let guard = TransactionGuard::new_read(id, self.transaction_tracker.clone());
```

### 4. 持久性（Durability）

redb提供可配置的持久性级别：

```rust
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Durability {
    /// 此持久性级别的提交不会持久化到磁盘，除非随后有
    /// [`Durability::Immediate`]的提交
    None,
    /// 此持久性级别的提交保证在[`WriteTransaction::commit`]返回时立即持久化
    Immediate,
}
```

#### 4.1 持久性级别

| 级别 | 说明 | 使用场景 |
|------|------|----------|
| `None` | 数据不立即持久化，需要后续`Immediate`提交 | 批量写入，追求性能 |
| `Immediate` | 数据在`commit()`返回前已持久化 | 关键数据，需要强持久性保证 |

#### 4.2 崩溃安全机制

- **恢复标志**：数据库头部包含`recovery_required`标志
- **双副本头部**：使用双副本头部实现原子性更新
- **校验和保护**：使用128位校验和确保数据完整性

```rust
pub(crate) fn end_repair(&self) -> Result<()> {
    let mut state = self.state.lock().unwrap();
    state.header.recovery_required = false;
    self.write_header(&state.header)?;
    let result = self.storage.flush();
    self.needs_recovery.store(false, Ordering::Release);
    result
}
```

## 事务管理

### 1. 事务类型

#### 1.1 读事务（ReadTransaction）

- 只读访问数据库
- 获取事务开始时的一致性快照
- 不会被写事务阻塞

#### 1.2 写事务（WriteTransaction）

- 支持读写操作
- 通过`TransactionGuard`实现独占访问
- 支持保存点和回滚

### 2. 事务生命周期

```rust
// 1. 开始事务
let mut tx = db.begin_write().unwrap();

// 2. 设置持久性级别（可选）
tx.set_durability(Durability::Immediate);

// 3. 启用两阶段提交（可选）
tx.set_two_phase_commit(true);

// 4. 创建保存点（可选）
let savepoint = tx.ephemeral_savepoint().unwrap();

// 5. 执行操作
{
    let mut table = tx.open_table(table_def).unwrap();
    table.insert(&key, &value).unwrap();
}

// 6. 回滚到保存点（可选）
tx.restore_savepoint(&savepoint).unwrap();

// 7. 提交或中止
tx.commit().unwrap();  // 或 tx.abort().unwrap()
```

### 3. 保存点（Savepoint）

保存点允许在事务内部设置恢复点：

```rust
// 创建临时保存点
let savepoint = tx.ephemeral_savepoint().unwrap();

// 恢复到保存点
tx.restore_savepoint(&savepoint).unwrap();
```

**注意事项**：
- 保存点可以在事务内嵌套使用
- 释放保存点后无法恢复到该点
- 保存点适用于复杂事务中的部分回滚

### 4. 两阶段提交

两阶段提交提供更严格的原子性保证：

```rust
tx.set_two_phase_commit(true);
```

**优势**：
- 减少提交过程中崩溃导致的数据不一致风险
- 适用于关键业务数据

## 并发模型

### 1. MVCC实现

redb的MVCC实现特点：

- **版本链**：数据修改时创建新版本，旧版本保留供读事务使用
- **垃圾回收**：当没有读事务引用旧版本时自动清理
- **快照隔离**：读事务看到事务开始时的一致性快照

### 2. 锁策略

- **写锁**：写事务获取独占锁，确保同一时间只有一个写事务
- **读锁**：读事务无锁或使用轻量级锁，支持高并发读取

### 3. 事务隔离级别

redb默认提供**快照隔离（Snapshot Isolation）**，具有以下特点：

- 防止脏读（Dirty Read）
- 防止不可重复读（Non-repeatable Read）
- 防止幻读（Phantom Read）

## 错误处理

### 1. 事务错误类型

```rust
pub enum TransactionError {
    Storage(StorageError),
    // ...
}

pub enum CommitError {
    Storage(StorageError),
    // ...
}
```

### 2. I/O错误处理

- **提前检查**：`begin_write()`时检查I/O错误
- **存储失败标记**：检测到存储失败后标记，阻止后续提交
- **自动恢复**：重启后自动检测并恢复不一致状态

### 3. 恢复机制

```rust
let needs_recovery = header.recovery_required || header.layout().len() != storage.raw_file_len()?;
if needs_recovery {
    // 重新计算布局
    header.set_layout(DatabaseLayout::recalculate(...));
    // 选择主副本进行修复
    header.pick_primary_for_repair(repair_info)?;
    // 写入更新后的头部
    storage.write(0, DB_HEADER_SIZE, true)?.mem_mut().copy_from_slice(&header.to_bytes(true));
    storage.flush()?;
}
```

## 最佳实践

### 1. 事务使用建议

- **短事务**：尽量缩短事务生命周期，减少锁持有时间
- **批量操作**：使用`Durability::None`进行批量写入，最后使用`Durability::Immediate`提交
- **保存点**：在复杂事务中使用保存点实现部分回滚

### 2. 性能优化

- **读多写少**：利用MVCC特性，读操作不会阻塞写操作
- **缓存配置**：合理设置缓存大小`set_cache_size()`
- **页大小**：根据数据特征选择合适的页大小`set_page_size()`

### 3. 错误处理

- **始终处理错误**：事务操作可能失败，需要正确处理错误
- **使用`?`操作符**：简化错误传播
- **日志记录**：启用`logging`特性记录事务操作

## 总结

redb通过以下设计实现了完整的ACID保证：

1. **原子性**：事务边界 + 自动回滚 + 两阶段提交
2. **一致性**：B树结构 + 事务隔离 + 约束检查
3. **隔离性**：MVCC + 读写分离 + 快照隔离
4. **持久性**：可配置持久性级别 + 崩溃安全 + 自动恢复

这些特性使redb成为一个可靠、高性能的嵌入式数据库，适用于需要ACID保证的应用场景。
