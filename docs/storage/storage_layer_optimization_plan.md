# 存储层四模块分析与优化方案

> 本文档基于对 `src/storage/vertex`、`src/storage/metadata`、`src/storage/index`、`src/storage/edge` 四个目录的代码分析，识别现有问题并给出分阶段优化方案。

---

## 一、vertex — 顶点存储模块

### 1.1 Column 变长/定长双路径问题

**问题描述：**
`Column` 结构体的 `set()` 和 `get()` 方法内部需要区分变长类型（String）和定长类型（所有其他类型），导致：

- 方法内部存在大量 `if self.is_variable_length()` 分支判断
- 变长类型使用独立的 `offsets: Vec<usize>` 机制，与定长类型的 `data: Vec<u8>` 设计不对称
- 添加新变长类型（如 JSON、Bytes）时需要在多处同步修改

**优化方案：**

将 Column 拆分为两种具体实现，通过枚举统一访问：

```rust
enum ColumnImpl {
    Fixed(FixedWidthColumn),
    Variable(VariableWidthColumn),
}

struct FixedWidthColumn {
    data: Vec<u8>,
    element_size: usize,
    null_bitmap: Option<BitVec>,
    row_count: usize,
    encoding: ColumnEncoding,
}

struct VariableWidthColumn {
    data: Vec<u8>,
    offsets: Vec<usize>,
    null_bitmap: Option<BitVec>,
    row_count: usize,
    encoding: ColumnEncoding,
}
```

**收益：**

- 消除内部类型分支判断，提高可读性
- 各自独立优化（如 FixedWidthColumn 可直接随机访问）
- 新增变长类型仅需在 VariableWidthColumn 中修改

**工作量：** 中（约 1-2 天）
**兼容性：** 内部重构，对外 API 不变

### 1.2 Column 与 Encoding 数据同步问题

**问题描述：**
`Column` 同时持有 raw `data` 和 `encoding` 两份数据，写操作需要同步写入两份，读操作需要判断优先读取哪一份。编码切换时数据一致性维护复杂。

**优化方案：**

采用"编码即存储"模式：Column 在创建时即选定编码方式，数据直接以编码形式存储：

```rust
struct Column {
    pub name: String,
    pub col_id: i32,
    pub data_type: DataType,
    pub nullable: bool,
    storage: ColumnStorage,  // 统一存储后端
}

enum ColumnStorage {
    Raw {
        data: Vec<u8>,
        offsets: Vec<usize>,
    },
    Encoded(ColumnEncoding),
}
```

当使用压缩编码时，Column 不再保留 raw data，直接通过 `EncodedColumn` trait 读写。需要全量扫描时临时解压。

**收益：**

- 消除数据冗余
- 消除写操作的同步开销
- 编码切换时无需数据迁移

**工作量：** 大（约 3-5 天，需重构 ColumnStore 的使用方）
**建议：** 延后实施，等待列存访问模式稳定后再重构

---

## 二、metadata — 元数据管理模块

### 2.1 Schema 结构体冗余

**问题描述：**
`schema.rs` 中定义的 `Schema` 结构体在存储层中实际使用率很低：

- 上层模块（如 GraphStorage）直接使用 `TagInfo`/`EdgeTypeInfo`
- `SchemaManager` 内部主要操作 `TagData`/`EdgeTypeData` 而非 `Schema`
- `Schema` 的 `estimated_row_size()` 等方法未在实际写入路径中使用

**优化方案：**

弃用 `Schema` 结构体，将其职责合并到 `schema_manager.rs` 的内部类型中：

| 当前                       | 目标                                     |
| -------------------------- | ---------------------------------------- |
| `Schema`（已暴露 pub）     | 标记为 `#[doc(hidden)]` 或移除           |
| `FieldDef` + `offset` 字段 | 使用 `PropertyDef` + `BTreeMap` 直接管理 |
| `estimated_*` 方法         | 如需要则迁移到 `StoragePropertyDef`      |

**收益：**

- 减少约 80 行未使用的代码
- 降低新人的理解成本

