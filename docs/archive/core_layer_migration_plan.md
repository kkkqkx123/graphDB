# Core 层迁移方案

## 一、现状分析

### 1.1 当前模块分层

```
┌─────────────────────────────────────────────────────────────┐
│  src/api          (HTTP API / 服务层)                        │
├─────────────────────────────────────────────────────────────┤
│  src/query        (查询引擎)                                 │
├─────────────────────────────────────────────────────────────┤
│  src/sync         (外部索引同步)     src/transaction (事务)   │
├─────────────────────────────────────────────────────────────┤
│  src/storage      (存储引擎)                                 │
├─────────────────────────────────────────────────────────────┤
│  src/interfaces   (跨模块接口 - 部分迁移中)                    │
├─────────────────────────────────────────────────────────────┤
│  src/core         (核心类型 / 值系统 / 错误系统)               │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 当前 core 层已有内容

| 子模块                     | 内容                                                                                                                                | 状态      |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | --------- |
| `core::types::storage_ids` | `Timestamp`, `LabelId`, `EdgeId`, `ColumnId`, `VertexId`, `EdgeKey`, `EdgeIdentifier`, `VertexIdentifier`, `EdgeDeletionContext` 等 | ✅ 已就位 |
| `core::vertex_edge_path`   | `Tag`, `Vertex`, `Edge`, `Path`, `Step`                                                                                             | ✅ 已就位 |
| `core::value`              | `Value` 及所有值类型                                                                                                                | ✅ 已就位 |
| `core::error`              | 统一错误系统 `DBError`, `StorageError`, `QueryError` 等                                                                             | ✅ 已就位 |
| `core::types`              | `DataType`, `PropertyDef`, `SpaceInfo`, `TagInfo`, `EdgeTypeInfo`, `Index` 等 schema 类型                                           | ✅ 已就位 |

### 1.3 当前 interfaces 层已有内容

| 子模块                           | 内容                                                                   | 状态                                |
| -------------------------------- | ---------------------------------------------------------------------- | ----------------------------------- |
| `interfaces::undo`               | 重导出 `UndoTarget`, `UndoLogEntry`, `UndoLogManager`, `PropertyValue` | ⚠️ 仅重导出，实际定义在 transaction |
| `interfaces::compact`            | `CompactTarget`, `CompactConfig`, `CompactStats`                       | ✅ 已就位                           |
| `interfaces::recovery`           | 重导出 `RecoveryApplier`                                               | ⚠️ 仅重导出，实际定义在 transaction |
| `interfaces::transaction_buffer` | `TransactionBuffer` trait                                              | ✅ 已就位                           |

### 1.4 跨模块共享但未在 core/interfaces 的类型

以下是当前被多个模块引用、但定义位置不合理的类型：

| 类型                                   | 当前定义位置                                                                 | 被哪些模块使用                                     | 问题                                             |
| -------------------------------------- | ---------------------------------------------------------------------------- | -------------------------------------------------- | ------------------------------------------------ |
| `TransactionId`                        | `transaction::types`                                                         | sync, storage, transaction                         | sync 不应依赖 transaction 模块                   |
| `TableId`, `TableTracker`, `TableType` | `storage::metadata`                                                          | transaction (WAL checkpoint)                       | transaction 不应依赖 storage 模块                |
| `DurabilityLevel`                      | `transaction::types` 和 `storage::engine::graph_storage::transaction_config` | transaction, storage                               | **重复定义**，两处各有一套                       |
| `IsolationLevel`                       | `transaction::types` 和 `core::types::space`                                 | transaction, storage                               | **重复定义**，两处各有一套                       |
| `VersionManager`                       | `transaction::version_manager`                                               | storage (GraphStorageContext)                      | storage 直接依赖 transaction 内部实现            |
| `TransactionContext`                   | `transaction::context`                                                       | storage (GraphStorageContext, StorageClient trait) | storage 直接依赖 transaction 内部实现            |
| `UndoLogManager`                       | `transaction::undo_log`                                                      | storage (GraphStorageContext, TransactionSupport)  | storage 直接依赖 transaction 内部实现            |
| `PropertyValue`                        | `transaction::undo_log`                                                      | storage (engine::transaction)                      | 已通过 interfaces 重导出，但定义仍在 transaction |
| `WalWriter` trait                      | `transaction::wal::writer`                                                   | storage (PropertyGraph, PersistenceCoordinator)    | storage 直接依赖 transaction WAL                 |
| `Lsn`                                  | `transaction::wal::types`                                                    | storage (PersistenceCoordinator)                   | storage 直接依赖 transaction WAL                 |
| `WalOpType`                            | `transaction::wal::types`                                                    | storage (WalManager)                               | storage 直接依赖 transaction WAL                 |
| `CheckpointManager`                    | `transaction::wal::checkpoint`                                               | storage (PersistenceCoordinator)                   | storage 直接依赖 transaction WAL                 |

### 1.5 当前依赖关系图（问题视角）

```
storage ──→ transaction::context::TransactionContext     (❌ 反向依赖)
storage ──→ transaction::version_manager::VersionManager  (❌ 反向依赖)
storage ──→ transaction::undo_log::UndoLogManager         (❌ 反向依赖)
storage ──→ transaction::wal::*                           (❌ 反向依赖)
transaction ──→ storage::metadata::{TableId, TableTracker} (❌ 反向依赖)
sync ──→ transaction::types::TransactionId                (⚠️ 不必要的依赖)
```

理想的分层应该是：

```
api → query → {sync, transaction, storage} → interfaces → core
```

即所有模块只依赖 `interfaces` 和 `core`，模块之间不互相依赖。

---

## 二、迁移方案

### 2.1 总体策略

分四个阶段执行，每个阶段独立可验证，风险从低到高：

- **Phase 1**：基础类型迁移（低风险，消除类型重复和反向依赖）
- **Phase 2**：Trait 接口迁移（中风险，将 interfaces 层内容整合到 core）
- **Phase 3**：MVCC 核心迁移（高风险，VersionManager 和 TransactionContext）
- **Phase 4**：WAL 接口迁移（高风险，WAL 相关 trait 和基础类型）

### 2.2 Phase 1：基础类型迁移

**目标**：消除 `TransactionId` 和 `TableTracker` 的反向依赖，合并重复定义。

#### 2.2.1 迁移 `TransactionId` 到 `core::types`

**当前状态**：

- 定义在 [transaction/types.rs](file:///d:/项目/database/graphDB/src/transaction/types.rs) 第 12 行：`pub type TransactionId = u64;`
- 被 `sync::manager`, `sync::coordinator`, `sync::batch::processor`, `sync::vector_sync` 引用
- 被 `storage::engine::sync_wrapper` 引用
- 被 `interfaces::transaction_buffer` 引用

**迁移步骤**：

1. 在 `core::types::storage_ids.rs` 添加 `pub type TransactionId = u64;`
2. 在 `core::types::mod.rs` 中重导出
3. 修改 `transaction::types` 为 `pub use crate::core::types::TransactionId;`
4. 修改 `sync` 模块的 import 为 `use crate::core::types::TransactionId;`
5. 修改 `interfaces::transaction_buffer` 的 import

**影响范围**：约 6 个文件，纯 import 路径修改。

#### 2.2.2 迁移 `TableTracker` 系列到 `core::types`

**当前状态**：

- 定义在 [storage/metadata/table_tracker.rs](file:///d:/项目/database/graphDB/src/storage/metadata/table_tracker.rs)
- `TableId`, `TableType`, `TableTracker`, `TableTrackerConfig`
- 被 `transaction::wal::mod.rs` 以别名方式引用：`DirtyPageId = TableId`, `DirtyPageTracker = TableTracker`
- 被 `storage::engine::property_graph` 使用

**迁移步骤**：

1. 在 `core::types` 下新建 `table_tracker.rs`
2. 将 `TableId`, `TableType`, `TableTracker`, `TableTrackerConfig` 移入
3. 在 `core::types::mod.rs` 中重导出
4. 修改 `storage::metadata::table_tracker` 为 `pub use crate::core::types::table_tracker::*;`
5. 修改 `transaction::wal::mod.rs`，移除别名重导出，改为直接 `pub use crate::core::types::{TableId, TableTracker, TableTrackerConfig, TableType};`
6. 删除 `DirtyPageId` / `DirtyPageTracker` 别名（统一使用原名）

**影响范围**：约 5 个文件。

#### 2.2.3 合并重复的 `DurabilityLevel`

**当前状态**：

- `transaction::types` 中定义了一套 `DurabilityLevel`
- `storage::engine::graph_storage::transaction_config` 中定义了另一套 `DurabilityLevel`

**迁移步骤**：

1. 在 `core::types` 下新建 `transaction_config.rs`
2. 定义统一的 `DurabilityLevel` 枚举
3. 两处原有定义改为 `pub use crate::core::types::DurabilityLevel;`
4. 确保两套枚举的变体一致（如不一致需对齐）

**影响范围**：约 3 个文件。

#### 2.2.4 合并重复的 `IsolationLevel`

**当前状态**：

- `transaction::types` 中定义了 `IsolationLevel`（只有 `RepeatableRead`）
- `core::types::space` 中也定义了 `IsolationLevel`（用于 Space 配置）

**分析**：这两个 `IsolationLevel` 语义不同：

- `transaction::types::IsolationLevel` — 事务隔离级别
- `core::types::space::IsolationLevel` — Space 级别的隔离配置

**建议**：保留两个独立定义，但将 `transaction::types::IsolationLevel` 重命名为 `TransactionIsolationLevel` 并移至 core，避免混淆。

#### 2.2.5 迁移 `PropertyValue` 到 `core::types`

**当前状态**：

- 定义在 [transaction/undo_log.rs](file:///d:/项目/database/graphDB/src/transaction/undo_log.rs) 第 42 行
- 被 `storage::engine::transaction` 使用
- 已通过 `interfaces::undo` 重导出

**迁移步骤**：

1. 在 `core::types` 下新建 `property_value.rs`
2. 将 `PropertyValue` 枚举移入
3. 修改 `transaction::undo_log` 为 `pub use crate::core::types::PropertyValue;`
4. 修改 `interfaces::undo` 的重导出路径

**影响范围**：约 4 个文件。

### 2.3 Phase 2：Trait 接口迁移

**目标**：将 `src/interfaces` 中的 trait 定义整合到 `core` 层，使 `interfaces` 成为纯重导出层（或直接废弃）。

#### 2.3.1 迁移 `UndoTarget` trait 到 `core::types`

**当前状态**：

- 定义在 [transaction/undo_log.rs](file:///d:/项目/database/graphDB/src/transaction/undo_log.rs) 第 55 行
- 被 `storage::engine::property_graph` 实现
- 已通过 `interfaces::undo` 重导出

**迁移步骤**：

1. 在 `core::types` 下新建 `undo.rs`
2. 将 `UndoTarget` trait 及 `UndoLogError`, `UndoLogResult` 移入
3. `UndoLogEntry` 枚举和 `UndoLogManager` 保留在 transaction（它们是具体实现）
4. 修改 `transaction::undo_log` 为 `pub use crate::core::types::undo::*;`
5. 修改 `interfaces::undo` 的重导出路径

**注意**：`UndoTarget` trait 的方法签名引用了 `LabelId`, `VertexId`, `Timestamp`, `ColumnId`, `EdgeKey`, `VertexIdentifier`, `EdgeIdentifier`, `EdgeDeletionContext` — 这些已在 `core::types::storage_ids` 中，无循环依赖风险。

#### 2.3.2 迁移 `CompactTarget` trait 到 `core::types`

**当前状态**：

- 定义在 [interfaces/compact.rs](file:///d:/项目/database/graphDB/src/interfaces/compact.rs)
- `CompactConfig`, `CompactStats`, `CompactError`, `CompactTarget`

**迁移步骤**：

1. 在 `core::types` 下新建 `compact.rs`
2. 将全部内容移入
3. 修改 `interfaces::compact` 为 `pub use crate::core::types::compact::*;`

#### 2.3.3 迁移 `RecoveryApplier` trait 到 `core::types`

**当前状态**：

- 定义在 [transaction/wal/recovery.rs](file:///d:/项目/database/graphDB/src/transaction/wal/recovery.rs)
- 被 `storage::engine::property_graph` 实现
- 已通过 `interfaces::recovery` 重导出

**迁移步骤**：

1. 在 `core::types` 下新建 `recovery.rs`
2. 将 `RecoveryApplier` trait 移入
3. `RecoveryManager`, `RecoveryConfig`, `RecoveryStats` 保留在 transaction（它们是具体实现）
4. 修改 `interfaces::recovery` 的重导出路径

#### 2.3.4 迁移 `TransactionBuffer` trait 到 `core::types`

**当前状态**：

- 定义在 [interfaces/transaction_buffer.rs](file:///d:/项目/database/graphDB/src/interfaces/transaction_buffer.rs)
- 被 `sync::batch` 实现

**迁移步骤**：

1. 在 `core::types` 下新建 `transaction_buffer.rs`
2. 将 trait 定义移入
3. 修改 `interfaces::transaction_buffer` 为 `pub use crate::core::types::transaction_buffer::*;`

**注意**：此 trait 引用了 `sync::external_index::IndexOperation` 和 `sync::batch::error::BatchResult`。迁移时需要将这些类型也提升到 core，或者修改 trait 使用更泛化的类型参数。

### 2.4 Phase 3：MVCC 核心迁移

**目标**：将 `VersionManager` 和 `TransactionContext` 移到 core，使 storage 不再直接依赖 transaction。

#### 2.4.1 迁移 `VersionManager` 到 `core::mvcc`

**当前状态**：

- 定义在 [transaction/version_manager.rs](file:///d:/项目/database/graphDB/src/transaction/version_manager.rs)
- 被 `storage::engine::graph_storage::context` 直接创建实例
- 被 `storage::index::secondary::index_gc_manager` 使用
- 被 `transaction::manager` 创建实例

**问题**：`TransactionManager` 和 `GraphStorageContext` 各自创建独立的 `VersionManager` 实例，导致时间戳不同步。

**迁移步骤**：

1. 在 `core` 下新建 `mvcc` 模块：`core::mvcc`
2. 将 `VersionManager`, `VersionManagerConfig`, `VersionManagerError` 移入
3. 修改 `transaction::version_manager` 为 `pub use crate::core::mvcc::*;`
4. 修改 `storage` 和 `transaction` 的 import 路径

**额外收益**：迁移后可以确保全局只有一个 `VersionManager` 实例（通过 `Arc<VersionManager>` 共享），解决时间戳不同步问题。

#### 2.4.2 迁移 `TransactionContext` 到 `core::transaction`

**当前状态**：

- 定义在 [transaction/context.rs](file:///d:/项目/database/graphDB/src/transaction/context.rs)
- 被 `storage::engine::graph_storage::context` 持有
- 被 `storage::interface::storage_client` trait 的默认方法引用

**分析**：`TransactionContext` 包含大量 transaction 特有的逻辑（savepoint、operation log、状态转换）。直接迁移整个结构体会导致 core 层过重。

**建议**：采用 **trait 抽象** 方式：

1. 在 `core::transaction` 定义 `TransactionContext` trait：
   ```rust
   pub trait TransactionContext: Send + Sync {
       fn id(&self) -> TransactionId;
       fn timestamp(&self) -> Timestamp;
       fn is_read_only(&self) -> bool;
       fn state(&self) -> TransactionState;
   }
   ```
2. `transaction::context::TransactionContext` 实现该 trait
3. `storage` 模块依赖 trait 而非具体类型

**或者**：将 `TransactionContext` 的**数据部分**（id, timestamp, state, read_only）提取到 core，将**行为部分**（savepoint, operation log）保留在 transaction。

### 2.5 Phase 4：WAL 接口迁移

**目标**：将 WAL 相关的 trait 和基础类型移到 core，使 storage 不直接依赖 transaction WAL。

#### 2.5.1 迁移 `WalWriter` trait 到 `core::wal`

**当前状态**：

- 定义在 [transaction/wal/writer/traits.rs](file:///d:/项目/database/graphDB/src/transaction/wal/writer/traits.rs)
- 被 `storage::engine::property_graph` 和 `storage::engine::wal_manager` 使用

**迁移步骤**：

1. 在 `core` 下新建 `wal` 模块：`core::wal`
2. 将 `WalWriter` trait 移入
3. 具体实现（`LocalWalWriter`, `DummyWalWriter` 等）保留在 transaction

#### 2.5.2 迁移 WAL 基础类型到 `core::wal`

**迁移内容**：

- `Lsn` — 日志序列号
- `WalOpType` — WAL 操作类型枚举
- `WalHeader` — WAL 条目头部
- `WalError`, `WalResult` — WAL 错误类型
- `WalConfig` — WAL 配置

**保留在 transaction 的内容**：

- `LocalWalWriter`, `DummyWalWriter` — 具体实现
- `LocalWalParser`, `ParallelWalParser` — 具体实现
- `CheckpointManager` — 具体实现
- `RecoveryManager` — 具体实现
- Redo 类型（`InsertVertexRedo` 等）— 具体实现

---

## 三、迁移后的目标架构

```
┌─────────────────────────────────────────────────────────────┐
│  src/api          (HTTP API / 服务层)                        │
├─────────────────────────────────────────────────────────────┤
│  src/query        (查询引擎)                                 │
├─────────────────────────────────────────────────────────────┤
│  src/sync         (外部索引同步)                              │
│  src/transaction  (事务管理 - 具体实现)                       │
│  src/storage      (存储引擎 - 具体实现)                       │
├─────────────────────────────────────────────────────────────┤
│  src/interfaces   (跨模块接口 - 纯重导出层，可逐步废弃)        │
├─────────────────────────────────────────────────────────────┤
│  src/core                                                      │
│  ├── types/         (所有共享类型定义)                          │
│  │   ├── storage_ids.rs    (Timestamp, LabelId, VertexId...)  │
│  │   ├── table_tracker.rs  (TableId, TableTracker...)         │
│  │   ├── transaction_config.rs (DurabilityLevel...)           │
│  │   ├── property_value.rs (PropertyValue)                    │
│  │   ├── undo.rs           (UndoTarget trait)                 │
│  │   ├── compact.rs        (CompactTarget trait)              │
│  │   ├── recovery.rs       (RecoveryApplier trait)            │
│  │   └── transaction_buffer.rs (TransactionBuffer trait)     │
│  ├── mvcc/          (MVCC 版本管理)                            │
│  │   └── version_manager.rs                                   │
│  ├── wal/           (WAL 接口和基础类型)                        │
│  │   ├── traits.rs          (WalWriter trait)                 │
│  │   ├── types.rs           (Lsn, WalOpType, WalHeader...)    │
│  │   └── error.rs           (WalError)                        │
│  ├── transaction/   (事务上下文 trait)                          │
│  │   └── context.rs         (TransactionContext trait)        │
│  ├── value/         (值系统)                                   │
│  ├── error/         (统一错误系统)                              │
│  └── vertex_edge_path.rs (Vertex, Edge, Path)                 │
└─────────────────────────────────────────────────────────────┘
```

**依赖规则**：

- `core` 不依赖任何其他模块（零外部依赖）
- `interfaces` 仅重导出 `core` 中的类型（可逐步废弃）
- `transaction`, `storage`, `sync` 之间**不直接相互依赖**，仅依赖 `core`
- 跨模块通信通过 `core` 中定义的 trait 进行

---

## 四、实施优先级与风险评估

| 阶段      | 内容                           | 风险  | 收益                          | 预计影响文件数 |
| --------- | ------------------------------ | ----- | ----------------------------- | -------------- |
| Phase 1.1 | 迁移 `TransactionId`           | 🟢 低 | 消除 sync→transaction 依赖    | ~6             |
| Phase 1.2 | 迁移 `TableTracker` 系列       | 🟢 低 | 消除 transaction→storage 依赖 | ~5             |
| Phase 1.3 | 合并 `DurabilityLevel`         | 🟢 低 | 消除重复定义                  | ~3             |
| Phase 1.4 | 合并 `IsolationLevel`          | 🟢 低 | 消除命名混淆                  | ~3             |
| Phase 1.5 | 迁移 `PropertyValue`           | 🟢 低 | 消除 storage→transaction 依赖 | ~4             |
| Phase 2.1 | 迁移 `UndoTarget` trait        | 🟡 中 | 接口与实现分离                | ~5             |
| Phase 2.2 | 迁移 `CompactTarget` trait     | 🟢 低 | 已在 interfaces，仅换位置     | ~3             |
| Phase 2.3 | 迁移 `RecoveryApplier` trait   | 🟡 中 | 接口与实现分离                | ~4             |
| Phase 2.4 | 迁移 `TransactionBuffer` trait | 🟡 中 | 需同步迁移依赖类型            | ~5             |
| Phase 3.1 | 迁移 `VersionManager`          | 🔴 高 | 解决时间戳不同步问题          | ~8             |
| Phase 3.2 | 抽象 `TransactionContext`      | 🔴 高 | 解耦 storage 和 transaction   | ~10            |
| Phase 4.1 | 迁移 `WalWriter` trait         | 🔴 高 | 解耦 storage 和 WAL 实现      | ~8             |
| Phase 4.2 | 迁移 WAL 基础类型              | 🔴 高 | 解耦 storage 和 WAL 实现      | ~10            |

---

## 五、建议执行顺序

1. **立即执行 Phase 1**（1-2 天）：风险最低，收益明确，消除所有反向依赖
2. **随后执行 Phase 2**（2-3 天）：将 interfaces 层内容正式归入 core
3. **评估后执行 Phase 3**（3-5 天）：需要仔细设计 trait 抽象，但收益最大（解决 VersionManager 不同步问题）
4. **最后执行 Phase 4**（3-5 天）：WAL 接口迁移涉及较多文件，需要充分测试

每个 Phase 完成后运行 `cargo clippy --all-targets --all-features` 和 `cargo test` 确保无回归。
