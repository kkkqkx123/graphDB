# 基于代价的优化规则分析报告

## 概述

本文档分析 `src\query\planner\rewrite` 目录中已实现的启发式优化规则，并与基于代价的优化建议进行对比，筛选出不重复的、值得实现的基于代价的优化规则。

---

## 一、已实现的启发式优化规则

### 1. 谓词下推规则 (predicate_pushdown)

**已实现的规则：**

| 规则名称 | 说明 |
|---------|------|
| `PushFilterDownTraverseRule` | 将过滤条件下推到 Traverse 节点 |
| `PushFilterDownExpandAllRule` | 将过滤条件下推到 ExpandAll 节点 |
| `PushFilterDownNodeRule` | 将过滤条件下推到 Node 操作 |
| `PushEFilterDownRule` | 将边过滤条件下推 |
| `PushVFilterDownScanVerticesRule` | 将顶点过滤条件下推到 ScanVertices |
| `PushFilterDownInnerJoinRule` | 将过滤条件下推到 InnerJoin |
| `PushFilterDownHashInnerJoinRule` | 将过滤条件下推到 HashInnerJoin |
| `PushFilterDownHashLeftJoinRule` | 将过滤条件下推到 HashLeftJoin |
| `PushFilterDownCrossJoinRule` | 将过滤条件下推到 CrossJoin |
| `PushFilterDownGetNbrsRule` | 将过滤条件下推到 GetNeighbors |
| `PushFilterDownAllPathsRule` | 将过滤条件下推到 AllPaths |

**特点：** 这些规则是**无条件**应用的启发式规则，只要满足模式匹配就进行下推，不考虑数据分布和代价。

---

### 2. LIMIT 下推规则 (limit_pushdown)

**已实现的规则：**

| 规则名称 | 说明 |
|---------|------|
| `PushLimitDownGetVerticesRule` | 将 LIMIT 下推到 GetVertices |
| `PushLimitDownGetEdgesRule` | 将 LIMIT 下推到 GetEdges |
| `PushLimitDownScanVerticesRule` | 将 LIMIT 下推到 ScanVertices |
| `PushLimitDownScanEdgesRule` | 将 LIMIT 下推到 ScanEdges |
| `PushLimitDownIndexScanRule` | 将 LIMIT 下推到 IndexScan |
| `PushTopNDownIndexScanRule` | 将 TopN 下推到 IndexScan |

**特点：** 同样是**无条件**应用的启发式规则。

---

### 3. 操作合并规则 (merge)

**已实现的规则：**

| 规则名称 | 说明 |
|---------|------|
| `CombineFilterRule` | 合并连续的 Filter 节点 |
| `CollapseProjectRule` | 合并 Project 节点 |
| `CollapseConsecutiveProjectRule` | 合并连续的 Project 节点 |
| `MergeGetVerticesAndProjectRule` | 合并 GetVertices 和 Project |
| `MergeGetVerticesAndDedupRule` | 合并 GetVertices 和 Dedup |
| `MergeGetNbrsAndProjectRule` | 合并 GetNeighbors 和 Project |
| `MergeGetNbrsAndDedupRule` | 合并 GetNeighbors 和 Dedup |

**特点：** 消除冗余操作，减少中间结果。

---

### 4. 消除规则 (elimination)

**已实现的规则：**

| 规则名称 | 说明 |
|---------|------|
| `EliminateFilterRule` | 消除永真/永假过滤条件 |
| `RemoveNoopProjectRule` | 消除无操作投影 |
| `EliminateAppendVerticesRule` | 消除冗余 AppendVertices |
| `RemoveAppendVerticesBelowJoinRule` | 消除 Join 下的 AppendVertices |
| `EliminateRowCollectRule` | 消除 RowCollect |
| `EliminateEmptySetOperationRule` | 消除空集合操作 |
| `DedupEliminationRule` | 消除不必要的去重 |

**特点：** 识别并移除无效或冗余操作。

---

### 5. 投影下推规则 (projection_pushdown)

**已实现的规则：**

| 规则名称 | 说明 |
|---------|------|
| `ProjectionPushDownRule` | 投影下推主规则 |
| `PushProjectDownRule` | 将投影推向数据源 |