**工作量：** 小（约 0.5 天）
**注意：** 需检查架构图上层的 re-export 链，避免破坏 pub API

### 2.2 ID 计数器锁粒度优化

**问题描述：**
`SchemaManager` 中的 `tag_id_counter` 和 `edge_type_id_counter` 是 `RwLock<HashMap<u64, AtomicU32>>`：

- 外层 `RwLock` 保护 `HashMap` 的结构
- 内层 `AtomicU32` 保护具体计数器的增减
- 创建新空间时需要竞争锁，需要获取 `tag_id_counter.write()`，这会阻塞所有其他空间的 ID 分配

**优化方案：**

使用 `DashMap<u64, AtomicU32>` 替代：

```rust
// 当前
tag_id_counter: Arc<RwLock<HashMap<u64, AtomicU32>>>,

// 优化后
tag_id_counter: Arc<DashMap<u64, AtomicU32>>,
```

这样可以做到不同空间的 ID 分配完全不互斥。

**收益：**

- 消除跨空间的锁竞争
- 高频创建空间场景下性能提升明显

**工作量：** 小（约 0.5 天）

### 2.3 ExtendedSchemaManager 导入导出未完成

**问题描述：**
`export_schema()` 和 `import_schema()` 方法目前为 stub，返回 `Err`。

**优化方案：**

两种选择：

1. **短期**：移除这两个方法，将 trait 中对应的签名改为 `unimplemented!()` 并标注 TODO
2. **中期**：补充实现，基于 `SchemaSnapshot` 的序列化/反序列化完成导入导出

建议短期选择方案 1，避免未完成功能暴露在 API 中给调用方造成困惑。

**工作量：** 极小（方案 1 约 0.2 天）

---

## 三、index — 索引模块

### 3.1 VertexIndexManager 与 EdgeIndexManager 代码重复

**问题描述：**
两个结构体几乎完全对称：

- 相同的数据结构（`forward_index` + `reverse_index` + `compressor`）
- 相同的压缩方法（`compress_key` / `decompress_key` / `train_compression`）
- 相同的 MVCC 处理逻辑
- 唯一的区别是 key 构建/解析逻辑（KeyBuilder 方法不同）

重复代码量估计：约 60%（两个文件合计约 800 行，其中至少 500 行重复）

**优化方案：**

**方案 A（推荐 — 泛型参数化）：**

```rust
struct GenericIndexManager<T: IndexKeyGenerator> {
    forward_index: Arc<RwLock<BTreeMap<SecondaryIndexKey, IndexEntry>>>,
    reverse_index: Arc<RwLock<BTreeMap<SecondaryIndexKey, IndexEntry>>>,
    compressor: Option<Arc<RwLock<IndexCompressor>>>,
    _marker: PhantomData<T>,
}

trait IndexKeyGenerator {
    fn build_forward_key(space_id: u64, index_name: &str, prop_value: &Value, ids: &[&Value]) -> Result<ByteKey>;
    fn build_reverse_key(space_id: u64, ids: &[&Value], index_name: &str) -> Result<ByteKey>;
    fn parse_reverse_key(key: &[u8]) -> Result<(Vec<u8>, String)>;
    fn type_name() -> &'static str;
}

// 具体实现
struct VertexIndexKeyGen;
impl IndexKeyGenerator for VertexIndexKeyGen { ... }

struct EdgeIndexKeyGen;
impl IndexKeyGenerator for EdgeIndexKeyGen { ... }

// 对外类型别名
type VertexIndexManager = GenericIndexManager<VertexIndexKeyGen>;
type EdgeIndexManager = GenericIndexManager<EdgeIndexKeyGen>;
```

**方案 B（宏生成）：**

```rust
macro_rules! define_index_manager {
    ($name:ident, $key_gen:expr, $build_forward:path, $build_reverse:path, $parse_reverse:path) => {
        pub struct $name { ... }
        // 生成完整实现
    };
}

define_index_manager!(VertexIndexManager, ...);
define_index_manager!(EdgeIndexManager, ...);
```

**收益：**

