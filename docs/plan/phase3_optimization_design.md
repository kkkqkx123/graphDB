# 第3阶段基于代价的优化任务可行性分析报告

## 概述

本文档基于对 `src/query/optimizer` 目录现有实现的深入分析，评估第3阶段三个高级优化任务的可行性：

1. **子查询去关联化优化器** (subquery_unnesting)
2. **物化策略选择器** (materialization_strategy)
3. **分区裁剪优化器** (partition_pruning)

**分析结论**：三个任务均**无法**仅基于现有实现给出精准计算或高度可靠的估算，建议**不引入**这些基于代价的策略，或采用更简单的启发式规则替代。

---

## 一、现有实现能力评估

### 1.1 统计信息体系

现有 `StatisticsManager` 提供以下统计信息：

| 统计类型 | 包含信息 | 适用场景 |
|---------|---------|---------|
| **TagStatistics** | 顶点数量、平均出度/入度、平均顶点大小 | 标签扫描代价估算、遍历代价估算 |
| **EdgeTypeStatistics** | 边数量、平均/最大出度/入度、唯一源/目标顶点数 | 边遍历代价估算、连接选择性估算 |
| **PropertyStatistics** | 不同值数量、空值比例、最小/最大值 | 等值条件选择性估算、范围查询选择性 |

**局限性**：
- 缺乏**直方图**统计，无法精准估算范围条件选择性
- 缺乏**相关性**统计，无法处理多列条件联合选择性
- 缺乏**数据分布**信息（如倾斜度、热点数据）
- 统计信息可能**过期**，没有自动更新机制

### 1.2 代价计算体系

`CostCalculator` 提供以下计算能力：

| 计算类型 | 精度 | 依赖 |
|---------|------|------|
| 扫描代价 | 中 | 行数统计 |
| 过滤代价 | 中 | 选择性估算 |
| 连接代价 | 中 | 行数、选择性 |
| 排序代价 | 高 | 行数、内存阈值 |
| 聚合代价 | 中 | 行数、分组键数量 |
| 图遍历代价 | 中 | 度数统计 |

**局限性**：
- 选择性估算依赖**启发式假设**（如连接选择性固定为0.3）
- 缺乏**实际执行反馈**（无法校准代价模型）
- 缓存感知计算基于**简化假设**（固定缓存命中率）

### 1.3 计划节点体系

现有节点支持：

- **控制流节点**：LoopNode、SelectNode、ArgumentNode、PassThroughNode
- **数据处理节点**：RollUpApplyNode、PatternApplyNode、UnionNode、UnwindNode、DedupNode
- **图遍历节点**：ExpandNode、ExpandAllNode、TraverseNode
- **连接节点**：HashInnerJoinNode、HashLeftJoinNode、InnerJoinNode、LeftJoinNode、CrossJoinNode

---

## 二、任务可行性详细分析

### 2.1 子查询去关联化优化器

#### 2.1.1 需要的信息

| 信息类型 | 用途 | 现有支持 |
|---------|------|---------|
| 子查询结果集大小 | 评估物化代价 | 部分支持（基于统计信息估算） |
| 子查询执行代价 | 比较嵌套循环 vs 连接 | 支持 |
| 子查询选择性 | 估算连接输出行数 | **不支持**（缺乏实际选择性统计） |
| 关联列分布 | 估算连接条件选择性 | **不支持** |
| 子查询是否含聚合 | 决定是否可转换 | 支持（检查 AggregateNode） |
| 子查询是否排序 | 评估排序消除收益 | 支持 |

#### 2.1.2 关键问题

**问题1：子查询选择性估算不可靠**

去关联化决策核心是比较：
```
cost_nested_loop = 左表行数 × (子查询启动代价 + 子查询执行代价)
cost_join = hash_join_cost(左表行数, 右表行数 × 选择性)
```

现有实现中，子查询选择性估算依赖启发式规则（如固定0.3），对于复杂子查询（含多表连接、聚合）误差可能达到**数量级**。

**问题2：无法准确估算子查询结果大小**

RollUpApplyNode 和 PatternApplyNode 的子查询可能包含复杂操作，现有估算器无法精准预测其输出行数。

**问题3：去关联化后计划质量不确定**

即使代价估算准确，去关联化引入的新连接可能：
- 破坏原有连接顺序优化
- 增加内存压力（哈希表构建）
- 改变数据流特性（流式→物化）

#### 2.1.3 结论

**不建议**实现基于代价的子查询去关联化优化器。

**替代方案**：在 `src/query/planner/rewrite` 目录实现**启发式规则**

```
规则名：UnnestSimpleSubqueryRule
适用条件：
1. 子查询是简单的单表扫描（无聚合、无连接）
2. 子查询只包含等值过滤条件
3. 子查询结果集估算 < 1000 行（基于统计信息）

转换：PatternApply → HashInnerJoin
```