---

### 6. 聚合优化规则 (aggregate)

**已实现的规则：**

| 规则名称 | 说明 |
|---------|------|
| `PushFilterDownAggregateRule` | 将过滤条件下推到聚合操作 |

---

## 二、建议的基于代价的优化规则 vs 已实现规则对比

### 对比分析表

| 建议的代价优化规则 | 依赖的估算器 | 与启发式规则的关系 | 是否重复 | 建议 |
|-------------------|-------------|-------------------|---------|------|
| **谓词下推优化器** | DataProcessingEstimator | rewrite 已实现无条件下推 | **部分重复** | 保留启发式版本，**增加选择性评估**来决定是否下推 |
| **Limit 下推优化器** | SortLimitEstimator | rewrite 已实现无条件下推 | **部分重复** | 保留启发式版本，**增加代价评估**来决定下推策略 |
| **聚合策略选择器** | SortLimitEstimator | rewrite 只有简单下推 | **不重复** | **建议实现**：选择哈希聚合 vs 排序聚合 |
| **连接顺序优化器** | JoinEstimator | rewrite 无连接重排序 | **不重复** | **建议实现**：多表连接顺序优化 |
| **图遍历方向优化器** | GraphTraversalEstimator | rewrite 无方向优化 | **不重复** | **建议实现**：基于度数选择遍历方向 |
| **排序消除优化器** | SortLimitEstimator | rewrite 无排序消除 | **不重复** | **建议实现**：利用索引顺序消除排序 |
| **子查询去关联化** | ControlFlowEstimator + JoinEstimator | rewrite 无子查询优化 | **不重复** | **建议实现**：相关子查询转连接 |
| **物化策略选择器** | CostCalculator | rewrite 无物化决策 | **不重复** | **建议实现**：决定是否物化中间结果 |
| **分区裁剪优化器** | SelectivityEstimator | rewrite 无分区优化 | **不重复** | **建议实现**：基于谓词裁剪分区 |
| **缓存感知优化器** | CostCalculator | rewrite 无缓存感知 | **不重复** | **建议实现**：根据缓存状态调整策略 |

---

## 三、不重复的基于代价的优化规则（推荐实现）

### 高优先级（建议立即实现）

#### 1. 聚合策略选择器 (`aggregate_strategy.rs`)

**背景：** rewrite 模块只有简单的聚合下推，没有聚合算法选择。

**功能：**
- 在哈希聚合和排序聚合之间选择
- 基于输入行数、内存限制、分组键数量进行代价比较

**依赖：**
- `SortLimitEstimator` 中的聚合代价计算
- `CostCalculator.calculate_aggregate_cost()`
- `CostCalculator.calculate_sort_cost()`

**实现要点：**
```rust
pub enum AggregateStrategy {
    HashAggregate,    // 使用 HashMap
    SortAggregate,    // 先排序再聚合
    StreamingAggregate, // 流式聚合（输入已排序）
}
```

---

#### 2. 连接顺序优化器 (`join_order.rs`)

**背景：** rewrite 模块实现了连接条件下的谓词下推，但没有连接顺序优化。

**功能：**
- 为多个表的连接选择最优顺序
- 支持左深树和浓密树计划
- 使用动态规划或贪心算法

**依赖：**
- `JoinEstimator` 的连接代价计算
- `CostCalculator.calculate_hash_join_cost()`
- `CostCalculator.calculate_nested_loop_join_cost()`

**实现要点：**
```rust
pub struct JoinOrderOptimizer {
    cost_calculator: Arc<CostCalculator>,
}

impl JoinOrderOptimizer {
    pub fn optimize_join_order(
        &self,
        tables: Vec<TableInfo>,
        join_conditions: Vec<JoinCondition>,
    ) -> JoinOrder {
        // 使用动态规划选择最优连接顺序
    }
}
```

---

#### 3. 图遍历方向优化器 (`traversal_direction.rs`)

**背景：** rewrite 模块没有图遍历方向优化。

**功能：**
- 基于边的出度/入度统计选择遍历方向
- 选择度数较小的方向以减少中间结果

**依赖：**
- `GraphTraversalEstimator` 的度数统计
- `CostCalculator.statistics_manager().get_edge_stats()`

