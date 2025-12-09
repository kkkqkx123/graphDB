# 调度器实现方案

## 概述

本文档详细说明了如何将nebula-graph的`AsyncMsgNotifyBasedScheduler`功能迁移到Rust实现中，包括需要从哪些文件迁移什么功能，以及如何根据Rust和C++的差异进行适当调整。

## 当前状态分析

### 已实现的Rust调度器功能

1. **基础调度框架**
   - `AsyncMsgNotifyBasedScheduler` 结构体已创建
   - `ExecutionState` 用于跟踪执行状态
   - `ExecutionPlan` 用于管理执行器和依赖关系
   - 基本的并行执行能力（使用tokio::spawn）

2. **核心调度逻辑**
   - 实现了广度优先搜索调度算法
   - 支持执行器的并行执行
   - 基本的依赖关系检查
   - 执行结果收集和错误处理

### 缺失的核心功能

1. 特殊执行器类型支持（SelectExecutor、LoopExecutor、Argument节点）
2. 高级调度功能（生命周期优化、内存监控）
3. 消息通知机制
4. 调试和监控功能
5. 错误处理和状态管理

## 实现方案

### 1. 特殊执行器类型支持

#### 1.1 SelectExecutor支持

**源文件**: `nebula-3.8.0/src/graph/executor/logic/SelectExecutor.cpp`

**需要迁移的功能**:
- 条件表达式评估
- 根据条件结果选择执行分支
- 分支执行结果处理

**Rust实现调整**:
```rust
// 在 src/query/executor/data_processing/mod.rs 中添加
pub mod logic;

// 在 src/query/executor/data_processing/logic.rs 中实现
pub struct SelectExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    condition: Expression,
    then_body: Box<dyn Executor<S>>,
    else_body: Box<dyn Executor<S>>,
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for SelectExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 评估条件表达式
        let condition_result = self.evaluate_condition(&self.condition)?;
        
        // 根据条件选择执行分支
        if condition_result.is_true() {
            self.then_body.execute().await
        } else {
            self.else_body.execute().await
        }
    }
}
```

**调度器调整**:
```rust
// 在 src/query/scheduler/async_scheduler.rs 中添加
impl<S: StorageEngine + Send + 'static> AsyncMsgNotifyBasedScheduler<S> {
    async fn run_select(
        &self,
        futures: Vec<tokio::task::JoinHandle<Result<ExecutionResult, QueryError>>>,
        select: &mut SelectExecutor<S>,
    ) -> Result<ExecutionResult, QueryError> {
        // 等待所有依赖完成
        for future in futures {
            future.await??;
        }
        
        // 执行选择逻辑
        select.execute().await
    }
}
```

#### 1.2 LoopExecutor支持

**源文件**: `nebula-3.8.0/src/graph/executor/logic/LoopExecutor.cpp`

**需要迁移的功能**:
- 循环条件评估
- 循环体重复执行
- 循环终止条件检查

**Rust实现调整**:
```rust
// 在 src/query/executor/data_processing/logic.rs 中实现
pub struct LoopExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    condition: Expression,
    loop_body: Box<dyn Executor<S>>,
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for LoopExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        loop {
            // 评估循环条件
            let condition_result = self.evaluate_condition(&self.condition)?;
            
            if !condition_result.is_true() {
                break;
            }
            
            // 执行循环体
            self.loop_body.execute().await?;
        }
        
        Ok(ExecutionResult::Success)
    }
}
```

#### 1.3 Argument节点支持

**源文件**: `nebula-3.8.0/src/graph/executor/logic/ArgumentExecutor.cpp` (如果存在)

**需要迁移的功能**:
- 变量依赖关系处理
- 输入变量传递

**Rust实现调整**:
```rust
// 在 src/query/executor/data_processing/logic.rs 中实现
pub struct ArgumentExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    input_var: String,
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for ArgumentExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 从执行上下文获取输入变量
        if let Some(result) = self.context.get_result(&self.input_var) {
            Ok(result.clone())
        } else {
            Err(QueryError::InvalidQuery(format!("Variable {} not found", self.input_var)))
        }
    }
}
```

### 2. 高级调度功能

#### 2.1 生命周期优化（analyzeLifetime）

**源文件**: `nebula-3.8.0/src/graph/scheduler/Scheduler.cpp`

**需要迁移的功能**:
- 变量生命周期分析
- 内存使用优化
- 资源及时释放

