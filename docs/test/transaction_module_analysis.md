# GraphDB 事务模块功能分析与集成测试设计

## 1. 事务模块概述

GraphDB 事务模块负责管理数据库事务的生命周期，提供 ACID 特性支持。该模块基于 redb 存储引擎构建，实现了单节点环境下的并发控制和数据一致性保障。

### 1.1 模块结构

```
src/transaction/
├── mod.rs              # 模块入口，提供便捷函数
├── types.rs            # 类型定义（状态、配置、错误等）
├── manager.rs          # 事务管理器（TransactionManager）
├── manager_test.rs     # 事务管理器单元测试
├── context.rs          # 事务上下文（TransactionContext）
├── context_test.rs     # 事务上下文单元测试
└── index_buffer.rs     # 索引更新缓冲区
```

### 1.2 核心组件关系

```
┌─────────────────────────────────────────────────────────────┐
│                    TransactionManager                       │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  - 事务生命周期管理（begin/commit/abort）              │  │
│  │  - 并发控制（写事务互斥）                              │  │
│  │  - 活跃事务追踪（DashMap<txn_id, context>）            │  │
│  │  - 统计信息收集                                        │  │
│  │  - 与 SyncManager 集成（全文索引同步）                 │  │
│  └───────────────────────────────────────────────────────┘  │
│                          │                                  │
│                          ▼                                  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │              TransactionContext                       │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │ 状态管理：Active → Committing → Committed       │  │  │
│  │  │            Active → Aborting → Aborted          │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │ 超时控制：事务超时、查询超时、语句超时、空闲超时 │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │ 操作日志：Insert/Update/Delete Vertex/Edge      │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │ Savepoint：创建、释放、回滚到保存点             │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │ redb 事务封装：WriteTransaction/ReadTransaction │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. 功能详细分析

### 2.1 事务类型

| 类型 | 特性 | 使用场景 |
|------|------|----------|
| **读写事务** | 独占式访问，同一时间只能有一个活跃 | INSERT, UPDATE, DELETE |
| **只读事务** | 支持并发，多个只读事务可同时存在 | SELECT, MATCH 查询 |

### 2.2 事务状态机

```
                    ┌─────────────┐
         ┌─────────│   Active    │◄────────┐
         │         └──────┬──────┘         │
         │                │                │
   begin │                │ begin          │ abort (rollback)
         │        ┌───────▼────────┐       │
         │        │   Committing   │       │
         │        └───────┬────────┘       │
         │                │                │
         │                │ commit         │
         │                ▼                │
         │         ┌─────────────┐         │
         └────────►│  Committed  │         │
                   └─────────────┘         │
                                           │
                   ┌─────────────┐         │
                   │   Aborted   │◄────────┘
                   └─────────────┘
```

### 2.3 配置选项

#### TransactionOptions（事务级配置）

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `timeout` | `Option<Duration>` | `None` | 事务总超时时间 |
| `read_only` | `bool` | `false` | 是否为只读事务 |
| `durability` | `DurabilityLevel` | `Immediate` | 持久化级别 |
| `isolation_level` | `IsolationLevel` | `RepeatableRead` | 隔离级别 |
| `query_timeout` | `Option<Duration>` | `None` | 单次查询超时 |
| `statement_timeout` | `Option<Duration>` | `None` | 单条语句超时 |
| `idle_timeout` | `Option<Duration>` | `None` | 空闲超时 |
| `two_phase_commit` | `bool` | `false` | 是否启用两阶段提交 |

#### TransactionManagerConfig（管理器级配置）

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `default_timeout` | `Duration` | 30s | 默认事务超时 |
| `max_concurrent_transactions` | `usize` | 1000 | 最大并发事务数 |
| `auto_cleanup` | `bool` | `true` | 自动清理过期事务 |

### 2.4 错误类型

```rust
pub enum TransactionError {
    BeginFailed(String),           // 事务开始失败
    CommitFailed(String),          // 提交失败
    AbortFailed(String),           // 中止失败
    TransactionNotFound(TransactionId),  // 事务不存在
    InvalidStateTransition { from, to }, // 无效状态转换
    TransactionTimeout,            // 事务超时
    TransactionExpired,            // 事务已过期
    TooManyTransactions,           // 并发事务过多
    WriteTransactionConflict,      // 写事务冲突
    SyncFailed(String),            // 索引同步失败
    // ... 其他错误
}
```

### 2.5 操作日志类型

```rust
pub enum OperationLog {
    InsertVertex { space, vertex_id, previous_state },
    UpdateVertex { space, vertex_id, previous_data },
    DeleteVertex { space, vertex_id, vertex },
    InsertEdge { space, edge_id, previous_state },
    UpdateEdge { space, edge_id, previous_data },
    DeleteEdge { space, edge_id, edge },
}
```

### 2.6 Savepoint 功能

Savepoint（保存点）允许在事务内部设置回滚点：

```rust
// 创建保存点
let sp_id = manager.create_savepoint(txn_id, Some("after_insert".to_string()))?;

// 回滚到保存点
manager.rollback_to_savepoint(txn_id, sp_id)?;

