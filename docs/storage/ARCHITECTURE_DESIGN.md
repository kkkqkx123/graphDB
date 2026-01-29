# 存储层架构设计文档

## 一、概述

本文档描述 GraphDB 项目存储层的重构设计方案，目标是建立一套清晰、高效、可扩展的存储架构。设计参考 Nebula-Graph 的成熟实现，同时结合 Rust 语言特性和项目实际情况进行优化。

当前存储模块存在职责混乱、接口臃肿、缺少执行计划层等问题，严重影响了代码的可维护性和性能优化空间。通过本次重构，我们将建立分层的存储架构，实现元数据与数据存储分离，并引入查询计划执行机制。

本文档适用于项目开发阶段的架构重构，由于项目仍处于早期开发周期，本次重构不保留向后兼容性，可以进行大胆的接口调整和模块重组。

## 二、当前问题分析

### 2.1 StorageEngine Trait 职责过重

当前 `StorageEngine` trait 包含超过 50 个方法，涵盖了数据操作、DDL 语句、索引管理、事务处理等多个完全不同的职责领域。这种设计导致 trait 极其臃肿，任何存储实现的添加都需要实现所有方法，增加了不必要的复杂度。更严重的是，这种设计违反了面向对象设计中的单一职责原则，使得接口的演进和维护变得困难。当需要为特定场景添加新方法时，需要修改所有实现类，这在大规模项目中是不可接受的。

从代码组织的角度来看，50 多个方法挤在一个 trait 中，让代码阅读者难以快速定位和理解各个功能模块的边界。理想的设计应该是每个功能领域有独立的接口定义，使用者按需引入所需的功能集。当前设计还导致了测试复杂度上升，因为为每种存储实现编写完整的 mock 对象需要实现所有方法，这大大增加了单元测试的编写成本。

### 2.2 缺少查询计划执行层

当前存储层直接暴露原子操作接口，如 `scan_vertices_by_tag`、`get_node_edges` 等，缺少查询编译和执行计划层。这种设计使得复杂查询无法进行全局优化，例如当执行「获取所有年龄大于 30 的用户的朋友」这样的查询时，系统无法智能地将过滤条件下推到存储层执行，也无法合并多个扫描操作以减少 IO 次数。在 Nebula-Graph 中，查询会被编译为执行计划树，各算子之间可以流水线式地传递数据，避免中间结果物化带来的内存压力。当前设计无法支持这类高级优化。

没有执行计划层的另一个问题是查询语义与存储实现紧密绑定。如果未来需要更换存储引擎或者支持多种存储后端，现有查询逻辑无法平滑迁移。执行计划层作为抽象层，可以隔离查询处理逻辑与存储实现细节，提高系统的可移植性和可扩展性。

### 2.3 元数据与数据存储高度耦合

在 `MemoryStorage` 结构体中，元数据（SpaceInfo、TagInfo、EdgeTypeSchema、IndexInfo）与数据（vertices、edges）使用相同的 Arc<Mutex<HashMap>> 模式存储。这种耦合导致几个问题：首先，元数据的变更需要获取数据锁，可能影响并发性能；其次，元数据的生命周期与数据绑定，难以独立演进；第三，无法支持元数据的持久化缓存。

在成熟的数据库系统中，元数据通常由专门的元数据服务管理，数据存储层只负责根据元数据定义存储和检索数据。这种分离可以支持在线 Schema 演进、Schema 版本管理、多租户隔离等高级特性。当前设计无法满足这些需求。

### 2.4 迭代器设计不完整

现有的迭代器实现包括 default_iter、get_neighbors_iter、prop_iter 和 sequential_iter，但这些迭代器之间缺乏统一的接口抽象，功能边界也不清晰。缺少 Nebula-Graph 中的 PipeLine 模式支持，无法实现算子间的流水线数据传递。迭代器应当提供统一的遍历接口，同时支持过滤、映射、聚合等操作，但当前实现远未达到这个目标。

迭代器的缺失还导致了数据处理效率问题。当前很多操作采用全量数据加载到内存的方式处理，对于大规模数据场景，这种方式的内存开销和延迟都不可接受。Nebula-Graph 的存储层支持增量扫描和延迟物化，这些特性都需要完善的迭代器支持。

## 三、目标架构设计

