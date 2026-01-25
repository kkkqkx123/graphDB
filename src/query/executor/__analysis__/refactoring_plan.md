# GraphDB 执行器模块重构方案

## 一、问题概述

### 1.1 当前问题总结

经过对 GraphDB 执行器模块的深入分析，发现以下核心问题需要优先解决：

**代码重复与冗余问题**

执行器模块中存在明显的基础类型定义重复问题，具体表现为 BaseExecutor 和 ExecutionResult 两个核心类型在多个文件中重复定义。这种重复不仅增加了代码维护的复杂度，还容易导致类型不一致和接口混乱。

在 base.rs 文件中定义的 BaseExecutor 包含了 ExecutionContext 字段，用于存储执行过程中的中间结果和变量。而 traits.rs 文件中定义的同名 BaseExecutor 则没有这个字段，采用了不同的设计理念。两个版本的 BaseExecutor 字段定义存在差异，base.rs 版本的字段包括 id、name、description、storage、context、is_open 和 stats，而 traits.rs 版本的字段包括 storage、id、name、description、is_open 和 stats。这种不一致会导致开发者在选择使用哪个版本时产生困惑。

ExecutionResult 类型的重复定义同样值得关注。traits.rs 中定义了新的 ExecutionResult 枚举，而 base.rs 中保留了标记为 Legacy 的 OldExecutionResult 枚举。虽然 OldExecutionResult 标记为遗留类型，但代码库中仍然存在对其的引用，表明遗留代码的清理工作尚未完成。

**执行器设计不一致问题**

当前代码库中存在多种不同的执行器基础类型，开发者需要根据具体情况选择使用哪种类型，这增加了学习和使用成本。这种不一致主要体现在以下几个方面：

首先是基础执行器使用混乱的问题。data_access.rs 中的执行器使用来自 base.rs 的 BaseExecutor，result_processing/ 中的执行器使用 BaseResultProcessor，部分执行器则选择不嵌入任何基础类型直接内联实现 Executor trait。这种多元化的设计虽然提供了灵活性，但也导致了更大的代码重复和维护困难。

其次是输入处理机制不统一的问题。执行器获取输入数据的方式存在多种不同的设计模式。第一种是 InputExecutor Trait，在 base.rs 中定义，提供了 set_input 和 get_input 方法。第二种是 BaseResultProcessor.input 字段，通过结构体字段存储输入结果。第三种是直接访问 ExecutionContext，缺乏类型安全性。这三种模式各有优缺点，但缺乏统一标准使得代码库的一致性下降。

最后是结果处理接口分散的问题。ExecutionResult 和 ExecutionStats 在 traits.rs 中定义，而 ResultProcessor 和 BaseResultProcessor 在 result_processing/traits.rs 中定义。这种分散的设计使得新增结果处理功能时需要同时修改多个文件，难以建立统一的结果处理流水线。

**并行处理未启用问题**

在 data_processing/join/mod.rs 中，并行处理模块被明确注释禁用。虽然 parallel.rs 中实现了完整的并行 JOIN 框架，但当前单线程版本尚未稳定，无法在生产环境中使用。FilterExecutor 虽然导入了 rayon 库以支持并行处理，但这种并行处理仅限于单个执行器内部的数据处理，并未实现跨执行器的并行调度。

### 1.2 修改目标

本次重构的核心目标是消除代码重复、统一设计规范，为后续执行器类型扩展奠定坚实基础。具体目标包括：

统一基础类型定义，将所有执行器基础类型集中到一个模块中，消除重复定义，确保类型一致性。规范执行器继承体系，建立清晰的执行器分类标准，明确每类执行器应实现的接口。统一输入处理机制，采用单一标准的输入获取方式，简化执行器之间的数据传递。清理遗留代码，移除 OldExecutionResult 等遗留类型和相关引用。保留并行框架，为后续启用并行处理保留代码基础。

## 二、修改方案

### 2.1 目录结构调整

为了统一管理执行器的基础类型，建议对目录结构进行如下调整：

