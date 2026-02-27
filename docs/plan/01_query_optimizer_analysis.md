# 查询优化器分析文档

## 概述

本文档分析 GraphDB 项目中查询优化器的当前实现状态，并提出可进一步改进的方向。

## 当前已实现的优化

### 1. 计划缓存（Plan Cache）

**实现位置**: `src/query/planner/planner.rs`

**功能说明**:
- 使用 LRU 缓存策略，默认缓存 1000 条查询计划
- 支持参数化查询缓存，将具体参数值替换为占位符
- 缓存键包含查询模板、图空间 ID、语句类型和模式指纹

**关键代码**:
```rust
pub struct QueryPlanner {
    plan_cache: Arc<Mutex<LruCache<PlanCacheKey, Arc<ExecutionPlan>>>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PlanCacheKey {
    query_template: String,
    space_id: Option<i32>,
    statement_type: SentenceKind,
    pattern_fingerprint: Option<String>,
}
```

**优化效果**: 避免重复生成相同查询的执行计划，减少解析和规划开销。

### 2. 计划重写（Plan Rewrite）

**实现位置**: `src/query/planner/rewrite/`

**功能说明**:
- 包含 20+ 条启发式优化规则
- 使用静态分发（枚举）替代动态分发，避免虚函数表查找开销
- 规则按顺序迭代应用，直到计划不再变化

**已实现的重写规则**:

| 规则类别 | 具体规则 | 说明 |
|---------|---------|------|
| 谓词下推 | `push_filter_down_scan_vertices` | 将过滤条件下推到顶点扫描 |
| 谓词下推 | `push_filter_down_get_nbrs` | 将过滤条件下推到邻居查询 |
| 谓词下推 | `push_filter_down_inner_join` | 将过滤条件下推到 Join 操作 |
| 投影下推 | `push_project_down` | 减少中间数据传输 |
| 操作合并 | `merge_get_nbrs_and_project` | 合并邻居查询和投影 |
| 操作合并 | `collapse_consecutive_project` | 合并连续投影 |
| LIMIT 下推 | `push_limit_down_index_scan` | 下推 LIMIT 到索引扫描 |
| LIMIT 下推 | `push_limit_down_scan_vertices` | 下推 LIMIT 到顶点扫描 |
| 冗余消除 | `eliminate_empty_set_operation` | 消除空集合操作 |
| 冗余消除 | `remove_noop_project` | 消除无操作投影 |
| 聚合优化 | `push_filter_down_aggregate` | 下推过滤到聚合前 |

**关键代码**:
```rust
pub struct PlanRewriter {
    rules: Vec<RewriteRuleEnum>, // 静态分发，无动态分发开销
    max_iterations: usize,
}

pub fn rewrite(&self, plan: ExecutionPlan) -> RewriteResult<ExecutionPlan> {
    // 迭代应用规则直到收敛
    for _ in 0..self.max_iterations {
        let new_plan = self.apply_rules(plan)?;
        if new_plan == plan { break; }
        plan = new_plan;
    }
    Ok(plan)
}
```

**性能优势**:
- 无动态分发开销
- 无堆分配
- 更好的缓存局部性
- 编译器可内联优化

### 3. 索引选择（Index Seek）

**实现位置**: `src/query/planner/statements/seeks/`

**功能说明**:
- 根据查询条件自动选择最优索引
- 支持标签索引和边类型索引
- 支持属性索引查找

**关键组件**:
- `IndexSeekPlanner`: 索引选择规划器
- `PropIndexSeek`: 属性索引查找
- `VariablePropIndexSeek`: 变量属性索引查找
- `ScanSeek`: 扫描策略（无索引时使用）

## 可进一步改进的方向

### 1. 代价模型（Cost-based Optimization）

**当前状态**: 仅使用启发式规则，无代价估算

**建议实现**:

```rust
/// 代价模型
pub struct CostModel {
    pub cpu_cost: f64,      // CPU 计算代价
    pub io_cost: f64,       // IO 操作代价
    pub memory_cost: f64,   // 内存使用代价
    pub network_cost: f64,  // 网络传输代价（分布式场景）
}

/// 表统计信息
pub struct TableStatistics {
    pub row_count: u64,
    pub avg_row_size: usize,
    pub distinct_values: HashMap<String, u64>,
    pub data_distribution: Histogram,
    pub null_fraction: f64,
}

/// 索引统计信息
pub struct IndexStatistics {
    pub index_name: String,
    pub selectivity: f64,           // 选择性（0-1）
    pub avg_entries_per_key: f64,
    pub index_size: usize,
    pub last_analyzed: SystemTime,
}
```

