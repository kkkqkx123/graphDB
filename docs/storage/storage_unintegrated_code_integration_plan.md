# storage 未集成代码现状整理与后续集成计划

## 当前基线

`graphdb-storage` 当前的主入口仍应保持为：

```text
StorageClient/StorageWriter/StorageReader
        ↓
GraphStorage
        ↓
PropertyGraph
        ↓
VertexTable / EdgeTable / IndexDataManager / PersistenceCoordinator
```

这个方向是正确的。需要避免的问题不是“底层能力太多”，而是已经实现的能力绕过了 `GraphStorage` 的 schema、timestamp、index、metrics、sync、事务和持久化生命周期，形成第二套入口或半接入状态。

当前验证结果：

- `cargo check -p graphdb-storage --all-targets` 可以通过。
- 仍有大量 unused/dead_code warning：lib 约 140 个 warning，lib test 约 104 个 warning，其中部分重复。
- 不应通过 `#[allow(dead_code)]` 批量隐藏这些 warning。它们现在是判断哪些能力没有完整生命周期的信号。

## 已完成整理

### MetricsStorage 接入

`MetricsStorage` 已作为 `StorageClient` 装饰器接入 server 启动链路：

```text
GraphStorage -> MetricsStorage -> SyncWrapper
```

该顺序合理：

- `GraphStorage` 保持核心存储语义。
- `MetricsStorage` 记录所有读写操作指标。
- `SyncWrapper` 继续负责 fulltext/vector 同步包装。

后续只需要补充启动链路测试，不需要再引入新的 metrics 入口。

### 批量写入口收敛

已删除未接入生命周期的底层批处理模块：

- `crates/graphdb-storage/src/storage/engine/batch.rs`

当前批量写统一由 `GraphStorage` 的高层入口承接：

- `batch_insert_vertices`
- `batch_insert_edges`

已完成的语义：

- 批内使用同一个 write timestamp。
- 批量写入前先做 space/tag/edge type/schema 预校验。
- 插入失败时反向回滚已写入 vertex/edge。
- 回滚同时清理对应 index。
- 不再暴露会绕过 schema、index、timestamp、metrics、sync 的底层 batch writer。

仍需注意：

- 当前实现是生命周期正确优先，不是极限性能优先。
- 后续如果要做真正的底层批量化，应在 `GraphStorage` 内部增加分组写 helper，而不是重新暴露 `VertexTable`/`EdgeTable` writer。

### storage 内部事务配置删除

已删除未生效的 storage 内部事务配置：

- `crates/graphdb-storage/src/storage/engine/graph_storage/transaction_config.rs`

事务超时、并发、隔离等配置应继续由 `graphdb-config`、`graphdb-api` 和 transaction crate 负责。storage 侧只保留真正属于存储的语义，例如 WAL、recovery、checkpoint、undo/persistence 相关配置。

### schema 辅助类型删除

已删除未接入当前 schema/layout 生命周期的类型：

- `FieldDef`
- `GeoShape`
- `ColumnDef`

`StoragePropertyDef` 保留，因为它已经被 vertex/edge schema 构建路径实际使用。

删除这些类型后，不再需要 `GeoShape` 上的 `#[allow(dead_code)]`。如果未来要做 binary row layout 或 GIS schema，应重新在明确的 schema/layout 模块中设计，而不是把临时类型放回 `storage_types.rs`。

## 当前剩余问题分组

### 1. 列编码与物理压缩仍未进入生命周期

相关位置：

- `crates/graphdb-storage/src/storage/compression.rs`
- `crates/graphdb-storage/src/storage/engine/config.rs`
- `crates/graphdb-storage/src/storage/vertex/encoding/`
- `crates/graphdb-storage/src/storage/vertex/column_store.rs`
- `crates/graphdb-storage/src/storage/edge/property_table.rs`

现状：

- `CompressionType` 已出现在配置中，但 flush/load/checkpoint 文件格式没有真实使用它。
- dictionary、RLE、bitpacking、FSST、ALP 等列编码能力已经实现。
- `ColumnStore::auto_apply_encodings` 和 `PropertyTable::auto_apply_encodings` 存在，但没有接入 compact/freeze/flush/load。

主要风险：

- 在普通写路径即时压缩会影响更新、回滚、MVCC 可见性和写放大。
- 只压缩 dump bytes，但不持久化 encoding metadata，会导致 load 无法恢复。
- 列编码和物理文件压缩如果共用一个配置语义，会让用户无法判断压缩发生在哪一层。

处理方向：

