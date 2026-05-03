# NeuG Storage Layer Architecture Analysis

## 1. Overview

NeuG 的 `storages` 模块是图数据库的核心存储引擎，负责属性图（Property Graph）的高效存储与检索。整体采用分层架构设计，从底层的内存映射容器到上层的图数据接口，形成完整的存储栈。

```
storages/
├── container/    # 底层内存映射容器
├── csr/          # CSR 压缩稀疏行图结构
├── graph/        # 属性图高层接口
└── loader/       # 数据加载器
```

## 2. 模块详细分析

### 2.1 Container 模块 — 内存映射容器

**核心文件：**
- `i_container.h` — 容器接口定义
- `mmap_container.h` — mmap 容器基类
- `anon_mmap_container.h` — 匿名内存映射实现
- `file_mmap_container.h` — 文件-backed 内存映射实现

**架构设计：**

```
IDataContainer (interface)
├── MMapContainer (base)
│   ├── AnonMMap          — MAP_PRIVATE | MAP_ANONYMOUS
│   ├── AnonHugeMMap      — 大页内存映射 (TLB 优化)
│   ├── FilePrivateMMap   — MAP_PRIVATE 文件映射 (COW)
│   └── FileSharedMMap    — MAP_SHARED 文件映射 (直写)
```

**关键特性：**
- 所有容器通过 `mmap` 实现，支持虚拟内存级别的扩展
- `ContainerType` 枚举定义了 4 种存储策略
- 支持 `Resize`、`Sync`、`Dump` 等操作
- `FileSharedMMap` 使用 reflink/COW 克隆技术保持稀疏结构

### 2.2 CSR 模块 — 压缩稀疏行图结构

**核心文件：**
- `csr_base.h` — CSR 基类定义
- `mutable_csr.h` — 可变 CSR 实现
- `immutable_csr.h` — 不可变 CSR 实现
- `generic_view.h` — 通用遍历视图
- `nbr.h` — 邻居节点结构

**CSR 类型体系：**

```
CsrBase (abstract)
├── TypedCsrBase<EDATA_T>
│   ├── MutableCsr<EDATA_T>        — 支持动态增删边
│   ├── SingleMutableCsr<EDATA_T>  — 单边模式 (每对顶点最多一条边)
│   ├── ImmutableCsr<EDATA_T>      — 不可变快照
│   ├── SingleImmutableCsr<EDATA_T>
│   └── EmptyCsr<EDATA_T>          — 空 CSR
```

**邻居结构 (`nbr.h`)：**

| 类型 | 字段 | 用途 |
|------|------|------|
| `ImmutableNbr<T>` | `neighbor`, `data` | 快照数据，无时间戳 |
| `MutableNbr<T>` | `neighbor`, `timestamp`, `data` | 可修改，支持 MVCC |

**关键设计：**
- `MutableCsr` 使用 `SpinLock` 保护每个顶点的邻接表，支持并发写入
- 扩容策略：容量不足时翻倍分配，通过 `ArenaAllocator` 管理内存
- `GenericView` 提供统一的图遍历接口，自动过滤不可见边（MVCC）
- `NbrIterator` 在遍历时跳过 `timestamp > read_ts` 的边

**MVCC 可见性规则：**
```cpp
// 遍历时只返回 timestamp <= read_ts 的边
while (cur != end && get_timestamp() > timestamp) {
    cur = static_cast<const char*>(cur) + stride;
}
```

### 2.3 Graph 模块 — 属性图接口

**核心文件：**
- `property_graph.h` — 核心属性图存储引擎
- `graph_interface.h` — 存储访问接口
- `schema.h` — 图模式定义
- `vertex_table.h` — 顶点表
- `edge_table.h` — 边表
- `vertex_timestamp.h` — 顶点时间戳管理

#### 2.3.1 Schema 模式管理

`Schema` 类管理完整的类型系统：

```cpp
Schema
├── vlabel_indexer_    // 顶点标签索引器
├── elabel_indexer_    // 边标签索引器
├── v_schemas_         // 顶点 Schema 列表
├── e_schemas_         // 边 Schema 映射 (key=三元组哈希)
└── tombstone bitsets  // 软删除标记
```