- 消除 500+ 行重复代码
- 新增索引类型（如 FulltextIndex）只需实现 `IndexKeyGenerator` trait
- 统一 bug 修复，不再需要同步修改两个文件

**工作量：** 中（约 1-2 天）
**风险：** 泛型 + PhantomData 会增加类型复杂度，注意保持 Debug/Clone 约束

### 3.2 IndexDataManager trait 接口过大

**问题描述：**
`IndexDataManager` trait 定义了约 20+ 个方法，包括：

- `update_vertex_indexes` / `update_vertex_indexes_mvcc`
- `update_edge_indexes` / `update_edge_indexes_mvcc`
- `delete_vertex_indexes` / `delete_vertex_indexes_mvcc`
- `lookup_tag_index` / `lookup_tag_index_mvcc`
- `clear_tag_index` / `clear_edge_index`
- `build_vertex_index_entry` / `build_edge_index_entry`
- ...以及对应的 native 类型方法

这种"大 trait"设计导致：

- 实现方必须实现所有方法（即使有些方法可以默认实现）
- 测试 mock 时需要实现大量方法
- 符合 Open-Closed Principle 较差

**优化方案：**

拆分为多个细粒度 trait：

```rust
trait VertexIndexOps {
    fn update_vertex_indexes_mvcc(...) -> Result<()>;
    fn delete_vertex_indexes_mvcc(...) -> Result<()>;
    fn lookup_tag_index_mvcc(...) -> Result<Vec<Value>>;
    fn clear_tag_index(...) -> Result<()>;
    fn build_vertex_index_entry(...) -> Result<()>;
}

trait EdgeIndexOps {
    fn update_edge_indexes_mvcc(...) -> Result<()>;
    fn delete_edge_indexes_mvcc(...) -> Result<()>;
    fn lookup_edge_index_mvcc(...) -> Result<Vec<Value>>;
    fn clear_edge_index(...) -> Result<()>;
    fn build_edge_index_entry(...) -> Result<()>;
}

trait IndexGcOps {
    fn gc_tombstones_incremental(...) -> Result<GcStats>;
    fn tombstone_count() -> usize;
}

// 组合 trait 保持向后兼容
trait IndexDataManager: VertexIndexOps + EdgeIndexOps + IndexGcOps {}
```

**收益：**

- 调用方按需依赖小接口，降低耦合
- mock 测试更简洁
- 新增索引操作不影响已有实现

**工作量：** 中（约 1 天）
**注意：** 需要同步更新 `IndexUpdater`、`IndexManagerOps` 等调用方

### 3.3 PrimaryIndex trait 抽象价值不足

**问题描述：**
定义的 `PrimaryIndex` trait 只包含 `index_name()`、`entry_count()`、`clear()`、`memory_usage()` 四个方法，而 `EdgeIdIndex` 和 `DegreeIndex` 的核心操作（`insert_edge/remove_edge/insert` 等）完全不在 trait 中。trait 实际上只提供了统计信息查询，而 `PrimaryIndexManager` 直接组合具体类型而非通过 trait 使用。

**优化方案：**

两种选择：

1. **移除 trait**，将统计信息方法直接放在各结构体上，消除不必要的抽象层
2. **扩展 trait**，增加 `insert_edge(edge_id, src, dst)` / `remove_edge(edge_id)` 等方法

推荐方案 1，因为 `EdgeIdIndex` 和 `DegreeIndex` 的接口差异过大，强行统一 trait 会导致接口膨胀和方法签名复杂化。

**收益：**

- 减少约 30 行无实际用途的代码
- 消除对 PrimaryIndex trait 的理解困惑

**工作量：** 小（约 0.3 天）

### 3.4 KeyBuilder 与 KeyParser 紧耦合

**问题描述：**
Key 的构建和解析逻辑分布在 `key_builder.rs`、`key_parser.rs`、`key_types.rs` 三个文件中，存在以下问题：

- 修改 Key 格式需要同时修改构建和解析两端的代码
- Key 格式的文档化不足，只有代码本身的注释
- 不同索引类型（vertex forward/edge、forward/reverse）的 Key 格式各不相同

**优化方案：**

定义 Key 格式的结构化描述，将构建和解析统一：