**Rust实现调整**:
```rust
// 在 src/query/scheduler/async_scheduler.rs 中添加
impl<S: StorageEngine + Send + 'static> AsyncMsgNotifyBasedScheduler<S> {
    fn analyze_lifetime(&self, root: &dyn Executor<S>) {
        // 使用Rust的所有权系统进行生命周期分析
        // 实现变量引用计数和及时释放
        self.traverse_and_analyze(root, &mut HashMap::new());
    }
    
    fn traverse_and_analyze(
        &self,
        executor: &dyn Executor<S>,
        ref_counts: &mut HashMap<String, usize>,
    ) {
        // 分析执行器的变量引用
        // 更新引用计数
        // 确定变量可以释放的时机
    }
}
```

#### 2.2 内存监控和错误处理

**源文件**: `nebula-3.8.0/src/graph/scheduler/AsyncMsgNotifyBasedScheduler.cpp`

**需要迁移的功能**:
- 内存使用监控
- 内存分配异常处理
- 内存不足错误处理

**Rust实现调整**:
```rust
// 在 src/query/scheduler/async_scheduler.rs 中添加
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MemoryMonitor {
    allocated_bytes: AtomicUsize,
    max_allowed_bytes: usize,
}

impl MemoryMonitor {
    pub fn new(max_allowed_bytes: usize) -> Self {
        Self {
            allocated_bytes: AtomicUsize::new(0),
            max_allowed_bytes,
        }
    }
    
    pub fn check_memory(&self) -> Result<(), QueryError> {
        let current = self.allocated_bytes.load(Ordering::Relaxed);
        if current > self.max_allowed_bytes {
            Err(QueryError::MemoryExceeded(current))
        } else {
            Ok(())
        }
    }
}

// 在执行器执行时添加内存检查
impl<S: StorageEngine + Send + 'static> AsyncMsgNotifyBasedScheduler<S> {
    async fn execute_with_memory_check(
        &self,
        executor: &mut Box<dyn Executor<S>>,
    ) -> Result<ExecutionResult, QueryError> {
        // 执行前检查内存
        self.memory_monitor.check_memory()?;
        
        // 执行执行器
        let result = executor.execute().await;
        
        // 执行后检查内存
        self.memory_monitor.check_memory()?;
        
        result
    }
}
```

### 3. 消息通知机制

#### 3.1 依赖关系构建

**源文件**: `nebula-3.8.0/src/graph/scheduler/AsyncMsgNotifyBasedScheduler.cpp`

**需要迁移的功能**:
- Promise/Future机制
- 依赖关系映射
- 消息通知系统

**Rust实现调整**:
```rust
// 在 src/query/scheduler/async_scheduler.rs 中添加
use tokio::sync::{oneshot, watch};
use std::collections::HashMap;

pub struct MessageNotifier {
    promises: HashMap<usize, Vec<oneshot::Sender<Result<ExecutionResult, QueryError>>>>,
    futures: HashMap<usize, Vec<oneshot::Receiver<Result<ExecutionResult, QueryError>>>>,
}

impl MessageNotifier {
    pub fn new() -> Self {
        Self {
            promises: HashMap::new(),
            futures: HashMap::new(),
        }
    }
    
    pub fn create_dependency(&mut self, from: usize, to: usize) {
        let (tx, rx) = oneshot::channel();
        
        self.promises.entry(from).or_insert_with(Vec::new).push(tx);
        self.futures.entry(to).or_insert_with(Vec::new).push(rx);
    }
    
    pub fn notify_success(&mut self, executor_id: usize, result: ExecutionResult) {
        if let Some(promises) = self.promises.remove(&executor_id) {
            for promise in promises {
                let _ = promise.send(Ok(result.clone()));
            }
        }
    }
    
    pub fn notify_error(&mut self, executor_id: usize, error: QueryError) {
        if let Some(promises) = self.promises.remove(&executor_id) {
            for promise in promises {
                let _ = promise.send(Err(error.clone()));
            }
        }
    }
}
```

#### 3.2 执行器类型识别

**源文件**: `nebula-3.8.0/src/graph/scheduler/AsyncMsgNotifyBasedScheduler.cpp`

**需要迁移的功能**:
- 执行器类型识别
- 不同类型执行器的调度策略

