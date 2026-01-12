# NebulaGraph 执行架构参考与 GraphDB 对标分析

**日期**: 2025-12-10  
**版本**: v1.0  
**参考源**: nebula-3.8.0  
**用途**: 作为 GraphDB 架构改造的参考模型

---

## 文档概述

本文档分析 NebulaGraph 的查询执行流程，为 GraphDB 的架构改造提供参考。GraphDB 应该采用类似的 Parser → Validator → Optimizer → Scheduler → Executor 的五层架构。

---

## NebulaGraph 执行流程总览

### 完整的查询执行链路

```
SQL 字符串
    ↓
[1] Parser (GQLParser)
    ├─ 解析 SQL 成 AST
    └─ 输出: Sentence
    ↓
[2] Validator (Validator::validate)
    ├─ 语义检查
    ├─ 方案生成 (Plan Generation)
    └─ 输出: ExecutionPlan
    ↓
[3] Optimizer (Optimizer::findBestPlan)
    ├─ 应用优化规则
    ├─ 规则集: DefaultRules, QueryRules0, QueryRules
    └─ 输出: 优化后的 ExecutionPlan
    ↓
[4] Scheduler (AsyncMsgNotifyBasedScheduler)
    ├─ Executor 树构建
    ├─ 依赖关系分析
    └─ 异步消息驱动调度
    ↓
[5] Executor (100+ 种具体 Executor)
    ├─ open() - 资源初始化
    ├─ execute() - 执行逻辑
    ├─ close() - 资源清理
    └─ 输出: 查询结果
    ↓
结果序列化与返回给客户端
```

---

## 分层架构详解

### 第一层：Parser（解析）

**文件位置**: `nebula-3.8.0/src/parser/` (外部库)

**责任**: 将 SQL 字符串转换为抽象语法树 (AST)

**关键类型**:
- `Sentence` - AST 根节点，可以是 `GoSentence`、`MatchSentence` 等

**NebulaGraph 使用**:
```cpp
// QueryInstance.cpp line 82
auto result = GQLParser(qctx()).parse(rctx->query());
sentence_ = std::move(result).value();
```

**GraphDB 当前状态** ⚠️：
- ✅ 有 `Parser` 模块 (`src/query/parser/`)
- ✅ 可以解析成 `Statement` AST
- ✅ 状态: **良好**

---

### 第二层：Validator（验证与初始规划）

**文件位置**: `nebula-3.8.0/src/graph/validator/`

**责任**:
1. 检查 SQL 的语义合法性（表名、列名等）
2. 根据 Sentence 类型生成初始执行计划
3. 通过工厂方法模式创建对应的 Validator 子类

**关键类和方法**:

```cpp
// Validator.h
class Validator {
public:
    static StatusOr<const PlanNode*> validate(QueryContext* qctx, Sentence* sentence);
    
protected:
    virtual Status validateImpl() = 0;
    const PlanNode* toPlan();  // 通过 Planner 转换为 PlanNode 树
};

// Validator 子类（按 Sentence 类型）
class GoValidator : public Validator;
class MatchValidator : public Validator;
class FetchVerticesValidator : public Validator;
class InsertVerticesValidator : public Validator;
class UpdateVertexValidator : public Validator;
class DeleteVertexValidator : public Validator;
// ... 20+ 种 Validator
```

**执行流程**:

1. 根据 Sentence 的 Kind 选择对应 Validator
2. 调用 `validateImpl()` 进行语义检查
3. 调用 `toPlan()` 通过 Planner 生成 PlanNode 树
4. 返回 ExecutionPlan

**GraphDB 当前状态** ⚠️：
- ❌ 没有 Validator 层
- ❌ 没有初始规划
- ❌ 直接跳到 Executor 执行
- ❌ 状态: **缺失**

**需要改进**: 建立 Validator 层，按 Statement 类型创建不同的 Validator，生成初始执行计划

---

### 第三层：Optimizer（优化）

**文件位置**: `nebula-3.8.0/src/graph/optimizer/`

**责任**:
1. 对执行计划应用优化规则
2. 选择成本最低的执行方案
3. 支持多种规则集

**关键方法**:

