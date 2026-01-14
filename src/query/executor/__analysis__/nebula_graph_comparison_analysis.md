# Executor 模块简化实现分析与改进方案

## 📋 分析概述

本文档对比分析了 GraphDB 的 `src/query/executor` 模块与 NebulaGraph 3.8.0 的实现，识别了简化的实现点，并提出了改进建议。

## 🔍 对比分析

### 1. Executor Trait 设计

#### GraphDB 当前实现
```rust
// src/query/executor/traits.rs
#[async_trait]
pub trait Executor<S: StorageEngine>: Send + Sync {
    async fn execute(&mut self) -> DBResult<ExecutionResult>;
    fn open(&mut self) -> DBResult<()>;
    fn close(&mut self) -> DBResult<()>;
    fn is_open(&self) -> bool;
    fn id(&self) -> i64;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
}
```

#### NebulaGraph 完整实现
```cpp
// nebula-3.8.0/src/graph/executor/Executor.h
virtual Status open();
virtual folly::Future<Status> execute() = 0;
virtual Status close();

// 执行器依赖关系管理
const std::set<Executor *> &depends() const;
const std::set<Executor *> &successors() const;
Executor *dependsOn(Executor *dep);

// 性能统计
uint64_t numRows_{0};
uint64_t execTime_{0};
time::Duration totalDuration_;

// 内存管理
Status checkMemoryWatermark();
```

**简化点**：
- ❌ 缺少执行器依赖关系管理
- ❌ 缺少性能统计功能
- ❌ 缺少内存水位检查机制
- ❌ 缺少执行器拓扑管理

---

### 2. ExpandExecutor 实现

#### GraphDB 当前实现
```rust
// src/query/executor/data_processing/graph_traversal/expand.rs
async fn expand_step(&mut self, input_nodes: Vec<Value>) -> Result<Vec<Value>, QueryError> {
    let mut expanded_nodes = Vec::new();
    
    for node_id in input_nodes {
        if self.visited_nodes.contains(&node_id) {
            continue;
        }
        self.visited_nodes.insert(node_id.clone());
        let neighbors = self.get_neighbors(&node_id).await?;
        self.adjacency_cache.insert(node_id.clone(), neighbors.clone());
        
        for neighbor in neighbors {
            if !self.visited_nodes.contains(&neighbor) {
                expanded_nodes.push(neighbor);
            }
        }
    }
    Ok(expanded_nodes)
}
```

**简化点**：
- ❌ 缺少多步扩展的递归优化
- ❌ 缺少采样支持 (`sample_`, `stepLimits_`)
- ❌ 缺少性能计时和状态统计
- ❌ 缺少 `GetDstBySrc` 和 `getNeighbors` 两种模式的区分
- ❌ 缺少 `joinInput` 支持

#### NebulaGraph 完整实现
```cpp
// nebula-3.8.0/src/graph/executor/query/ExpandExecutor.cpp
folly::Future<Status> ExpandExecutor::execute() {
    maxSteps_ = expand_->maxSteps();
    sample_ = expand_->sample();
    stepLimits_ = expand_->stepLimits();

    if (maxSteps_ == 0) {
        // 直接返回
    }
    if (expand_->joinInput() || !stepLimits_.empty()) {
        return getNeighbors();  // 需要连接或有限制
    }
    return GetDstBySrc();  // 简单模式
}

folly::Future<Status> ExpandExecutor::GetDstBySrc() {
    currentStep_++;
    // RPC 调用获取邻居
    return storageClient->getDstBySrc(...)
        .thenValue([this](StorageRpcResponse<GetDstBySrcResponse>&& resps) {
            if (currentStep_ < maxSteps_) {
                return GetDstBySrc();  // 递归继续扩展
            }
            // 构建结果
        });
}
```

---

### 3. FilterExecutor 实现

