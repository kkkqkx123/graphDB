# GraphDB 与 Nebula-Graph 上下文类型对比分析

## 概述

本文档详细对比 GraphDB（Rust）和 Nebula-Graph（C++）在查询上下文模块中的类型设计差异。通过系统性分析，揭示两个项目在架构设计、类型系统、并发模型等方面的设计哲学差异。

---

## 一、上下文类型总览

### 1.1 GraphDB 上下文类型

| 上下文类型 | 文件位置 | 主要职责 |
|-----------|----------|----------|
| RequestContext | `request_context.rs` | 请求生命周期管理 |
| QueryContext | `execution/query_execution.rs` | 聚合查询相关的所有上下文 |
| ValidationContext | `validate/context.rs` | 语义验证阶段上下文 |
| BasicValidationContext | `validate/basic_context.rs` | 基础验证上下文 |
| QueryExecutionContext | `execution/query_execution.rs` | 执行时变量管理 |
| SymbolTable | `symbol/symbol_table.rs` | 符号表管理 |
| RuntimeContext | `runtime_context.rs` | 存储层运行时上下文 |
| SchemaManager | `managers/schema_manager.rs` | Schema管理接口 |
| IndexManager | `managers/index_manager.rs` | 索引管理接口 |
| StorageClient | `managers/storage_client.rs` | 存储层客户端接口 |
| MetaClient | `managers/meta_client.rs` | 元数据客户端接口 |
| TransactionManager | `managers/transaction.rs` | 事务管理 |

### 1.2 Nebula-Graph 上下文类型

| 上下文类型 | 文件位置 | 主要职责 |
|-----------|----------|----------|
| RequestContext | `service/RequestContext.h` | 请求生命周期管理 |
| QueryContext | `context/QueryContext.h` | 聚合查询相关的所有上下文 |
| ExecutionContext | `context/ExecutionContext.h` | 执行时变量和结果管理 |
| ValidateContext | `context/ValidateContext.h` | 语义验证阶段上下文 |
| SymbolTable | `context/Symbols.h` | 符号表管理 |
| QueryExpressionContext | `context/QueryExpressionContext.h` | 表达式求值上下文 |
| Result | `context/Result.h` | 查询结果容器 |
| Iterator | `context/Iterator.h` | 结果迭代器抽象 |

---

## 二、核心上下文详细对比

### 2.1 请求上下文对比

#### GraphDB RequestContext

**文件**: `request_context.rs`

```rust
#[derive(Debug, Clone)]
pub struct RequestContext {
    session_info: Option<SessionInfo>,
    request_params: Arc<RwLock<RequestParams>>,
    response: Arc<RwLock<Response>>,
    start_time: SystemTime,
    status: Arc<RwLock<RequestStatus>>,
    attributes: Arc<RwLock<HashMap<String, Value>>>,
    cancelled: Arc<AtomicBool>,
    timed_out: Arc<AtomicBool>,
    execution_count: Arc<AtomicU64>,
    logs: Arc<RwLock<Vec<RequestLog>>>,
    statistics: Arc<RwLock<RequestStatistics>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct RequestStatistics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub cancelled_queries: u64,
    pub timed_out_queries: u64,
    pub total_execution_time_ms: u64,
    pub avg_execution_time_ms: f64,
    pub max_execution_time_ms: u64,
    pub min_execution_time_ms: u64,
}
```

**设计特点**:
- 使用 `Arc<RwLock>` 实现线程安全
- 支持请求级别的统计和日志
- 包含完整的生命周期状态管理
- 支持自定义属性扩展

#### Nebula-Graph RequestContext

**文件**: `service/RequestContext.h`

```cpp
template<typename T>
class RequestContext {
 public:
  using RequestContextPtr = std::unique_ptr<RequestContext>;

  RequestContext() = default;

  void setRequest(T&& req) {
    request_ = std::move(req);
  }

  T& request() {
    return request_;
  }

  void setResponse(GraphResponse&& resp) {
    response_ = std::move(resp);
  }

  GraphResponse& response() {
    return response_;
  }

 private:
  T request_;
  GraphResponse response_;
  std::atomic<bool> finished_{false};
};
```