```
src/query/executor/
├── __analysis__/           # 保留：分析文档目录
├── base/                   # 新增：基础类型统一模块
│   ├── mod.rs              # 统一导出
│   ├── executor_base.rs    # BaseExecutor 统一定义
│   ├── execution_context.rs # ExecutionContext 统一定义
│   ├── execution_result.rs # ExecutionResult 统一定义
│   └── executor_stats.rs   # ExecutorStats 统一定义
├── data_access.rs          # 保持：数据访问执行器
├── data_modification.rs    # 保持：数据修改执行器
├── factory.rs              # 修改：使用统一的基础类型
├── graph_query_executor.rs # 保持：图查询执行器入口
├── logic/                  # 保持：逻辑控制执行器
├── result_processing/      # 保持：结果处理执行器
├── data_processing/        # 保持：数据处理执行器
├── traits.rs               # 删除：迁移到 base/ 目录
├── base.rs                 # 删除：迁移到 base/ 目录
├── mod.rs                  # 修改：更新导出
├── object_pool.rs          # 保持：对象池管理
├── recursion_detector.rs   # 保持：递归检测机制
└── tag_filter.rs           # 保持：标签过滤
```

### 2.2 BaseExecutor 统一定义

**设计决策**

选择 traits.rs 中的 BaseExecutor 定义作为标准实现，因为该版本更简洁，且已经考虑了与 HasStorage trait 的分离。但需要添加 ExecutionContext 字段以支持执行过程中的中间结果存储。

**统一后的 BaseExecutor 定义**

```rust
use crate::core::error::DBError;
use crate::storage::StorageEngine;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// 执行器统计信息
#[derive(Debug, Clone, Default)]
pub struct ExecutorStats {
    pub num_rows: usize,
    pub exec_time_us: u64,
    pub total_time_us: u64,
    pub other_stats: HashMap<String, String>,
}

impl ExecutorStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_row(&mut self, count: usize) {
        self.num_rows += count;
    }

    pub fn add_exec_time(&mut self, duration: Duration) {
        self.exec_time_us += duration.as_micros() as u64;
    }

    pub fn add_total_time(&mut self, duration: Duration) {
        self.total_time_us += duration.as_micros() as u64;
    }

    pub fn add_stat(&mut self, key: String, value: String) {
        self.other_stats.insert(key, value);
    }

    pub fn get_stat(&self, key: &str) -> Option<&String> {
        self.other_stats.get(key)
    }
}

/// 执行结果类型
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Values(Vec<crate::core::Value>),
    Vertices(Vec<crate::core::Vertex>),
    Edges(Vec<crate::core::Edge>),
    DataSet(crate::core::DataSet),
    Result(CoreResult),
    Success,
    Error(String),
    Count(usize),
    Paths(Vec<crate::core::vertex_edge_path::Path>),
}

impl ExecutionResult {
    pub fn count(&self) -> usize {
        match self {
            ExecutionResult::Values(v) => v.len(),
            ExecutionResult::Vertices(v) => v.len(),
            ExecutionResult::Edges(v) => v.len(),
            ExecutionResult::DataSet(ds) => ds.rows.len(),
            ExecutionResult::Result(r) => r.row_count(),
            ExecutionResult::Count(c) => *c,
            ExecutionResult::Success => 0,
            ExecutionResult::Error(_) => 0,
            ExecutionResult::Paths(p) => p.len(),
        }
    }
}

/// 结果类型别名
pub type DBResult<T> = Result<T, DBError>;

/// 统一的执行器 trait
#[async_trait]
pub trait Executor<S: StorageEngine>: Send + Sync {
    async fn execute(&mut self) -> DBResult<ExecutionResult>;
    fn open(&mut self) -> DBResult<()>;
    fn close(&mut self) -> DBResult<()>;
    fn is_open(&self) -> bool;
    fn id(&self) -> i64;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn stats(&self) -> &ExecutorStats;
    fn stats_mut(&mut self) -> &mut ExecutorStats;
    fn check_memory(&self) -> DBResult<()> {
        Ok(())
    }
}

/// 存储访问 trait
pub trait HasStorage<S: StorageEngine> {
    fn get_storage(&self) -> &Arc<Mutex<S>>;
}

/// 输入访问 trait
pub trait HasInput<S: StorageEngine> {
    fn get_input(&self) -> Option<&ExecutionResult>;
    fn set_input(&mut self, input: ExecutionResult);
}

/// 执行上下文
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub results: HashMap<String, ExecutionResult>,
    pub variables: HashMap<String, crate::core::Value>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_result(&mut self, name: String, result: ExecutionResult) {
        self.results.insert(name, result);
    }

    pub fn get_result(&self, name: &str) -> Option<&ExecutionResult> {
        self.results.get(name)
    }

    pub fn set_variable(&mut self, name: String, value: crate::core::Value) {
        self.variables.insert(name, value);
    }

    pub fn get_variable(&self, name: &str) -> Option<&crate::core::Value> {
        self.variables.get(name)
    }
}

/// 基础执行器
#[derive(Clone, Debug)]
pub struct BaseExecutor<S: StorageEngine> {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub storage: Option<Arc<Mutex<S>>>,
    pub context: Option<ExecutionContext>,
    is_open: bool,
    stats: ExecutorStats,
}

impl<S: StorageEngine> BaseExecutor<S> {
    pub fn new(id: i64, name: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage: Some(storage),
            context: None,
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    pub fn without_storage(id: i64, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage: None,
            context: None,
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    pub fn with_context(id: i64, name: String, storage: Arc<Mutex<S>>, context: ExecutionContext) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            storage: Some(storage),
            context: Some(context),
            is_open: false,
            stats: ExecutorStats::new(),
        }
    }

    pub fn get_stats(&self) -> &ExecutorStats {
        &self.stats
    }

    pub fn get_stats_mut(&mut self) -> &mut ExecutorStats {
        &mut self.stats
    }
}

impl<S: StorageEngine> HasStorage<S> for BaseExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.storage.as_ref().expect("Storage not set")
    }
}

#[async_trait]
impl<S: StorageEngine> Executor<S> for BaseExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = Ok(ExecutionResult::Success);
        self.stats_mut().add_total_time(start.elapsed());
        result
    }

    fn open(&mut self) -> DBResult<()> {
        self.is_open = true;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.is_open = false;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn stats(&self) -> &ExecutorStats {
        &self.stats
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        &mut self.stats
    }
}
```