#### GraphDB 当前实现
```rust
// src/query/executor/result_processing/filter.rs
fn apply_filter(&self, dataset: &mut DataSet) -> DBResult<()> {
    let mut filtered_rows = Vec::new();
    
    for row in &dataset.rows {
        let mut context = DefaultExpressionContext::new();
        for (i, col_name) in dataset.col_names.iter().enumerate() {
            if i < row.len() {
                context.set_variable(col_name.clone(), row[i].clone());
            }
        }
        
        let condition_result = ExpressionEvaluator::evaluate(&self.condition, &mut context)?;
        if let crate::core::Value::Bool(true) = condition_result {
            filtered_rows.push(row.clone());
        }
    }
    dataset.rows = filtered_rows;
    Ok(())
}
```

**简化点**：
- ❌ 缺少并行处理能力（`runMultiJobs`）
- ❌ 缺少批量处理优化
- ❌ 缺少内存检查机制
- ❌ 缺少数据移动优化（`movable`）

#### NebulaGraph 完整实现
```cpp
// nebula-3.8.0/src/graph/executor/query/FilterExecutor.cpp
folly::Future<Status> FilterExecutor::execute() {
    if (FLAGS_max_job_size == 1 || iter->isGetNeighborsIter()) {
        return handleSingleJobFilter();
    } else {
        // 多任务并行处理
        auto scatter = [this](size_t begin, size_t end, Iterator *tmpIter) {
            return handleJob(begin, end, tmpIter);
        };
        auto gather = [this, result = std::move(ds), kind = iter->kind()]
                      (std::vector<folly::Try<StatusOr<DataSet>>> &&results) {
            // 收集结果
        };
        return runMultiJobs(std::move(scatter), std::move(gather), iter.get());
    }
}
```

---

### 4. JoinExecutor 实现

#### GraphDB 当前实现
```rust
// src/query/executor/data_processing/join/base_join.rs
pub fn build_single_key_hash_table_with_evaluator<C: ExpressionContext>(
    &self,
    dataset: &DataSet,
    hash_key_expr: &Expression,
    _evaluator: &JoinKeyEvaluator,
    context: &mut C,
) -> Result<HashMap<Value, Vec<Vec<Value>>>, QueryError> {
    let mut hash_table = HashMap::new();
    
    for row in &dataset.rows {
        let key = JoinKeyEvaluator::evaluate_key(hash_key_expr, context)?;
        hash_table.entry(key).or_insert_with(Vec::new).push(row.clone());
    }
    Ok(hash_table)
}
```

**简化点**：
- ❌ 缺少并行处理支持
- ❌ 缺少左右输入交换优化
- ❌ 缺少数据移动优化（`movable` 检查）
- ❌ 缺少单键和多键的优化区分

#### NebulaGraph 完整实现
```cpp
// nebula-3.8.0/src/graph/executor/query/InnerJoinExecutor.h
// 支持单任务和多任务两种模式
folly::Future<Status> join(const std::vector<Expression*>& hashKeys,
                           const std::vector<Expression*>& probeKeys,
                           const std::vector<std::string>& colNames);

folly::Future<Status> joinMultiJobs(const std::vector<Expression*>& hashKeys,
                                    const std::vector<Expression*>& probeKeys,
                                    const std::vector<std::string>& colNames);

// 支持左右输入交换优化
bool exchange_{false};
bool mv_{false};  // 探测结果是否可移动
```

---

### 5. Executor Factory 实现

#### GraphDB 当前实现
```rust
// src/query/executor/factory.rs
pub fn create_executor(
    &self,
    plan_node: &PlanNodeEnum,
    storage: Arc<Mutex<S>>,
    _context: &ExecutionContext,
) -> Result<Box<dyn Executor<S>>, QueryError> {
    match plan_node {
        PlanNodeEnum::Start(node) => Ok(Box::new(StartExecutor::new(node.id()))),
        PlanNodeEnum::Filter(node) => Ok(Box::new(FilterExecutor::new(...))),
        // ... 直接匹配
        _ => Err(QueryError::ExecutionError(format!("暂不支持执行器类型: {:?}", plan_node.type_name()))),
    }
}
```

**简化点**：
- ❌ 缺少对象池管理
- ❌ 缺少统计信息收集
- ❌ 缺少递归构建执行器树的支持

