# NebulaGraph 架构分析与 GraphDB 改造方案

## 一、NebulaGraph 核心架构剖析

### 1.1 分层架构设计

NebulaGraph 采用清晰的分层架构，从底层存储到顶层服务形成完整的调用链。

**架构层次图：**

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Graph Layer (src/graph)                    │
│              查询解析、优化器、执行计划生成                           │
├─────────────────────────────────────────────────────────────────────┤
│                       Meta Layer (src/meta)                         │
│              元数据管理：Space、Tag、Edge、Index 定义                │
├─────────────────────────────────────────────────────────────────────┤
│                      Storage Layer (src/storage)                    │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                    Query Interface                            │  │
│  │     GetVerticesProcessor / GetEdgesProcessor / ScanProcessor  │  │
│  ├───────────────────────────────────────────────────────────────┤  │
│  │                    Processor Layer                            │  │
│  │          BaseProcessor<RESP> ← QueryBaseProcessor<REQ,RESP>   │  │
│  ├───────────────────────────────────────────────────────────────┤  │
│  │                    Execution Layer                            │  │
│  │           IndexNode Tree (IndexScan / IndexLimit / ...)       │  │
│  ├───────────────────────────────────────────────────────────────┤  │
│  │                    Storage Layer                              │  │
│  │           RocksDB / NebulaStore / KVStore                     │  │
│  └───────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### 1.2 BaseProcessor 模板基类设计

`BaseProcessor<RESP>` 是整个存储层的核心基类，采用模板方法模式实现通用功能：

```cpp
template <typename RESP>
class BaseProcessor {
 public:
  explicit BaseProcessor(StorageEnv* env, const ProcessorCounters* counters = nullptr)
      : env_(env), counters_(counters) {}

  // 模板方法模式：提供通用流程，子类实现具体逻辑
  virtual void onFinished() {
    memory::MemoryCheckOffGuard guard;
    if (counters_) {
      stats::StatsManager::addValue(counters_->numCalls_);
      if (!this->codes_.empty()) {
        stats::StatsManager::addValue(counters_->numErrors_);
      }
    }
    
    this->result_.latency_in_us_ref() = this->duration_.elapsedInUSec();
    this->result_.failed_parts_ref() = std::move(this->codes_);
    this->resp_.result_ref() = std::move(this->result_);
    this->promise_.setValue(std::move(this->resp_));
    
    delete this;  // 自销毁模式
  }

  virtual void onError() {
    // 错误处理通用逻辑
  }

 protected:
  StorageEnv* env_;                    // 存储环境（包含 kvstore、schemaMan 等）
  const ProcessorCounters* counters_;  // 统计计数器
  nebula::cpp2::ErrorCode code_;       // 错误码
  RESP resp_;                          // 响应类型（模板参数）
  folly::Promise<RESP> promise_;       // Future/Promise 异步支持
  time::Duration duration_;            // 执行时间统计
};
```

**设计要点：**

| 特性 | 实现方式 | 优势 |
|-----|---------|------|
| 响应类型泛化 | `template <typename RESP>` | 支持不同查询返回不同类型 |
| 异步处理 | `folly::Future<RESP>` | 高并发场景下的非阻塞 IO |
| 内存管理 | 自销毁模式 | 自动释放处理器资源 |
| 统计监控 | ProcessorCounters | 细粒度的性能监控 |
| 错误传播 | 统一的 ErrorCode | 分布式场景下的错误传递 |

### 1.3 IndexNode 执行计划树

NebulaGraph 的执行层采用**组合模式**构建树形执行计划：

```cpp
class IndexNode {
 public:
  IndexNode(RuntimeContext* context, const std::string& name);
  virtual ~IndexNode() = default;

  // 组合模式：支持子节点
  void addChild(std::unique_ptr<IndexNode> child) {
    children_.emplace_back(std::move(child));
  }

  // 执行生命周期：init → execute → next
  virtual ::nebula::cpp2::ErrorCode init(InitContext& initCtx) {
    // 默认初始化子节点
    return children_[0]->init(initCtx);
  }

  inline nebula::cpp2::ErrorCode execute(PartitionID partId) {
    beforeExecute();
    auto ret = doExecute(partId);
    afterExecute();
    return ret;
  }

  inline Result next() {
    beforeNext();
    if (context_->isPlanKilled()) {
      return Result(::nebula::cpp2::ErrorCode::E_PLAN_IS_KILLED);
    }
    Result ret = doNext();
    afterNext();
    return ret;
  }

 protected:
  virtual Result doNext() = 0;  // 子类实现核心逻辑
  virtual nebula::cpp2::ErrorCode doExecute(PartitionID partId);

 private:
  void beforeExecute();
  void afterExecute();
  void beforeNext();
  void afterNext();

  RuntimeContext* context_;                    // 运行时上下文
  GraphSpaceID spaceId_;                       // 图空间 ID
  std::vector<std::unique_ptr<IndexNode>> children_;  // 子节点
  std::string name_;                           // 节点名称
  time::Duration duration_;                    // 执行时间统计
  bool profileDetail_{false};                  // 是否记录性能详情
};
```