**应用场景**:
- 多索引选择时，选择代价最低的索引
- Join 策略选择（Hash Join vs Nested Loop Join）
- 扫描方式选择（全表扫描 vs 索引扫描）

**实现复杂度**: 高
**预期收益**: 中（复杂查询场景）

### 2. Join 优化

**当前状态**: 仅支持 Hash Join

**建议扩展**:

```rust
pub enum JoinStrategy {
    /// 哈希连接 - 适用于大数据量
    HashJoin {
        estimated_build_size: usize,
        probe_table: String,
    },
    /// 嵌套循环连接 - 适用于小表
    NestedLoop {
        outer_table: String,
        inner_table: String,
        outer_cardinality: u64,
    },
    /// 排序合并连接 - 适用于有序数据
    SortMergeJoin {
        sort_keys: Vec<String>,
    },
}
```

**选择策略**:
- 小表（< 1000 行）: Nested Loop Join
- 有序数据: Sort-Merge Join
- 大数据量: Hash Join

**实现复杂度**: 高
**预期收益**: 中（多表连接场景）

### 3. 自适应查询优化

**概念**: 运行时根据实际数据量动态调整执行策略

**建议实现**:

```rust
pub struct AdaptiveOptimizer {
    pub runtime_stats: RuntimeStatistics,
    pub reoptimization_threshold: f64, // 重新优化阈值
}

pub struct RuntimeStatistics {
    pub actual_rows: HashMap<PlanNodeId, u64>,
    pub execution_time_ms: HashMap<PlanNodeId, u64>,
}

impl AdaptiveOptimizer {
    /// 根据运行时统计调整计划
    pub fn adapt_plan(&self, plan: &mut ExecutionPlan) {
        for node in plan.nodes() {
            let estimated = node.estimated_rows();
            let actual = self.runtime_stats.actual_rows.get(&node.id());
            
            if let Some(actual_rows) = actual {
                let ratio = *actual_rows as f64 / estimated as f64;
                if ratio > self.reoptimization_threshold {
                    // 实际行数远超估计，可能需要调整策略
                    self.reoptimize_node(node);
                }
            }
        }
    }
}
```

**应用场景**:
- 数据分布不均匀时调整 Join 策略
- 根据实际选择性调整索引使用

**实现复杂度**: 高
**预期收益**: 中（数据分布不均匀场景）

### 4. 子查询优化

**当前状态**: 子查询通常转换为 Join 执行

**建议优化**:

```rust
pub enum SubqueryStrategy {
    /// 转换为 Join
    RewriteToJoin,
    /// 使用 Semi-Join（存在性查询）
    SemiJoin,
    /// 使用 Anti-Join（非存在性查询）
    AntiJoin,
    /// 物化子查询结果
    Materialize,
    /// 关联子查询去关联化
    Decorrelation,
}
```

**具体优化**:
- `EXISTS` 子查询 → Semi-Join
- `NOT EXISTS` 子查询 → Anti-Join
- 标量子查询 → 左外连接 + 聚合
- 关联子查询 → 去关联化

**实现复杂度**: 中
**预期收益**: 中（含子查询的场景）

### 5. 并行查询执行

**当前状态**: 单线程执行

**建议实现**:

```rust
pub struct ParallelExecutor {
    pub parallelism: usize,
    pub partition_strategy: PartitionStrategy,
}

pub enum PartitionStrategy {
    /// 哈希分区
    HashPartition { keys: Vec<String> },
    /// 范围分区
    RangePartition { ranges: Vec<Range<Value>> },
    /// 轮询分区
    RoundRobin,
}
```

**可并行化的操作**:
- 全表扫描（分区扫描）
- Hash Join（并行构建哈希表）
- 聚合（两阶段聚合）
- 排序（并行排序归并）

**实现复杂度**: 高
**预期收益**: 高（大数据量场景）

## 总结

### 已实现的优势

1. **完善的计划缓存机制** - 避免重复规划
2. **丰富的重写规则** - 20+ 条启发式优化
3. **静态分发设计** - 高性能规则引擎
4. **索引自动选择** - 减少全表扫描

### 建议优先级

| 优先级 | 优化项 | 预期收益 | 实现复杂度 |
|-------|--------|---------|-----------|
| P1 | 索引统计信息 | 高 | 中 |
| P2 | 子查询优化 | 中 | 中 |
| P3 | 代价模型 | 中 | 高 |
| P4 | Join 策略扩展 | 中 | 高 |
| P5 | 自适应优化 | 中 | 高 |
| P6 | 并行执行 | 高 | 高 |

当前优化器已经具备了良好的基础架构，建议优先完善统计信息收集机制，为后续代价模型奠定基础。