### 3.1 整体架构图

```
src/storage/
├── mod.rs                          # 统一导出，对外提供简洁接口
│
├── engine/                         # 存储引擎层（基础 KV 操作）
│   ├── mod.rs
│   ├── engine_trait.rs             # Engine trait（精简版，只含基础操作）
│   │   ├── fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>
│   │   ├── fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()>
│   │   ├── fn delete(&mut self, key: &[u8]) -> Result<()>
│   │   ├── fn scan(&self, prefix: &[u8]) -> Result<Box<dyn StorageIterator>>
│   │   └── fn batch(&mut self, ops: Vec<Operation>) -> Result<()>
│   │
│   ├── memory_engine.rs            # 内存引擎实现（基于 HashMap）
│   ├── redb_engine.rs              # 持久化引擎实现（基于 redb）
│   └── rocksdb_engine.rs            # RocksDB 引擎实现（可选）
│
├── operations/                     # 操作层（数据读写封装）
│   ├── mod.rs
│   │
│   ├── reader/                     # 读取操作封装
│   │   ├── mod.rs
│   │   ├── vertex_reader.rs        # 点读取器
│   │   ├── edge_reader.rs          # 边读取器
│   │   ├── scan_reader.rs          # 扫描读取器
│   │   └── index_reader.rs         # 索引读取器
│   │
│   └── writer/                     # 写入操作封装
│       ├── mod.rs
│       ├── vertex_writer.rs        # 点写入器
│       └── edge_writer.rs          # 边写入器
│
├── plan/                           # 查询计划层（编译与执行）
│   ├── mod.rs
│   ├── plan_trait.rs               # Plan trait
│   ├── context.rs                  # 执行上下文
│   │
│   ├── nodes/                      # 执行节点定义
│   │   ├── mod.rs
│   │   ├── scan_node.rs            # 扫描节点
│   │   ├── filter_node.rs          # 过滤节点
│   │   ├── get_neighbors_node.rs   # 获取邻居节点
│   │   ├── project_node.rs         # 投影节点
│   │   ├── aggregate_node.rs       # 聚合节点
│   │   ├── limit_node.rs           # 限制节点
│   │   └── dedup_node.rs           # 去重节点
│   │
│   └── executors/                  # 执行器实现
│       ├── mod.rs
│       ├── scan_executor.rs        # 扫描执行器
│       ├── filter_executor.rs      # 过滤执行器
│       ├── get_neighbors_executor.rs
│       ├── project_executor.rs
│       ├── aggregate_executor.rs
│       └── join_executor.rs        # 连接执行器
│
├── metadata/                       # 元数据层（Schema 管理）
│   ├── mod.rs
│   ├── schema_manager.rs           # Schema 管理器
│   ├── index_manager.rs            # 索引管理器
│   ├── types.rs                    # 元数据类型定义
│   └── schema_change.rs            # Schema 变更操作
│
├── iterator/                       # 迭代器层
│   ├── mod.rs
│   ├── storage_iter_trait.rs       # 存储迭代器 trait
│   ├── scan_iter.rs                # 扫描迭代器
│   ├── filter_iter.rs              # 过滤迭代器
│   ├── transform_iter.rs           # 转换迭代器
│   ├── limit_iter.rs               # 限制迭代器
│   └── chain_iter.rs               # 链式迭代器（PipeLine）
│
└── transaction/                    # 事务层
    ├── mod.rs
    ├── transaction_manager.rs      # 事务管理器
    ├── transaction.rs              # 事务接口
    └── lock_manager.rs             # 锁管理器
```

### 3.2 核心设计原则

本架构设计遵循以下核心原则，确保系统具备良好的可维护性、可扩展性和性能表现。

**职责分离原则**要求每个模块只负责单一的功能领域。Engine 层只处理最底层的 KV 操作，不了解任何图语义；Operations 层封装图操作但不涉及查询优化；Plan 层负责查询编译和执行，但不了解存储细节。这种分离使得各层可以独立演进，降低耦合度。当存储引擎需要更换时，只需要修改 Engine 层的实现，上层的 Operations 和 Plan 完全不需要改动。

**接口隔离原则**强调使用方只依赖它们实际使用的方法。在新的设计中，Reader 和 Writer 可能是独立的 trait，用户可以根据需要只引入读取接口或写入接口。这种设计避免了当前 StorageEngine 那种「all-or-nothing」的问题。

