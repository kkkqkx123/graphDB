# GraphDB 代价模型集成分析报告

## 一、架构现状分析

### 1.1 当前架构确认

经过代码审查，确认当前 GraphDB 的架构如下：

```
┌─────────────────────────────────────────────────────────────┐
│                    Query Planning Pipeline                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. Parser → AST                                           │
│                                                             │
│  2. Planner → Initial Plan (逻辑计划)                       │
│     └─ 选择访问路径（ScanVertices, IndexScan 等）            │
│                                                             │
│  3. Rewrite → Optimized Plan (逻辑计划)                     │
│     ├─ 谓词下推（规则驱动）                                 │
│     ├─ 投影下推（规则驱动）                                 │
│     ├─ 消除冗余节点（规则驱动）                             │
│     ├─ 合并操作（规则驱动）                                 │
│     └─ Limit 下推（规则驱动）                               │
│                                                             │
│  4. Optimizer → Physical Plan (物理计划)                    │
│     └─ 基于代价的优化（待集成）                             │
│                                                             │
│  5. Executor → Execute                                     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 Planner 模块确认

**关键发现：Planner 模块已经完全是规则驱动的，没有使用 optimizer 模块**

检查结果：
- `src/query/planner/` 模块中没有任何对 `optimizer` 模块的引用
- Planner 模块未导入：`CostCalculator`、`StatisticsManager`、`SelectivityEstimator` 等
- Rewrite 规则完全基于启发式，不依赖代价计算

### 1.3 PlanNode 中的 Cost 定义

PlanNode 保留了对 cost 的基础定义（用于存储计算后的代价）：

```rust
// src/query/planner/plan/core/nodes/plan_node_traits.rs