```cpp
// Optimizer.h
class Optimizer {
public:
    StatusOr<const PlanNode*> findBestPlan(QueryContext* qctx);
    
private:
    // 优化规则集
    RuleSet rules_;
};
```

**优化规则类型**:
- **DefaultRules**: 通用规则（投影下推、列裁剪等）
- **QueryRules0**: 查询特定规则（第一阶段）
- **QueryRules**: 查询特定规则（第二阶段）

**优化示例**:
- 列剪枝 (Column Pruning)
- 投影下推 (Projection Pushdown)
- 谓词下推 (Predicate Pushdown)
- 索引使用优化
- Join 顺序优化

**GraphDB 当前状态** ⚠️：
- ❌ 没有 Optimizer 层
- ❌ 没有规则引擎
- ❌ 状态: **缺失**

**需要改进**: 未来可以逐步添加

---

### 第四层：Scheduler（调度器）

**文件位置**: `nebula-3.8.0/src/graph/scheduler/`

**责任**:
1. 将 PlanNode 树转换为 Executor 树
2. 分析执行器之间的依赖关系
3. 异步驱动执行器的执行
4. 处理依赖等待和结果传递

**关键方法**:

```cpp
// Scheduler.h
class Scheduler {
public:
    virtual folly::Future<Status> schedule() = 0;
};

// AsyncMsgNotifyBasedScheduler.h
class AsyncMsgNotifyBasedScheduler : public Scheduler {
public:
    folly::Future<Status> schedule();
    
private:
    // BFS 遍历构建 Executor 树
    folly::Future<Status> doSchedule(Executor* root);
    
    // 异步执行单个 Executor
    folly::Future<Status> scheduleExecutor(
        std::vector<folly::Future<Status>>&& futures,
        Executor* exe,
        folly::Executor* runner);
};
```

**调度流程**:

1. `schedule()` 遍历 PlanNode 树，为每个 PlanNode 创建 Executor
2. 通过 BFS 建立 Executor 之间的依赖关系
3. 对于每个 Executor：
   - 使用 `folly::collect` 等待所有依赖完成
   - 调用 `execute()`
   - 完成后通过 Promise 唤醒后续 Executor
4. 支持 Future 链，实现完全异步执行

**异步执行示例**:
```cpp
// 等待依赖 + 执行 + 链式调用
return folly::collect(futures)
    .thenValue([executor](auto&&) { return executor->execute(); })
    .thenValue([executor](auto&& s) { return executor->close(); });
```

**GraphDB 当前状态** ✅：
- ✅ 有 Scheduler 实现 (`src/query/scheduler/`)
- ✅ 有 AsyncScheduler、AsyncMsgNotifyBasedScheduler
- ✅ 有执行计划模型 (ExecutionPlan)
- ✅ 状态: **基本完成**

**需要改进**:
- 完善与 Executor 框架的集成
- 优化异步调度的效率

---

### 第五层：Executor（执行器）

**文件位置**: `nebula-3.8.0/src/graph/executor/`

**责任**: 实现具体的数据操作（读取、过滤、联接、聚合、修改等）

**Executor 基类**:

```cpp
// Executor.h
class Executor : private boost::noncopyable {
public:
    // 工厂方法：根据 PlanNode 类型创建对应 Executor
    static Executor* create(const PlanNode* node, QueryContext* qctx);
    
    // 执行流程
    virtual Status open();                           // 资源初始化
    virtual folly::Future<Status> execute() = 0;    // 执行逻辑
    virtual Status close();                         // 资源清理
    
    // 依赖管理
    Executor* dependsOn(Executor* dep);
    const std::set<Executor*>& depends() const;
    const std::set<Executor*>& successors() const;
};
```

**Executor 子类** (100+ 种)：

| 类别 | Executor | 作用 |
|------|----------|------|
| 查询 | FilterExecutor | 条件过滤 |
| | ProjectExecutor | 列投影 |
| | AggregateExecutor | 聚合函数 |
| | JoinExecutor | 数据联接 |
| | SortExecutor | 排序 |
| | GroupByExecutor | 分组 |
| 遍历 | TraverseExecutor | 图遍历 |
| | GetNeighborsExecutor | 获取邻居 |
| | ExpandExecutor | 路径扩展 |
| 修改 | InsertVertexExecutor | 插入顶点 |
| | InsertEdgeExecutor | 插入边 |
| | UpdateVertexExecutor | 更新顶点 |
| | DeleteVertexExecutor | 删除顶点 |
| 管理 | CreateSpaceExecutor | 创建空间 |
| | ShowSpacesExecutor | 显示空间 |
| 逻辑 | SelectExecutor | SELECT 语句 |
| | LoopExecutor | 循环控制 |

