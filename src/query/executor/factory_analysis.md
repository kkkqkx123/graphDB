# ExecutorFactory 分析报告

## 当前职责分析

### 核心职责
`ExecutorFactory` 的核心职责是**根据执行计划节点创建对应的执行器实例**，这是一个典型的工厂模式实现，职责单一且明确。

### 当前实现的功能

1. **执行器创建器管理**
   - 维护 `PlanNodeKind` 到 `ExecutorCreator` 的映射关系
   - 支持动态注册新的执行器创建器

2. **执行器创建**
   - 根据计划节点类型查找对应的创建器
   - 调用创建器生成执行器实例

3. **默认创建器注册**
   - 自动注册所有内置的执行器创建器
   - 覆盖数据访问、结果处理、数据处理、图遍历和基础执行器

4. **执行器ID生成**
   - 提供唯一的执行器ID生成功能

5. **聚合表达式解析**
   - 解析和验证聚合表达式

## 问题分析

### 1. creators 模块引用问题

当前代码中引用了以下未实现的模块：
```rust
// 数据访问执行器创建器
mod data_access;
pub use data_access::{ScanEdgesCreator, ScanVerticesCreator};

// 结果处理执行器创建器
mod result_processing;
pub use result_processing::{
    AggregateCreator, FilterCreator, LimitCreator, ProjectCreator, SortCreator,
};

// 数据处理执行器创建器
mod data_processing;
pub use data_processing::JoinCreator;

// 图遍历执行器创建器
mod graph_traversal;
pub use graph_traversal::ExpandCreator;

// 基础执行器创建器
mod base;
pub use base::{DefaultCreator, StartCreator};
```

这些模块在 `src/query/executor/factory.rs` 的 `creators` 子模块中被引用，但实际上并不存在。

### 2. ExecutorFactory::new() 方法不一致

在 `QueryPipelineManager` 中调用：
```rust
let executor_factory = ExecutorFactory::new(Arc::clone(&storage));
```

但在 `ExecutorFactory` 实现中：
```rust
pub fn new() -> Self {
    // 不接受任何参数
}
```

这导致了编译错误。

### 3. ExecutorFactory 缺少 execute_plan 方法

在 `QueryPipelineManager` 中调用：
```rust
self.executor_factory.execute_plan(query_context, plan).await
```

但 `ExecutorFactory` 并没有实现 `execute_plan` 方法。

### 4. 职责混乱

`ExecutorFactory` 当前包含了多个不相关的功能：
- 执行器创建（核心职责）
- ID生成（应该独立）
- 聚合表达式解析（应该移到解析器模块）

## 引用分析

### 引用 ExecutorFactory 的模块

1. **src/lib.rs** - 公开导出
2. **src/core/query_pipeline_manager.rs** - 用于创建执行器
3. **src/core/executor_factory.rs** - 包装器，提供额外功能

### 实际需要的功能

从引用分析可以看出，`ExecutorFactory` 需要提供：

1. **创建执行器** - 根据计划节点创建执行器实例
2. **执行计划** - 可选，但当前实现中缺少此方法

## 建议的重构方案

### 1. 修复 creators 模块引用

要么实现这些创建器模块，要么移除这些引用。考虑到当前项目结构，建议：

- 将现有的执行器实现（在 `data_processing`、`result_processing` 等目录中）适配为创建器模式
- 或者简化实现，直接在 `ExecutorFactory` 中创建执行器

### 2. 修复 ExecutorFactory::new() 方法

要么修改调用方不传递参数，要么修改 `new()` 方法接受存储引擎参数。

### 3. 移除不相关功能

- 将 `ExecutorIdGenerator` 移到独立模块
- 将 `aggregation` 模块移到解析器相关模块

### 4. 实现 execute_plan 方法

要么在 `ExecutorFactory` 中实现，要么让调用方使用 `ExecutorFactoryWrapper` 中的实现。

## 结论

当前的 `ExecutorFactory` 实现存在多个问题，主要是：
1. 引用了不存在的模块
2. 方法签名不匹配
3. 包含了不相关的功能
4. 缺少必要的方法实现

建议进行重构，使其专注于执行器创建的核心职责，并修复当前的编译错误。