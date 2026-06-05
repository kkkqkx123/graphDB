# storage 层 WAL 集成设计方案

## 1. 背景

当前 `storage` 层已经具备部分 WAL 相关能力，但整体只完成了“恢复”和“checkpoint”闭环，普通写入路径仍然可以绕过 WAL。这样会导致两个问题：

1. 崩溃后只能恢复部分状态，不能保证写入顺序和持久性。
2. `storage` 层内部存在两套 WAL 语义，职责边界不清晰，容易造成 LSN、checkpoint、flush 状态不一致。

本方案的目标是把 WAL 明确为 `storage` 层持久化写入的唯一事实来源，并把它和事务、恢复、checkpoint、flush 串成一条一致的责任链。

---

## 2. WAL 的实际意义

WAL 不是普通日志，它在系统里承担的是“崩溃恢复边界”和“提交顺序边界”。

### 2.1 WAL 解决什么问题

- 保证崩溃后可以按顺序重放最近一次 checkpoint 之后的变更。
- 记录逻辑变更，而不是每次都写全量数据。
- 给 checkpoint 提供可追踪的 LSN 边界。
- 给恢复提供起点、终点和跳过规则。

### 2.2 WAL 不负责什么

- 不替代 flush。
- 不替代快照。
- 不负责查询加速。
- 不负责缓存一致性本身，缓存只是 WAL 保护下的派生状态。

### 2.3 WAL 在本项目中的定位

WAL 应该被视为：

- 写入提交前的持久性屏障。
- checkpoint 的恢复锚点。
- crash recovery 的重放源。
- 逻辑变更的单一审计链。

---

## 3. 当前集成现状

### 3.1 已接通的部分

- `GraphStorage::open()` 会在启动时触发恢复流程。
- `StorageRecoveryOps` 已经在 `GraphStorage`、`MetricsStorage` 上实现并转发。
- `PersistenceCoordinator` 已持有 `WalManager`，并参与 checkpoint 过程。
- `RecoveryManager` 已具备 WAL 解析和重放能力。

### 3.2 仍未接通的部分

- `storage/engine/graph_storage/writer.rs` 中的普通写操作没有先写 WAL。
- `SyncWrapper` 只是同步外部索引，不是 WAL 入口。
- `WalManager::truncate()` 目前只是调整内存中的 LSN，不是物理截断。
- `GraphStoragePersistent` 和 `PersistenceCoordinator` 两边都出现了 WAL 管理概念，边界重叠。

### 3.3 现状判断

当前状态可以概括为：

- 恢复链路是通的。
- checkpoint 链路是通的。
- 普通写入链路没有走 WAL。

因此，当前 `storage` 层的 WAL 集成是“半闭环”，不是完整闭环。

---

## 4. 设计目标

1. 所有持久化写入必须经过 WAL。
2. `PersistenceCoordinator` 成为持久化责任链的统一编排者。
3. `WalManager` 只负责 WAL 生命周期、LSN 和 checkpoint 相关状态。
4. 恢复逻辑只消费 WAL，不再嵌入业务写入逻辑。
5. flush、checkpoint、snapshot 的职责要分离，但彼此可追踪。
6. 事务层 WAL 与 storage 层持久化语义保持一致，不重复造轮子。

---

## 5. 推荐架构

### 5.1 责任链

推荐将持久化责任链固定为：

```text
写入操作
  -> WAL 追加 redo
  -> 内存态修改
  -> flush 到 data 目录
  -> checkpoint 记录一致性边界
  -> snapshot 作为可选的完整备份
```

### 5.2 核心组件职责

| 组件 | 职责 | 说明 |
|---|---|---|
| `WalManager` | WAL 生命周期、LSN、同步、checkpoint seq | 只做 WAL 相关状态，不做业务写入 |
| `PersistenceCoordinator` | flush/checkpoint/snapshot 编排 | 负责协调数据落盘边界 |
| `RecoveryManager` | WAL 解析与 replay | 只消费日志，不创建业务写入 |
| `GraphStorage` | 对外存储入口 | 负责将存储操作路由到正确的持久化路径 |
| `Transaction` 层 | 生成 redo 并提交 | 业务写入的主入口 |

---

## 6. 哪些模块必须使用 WAL

### 6.1 必须使用 WAL 的模块

这些模块会改变可持久化状态，必须进入 WAL：

- 事务写入模块
  - `crates/graphdb-transaction/src/transaction/insert_transaction.rs`
  - `crates/graphdb-transaction/src/transaction/update_transaction.rs`
  - `crates/graphdb-transaction/src/transaction/compact_transaction.rs`