---

### 2.2 物化策略选择器

#### 2.2.1 需要的信息

| 信息类型 | 用途 | 现有支持 |
|---------|------|---------|
| 子计划被引用次数 | 识别物化机会 | **不支持**（需引用计数分析） |
| 子计划结果集大小 | 评估存储代价 | 部分支持 |
| 子计划执行代价 | 比较重复执行 vs 物化 | 支持 |
| 内存可用容量 | 决定内存/磁盘物化 | **不支持**（缺乏运行时信息） |
| 数据访问模式 | 随机访问 vs 顺序访问 | **不支持** |
| 子计划是否确定性 | 决定是否可以物化 | **不支持**（需分析表达式） |

#### 2.2.2 关键问题

**问题1：无法准确识别重复子表达式**

现有计划树没有**引用计数**机制，无法区分：
- 同一子计划被多次引用（适合物化）
- 逻辑相同但物理不同的子计划（不适合物化）

**问题2：缺乏运行时信息**

物化决策需要：
- 当前可用内存
- 并发查询数量
- 缓存状态

这些信息在计划生成阶段**无法获取**。

**问题3：物化收益高度依赖执行环境**

同一查询在不同环境下物化策略可能完全不同：
- 内存充足时：内存物化收益高
- 内存紧张时：物化可能导致OOM
- 高并发时：物化减少重复计算，但增加内存压力

#### 2.2.3 结论

**不建议**实现基于代价的物化策略选择器。

**替代方案**：在 `src/query/planner/rewrite` 目录实现**启发式规则**

```
规则名：MaterializeCTERule
适用条件：
1. CTE 被引用次数 > 1
2. CTE 不包含非确定性函数（如 rand(), now()）
3. CTE 结果集估算 < 10000 行

转换：在 CTE 子计划根节点添加 MaterializeNode
```

---

### 2.3 分区裁剪优化器

#### 2.3.1 需要的信息

| 信息类型 | 用途 | 现有支持 |
|---------|------|---------|
| 分区元数据 | 了解分区键和范围 | **不支持**（无分区概念） |
| 分区统计信息 | 评估每个分区的选择性 | **不支持** |
| 谓词与分区键关系 | 确定可裁剪的分区 | **不支持** |
| 分区存储位置 | 生成分区扫描计划 | **不支持** |

#### 2.3.2 关键问题

**问题1：存储层无分区支持**

现有存储层（`src/storage`）基于 Redb，采用简单的键值存储，没有分区概念：
- 无分区元数据管理
- 无分区裁剪接口
- 无分区并行扫描能力

**问题2：分区策略不确定**

图数据库的分区策略多样：
- 按标签分区
- 按时间分区
- 按哈希分区
- 混合分区

不同策略需要不同的裁剪算法，现有代码无法统一支持。

**问题3：分区裁剪收益有限**

对于单节点图数据库：
- 数据量通常不大（百万级顶点）
- 全表扫描性能可接受
- 分区管理增加复杂度

#### 2.3.3 结论

**不建议**实现分区裁剪优化器。

**替代方案**：在 `src/query/planner/rewrite` 目录实现**标签裁剪启发式规则**

```
规则名：PruneTagScanRule
适用条件：
1. ScanVertices 节点无标签过滤
2. 查询中其他位置有明确的标签过滤条件

转换：将标签条件下推到 ScanVertices
```

**注意**：此规则已部分实现（谓词下推），可扩展以支持更激进的标签裁剪。

---

## 三、建议的替代方案

### 3.1 新增启发式规则（rewrite目录）

基于以上分析，建议在 `src/query/planner/rewrite` 目录新增以下启发式规则：

#### 3.1.1 简单子查询去关联化规则

```
文件：src/query/planner/rewrite/subquery_unnesting/simple_unnest.rs

规则：UnnestSimpleExistentialSubquery
适用条件：
1. PatternApplyNode 的右输入是简单查询（单表扫描 + 过滤）
2. 过滤条件只包含等值比较
3. 子查询估算行数 < 1000

转换：
PatternApply(is_anti_predicate=false)
├── 左输入
└── Filter(条件) → Scan(表)

转换为：
HashInnerJoin(条件)
├── 左输入
└── Scan(表)
```

#### 3.1.2 CTE物化规则

```
文件：src/query/planner/rewrite/materialization/cte_materialize.rs

规则：MaterializeSmallCTE
适用条件：
1. UnionNode 的 distinct = true（表示是 CTE）
2. CTE 被引用次数 > 1（通过引用分析）
3. CTE 估算行数 < 10000
4. CTE 不包含非确定性函数

转换：在 CTE 子计划根节点包装 MaterializeNode
```

#### 3.1.3 标签扫描裁剪规则