**数据惰性求值原则**要求尽量避免不必要的数据物化。查询执行过程中，数据应当以流的形式在算子间传递，只有在必要时才进行物化。迭代器设计支持 PipeLine 模式，前一个算子的输出直接作为后一个算子的输入，不需要中间缓存。

**元数据独立原则**将元数据管理与数据存储完全分离。元数据层独立管理 Schema、索引定义等元信息，不与具体数据绑定。这支持 Schema 演进、多版本管理、在线 DDL 等高级特性。

### 3.3 层次职责说明

**Engine 层**是整个存储架构的最底层，提供最基础的键值存储抽象。这一层的接口设计参考了 RocksDB 等成熟 KV 存储的 API，确保通用性和可替换性。该层只理解字节数组，不关心数据的具体含义。核心接口包括 `get` 用于单点查询、`put` 用于写入、`delete` 用于删除、`scan` 用于范围扫描、`batch` 用于批量操作。所有操作都是同步的，异步操作可以在上层封装。Engine 层需要支持事务，但这里的事务是 KV 级别的事务，与图语义无关。

**Operations 层**在 Engine 层之上封装图数据库特有的操作语义。该层将图概念（点、边、属性）转换为 KV 操作，理解图数据的编码格式。例如，将 Vertex 对象编码为字节序列存储到 Engine 中，或者从 Engine 中读取字节并解码为 Vertex 对象。Operations 层还负责数据的序列化和反序列化，确保存储格式与 Nebula-Graph 兼容。这一层不涉及查询优化，只负责正确地执行给定的操作。

**Plan 层**负责将用户查询编译为可执行的计划树。当用户执行一条 MATCH 或 GO 语句时，Planner 首先生成逻辑计划，然后 Optimizer 优化逻辑计划生成物理计划，最后由 Plan 层执行物理计划。执行时，各节点按拓扑顺序执行，数据在节点间流水线传递。Plan 层支持多种执行策略，如向量化执行、迭代器模式执行等，可以根据数据规模选择最优策略。

**Metadata 层**独立管理所有 Schema 信息，包括 Space、Tag、EdgeType、Index 等。该层提供 Schema 的增删改查接口，同时维护 Schema 版本信息，支持 Schema 演进。Metadata 层还负责将 Schema 信息转换为 Operations 层可用的格式，例如将 TagInfo 转换为列定义供序列化使用。

**Iterator 层**提供统一的数据遍历抽象。不同于 Rust 标准库的 Iterator，StorageIterator 针对存储场景设计，支持键值过滤、范围扫描、事务可见性等特性。Iterator 层还实现了 PipeLine 模式，支持多个迭代器链式组合，数据在链中流动而不需要中间存储。

**Transaction 层**提供事务语义支持，包括显式事务（用户 BEGIN/COMMIT）和隐式事务（单语句自动提交）。该层实现 MVCC 或两阶段锁等并发控制协议，保证事务的隔离性。Transaction 层与 Engine 层紧密配合，Engine 层提供 KV 级别的事务原语，Transaction 层在此基础上实现图语义的事务。

## 四、核心接口设计

### 4.1 Engine Trait

Engine 是存储架构的最底层接口，定义如下：

```rust
// engine/engine_trait.rs

use crate::core::StorageError;

/// 存储引擎 trait - 最底层的 KV 存储抽象
pub trait Engine: Send + Sync {
    /// 单点查询
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;

    /// 单点写入
    fn put(&mut self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;

    /// 单点删除
    fn delete(&mut self, key: &[u8]) -> Result<(), StorageError>;

    /// 范围扫描
    fn scan(
        &self,
        prefix: &[u8],
    ) -> Result<Box<dyn StorageIterator>, StorageError>;

    /// 批量操作
    fn batch(&mut self, ops: Vec<Operation>) -> Result<(), StorageError>;
}

/// 存储操作类型
pub enum Operation {
    Put { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
}

/// 存储迭代器 trait
pub trait StorageIterator: Send + {
    /// 获取当前键
    fn key(&self) -> Option<&[u8]>;

    /// 获取当前值
    fn value(&self) -> Option<&[u8]>;

    /// 移动到下一个键值对
    fn next(&mut self) -> bool;

    /// 估算剩余条目数量
    fn estimate_remaining(&self) -> Option<usize>;
}
```