**标签三元组编码：** 边类型通过 `(src_label, dst_label, edge_label)` 三元组唯一标识，编码为 `uint32_t` 作为哈希键。

**DDL 操作支持：**
- `AddVertexLabel` / `DeleteVertexLabel`
- `AddEdgeLabel` / `DeleteEdgeLabel`
- 属性增删改（支持软删除）
- 支持 YAML 序列化/反序列化

#### 2.3.2 VertexTable 顶点表

```cpp
VertexTable
├── indexer_        // ID 映射器 (外部 ID → 内部 ID)
├── table_          // 列存属性表
├── pk_type_        // 主键类型
├── vertex_schema_  // 顶点 Schema
├── v_ts_           // 顶点时间戳 (MVCC)
└── memory_level_   // 内存策略
```

**关键特性：**
- 外部 ID 到内部 ID 的映射通过 `IndexerType` 维护
- 属性以**列式存储**在 `Table` 中
- `VertexSet` 提供基于时间戳的迭代器，自动跳过已删除顶点
- 支持批量插入（`insert_primary_keys` 模板函数处理不同类型主键）

#### 2.3.3 EdgeTable 边表

```cpp
EdgeTable
├── meta_            // 边 Schema 元数据
├── out_csr_         // 出边 CSR
├── in_csr_          // 入边 CSR
├── table_           // 边属性表
├── table_idx_       // 属性表索引
└── capacity_        // 容量
```

**关键设计：**
- 每个边类型维护双向 CSR（出边 + 入边）
- 边属性采用**行式存储**（与顶点的列式不同）
- CSR 类型根据 `EdgeStrategy` 自动选择（`kMultiple`/`kSingle`/`kNone`）
- 支持 CSR 版本交替（`csr_alter_version_`）以支持 schema 变更

#### 2.3.4 PropertyGraph 核心类

`PropertyGraph` 是存储层的统一入口：

```cpp
PropertyGraph
├── schema_              // 图模式
├── vertex_tables_       // 顶点表数组 (按 label 索引)
├── edge_tables_         // 边表映射 (按三元组索引)
├── v_mutex_             // 顶点级锁
├── work_dir_            // 工作目录
└── memory_level_        // 内存级别
```

**内存级别：**
| 级别 | 说明 |
|------|------|
| `kSyncToFile` | 最低内存，文件同步 |
| `kInMemory` | 默认，内存映射 |
| `kPreferHugePage` | 优先大页 |
| `kForceHugePage` | 强制大页 |

**操作接口：**
- `Open()` / `Dump()` — 持久化加载/保存
- `CreateVertexType()` / `CreateEdgeType()` — DDL
- `AddVertex()` / `AddEdge()` — 数据插入
- `DeleteVertex()` / `DeleteEdge()` — 数据删除
- `UpdateVertexProperty()` / `UpdateEdgeProperty()` — 属性更新
- `GetGenericOutgoingGraphView()` / `GetGenericIncomingGraphView()` — 遍历视图

#### 2.3.5 存储访问接口

`graph_interface.h` 定义了分层的访问接口：

```
IStorageInterface
├── StorageReadInterface      // 只读 (带 read_ts)
├── StorageInsertInterface    // 只写
└── StorageUpdateInterface    // 读写 (继承 Read + Insert)
    └── StorageAPUpdateInterface // AP 模式实现
```

**接口职责分离：**
- `StorageReadInterface`：查询执行使用，基于时间戳的快照读
- `StorageInsertInterface`：批量加载使用
- `StorageUpdateInterface`：事务更新使用，支持 DDL

### 2.4 Loader 模块 — 数据加载器

**核心文件：**
- `abstract_property_graph_loader.h` — 抽象加载器基类
- `csv_property_graph_loader.h` — CSV 格式加载器
- `loader_factory.h` — 加载器工厂
- `loading_config.h` — 加载配置