**实现要点：**
```rust
pub enum TraversalDirection {
    Forward,  // 出边方向
    Backward, // 入边方向
}

impl TraversalDirectionOptimizer {
    pub fn optimize_direction(
        &self,
        edge_type: &str,
    ) -> TraversalDirection {
        // 比较 avg_out_degree 和 avg_in_degree
    }
}
```

---

### 中优先级（建议后续实现）

#### 4. 排序消除优化器 (`sort_elimination.rs`)

**背景：** rewrite 模块没有排序消除优化。

**功能：**
- 检查是否可以利用索引顺序避免排序
- 合并相邻的排序操作
- 将 Sort + Limit 转换为 TopN

**依赖：**
- `SortLimitEstimator` 的排序代价计算
- 索引元数据（索引是否有序）

---

#### 5. 子查询去关联化优化器 (`subquery_unnesting.rs`)

**背景：** rewrite 模块没有子查询优化。

**功能：**
- 将相关子查询转换为连接
- 基于代价评估去关联化的收益

**依赖：**
- `ControlFlowEstimator` 的循环代价
- `JoinEstimator` 的连接代价

---

### 低优先级（可选实现）

#### 6. 物化策略选择器 (`materialization_strategy.rs`)

**功能：** 决定是否物化中间结果以支持复用。

#### 7. 分区裁剪优化器 (`partition_pruning.rs`)

**功能：** 基于查询条件裁剪不需要的数据分区。

#### 8. 缓存感知优化器 (`cache_aware_optimizer.rs`)

**功能：** 根据缓存状态选择扫描策略（顺序/随机）。

---

## 四、与启发式规则协作的建议

### 优化器架构建议

```
查询计划优化流程：

1. 启发式重写阶段 (Plan Rewriter)
   ├── 谓词下推 (无条件)
   ├── LIMIT 下推 (无条件)
   ├── 操作合并
   ├── 冗余消除
   └── 投影下推

2. 基于代价的优化阶段 (Cost-Based Optimizer)
   ├── 连接顺序优化
   ├── 聚合策略选择
   ├── 遍历方向选择
   ├── 排序消除
   └── 子查询去关联化

3. 物理计划生成阶段
   └── 索引选择、访问路径选择
```

### 关键原则

1. **启发式规则优先**：先应用启发式规则进行基本优化
2. **代价优化补充**：在启发式规则基础上，使用代价模型进行决策
3. **避免重复**：不重复实现启发式规则已有的功能
4. **协作而非替代**：代价优化与启发式优化是互补关系

---

## 五、实现路线图

### 第一阶段（核心功能）

1. **聚合策略选择器**
   - 实现哈希聚合 vs 排序聚合的选择逻辑
   - 集成到 `src\query\optimizer\strategy`

2. **连接顺序优化器**
   - 实现动态规划算法
   - 支持左深树和浓密树

### 第二阶段（图数据库特色）

3. **图遍历方向优化器**
   - 利用边统计信息
   - 集成到遍历起点选择器

4. **排序消除优化器**
   - 利用索引有序性
   - Sort + Limit → TopN 转换

### 第三阶段（高级功能）

5. **子查询去关联化**
6. **物化策略选择器**
7. **分区裁剪优化器**

---

## 六、总结

### 不重复的基于代价的优化规则（共 8 个）

| 优先级 | 优化规则 | 实现复杂度 | 预期收益 |
|--------|----------|-----------|---------|
| 高 | 聚合策略选择器 | 中 | 高 |
| 高 | 连接顺序优化器 | 高 | 高 |
| 高 | 图遍历方向优化器 | 低 | 中 |
| 中 | 排序消除优化器 | 中 | 中 |
| 中 | 子查询去关联化 | 高 | 高 |
| 低 | 物化策略选择器 | 中 | 中 |
| 低 | 分区裁剪优化器 | 中 | 中 |
| 低 | 缓存感知优化器 | 低 | 低 |

### 与启发式规则的关系

- **谓词下推**和**LIMIT 下推**已在 rewrite 模块通过启发式规则实现，strategy 模块不需要重复实现
- strategy 模块应专注于**需要代价决策**的优化场景
- 两个模块是**协作关系**，而非替代关系