pub trait PlanNode {
    /// 获取节点的成本估计值
    /// 返回 None 表示代价未计算，Some(cost) 表示已计算
    fn cost(&self) -> Option<f64>;
    // ...
}
```

这个设计是正确的：
- PlanNode 只定义代价的**存储能力**
- 实际代价计算由 Optimizer 层的 `CostAssigner` 负责
- Planner 不使用代价，只生成初始计划

---

## 二、当前代价体系集成状态

### 2.1 已实现的模块

#### 统计信息模块 (`src/query/optimizer/stats/`)

| 文件 | 状态 | 说明 |
|------|------|------|
| `manager.rs` | ✅ 已实现 | StatisticsManager 统一管理统计信息 |
| `collector.rs` | ✅ 已实现 | StatisticsCollector 从存储引擎收集统计信息 |
| `tag.rs` | ✅ 已实现 | TagStatistics 标签级别统计 |
| `edge.rs` | ✅ 已实现 | EdgeTypeStatistics 边类型统计 |
| `property.rs` | ✅ 已实现 | PropertyStatistics 属性统计 |

#### 代价计算模块 (`src/query/optimizer/cost/`)

| 文件 | 状态 | 说明 |
|------|------|------|
| `calculator.rs` | ✅ 已实现 | CostCalculator 完整的代价计算方法 |
| `selectivity.rs` | ✅ 已实现 | SelectivityEstimator 选择性估计 |
| `config.rs` | ✅ 已实现 | CostModelConfig 可配置的代价参数 |
| `assigner.rs` | ✅ 已实现 | CostAssigner 为计划节点赋值代价 |

#### 优化策略模块 (`src/query/optimizer/strategy/`)

| 文件 | 状态 | 说明 |
|------|------|------|
| `traversal_start.rs` | ✅ 已实现 | TraversalStartSelector 遍历起点选择 |
| `index.rs` | ✅ 已实现 | IndexSelector 索引选择策略 |

#### ANALYZE 命令 (`src/query/executor/admin/`)

| 文件 | 状态 | 说明 |
|------|------|------|
| `analyze.rs` | ✅ 已实现 | AnalyzeExecutor 统计信息收集 |

### 2.2 架构设计确认

当前架构符合业界最佳实践：

| 层次 | 职责 | 是否使用代价 |
|------|------|-------------|
| **Parser** | 解析查询 | ❌ |
| **Planner** | 生成初始计划 | ❌ |
| **Rewrite** | 启发式规则优化 | ❌ **保持纯粹** |
| **Optimizer** | 基于代价的优化 | ✅ **在此使用** |
| **Executor** | 执行计划 | ❌ |

---

## 三、应通过基于代价的优化策略来优化的操作

### 3.1 高优先级优化

#### 3.1.1 索引选择（Index Selection）

**当前实现：** LookupPlanner 使用简单启发式

```rust
// 当前实现：lookup_planner.rs
// 简单启发式：选择第一个可用索引
let index = available_indexes.first().cloned();
```

**应优化为：** 使用 IndexSelector 基于代价选择

| 场景 | 当前策略 | 推荐策略 |
|------|---------|---------|
| 有多个可用索引 | 选择第一个 | 基于选择性选择最优索引 |
| 索引 vs 全表扫描 | 总是全表扫描 | 基于代价选择 |

**相关文件：**
- [lookup_planner.rs](file:///d:/项目/database/graphDB/src/query/planner/statements/lookup_planner.rs)
- [index.rs](file:///d:/项目/database/graphDB/src/query/optimizer/strategy/index.rs)

#### 3.1.2 遍历起点选择（Traversal Start Selection）

**当前实现：** MatchStatementPlanner 固定遍历顺序

**应优化为：** 使用 TraversalStartSelector 选择代价最小的起点

| 场景 | 当前策略 | 推荐策略 |
|------|---------|---------|
| 多节点路径模式 | 固定顺序 | 基于节点选择性选择起点 |
| 边类型过滤 | 不考虑 | 基于边类型统计选择 |

**相关文件：**
- [match_statement_planner.rs](file:///d:/项目/database/graphDB/src/query/planner/statements/match_statement_planner.rs)
- [traversal_start.rs](file:///d:/项目/database/graphDB/src/query/optimizer/strategy/traversal_start.rs)

#### 3.1.3 连接算法选择（Join Algorithm Selection）

**当前实现：** 固定使用哈希连接

**应优化为：** 基于左右表数据量选择连接算法

| 场景 | 当前策略 | 推荐策略 |
|------|---------|---------|
| 小表 JOIN 大表 | 哈希连接 | 嵌套循环连接可能更优 |
| 大表 JOIN 大表 | 哈希连接 | 哈希连接或排序合并连接 |

**相关代价计算：**
- [calculator.rs - calculate_hash_join_cost](file:///d:/项目/database/graphDB/src/query/optimizer/cost/calculator.rs)
- [calculator.rs - calculate_nested_loop_join_cost](file:///d:/项目/database/graphDB/src/query/optimizer/cost/calculator.rs)

### 3.2 中优先级优化

#### 3.2.1 路径算法选择

**相关文件：** [path_planner.rs](file:///d:/项目/database/graphDB/src/query/planner/statements/path_planner.rs)

| 场景 | 当前策略 | 推荐策略 |
|------|---------|---------|
| 稀疏图最短路径 | BFS | BFS |
| 带权最短路径 | BFS | Dijkstra 或 A* |
| 全路径查询 | 全部展开 | 基于数据量选择展开深度 |

#### 3.2.2 聚合策略选择

**相关文件：** [group_by_planner.rs](file:///d:/项目/database/graphDB/src/query/planner/statements/group_by_planner.rs)

| 场景 | 当前策略 | 推荐策略 |
|------|---------|---------|
| 小数据量聚合 | 哈希聚合 | 流式聚合或哈希聚合 |
| 大数据量聚合 | 哈希聚合 | 基于数据量选择 |

#### 3.2.3 子图扩展策略

**相关文件：** [subgraph_planner.rs](file:///d:/项目/database/graphDB/src/query/planner/statements/subgraph_planner.rs)

| 场景 | 当前策略 | 推荐策略 |
|------|---------|---------|
| 多步扩展 | 顺序扩展 | 基于边类型选择性优化顺序 |
| 零步扩展 | 不优化 | 直接返回起始顶点 |

### 3.3 低优先级优化

| 操作 | 优化策略 |
|------|---------|
| 集合操作 (UNION/INTERSECT/MINUS) | 基于数据量选择实现方式 |
| 去重策略 | 基于选择性选择哈希或排序去重 |
| 排序策略 | 基于数据量选择内存排序或外部排序 |

---

## 四、代价优化器层设计建议

### 4.1 建议新增模块

```
src/query/
├── optimizer/
│   ├── mod.rs
│   ├── stats/                    # 现有：统计信息模块
│   │   ├── mod.rs
│   │   ├── manager.rs
│   │   ├── collector.rs
│   │   ├── tag.rs
│   │   ├── edge.rs
│   │   └── property.rs
│   ├── cost/                     # 现有：代价计算模块
│   │   ├── mod.rs
│   │   ├── calculator.rs
│   │   ├── selectivity.rs
│   │   ├── config.rs
│   │   └── assigner.rs
│   ├── strategy/                 # 现有：优化策略模块
│   │   ├── mod.rs
│   │   ├── traversal_start.rs
│   │   └── index.rs
│   └── cost_based/               # 新增：基于代价的优化器
│       ├── mod.rs
│       ├── optimizer.rs           # 主优化器
│       ├── index_optimizer.rs     # 索引优化
│       ├── join_optimizer.rs      # 连接优化
│       └── traversal_optimizer.rs # 遍历优化
```

### 4.2 优化器接口设计

```rust
// src/query/optimizer/cost_based/optimizer.rs