#### NebulaGraph 完整实现
```cpp
// nebula-3.8.0/src/graph/executor/Executor.cpp
Executor *Executor::makeExecutor(QueryContext *qctx, const PlanNode *node) {
    auto pool = qctx->objPool();  // 使用对象池管理生命周期
    
    switch (node->kind()) {
        case PlanNode::Kind::kFilter:
            return pool->makeAndAdd<FilterExecutor>(node, qctx);
        case PlanNode::Kind::kAggregate:
            stats::StatsManager::addValue(kNumAggregateExecutors);
            return pool->makeAndAdd<AggregateExecutor>(node, qctx);
        // ... 统计 + 创建
    }
}
```

---

## 📊 功能对比表

| 功能 | GraphDB | NebulaGraph | 优先级 |
|------|---------|-------------|--------|
| 异步执行 | ✅ (async/await) | ✅ (folly::Future) | - |
| 并行处理 | ❌ | ✅ (runMultiJobs) | 高 |
| 性能统计 | ❌ | ✅ (SCOPED_TIMER) | 高 |
| 内存管理 | ❌ | ✅ (MemoryCheckGuard) | 高 |
| 执行器依赖 | ❌ | ✅ (depends/successors) | 中 |
| 对象池 | ❌ | ✅ (ObjectPool) | 低 |
| 采样支持 | ❌ | ✅ (ReservoirSampling) | 中 |
| 数据移动优化 | ❌ | ✅ (movable) | 中 |
| 多步扩展优化 | ❌ | ✅ | 中 |
| Join 左右交换 | ❌ | ✅ (exchange_) | 中 |

---

## 🔧 改进方案

### 优先级 1：添加性能统计和监控

**目标**：为所有执行器添加性能统计功能，用于监控和优化查询性能。

**实现**：
1. 在 `Executor` trait 中添加统计方法
2. 在 `BaseExecutor` 中添加统计字段
3. 在执行过程中记录关键指标

**预期收益**：
- 可以监控每个执行器的执行时间
- 可以统计处理的行数
- 可以识别性能瓶颈

---

### 优先级 2：添加并行处理支持

**目标**：为 Filter、Join 等执行器添加并行处理能力，提升大数据集处理性能。

**实现**：
1. 使用 `rayon` 或 `tokio` 实现并行处理
2. 实现批量处理机制
3. 添加并行度配置

**预期收益**：
- 大数据集处理性能提升 2-4 倍
- 更好地利用多核 CPU

---

### 优先级 3：优化 Join 执行器

**目标**：添加左右输入交换和数据移动优化。

**实现**：
1. 实现左右输入大小比较
2. 自动选择较小的表构建哈希表
3. 检查数据是否可以移动，避免不必要的拷贝

**预期收益**：
- 减少内存使用
- 提升 Join 性能 20-50%

---

### 优先级 4：优化 Expand 执行器

**目标**：添加多步扩展优化和采样支持。

**实现**：
1. 实现递归优化的多步扩展
2. 添加水库采样算法
3. 支持每步限制

**预期收益**：
- 提升图遍历性能
- 支持大规模图查询

---

### 优先级 5：添加内存管理

**目标**：防止内存溢出，提升系统稳定性。

**实现**：
1. 添加内存使用监控
2. 实现内存水位检查
3. 超过阈值时触发清理或报错

**预期收益**：
- 提升系统稳定性
- 防止 OOM 错误

---

## 📋 实施计划

### 第一阶段（1-2周）
1. ✅ 添加性能统计功能
2. ✅ 添加内存管理基础

### 第二阶段（2-3周）
1. ✅ 为 FilterExecutor 添加并行处理
2. ✅ 优化 JoinExecutor

### 第三阶段（1-2周）
1. ✅ 优化 ExpandExecutor
2. ✅ 添加采样支持

---

## 🎯 预期收益

1. **性能提升**：大数据集处理性能提升 2-4 倍
2. **稳定性**：防止内存溢出，提升系统稳定性
3. **可观测性**：完善的性能统计，便于监控和优化
4. **可扩展性**：为后续功能扩展打下基础

---

## ⚠️ 注意事项

1. **向后兼容性**：确保现有查询计划仍然可用
2. **测试覆盖**：添加全面的测试用例
3. **性能监控**：建立性能基准测试
4. **文档更新**：同步更新架构文档和 API 文档
