# 数据处理执行器迁移计划

## 概述

本文档详细分析了 `src/query/executor/data_processing` 目录与 nebula-graph 的差异，并提供了完整的迁移和实现计划。

## 当前实现状态

### 已实现的执行器

| 执行器 | 实现状态 | 完成度 | 备注 |
|--------|----------|--------|------|
| FilterExecutor | 基础框架 | 30% | 缺少条件表达式处理 |
| ExpandExecutor | 单步扩展 | 60% | 缺少多步扩展和采样 |
| InnerJoinExecutor | 哈希连接 | 70% | 缺少并行优化 |
| LeftJoinExecutor | 左连接 | 70% | 缺少并行优化 |
| UnionExecutor | 集合运算 | 80% | 基本功能完整 |
| IntersectExecutor | 集合运算 | 80% | 基本功能完整 |
| MinusExecutor | 集合运算 | 80% | 基本功能完整 |
| UnwindExecutor | 列表展开 | 85% | 功能基本完整 |

### 缺失的执行器

| 执行器 | nebula-graph 对应 | 优先级 | 功能描述 |
|--------|------------------|--------|----------|
| LoopExecutor | LoopExecutor | 高 | 循环控制逻辑 |
| TraverseExecutor | TraverseExecutor | 高 | 多步图遍历 |
| DedupExecutor | DedupExecutor | 高 | 数据去重 |
| SampleExecutor | SampleExecutor | 高 | 数据采样 |
| AssignExecutor | AssignExecutor | 中 | 变量赋值 |
| AppendVerticesExecutor | AppendVerticesExecutor | 中 | 附加顶点属性 |
| PatternApplyExecutor | PatternApplyExecutor | 中 | 模式应用 |
| RollUpApplyExecutor | RollUpApplyExecutor | 中 | 聚合应用 |
| ShortestPathExecutor | ShortestPathExecutor | 低 | 最短路径算法 |
| ExpandAllExecutor | ExpandExecutor | 低 | 全扩展 |

## 详细实现计划

### 1. FilterExecutor - 条件过滤执行器

#### 当前问题
- 只返回输入结果，缺少实际过滤逻辑
- 没有集成表达式引擎

#### 实现方案
```rust
// 需要添加的核心功能
impl<S: StorageEngine> FilterExecutor<S> {
    // 评估条件表达式
    async fn evaluate_condition(&self, row: &Row) -> Result<bool, QueryError>;
    
    // 应用过滤条件
    fn apply_filter(&self, input: ExecutionResult) -> Result<ExecutionResult, QueryError>;
}
```

#### 关键点
- 集成表达式引擎进行条件评估
- 支持多种数据类型的过滤
- 优化过滤性能

### 2. LoopExecutor - 循环控制执行器

#### 当前问题
- 只有占位符实现
- 缺少循环控制逻辑

#### 实现方案
```rust
pub struct LoopExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    condition: Expression,
    body_executor: Box<dyn Executor<S>>,
    max_iterations: Option<usize>,
    current_iteration: usize,
}

impl<S: StorageEngine> LoopExecutor<S> {
    // 评估循环条件
    async fn evaluate_condition(&self) -> Result<bool, QueryError>;
    
    // 执行循环体
    async fn execute_body(&mut self) -> Result<ExecutionResult, QueryError>;
}
```

#### 关键点
- 支持条件评估和循环终止
- 防止无限循环
- 支持最大迭代次数限制

### 3. TraverseExecutor - 图遍历执行器

#### 当前问题
- 缺少多步遍历实现
- 没有路径构建功能

#### 实现方案
```rust
pub struct TraverseExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    step_range: (usize, usize),  // 最小和最大步数
    edge_types: Option<Vec<String>>,
    edge_direction: EdgeDirection,
    vertex_filter: Option<Expression>,
    edge_filter: Option<Expression>,
    generate_path: bool,
}

impl<S: StorageEngine> TraverseExecutor<S> {
    // 执行多步遍历
    async fn multi_step_traverse(&mut self, start_vertices: Vec<Value>) -> Result<ExecutionResult, QueryError>;
    
    // 构建路径
    fn build_paths(&self, traversal_result: &TraversalResult) -> Vec<Path>;
}
```

#### 关键点
- 支持可配置的步数范围
- 实现高效的路径构建
- 支持顶点和边过滤

### 4. DedupExecutor - 去重执行器

#### 实现方案
```rust
pub struct DedupExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    input_var: String,
    dedup_keys: Option<Vec<String>>,  // 指定去重的键
}

impl<S: StorageEngine> DedupExecutor<S> {
    // 执行去重操作
    fn execute_dedup(&self, input: ExecutionResult) -> Result<ExecutionResult, QueryError>;
    
    // 基于哈希的去重
    fn hash_based_dedup(&self, rows: Vec<Row>) -> Vec<Row>;
}
```

#### 关键点
- 高效的去重算法
- 支持指定键去重
- 内存优化

### 5. SampleExecutor - 采样执行器

#### 实现方案
```rust
pub struct SampleExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    input_var: String,
    count: Expression,  // 采样数量
    method: SampleMethod,  // 采样方法
}

pub enum SampleMethod {
    Random,
    Reservoir,
    Systematic,
}
```

#### 关键点
- 支持多种采样算法
- 处理大数据集的内存效率
- 支持动态采样数量

## 性能优化策略

### 1. 内存管理
- 实现内存使用监控
- 优化大数据集处理
- 实现流式处理

### 2. 并行处理
- 利用 Rust 的并发特性
- 实现工作窃取调度
- 优化锁竞争

### 3. 缓存策略
- 实现查询结果缓存
- 优化重复计算
- 智能缓存失效

## 错误处理机制

### 1. 统一错误类型
```rust
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Expression evaluation error: {0}")]
    ExpressionError(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    
    #[error("Memory limit exceeded")]
    MemoryLimitExceeded,
    
    #[error("Timeout error")]
    TimeoutError,
}
```

### 2. 错误恢复策略
- 实现断点续传
- 支持部分结果返回
- 提供详细错误信息

## 测试策略

### 1. 单元测试
- 每个执行器的独立测试
- 边界条件测试
- 错误场景测试

### 2. 集成测试
- 执行器链测试
- 性能基准测试
- 内存使用测试

### 3. 端到端测试
- 完整查询流程测试
- 与 nebula-graph 兼容性测试

## 实施时间表

### 第一阶段（2周）- 核心功能
1. FilterExecutor 条件表达式实现
2. LoopExecutor 循环控制逻辑
3. DedupExecutor 去重功能
4. SampleExecutor 采样功能

### 第二阶段（2周）- 图遍历
1. TraverseExecutor 多步遍历
2. ExpandExecutor 多步扩展
3. 路径构建优化

### 第三阶段（2周）- 性能优化
1. Join 执行器并行优化
2. 内存管理优化
3. 缓存策略实现

### 第四阶段（1周）- 完善功能
1. ShortestPathExecutor 实现
2. 错误处理完善
3. 文档和测试补充

## 技术债务和风险

### 1. 技术债务
- 表达式引擎集成复杂度
- 内存管理实现难度
- 并发安全性保证

### 2. 风险缓解
- 分阶段实施
- 充分测试
- 性能监控

## 总结

本迁移计划提供了从 nebula-graph 到 Rust 实现的完整路径，重点关注核心功能的实现和性能优化。通过分阶段实施，可以确保系统的稳定性和可维护性。