**具体节点实现示例：**

```cpp
class IndexScanNode : public IndexNode {
 public:
  IndexScanNode(RuntimeContext* context, bool isVertex, 
                std::vector<PartitionID> parts,
                std::vector<storage::cpp2::IndexQueryHint> hints);
  
  nebula::cpp2::ErrorCode init(InitContext& ctx) override;
  std::unique_ptr<IndexNode> copy() override;
  std::string identify() override;
  Result doNext() override;

 private:
  bool isVertex_;
  std::vector<PartitionID> partitions_;
  std::vector<storage::cpp2::IndexQueryHint> hints_;
  std::unique_ptr<StorageRowReader> reader_;
};

class IndexLimitNode : public IndexNode {
 public:
  IndexLimitNode(RuntimeContext* context, uint64_t offset, uint64_t limit);
  IndexLimitNode(RuntimeContext* context, uint64_t limit);
  
  nebula::cpp2::ErrorCode init(InitContext& ctx) override;
  Result doNext() override;

 private:
  uint64_t offset_;
  uint64_t limit_;
  uint64_t count_{0};
};

class IndexProjectionNode : public IndexNode {
 public:
  nebula::cpp2::ErrorCode init(InitContext& ctx) override;
  Result doNext() override;

 private:
  std::vector<size_t> requiredColumns_;
  std::vector<Expression*> expressions_;
};
```

### 1.4 QueryBaseProcessor 职责分工

在 `BaseProcessor` 基础上，`QueryBaseProcessor` 处理查询特有的逻辑：

```cpp
template <typename REQ, typename RESP>
class QueryBaseProcessor : public BaseProcessor<RESP> {
 protected:
  // 属性上下文：解析请求中的属性
  TagContext tagContext_;       // Tag 属性信息
  EdgeContext edgeContext_;     // Edge 属性信息

  // 过滤器
  Expression* filter_{nullptr};     // 主过滤器
  Expression* tagFilter_{nullptr};  // Tag 过滤器

  // 表达式求值上下文
  ExpressionContext expCtx_;

  // 结果数据集
  DataSetWrapper resultDataSet_;

 public:
  nebula::cpp2::ErrorCode buildFilter(const REQ& req);
  nebula::cpp2::ErrorCode handleVertexProps(std::vector<cpp2::VertexProp>& vertexProps);
  nebula::cpp2::ErrorCode handleEdgeProps(std::vector<cpp2::EdgeProp>& edgeProps);
  void buildTagTTLInfo();
  void buildEdgeTTLInfo();
};
```

## 二、GraphDB 当前架构问题

### 2.1 架构对比矩阵

| 维度 | NebulaGraph | GraphDB 当前实现 | 问题 |
|-----|------------|-----------------|------|
| **处理器基类** | `BaseProcessor<RESP>` 模板类 | 无统一基类 | 重复代码、职责不清 |
| **执行计划** | IndexNode 树形结构 | storage/plan 未使用 | 死代码、功能重复 |
| **查询处理** | `QueryBaseProcessor` 分层 | 直接在 Executor 中处理 | 逻辑耦合 |
| **错误处理** | 统一的 ErrorCode 枚举 | 各处定义错误类型 | 不一致 |
| **分区支持** | PartitionID 支持 | 无分区概念 | 限制了分布式扩展 |
| **异步模型** | folly::Future | async-trait 简单封装 | 异步支持不完善 |
| **内存管理** | MemoryCheckOffGuard | 无系统化内存管理 | 潜在内存问题 |
| **统计监控** | ProcessorCounters | 无系统化监控 | 性能调优困难 |

### 2.2 具体代码问题