1. 先区分两类压缩：
   - 列编码：作用在列数据结构上，应接入 compact/freeze。
   - 物理压缩：作用在持久化 payload 上，应接入 flush/load/checkpoint。
2. 先做列编码生命周期：
   - compact 时选择编码。
   - 持久化 encoding metadata。
   - load 后按 metadata 恢复。
   - 增加 compact 前后读取一致性测试。
3. 再做物理压缩：
   - 文件 header 记录压缩类型、版本和校验信息。
   - `CompressionType::None` 和压缩模式都要可恢复。

### 2. 不可变 CSR 仍没有作为 edge 稳定段参与读写

相关位置：

- `crates/graphdb-storage/src/storage/edge/csr.rs`
- `crates/graphdb-storage/src/storage/edge/csr_trait.rs`
- `crates/graphdb-storage/src/storage/edge/mutable_csr.rs`
- `crates/graphdb-storage/src/storage/edge/single_mutable_csr.rs`
- `crates/graphdb-storage/src/storage/edge/mutable_csr_variant.rs`
- `crates/graphdb-storage/src/storage/edge/edge_table.rs`

现状：

- 当前 edge 写路径主要依赖 `MutableCsrVariant`。
- `Csr` 和 `ImmutableNbr` 具备读优化结构和 dump/load 能力，但没有成为 `EdgeTable` 的 base segment。
- CSR trait 中存在多组未使用方法，说明抽象面比实际调用链更宽。

主要风险：

- 直接用 immutable CSR 替换 mutable CSR 会破坏写入、删除、回滚和 MVCC。
- 只在 compact 中生成 immutable CSR，但读路径不合并 mutable delta，会产生漏读或重复读。
- 删除如果没有 tombstone/delta delete 表达，immutable segment 中旧边会错误可见。

处理方向：

1. `EdgeTable` 改成 immutable base segment + mutable delta。
2. 普通写入只进入 mutable delta。
3. compact/freeze 将稳定 delta 转成 immutable CSR segment。
4. 读路径统一合并 immutable base 和 mutable delta。
5. 删除通过 tombstone 或 delta delete 表达。
6. flush/load 持久化 segment metadata。

### 3. index key compression 与 index 宽 API 尚未收敛

相关位置：

- `crates/graphdb-storage/src/storage/index/key_codec/compression.rs`
- `crates/graphdb-storage/src/storage/index/key_codec/`
- `crates/graphdb-storage/src/storage/index/generic_index_manager.rs`
- `crates/graphdb-storage/src/storage/index/index_data_manager.rs`
- `crates/graphdb-storage/src/storage/index/vertex_index_manager.rs`
- `crates/graphdb-storage/src/storage/index/edge_index_manager.rs`
- `crates/graphdb-storage/src/storage/index/index_updater.rs`

现状：

- vertex/edge index 主写链路已由 `GraphStorage` 调用 `PropertyGraph` 的 MVCC index update/delete。
- key builder/parser/generator 已经参与 key 构造。
- key compression、generic index manager、index updater、undo/stats 等能力仍有较多未使用项。

主要风险：

- 如果只压缩写入 key，但 lookup/rebuild/delete 没有同一个 compressor 状态，会查不到数据。
- dictionary/prefix compressor 需要训练结果，必须持久化。
- live BTree key 压缩可能破坏 range lookup 和排序语义。
- `IndexUpdater` 与当前 `GraphStorage` writer 内部 helper 有重叠，可能形成第二套索引维护入口。

处理方向：

1. 明确 index 主入口：
   - 保留 `GraphStorage -> PropertyGraph -> IndexDataManager`。
   - 不再新增上层可绕过 `GraphStorage` 的 index updater。
2. 先在 rebuild/compact 阶段训练 compressor。
3. 持久化 compressor metadata。
4. update/delete/lookup/rebuild 统一走同一 codec。
5. 优先做持久化冷数据压缩，再评估 live index key 压缩。
6. 对 `IndexUpdater`、generic manager、undo/stats 做二选一：
   - 若接入主链路，必须有调用路径和测试。
   - 若不接入，应删除或降级为测试辅助。

### 4. transaction helper 仍与当前写路径重叠

相关位置：

- `crates/graphdb-storage/src/storage/engine/transaction/transactional.rs`
- `crates/graphdb-storage/src/storage/engine/transaction/ops.rs`
- `crates/graphdb-storage/src/storage/engine/transaction/targets/`
- `crates/graphdb-storage/src/storage/engine/graph_storage/writer.rs`

现状：