Engine 接口的设计遵循简洁性原则，只包含最必要的操作。任何复杂的图操作都可以通过组合这些基本操作实现。`batch` 方法支持原子地执行多个操作，是实现事务的基础。`scan` 方法返回迭代器，支持惰性加载大量数据。

### 4.2 Reader/Writer 接口

Operations 层定义 Reader 和 Writer 接口：

```rust
// operations/reader/mod.rs

use crate::core::{Value, Vertex, Edge, Schema};

/// 点读取器 trait
pub trait VertexReader: Send + Sync {
    /// 根据 ID 获取点
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError>;

    /// 扫描指定空间的所有点
    fn scan_vertices(&self, space: &str) -> Result<ScanResult<Vertex>, StorageError>;

    /// 扫描指定 Tag 的所有点
    fn scan_vertices_by_tag(
        &self,
        space: &str,
        tag_name: &str,
    ) -> Result<ScanResult<Vertex>, StorageError>;

    /// 根据属性扫描点
    fn scan_vertices_by_prop(
        &self,
        space: &str,
        tag_name: &str,
        prop_name: &str,
        value: &Value,
    ) -> Result<ScanResult<Vertex>, StorageError>;
}

/// 边读取器 trait
pub trait EdgeReader: Send + Sync {
    /// 获取指定边
    fn get_edge(
        &self,
        space: &str,
        src: &Value,
        dst: &Value,
        edge_type: &str,
    ) -> Result<Option<Edge>, StorageError>;

    /// 获取点的所有边
    fn get_node_edges(
        &self,
        space: &str,
        node_id: &Value,
        direction: EdgeDirection,
    ) -> Result<ScanResult<Edge>, StorageError>;

    /// 扫描指定类型的所有边
    fn scan_edges_by_type(
        &self,
        space: &str,
        edge_type: &str,
    ) -> Result<ScanResult<Edge>, StorageError>;
}

/// 扫描结果，支持惰性加载
pub struct ScanResult<T> {
    data: Box<dyn StorageIterator>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ScanResult<T> {
    pub fn new(data: Box<dyn StorageIterator>) -> Self {
        Self {
            data,
            _phantom: std::marker::PhantomData,
        }
    }

    /// 转换为 Vec（全部加载到内存）
    pub fn collect(self) -> Result<Vec<T>, StorageError>
    where
        T: serde::de::DeserializeOwned,
    {
        // 实现反序列化逻辑
    }

    /// 获取迭代器（惰性加载）
    pub fn into_iter(self) -> impl Iterator<Item = Result<T, StorageError>>
    where
        T: serde::de::DeserializeOwned,
    {
        // 实现惰性迭代逻辑
    }
}
```

Reader 和 Writer 接口按功能分离，用户可以根据需要选择引入。这种设计比当前的大一统接口更加灵活。ScanResult 支持两种使用模式：一次性加载到内存的 `collect` 方法，和惰性加载的迭代器模式。

### 4.3 Plan/Executor 接口

查询计划层定义如下：

```rust
// plan/mod.rs

use super::context::ExecutionContext;

/// 可执行的查询计划 trait
pub trait Plan: Send + {
    /// 执行计划
    fn execute(
        &self,
        ctx: &ExecutionContext,
    ) -> Result<Box<dyn DataSet>, ExecutionError>;
}

/// 数据集 - 查询计划的输出
pub trait DataSet: Send + {
    /// 获取模式信息
    fn schema(&self) -> &ResultSetSchema;

    /// 转换为迭代器
    fn into_iter(self) -> Box<dyn Iterator<Item = ResultRow> + Send>;
}

/// 执行结果行
pub struct ResultRow {
    columns: HashMap<String, crate::core::Value>,
}

/// 执行上下文
pub struct ExecutionContext {
    /// 空间信息
    space: SpaceInfo,
    /// Schema 管理器
    schema_manager: Arc<dyn SchemaManager>,
    /// 存储引擎
    engine: Arc<dyn Engine>,
    /// 运行时状态
    runtime: HashMap<String, Box<dyn std::any::Any>>,
}
```

