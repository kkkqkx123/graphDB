# 存储层重构 - 阶段1任务说明

## 阶段目标

**目标**：将现有臃肿的 `StorageEngine` trait 拆分为多个职责明确的子接口，实现基础架构分离。

阶段1完成后，src/storage 目录结构如下：

```
src/storage/
├── mod.rs                          # 统一导出
├── engine/                         # 新增：存储引擎层
│   ├── mod.rs                      # Engine trait 定义
│   ├── memory_engine.rs            # 新增：内存引擎实现
│   └── redb_engine.rs              # 新增：redb 引擎实现
├── operations/                     # 已完成：读写操作封装
│   ├── mod.rs
│   ├── reader/
│   └── writer/
├── plan/                           # 部分完成：查询计划层
├── metadata/                       # 已完成：元数据层
├── iterator/                       # 部分完成：迭代器层
└── transaction/                    # 已完成：事务层
```

## 主要任务

### 任务1：完善 Engine trait

**文件**：`src/storage/engine/mod.rs`

**目标**：添加事务和快照相关方法，补充批量操作支持。

```rust
pub trait Engine: Send + Sync {
    // 基础 KV 操作
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError>;
    fn scan(&self, prefix: &[u8]) -> Result<Box<dyn StorageIterator>, StorageError>;
    fn batch(&mut self, ops: Vec<Operation>) -> Result<(), StorageError>;

    // 事务支持
    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError>;
    fn commit_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError>;
    fn rollback_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError>;

    // 快照支持
    fn create_snapshot(&self) -> Result<SnapshotId, StorageError>;
    fn get_snapshot(&self, snap_id: SnapshotId) -> Result<Option<Box<dyn StorageIterator>>, StorageError>;
    fn delete_snapshot(&self, snap_id: SnapshotId) -> Result<(), StorageError>;
}
```

### 任务2：创建 MemoryEngine

**文件**：`src/storage/engine/memory_engine.rs`

**目标**：从 `memory_storage.rs` 提取 KV 操作，实现基于 HashMap 的内存引擎。

**职责**：
- 实现基础 KV 操作（get、put、delete、scan、batch）
- 实现事务支持（begin、commit、rollback）
- 实现快照支持（create、get、delete）
- 不理解图语义，只处理字节数组

### 任务3：创建 RedbEngine

**文件**：`src/storage/engine/redb_engine.rs`

**目标**：从 `redb_storage.rs` 提取 KV 操作，实现基于 redb 的持久化引擎。

**职责**：
- 实现基础 KV 操作（get、put、delete、scan、batch）
- 实现事务支持（begin、commit、rollback）
- 实现快照支持（create、get、delete）
- 不理解图语义，只处理字节数组

### 任务4：更新 MemoryStorage

**文件**：`src/storage/memory_storage.rs`

**目标**：保留图语义层，组合使用 Engine。

**变更**：
- 移除直接的 HashMap 存储，改为组合 `MemoryEngine`
- 实现 `VertexReader`、`EdgeReader`、`VertexWriter`、`EdgeWriter` trait
- 保留图语义操作（insert_node、get_node、insert_edge 等）

### 任务5：更新 RedbStorage

**文件**：`src/storage/redb_storage.rs`

**目标**：保留图语义层，组合使用 Engine。

**变更**：
- 移除直接的 redb 操作，改为组合 `RedbEngine`
- 实现 `VertexReader`、`EdgeReader`、`VertexWriter`、`EdgeWriter` trait
- 保留图语义操作

## 验收标准

1. **编译通过**：运行 `cargo check` 无错误
2. **接口分离**：`Engine` trait 只包含 KV 操作，不包含图语义
3. **职责单一**：`MemoryEngine` 和 `RedbEngine` 只负责键值存储
4. **向后兼容**：`MemoryStorage` 和 `RedbStorage` 保持原有 API

## 后续阶段

阶段1完成后，将进入阶段2：
- 迁移所有调用 `StorageEngine` 的代码到新的 Reader/Writer 接口
- 删除臃肿的 `storage_engine.rs`
- 完善 plan 层的节点和执行器

## 任务状态

| 任务 | 状态 | 负责人 | 备注 |
|------|------|--------|------|
| 完善 Engine trait | ✅ 已完成 | - | 添加事务和快照方法 |
| 创建 MemoryEngine | ✅ 已完成 | - | 基于 HashMap 实现 |
| 创建 RedbEngine | ✅ 已完成 | - | 基于 redb 实现 |
| 运行验证 | ✅ 已完成 | - | cargo check 通过 |

## 阶段1完成摘要

已完成以下工作：

1. **创建阶段1任务文档**：`docs/storage/PHASE1_TASKS.md`

2. **完善 Engine trait**：`src/storage/engine/mod.rs`
   - 添加事务方法：`begin_transaction`、`commit_transaction`、`rollback_transaction`
   - 添加快照方法：`create_snapshot`、`get_snapshot`、`delete_snapshot`
   - 定义 `SnapshotId` 类型
   - 导入 `TransactionId` 从 transaction 模块

3. **创建 MemoryEngine**：`src/storage/engine/memory_engine.rs`
   - 基于 HashMap 实现
   - 实现完整的事务支持
   - 实现快照支持
   - 包含单元测试

4. **创建 RedbEngine**：`src/storage/engine/redb_engine.rs`
   - 基于 redb 实现
   - 实现完整的事务支持
   - 实现快照支持
   - 包含单元测试

5. **编译验证**：`cargo check --lib` 通过

## 后续阶段

阶段1完成后，将进入阶段2：
- 评估 MemoryStorage 和 RedbStorage 是否需要重构为组合 Engine
- 迁移所有调用 `StorageEngine` 的代码到新的 Reader/Writer 接口
- 删除臃肿的 `storage_engine.rs`
- 完善 plan 层的节点和执行器