**问题 1：两套并行的执行器系统**

```
src/storage/plan/           vs   src/query/executor/
├── Plan trait                    ├── Executor trait
├── Executor trait                ├── BaseExecutor
├── DataSet trait                └── 各种具体 Executor
└── 实现返回错误的占位符
```

**问题 2：storage/plan 中的占位符实现**

```rust
// storage/plan/executors.rs
impl Plan for ScanExecutor {
    fn execute(&self, _ctx: &ExecutionContext) -> Result<Box<dyn DataSet>, ExecutionError> {
        Err(ExecutionError {
            message: "ScanExecutor should be executed via plan executor".to_string(),
            cause: None,
        })
        // 这个实现永远不会被调用，是死代码
    }
}
```

**问题 3：重复的执行器定义**

```rust
// storage/plan/nodes.rs
pub struct LimitExecutor { offset: usize, count: usize }

// query/executor/result_processing/limit.rs
pub struct LimitExecutor<S: StorageClient> { ... }
// 功能相同但实现独立的两个类型
```

**问题 4：缺乏统一的上下文传递**

```rust
// 当前：每个执行器自己管理上下文
impl<S: StorageClient> Executor<S> for GetVerticesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())...  // 重复的锁定逻辑
    }
}
```

### 2.3 storage/plan 目录使用分析

**文件列表及状态：**

| 文件 | 类型 | 使用状态 | 问题 |
|-----|------|---------|------|
| `mod.rs` | 定义 | 被引用 | 定义了未使用的 Plan/DataSet trait |
| `executors.rs` | 实现 | 未被调用 | 所有 execute 方法返回错误，是死代码 |
| `nodes.rs` | 定义 | 未被引用 | 定义了未使用的节点结构 |

**使用分析：**

```rust
// mod.rs 中定义的 trait 从未被实际调用
pub trait Plan: Send + Sync {
    fn execute(&self, ctx: &ExecutionContext) -> Result<Box<dyn DataSet>, ExecutionError>;
    fn schema(&self) -> &ResultSetSchema;
}

pub trait Executor: Send {
    fn execute(
        &self,
        storage: &Arc<dyn StorageReader>,
        input: Option<&dyn DataSet>,
    ) -> Result<Box<dyn DataSet>, ExecutionError>;
}
```

## 三、改造方案

### 3.1 总体改造架构

```
src/storage/
├── mod.rs
├── processor/                          # 新增：存储处理器基类
│   ├── mod.rs                         # 导出 Processor
│   ├── base.rs                        # 定义 ProcessorBase<RESP>
│   └── context.rs                     # 定义 ProcessorContext
├── iterator/                          # 现有
│   └── ...
└── ...

src/query/
├── executor/
│   ├── base/                          # 增强
│   │   ├── mod.rs
│   │   ├── storage_processor.rs       # 新增：桥接 storage::Processor
│   │   └── context.rs                 # 增强：支持分区上下文
│   ├── data_access.rs                 # 重构：使用 Processor
│   ├── result_processing/             # 保持现有结构
│   └── ...
└── ...
```

### 3.2 定义存储处理器基类

**src/storage/processor/base.rs：**

```rust
use crate::core::error::{DBError, DBResult};
use crate::storage::{StorageClient, StorageEnv};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct ProcessorCounters {
    pub num_calls: i64,
    pub num_errors: i64,
    pub latency_us: i64,
}

pub struct ProcessorContext {
    pub env: Arc<Mutex<dyn StorageEnv>>,
    pub space_id: u32,
    pub part_id: u32,
    pub counters: Option<ProcessorCounters>,
}

pub trait StorageProcessor<RESP> {
    fn context(&self) -> &ProcessorContext;
    fn context_mut(&mut self) -> &mut ProcessorContext;
    
    fn execute(&mut self) -> DBResult<RESP>;
    
    fn on_finished(&mut self) -> DBResult<RESP>;
    fn on_error(&mut self) -> DBResult<RESP>;
}

pub struct ProcessorBase<RESP, S: StorageClient> {
    context: ProcessorContext,
    resp: Option<RESP>,
    duration: Duration,
    codes: Vec<PartCode>,
    storage: Arc<Mutex<S>>,
}

impl<RESP, S: StorageClient> ProcessorBase<RESP, S> {
    pub fn new(context: ProcessorContext, storage: Arc<Mutex<S>>) -> Self {
        Self {
            context,
            resp: None,
            duration: Duration::ZERO,
            codes: Vec::new(),
            storage,
        }
    }

    pub fn push_code(&mut self, code: DBError, part_id: u32) {
        self.codes.push(PartCode { code, part_id });
    }

    pub fn is_memory_exceeded(&self) -> bool {
        // 使用系统内存监控
        if let Some(limit) = self.context.memory_limit {
            return self.current_memory_usage > limit;
        }
        
        // 检查系统可用内存
        if let Ok(mem_info) = sys_info::mem_info() {
            return mem_info.avail < MIN_AVAILABLE_MEMORY;
        }
        
        false
    }

    pub fn memory_usage(&self) -> u64 {
        self.current_memory_usage
    }

    pub fn set_memory_limit(&mut self, limit: u64) {
        self.context.memory_limit = Some(limit);
    }
}

    pub fn storage(&self) -> &Arc<Mutex<S>> {
        &self.storage
    }

    pub fn env(&self) -> &ProcessorContext {
        &self.context
    }
}
```

