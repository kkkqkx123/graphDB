# NeuG 架构分析

## 项目概述

**NeuG**是由阿里巴巴 GraphScope 团队开发的**图数据库引擎**，面向 **HTAP（混合事务/分析处理）** 工作负载。当前版本 **0.1.1**，使用 **C++20** 编写，采用 **CMake** 构建系统，基于 **Apache License 2.0** 开源。

### 双模式架构

| 模式                         | 用途           | 优化方向                            |
| ---------------------------- | -------------- | ----------------------------------- |
| **嵌入模式 (Embedded Mode)** | 分析型工作负载 | 批量数据加载、复杂模式匹配、图分析  |
| **服务模式 (Service Mode)**  | 事务型工作负载 | 实时应用、并发用户访问（HTTP/BRPC） |

### 核心特性

- **查询语言**: Cypher（图查询语言，编译器借鉴 Kùzu）
- **运行时值系统**: 借鉴 DuckDB 架构
- **扩展框架**: DuckDB 风格的动态扩展加载
- **Python 客户端**: `pip install neug`
- **Java 驱动**: 独立的 Java 客户端库

---

## 顶层目录结构

```
neug-main/
├── src/                    # 核心 C++ 源码
│   ├── main/               # 入口：数据库生命周期、连接管理
│   ├── compiler/           # Cypher 编译器（解析→绑定→优化→计划）
│   ├── execution/          # 执行引擎（Pipeline + 算子）
│   ├── storages/           # 存储层（CSR、mmap、图存储）
│   ├── transaction/        # MVCC 事务（WAL、版本管理）
│   ├── server/             # HTTP 服务层（BRPC）
│   ├── utils/              # 跨层工具库
│   └── common/             # 共享类型
├── include/neug/           # 公共 API 头文件
├── proto/                  # Protocol Buffer 定义（IR、物理计划）
├── extension/              # 可插拔扩展（JSON、Parquet）
├── tools/                  # Python 绑定、Java 驱动
├── third_party/            # 第三方依赖
├── tests/                  # 单元测试、E2E 测试
├── examples/               # C++ 使用示例
├── cmake/                  # CMake 辅助脚本
├── scripts/                # 构建/检查脚本
└── doc/                    # Sphinx 文档
```

---

## 模块架构详解

### 1. Main 模块 — 数据库入口

```
src/main/
├── neug_db.cpp             # NeugDB 类：数据库生命周期（Open/Close）
├── connection.cpp           # Connection 类：查询执行接口
├── query_processor.cpp      # QueryProcessor：编译→执行管线
├── connection_manager.cpp   # 连接池管理
├── query_request.cpp        # 请求数据结构
├── query_result.cpp         # 结果数据结构
└── file_lock.cpp            # 文件级锁（防止并发进程访问）
```

**核心职责**：

- `NeugDB`：管理数据库打开/关闭、WAL 重放、Checkpoint/Compaction
- `Connection`：提供查询接口，支持 read/insert/update/schema 访问模式
- `QueryProcessor`：查询编译与执行管线，含查询缓存

---

### 2. Compiler 模块 — Cypher 查询编译器

```
src/compiler/
├── parser/                  # ANTLR 词法/语法分析
│   ├── antlr_parser/        # Cypher 语法文件
│   ├── expression/          # 解析表达式类型
│   └── transform/           # 解析树→已解析语句（17 种转换）
├── binder/                  # 名称解析、类型检查
│   ├── expression/          # 绑定表达式（聚合、case、字面量等）
│   ├── bind/                # 语句绑定（21 种：DDL、DML、查询等）
│   ├── query/               # 绑定查询子句
│   └── rewriter/            # 查询重写
├── catalog/                 # 目录管理
│   └── catalog_entry/       # 目录条目（函数、索引、节点表、关系表等）
├── planner/                 # 逻辑计划生成
│   ├── operator/            # 逻辑算子（extend、scan、join、aggregate 等 30+）
│   ├── plan/                # 计划构建（32 种 append/plan 文件）
│   └── join_order/          # 连接顺序优化（基数估计、代价模型）
├── optimizer/               # 优化器（23 种优化 Pass）
│   ├── push_down/           # 谓词/投影/限制下推
│   └── rewrite/             # 逻辑重写
├── gopt/                    # 图优化器（专用图查询规划器）
├── function/                # 内置函数实现
│   ├── aggregate/           # 聚合函数（avg、count、min、max、sum）
│   ├── arithmetic/          # 算术函数
│   ├── list/                # 列表操作
│   ├── path/                # 路径函数
│   └── gds/                 # 图数据科学（递归连接）
├── common/                  # 编译器基础设施
│   ├── types/               # 数据类型（日期、时间、UUID、int128 等）
│   ├── vector/              # 列式向量表示
│   ├── arrow/               # Apache Arrow 集成
│   └── task_system/         # 任务调度
├── graph/                   # 图抽象层
├── extension/               # 扩展加载管理
└── transaction/             # 编译器侧事务处理
```