```rust
struct IndexKeyFormat {
    /// Key 各段的定义
    segments: Vec<KeySegment>,
}

enum KeySegment {
    SpaceId,
    KeyType(u8),       // 固定值标记
    IndexName,
    PropertyValue,
    VertexId,
    EdgeSrc,
    EdgeDst,
    Timestamp,
}

impl IndexKeyFormat {
    fn build(&self, params: &KeyParams) -> Vec<u8>;
    fn parse(&self, bytes: &[u8]) -> Result<ParsedComponents>;
}
```

通过统一的格式描述，确保构建和解析的一致性。新的索引类型只需定义 `IndexKeyFormat` 即可。

**收益：**

- 构建和解析逻辑自动对称
- 格式变更只需修改一处
- 易于文档化和 code review

**工作量：** 大（约 3-5 天，涉及所有索引类型，需要充分测试）
**建议：** 延后实施，作为二期优化

---

## 四、edge — 边存储模块

### 4.1 MutableCsr 扩容性能问题

**问题描述：**
`MutableCsr.expand_ExpandVertexCapacity()` 使用 `Vec::splice()` 实现容量扩展：

```rust
fn expand_vertex_capacity(&mut self, src_idx: usize) {
    // ...
    self.nbr_list.splice(
        insert_pos..insert_pos,
        std::iter::repeat_n(empty_nbr, additional),
    );
    // splice 导致插入位置之后的所有元素需要移位
    for i in addition, for i in (src_idx + 1)..self.vertex_capacity {
        self.adj_offsets[i] += additional;
    }
}
```

在高频插入场景下，splice 操作的复杂度为 O(n)，每次插入新边都可能导致大量内存移动。

**优化方案：**

**方案 A（推荐）：分段预留 + 懒惰扩容**

将 nbr_list 改为 `Vec<Vec<Nbr>>`，每个顶点独立维护自己的 nbr 列表：

```rust
struct MutableCsrV2 {
    nbr_lists: Vec<Vec<Nbr>>,   // 每个顶点独立的列表
    degrees: Vec<u32>,
    edge_count: AtomicU64,
    vertex_capacity: usize,
}
```

- 插入新边时，只需 push 到对应顶点的 Vec 中
- 扩容时仅影响单个顶点的 Vec
- 删除使用软删除（timestamp = INVALID_TIMESTAMP）

**方案 B：预留 slot 池 + 指针链**

借鉴现代内存分配器的思路：

```rust
struct MutableCsr {
    edge_pool: Vec<Nbr>,        // 连续内存池
    vertex_heads: Vec<usize>,   // 每个顶点的起始索引
    vertex_counts: Vec<u32>,    // 每个顶点的边数
    free_slots: Vec<usize>,     // 空闲 slot 索引
}
```

**收益：**

- insert_edge 摊销复杂度从 O(n) 降为 O(1)
- 适用于高频写入场景

**工作量：** 大（约 2-3 天，需重写 CSR 核心逻辑）
**兼容性：** 可能影响持久化格式，需要版本兼容处理

### 4.2 SingleMutableCsr 与 MutableCsr 重复代码

**问题描述：**
`SingleMutableCsr` 和 `MutableCsr` 虽然数据结构不同，但很多操作逻辑高度相似：

- `delete_edge_by_dst()` 逻辑几乎一致
- `revert_delete()` / `revert_delete_by_offset()` 结构相同
- `compact()` / `compact_with_ts()` 模式接近
- `dump()` / `load()` 序列化逻辑独立重复

**优化方案：**

通过 `MutableCsrVariant` 枚举内的代码直接实现公共逻辑，而非逐个委托：

```rust
impl MutableCsrVariant {
    fn add_edge_to_both(
        &mut self,
        src: VertexId, dst: VertexId,
        edge_id: EdgeId, prop_offset: u32, ts: Timestamp,
    ) -> bool {
        match self {
            Multiple(csr) => csr.insert_edge(src, dst, edge_id, prop_offset, ts),
            Single(csr) => csr.insert_edge(src, dst, edge_id, prop_offset, ts),
        }
    }

    fn mark_tombstone_in_both(
        &mut self,
        src: VertexId, ts: Timestamp,
        edge_id_or_dst: Edge,
    ) -> bool {
        // 统一软删除逻辑
    }
}
```

