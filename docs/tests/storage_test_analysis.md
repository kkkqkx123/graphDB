# Storage 模块测试分析与策略

## 一、模块架构总览

`graphdb-storage` 是图数据库的核心存储层，位于依赖 DAG：`core → config → search → sync → transaction → **storage** → query → api`

### 目录结构

```
crates/graphdb-storage/src/storage/
├── container/          # 存储容器层（mmap 持久化 / 内存易失）
│   ├── persistent/     # 文件 mmap 持久化容器（OS 平台适配）
│   ├── volatile/       # 内存易失容器（临时/缓存/测试）
│   ├── container_trait # IDataContainer trait
│   └── types           # 容器配置/错误/统计
├── vertex/             # 顶点列式存储
│   ├── vertex_table    # 顶点主表 + MVCC
│   ├── column_store    # 列式属性存储
│   ├── id_indexer      # external_id <-> internal_id
│   ├── vertex_timestamp # MVCC 时间戳
│   └── encoding/       # ALP/RLE/Dict/FSST/Bitpack/Varint/Selector/Lazy
├── edge/               # 边 CSR 存储
│   ├── csr/csr_trait   # 不可变 CSR
│   ├── mutable_csr     # 多边可变 CSR
│   ├── single_mutable_csr # 单边 O(1) CSR
│   ├── mutable_csr_variant  # 策略枚举
│   ├── edge_table      # 出/入边 + 属性
│   └── property_table  # 边属性
├── index/              # 索引系统
│   ├── primary/        # CSR 感知索引（edge_id / degree）
│   ├── secondary/      # 二级属性索引（BTReeMap + MVCC + GC）
│   │   ├── vertex_index_manager / edge_index_manager
│   │   ├── index_data_manager / index_updater / index_gc_manager
│   │   └── key_codec/  # 键编码器/压缩
│   └── index_types
├── cache/              # 记录缓存（moka LRU + 权重淘汰）
├── engine/             # 存储引擎核心
│   ├── property_graph/ # PropertyGraph 门面（整合 vertex/edge/cache/index）
│   ├── graph_storage/  # GraphStorage = StorageClient trait 实现
│   ├── data_store      # GraphDataStore（HashMap<LabelId, VertexTable>）
│   ├── transaction/    # 事务操作原语
│   ├── persistence_coordinator # WAL->Flush->Checkpoint->Snapshot
│   ├── snapshot_manager/ wal_manager/ cache_manager/
│   ├── batch           # 批量导入
│   └── config/query/edge_params
├── extend/             # 扩展存储（全文搜索桥接）
├── metadata/           # re-export from core
├── utils/              # 转换/格式化工具
├── storage_client.rs   # 5 个 trait 定义
├── storage_types.rs    # PropertyId/EdgeOffset 值类型
├── compression.rs      # 压缩类型枚举
├── metrics.rs          # 监控包装 Storage
└── test_mock.rs        # MockStorage
```

### 持久化链路

```
Write → WAL → Memory → Flush (定时/增量刷盘) → Checkpoint (一致性快照) → Snapshot (全量备份)
```

### 核心 trait 体系

```
StorageClient (supertrait)
├── StorageReader   — 读操作（顶点/边/索引/SChema）
├── StorageWriter   — 写操作（增/删/改/批处理）
├── StorageSchemaOps — DDL（Space/Tag/EdgeType/Index）
├── StorageAuthOps  — 认证授权
└── StorageAdmin    — 管理（持久化/统计/维护/GC）
```

## 二、现有测试覆盖

### 2.1 包内单元测试（45 个 `#[cfg(test)]` 模块）

| 模块 | 覆盖内容 | 状态 |
|------|----------|------|
| compression.rs | CompressionType 序列化 | ✅ 良好 |
| container/persistent | mmap 创建/读写/校验/批量 | ✅ 良好 |
| container/volatile | 内存容器/大页 | ✅ 良好 |
| container/types | FileHeader | ✅ 基本 |
| vertex/vertex_table | 顶点 CRUD+扫描 | ✅ 良好 |
| vertex/column_store | 列存储 CRUD | ✅ 良好 |
| vertex/id_indexer | 内外 ID 映射 | ✅ 良好 |
| vertex/vertex_timestamp | MVCC 时间戳 | ✅ 良好 |
| vertex/encoding/* | 8 种编码器 | ✅ 良好 |
| edge/csr/csr_trait | 不可变 CSR | ✅ 基本 |
| edge/mutable_csr | 可变 CSR | ✅ 良好 |
| edge/single_mutable_csr | 单边 CSR | ✅ 良好 |
| edge/edge_table | 边表 | ✅ 良好 |
| edge/property_table | 边属性表 | ✅ 良好 |
| index/**/* | 主/二级索引 CRUD+GC | ✅ 良好 |
| cache/record_cache_test | 缓存基础/统计/淘汰/并发 | ✅ 良好 |
| engine/property_graph_tests | PropertyGraph 集成（~20 测试） | ✅ 良好 |
| engine/batch | 批量导入 | ✅ 基本 |
| engine/snapshot_manager | 快照管理 | ✅ 良好 |
| engine/persistence_coordinator | 持久化协调器 | ✅ 基本 |
| engine/wal_manager | WAL 管理 | ✅ 基本 |

### 2.2 未覆盖（缺少包内测试）