### 3.3 渐进式迁移策略

**阶段 1：创建基础架构（安全）**

```rust
// src/storage/processor/mod.rs
pub mod base;
pub mod context;

pub use base::{ProcessorBase, ProcessorContext, StorageProcessor};
pub use context::{TagContext, EdgeContext, PropContext};
```

**阶段 2：迁移数据访问执行器（高风险）**

选择 `GetVerticesExecutor` 作为试点：

```rust
// src/query/executor/data_access.rs
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for GetVerticesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 引入 Processor 基类的通用逻辑
        let storage = safe_lock(self.get_storage())?;
        
        // 内存检查
        if self.base.is_memory_exceeded() {
            return Err(DBError::Storage(
                crate::core::error::StorageError::MemoryExceeded
            ));
        }
        
        // 使用 ProcessorContext 获取分区信息
        let part_id = self.base.context().part_id;
        
        // 执行查询
        let vertices = storage.get_vertices("default", part_id, &self.vertex_ids)?;
        
        self.base.on_finished()
    }
}
```

**阶段 3：废弃 storage/plan（清理）**

迁移完成后删除死代码：

```bash
# 验证无使用后删除
rm -rf src/storage/plan/
```

## 四、NebulaGraph 架构优缺点分析

### 4.1 优点

**1. 清晰的职责分层**

```cpp
// 分层清晰，每层只做一件事
BaseProcessor<RESP>          // 通用处理：统计、内存管理、异步完成
    ↓ 继承
QueryBaseProcessor<REQ,RESP> // 查询特有：属性处理、过滤、TTL
    ↓ 使用
IndexNode Tree               // 执行计划：Scan → Limit → Project
```

**2. 强大的错误处理机制**

```cpp
// 统一的错误码
enum class ErrorCode {
    SUCCEEDED = 0,
    E_LEADER_CHANGED,
    E_SPACE_NOT_FOUND,
    E_TAG_NOT_FOUND,
    E_TAG_PROP_NOT_FOUND,
    // ... 70+ 种错误
};

// 错误传播
void handleErrorCode(nebula::cpp2::ErrorCode code,
                     GraphSpaceID spaceId,
                     PartitionID partId) {
    if (code != nebula::cpp2::ErrorCode::SUCCEEDED) {
        if (code == nebula::cpp2::ErrorCode::E_LEADER_CHANGED) {
            handleLeaderChanged(spaceId, partId);  // 自动处理 leader 切换
        } else {
            pushResultCode(code, partId);  // 收集错误
        }
    }
}
```

**3. 完善的分区支持**

```cpp
// 自动处理多分区查询
class IndexScanNode {
    std::vector<PartitionID> partitions_;  // 跨分区
    
    Result doNext() override {
        for (auto& part : partitions_) {
            auto ret = storage_->scan(spaceId_, part, ...);
            while (ret.hasNext()) {
                yield return ret.next();
            }
        }
    }
};
```

**4. 性能监控集成**

```cpp
// 性能计数器
ProcessorCounters counters_ {
    .numCalls_ = COUNTER_STORAGE_GET_VERTICES,
    .numErrors_ = COUNTER_STORAGE_GET_VERTICES_ERRORS,
    .latency_ = COUNTER_STORAGE_GET_VERTICES_LATENCY,
};

// 自动记录
stats::StatsManager::addValue(counters_->numCalls_);
stats::StatsManager::addValue(counters_->latency_, duration_.elapsedInUSec());
```