// 释放保存点
manager.release_savepoint(txn_id, sp_id)?;
```

---

## 3. 现有单元测试覆盖

### 3.1 manager_test.rs 测试项

| 测试函数 | 测试内容 |
|----------|----------|
| `test_transaction_manager_creation` | 管理器创建和配置验证 |
| `test_begin_write_transaction` | 写事务开始 |
| `test_begin_readonly_transaction` | 只读事务开始 |
| `test_begin_transaction_with_timeout` | 带超时的事务 |
| `test_commit_transaction` | 事务提交 |
| `test_abort_transaction` | 事务中止 |
| `test_commit_readonly_transaction` | 只读事务提交 |
| `test_get_transaction_not_found` | 获取不存在的事务 |
| `test_commit_transaction_not_found` | 提交不存在的事务 |
| `test_abort_transaction_not_found` | 中止不存在的事务 |
| `test_commit_already_committed_transaction` | 重复提交 |
| `test_abort_already_aborted_transaction` | 重复中止 |
| `test_write_transaction_conflict` | 写事务冲突检测 |
| `test_multiple_readonly_transactions` | 多并发只读事务 |
| `test_sequential_write_transactions` | 顺序写事务 |
| `test_transaction_timeout` | 事务超时处理 |
| `test_list_active_transactions` | 列出活跃事务 |
| `test_get_transaction_info` | 获取事务信息 |
| `test_max_concurrent_transactions` | 最大并发限制 |
| `test_transaction_stats` | 统计信息验证 |
| `test_cleanup_expired_transactions` | 过期事务清理 |
| `test_shutdown_manager` | 管理器关闭 |

### 3.2 context_test.rs 测试项

| 测试函数 | 测试内容 |
|----------|----------|
| `test_transaction_context_writable_creation` | 写事务上下文创建 |
| `test_transaction_context_readonly_creation` | 只读事务上下文创建 |
| `test_transaction_context_state_transitions` | 状态转换 |
| `test_transaction_context_invalid_state_transition` | 无效状态转换 |
| `test_transaction_context_timeout` | 超时检测 |
| `test_transaction_context_remaining_time` | 剩余时间计算 |
| `test_transaction_context_modified_tables` | 修改表记录 |
| `test_transaction_context_operation_log` | 操作日志管理 |
| `test_transaction_context_can_execute` | 执行能力检查 |
| `test_transaction_context_can_execute_expired` | 过期后执行检查 |
| `test_transaction_context_info` | 事务信息获取 |
| `test_transaction_context_take_write_txn` | 获取写事务 |
| `test_transaction_context_readonly_take_write_txn` | 只读事务获取写事务（应失败） |
| `test_transaction_context_with_write_txn` | 使用写事务执行操作 |
| `test_transaction_context_readonly_with_write_txn` | 只读事务使用写事务（应失败） |

---

## 4. 集成测试设计方案

### 4.1 测试目标

集成测试需要验证事务模块与以下组件的协同工作：

1. **Storage 层** - 事务与存储引擎的交互
2. **Query 层** - 查询执行与事务的集成
3. **Index 层** - 索引更新与事务一致性
4. **Sync 层** - 全文索引同步与事务

### 4.2 集成测试用例设计

#### 4.2.1 基础事务生命周期测试

```rust
#[test]
fn test_transaction_basic_lifecycle() {
    TestScenario::new()
        .expect("创建测试场景失败")
        // 创建图空间和Schema
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        // 开始事务并执行DML
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .assert_success()
        // 验证数据可见性
        .query("MATCH (v:Person) WHERE id(v) == 1 RETURN v")
        .assert_result_count(1)
        // 提交后验证数据持久化
        .assert_vertex_exists(1, "Person")
        .assert_vertex_props(1, "Person", hashmap! {
            "name" => Value::String("Alice".into()),
            "age" => Value::Int(30)
        });
}
```

#### 4.2.2 事务回滚测试

```rust
#[test]
fn test_transaction_rollback() {
    TestScenario::new()
        .expect("创建测试场景失败")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        // 插入初始数据
        .exec_dml("INSERT VERTEX Person(name) VALUES 1:('Before')")
        .assert_success()
        // 开始新事务，插入数据后回滚
        .exec_dml("INSERT VERTEX Person(name) VALUES 2:('RollbackTest')")
        .assert_success()
        // 模拟回滚（通过abort）
        // 验证回滚后数据不存在
        .query("MATCH (v:Person) WHERE id(v) == 2 RETURN v")
        .assert_result_empty();
}
```

#### 4.2.3 并发事务隔离测试

```rust
#[tokio::test]
async fn test_concurrent_transaction_isolation() {
    // 测试场景：
    // 1. 事务A开始并读取数据
    // 2. 事务B修改同一数据并提交
    // 3. 事务A再次读取，应看到旧数据（Repeatable Read）
    // 4. 事务A提交后，新事务应看到新数据
}
```

#### 4.2.4 写事务互斥测试

```rust
#[tokio::test]
async fn test_write_transaction_exclusivity() {
    // 测试场景：
    // 1. 事务A（写）开始
    // 2. 事务B（写）尝试开始，应失败
    // 3. 事务A提交
    // 4. 事务B可以正常开始
}
```

#### 4.2.5 事务超时测试

```rust
#[tokio::test]
async fn test_transaction_timeout_handling() {
    // 测试场景：
    // 1. 创建短超时事务（如100ms）
    // 2. 等待超时
    // 3. 尝试提交，应返回 TransactionTimeout 错误
    // 4. 验证事务已被清理
}
```

#### 4.2.6 Savepoint 集成测试

```rust
#[test]
fn test_savepoint_integration() {
    TestScenario::new()
        .expect("创建测试场景失败")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Account(id INT, balance INT)")
        .assert_success()
        // 创建账户
        .exec_dml("INSERT VERTEX Account(id, balance) VALUES 1:(1, 100)")
        .assert_success()
        // 创建保存点
        // 执行一系列操作
        // 回滚到保存点
        // 验证数据状态
        ;
}
```

#### 4.2.7 事务与索引同步测试

```rust
#[tokio::test]
async fn test_transaction_with_fulltext_sync() {
    // 测试场景：
    // 1. 创建全文索引
    // 2. 在事务中插入带索引数据
    // 3. 提交事务
    // 4. 验证全文索引已更新
    // 5. 回滚事务时验证索引未更新
}
```

#### 4.2.8 批量操作事务测试

```rust
#[test]
fn test_batch_operations_in_transaction() {
    TestScenario::new()
        .expect("创建测试场景失败")
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING)")
        .assert_success()
        // 批量插入
        .exec_dml("INSERT VERTEX Person(name) VALUES \
            1:('Alice'), 2:('Bob'), 3:('Charlie'), \
            4:('David'), 5:('Eve')")
        .assert_success()
        // 验证批量插入结果
        .query("MATCH (v:Person) RETURN count(v) as count")
        .assert_result_contains(vec![Value::Int(5)]);
}
```

#### 4.2.9 事务统计信息测试

```rust
#[test]
fn test_transaction_statistics() {
    // 测试场景：
    // 1. 获取初始统计
    // 2. 执行若干提交和回滚操作
    // 3. 验证统计信息正确更新
}
```

#### 4.2.10 故障恢复测试

```rust
#[test]
fn test_transaction_recovery() {
    // 测试场景：
    // 1. 开始事务并执行操作
    // 2. 模拟崩溃（如进程终止）
    // 3. 重启后验证数据一致性
    // 4. 未提交事务的操作应被回滚
}
```

### 4.3 测试文件组织建议

```
tests/
├── common/
│   ├── mod.rs
│   ├── test_scenario.rs      # 需要扩展事务相关API
│   └── transaction_helpers.rs # 新增：事务测试辅助函数
├── integration_transaction.rs  # 新增：事务集成测试主文件
└── integration_transaction_advanced.rs # 新增：高级事务测试
```

### 4.4 TestScenario 扩展建议

为支持事务集成测试，建议扩展 `TestScenario`：

```rust
impl TestScenario {
    /// 开始显式事务
    pub fn begin_transaction(mut self, options: TransactionOptions) -> Self;
    