**设计特点**:
- 模板化设计，支持不同请求类型
- 使用 `unique_ptr` 管理生命周期
- 简洁的状态管理（仅 `finished_` 标志）
- 无内置统计和日志功能

#### 对比分析

| 维度 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 状态管理 | 详细状态枚举 | 简单布尔标志 |
| 线程安全 | Arc+RwLock | 调用方保证 |
| 统计支持 | 内置完整统计 | 无 |
| 日志支持 | 内置日志系统 | 无 |
| 扩展性 | 自定义属性 | 模板化请求类型 |
| 复杂度 | 高 | 低 |

---

### 2.2 主查询上下文对比

#### GraphDB QueryContext

**文件**: `execution/query_execution.rs`

```rust
pub struct QueryContext {
    rctx: Option<Arc<RequestContext>>,
    vctx: ValidationContext,
    ectx: QueryExecutionContext,
    plan: Option<Box<ExecutionPlan>>,
    schema_manager: Option<Arc<dyn SchemaManager>>,
    index_manager: Option<Arc<dyn IndexManager>>,
    storage_client: Option<Arc<dyn StorageClient>>,
    meta_client: Option<Arc<dyn MetaClient>>,
    charset_info: Option<Box<CharsetInfo>>,
    obj_pool: ObjectPool<String>,
    id_gen: IdGenerator,
    sym_table: SymbolTable,
    killed: AtomicBool,
}
```

**设计特点**:
- 所有组件使用 `Arc` 包装
- 支持运行时组件替换
- 包含对象池和ID生成器
- 独立的符号表

#### Nebula-Graph QueryContext

**文件**: `context/QueryContext.h`

```cpp
class QueryContext {
 public:
  using RequestContextPtr = std::unique_ptr<RequestContext<ExecutionResponse>>;

  QueryContext(RequestContextPtr rctx,
               meta::SchemaManager* sm,
               meta::IndexManager* im,
               storage::StorageClient* storage,
               meta::MetaClient* metaClient,
               CharsetInfo* charsetInfo);

  void setRCtx(RequestContextPtr rctx) { rctx_ = std::move(rctx); }
  ValidateContext* vctx() const { return vctx_.get(); }
  ExecutionContext* ectx() const { return ectx_.get(); }
  ExecutionPlan* plan() const { return ep_.get(); }

  ObjectPool* objPool() const { return objPool_.get(); }
  int64_t genId() const { return idGen_->id(); }
  SymbolTable* symTable() const { return symTable_.get(); }

 private:
  void init();

  RequestContextPtr rctx_;
  std::unique_ptr<ValidateContext> vctx_;
  std::unique_ptr<ExecutionContext> ectx_;
  std::unique_ptr<ExecutionPlan> ep_;

  meta::SchemaManager* sm_{nullptr};
  meta::IndexManager* im_{nullptr};
  storage::StorageClient* storageClient_{nullptr};
  meta::MetaClient* metaClient_{nullptr};
  CharsetInfo* charsetInfo_{nullptr};

  std::unique_ptr<ObjectPool> objPool_;
  std::unique_ptr<IdGenerator> idGen_;
  std::unique_ptr<SymbolTable> symTable_;

  std::atomic<bool> killed_{false};
};
```

**设计特点**:
- 组件使用裸指针（外部保证有效性）
- 通过构造函数注入依赖
- 强制初始化顺序（`init()`）
- 符号表是 QueryContext 的组成部分

#### 对比分析

| 维度 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 组件所有权 | Arc 共享 | 裸指针（外部管理） |
| 依赖注入 | setter 方法 | 构造函数参数 |
| 符号表关系 | 独立字段 | 内部 owned 字段 |
| 对象池 | 泛型 ObjectPool | 原始 ObjectPool |
| 线程安全 | 显式 AtomicBool | 显式 AtomicBool |
| 可测试性 | 高（可替换组件） | 中（需 mock 指针） |

---

### 2.3 验证上下文对比

#### GraphDB ValidationContext

**文件**: `validate/context.rs`