**Rust实现调整**:
```rust
// 在 src/query/executor/base.rs 中添加
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutorType {
    Select,
    Loop,
    Argument,
    Leaf,
    Regular,
}

// 在Executor trait中添加类型方法
#[async_trait]
pub trait Executor<S: StorageEngine + Send + 'static>: Send + Sync {
    // 现有方法...
    
    // 新增方法
    fn executor_type(&self) -> ExecutorType;
}

// 在调度器中实现类型识别
impl<S: StorageEngine + Send + 'static> AsyncMsgNotifyBasedScheduler<S> {
    async fn schedule_by_type(
        &self,
        executor: &mut Box<dyn Executor<S>>,
        dependencies: Vec<oneshot::Receiver<Result<ExecutionResult, QueryError>>>,
    ) -> Result<ExecutionResult, QueryError> {
        match executor.executor_type() {
            ExecutorType::Select => {
                let select_executor = executor.as_any().downcast_mut::<SelectExecutor<S>>()
                    .ok_or(QueryError::InvalidQuery("Failed to downcast SelectExecutor".to_string()))?;
                self.run_select(dependencies, select_executor).await
            },
            ExecutorType::Loop => {
                let loop_executor = executor.as_any().downcast_mut::<LoopExecutor<S>>()
                    .ok_or(QueryError::InvalidQuery("Failed to downcast LoopExecutor".to_string()))?;
                self.run_loop(dependencies, loop_executor).await
            },
            ExecutorType::Argument => {
                self.run_argument(dependencies, executor).await
            },
            ExecutorType::Leaf => {
                self.run_leaf(executor).await
            },
            ExecutorType::Regular => {
                self.run_regular(dependencies, executor).await
            },
        }
    }
}
```

### 4. 调试和监控功能

#### 4.1 执行器树可视化

**源文件**: `nebula-3.8.0/src/graph/scheduler/AsyncMsgNotifyBasedScheduler.cpp`

**需要迁移的功能**:
- 执行器依赖关系树的可视化
- 调试信息输出

**Rust实现调整**:
```rust
// 在 src/query/scheduler/async_scheduler.rs 中添加
impl<S: StorageEngine + Send + 'static> AsyncMsgNotifyBasedScheduler<S> {
    pub fn format_pretty_dependency_tree(&self, root: &dyn Executor<S>) -> String {
        let mut result = String::new();
        self.append_executor(0, root, &mut result);
        result
    }
    
    fn append_executor(&self, indent: usize, executor: &dyn Executor<S>, result: &mut String) {
        let indent_str = " ".repeat(indent);
        result.push_str(&format!("{}[{},{}]\n", indent_str, executor.name(), executor.id()));
        
        // 添加依赖的执行器
        for dep_id in self.get_dependencies(executor.id()) {
            if let Some(dep_executor) = self.get_executor(dep_id) {
                self.append_executor(indent + 1, dep_executor.as_ref(), result);
            }
        }
    }
    
    pub fn format_pretty_id(&self, executor: &dyn Executor<S>) -> String {
        format!("[{},{}]", executor.name(), executor.id())
    }
}
```

#### 4.2 并发控制和同步

**源文件**: `nebula-3.8.0/src/graph/scheduler/AsyncMsgNotifyBasedScheduler.cpp`

**需要迁移的功能**:
- 执行器并发控制
- 同步等待机制

**Rust实现调整**:
```rust
// 在 src/query/scheduler/async_scheduler.rs 中添加
use tokio::sync::{Mutex, Condvar};
use std::sync::Arc;

pub struct ConcurrencyController {
    executing_count: Arc<Mutex<usize>>,
    condvar: Arc<Condvar>,
    max_concurrent: usize,
}

impl ConcurrencyController {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            executing_count: Arc::new(Mutex::new(0)),
            condvar: Arc::new(Condvar::new()),
            max_concurrent,
        }
    }
    
    pub async fn acquire(&self) -> Result<(), QueryError> {
        let mut count = self.executing_count.lock().await;
        
        while *count >= self.max_concurrent {
            count = self.condvar.wait(count).await;
        }
        
        *count += 1;
        Ok(())
    }
    
    pub async fn release(&self) {
        let mut count = self.executing_count.lock().await;
        *count -= 1;
        self.condvar.notify_one();
    }
    
    pub async fn wait_finish(&self) {
        let count = self.executing_count.lock().await;
        let _ = self.condvar.wait_when(count, |c| *c == 0).await;
    }
}
```