Plan 接口定义了查询计划的执行契约。每个 Plan 实现负责特定的查询逻辑，执行后返回 DataSet。DataSet 支持迭代器模式消费，避免不必要的数据物化。

### 4.4 Metadata 接口

元数据层接口设计如下：

```rust
// metadata/schema_manager.rs

use super::types::{SpaceInfo, TagInfo, EdgeTypeSchema, IndexInfo};

/// Schema 管理器 trait
pub trait SchemaManager: Send + Sync {
    /// 创建 Space
    fn create_space(&self, space: &SpaceInfo) -> Result<(), SchemaError>;

    /// 删除 Space
    fn drop_space(&self, space_name: &str) -> Result<(), SchemaError>;

    /// 获取 Space
    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, SchemaError>;

    /// 列出所有 Space
    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, SchemaError>;

    /// 创建 Tag
    fn create_tag(&self, space: &str, tag: &TagInfo) -> Result<(), SchemaError>;

    /// 获取 Tag Schema（转换为 expression::storage::Schema）
    fn get_tag_schema(&self, space: &str, tag: &str) -> Result<Schema, SchemaError>;

    /// 创建 EdgeType
    fn create_edge_type(&self, space: &str, edge: &EdgeTypeSchema) -> Result<(), SchemaError>;

    /// 获取 EdgeType Schema
    fn get_edge_type_schema(&self, space: &str, edge: &str) -> Result<Schema, SchemaError>;

    /// 创建索引
    fn create_index(&self, space: &str, index: &IndexInfo) -> Result<(), SchemaError>;
}
```

SchemaManager 接口专注于元数据管理，不涉及任何数据存储逻辑。这种设计使得元数据可以独立于数据存储进行优化和演进。

## 五、分阶段执行方案

### 5.1 阶段一：接口拆分与基础重构

**目标**：将现有臃肿的 StorageEngine trait 拆分为多个职责明确的子接口，实现基础架构分离。

**主要任务**：

第一项工作是创建 engine 子模块并提取核心 KV 接口。在 storage 目录下新建 engine 目录，创建 engine_trait.rs 定义 Engine trait，提取现有代码中的 get、put、delete、scan 基础操作，形成精简的 KV 接口。这部分工作相对独立，可以先完成接口定义，然后逐步迁移现有实现。

第二项工作是拆分 Reader/Writer 接口。创建 operations 目录，在其中定义 VertexReader 和 EdgeReader 接口，将现有 StorageEngine 中与读取相关的方法迁移到这些接口中。同理，将写入相关方法迁移到 VertexWriter 和 EdgeWriter 接口。这一步不需要修改实现逻辑，只是接口项工作是更新重新组织。

第三 MemoryStorage 实现。修改 MemoryStorage 使其实现新的子接口，而不是单一的大接口。使用 Rust 的 trait 组合能力，可以为不同功能定义不同 trait，MemoryStorage 实现需要的 trait 即可。

第四项工作是更新所有调用方。将所有使用 StorageEngine 的地方修改为使用具体的子接口，例如查询模块只需要引入 VertexReader 和 EdgeReader，不需要了解写入接口。

**产出**：

完成阶段一后，src/storage 目录结构如下：

```
src/storage/
├── mod.rs
├── engine/
│   ├── mod.rs
│   └── engine_trait.rs
├── operations/
│   ├── mod.rs
│   ├── reader/
│   │   ├── mod.rs
│   │   └── vertex_reader.rs
│   └── writer/
│       ├── mod.rs
│       └── vertex_writer.rs
├── memory_storage.rs      # 更新实现
└── redb_storage.rs        # 更新实现
```

**验证标准**：

所有现有功能保持正常工作，编译通过。接口使用更加清晰，调用方按需引入接口。

### 5.2 阶段二：元数据层独立

**目标**：将元数据管理从存储引擎中分离，建立独立的 Metadata 层。

**主要任务**：

第一项工作是定义元数据结构。创建 metadata/types.rs，将 SpaceInfo、TagInfo、EdgeTypeSchema、IndexInfo 等类型迁移到该文件。这些类型应当只包含元数据信息，不包含任何存储相关字段。

第二项工作是实现 SchemaManager。创建 metadata/schema_manager.rs，实现 MemorySchemaManager 结构体。该结构体独立管理元数据，不与具体数据绑定。实现 Space、Tag、EdgeType、Index 的 CRUD 操作。