```rust
#[derive(Clone)]
pub struct ValidationContext {
    basic_context: BasicValidationContext,
    schema_manager: Option<Arc<dyn SchemaProvider>>,
    anon_var_gen: AnonVarGenerator,
    anon_col_gen: AnonColGenerator,
    symbol_table: SymbolTable,
    schemas: HashMap<String, SchemaInfo>,
    query_parts: Vec<QueryPart>,
    alias_types: HashMap<String, AliasType>,
    validation_errors: Vec<ValidationError>,
}
```

**层次结构**:
```
BasicValidationContext
    └── ValidationContext (组合而非继承)
```

#### Nebula-Graph ValidateContext

**文件**: `context/ValidateContext.h`

```cpp
class ValidateContext final {
 public:
  explicit ValidateContext(std::unique_ptr<AnonVarGenerator> varGen) {
    anonVarGen_ = std::move(varGen);
    anonColGen_ = std::make_unique<AnonColGenerator>();
  }

  void switchToSpace(SpaceInfo space) { spaces_.emplace_back(std::move(space)); }
  const ColsDef& getVar(const std::string& var) const { /* ... */ }
  bool existVar(const std::string& var) const { /* ... */ }
  void registerVariable(std::string var, ColsDef cols) { vars_.emplace(std::move(var), std::move(cols)); }

  AnonVarGenerator* anonVarGen() const { return anonVarGen_.get(); }
  AnonColGenerator* anonColGen() const { return anonColGen_.get(); }

 private:
  std::vector<SpaceInfo> spaces_;
  std::unordered_map<std::string, ColsDef> vars_;
  std::unique_ptr<AnonVarGenerator> anonVarGen_;
  std::unique_ptr<AnonColGenerator> anonColGen_;
  Schemas schemas_;
  std::unordered_set<std::string> createSpaces_;
  std::unordered_set<std::string> indexes_;
};
```

**设计特点**:
- 扁平结构，所有字段平铺
- 不包含符号表（符号表在 QueryContext 中）
- 使用 `final` 禁止继承

#### 对比分析

| 维度 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 结构层次 | 组合模式 | 扁平结构 |
| 符号表 | 包含 | 不包含 |
| Schema管理 | 组合 SchemaProvider | 直接存储 schema map |
| 错误收集 | 内置 | 无（使用返回值） |
| 扩展方式 | 组合增强 | 字段添加 |

---

### 2.4 执行上下文对比

#### GraphDB QueryExecutionContext

**文件**: `execution/query_execution.rs`

```rust
#[derive(Debug, Clone)]
pub struct QueryExecutionContext {
    variables: HashMap<String, Value>,
}

impl QueryExecutionContext {
    pub fn set_value(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }
}
```

**设计特点**:
- 简单 HashMap 存储
- 无版本控制
- 无迭代器支持

#### Nebula-Graph ExecutionContext

**文件**: `context/ExecutionContext.h`

```cpp
class ExecutionContext {
 public:
  static constexpr int64_t kLatestVersion = 0;
  static constexpr int64_t kOldestVersion = 1;
  static constexpr int64_t kPreviousOneVersion = -1;

  void initVar(const std::string& name) { /* 预分配 */ }

  const Value& getValue(const std::string& name) const;
  const Result& getResult(const std::string& name) const;
  const Result& getVersionedResult(const std::string& name, int64_t version) const;

  void setVersionedResult(const std::string& name, Result&& result, int64_t version);
  size_t numVersions(const std::string& name) const;
  const std::vector<Result>& getHistory(const std::string& name) const;

  void truncHistory(const std::string& name, size_t numVersionsToKeep);

  bool exist(const std::string& name) const {
    folly::RWSpinLock::ReadHolder holder(lock_);
    return valueMap_.find(name) != valueMap_.end();
  }

 private:
  mutable folly::RWSpinLock lock_;
  std::unordered_map<std::string, std::vector<Result>> valueMap_;
};
```

**设计特点**:
- 多版本变量支持
- 使用 `folly::RWSpinLock` 并发控制
- 结果历史管理
- 包含 Result 而非仅 Value

#### 对比分析