**数据流**:

每个 Executor 的 `execute()` 方法：
1. 从 **ExecutionContext** 读取输入数据
2. 处理数据
3. 将结果存回 **ExecutionContext**
4. 返回 Status

```cpp
// 示例: ProjectExecutor
folly::Future<Status> ProjectExecutor::execute() {
    // 1. 从 ExecutionContext 读取输入
    auto* inputData = ectx_->getResult(inputVar_);
    
    // 2. 处理数据（投影列）
    auto outputs = projectColumns(inputData);
    
    // 3. 将结果存回 ExecutionContext
    ectx_->setResult(outputVar_, outputs);
    
    return Status::OK();
}
```

**GraphDB 当前状态** ⚠️：
- ✅ 有基础 Executor 框架 (`src/query/executor/base.rs`)
- ✅ 有部分 Executor 实现 (`data_processing.rs`, `result_processing.rs`)
- ❌ 数据修改 Executor 未被使用 (`data_access.rs`, `data_modification.rs`)
- ❌ 没有与 Scheduler 的完整集成
- ❌ Executor 创建没有工厂方法
- ❌ 状态: **部分完成，集成不足**

**需要改进**:
1. 删除未使用的 Executor
2. 完成 Executor 的工厂方法集成
3. 确保所有 Executor 正确读写 ExecutionContext
4. 与 Scheduler 完全集成

---

## 关键数据结构对标

### 上下文数据结构

| NebulaGraph | GraphDB | 说明 |
|-------------|---------|------|
| `QueryContext` | `QueryContext` | 整个查询的全局上下文 |
| `ExecutionContext` | `ExecutionContext` | 执行过程中的数据存储 |
| `RequestContext` | - | 请求级别的上下文 |
| `PlanNode` | - | 执行计划中的节点 |
| `Sentence` | `Statement` | SQL 解析后的 AST |

### 执行计划数据结构

| NebulaGraph | GraphDB | 说明 |
|-------------|---------|------|
| `ExecutionPlan` | `ExecutionPlan` | 完整的执行计划 |
| `PlanNode` 树 | PlanNode 概念缺失 | 执行计划节点树 |
| 各种 Plan 节点 | - | Go, Match, Fetch 等 |

---

## 执行流程对标

### NebulaGraph 的执行流程

**文件**: `nebula-3.8.0/src/graph/service/QueryInstance.cpp`

```cpp
folly::Future<Status> QueryInstance::execute() {
    // 1. 验证和优化
    validateAndOptimize()
        ├─ GQLParser().parse(sql)           // 解析
        ├─ Validator::validate()            // 验证
        └─ Optimizer::findBestPlan()        // 优化
    
    // 2. 解释或继续
    explainOrContinue()
        └─ 如果是 EXPLAIN，直接返回计划
    
    // 3. 调度执行
    scheduler_->schedule()
        ├─ Executor::create()               // 创建 Executor 树
        ├─ AsyncMsgNotifyBasedScheduler 调度  // 异步驱动
        └─ 执行结果
    
    // 4. 收尾
    onFinish() / onError()
        └─ 填充响应、统计信息
}
```

### GraphDB 当前的执行流程

**文件**: `src/query/mod.rs` 中的 `QueryExecutor`

```rust
async fn execute(&self, stmt: Statement) -> Result<QueryResult> {
    // 1. 直接根据 Statement 类型执行
    match stmt {
        Statement::Create(c) => self.handle_create(c),
        Statement::Insert(i) => self.handle_insert(i),
        Statement::Query(q) => self.handle_query(q),
        // ... match 所有类型
    }
    
    // ❌ 缺失: Validator
    // ❌ 缺失: Optimizer
    // ❌ 缺失: Scheduler 的完整使用
    // ❌ 直接操作存储，没有经过 Executor 框架
}
```