第三项工作是修改 MemoryStorage。将 MemoryStorage 中的元数据字段移除，改为持有 SchemaManager 的引用。数据存储不再直接管理元数据，而是通过 SchemaManager 查询元数据信息。

第四项工作是实现 Schema 转换。在 SchemaManager 中实现将内部 TagInfo/EdgeTypeSchema 转换为 expression::storage::Schema 的方法，供查询执行使用。

**产出**：

完成阶段二后，src/storage 目录增加 metadata 子模块：

```
src/storage/
├── metadata/
│   ├── mod.rs
│   ├── types.rs
│   ├── schema_manager.rs
│   └── schema_change.rs
├── engine/
├── operations/
├── memory_storage.rs
└── redb_storage.rs
```

MemoryStorage 结构体不再直接包含 spaces、tags、edge_type_infos 等字段，而是通过注入的 SchemaManager 访问元数据。

**验证标准**：

元数据操作正常工作，Schema 查询返回正确结果。数据操作仍能正确获取元数据信息。

### 5.3 阶段三：查询计划层实现

**目标**：引入执行计划层，支持查询编译和优化。

**主要任务**：

第一项工作是定义 Plan 和 Executor 接口。创建 plan/mod.rs 和 plan/plan_trait.rs，定义 Plan trait、DataSet、ExecutionContext 等核心类型。

第二项工作是实现基础执行节点。创建 plan/nodes/ 目录，实现 ScanNode、FilterNode、ProjectNode、LimitNode 等基础执行节点。每个节点实现 execute 方法，接受输入数据集，输出新的数据集。

第三项工作是实现执行器。创建 plan/executors/ 目录，为每个执行节点实现对应的执行器。执行器负责将节点逻辑应用到实际数据。

第四项工作是与查询模块集成。修改 src/query 模块，将原有的直接调用 StorageEngine 方式改为构建执行计划方式。查询先编译为计划，再由计划层执行。

**产出**：

完成阶段三后，src/storage 目录增加 plan 子模块：

```
src/storage/
├── metadata/
├── engine/
├── operations/
├── plan/
│   ├── mod.rs
│   ├── plan_trait.rs
│   ├── context.rs
│   ├── nodes/
│   │   ├── mod.rs
│   │   ├── scan_node.rs
│   │   ├── filter_node.rs
│   │   └── ...
│   └── executors/
│       ├── mod.rs
│       ├── scan_executor.rs
│       └── ...
├── memory_storage.rs
└── redb_storage.rs
```

**验证标准**：

复杂查询能够编译为执行计划并正确执行。执行计划支持优化（如过滤下推）。

### 5.4 阶段四：迭代器层完善

**目标**：实现统一的 StorageIterator 接口，支持 PipeLine 模式。

**主要任务**：

第一项工作是重构现有迭代器。将现有 iterator 目录下的迭代器统一到新的 StorageIterator trait 下。确保所有迭代器实现统一的接口。

第二项工作是实现链式迭代器。创建 chain_iter.rs，支持多个迭代器链式组合。数据在链中流动，前一个迭代器的输出直接作为后一个的输入。

第三项工作是实现过滤迭代器。创建 filter_iter.rs，支持在迭代过程中根据条件过滤数据。避免将不符合条件的数据加载到内存。

第四项工作是实现转换迭代器。创建 transform_iter.rs，支持在迭代过程中对数据进行转换操作，如投影、类型转换等。

**产出**：

完成阶段四后，iterator 目录结构如下：

```
src/storage/
├── iterator/
│   ├── mod.rs
│   ├── storage_iter_trait.rs
│   ├── scan_iter.rs
│   ├── filter_iter.rs
│   ├── transform_iter.rs
│   ├── limit_iter.rs
│   └── chain_iter.rs
```

**验证标准**：

迭代器可以链式组合使用。链式迭代器的内存使用优于全量物化方案。

### 5.5 阶段五：事务层实现

**目标**：实现完整的事务支持，包括显式事务和隐式事务。

**主要任务**：

第一项工作是定义事务接口。创建 transaction/mod.rs 和 transaction/transaction.rs，定义 Transaction trait 和事务配置。

第二项工作是实现事务管理器。创建 transaction/transaction_manager.rs，实现事务的创建、提交、回滚逻辑。管理活跃事务的生命周期。