| 维度 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 变量存储 | HashMap<String, Value> | HashMap<String, Vec<Result>> |
| 版本支持 | 无 | 多版本（kLatestVersion, kOldestVersion） |
| 并发控制 | 调用方保证 | RWSpinLock |
| 结果类型 | Value | Result（包含迭代器） |
| 历史管理 | 无 | 支持 truncate |

---

### 2.5 符号表对比

#### GraphDB SymbolTable

**文件**: `symbol/symbol_table.rs`

```rust
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub value_type: DataType,
    pub col_names: Vec<String>,
    pub readers: HashSet<PlanNodeRef>,
    pub writers: HashSet<PlanNodeRef>,
    pub source_clause: String,
    pub properties: Vec<String>,
    pub is_aggregated: bool,
}

pub struct SymbolTable {
    symbols: Arc<DashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new_variable(&mut self, name: &str) -> Result<Symbol, String> {
        if self.symbols.contains_key(name) {
            return Err(format!("变量 '{}' 已存在", name));
        }
        let symbol = Symbol::new(name.to_string(), DataType::DataSet);
        self.symbols.insert(name.to_string(), symbol.clone());
        Ok(symbol)
    }

    pub fn read_by(&mut self, var_name: &str, node: PlanNodeRef) -> Result<(), String> {
        if let Some(mut symbol) = self.symbols.get_mut(var_name) {
            symbol.readers.insert(node);
            Ok(())
        } else {
            Err(format!("变量 '{}' 不存在", var_name))
        }
    }
}
```

**设计特点**:
- 使用 DashMap 实现线程安全
- 跟踪读取者和写入者
- 包含丰富的元数据（source_clause, properties）
- 独立于 QueryContext

#### Nebula-Graph SymbolTable

**文件**: `context/Symbols.h`

```cpp
struct Variable {
  explicit Variable(std::string n) : name(std::move(n)) {}

  std::string name;
  Value::Type type{Value::Type::DATASET};
  std::vector<std::string> colNames;

  std::unordered_set<PlanNode*> readBy;
  std::unordered_set<PlanNode*> writtenBy;
  std::atomic<uint64_t> userCount{0};
};

class SymbolTable final {
 public:
  explicit SymbolTable(ObjectPool* objPool, ExecutionContext* ectx)
      : objPool_(DCHECK_NOTNULL(objPool)), ectx_(DCHECK_NOTNULL(ectx)) {}

  bool existsVar(const std::string& varName) const;
  Variable* newVariable(const std::string& name);
  bool readBy(const std::string& varName, PlanNode* node);
  bool writtenBy(const std::string& varName, PlanNode* node);
  Variable* getVar(const std::string& varName);

 private:
  ObjectPool* objPool_{nullptr};
  ExecutionContext* ectx_{nullptr};
  mutable folly::RWSpinLock lock_;
  std::unordered_map<std::string, Variable*> vars_;
};
```

**设计特点**:
- 依赖 ObjectPool 分配内存
- 依赖 ExecutionContext 管理生命周期
- 简单的读写者跟踪
- 与 QueryContext 紧密耦合

#### 对比分析

| 维度 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 内存管理 | DashMap 自动管理 | ObjectPool 手动分配 |
| 生命周期 | 独立 | 依赖 ExecutionContext |
| 线程安全 | 内置 | RWSpinLock |
| 依赖关系 | 无外部依赖 | 依赖 ObjectPool 和 Ectx |
| 元数据 | 丰富（source, properties） | 简单 |
| 使用统计 | 无 | userCount 原子计数 |

---

### 2.6 结果与迭代器对比

#### GraphDB 缺失迭代器抽象

**当前设计**:
```rust
pub struct ExecutionResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}
```

**缺失功能**:
- 无迭代器接口
- 无流式数据处理
- 无多种迭代器类型
- 无内存检查机制

#### Nebula-Graph Result 与 Iterator

**文件**: `context/Result.h`