**加载流程：**
```
AbstractPropertyGraphLoader
├── loadVertices()
│   ├── createVertexRecordBatchSupplier()  // 创建数据供给器
│   └── addVerticesToVertexTable()          // 批量写入顶点表
└── loadEdges()
    ├── createEdgeRecordBatchSupplier()     // 创建数据供给器
    └── addEdgesToEdgeTable()               // 批量写入边表
```

**设计模式：**
- 工厂模式：`LoaderFactory` 根据配置创建对应类型的加载器
- 策略模式：不同数据源实现 `IRecordBatchSupplier` 接口
- 并行加载：`thread_num_` 控制并行度

## 3. 持久化存储结构

文件组织（`file_names.h`）：

```
work_dir/
├── schema                           # Schema 序列化文件
├── checkpoint/                      # 检查点目录
├── wal/                             # Write-Ahead Log
│   ├── log_0
│   └── log_1
├── runtime/
│   ├── allocator/                   # 分配器数据
│   ├── tails/                       # 可变部分
│   ├── tmp/                         # 临时文件
│   └── update_txn_<version>/        # 事务数据
└── snapshots/                       # 快照目录
    ├── 0/                           # 初始快照
    │   ├── vertex_map_*.keys/.indices/.meta
    │   ├── vertex_table_*.col_*
    │   └── [io]e_*_*_*.deg/.nbr
    ├── 1234567/                     # 时间戳快照
    └── VERSION
```

**持久化策略：**
- 快照文件是**不可变**的
- 所有修改（Insert/Update）编码为 WAL
- Compaction 时合并快照与 WAL 生成新快照

## 4. 并发与 MVCC 机制

### 4.1 写入并发控制

| 组件 | 机制 |
|------|------|
| VertexTable 插入 | 原子计数器分配内部 ID |
| MutableCsr 边插入 | 每顶点 `SpinLock` |
| 内存分配 | `ArenaAllocator` 批量分配 |

### 4.2 MVCC 读取隔离

```
读事务 (read_ts)
    ↓
GenericView (timestamp = read_ts)
    ↓
NbrIterator 过滤
    ↓
只返回 timestamp <= read_ts 的边/顶点
```

**生命周期管理：**
- 邻接表通过 `ArenaAllocator` 管理，旧缓冲区不会立即释放
- 读取线程持有旧缓冲区的引用，直到遍历完成

## 5. 内存分配器

`ArenaAllocator` 设计：

```cpp
ArenaAllocator
├── batch_size = 16MB           // 批量分配单位
├── mmap_buffers_               // 内存映射缓冲区列表
├── cur_buffer_ / cur_loc_      // 当前缓冲区位置
└── strategy_                   // 内存策略 (决定容器类型)
```

**分配策略：**
- 小对象（< 8MB）：从当前缓冲区切分
- 大对象（≥ 8MB）：直接分配独立缓冲区
- 根据 `MemoryLevel` 选择匿名映射或文件映射

## 6. 关键设计模式总结

| 模式 | 应用场景 |
|------|----------|
| 接口分离 | Read/Insert/Update 三种接口 |
| 模板特化 | `MutableNbr<EmptyType>` 优化空数据 |
| 策略模式 | `MemoryLevel`、`EdgeStrategy`、`CsrType` |
| 工厂模式 | `LoaderFactory`、容器创建 |
| 视图模式 | `GenericView` 提供只读快照 |
| 迭代器模式 | `NbrIterator`、`VertexSet::iterator` |
| 访问者模式 | `EdgeDataAccessor` 统一属性访问 |

## 7. 性能优化要点

1. **列式顶点属性**：按需加载，减少内存访问
2. **行式边属性**：边数量大，行式更紧凑
3. **mmap 虚拟内存**：支持超大规模图，按需换页
4. **大页支持**：减少 TLB miss，提升遍历性能
5. **CSR 布局**：邻接表连续存储，缓存友好
6. **SpinLock 粒度**：每顶点锁，高并发友好
7. **MVCC 无锁读**：读事务不阻塞写事务