- storage 内部事务配置已经删除。
- `transactional.rs` 中仍有 `TransactionWriter`、`with_rollback`、`execute_in_transaction` 等未使用 helper。
- 当前 `GraphStorage` writer 已经有单条写和批量写的内部回滚逻辑。

主要风险：

- helper 保留但不接入，会误导后续开发者以为它是事务主路径。
- helper 与 writer 回滚逻辑分散，会导致单条写、批量写、事务写语义漂移。
- `UndoLogManager` 如果没有从 PropertyGraph 写入方法统一记录 undo，外层 helper 无法可靠回滚。

处理方向：

1. 短期：删除没有调用路径的 `TransactionWriter`，或把它移动到测试模块。
2. 中期：如果要保留 undo-log 回滚，必须让 PropertyGraph 写方法或 GraphStorage 写路径统一记录 undo。
3. 批量写、单条写和事务写应共用同一套 rollback primitive。
4. transaction crate 负责事务状态和锁；storage 负责具体 undo/persistence 语义。

### 5. persistence/snapshot 管理 API 有未接入能力

相关位置：

- `crates/graphdb-storage/src/storage/engine/persistence_coordinator.rs`
- `crates/graphdb-storage/src/storage/engine/snapshot_manager.rs`
- `crates/graphdb-storage/src/storage/engine/wal_manager.rs`
- `crates/graphdb-storage/src/storage/engine/graph_storage/persistence.rs`

现状：

- GraphStorage 已有 flush、checkpoint、recover 等入口。
- `PersistenceCoordinator` 和 `SnapshotManager` 仍有部分 stats、verification、cleanup、trigger 等 API 未被主链路调用。
- `PersistenceState::WalWritten`、`Flushing` 等状态目前没有完整状态机使用。

主要风险：

- 管理 API 宽于实际状态机，容易出现“方法存在但状态不可信”的问题。
- snapshot verification/cleanup 如果不进入 admin/maintenance 生命周期，长期会成为不可测能力。

处理方向：

1. 将 persistence admin 能力分成三类：
   - 已接入：flush、checkpoint、recover。
   - 应接入：cleanup、verify、stats。
   - 应删除或私有化：没有明确使用场景的 trigger/helper。
2. `PersistenceState` 应由真实状态转换驱动，否则删除未使用状态。
3. 增加 checkpoint/snapshot admin 测试。

### 6. QueryOps 与 EdgeTraversalParams 像是过时中间层

相关位置：

- `crates/graphdb-storage/src/storage/engine/query.rs`
- `crates/graphdb-storage/src/storage/engine/edge_params.rs`

现状：

- `QueryOps` 只提供薄封装，当前没有被主查询路径使用。
- `EdgeTraversalParams` 没有构造路径。
- 读路径已经通过 `GraphStorage` reader 和 query crate 组织。

主要风险：

- 这些薄封装如果保留，会让 storage 内出现“看似可用但不在主链路”的查询入口。
- 后续 query 优化可能错误地绕过 schema/index/timestamp 语义。

处理方向：

- 若 query crate 不需要这些类型，删除。
- 若确实需要，应迁移到明确的 reader/query adapter 模块，并由实际调用链覆盖。

### 7. 底层 table、CSR、column store API 面过宽

相关位置：

- `crates/graphdb-storage/src/storage/vertex/vertex_table.rs`
- `crates/graphdb-storage/src/storage/edge/edge_table.rs`
- `crates/graphdb-storage/src/storage/edge/*csr*.rs`
- `crates/graphdb-storage/src/storage/vertex/column_store.rs`
- `crates/graphdb-storage/src/storage/edge/property_table.rs`

现状：

- 底层结构暴露大量方法，其中一部分仅由测试覆盖，另一部分没有调用路径。
- 部分 API 是合理的未来能力，例如 compact、encoding、freeze、iterator。
- 另一部分 API 只是重复入口，例如多套 batch/update/delete/helper。

主要风险：

- API 面过宽会诱导上层绕过 `GraphStorage`。
- 未接入方法缺少真实生命周期验证，行为容易与主路径不一致。
- 未来维护者难以判断哪些方法是稳定能力、哪些只是实验代码。

处理方向：

1. 将底层 API 分成三类：
   - 主链路调用：保留。
   - compact/flush/load 后续要用：保留但尽快接入生命周期。
   - 无明确路径：删除或移入 `#[cfg(test)]`。
2. 不要为了消 warning 添加 `allow`。
3. 每完成一个集成阶段后重新跑 warning 清单，确认减少的是架构噪音，不是隐藏问题。

## 后续阶段计划