```cpp
class Result final {
 public:
  enum class State : uint8_t {
    kUnExecuted,
    kPartialSuccess,
    kSuccess,
  };

  std::shared_ptr<Value> valuePtr() const { return core_.value; }
  const Value& value() const { return *core_.value; }
  State state() const { return core_.state; }
  size_t size() const { return core_.iter->size(); }

  std::unique_ptr<Iterator> iter() const& { return core_.iter->copy(); }
  Iterator* iterRef() const { return core_.iter.get(); }

  void checkMemory(bool checkMemory) {
    core_.checkMemory = checkMemory;
    if (core_.iter) core_.iter->setCheckMemory(checkMemory);
  }

 private:
  struct Core {
    bool checkMemory{false};
    State state;
    std::string msg;
    std::shared_ptr<Value> value;
    std::unique_ptr<Iterator> iter;
  };

  Result::Core core_;
};
```

**文件**: `context/Iterator.h`

```cpp
class Iterator {
 public:
  enum class Kind {
    kDefault,
    kSequential,
    kGetNeighbors,
    kProp
  };

  virtual ~Iterator() = default;
  virtual std::unique_ptr<Iterator> copy() const = 0;
  virtual bool valid() const = 0;
  virtual void next() = 0;
  virtual const Value& getValue() const = 0;
  virtual size_t size() const = 0;

  void setCheckMemory(bool checkMemory) { checkMemory_ = checkMemory; }

 protected:
  bool checkMemory_{false};
};
```

**多种迭代器实现**:
- `SequentialIter`: 顺序迭代
- `GetNeighborsIter`: 邻居遍历
- `PropIter`: 属性迭代
- `DefaultIter`: 默认迭代

#### 对比分析

| 维度 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 结果封装 | 简单结构体 | Result 类（含状态） |
| 迭代器 | 无 | 完整抽象（接口+实现） |
| 迭代器类型 | 无 | 4种+ |
| 内存检查 | 无 | checkMemory 支持 |
| 状态追踪 | 无 | kUnExecuted/kPartialSuccess/kSuccess |
| 拷贝能力 | N/A | iter()->copy() |

---

## 三、管理器接口对比

### 3.1 Schema 管理器

#### GraphDB SchemaManager

**文件**: `managers/schema_manager.rs`

```rust
pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    // 基本操作
    fn get_schema(&self, name: &str) -> Option<Schema>;
    fn list_schemas(&self) -> Vec<String>;
    fn has_schema(&self, name: &str) -> bool;

    // Tag 操作 (6 方法)
    fn create_tag(&self, space_id: i32, tag_name: &str, fields: Vec<FieldDef>) -> ManagerResult<i32>;
    fn drop_tag(&self, space_id: i32, tag_id: i32) -> ManagerResult<()>;
    fn get_tag(&self, space_id: i32, tag_id: i32) -> Option<TagDefWithId>;
    fn list_tags(&self, space_id: i32) -> ManagerResult<Vec<TagDefWithId>>;
    fn has_tag(&self, space_id: i32, tag_id: i32) -> bool;

    // EdgeType 操作 (6 方法)
    fn create_edge_type(&self, ...) -> ManagerResult<i32>;
    fn drop_edge_type(&self, ...) -> ManagerResult<()>;
    // ...

    // 版本控制 (6 方法)
    fn create_schema_version(&self, ...) -> ManagerResult<i32>;
    fn get_schema_version(&self, ...) -> Option<SchemaVersion>;
    // ...

    // 字段操作 (6 方法)
    fn add_tag_field(&self, ...) -> ManagerResult<()>;
    // ...

    // 变更历史 (3 方法)
    fn record_schema_change(&self, ...) -> ManagerResult<()>;
    // ...

    // 导出导入 (2 方法)
    fn export_schema(&self, ...) -> ManagerResult<String>;
    // ...
}
```

#### Nebula-Graph SchemaManager

**文件**: `common/meta/SchemaManager.h`

```cpp
class SchemaManager {
 public:
  virtual ~SchemaManager() = default;

  virtual std::unique_ptr<MetaServiceClient> client() = 0;

  virtual GraphSpaceID createSpace(const std::string& name,
                                    int partitionNum,
                                    int replicaFactor,
                                    const CharsetInfo* charsetInfo) = 0;

  virtual std::unique_ptr<Schema> createSchema(GraphSpaceID space,
                                                const std::string& name,
                                                bool ifNotExists) = 0;

  virtual bool dropSchema(GraphSpaceID space,
                          const std::string& name,
                          bool ifExists) = 0;

  virtual std::unique_ptr<Schema> getSchema(GraphSpaceID space,
                                             const std::string& name) = 0;

  virtual StatusOr<std::vector<SpaceInfo>> listSpaces() = 0;
  // ...
};
```

