# 调度器迁移映射表

## 概述

本文档详细映射了nebula-graph调度器相关文件到Rust项目的具体迁移路径，包括每个文件中的具体功能、目标位置和必要的调整。

## 文件映射表

### 1. 核心调度器文件

#### 1.1 AsyncMsgNotifyBasedScheduler.h → src/query/scheduler/async_scheduler.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| `class AsyncMsgNotifyBasedScheduler` | `pub struct AsyncMsgNotifyBasedScheduler<S: StorageEngine>` | 使用泛型和trait约束 |
| `folly::Future<Status> schedule()` | `async fn schedule(&mut self, execution_plan: ExecutionPlan<S>) -> Result<ExecutionResult, QueryError>` | 使用Rust的async/await和Result类型 |
| `void waitFinish()` | `async fn wait_finish(&mut self) -> Result<(), QueryError>` | 异步等待实现 |
| `folly::Future<Status> doSchedule(Executor* root)` | `async fn do_schedule(&self, root: &dyn Executor<S>) -> Result<ExecutionResult, QueryError>` | 使用trait对象 |
| `folly::Future<Status> scheduleExecutor()` | `async fn schedule_executor()` | 根据执行器类型分发 |
| `folly::Future<Status> runSelect()` | `async fn run_select()` | SelectExecutor特殊处理 |
| `folly::Future<Status> runLoop()` | `async fn run_loop()` | LoopExecutor特殊处理 |
| `folly::Future<Status> runExecutor()` | `async fn run_executor()` | 通用执行器处理 |
| `folly::Future<Status> runLeafExecutor()` | `async fn run_leaf_executor()` | 叶子节点执行器处理 |
| `Status checkStatus()` | `fn check_status()` | 状态检查逻辑 |
| `void notifyOK()` | `fn notify_ok()` | 成功通知 |
| `void notifyError()` | `fn notify_error()` | 错误通知 |
| `folly::Future<Status> execute()` | `async fn execute()` | 执行器执行 |
| `void addExecuting()` | `fn add_executing()` | 添加执行中状态 |
| `void removeExecuting()` | `fn remove_executing()` | 移除执行中状态 |
| `void setFailStatus()` | `fn set_fail_status()` | 设置失败状态 |
| `bool hasFailStatus()` | `fn has_fail_status()` | 检查失败状态 |
| `std::string formatPrettyId()` | `fn format_pretty_id()` | 格式化ID |
| `std::string formatPrettyDependencyTree()` | `fn format_pretty_dependency_tree()` | 格式化依赖树 |

#### 1.2 AsyncMsgNotifyBasedScheduler.cpp → src/query/scheduler/async_scheduler.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| 构造函数实现 | `impl<S: StorageEngine> AsyncMsgNotifyBasedScheduler<S>` | 使用Rust构造函数模式 |
| `waitFinish()` 实现 | `async fn wait_finish()` | 使用tokio的同步原语 |
| `schedule()` 实现 | `async fn schedule()` | 使用Rust的async/await |
| `doSchedule()` 实现 | `async fn do_schedule()` | 使用HashMap替代std::unordered_map |
| Promise/Future机制 | `MessageNotifier` 结构体 | 使用tokio::sync::oneshot |
| `scheduleExecutor()` switch语句 | match语句 | 使用Rust的match表达式 |
| `runSelect()` 实现 | `async fn run_select()` | 条件分支处理 |
| `runLoop()` 实现 | `async fn run_loop()` | 循环处理 |
| 错误处理链 | `?` 操作符和Result链 | 使用Rust的错误处理 |
| 内存检查 | `MemoryMonitor` 结构体 | 使用Rust的内存监控 |

### 2. 基础调度器文件

#### 2.1 Scheduler.h → src/query/scheduler/types.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| `class Scheduler` | `pub trait QueryScheduler<S: StorageEngine>` | 使用trait定义接口 |
| `virtual folly::Future<Status> schedule() = 0` | `async fn schedule(&mut self, execution_plan: ExecutionPlan<S>) -> Result<ExecutionResult, QueryError>` | 异步方法定义 |
| `virtual void waitFinish() = 0` | `fn wait_finish(&mut self) -> Result<(), QueryError>` | 同步等待方法 |
| `static void analyzeLifetime()` | `fn analyze_lifetime()` | 生命周期分析 |
| `std::string query_` | `query: String` | 查询字符串存储 |

### 3. 执行器逻辑文件

#### 3.1 SelectExecutor.h → src/query/executor/data_processing/logic.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| `class SelectExecutor` | `pub struct SelectExecutor<S: StorageEngine>` | 使用泛型结构体 |
| `folly::Future<Status> execute()` | `async fn execute(&mut self) -> Result<ExecutionResult, QueryError>` | 异步执行方法 |
| 条件表达式处理 | `condition: Expression` | 使用Rust的表达式类型 |
| then/else分支 | `then_body: Box<dyn Executor<S>>`, `else_body: Box<dyn Executor<S>>` | 使用trait对象 |

