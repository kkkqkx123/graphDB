# Transaction / Storage / Sync 集成分析报告

## 一、概述

本文档分析 `src/transaction`、`src/storage`、`src/sync` 三个核心模块的集成现状，
识别设计问题，并提出重构方案。

## 二、当前集成架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              GraphDatabase                                  │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────┐  ┌────────────────┐  │
│  │ Storage     │  │ Transaction  │  │ SyncManager   │  │ QueryApi       │  │
│  │ (GraphStorage)│  │ Manager     │  │               │  │                │  │
│  └──────┬──────┘  └──────┬───────┘  └───────┬───────┘  └───────┬────────┘  │
└─────────┼────────────────┼──────────────────┼──────────────────┼───────────┘
          │                │                  │                  │
          ▼                ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Integration Layer                                  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                    interfaces module (抽象层)                          │  │
│  │  • TransactionBuffer  • UndoTarget  • CompactTarget  • RecoveryApplier│  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 各模块职责

| 模块          | 职责                                | 核心组件                                                                     |
| ------------- | ----------------------------------- | ---------------------------------------------------------------------------- |
| `transaction` | 事务生命周期、MVCC、WAL、Undo Log   | TransactionManager, TransactionContext, InsertTransaction, UpdateTransaction |
| `storage`     | 图数据存储、索引、缓存、持久化      | PropertyGraph, GraphStorage, WalManager, TransactionSupport                  |
| `sync`        | 外部索引同步（全文/向量）、批量缓冲 | SyncManager, SyncCoordinator, BatchProcessor                                 |

### 2.2 集成方式

1. **Transaction ↔ Storage**:
   - 通过 `InsertTarget` / `UpdateTarget` / `UndoTarget` traits 交互
   - Storage 直接持有 `TransactionContext` 引用（反向依赖）
   - Storage 内嵌 `TransactionSupport` 管理内部 undo log

2. **Storage ↔ Sync**:
   - 通过 `SyncManager.on_vertex_change_with_txn()` 方法
   - 通过 `TransactionBuffer` trait 缓冲操作

3. **Transaction ↔ Sync**:
   - 通过 `TransactionId` 关联（已迁移到 `core::types`）
   - 通过 `TransactionBuffer` trait 实现两阶段提交

## 三、Core 层共享类型分析

### 3.1 已迁移到 Core 的类型

| 类型                                                     | 位置                              | 状态 |
| -------------------------------------------------------- | --------------------------------- | ---- |
| `Timestamp`, `LabelId`, `EdgeId`, `ColumnId`, `VertexId` | `core::types::storage_ids`        | ✅   |
| `TransactionId`                                          | `core::types::storage_ids`        | ✅   |
| `DurabilityLevel`, `TransactionIsolationLevel`           | `core::types::transaction_config` | ✅   |
| `PropertyValue`                                          | `core::types::property_value`     | ✅   |
| `UndoTarget`, `UndoLogError`, `UndoLogResult`            | `core::types::undo`               | ✅   |
| `WalWriter` trait                                        | `core::wal::traits`               | ✅   |
| `RecoveryApplier` trait                                  | `core::wal::traits`               | ✅   |
| `Lsn`, `WalOpType`, `WalConfig`, `WalHeader` 等          | `core::wal::types`                | ✅   |
| Redo 类型 (`InsertVertexRedo` 等)                        | `core::wal::redo`                 | ✅   |
| `CompactTarget`, `CompactConfig`, `CompactStats`         | `core::types::compact`            | ✅   |
| `VersionManager`                                         | `core::mvcc`                      | ✅   |

### 3.2 Interfaces 层现状

| 子模块                           | 内容                                                  | 状态                                                |
| -------------------------------- | ----------------------------------------------------- | --------------------------------------------------- |
| `interfaces::undo`               | 重导出 `UndoTarget`, `UndoLogEntry`, `UndoLogManager` | ⚠️ `UndoLogEntry`/`UndoLogManager` 仍在 transaction |
| `interfaces::compact`            | 重导出 `CompactTarget` 等                             | ✅                                                  |
| `interfaces::recovery`           | 重导出 `RecoveryApplier`                              | ✅                                                  |
| `interfaces::transaction_buffer` | `TransactionBuffer` trait                             | ✅                                                  |