第三项工作是实现锁管理器。创建 transaction/lock_manager.rs，实现行级锁或版本号机制，保证事务的隔离性。

第四项工作是与 Engine 层集成。修改 Engine 实现，支持 KV 级别的事务操作，如 write_batch、snapshot 等。

**产出**：

完成阶段五后，src/storage 目录增加 transaction 子模块：

```
src/storage/
├── metadata/
├── engine/
├── operations/
├── plan/
├── iterator/
└── transaction/
    ├── mod.rs
    ├── transaction.rs
    ├── transaction_manager.rs
    └── lock_manager.rs
```

**验证标准**：

BEGIN/COMMIT/ROLLBACK 正常工作。事务隔离级别符合预期。并发事务正确处理冲突。

### 5.6 阶段间依赖关系

各阶段之间存在依赖关系，必须按顺序执行：

```
阶段一 ──┬──> 阶段二 ──┬──> 阶段三 ──┬──> 阶段四 ──┬──> 阶段五
         │            │            │            │
         └────────────┴────────────┴────────────┘
```

阶段一是基础，必须首先完成。阶段二依赖阶段一的基础设施。阶段三依赖阶段二的元数据支持。阶段四和阶段五可以在阶段三之后并行开发。

每个阶段完成后应当有可运行的版本，确保重构过程可控。

## 六、接口兼容性说明

由于项目处于开发阶段，本次重构不保留向后兼容性。具体影响如下：

所有现有调用 StorageEngine 的代码都需要修改为使用新的子接口。建议在阶段一开始前，统计所有使用 StorageEngine 的位置，制定迁移计划。修改过程中保持功能一致，确保每一步修改后都能通过测试。

对于外部使用者（如 CLI 工具、测试代码），需要同步更新接口调用方式。由于项目规模较小，这部分修改成本可控。

本次重构不提供过渡期的兼容层，而是直接切换到新架构。这样可以避免维护两套接口的成本，也能确保新架构的纯净性。

## 七、测试策略

每个阶段完成后，需要确保以下测试全部通过：

单元测试覆盖新接口的每个方法。单元测试应当独立运行，不依赖外部资源。使用 Mock 对象模拟 Engine 和存储。

集成测试验证多个组件的协作。测试 Reader/Writer 与 Engine 的交互，测试 Plan 与 Metadata 的交互等。

端到端测试验证完整查询流程。从查询解析到计划执行，测试整个数据流。确保 Nebula-Graph 兼容的查询语义正确实现。

性能测试确保重构不引入性能退化。特别关注迭代器链式调用的性能，确保优于全量物化方案。

## 八、风险与缓解措施

### 8.1 技术风险

**风险一：接口设计不合理**。如果在阶段一完成后发现接口设计存在问题，修改成本较高。缓解措施是在阶段一投入足够时间进行接口评审，确保设计经过充分讨论后再开始编码。

**风险二：性能退化**。新的分层架构可能引入额外开销。缓解措施是在每个阶段完成后进行性能测试，及时发现和解决性能问题。

**风险三：迁移工作量超预期**。调用方修改可能比预期复杂。缓解措施是提前统计所有使用点，制定详细的迁移清单。

### 8.2 进度风险

**风险：阶段间依赖导致阻塞**。某个阶段的延迟会影响后续阶段。缓解措施是设置每个阶段的里程碑，定期检查进度，及时调整计划。

建议为每个阶段预留 20% 的缓冲时间。阶段三（查询计划层）和阶段五（事务层）是工作量最大的阶段，可能需要更多时间。

## 九、总结

本文档描述了 GraphDB 存储层的重构设计方案，通过建立分层架构解决当前接口臃肿、职责混乱的问题。重构分为五个阶段，从接口拆分到事务支持，逐步构建完整的存储体系。

核心设计要点包括：Engine 层提供精简的 KV 接口，Operations 层封装图操作语义，Plan 层支持查询编译和优化，Metadata 层独立管理 Schema，Iterator 层支持惰性数据处理，Transaction 层提供事务语义。

整个重构过程不保留向后兼容性，充分利用开发阶段的优势进行大胆的架构调整。预计总工作量约为 10-15 人周，具体进度需要根据实际情况调整。