### 2.3 移除 OldExecutionResult

**修改策略**

OldExecutionResult 枚举仅在 base.rs 中定义和使用，需要完全移除该类型和相关辅助函数。所有使用 OldExecutionResult 的代码应迁移到 ExecutionResult。

**需要修改的文件**

检查 data_access.rs 和其他文件中是否使用了 OldExecutionResult，将其替换为 ExecutionResult。

### 2.4 统一输入处理机制

**设计决策**

采用统一的 HasInput trait 作为标准输入获取方式，替代当前分散的 InputExecutor、BaseResultProcessor.input 和 ExecutionContext 三种模式。

**统一的 HasInput trait**

```rust
/// 输入访问 trait - 统一输入处理机制
pub trait HasInput<S: StorageEngine> {
    fn get_input(&self) -> Option<&ExecutionResult>;
    fn set_input(&mut self, input: ExecutionResult);
}
```

**执行器修改示例**

对于 ProjectExecutor，需要将使用 InputExecutor trait 的方式改为使用 HasInput trait：

```rust
pub struct ProjectExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    columns: Vec<ProjectionColumn>,
    input: Option<ExecutionResult>,
}

impl<S: StorageEngine> HasInput<S> for ProjectExecutor<S> {
    fn get_input(&self) -> Option<&ExecutionResult> {
        self.input.as_ref()
    }

    fn set_input(&mut self, input: ExecutionResult) {
        self.input = Some(input);
    }
}
```

### 2.5 保留并行框架

**修改策略**

并行框架（parallel.rs）目前被注释禁用，这是合理的决策，因为单线程版本尚未稳定。本次修改不启用并行框架，但保留其代码基础，为后续稳定后启用做好准备。

**需要做的事情**

在 parallel.rs 中添加稳定性标记，说明该模块当前为实验性质：

```rust
//! 并行处理模块（实验性质）
//!
//! 当前版本尚未稳定，请勿在生产环境中使用。
//! 启用方法：取消 data_processing/join/mod.rs 中 parallel 模块的注释
//!
```

### 2.6 修改顺序与影响范围