### Phase 1：收敛 transaction helper 和过时薄封装

目标：

- 删除或迁移没有调用路径的事务 helper、QueryOps、EdgeTraversalParams。
- 确认 GraphStorage writer 是唯一 DML 回滚入口。

建议修改：

- 评估并删除 `TransactionWriter`。
- 评估 `with_rollback`、`execute_in_transaction` 是否能被真实 undo-log 路径使用；不能使用则删除。
- 删除或迁移 `QueryOps`、`EdgeTraversalParams`。

验收：

- `cargo check -p graphdb-storage --all-targets` 通过。
- transaction/query 相关 unused warning 明显减少。
- 不引入新的 DML 入口。

### Phase 2：index API 主链路收敛

目标：

- 让 index update/delete/lookup/rebuild 只有一套主路径。
- 为 key compression 做生命周期准备。

建议修改：

- 判断 `IndexUpdater` 是否合并进当前 writer helper；如果不合并则删除。
- 判断 generic index manager 是否真正用于 vertex/edge manager；如果不使用则删除或推迟。
- 将 compression config 和 compressor metadata 设计成 rebuild/compact 可持久化结构。

验收：

- index DML、lookup、rebuild 测试通过。
- 无第二套 index update 入口。

### Phase 3：列编码接入 compact/freeze

目标：

- 让 column encoding 从实验能力变成真实存储优化。

建议修改：

- 在 vertex/edge compact 中调用 encoding selector。
- 持久化 encoding metadata。
- load 后恢复 encoded column。
- 增加 compact 前后、flush/load 后读取一致性测试。

验收：

- compact 前后 vertex/edge 读取一致。
- 编码 metadata 缺失或损坏时返回明确错误。

### Phase 4：物理压缩接入持久化格式

目标：

- 让 `CompressionType` 真正影响 flush/load/checkpoint。

建议修改：

- 文件 header 写入压缩类型、格式版本、校验信息。
- flush/checkpoint 压缩 payload。
- load/recovery 根据 header 解压。

验收：

- `None` 与压缩模式都可恢复。
- 损坏压缩 payload 有明确错误。

### Phase 5：immutable CSR segment

目标：

- 让不可变 CSR 成为 edge 冷数据/稳定段。

建议修改：

- `EdgeTable` 增加 immutable segment 集合。
- mutable delta 承接新写入。
- compact/freeze 生成 immutable CSR。
- 读路径合并 base segment 和 delta。
- 删除使用 tombstone 或 delta delete。

验收：

- freeze 前后 edge scan/get 一致。
- 删除和更新不会让旧 segment 中数据错误可见。
- flush/load 后 segment metadata 正确恢复。

### Phase 6：persistence/snapshot admin 能力补齐

目标：

- 让已有 checkpoint/snapshot 辅助 API 要么接入 admin 生命周期，要么删除。

建议修改：

- 接入 snapshot verify、cleanup、stats。
- 精简未被状态机使用的 `PersistenceState`。
- 增加 admin/maintenance 测试。

验收：

- snapshot/checkpoint 管理操作有真实入口和测试。
- 未使用 persistence warning 减少。

### Phase 7：底层宽 API 最终清理

目标：

- 清理完成前几阶段后仍没有调用路径的方法和类型。

建议修改：

- 删除无主链路、无近期计划、无测试价值的底层方法。
- 测试专用 helper 移入 `#[cfg(test)]`。
- 将保留的内部能力改成 `pub(crate)` 或更窄可见性。

验收：

- `cargo check -p graphdb-storage --all-targets` 通过。
- 剩余 warning 都对应明确的后续功能，而不是历史遗留。

## 推荐优先级

短期优先：

1. transaction helper 收敛。
2. QueryOps / EdgeTraversalParams 删除或迁移。
3. IndexUpdater / generic index manager 主链路判断。

中期推进：

1. 列编码 compact/freeze。
2. 物理压缩 flush/load/checkpoint。
3. persistence/snapshot admin 补齐。

长期推进：

1. immutable CSR segment。
2. live index key compression。
3. 底层 table/CSR/column store 宽 API 最终清理。

## 执行原则

1. 所有新增能力必须从 `GraphStorage` 或明确的 admin/maintenance 入口进入。
2. 不添加新的 `dead_code` 抑制。
3. 不为了减少 warning 保留空调用或假集成。
4. 对“未来可能用”的代码，必须写清楚未来入口；写不清楚就删除或移动到测试。
5. 每个阶段都运行：

```shell
cargo check -p graphdb-storage --all-targets
```

并针对修改路径补充单元测试或集成测试。
