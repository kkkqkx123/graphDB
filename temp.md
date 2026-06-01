• 总体判断：当前 graphdb-storage 的主线设计是合理的，底层用 VertexTable 列式存点、EdgeTable + CSR 存边，上层用 GraphStorage 实现 StorageClient 给 query/api 使用。这条分层方向对单
  节点图数据库是对的。但包内职责已经明显膨胀，持久化/WAL/事务/metadata 的边界还不够收敛，适合开发期推进，不适合直接视为稳定架构。

  模块作用

  - crates/graphdb-storage/src/storage/mod.rs:9：总入口，公开 cache/edge/engine/index/metadata/metrics/vertex 等模块，并重导出大量底层类型。
  - crates/graphdb-storage/src/storage/storage_client.rs:11：定义对外接口，拆成 StorageReader、StorageWriter、StorageSchemaOps、StorageAuthOps、StorageAdmin，最后组合成
    StorageClient。

  - crates/graphdb-storage/src/storage/engine/graph_storage/mod.rs:1：高层适配层，GraphStorage 实现 StorageClient，把请求分发给 reader/writer/schema/index/persistence/user_ops。
  - crates/graphdb-storage/src/storage/engine/property_graph/mod.rs:1：真正的数据引擎门面，持有 GraphDataStore、缓存、WAL manager、表修改追踪、二级索引数据。
  - crates/graphdb-storage/src/storage/engine/data_store.rs:1：低层表容器，维护 LabelId -> VertexTable、EdgeTableKey -> EdgeTable、label name 到 id 的映射。
  - crates/graphdb-storage/src/storage/vertex/mod.rs:1：点存储，核心是列式 VertexTable、外部 ID 到内部 ID 的 IdIndexer、ColumnStore、MVCC timestamp、列编码。
  - crates/graphdb-storage/src/storage/edge/mod.rs:1：边存储，核心是 CSR、可变 CSR 变体、边属性表、EdgeTable。
  - crates/graphdb-storage/src/storage/index/mod.rs:1：索引数据层，primary index 偏 CSR 内部结构，secondary index 偏属性查询和 MVCC。
  - crates/graphdb-storage/src/storage/cache/mod.rs:1：记录缓存和 ID 映射缓存。
  - crates/graphdb-storage/src/storage/metadata/mod.rs:1：当前只重导出 core::metadata，但目录里仍有旧的 schema_manager.rs/index_manager.rs 文件，实际没有挂进模块树。
  - crates/graphdb-storage/src/storage/engine/sync_wrapper.rs:1、crates/graphdb-storage/src/storage/metrics.rs:1：装饰器式包装，用于同步外部索引、记录统计。

  使用情况
  上层使用方式比较集中：graphdb-query 和 graphdb-api 基本都依赖 StorageClient trait，而不是直接依赖 VertexTable/EdgeTable。这是好的，说明查询层和 API 层大体被接口隔离了。集成测试
  和 embedded/server 初始化更多直接用 GraphStorage。底层 PropertyGraph、VertexTable、EdgeTable 主要在 storage crate 内部和测试中使用。

  合理之处

  - trait 拆分方向正确：读、写、DDL、Auth、Admin 分开，便于上层按能力依赖。
  - GraphStorage 作为适配层、PropertyGraph 作为数据引擎，这个双层结构合理。
  - 点列式存储 + 边 CSR 存储符合图数据库访问模式。
  - LabelId、PropertyId、EdgeOffset 这类紧凑 ID 能减少字符串查找和内存开销。
  - 索引 metadata 和 index data 分开，是正确方向。
  - SyncWrapper、MetricsStorage 用装饰器扩展能力，比把所有逻辑塞进 GraphStorage 更好。

  主要问题

  1. 公开面过宽。storage/mod.rs 把 PropertyGraph、VertexTable、EdgeTable、CSR、缓存等底层结构都重导出，上层很容易绕过 StorageClient，未来重构成本会高。
  2. metadata 有遗留文件。当前 metadata/mod.rs 只重导出 core metadata，但目录下仍有未挂载的 schema_manager.rs/index_manager.rs，会误导维护者，也容易出现“两套 metadata”错觉。
  3. 持久化/WAL 还没有形成闭环。PropertyGraph 自己有 wal_manager，PersistenceCoordinator 也有 wal_manager；普通 CRUD 路径里没有看到追加 WAL 或稳定标记脏表的逻辑。也就是说接口已经
     有了，但数据写入、WAL、flush/checkpoint 的一致性链条还不完整。

  4. schema 与数据引擎是双写关系。创建 tag/edge type 时先写 SchemaManager，再写 PropertyGraph，失败时缺少统一事务边界。DDL 半成功会造成 schema 和实际表结构不一致。
  5. space 隔离存在风险。PropertyGraph 的 label name 映射看起来是全局的，create_tag 用 tag name 建 vertex type。如果不同 space 允许同名 tag，底层 label name 会冲突。应以 space_id
     + tag_name 或 schema 分配的全局 label id 为唯一键。

  6. 事务职责有重叠。storage 内有 engine/transaction，同时依赖 graphdb-transaction crate。当前看起来既有 undo/recovery/compact target，又有外部 transaction crate，边界需要重新定
     义。

  7. 部分模块像占位或过早抽象。例如 extend/fulltext_storage.rs 是空文件；query.rs 很薄；primary index 模块存在但主写路径维护情况不明显。

  建议
  短期优先做架构收敛：收窄 storage/mod.rs 的公开 API，只稳定暴露 StorageClient、GraphStorage 和必要 DTO；清理 metadata 旧文件；明确 SchemaManager 唯一归属在 core 还是 storage。

  中期应重点修持久化链路：统一 WalManager 来源，普通写路径必须追加 WAL、标记修改表、更新索引，并让 checkpoint/flush 基于同一套状态工作。new_with_path 里持久化初始化失败不应静
  默 .ok()。

  长期建议按职责拆边界：低层表结构和 CSR 是 storage engine；schema/auth/admin 是 storage service；sync/metrics 是 wrapper；transaction recovery 和 WAL 要么归 transaction crate，要
  么由 storage 明确实现，不要两边各做一半。