#### 对比分析

| 维度 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 接口大小 | 40+ 方法 | 约 15 核心方法 |
| 设计风格 | 单一 trait | 层次化接口 |
| 分布式支持 | 无 | 远程 meta 服务 |
| 版本控制 | 内置 | 通过 meta 服务 |

---

### 3.2 存储客户端

#### GraphDB StorageClient

**文件**: `managers/storage_client.rs`

```rust
pub trait StorageClient: Send + Sync + std::fmt::Debug {
    // 执行存储操作
    fn execute(&self, operation: StorageOperation) -> ManagerResult<StorageResponse>;
    fn is_connected(&self) -> bool;

    // Vertex 操作 (7 方法)
    fn add_vertex(&self, space_id: i32, vertex: Vertex) -> ManagerResult<ExecResponse>;
    fn add_vertices(&self, space_id: i32, vertices: Vec<NewVertex>) -> ManagerResult<ExecResponse>;
    fn get_vertex(&self, space_id: i32, vid: &Value) -> ManagerResult<Option<Vertex>>;
    // ...

    // Edge 操作 (7 方法)
    fn add_edge(&self, space_id: i32, edge: Edge) -> ManagerResult<ExecResponse>;
    fn add_edges(&self, space_id: i32, edges: Vec<NewEdge>) -> ManagerResult<ExecResponse>;
    // ...

    // 扫描操作 (6 方法)
    fn scan_vertices(&self, space_id: i32, limit: Option<usize>) -> ManagerResult<Vec<Vertex>>;
    // ...
}
```

#### Nebula-Graph StorageClient

**文件**: `clients/storage/StorageClient.h`

```cpp
class StorageClient {
 public:
  explicit StorageClient(std::shared_ptr<MetaClient> metaClient);

  folly::Future<StorageRpcResponse<ExecResponse>> addVertex(
      GraphSpaceID spaceId,
      const Vertex& vertex,
      const std::vector< std::string >& returnNames);

  folly::Future<StorageRpcResponse<cpp2::ExecResponse>> addEdge(
      GraphSpaceID spaceId,
      const Edge& edge,
      const std::vector< std::string >& returnNames);

  folly::Future<StorageRpcResponse<nebula::cpp2::QueryResponse>> scanVertex(
      GraphSpaceID spaceId,
      const std::vector< std::string >& returnColumns,
      const StorageGraphType& graphType);

  folly::Future<StorageRpcResponse<nebula::cpp2::QueryResponse>> scanEdge(
      GraphSpaceID spaceId,
      const std::string& edgeType,
      const std::vector< std::string >& returnColumns,
      const StorageGraphType& graphType);
  // ...
};
```

#### 对比分析

| 维度 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 通信模式 | 同步调用 | 异步 Future |
| 分布式支持 | 无 | RPC 远程调用 |
| 响应处理 | 直接返回 | RPC 响应封装 |
| 批量操作 | 显式方法 | 批量接口 |

---

## 四、上下文类型层次结构

### 4.1 GraphDB 上下文层次

```
RequestContext (请求级)
    └── QueryContext (查询级)
            ├── vctx: ValidationContext
            │       ├── BasicValidationContext
            │       └── symbol_table: SymbolTable
            ├── ectx: QueryExecutionContext
            └── sym_table: SymbolTable (独立)
```

### 4.2 Nebula-Graph 上下文层次

```
RequestContext<ExecutionResponse> (请求级)
    └── QueryContext (查询级)
            ├── vctx: ValidateContext
            ├── ectx: ExecutionContext
            │       └── valueMap: HashMap<String, Vec<Result>>
            └── symTable: SymbolTable (内部 Owned)
                    └── Variable (ObjectPool 分配)
```

---

## 五、设计哲学差异总结

### 5.1 内存管理

| 方面 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 智能指针 | Arc, Rc | shared_ptr, unique_ptr |
| 手动管理 | 无 | ObjectPool |
| 引用计数 | 显式 Arc | 模板 shared_ptr |
| 并发安全 | 类型级保证 | 运行时锁 |