**问题**:
1. 跳过了 Validator 和 Optimizer 层
2. 没有使用 Executor 框架
3. 执行逻辑分散在 `match` 语句中
4. 难以优化、难以扩展

---

## GraphDB 改造方案：采用 NebulaGraph 五层架构

### 目标架构

```
SQL 字符串
    ↓
[1] Parser
    └─ 现有: src/query/parser/
    ✅ 状态: 可用
    ↓
[2] Validator (新增)
    ├─ 创建: src/query/validator/
    ├─ 包含各类型 Validator
    └─ 生成初始执行计划
    ↓
[3] Optimizer (新增/增强)
    ├─ 创建: src/query/optimizer/
    ├─ 应用优化规则
    └─ 优化执行计划
    ↓
[4] Scheduler (改进现有)
    ├─ 现有: src/query/scheduler/
    ✅ 基本完成
    └─ 改进与 Executor 的集成
    ↓
[5] Executor (完成改造)
    ├─ 现有: src/query/executor/
    ├─ 删除: data_access.rs, data_modification.rs
    ├─ 创建工厂方法
    └─ 完整集成
    ↓
结果返回
```

### 改造步骤

#### 第一步：清理 Executor 层（1 天）

**立即执行**（参考前面的分析）：
- 删除 `data_access.rs` 和 `data_modification.rs`
- 更新 `mod.rs` 导出
- 验证编译和测试

#### 第二步：建立 Validator 层（3 天）

**创建** `src/query/validator/` 目录：

```rust
// src/query/validator/mod.rs
pub trait Validator {
    fn validate(&self, ctx: &mut QueryContext) -> Result<PlanNode>;
}

pub struct CreateValidator;
pub struct InsertValidator;
pub struct QueryValidator;
// ... 其他 Validator

pub fn create_validator(stmt: &Statement) -> Result<Box<dyn Validator>> {
    match stmt {
        Statement::Create(_) => Ok(Box::new(CreateValidator)),
        Statement::Insert(_) => Ok(Box::new(InsertValidator)),
        Statement::Query(_) => Ok(Box::new(QueryValidator)),
        // ...
    }
}
```

**Validator 的职责**:
1. 语义检查（表名、列名等）
2. 通过 Planner 生成初始 PlanNode
3. 返回 ExecutionPlan

#### 第三步：优化 Scheduler 集成（2 天）

**改进** `src/query/scheduler/`：
- 确保 Scheduler 正确创建 Executor 树
- 完善依赖关系管理
- 验证异步执行的正确性

#### 第四步：完成 Executor 工厂（2 天）

**创建** `src/query/executor/factory.rs`：

```rust
pub struct ExecutorFactory;

impl ExecutorFactory {
    pub fn create_executor(
        plan_node: &PlanNode,
        ctx: Arc<QueryContext>,
    ) -> Result<Box<dyn Executor>> {
        match plan_node.kind {
            NodeKind::Create => Ok(Box::new(CreateExecutor::new(plan_node, ctx))),
            NodeKind::Insert => Ok(Box::new(InsertExecutor::new(plan_node, ctx))),
            // ... 其他类型
        }
    }
}
```

#### 第五步：统一查询执行入口（1 天）

**改造** `src/query/mod.rs`：

```rust
pub async fn execute(&self, stmt: Statement) -> Result<QueryResult> {
    // 1. 验证
    let validator = create_validator(&stmt)?;
    let plan = validator.validate(&mut ctx)?;
    
    // 2. 优化（可选，暂时跳过）
    
    // 3. 调度执行
    let result = self.scheduler.execute(&plan).await?;
    
    // 4. 返回结果
    Ok(result)
}
```

---

## 预期时间表

| 阶段 | 任务 | 工作量 | 时机 |
|------|------|--------|------|
| 0 | 删除冗余 Executor | 1 天 | 立即 |
| 1 | 建立 Validator 层 | 3 天 | 1 周内 |
| 2 | 完善 Scheduler 集成 | 2 天 | 2 周内 |
| 3 | Executor 工厂方法 | 2 天 | 2-3 周内 |
| 4 | 统一执行入口 | 1 天 | 3 周内 |
| 5 | 测试验证 | 2 天 | 3-4 周内 |
| **总计** | | **11 天** | **约 1 个月** |