对于简单的委托方法（如 `resize`、`clear`、`vertex_capacity`），保持当前方式也是 ok 的。重点消除那些包含业务逻辑的重复方法。

**收益：**

- 消除约 100-150 行重复代码
- 统一逻辑减少 bug

**工作量：** 小（约 0.5 天）

### 4.3 PropertyTable 适配层开销

**问题描述：**
`PropertyTable` 注释中自述为 "row-oriented API on top of Column infrastructure"。当前实现：

- 将 Column 的列式存储包装为行式写入
- 每个属性设置都调用 `Column::set()`，需要行索引定位
- 额外维护 OverflowStore 处理大值（>256 bytes）
- 维护 `NameIndexer` 做属性名到列的查找

这种设计的问题在于：

- 行式写入对列式存储的写入不是最优的（每次写入只更新一列中的一行）
- OverflowStore 独立于 Column 存储，导致属性值可能分散在两地
- `NameIndexer` 与 Column 内部的 name 重复维护

**优化方案：**

为边缘场景设计专用的行存接口，而不是构建在列存之上：

```rust
struct EdgePropertyStore {
    // 紧凑行存储
    data: Vec<u8>,        // 所有属性值连续存储
    offsets: Vec<usize>,  // 每行的起始偏移
    schema: Vec<PropertySchema>,
    name_indexer: NameIndexer,
}
```

这种方式更符合"边属性数量少、单行访问为主的边缘属性访问模式。

**收益：**

- 消除适配层开销
- 单行访问更快（不需要分别查找每个 column）
- 写入更紧凑

**工作量：** 大（约 3-4 天，需要重写 PropertyTable 且更新所有调用方）
**建议：** 延后实施，作为二期优化

### 4.4 EdgeTable 中 cache 的侵入性耦合

**问题描述：**
`EdgeTable` 中 cache 的侵入性直接嵌入在 EdgeTable 中：

```rust
pub struct EdgeTable {
    // ... 核心存储字段 ...
    property_cache: Option<Arc<EdgePropertyCache>>,
}
```

cache 的失效逻辑散布在 `delete_edge()`、`delete_edge_by_offset()`、`update_edge_property_by_offset()` 等方法中。这导致：

- 核心存储逻辑和缓存逻辑混合
- 测试时需要 mock cache
- 如果要切换缓存策略，需要修改 EdgeTable 代码

**优化方案：**

采用 AOP 风格的 CacheManager，而非内嵌：

```rust
// EdgeTable 不再关心缓存
pub struct EdgeTable { /* 只包含存储逻辑 */ }

// 在调用方（例如 PropertyGraph）组合缓存
pub struct PropertyGraph {
    edge_tables: HashMap<LabelId, EdgeTable>,
    cache_manager: CacheManager,
}

impl PropertyGraph {
    fn delete_edge(&mut self, ...) -> Result<()> {
        let result = edge_table.delete_edge(...);
        self.cache_manager.invalidate_for_edge(src, dst);
        result
    }
}
```

如果确实需要在 EdgeTable 层面做缓存，使用 `trait EdgeTableCache` 注入：

```rust
trait EdgeTableCache: Send + Sync {
    fn get_by_offset(&self, offset: u32) -> Option<Vec<(String, Value)>>;
    fn invalidate_by_offset(&self, offset: u32);
    fn set_by_offset(&self, offset: u32, props: Vec<(String, Value)>);
}

pub struct EdgeTable {
    // ...
    cache: Option<Box<dyn EdgeTableCache>>,
}
```

**收益：**

- EdgeTable 职责单一
- 缓存策略可插拔
- 测试更简单

**工作量：** 中（约 1 天）
**兼容性：** 不影响存储格式

---

## 五、优先级与实施路线图