### 3.3 仍存在的问题

| 问题                         | 描述                                                           | 严重程度 |
| ---------------------------- | -------------------------------------------------------------- | -------- |
| 双重 Undo Log 管理           | `TransactionSupport` 和 `TransactionContext` 各有一套 undo log | 高       |
| Storage 反向依赖 Transaction | `GraphStorageContext` 直接持有 `TransactionContext`            | 高       |
| Interfaces 泄漏实现细节      | `interfaces::undo` 重导出 `UndoLogEntry`/`UndoLogManager`      | 中       |
| 双重 WAL 管理                | `WalManager` 包装 `LocalWalWriter`，存在两套 LSN 追踪          | 中       |
| Sync 事务集成不完整          | 缺乏真正的两阶段提交支持                                       | 中       |

## 四、正确架构设计

### 4.1 目标分层

```
┌─────────────────────────────────────────────────────────────┐
│  api / query / sync / transaction / storage                  │
│  (平级模块，通过 interfaces 交互)                              │
├─────────────────────────────────────────────────────────────┤
│  interfaces (跨模块接口定义)                                   │
│  • UndoTarget  • TransactionBuffer  • CompactTarget          │
│  • RecoveryApplier  • TransactionContextProvider              │
├─────────────────────────────────────────────────────────────┤
│  core (基础类型和核心实现)                                     │
│  • types (共享类型)  • wal (WAL 核心)  • mvcc (版本管理)      │
│  • value (值系统)  • error (错误系统)                         │
└─────────────────────────────────────────────────────────────┘
```

### 4.2 依赖规则

1. **Core 层**：不依赖任何其他模块
2. **Interfaces 层**：只依赖 core 层
3. **业务模块**（transaction/storage/sync/query/api）：
   - 可以依赖 core 层
   - 可以依赖 interfaces 层
   - **不能互相依赖**（通过 interfaces 交互）

### 4.3 当前违反的依赖

```
storage ──→ transaction::context::TransactionContext     (❌ 反向依赖)
storage ──→ transaction::undo_log::UndoLogManager         (❌ 反向依赖)
interfaces ──→ transaction::undo_log::UndoLogEntry        (❌ 泄漏实现)
```

### 4.4 正确的集成方式

```
TransactionManager ──→ TransactionContext
                              │
                              │ (通过 trait 抽象)
                              ▼
                    TransactionContextProvider (interfaces)
                              │
                              ▼
                    GraphStorageContext (只依赖 trait)
```

## 五、重构方案

### Phase 1: 消除 TransactionSupport 重复

**问题**：`GraphStorageContext` 同时持有 `TransactionSupport` 和 `TransactionContext`，
两者都管理 undo log，职责重叠。

**方案**：

1. 移除 `GraphStorageContext.txn_support` 字段
2. 将 `TransactionSupport` 的功能合并到 `TransactionContext`
3. 修改所有使用 `txn_support` 的代码路径

### Phase 2: 抽象 TransactionContext 依赖

**问题**：`GraphStorageContext` 直接依赖 `transaction::context::TransactionContext`。

**方案**：

1. 在 `interfaces` 中定义 `TransactionContextProvider` trait
2. `GraphStorageContext` 只依赖该 trait
3. `TransactionContext` 实现该 trait

### Phase 3: 清理 Interfaces 层

**问题**：`interfaces::undo` 重导出 `UndoLogEntry`/`UndoLogManager` 等实现细节。

**方案**：

1. 移除 `interfaces::undo` 中对 `UndoLogEntry`/`UndoLogManager` 的重导出
2. 调用方直接引用 `transaction::undo_log` 中的具体类型

### Phase 4: 统一 WAL 管理

**问题**：`WalManager` 包装 `LocalWalWriter`，存在两套 LSN 追踪。

**方案**：

1. `WalManager` 作为唯一的外部 WAL 接口
2. 移除对 `LocalWalWriter` 的直接引用
3. 统一 LSN 管理入口

### Phase 5: 完善 Sync 事务集成

**问题**：Sync 模块缺乏真正的两阶段提交支持。

**方案**：

1. 在事务 prepare 阶段缓冲外部索引操作
2. 在事务 commit 阶段执行同步
3. 在事务 rollback 阶段丢弃缓冲