### 5. 错误处理和状态管理

#### 5.1 错误传播机制

**源文件**: `nebula-3.8.0/src/graph/scheduler/AsyncMsgNotifyBasedScheduler.cpp`

**需要迁移的功能**:
- 错误状态传播
- 失败状态管理

**Rust实现调整**:
```rust
// 在 src/query/scheduler/async_scheduler.rs 中添加
use std::sync::Arc;

pub struct ErrorManager {
    failed_status: Arc<Mutex<Option<QueryError>>>,
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {
            failed_status: Arc::new(Mutex::new(None)),
        }
    }
    
    pub async fn set_failure(&self, error: QueryError) {
        let mut status = self.failed_status.lock().await;
        if status.is_none() {
            *status = Some(error);
        }
    }
    
    pub async fn has_failure(&self) -> bool {
        let status = self.failed_status.lock().await;
        status.is_some()
    }
    
    pub async fn take_failure(&self) -> Option<QueryError> {
        let mut status = self.failed_status.lock().await;
        status.take()
    }
    
    pub async fn notify_error(&self, promises: &mut Vec<oneshot::Sender<Result<ExecutionResult, QueryError>>>, error: QueryError) {
        for promise in promises.drain(..) {
            let _ = promise.send(Err(error.clone()));
        }
    }
}
```

#### 5.2 执行器状态管理

**源文件**: `nebula-3.8.0/src/graph/scheduler/AsyncMsgNotifyBasedScheduler.cpp`

**需要迁移的功能**:
- 执行器open/close调用
- 执行状态跟踪

**Rust实现调整**:
```rust
// 在 src/query/scheduler/async_scheduler.rs 中添加
impl<S: StorageEngine + Send + 'static> AsyncMsgNotifyBasedScheduler<S> {
    async fn execute_with_lifecycle(
        &self,
        executor: &mut Box<dyn Executor<S>>,
    ) -> Result<ExecutionResult, QueryError> {
        // 调用open方法
        executor.open()?;
        
        // 执行执行器
        let result = executor.execute().await;
        
        // 无论成功还是失败都调用close方法
        let close_result = executor.close();
        
        // 如果close失败，记录错误但不覆盖执行结果
        if let Err(close_error) = close_result {
            log::error!("Executor close failed: {:?}", close_error);
        }
        
        result
    }
}
```

## 实现优先级

### 第一阶段（高优先级）
1. SelectExecutor支持
2. LoopExecutor支持
3. 执行器类型识别
4. 基本的消息通知机制

### 第二阶段（中优先级）
1. Argument节点支持
2. 执行器状态管理（open/close）
3. 错误传播机制
4. 并发控制和同步

### 第三阶段（低优先级）
1. 生命周期优化
2. 内存监控
3. 执行器树可视化
4. 高级调试功能

## Rust与C++差异考虑

### 1. 内存管理
- C++使用手动内存管理和智能指针
- Rust使用所有权系统和借用检查器
- 调整：利用Rust的所有权系统简化内存管理，减少手动内存操作

### 2. 并发模型
- C++使用folly::Future和Promise
- Rust使用tokio::async/await和tokio::sync
- 调整：使用Rust的async/await语法，利用tokio的同步原语

### 3. 错误处理
- C++使用异常和Status对象
- Rust使用Result<T, E>和?操作符
- 调整：使用Rust的Result类型进行错误处理，避免异常

### 4. 类型系统
- C++使用运行时类型信息（RTTI）
- Rust使用编译时类型检查和trait对象
- 调整：使用Rust的trait对象和downcast进行类型识别

### 5. 生命周期
- C++需要手动管理对象生命周期
- Rust通过所有权系统自动管理生命周期
- 调整：利用Rust的生命周期标注和借用检查器确保安全

## 测试策略

### 单元测试
- 为每个新增的执行器类型编写单元测试
- 测试消息通知机制的正确性
- 测试错误处理和状态管理

### 集成测试
- 测试完整的查询执行流程
- 测试复杂依赖关系的调度
- 测试并发执行的正确性

### 性能测试
- 对比Rust实现与C++实现的性能
- 测试内存使用情况
- 测试并发性能

## 总结

本实现方案详细说明了如何将nebula-graph的调度器功能迁移到Rust中，考虑了两种语言的差异，并提供了适当的调整方案。通过分阶段实现，可以确保系统的稳定性和可维护性。