### 4.2 缺点

**1. 代码复杂度高**

```cpp
// 模板 + 继承 + 组合模式的组合
template <typename REQ, typename RESP>
class QueryBaseProcessor<REQ, RESP> 
    : public BaseProcessor<RESP> 
{
    // 1800+ 行头文件
    // 理解成本高
};
```

**2. 内存管理复杂性**

```cpp
// 自销毁模式容易出错
virtual void onFinished() {
    // ...
    delete this;  // 谁调用谁负责，否则内存泄漏
}
```

**3. 分布式假设过强**

```cpp
// 针对分布式场景设计，单节点部署时有很多冗余
void handleLeaderChanged(GraphSpaceID spaceId, PartitionID partId) {
    auto addrRet = env_->kvstore_->partLeader(spaceId, partId);
    // 单节点场景下永远获取不到 leader
}
```

**4. 编译时间**

```cpp
// 模板实例化导致编译时间长
using GetVerticesProcessor = QueryBaseProcessor<cpp2::GetVerticesRequest,
                                                 cpp2::GetVerticesResponse>;
```

### 4.3 对 GraphDB 的适用性建议

| NebulaGraph 特性 | 是否采用 | 理由 | 实现建议 |
|-----------------|---------|------|---------|
| BaseProcessor<RESP> | ✅ 采用 | 统一处理流程 | 简化版实现 |
| IndexNode 树 | ⚠️ 评估 | 适合复杂查询 | 可选实现 |
| 分区支持 | ❌ 跳过 | 单节点场景不需要 | 预留给未来 |
| 自销毁模式 | ❌ 跳过 | Rust 不需要 | 使用 Drop |
| 错误码枚举 | ✅ 采用 | 统一错误处理 | 扩展当前枚举 |
| 性能监控 | ✅ 采用 | 渐进式引入 | 先实现基础统计 |
| folly::Future | ✅ 采用 | 高性能异步 | 使用 tokio |

## 五、改造优先级

| 优先级 | 任务 | 工作量 | 风险 | 影响 |
|-------|------|-------|------|-----|
| P0 | 创建 storage/processor 基础模块 | 3-5 天 | 低 | 架构统一 |
| P0 | 统一错误码定义 | 1-2 天 | 低 | 错误处理 |
| P1 | 迁移 GetVerticesExecutor | 2-3 天 | 中 | 数据访问 |
| P1 | 迁移 GetNeighborsExecutor | 3-5 天 | 高 | 图遍历 |
| P2 | 实现执行计划节点 | 5-10 天 | 高 | 复杂查询 |
| P2 | 删除 storage/plan 死代码 | 1 天 | 低 | 清理代码 |
| P3 | 性能监控集成 | 3-5 天 | 中 | 可观测性 |

**建议的改造路径：**

```
1. 先创建 processor 基类（2 周）
   ↓
2. 迁移核心执行器（2-3 周）
   ↓
3. 验证功能正确性（1 周）
   ↓
4. 删除旧代码，清理架构（1 周）
```

**预计总工期：6-8 周**

## 六、立即执行的操作

### 6.1 删除 storage/plan 死代码

由于 `storage/plan` 目录中的代码从未被实际使用，建议立即删除：

```bash
# 检查引用
grep -r "storage::plan" src/ --include="*.rs"

# 如果无引用，删除目录
rm -rf src/storage/plan/
```

### 6.2 创建存储处理器模块

创建基础架构：

1. 创建 `src/storage/processor/mod.rs`
2. 创建 `src/storage/processor/base.rs`
3. 创建 `src/storage/processor/context.rs`
4. 更新 `src/storage/mod.rs` 导出新模块

### 6.3 统一错误码

检查并统一 `src/core/error` 中的错误定义，确保与 NebulaGraph 的错误码体系兼容。

### 6.4 代码迁移示例

#### 6.4.1 GetVerticesExecutor 迁移

**迁移前：**

```rust
// src/query/executor/data_access/get_vertices_executor.rs
impl<S: StorageClient> Executor<S> for GetVerticesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = safe_lock(self.get_storage())?;
        
        // 重复的内存检查逻辑
        if self.current_memory_usage > MAX_MEMORY {
            return Err(DBError::Storage(StorageError::MemoryExceeded));
        }
        
        // 重复的统计逻辑
        let start = Instant::now();
        let result = storage.get_vertices(...)?;
        let latency = start.elapsed();
        
        self.stats.record_latency(latency);
        
        Ok(ExecutionResult::new(result))
    }
}
```