#### 3.2 SelectExecutor.cpp → src/query/executor/data_processing/logic.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| 构造函数 | `impl<S: StorageEngine> SelectExecutor<S>` | Rust构造函数 |
| `execute()` 实现 | `async fn execute()` | 条件评估和分支选择 |
| 条件表达式评估 | `evaluate_condition()` | 表达式求值逻辑 |

#### 3.3 LoopExecutor.h → src/query/executor/data_processing/logic.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| `class LoopExecutor` | `pub struct LoopExecutor<S: StorageEngine>` | 使用泛型结构体 |
| `folly::Future<Status> execute()` | `async fn execute(&mut self) -> Result<ExecutionResult, QueryError>` | 异步执行方法 |
| 循环条件 | `condition: Expression` | 循环条件表达式 |
| 循环体 | `loop_body: Box<dyn Executor<S>>` | 循环体执行器 |

#### 3.4 LoopExecutor.cpp → src/query/executor/data_processing/logic.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| 构造函数 | `impl<S: StorageEngine> LoopExecutor<S>` | Rust构造函数 |
| `execute()` 实现 | `async fn execute()` | 循环条件检查和执行 |
| 循环终止逻辑 | `finally_` 字段处理 | 循环终止条件 |

### 4. 执行器基础文件

#### 4.1 Executor.h → src/query/executor/base.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| `class Executor` | `pub trait Executor<S: StorageEngine>` | 使用trait定义接口 |
| `virtual folly::Future<Status> execute() = 0` | `async fn execute(&mut self) -> Result<ExecutionResult, QueryError>` | 异步执行方法 |
| `virtual Status open() = 0` | `fn open(&mut self) -> Result<(), QueryError>` | 资源初始化 |
| `virtual Status close() = 0` | `fn close(&mut self) -> Result<(), QueryError>` | 资源清理 |
| `int64_t id() const` | `fn id(&self) -> usize` | 获取执行器ID |
| `const std::string& name() const` | `fn name(&self) -> &str` | 获取执行器名称 |
| `const std::vector<Executor*>& depends()` | `fn depends(&self) -> Vec<usize>` | 获取依赖列表 |

#### 4.2 Executor.cpp → src/query/executor/base.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| 基础执行器实现 | `pub struct BaseExecutor<S: StorageEngine>` | 基础结构体 |
| `Executor::create()` | `create_executor()` 工厂函数 | 执行器创建逻辑 |
| 依赖关系管理 | `depends()` 方法实现 | 依赖列表管理 |

### 5. 计划节点文件

#### 5.1 PlanNode.h → src/query/planner/plan/plan_node.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| `enum class Kind` | `pub enum PlanNodeKind` | 节点类型枚举 |
| `class PlanNode` | `pub trait PlanNode` | 节点trait定义 |
| `virtual std::unique_ptr<PlanNode> clone() const = 0` | `fn clone(&self) -> Box<dyn PlanNode>` | 克隆方法 |
| `virtual void accept(PlanNodeVisitor* visitor) = 0` | `fn accept(&self, visitor: &mut dyn PlanNodeVisitor)` | 访问者模式 |
| `int64_t id() const` | `fn id(&self) -> usize` | 获取节点ID |
| `const std::vector<PlanNode*>& dependencies()` | `fn dependencies(&self) -> Vec<usize>` | 获取依赖节点 |

### 6. 查询上下文文件

#### 6.1 QueryContext.h → src/query/context.rs (新建)

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| `class QueryContext` | `pub struct QueryContext` | 查询上下文结构体 |
| `ExecutionPlan* plan()` | `fn plan(&self) -> &ExecutionPlan` | 获取执行计划 |
| `RequestContext* rctx()` | `fn rctx(&self) -> &RequestContext` | 获取请求上下文 |
| `ExpressionContext* ectx()` | `fn ectx(&self) -> &ExpressionContext` | 获取表达式上下文 |
| `SymTable* symTable()` | `fn sym_table(&self) -> &SymbolTable` | 获取符号表 |

### 7. 表达式系统文件

#### 7.1 Expression.h → src/query/expression/mod.rs

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| `class Expression` | `pub trait Expression` | 表达式trait |
| `virtual Value eval(ExpressionContext& ctx) const = 0` | `fn eval(&self, ctx: &ExpressionContext) -> Result<Value, QueryError>` | 表达式求值 |
| `virtual void accept(ExprVisitor* visitor) = 0` | `fn accept(&self, visitor: &mut dyn ExprVisitor)` | 访问者模式 |
| 表达式类型枚举 | `pub enum ExpressionKind` | 表达式类型 |

### 8. 符号表文件

#### 8.1 SymTable.h → src/query/symbol_table.rs (新建)