| 文件 | 风险说明 |
|------|----------|
| engine/graph_storage/reader.rs | reader 核心函数无直接测试 |
| engine/graph_storage/writer.rs | writer 核心函数无直接测试 |
| engine/graph_storage/schema_adapter.rs | schema 适配无直接测试 |
| engine/graph_storage/persistence.rs | 持久化操作无直接测试 |
| engine/data_store.rs | 数据存储门面无测试 |
| engine/config.rs | 配置类型无测试 |
| engine/query.rs | 查询操作无测试 |
| engine/cache_manager.rs | 缓存管理器无直接测试 |
| engine/edge_params.rs | 边参数类型无测试 |
| extend/fulltext_storage.rs | 全文搜索桥接无测试 |
| metrics.rs | 监控包装无测试 |
| utils/convert.rs / persistence_format.rs | 工具函数无测试 |

### 2.3 全局集成测试（`tests/` 目录）

| 文件 | 覆盖内容 |
|------|----------|
| integration_index.rs | 索引元数据 + 索引查询 |
| integration_data_flow.rs | 完整 CRUD 流 |
| integration_query.rs | 查询引擎集成 |
| integration_transaction.rs | 事务集成 |
| transaction/storage_integration.rs | 事务+存储集成 |
| integration_ddl/dml/dql | DDL/DML/DQL 全流程 |
| common/test_scenario.rs | TestScenario 框架 |
| common/storage_helpers.rs | 存储辅助函数 |

## 三、测试策略

### 3.1 分层模型

```
┌──────────────────────────────────────────┐
│  全局 tests/ : 跨 crate 集成测试           │
│  完整链路：GraphStorage → TXN → Query     │
├──────────────────────────────────────────┤
│  包内集成测试 (mod integration_tests) :    │
│  子模块交互：PropertyGraph+Index+Cache+WAL │
├──────────────────────────────────────────┤
│  包内单元测试 (#[cfg(test)]) :             │
│  单组件：CSR/Container/VertexTable/编码   │
└──────────────────────────────────────────┘
```

### 3.2 新增测试优先级

#### 包内测试（高优先级）

| 测试文件 | 位置 | 测试内容 |
|----------|------|----------|
| `engine/graph_storage/test.rs` | 包内 | GraphStorage 的 reader/writer/schema_adapter 核心方法 |
| `engine/data_store_test.rs` | 包内 | DataStore 表管理 + VertexTable/EdgeTable 协作 |
| `engine/persistence_test.rs` | 包内 | flush/checkpoint/load 持久化链路 |

#### 全局测试（高优先级）

| 测试文件 | 位置 | 测试内容 |
|----------|------|----------|
| `tests/storage/mod.rs` + 子模块 | 全局 | 持久化恢复、批量完整性、缓存一致性、配置变体 |
| `tests/integration_storage.rs` | 全局 | StorageClient 完整 API 端到端测试 |

### 3.3 测试内容分类

| 场景 | 包内单元 | 包内集成 | 全局集成 |
|------|:--------:|:--------:|:--------:|
| 单 CSR 操作 | ✅ 已有 | — | — |
| 单编码器 | ✅ 已有 | — | — |
| Container 读写 | ✅ 已有 | — | — |
| VertexTable CRUD | ✅ 已有 | — | — |
| PropertyGraph CRUD | — | ✅ 已有 | — |
| Cache 命中/淘汰 | ✅ 已有 | — | — |
| 索引自动维护 | — | ✅ 已有 | — |
| GraphStorage API | — | ⚠️ 新增 | ✅ 已有 |
| WAL+存储一致性 | — | ⚠️ 新增 | — |
| Flush+Checkpoint 恢复 | — | ⚠️ 新增 | ⚠️ 新增 |
| 事务+存储原子性 | — | — | ✅ 已有 |
| 全文搜索集成 | — | ⚠️ 新增 | — |
| 端到端数据流 | — | — | ✅ 已有 |
| 批量导入 | — | ✅ 已有 | — |
| 快照管理 | ✅ 已有 | — | — |
| 持久化协调器 | ✅ 已有 | — | — |
| 配置变体 | — | — | ⚠️ 新增 |

## 四、实施计划

### 阶段一：包内测试新增

1. **[engine/graph_storage/test.rs](file:///d:/项目/database/graphDB/crates/graphdb-storage/src/storage/engine/graph_storage/test.rs)** — 通过 GraphStorage 直接调用 reader/writer/schema_adapter 方法，验证：
   - 创建 Space/Tag/EdgeType 后能正确读写顶点和边
   - 索引创建和查询
   - 用户管理操作
   - Stats 统计

2. **[engine/data_store_test.rs](file:///d:/项目/database/graphDB/crates/graphdb-storage/src/storage/engine/data_store_test.rs)** — 直接构造 GraphDataStore，验证：
   - VertexTable/EdgeTable 注册、查找、删除
   - Label 名称 <-> ID 映射
   - 计数器递增

3. **[engine/persistence_test.rs](file:///d:/项目/database/graphDB/crates/graphdb-storage/src/storage/engine/persistence_test.rs)** — 通过 PropertyGraph 验证：
   - flush_to_disk + load_data 的序列化完整性
   - flush_incremental 增量刷盘
   - 写入数据后 flush，重新加载后数据一致

### 阶段二：全局集成测试新增

1. **`tests/storage/moD.rs` + 子模块** — 存储集成测试集合
2. **`tests/storage/persistence_recovery.rs`** — 持久化 + 恢复
3. **`tests/storage/bulk_data_integrity.rs`** — 批量数据完整性
4. **`tests/storage/cache_coherence.rs`** — 缓存一致性
5. **`tests/storage/config_variants.rs`** — 配置参数变体