- 存储持久化编排
  - `crates/graphdb-storage/src/storage/engine/persistence_coordinator.rs`
  - `crates/graphdb-storage/src/storage/engine/graph_storage/persistence.rs`
- 恢复入口
  - `crates/graphdb-storage/src/storage/client.rs`
  - `crates/graphdb-storage/src/storage/engine/graph_storage/mod.rs`
- schema/catalog 持久化变更
  - space / tag / edge type / index 的创建、删除、修改
- auth 持久化变更
  - 用户、密码、角色授权相关修改
- 任何直接影响磁盘一致性的元数据变更

### 6.2 通常不需要单独使用 WAL 的模块

这些模块通常只是读取、缓存或派生状态：

- query 执行和读取路径
- cache 层
- index 查询层
- WAL parser / recovery applier 本身
- 可重建的临时派生数据

### 6.3 条件性需要 WAL 的模块

如果某个模块维护的是独立持久化状态，而不是纯派生状态，那么它也必须进 WAL：

- fulltext 同步元数据
- vector 同步元数据
- 任何无法从主数据重建的外部副本状态

判断标准很简单：

> 能否在重启后仅靠主数据和 WAL 完整重建？

如果不能，就必须单独纳入持久化链路。

---

## 7. 现有代码的问题点

### 7.1 写路径绕过 WAL

`storage/engine/graph_storage/writer.rs` 里的 `insert_vertex`、`update_vertex`、`delete_vertex`、`insert_edge` 等接口直接修改内存结构和索引，没有先写 WAL。

这意味着：

- 写入可能已经对外可见，但崩溃后无法回放。
- checkpoint / recovery 只能覆盖一部分状态。

### 7.2 WAL 所有权分散

当前存在两套 WAL 语义：

- `GraphStoragePersistent` 内部有 `wal_manager`
- `PersistenceCoordinator` 内部也有 `wal_manager`

这会导致：

- 状态来源不唯一
- LSN 语义容易分裂
- checkpoint / flush 的边界判断容易出错

### 7.3 `truncate` 语义不完整

`WalManager::truncate()` 现在只是调用 `set_current_lsn()`，而 `LocalWalWriter::set_current_lsn()` 也只是更新内存变量。

这不是物理截断，也不是日志回收。

后果是：

- 逻辑上 checkpoint 已推进
- 物理 WAL 文件却未真正回收
- 恢复语义可能依赖错误的状态判断

### 7.4 恢复逻辑和写入逻辑耦合度不够清晰

`RecoveryManager` 已经具备 replay 能力，但它应只负责“读 WAL + 回放 WAL”。

现在问题不是恢复逻辑不够，而是写入链路没有统一进 WAL。

---

## 8. 推荐的正确集成方式

### 8.1 统一 WAL 入口

建议将 WAL 入口统一到一个地方：

- 若事务层是唯一写入口，则所有写入都必须通过事务层。
- 若 `StorageWriter` 仍要对外开放，则它必须是 WAL-aware 的包装层，而不是直接写内存。

推荐原则：

> 任何会改变持久状态的操作，必须先落 WAL，再落内存，再进入 checkpoint/flush 流程。

### 8.2 单一 WAL 所有权

建议让 `PersistenceCoordinator` 成为唯一的 WAL 编排者，`WalManager` 成为它内部的组成部分。

不建议：

- 一边在 `GraphStoragePersistent` 保存一份 WAL 状态
- 一边在 `PersistenceCoordinator` 再保存一份 WAL 状态

### 8.3 恢复只消费 WAL

恢复阶段的流程应保持为：

1. 读取 schema / index 元数据。
2. 恢复 checkpoint 数据。
3. 从 checkpoint LSN 之后开始回放 WAL。
4. 更新恢复后的 checkpoint / flush 边界状态。

### 8.4 写入必须带 redo

写入必须遵循：

1. 构造 redo。
2. append WAL。
3. 修改内存态。
4. 必要时触发 sync / checkpoint / flush。

如果 redo 不能反映业务变化，这条链路就不完整。

---

## 9. 事务层与 storage 层如何分工

### 9.1 事务层负责什么

事务层负责：

- 生成 redo
- 保证写入顺序
- 写 WAL
- 提交或回滚
- 需要时触发 replay 兼容逻辑

### 9.2 storage 层负责什么

storage 层负责：

- 具体表结构修改
- 内存态和磁盘态映射
- checkpoint / flush / snapshot
- 恢复时的 replay 应用

### 9.3 两层之间的关系

事务层是“写入协议层”，storage 层是“状态执行层”。