**编译管线**：

```
Cypher 字符串 → Parser(ANTLR) → 解析树 → Transformer → 已解析语句
    → Binder → 绑定语句 → Planner → 逻辑计划
    → Optimizer → 优化逻辑计划 → GOPT/Greedy Planner → 物理计划(Protobuf)
```

---

### 3. Execution 模块 — 运行时执行引擎

```
src/execution/
├── execute/                 # Pipeline 执行
│   ├── pipeline.cc          # Pipeline 执行（算子链）
│   ├── plan_parser.cc       # 从 Protobuf 解析物理计划
│   └── ops/                 # 物理算子实现
│       ├── retrieve/        # 查询算子（scan、select、join、project 等 21 种）
│       ├── insert/          # 单顶点/边插入
│       ├── batch/           # 批量 DML（批量插入/删除/更新顶点/边）
│       ├── ddl/             # 模式 DDL（创建/删除/重命名顶点/边类型）
│       └── admin/           # 管理操作（Checkpoint、扩展管理）
├── expression/              # 表达式求值
│   ├── expr.cc              # 表达式求值基类
│   ├── exprs/               # 算术、case-when、路径、结构体、UDF 等
│   └── accessors/           # 常量、边、记录、顶点访问器
├── common/                  # 执行公共组件
│   ├── types/               # 图类型、值
│   ├── columns/             # 列实现（Arrow、边、列表、路径、结构体、顶点）
│   └── operators/           # 插入/检索算子接口
└── extension/               # 运行时扩展加载
```

**执行模型**：采用 **Pipeline 执行模型**，算子链接成 Pipeline 实现向量化执行。

---

### 4. Storage 模块 — 物理存储层

```
src/storages/
├── graph/                   # 图存储核心
│   ├── property_graph.cpp   # PropertyGraph：顶点/边表、模式、CRUD、Compaction
│   ├── vertex_table.cpp     # 顶点表存储
│   ├── edge_table.cpp       # 边表存储
│   ├── schema.cpp           # 模式管理（标签、属性、约束）
│   └── vertex_timestamp.cpp # MVCC 时间戳管理
├── csr/                     # CSR（压缩稀疏行）图格式
│   ├── immutable_csr.cpp    # 只读 CSR（快照数据）
│   └── mutable_csr.cpp      # 可变 CSR（事务写入）
├── container/               # mmap 容器基础设施
│   ├── mmap_container.cpp   # mmap 容器基类
│   ├── file_mmap_container  # 文件映射容器
│   └── anon_mmap_container  # 匿名映射容器
└── loader/                  # 数据加载
    ├── abstract_property_graph_loader.cpp
    ├── csv_property_graph_loader.cpp
    └── loader_factory.cpp
```

**存储设计**：

- **CSR 分层**：Immutable CSR（快照层）+ Mutable CSR（增量层）
- **内存映射**：mmap 容器支持高效 I/O（sync-to-disk / mmap / hugepages 三级）
- **页面大小**：4KB（`NEUG_PAGE_SIZE_LOG2=12`）
- **节点组**：131072 个节点/组（`NEUG_NODE_GROUP_SIZE_LOG2=17`）

---

### 5. Transaction 模块 — MVCC 事务

```
src/transaction/
├── read_transaction.cpp      # 只读快照事务
├── insert_transaction.cpp    # 顶点/边插入（含 WAL）
├── update_transaction.cpp    # 属性更新、模式变更、undo log
├── compact_transaction.cpp   # 数据压缩事务
├── version_manager.cpp       # MVCC 版本管理
├── undo_log.cpp              # 回滚 Undo Log
└── wal/                      # 预写式日志
    ├── wal.cpp
    ├── local_wal_writer.cpp  # 本地文件 WAL
    ├── local_wal_parser.cpp  # WAL 解析
    └── dummy_wal_writer.cpp  # 无操作 WAL（只读模式）
```

**事务设计**：