**迁移后：**

```rust
// src/query/executor/data_access/get_vertices_executor.rs
impl<S: StorageClient + Send + Sync + 'static> Executor<S> for GetVerticesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 使用 Processor 基类的通用逻辑
        self.base.start_timer();
        
        // 内存检查由基类处理
        if self.base.is_memory_exceeded() {
            return Err(DBError::Storage(StorageError::MemoryExceeded));
        }
        
        // 执行查询
        let result = self.storage.get_vertices(...)?;
        
        // 设置响应
        self.base.set_response(ExecutionResult::new(result));
        
        // 统一的完成处理（包含统计、错误收集）
        self.base.on_finished()
    }
}
```

#### 6.4.2 迁移检查清单

每个执行器迁移时需检查：

| 检查项 | 状态 | 说明 |
|-------|------|------|
| 内存检查 | ☐ | 使用 `base.is_memory_exceeded()` |
| 计时统计 | ☐ | 使用 `base.start_timer()` / `base.stop_timer()` |
| 错误收集 | ☐ | 使用 `base.push_code()` |
| 响应设置 | ☐ | 使用 `base.set_response()` |
| 完成回调 | ☐ | 使用 `base.on_finished()` |

### 6.5 内存管理实现

#### 6.5.1 内存监控策略

```rust
pub struct MemoryConfig {
    /// 单个处理器内存限制（字节）
    pub processor_limit: u64,
    /// 系统最小可用内存（字节）
    pub min_available_memory: u64,
    /// 内存检查间隔（毫秒）
    pub check_interval: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            processor_limit: 512 * 1024 * 1024, // 512MB
            min_available_memory: 100 * 1024 * 1024, // 100MB
            check_interval: 1000, // 1秒
        }
    }
}
```

#### 6.5.2 内存使用跟踪

```rust
impl<RESP, S: StorageClient> ProcessorBase<RESP, S> {
    /// 记录内存分配
    pub fn track_allocation(&mut self, size: u64) {
        self.current_memory_usage += size;
        self.allocation_count += 1;
    }

    /// 记录内存释放
    pub fn track_deallocation(&mut self, size: u64) {
        self.current_memory_usage = self.current_memory_usage.saturating_sub(size);
    }

    /// 批量操作开始
    pub fn begin_batch(&mut self) {
        self.batch_memory_start = self.current_memory_usage;
    }

    /// 批量操作结束
    pub fn end_batch(&mut self) -> u64 {
        let end = self.current_memory_usage;
        end - self.batch_memory_start
    }
}
```

### 6.6 性能监控集成

#### 6.6.1 计数器配置

```rust
pub struct ProcessorCounters {
    /// 调用次数计数器 ID
    pub num_calls: CounterId,
    /// 错误次数计数器 ID
    pub num_errors: CounterId,
    /// 延迟计数器 ID
    pub latency_us: CounterId,
    /// 内存使用计数器 ID
    pub memory_bytes: CounterId,
}

impl Default for ProcessorCounters {
    fn default() -> Self {
        Self {
            num_calls: CounterId::new("storage_processor_calls"),
            num_errors: CounterId::new("storage_processor_errors"),
            latency_us: CounterId::new("storage_processor_latency_us"),
            memory_bytes: CounterId::new("storage_processor_memory_bytes"),
        }
    }
}
```

#### 6.6.2 自动统计收集

```rust
impl<RESP, S: StorageClient> ProcessorBase<RESP, S> {
    fn record_stats(&self) {
        if let Some(counters) = &self.context.counters {
            // 记录调用次数
            StatsManager::add_value(counters.num_calls, 1);
            
            // 记录错误次数
            if self.has_errors() {
                StatsManager::add_value(counters.num_errors, self.codes.len() as i64);
            }
            
            // 记录延迟
            StatsManager::add_value(counters.latency_us, self.duration.as_micros() as i64);
            
            // 记录内存使用
            StatsManager::add_value(counters.memory_bytes, self.memory_usage() as i64);
        }
    }
}
```

## 七、测试策略