正确关系应是：

```text
事务层 redo
  -> WAL
  -> storage 执行
  -> checkpoint / flush
  -> recovery 重放
```

而不是：

```text
storage 直接修改内存
  -> 顺手通知同步器
  -> 再补日志
```

后者会把崩溃恢复能力做成“事后补丁”。

---

## 10. 模块级改造建议

### 10.1 `crates/graphdb-storage/src/storage/engine/wal_manager.rs`

建议：

- 保留 `WalManager`，但明确它只是 WAL 门面。
- 补齐真正的 checkpoint / 回收语义。
- 避免把 `set_current_lsn()` 当成“truncate”。

### 10.2 `crates/graphdb-storage/src/storage/engine/persistence_coordinator.rs`

建议：

- 让它成为唯一的 checkpoint / snapshot / flush 编排中心。
- 所有 WAL 状态都从这里读取和更新。
- 增加清晰的状态机，避免 checkpoint 期间重复推进状态。

### 10.3 `crates/graphdb-storage/src/storage/engine/graph_storage/writer.rs`

建议：

- 不要直接作为对外写入口。
- 如果保留，要么接 WAL adapter，要么只给内部事务使用。

### 10.4 `crates/graphdb-storage/src/storage/engine/graph_storage/persistence.rs`

建议：

- 只做恢复、flush、checkpoint、snapshot 相关逻辑。
- 不要再承担写入编排职责。

### 10.5 `crates/graphdb-storage/src/storage/client.rs`

建议：

- 把恢复能力继续保留为 `StorageRecoveryOps` 的统一入口。
- `init_with_recovery()` 应该成为初始化阶段的标准入口。

### 10.6 `crates/graphdb-transaction/src/transaction/*`

建议：

- 保持事务层作为 WAL redo 的生成者。
- 不要重复实现另一套 storage 专用日志系统。
- 事务 commit 的语义要和 storage recovery replay 语义对齐。

---

## 11. 推荐实施顺序

### Phase 1：统一职责边界

- 明确 `PersistenceCoordinator` 是持久化编排中心。
- 去掉重复 WAL 语义。
- 明确 `WalManager` 的职责边界。

### Phase 2：接通普通写路径

- 让所有 `StorageWriter` 写操作经过 WAL。
- 或者收缩 `StorageWriter` 的对外暴露范围。

### Phase 3：修正 LSN / truncate 语义

- 实现真正的 WAL 回收或段级清理。
- 保证 checkpoint 后的 LSN 与磁盘状态一致。

### Phase 4：完善恢复测试

建议补充以下测试场景：

- WAL 已写，flush 未完成
- WAL 已写，checkpoint 未完成
- checkpoint 已更新，但 WAL 未真正回收
- 恢复后再执行新写入，LSN 连续性正确

### Phase 5：审计所有持久化变更

- schema
- auth
- index metadata
- search / vector 同步元数据

确认这些路径是否都已经纳入 WAL 或能从主 WAL 重建。

---

## 12. 风险点

1. 双写风险
- 如果写入先改内存，再补 WAL，崩溃窗口里会出现不可恢复状态。

2. LSN 漂移风险
- 如果 `current_lsn`、`checkpoint_lsn`、文件实际偏移不同步，恢复判断会失真。

3. 责任重复风险
- 如果 `GraphStorage`、`PersistenceCoordinator`、`Transaction` 都各自维护 WAL 语义，最终会不可维护。

4. 外部同步状态风险
- fulltext / vector / sync 这类派生状态如果不区分“可重建”和“不可重建”，很容易漏进 WAL 范围。

---

## 13. 验收标准

当 WAL 集成正确后，应满足以下条件：

- 任何持久化写操作都有对应 WAL 记录。
- `GraphStorage::open()` 能在崩溃后恢复到最后一个一致点。
- checkpoint 之后的 WAL 可以被准确跳过或回收。
- `needs_recovery()` 的判断与实际磁盘状态一致。
- 恢复后再写入不会破坏 LSN 连续性。
- 事务层和 storage 层对 redo / replay 的理解一致。

---

## 14. 结论

`storage` 层的 WAL 不是“附带功能”，而是持久化写入与崩溃恢复的核心协议。

当前最需要补的是：

1. 统一 WAL 所有权。
2. 让普通写路径真正经过 WAL。
3. 修正 checkpoint / truncate 的实际语义。
4. 让事务层 redo 和 storage 恢复逻辑对齐。

只要这四点完成，WAL 才算真正集成到 `storage` 架构里，而不是只存在于恢复和 checkpoint 的边缘位置。