| 优先级 | 编号 | 优化点                                       | 工作量化   | 影响范围               | 建议阶段 |
| ------ | ---- | -------------------------------------------- | ---------- | ---------------------- | -------- |
| P0     | 4.2  | MutableCsrVariant 重复代码消除               | 小(0.5d)   | edge 模块              | 阶段一   |
| P0     | 2.3  | ExtendedSchemaManager stub cleanup           | 极小(0.2d) | metadata 模块          | 阶段一   |
| P0     | 3.3  | PrimaryIndex trait 评估                      | 小(0.3d)   | index 模块             | 阶段一   |
| P1     | 3.1  | VertexIndexManager + EdgeIndexManager 泛型化 | 中(1-2d)   | index 模块             | 阶段一   |
| P1     | 3.2  | IndexDataManager trait 拆分                  | 中(1d)     | index 模块 + engine 层 | 阶段一   |
| P1     | 2.2  | ID 计数器 DashMap 优化                       | 小(0.5d)   | metadata 模块          | 阶段一   |
| P1     | 4.4  | EdgeTable cache 解耦                         | 中(1d)     | edge + engine          | 阶段二   |
| P1     | 1.1  | Column 变长/定长拆分                         | 中(1-2d)   | vertex 模块            | 阶段二   |
| P2     | 4.1  | MutableCsr 扩容性能优化                      | 大(2-3d)   | edge 模块              | 阶段二   |
| P2     | 2.1  | Schema 结构体清理                            | 小(0.5d)   | metadata 模块          | 阶段二   |
| P2     | 1.2  | Column + Encoding 数据同步重构               | 大(3-5d)   | vertex 模块 + encoding | 阶段三   |
| P2     | 4.3  | PropertyTable 行存重构                       | 大(3-4d)   | edge 模块              | 阶段三   |
| P2     | 3.4  | KeyBuilder/KeyParser 统一化                  | 大(3-5d)   | index 模块             | 阶段三   |

### 阶段一：快速清理（约 3-4 天）

重点消除明显的代码重复和未完成功能，收益高、风险低：

- 3.1 索引管理器泛型化（2天）
- 3.2 IndexDataManager trait 拆分（1天）
- 2.2 ID 计数器优化（0.5天）
- 2.3 / 3.3 小清理（0.5天）

### 阶段二：架构优化（约 4-6 天）

核心存储路径的优化，需要充分测试：

- 4.2 EdgeTable cache 解耦（1天）
- 1.1 Column 变长/定长拆分（1-2天）
- 4.1 MutableCsr 扩容优化（2-3天）
- 2.1 Schema 清理（0.5天）

### 阶段三：深度重构（约 9-14 天）

大规模重构，需要完善的测试覆盖和性能基准：

- 1.2 Column + Encoding 整合
- 4.3 PropertyTable 行存重构
- 3.4 Key 构建/解析统一

---

## 六、测试策略

| 重构类型                 | 测试要求            | 方法                         |
| ------------------------ | ------------------- | ---------------------------- |
| 纯新增代码（trait/泛型） | 单元测试            | 与现有实现对比结果           |
| 代码内重构（拆分/合并）  | 单元测试 + 集成测试 | 保持接口不变，运行所有用例   |
| 性能优化                 | 单元测试 + 基准测试 | 新增 benchmark，对比优化前后 |
| 数据格式变更             | 兼容性测试          | 测试旧格式到新格式的迁移路径 |

对每个优化点，建议遵循流程：

1. 编写测试确认当前行为（如果测试不存在）
2. 实施优化
3. 运行所有测试确认行为未变
4. 对比性能基准

---

## 七、总结

本次分析覆盖了 `src/storage` 下四个核心模块（vertex、metadata、index、edge），共计 **12 个可优化点**，按优先级分为三个阶段：

- **阶段一（P0-P1）**：以消除重复代码和未完成功能为主，风险低、收益明确
- **阶段二（P1-P2）**：以核心存储路径优化为主，需要充分测试
- **阶段三（P2）**：以深度架构重构为主，需要完善的测试覆盖

整体来看，现有设计合理且成熟，不存在根本性的设计缺陷。上述优化属于"好上加好"的类型，可以在后续迭代中按需推进。