| nebula-graph功能 | Rust实现位置 | 调整说明 |
|------------------|-------------|----------|
| `class SymTable` | `pub struct SymbolTable` | 符号表结构体 |
| `Variable* newVariable(const std::string& name)` | `fn new_variable(&mut self, name: String) -> &mut Variable` | 创建变量 |
| `Variable* getVariable(const std::string& name)` | `fn get_variable(&self, name: &str) -> Option<&Variable>` | 获取变量 |
| `bool existsVar(const std::string& name)` | `fn exists_var(&self, name: &str) -> bool` | 检查变量存在 |

## 新增Rust特有文件

### 1. src/query/scheduler/message_notifier.rs

| 功能 | 说明 |
|------|------|
| `MessageNotifier` 结构体 | 基于tokio::sync的消息通知机制 |
| `create_dependency()` 方法 | 创建执行器间依赖关系 |
| `notify_success()` 方法 | 通知执行成功 |
| `notify_error()` 方法 | 通知执行失败 |

### 2. src/query/scheduler/memory_monitor.rs

| 功能 | 说明 |
|------|------|
| `MemoryMonitor` 结构体 | 内存使用监控 |
| `check_memory()` 方法 | 检查内存使用情况 |
| `allocate()` 方法 | 记录内存分配 |
| `deallocate()` 方法 | 记录内存释放 |

### 3. src/query/scheduler/concurrency_controller.rs

| 功能 | 说明 |
|------|------|
| `ConcurrencyController` 结构体 | 并发控制管理 |
| `acquire()` 方法 | 获取执行许可 |
| `release()` 方法 | 释放执行许可 |
| `wait_finish()` 方法 | 等待所有执行完成 |

### 4. src/query/scheduler/error_manager.rs

| 功能 | 说明 |
|------|------|
| `ErrorManager` 结构体 | 错误状态管理 |
| `set_failure()` 方法 | 设置失败状态 |
| `has_failure()` 方法 | 检查失败状态 |
| `take_failure()` 方法 | 获取并清除失败状态 |

## 实现顺序建议

### 第一阶段：基础框架
1. 扩展 `src/query/scheduler/types.rs` - 添加QueryScheduler trait
2. 扩展 `src/query/executor/base.rs` - 添加ExecutorType和executor_type方法
3. 创建 `src/query/scheduler/message_notifier.rs` - 实现消息通知机制

### 第二阶段：特殊执行器
1. 创建 `src/query/executor/data_processing/logic.rs` - 实现SelectExecutor和LoopExecutor
2. 扩展 `src/query/scheduler/async_scheduler.rs` - 添加特殊执行器调度逻辑
3. 创建 `src/query/scheduler/error_manager.rs` - 实现错误管理

### 第三阶段：高级功能
1. 创建 `src/query/scheduler/memory_monitor.rs` - 实现内存监控
2. 创建 `src/query/scheduler/concurrency_controller.rs` - 实现并发控制
3. 扩展 `src/query/scheduler/async_scheduler.rs` - 添加生命周期优化

### 第四阶段：调试和监控
1. 扩展 `src/query/scheduler/async_scheduler.rs` - 添加可视化功能
2. 创建 `src/query/context.rs` - 实现查询上下文
3. 创建 `src/query/symbol_table.rs` - 实现符号表

## Rust特有调整

### 1. 类型系统调整
- 使用trait对象替代C++的虚函数
- 使用泛型替代C++的模板
- 使用枚举替代C++的常量定义

### 2. 内存管理调整
- 利用Rust的所有权系统自动管理内存
- 使用Arc/Mutex替代C++的智能指针
- 避免手动内存管理

### 3. 并发模型调整
- 使用tokio的async/await替代folly::Future
- 使用tokio::sync替代std::mutex和condition_variable
- 使用channel进行消息传递

### 4. 错误处理调整
- 使用Result<T, E>替代C++的异常
- 使用?操作符简化错误处理
- 定义统一的错误类型

### 5. 生命周期调整
- 利用生命周期标注确保引用安全
- 使用借用检查器防止内存安全问题
- 减少显式生命周期管理

## 测试映射

### 单元测试文件映射
| nebula-graph测试 | Rust测试位置 |
|------------------|-------------|
| `AsyncMsgNotifyBasedSchedulerTest.cpp` | `tests/query/scheduler/async_scheduler_test.rs` |
| `SelectExecutorTest.cpp` | `tests/query/executor/logic_test.rs` |
| `LoopExecutorTest.cpp` | `tests/query/executor/logic_test.rs` |

### 集成测试文件映射
| nebula-graph测试 | Rust测试位置 |
|------------------|-------------|
| `SchedulerIntegrationTest.cpp` | `tests/query/scheduler/integration_test.rs` |
| `ExecutorIntegrationTest.cpp` | `tests/query/executor/integration_test.rs` |

## 总结

本映射表详细说明了从nebula-graph到Rust的完整迁移路径，包括每个功能的具体实现位置和必要的调整。通过按照建议的顺序实现，可以确保系统的稳定性和可维护性。