- **MVCC**：基于时间戳的快照隔离
- **WAL**：预写式日志保证崩溃恢复
- **Undo Log**：支持事务回滚

---

### 6. Server 模块 — HTTP 服务层

```
src/server/
├── neug_db_service.cc        # BRPC HTTP 服务
├── neug_db_session.cc        # 会话管理
├── session_pool.cc           # 连接池
└── brpc_service_mgr.cc       # BRPC 服务管理器
```

构建时需开启 `BUILD_HTTP_SERVER=ON`，依赖 BRPC + LevelDB。

---

### 7. Utils 模块 — 工具库

```
src/utils/
├── exception/                # 异常处理与错误信息
├── file_sys/                 # 文件系统抽象
├── property/                 # 属性系统（列、属性、表、类型）
├── reader/                   # 数据读取器
├── writer/                   # 数据写入器
├── arrow_utils.cc            # Arrow 工具
├── bitset.cc                 # 位集合
├── encoder.cc                # 编码器
├── serialization/            # 序列化
└── yaml_utils.cc             # YAML 工具
```

头文件中还包含：`id_indexer.h`、`indexers.h`、`spinlock.h`、`string_utils.h`、`bolt_utils.h`、`pb_utils.h` 等。

---

## Protocol Buffer 定义

`proto/` 目录定义了查询的**中间表示（IR）**：

| 文件                     | 用途                                                                |
| ------------------------ | ------------------------------------------------------------------- |
| `basic_type.proto`       | 原始类型（int/bool/float/double、字符串、时间类型、数组/元组/映射） |
| `common.proto`           | 共享类型：Value、NameOrId、数组                                     |
| `type.proto`             | IrDataType（统一类型系统）、GraphDataType                           |
| `expr.proto`             | 表达式树：逻辑/算术运算、变量、属性、case、函数、UDF                |
| `schema.proto`           | 模式定义：LabelMeta、ColumnMeta、EntityMeta、RelationMeta           |
| `algebra.proto`          | **逻辑计划算子**：Project、Select、Join、Union、GroupBy 等          |
| `physical.proto`         | **物理计划算子**：逻辑算子 + DML + DDL + 管理操作                   |
| `cypher_ddl.proto`       | DDL 操作：创建/删除/重命名顶点/边模式和属性                         |
| `cypher_dml.proto`       | DML 操作：数据源、导出、批量插入、设置、删除                        |
| `stored_procedure.proto` | 存储过程定义                                                        |
| `http_svc.proto`         | HTTP 服务：GetSchema、GetServiceStatus、PostCypherQuery             |
| `response.proto`         | 查询响应：类型化数组、QueryResponse                                 |
| `error.proto`            | 错误码（1000-9999）：权限、版本、锁、损坏、编译、事务、模式等       |

---

## 扩展框架

```
extension/
├── json/                    # JSON 格式扩展
│   ├── include/
│   ├── src/
│   └── tests/
└── parquet/                 # Parquet 格式扩展
    ├── include/
    ├── src/
    └── tests/
```

通过 `BUILD_EXTENSIONS=json;parquet` 启用，采用 DuckDB 风格的动态扩展加载模型。

---

## 第三方依赖

| 库                               | 用途                           |
| -------------------------------- | ------------------------------ |
| Apache Arrow                     | 列式数据交换、Parquet/CSV 支持 |
| Protobuf                         | IR 序列化、RPC                 |
| ANTLR4                           | Cypher 查询解析                |
| glog/gflags                      | 日志和命令行参数               |
| mimalloc                         | 高性能内存分配器               |
| OpenSSL                          | 加密（HTTPS、认证）            |
| yaml-cpp                         | YAML 配置解析                  |
| RE2                              | 正则表达式引擎                 |
| utf8proc                         | Unicode 字符串处理             |
| RapidJSON                        | JSON 解析                      |
| pybind11                         | Python 绑定                    |
| BRPC（可选）                     | HTTP 服务框架                  |
| LevelDB（可选）                  | HTTP 服务元数据存储            |
| flat_hash_map / parallel-hashmap | 高性能哈希表                   |
| date (Howard Hinnant)            | 日期时间处理                   |
| expected                         | std::expected 风格错误处理     |
| fast_float                       | 快速数字解析                   |
| glob                             | 文件通配匹配                   |

---

## 查询执行完整数据流