use crate::query::optimizer::stats::StatisticsManager;
use crate::query::optimizer::cost::{CostCalculator, SelectivityEstimator, CostAssigner};
use crate::query::optimizer::strategy::{TraversalStartSelector, IndexSelector};
use crate::query::planner::plan::ExecutionPlan;

/// 基于代价的优化器
///
/// 在 Rewrite 之后应用，负责需要代价判断的优化决策
pub struct CostBasedOptimizer {
    stats_manager: Arc<StatisticsManager>,
    cost_calculator: Arc<CostCalculator>,
    selectivity_estimator: Arc<SelectivityEstimator>,
    index_selector: Arc<IndexSelector>,
    traversal_start_selector: Arc<TraversalStartSelector>,
    cost_assigner: CostAssigner,
}

impl CostBasedOptimizer {
    pub fn new(stats_manager: Arc<StatisticsManager>) -> Self {
        // 初始化所有组件
    }

    /// 优化执行计划
    pub fn optimize(&self, plan: ExecutionPlan) -> Result<ExecutionPlan, OptimizerError> {
        // 1. 索引选择优化
        let plan = self.optimize_index_selection(plan)?;
        
        // 2. 遍历起点优化
        let plan = self.optimize_traversal_start(plan)?;
        
        // 3. 连接算法优化
        let plan = self.optimize_join_algorithm(plan)?;
        
        // 4. 代价赋值（用于 EXPLAIN）
        self.assign_costs(plan)
    }
}
```

---

## 五、实施优先级建议

### 第一阶段：核心优化（高优先级）

1. **索引选择优化**
   - 在 LookupPlanner 中集成 IndexSelector
   - 实现自动选择最优索引

2. **遍历起点优化**
   - 在 MatchStatementPlanner 中集成 TraversalStartSelector
   - 实现基于统计信息选择最优遍历起点

3. **连接算法优化**
   - 实现哈希连接 vs 嵌套循环连接的选择

### 第二阶段：扩展优化（中优先级）

4. **路径算法优化**
5. **聚合策略优化**
6. **子图扩展优化**

### 第三阶段：高级优化（低优先级）

7. **计划枚举和选择**
8. **代价感知的计划缓存**
9. **动态统计信息更新**

---

## 六、总结

### 架构确认

- ✅ Planner 模块已经是纯粹规则驱动的，没有使用 optimizer
- ✅ PlanNode 中保留了 cost 定义，用于存储计算后的代价
- ✅ Optimizer 模块已完整实现，可以进行集成

### 核心原则

1. **Rewrite 层保持纯粹规则驱动**
   - 规则总是产生更优或等价的计划
   - 不需要代价判断
   - 专注于逻辑优化

2. **Optimizer 层负责基于代价的优化**
   - 需要权衡不同策略的代价
   - 使用统计信息
   - 专注于物理优化

### 待集成操作

| 优先级 | 操作 | 当前状态 | 目标状态 |
|--------|------|---------|---------|
| 高 | 索引选择 | 简单启发式 | 基于代价选择 |
| 高 | 遍历起点选择 | 固定顺序 | 基于代价选择 |
| 高 | 连接算法选择 | 固定哈希连接 | 基于代价选择 |
| 中 | 路径算法选择 | 固定BFS | 基于图特征选择 |
| 中 | 聚合策略选择 | 固定策略 | 基于数据量选择 |
| 低 | 集合操作实现 | 固定实现 | 基于数据量选择 |

---

*文档更新时间：2026-02-27*