    /// 提交当前事务
    pub fn commit_transaction(mut self) -> Self;
    
    /// 回滚当前事务
    pub fn rollback_transaction(mut self) -> Self;
    
    /// 创建保存点
    pub fn create_savepoint(mut self, name: &str) -> Self;
    
    /// 回滚到保存点
    pub fn rollback_to_savepoint(mut self, name: &str) -> Self;
    
    /// 断言事务活跃
    pub fn assert_transaction_active(&self) -> &Self;
    
    /// 断言事务已提交
    pub fn assert_transaction_committed(&self) -> &Self;
    
    /// 断言事务已回滚
    pub fn assert_transaction_rolled_back(&self) -> &Self;
}
```

---

## 5. 测试优先级建议

| 优先级 | 测试类别 | 说明 |
|--------|----------|------|
| P0 | 基础生命周期 | begin/commit/abort 基本功能 |
| P0 | 数据一致性 | 提交后数据持久化，回滚后数据恢复 |
| P1 | 并发控制 | 写事务互斥、读写并发 |
| P1 | 超时处理 | 事务超时自动清理 |
| P2 | Savepoint | 保存点创建和回滚 |
| P2 | 索引同步 | 事务与全文索引集成 |
| P3 | 统计监控 | 事务统计信息 |
| P3 | 故障恢复 | 崩溃恢复测试 |

---

## 6. 总结

GraphDB 事务模块提供了完整的事务管理功能，包括：

1. **生命周期管理** - 开始、提交、中止事务
2. **并发控制** - 写事务互斥、多读取并发
3. **超时管理** - 多级超时检测
4. **状态追踪** - 完整的状态机实现
5. **操作日志** - 支持回滚操作
6. **Savepoint** - 细粒度回滚控制
7. **统计监控** - 事务统计信息
8. **索引集成** - 与全文索引同步

集成测试应重点验证事务与存储层、查询层的协同工作，确保 ACID 特性在真实场景下的正确性。