为了确保修改过程可控，建议按以下顺序进行：

**第一阶段：创建统一基础类型模块**

创建 base/ 目录及其文件，定义统一的 BaseExecutor、ExecutionResult、ExecutorStats、ExecutionContext 和相关 trait。这一阶段不修改现有代码，仅创建新模块。

**第二阶段：更新模块导出**

修改 executor/mod.rs，从 base/ 模块导入并重新导出所有基础类型。更新 traits.rs 和 base.rs 的导出，确保新模块被正确引用。

**第三阶段：迁移执行器实现**

按照以下顺序迁移各个执行器模块：
- data_access.rs（5 个执行器）
- data_modification.rs（1 个执行器）
- logic/（3 个执行器）
- result_processing/（12 个执行器）
- data_processing/（8 个执行器）
- factory.rs

**第四阶段：清理遗留代码**

移除 base.rs 中的 OldExecutionResult 相关代码。移除 traits.rs 中的重复定义（保留一个版本）。

**第五阶段：验证与测试**

运行 cargo build 和 cargo test，确保所有修改正确。检查是否有遗漏的编译错误。

## 三、文件修改详情

### 3.1 新建文件清单

| 文件路径 | 说明 |
|----------|------|
| src/query/executor/base/mod.rs | 基础模块入口，统一导出 |
| src/query/executor/base/executor_base.rs | BaseExecutor 定义 |
| src/query/executor/base/execution_context.rs | ExecutionContext 定义 |
| src/query/executor/base/execution_result.rs | ExecutionResult 定义 |
| src/query/executor/base/executor_stats.rs | ExecutorStats 定义 |

### 3.2 修改文件清单

| 文件路径 | 修改内容 |
|----------|----------|
| src/query/executor/mod.rs | 更新导出，引入 base/ 模块 |
| src/query/executor/traits.rs | 保留 trait 定义，移除重复类型 |
| src/query/executor/base.rs | 移除 OldExecutionResult，引用 base/ 模块 |
| src/query/executor/data_access.rs | 更新 BaseExecutor 引用 |
| src/query/executor/data_modification.rs | 更新 BaseExecutor 引用 |
| src/query/executor/factory.rs | 更新 BaseExecutor 引用 |
| src/query/executor/logic/mod.rs | 更新引用 |
| src/query/executor/result_processing/mod.rs | 更新引用 |
| src/query/executor/result_processing/traits.rs | 保留 ResultProcessor，引用基础类型 |
| src/query/executor/result_processing/*.rs | 更新各执行器的引用 |

## 四、风险评估与应对措施

### 4.1 主要风险

**编译错误风险**

由于涉及大量文件的引用路径修改，可能出现遗漏修改导致的编译错误。应对措施是分阶段进行修改，每完成一个阶段都进行编译验证。

**类型不兼容风险**

ExecutionResult 和 OldExecutionResult 的变体设计存在差异，直接替换可能导致类型不兼容。应对措施是仔细对比两个枚举的变体，确保一一对应。

**测试失败风险**

修改可能影响现有测试用例的执行结果。应对措施是修改后运行完整测试套件，确保测试通过。

### 4.2 验证步骤

每个修改阶段完成后，执行以下验证步骤：

```bash
cd src/query/executor
cargo check --lib
cargo test --lib -- --nocapture
```

确认无编译错误和测试失败后，再进行下一阶段修改。

## 五、预期效果

完成本次重构后，执行器模块将具有以下改进：

**代码组织更清晰**

基础类型集中在 base/ 目录中，职责划分明确。新开发者可以通过阅读 base/ 目录快速理解执行器的基本架构。

**消除重复定义**

BaseExecutor 和 ExecutionResult 不再重复定义，避免了类型不一致的问题。所有执行器使用统一的类型定义。

**设计规范统一**

输入处理采用统一的 HasInput trait，消除了多种设计模式混用的问题。执行器继承体系更加清晰。

**维护成本降低**

修改基础类型时只需修改 base/ 目录中的文件，无需在多个文件中查找和修改。代码复用率提高，维护成本降低。

**为后续扩展奠定基础**

统一的架构设计使得后续添加新执行器类型更加规范和便捷。执行器类型扩展时可以遵循统一的设计模式。