```
┌─────────────────────────────────────────┐
│           用户 Cypher 查询               │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│         Connection::Query()              │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│      QueryProcessor::execute()           │
│      （检查查询缓存）                      │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│  Parser (ANTLR) → 解析树                  │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│  Transformer → 已解析语句                 │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│  Binder → 绑定语句（名称解析、类型检查）    │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│  Planner → 逻辑计划（DAG 逻辑算子）        │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│  Optimizer → 优化逻辑计划                 │
│  （下推、重写、连接重排序）                │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│  GOPT / Greedy Planner → 物理计划(Proto) │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│  Plan Parser → Pipeline<IOperator>       │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│  Pipeline::Execute()                     │
│  Scan → Select → Join → Project → ...   │
└──────────────┬──────────────────────────┘
               ▼
┌─────────────────────────────────────────┐
│         QueryResult 返回                  │
└─────────────────────────────────────────┘
```

---

## 核心设计模式

| 模式                   | 描述                                                        |
| ---------------------- | ----------------------------------------------------------- |
| **对象库聚合**         | 所有子模块构建为 OBJECT 库，聚合为单一 `libneug` 共享库     |
| **Protobuf 为中心 IR** | 逻辑/物理计划均定义为 Protobuf 消息，支持序列化和跨语言兼容 |
| **MVCC + 时间戳**      | 基于时间戳的快照隔离并发控制                                |
| **CSR 图存储**         | 压缩稀疏行格式 + 不可变（快照）/可变（增量）分层            |
| **内存映射存储**       | mmap 容器实现高效 I/O，支持三级内存策略                     |
| **Pipeline 执行**      | 算子链式连接为 Pipeline，实现向量化执行                     |
| **动态扩展**           | 运行时扩展加载 + 表函数注册（DuckDB 风格）                  |
| **Arena 分配器**       | 自定义内存分配器 + mmap 批量分配                            |
| **WAL 持久化**         | 预写式日志保障崩溃恢复                                      |
| **双模式运行**         | 嵌入模式（直接库调用）vs 服务模式（HTTP/BRPC 服务器）       |

---

## 构建选项

| 选项                | 默认值 | 说明                                 |
| ------------------- | ------ | ------------------------------------ |
| `BUILD_COMPILER`    | ON     | 构建 Cypher 编译器                   |
| `BUILD_HTTP_SERVER` | OFF    | 启用 HTTP 服务模式（BRPC + LevelDB） |
| `BUILD_EXECUTABLES` | OFF    | 构建可执行工具                       |
| `BUILD_TEST`        | OFF    | 构建测试套件                         |
| `BUILD_PYTHON`      | ON     | 构建 Python 绑定（pybind11）         |
| `BUILD_EXTENSIONS`  | ""     | 扩展列表（如 `json;parquet`）        |
| `WITH_MIMALLOC`     | ON     | 使用 mimalloc 分配器                 |
| `ENABLE_BACKTRACES` | OFF    | 启用 cpptrace 栈回溯                 |
| `ENABLE_GCOV`       | OFF    | 启用代码覆盖率                       |
| `ENABLE_LTO`        | OFF    | 链接时优化                           |

**关键性能参数**：

- `NEUG_PAGE_SIZE_LOG2=12`（页面大小 4KB）
- `NEUG_VECTOR_CAPACITY_LOG2=11`（向量容量 2048）
- `NEUG_NODE_GROUP_SIZE_LOG2=17`（节点组大小 131072）

---

## 测试体系

```
tests/
├── unittest/                # 单元测试
├── compiler/                # 编译器测试
├── execution/               # 执行引擎测试
├── storage/                 # 存储层测试
├── transaction/             # 事务测试
├── main/                    # 主模块测试
├── e2e/                     # 端到端查询测试
└── utils/                   # 测试工具
```

---

## 关键文件索引

| 路径                                    | 作用                     |
| --------------------------------------- | ------------------------ |
| `CMakeLists.txt`                        | 主构建配置               |
| `src/CMakeLists.txt`                    | 子模块构建、libneug 链接 |
| `include/neug/neug.h`                   | 公共 API 入口            |
| `src/main/neug_db.cpp`                  | 数据库核心实现           |
| `src/main/query_processor.cpp`          | 查询处理管线             |
| `src/storages/graph/property_graph.cpp` | 图存储核心               |
| `src/execution/execute/pipeline.cc`     | Pipeline 执行引擎        |
| `src/transaction/version_manager.cpp`   | MVCC 版本管理            |
| `proto/algebra.proto`                   | 逻辑计划定义             |
| `proto/physical.proto`                  | 物理计划定义             |