```
文件：src/query/planner/rewrite/partition_pruning/tag_pruning.rs

规则：PruneUnusedTagScans
适用条件：
1. 查询包含多个标签的扫描
2. 某些标签的扫描结果在后续操作中被过滤掉
3. 过滤条件可以在扫描前应用

转换：将过滤条件下推到 ScanVertices，可能消除某些扫描
```

### 3.2 增强现有统计信息

在不引入复杂代价优化的前提下，可以增强统计信息以支持更精准的启发式规则：

```rust
// 在 TagStatistics 中增加
pub struct TagStatistics {
    // 现有字段...
    
    /// 常用属性选择性（预计算）
    pub common_property_selectivity: HashMap<String, f64>,
    /// 数据更新频率（用于判断统计信息新鲜度）
    pub update_frequency: UpdateFrequency,
    /// 数据分布类型（均匀、倾斜、热点）
    pub distribution_type: DistributionType,
}
```

### 3.3 简化代价模型

如果确实需要代价辅助决策，可以实现**简化代价模型**：

```rust
/// 简化代价比较（仅用于启发式规则）
pub enum CostComparison {
    /// 明显更优（代价差 > 10倍）
    SignificantlyBetter,
    /// 略微更优（代价差 2-10倍）
    SlightlyBetter,
    /// 相当（代价差 < 2倍）
    Comparable,
    /// 不确定（缺乏统计信息）
    Unknown,
}

/// 仅在代价差异显著时才做决策
pub fn should_apply_optimization(
    baseline_cost: f64,
    optimized_cost: f64,
) -> CostComparison {
    if baseline_cost / optimized_cost > 10.0 {
        CostComparison::SignificantlyBetter
    } else if baseline_cost / optimized_cost > 2.0 {
        CostComparison::SlightlyBetter
    } else if baseline_cost / optimized_cost > 1.5 {
        CostComparison::Comparable
    } else {
        CostComparison::Unknown
    }
}
```

---

## 四、总结

### 4.1 三个任务的最终建议

| 任务 | 建议 | 理由 |
|------|------|------|
| **子查询去关联化** | ❌ 不实现基于代价的版本 | 子查询选择性估算不可靠，决策风险高 |
| **物化策略选择** | ❌ 不实现基于代价的版本 | 缺乏运行时信息，无法做可靠决策 |
| **分区裁剪** | ❌ 不实现 | 存储层无分区支持，收益有限 |

### 4.2 替代方案建议

| 替代方案 | 位置 | 预期收益 |
|---------|------|---------|
| 简单子查询去关联化（启发式） | `rewrite/subquery_unnesting` | 处理常见简单子查询 |
| CTE物化（启发式） | `rewrite/materialization` | 减少重复计算 |
| 标签扫描裁剪（启发式） | `rewrite/partition_pruning` | 减少扫描数据量 |

### 4.3 与现有架构的集成

```
src/query/planner/rewrite/
├── mod.rs                          # 现有
├── ...                             # 现有规则
├── subquery_unnesting/             # 新增
│   ├── mod.rs
│   └── simple_unnest.rs            # 简单子查询去关联化
├── materialization/                # 新增
│   ├── mod.rs
│   └── cte_materialize.rs          # CTE物化
└── partition_pruning/              # 新增
    ├── mod.rs
    └── tag_pruning.rs              # 标签扫描裁剪
```

### 4.4 关键原则

1. **保守优先**：只在代价差异显著（>10倍）时才做决策
2. **可回退**：所有优化都保留原始计划作为备选
3. **简单可靠**：优先实现启发式规则，避免复杂代价计算
4. **渐进增强**：先实现基础版本，根据实际效果逐步增强

---

## 附录：现有代码文件索引

### 优化器核心
- `src/query/optimizer/mod.rs` - 优化器模块入口
- `src/query/optimizer/engine.rs` - 优化器引擎
- `src/query/optimizer/cost/calculator.rs` - 代价计算器
- `src/query/optimizer/cost/config.rs` - 代价模型配置

### 统计信息
- `src/query/optimizer/stats/manager.rs` - 统计信息管理器
- `src/query/optimizer/stats/tag.rs` - 标签统计
- `src/query/optimizer/stats/edge.rs` - 边类型统计
- `src/query/optimizer/stats/property.rs` - 属性统计

### 重写规则
- `src/query/planner/rewrite/mod.rs` - 重写模块入口
- `src/query/planner/rewrite/plan_rewriter.rs` - 计划重写器
- `src/query/planner/rewrite/rule.rs` - 规则trait定义

### 计划节点
- `src/query/planner/plan/core/nodes/plan_node_enum.rs` - 计划节点枚举
- `src/query/planner/plan/core/nodes/data_processing_node.rs` - 数据处理节点（含RollUpApply、PatternApply）
- `src/query/planner/plan/core/nodes/control_flow_node.rs` - 控制流节点