---

## 参考 NebulaGraph 的最佳实践

### 1. 工厂模式

```rust
// 不好的做法
match stmt {
    Statement::Create(c) => { /* 100 行代码 */ },
    Statement::Insert(i) => { /* 100 行代码 */ },
    // ...
}

// 好的做法（NebulaGraph 采用）
let validator = ValidatorFactory::create(&stmt)?;
let plan = validator.validate(&ctx)?;
// 清晰、易扩展
```

### 2. 依赖注入

```rust
// NebulaGraph: 所有依赖通过 QueryContext 传入
pub fn execute(ctx: Arc<QueryContext>) -> Result<ExecutionResult> {
    // 获取存储、Schema 等
    let storage = ctx.storage();
    let schema = ctx.schema();
}

// 避免全局状态和 thread-local
```

### 3. Future 链

```rust
// NebulaGraph: 充分利用异步特性
executor.open()
    .then(|_| executor.execute())
    .then(|_| executor.close())
    .await?
```

### 4. 清晰的数据流

```rust
// 每个 Executor 是独立的管道
Input (ExecutionContext)
    ↓
Executor::execute()
    ↓
Output (ExecutionContext)
```

---

## 学习资源

### NebulaGraph 源代码位置

| 模块 | 文件 |
|------|------|
| 服务入口 | `src/graph/service/GraphService.cpp` |
| 查询实例 | `src/graph/service/QueryInstance.cpp` |
| 验证器 | `src/graph/validator/Validator.h` |
| 优化器 | `src/graph/optimizer/Optimizer.h` |
| 调度器 | `src/graph/scheduler/AsyncMsgNotifyBasedScheduler.h` |
| 执行器 | `src/graph/executor/Executor.h` |

### 关键概念

1. **PlanNode 树**: 执行计划的树形表示
2. **Executor 树**: 与 PlanNode 树对应的执行器树
3. **ExecutionContext**: 执行过程中的数据存储和流转
4. **Future 链**: 异步执行和依赖管理
5. **工厂模式**: 根据类型创建对应的对象

---

## 迁移清单

### 第 0 阶段：清理（立即）
- [ ] 删除 `data_access.rs` 和 `data_modification.rs`
- [ ] 更新 `executor/mod.rs`
- [ ] 验证编译和测试

### 第 1 阶段：Validator（1-2 周）
- [ ] 创建 `src/query/validator/` 目录
- [ ] 实现 ValidatorFactory
- [ ] 为各语句类型实现 Validator
- [ ] 集成到查询执行流程

### 第 2 阶段：Scheduler 优化（1-2 周）
- [ ] 审查 Scheduler 实现
- [ ] 完善 Executor 树构建
- [ ] 验证异步执行
- [ ] 性能优化

### 第 3-4 阶段：集成完成（2-3 周）
- [ ] 创建 ExecutorFactory
- [ ] 统一执行入口
- [ ] 完整测试覆盖
- [ ] 性能基准测试

---

## 总结

| 方面 | NebulaGraph | GraphDB 现状 | 改造方向 |
|------|-----------|----------|---------|
| **Parser** | ✅ 完整 | ✅ 完整 | 保持 |
| **Validator** | ✅ 完整 | ❌ 缺失 | 新增 |
| **Optimizer** | ✅ 完整 | ❌ 缺失 | 后续增强 |
| **Scheduler** | ✅ 完整 | ⚠️ 部分 | 完善集成 |
| **Executor** | ✅ 完整 | ⚠️ 不完整 | 删除冗余、完成集成 |
| **数据流** | 清晰 | 混乱 | 统一通过 ExecutionContext |
| **工厂模式** | ✅ 广泛应用 | ❌ 缺失 | 全面引入 |

**关键收获**: GraphDB 应该完全采用 NebulaGraph 的五层架构（Parser → Validator → Optimizer → Scheduler → Executor），而不是直接执行。这样既能保留现有实现，也能为将来的优化和扩展打好基础。

---

**版本**: v1.0  
**创建时间**: 2025-12-10  
**维护者**: GraphDB 开发团队
