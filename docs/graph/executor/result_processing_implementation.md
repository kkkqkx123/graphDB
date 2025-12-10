# 结果处理执行器实现方案

## 概述

本文档基于对nebula-graph原始实现的分析，为`src/query/executor/result_processing`目录中的空方法提供实现方案。

## 当前状态分析

### 已实现的执行器
- `dedup.rs` - DistinctExecutor ✅
- `limiting.rs` - LimitExecutor, OffsetExecutor ✅
- `sampling.rs` - SampleExecutor ✅
- `topn.rs` - TopNExecutor ✅

### 待实现的执行器
- `aggregation.rs` - AggregateExecutor (空文件)
- `projection.rs` - ProjectExecutor (空文件)
- `sorting.rs` - SortExecutor (空文件)

## nebula-graph实现分析

### AggregateExecutor
**文件位置**: `nebula-3.8.0/src/graph/executor/query/AggregateExecutor.cpp`

**核心逻辑**:
1. 处理`COUNT(*)`的特殊情况
2. 使用哈希表按分组键聚合数据
3. 支持多种聚合函数(COUNT, SUM, AVG, MAX, MIN等)
4. 处理空数据集的情况

**关键数据结构**:
```cpp
std::unordered_map<List, std::vector<std::unique_ptr<AggData>>, std::hash<nebula::List>> result;
```

### ProjectExecutor
**文件位置**: `nebula-3.8.0/src/graph/executor/query/ProjectExecutor.cpp`

**核心逻辑**:
1. 支持多线程处理大数据集
2. 对每行数据应用投影表达式
3. 处理列选择和重命名

**关键特性**:
- 支持并行处理(`FLAGS_max_job_size > 1`)
- 使用`handleJob`函数处理数据块

### SortExecutor
**文件位置**: `nebula-3.8.0/src/graph/executor/query/SortExecutor.cpp`

**核心逻辑**:
1. 验证输入迭代器类型
2. 创建自定义比较器
3. 使用标准库`std::sort`进行排序

**关键特性**:
- 支持多列排序
- 支持升序/降序
- 仅支持顺序迭代器

## Rust实现方案

### 1. AggregateExecutor实现方案

#### 设计思路
```rust
pub struct AggregateExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    group_keys: Vec<String>,           // 分组键
    aggregate_functions: Vec<AggregateFunction>, // 聚合函数
    input_executor: Option<Box<dyn Executor<S>>>,
}

pub enum AggregateFunction {
    Count,
    Sum(String),    // 字段名
    Avg(String),    // 字段名
    Max(String),    // 字段名
    Min(String),    // 字段名
}
```

#### 实现步骤
1. 处理`COUNT(*)`的特殊优化
2. 使用`HashMap`按分组键聚合数据
3. 为每个聚合函数维护状态
4. 支持空数据集的处理

### 2. ProjectExecutor实现方案

#### 设计思路
```rust
pub struct ProjectExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    columns: Vec<ProjectionColumn>,    // 投影列定义
    input_executor: Option<Box<dyn Executor<S>>>,
}

pub struct ProjectionColumn {
    pub name: String,                   // 输出列名
    pub expression: Expression,         // 投影表达式
}
```

#### 实现步骤
1. 支持单线程和多线程处理模式
2. 对每行数据应用投影表达式
3. 处理列选择和重命名
4. 支持表达式求值

### 3. SortExecutor实现方案

#### 设计思路
```rust
pub struct SortExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    sort_columns: Vec<SortColumn>,      // 排序列定义
    input_executor: Option<Box<dyn Executor<S>>>,
}

pub struct SortColumn {
    pub name: String,                   // 列名
    pub ascending: bool,                // 排序方向
}
```

#### 实现步骤
1. 验证输入结果类型
2. 创建自定义比较器
3. 使用`sort_by`进行排序
4. 支持多列排序

## 实现优先级

1. **ProjectExecutor** - 基础且必要
2. **SortExecutor** - 常用功能
3. **AggregateExecutor** - 复杂但重要

## 技术要点

### 内存管理
- 使用Rust的所有权系统避免内存泄漏
- 合理使用`Arc`和`Mutex`处理并发

### 错误处理
- 使用`Result`类型处理错误
- 提供详细的错误信息

### 性能优化
- 避免不必要的克隆
- 使用迭代器避免中间集合
- 支持并行处理大数据集

## 测试策略

1. 单元测试每个执行器的核心逻辑
2. 集成测试执行器链
3. 性能测试大数据集处理

## 下一步行动

1. 实现ProjectExecutor
2. 实现SortExecutor  
3. 实现AggregateExecutor
4. 编写测试用例
5. 性能优化