### 7.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_limit_check() {
        let context = ProcessorContext::new(1, 100);
        context.memory_limit = Some(1000);
        
        let mut base = ProcessorBase::new(context, storage);
        base.current_memory_usage = 500;
        
        assert!(!base.is_memory_exceeded());
        
        base.current_memory_usage = 1500;
        assert!(base.is_memory_exceeded());
    }

    #[test]
    fn test_error_collection() {
        let context = ProcessorContext::new(1, 100);
        let base = ProcessorBase::new(context, storage);
        
        base.push_code(DBError::Storage(StorageError::NotFound), 100);
        base.push_code(DBError::Storage(StorageError::NotFound), 200);
        
        assert!(base.has_errors());
        assert_eq!(base.failed_parts().len(), 2);
    }
}
```

### 7.2 集成测试

```rust
#[tokio::test]
async fn test_get_vertices_with_processor() {
    // 准备测试数据
    let storage = create_test_storage();
    let context = ProcessorContext::new(space_id, part_id);
    
    // 创建处理器
    let mut processor = GetVerticesProcessor::new(context, storage);
    
    // 执行查询
    let result = processor.execute().await;
    
    // 验证结果
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.results.is_empty());
    
    // 验证统计信息
    assert_eq!(processor.duration().as_millis(), 0); // 快速执行
}
```

### 7.3 性能基准测试

```rust
fn benchmark_processor(benchmark: &mut Bencher) {
    let storage = create_test_storage();
    let context = ProcessorContext::new(1, 100);
    
    benchmark.iter(|| {
        let processor = GetVerticesProcessor::new(context.clone(), storage.clone());
        processor.execute()
    });
}
```

## 八、风险分析与回滚计划

### 8.1 风险识别

| 风险 | 概率 | 影响 | 缓解措施 |
|-----|------|------|---------|
| 现有功能回归 | 中 | 高 | 完整的测试覆盖，渐进式迁移 |
| 性能下降 | 低 | 中 | 性能基准测试，性能监控 |
| 内存泄漏 | 低 | 高 | 内存跟踪，集成测试 |
| 编译时间增加 | 中 | 低 | 合理拆分模块，避免过度泛化 |
| 开发者学习成本 | 中 | 低 | 文档完善，培训分享 |

### 8.2 回滚计划

**阶段 1：代码级回滚**

```bash
# 使用 Git 回滚
git checkout HEAD~1 -- src/storage/processor/
```

**阶段 2：功能开关**

```rust
// 在配置中启用/禁用新处理器
pub struct Config {
    pub use_new_processor: bool,
}

// 执行器根据配置选择实现
impl<S: StorageClient> Executor<S> for GetVerticesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        if self.config.use_new_processor {
            self.execute_with_processor().await
        } else {
            self.execute_legacy().await
        }
    }
}
```

**阶段 3：功能标志**

```rust
// 使用 feature flag 控制
#[cfg(feature = "new-processor")]
use processor_new::GetVerticesProcessor;

#[cfg(not(feature = "new-processor"))]
use processor_legacy::GetVerticesExecutor;
```

### 8.3 迁移验证清单

在每个阶段迁移完成后，验证以下项目：

- [ ] 所有单元测试通过
- [ ] 集成测试通过
- [ ] 性能基准测试无显著下降（< 10%）
- [ ] 内存使用在预期范围内
- [ ] 错误处理逻辑正确
- [ ] 日志输出正常
- [ ] 监控指标正常

## 九、总结

NebulaGraph 的架构设计在分布式场景下非常优秀，但对于 GraphDB 的单节点定位，建议采用"取其精华、简化实现"的策略：

1. **采用**：Processor 基类、统一错误处理、性能监控框架
2. **简化**：移除分区逻辑、简化异步模型、使用 Rust 所有权替代手动内存管理
3. **跳过**：IndexNode 树（当前场景下收益不明显）

通过渐进式改造，可以在保持系统稳定性的同时逐步提升架构质量。`storage/plan` 目录中的死代码应该立即删除，以避免代码腐烂和理解混淆。

### 关键里程碑

| 里程碑 | 目标 | 验收标准 |
|-------|------|---------|
| M1: 基础架构 | 创建 processor 模块 | 编译通过，单元测试通过 |
| M2: 核心迁移 | 迁移 GetVerticesExecutor | 功能测试通过，性能无下降 |
| M3: 全面迁移 | 迁移所有执行器 | 全部测试通过 |
| M4: 清理完成 | 删除旧代码 | 代码库无冗余代码 |

**预计总工期：6-8 周**