### 5.2 依赖管理

| 方面 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 依赖注入 | setter / Arc | 构造函数参数 |
| 指针类型 | Arc<dyn Trait> | 裸指针 |
| 生命周期 | 自动推断 | 外部保证 |
| 可替换性 | 高 | 低 |

### 5.3 错误处理

| 方面 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 错误类型 | ManagerError 枚举 | ErrorCode 枚举 |
| 返回模式 | Result<T, E> | StatusOr<T> |
| 错误传播 | ? 操作符 | 手动处理 |
| 用户友好 | 部分支持 | 通过 msg 字段 |

### 5.4 扩展性

| 方面 | GraphDB | Nebula-Graph |
|------|---------|--------------|
| 新增上下文 | 组合现有类型 | 修改 QueryContext |
| 迭代器 | 不支持 | 多态接口 |
| 插件机制 | trait 系统 | 继承+虚函数 |
| 配置扩展 | attributes 字段 | 请求模板化 |

---

## 六、迁移与改进建议

### 6.1 短期改进（高优先级）

1. **引入迭代器系统**
   - 定义 `ResultIterator` trait
   - 实现 `SequentialIter`, `GetNeighborsIter`
   - 在 `ExecutionResponse` 中添加迭代器字段

2. **改进类型系统**
   - 用 `DataType` 枚举替换 `String`
   - 添加字段级元数据

3. **统一错误处理**
   - 定义 `QueryError` 枚举
   - 实现 From trait 支持

### 6.2 中期改进（中优先级）

1. **拆分 QueryContext**
   - 分离 CoreQueryContext
   - 分离 ComponentAccess

2. **改进 SchemaManager 接口**
   - 拆分为多个 trait
   - 实现渐进式接口

3. **添加表达式上下文**
   - 实现 `ExpressionContext`
   - 支持变量/属性访问

### 6.3 长期改进（低优先级）

1. **优化并发模型**
   - 根据使用场景选择同步/异步
   - 移除不必要的 Arc<RwLock>

2. **引入优化器上下文**
   - 添加 OptContext
   - 支持代价模型

3. **增强测试覆盖**
   - 并发场景测试
   - 边界条件测试

---

## 七、上下文类型速查表

### GraphDB 快速参考

| 类型 | 职责 | 线程安全 |
|------|------|----------|
| RequestContext | 请求生命周期 | Arc<RwLock> |
| QueryContext | 主查询上下文 | 依赖组件 |
| ValidationContext | 语义验证 | 依赖组件 |
| QueryExecutionContext | 变量管理 | 否 |
| SymbolTable | 符号表 | DashMap |
| RuntimeContext | 存储运行时 | 否 |

### Nebula-Graph 快速参考

| 类型 | 职责 | 线程安全 |
|------|------|----------|
| RequestContext | 请求生命周期 | 调用方保证 |
| QueryContext | 主查询上下文 | 原子标志 |
| ValidateContext | 语义验证 | 调用方保证 |
| ExecutionContext | 变量+结果 | RWSpinLock |
| SymbolTable | 符号表 | RWSpinLock |
| Result | 查询结果 | 依赖 Iterator |

---

## 附录：完整类型映射表

| GraphDB | Nebula-Graph | 对应关系 |
|---------|--------------|----------|
| RequestContext | RequestContext | 请求级 |
| QueryContext | QueryContext | 查询级 |
| ValidationContext | ValidateContext | 验证级 |
| BasicValidationContext | - | 子集 |
| QueryExecutionContext | ExecutionContext | 执行级 |
| RuntimeContext | 存储层上下文 | 存储级 |
| SymbolTable | SymbolTable | 符号表 |
| - | QueryExpressionContext | 表达式级 |
| - | Result | 结果级 |
| - | Iterator | 迭代器 |
| SchemaManager | SchemaManager | Schema管理 |
| IndexManager | IndexManager | 索引管理 |
| StorageClient | StorageClient | 存储访问 |
| MetaClient | MetaClient | 元数据访问 |
| TransactionManager | - | 事务管理（新增） |
