# GraphDB 代价模型集成分析文档

## 概述

本文档基于 `cost_model_refined_plan.md` 和 `cost_model_introduction_analysis.md`，详细分析 GraphDB 当前代价体系的正式集成方案，以及如何为每个操作类型提供代价计算。

---

## 一、现有代价体系架构分析

### 1.1 当前架构概览

| 组件 | 位置 | 状态 |
|------|------|------|
| `cost` 字段 | `src/query/planner/plan/core/nodes/plan_node_traits.rs` | 所有节点都有 `cost: f64` |
| `cost()` 方法 | `src/query/planner/plan/core/nodes/plan_node_traits.rs` | 已定义在 `PlanNode` trait 中 |
| `CostCalculator` | `src/query/optimizer/cost/calculator.rs` | 已实现基础计算 |
| `StatisticsManager` | `src/query/optimizer/stats/` | 已实现统计信息管理 |
| `SeekStrategy` 代价 | `src/query/planner/statements/seeks/seek_strategy.rs` | 已有 `estimated_cost()` 接口 |

### 1.2 现有问题

1. **代价恒为 0.0**：所有节点创建时 `cost: 0.0`，且从未更新
2. **代价计算与计划生成脱节**：`CostCalculator` 存在但未被 `QueryPlanner` 使用
3. **缺乏统一的代价赋值机制**：没有集中的地方为计划节点计算并设置代价

---

## 二、各操作类型的代价计算方法

### 2.1 扫描操作（Scan Operations）

```rust
/// 全表扫描顶点代价
/// 公式：行数 × CPU处理代价
pub fn calculate_scan_vertices_cost(&self, tag_name: &str) -> f64 {
    let row_count = self.stats_manager.get_vertex_count(tag_name);
    row_count as f64 * self.config.cpu_tuple_cost
}

/// 全表扫描边代价
pub fn calculate_scan_edges_cost(&self, edge_type: &str) -> f64 {
    let edge_stats = self.stats_manager.get_edge_stats(edge_type);
    let row_count = edge_stats.map(|s| s.edge_count).unwrap_or(0);
    row_count as f64 * self.config.cpu_tuple_cost
}
```

### 2.2 索引扫描（Index Scan）

```rust
/// 索引扫描代价
/// 公式：索引访问代价 + 回表代价
pub fn calculate_index_scan_cost(
    &self,
    tag_name: &str,
    property_name: &str,
    selectivity: f64,
) -> f64 {
    let table_rows = self.stats_manager.get_vertex_count(tag_name);
    let matching_rows = (selectivity * table_rows as f64).max(1.0) as u64;
    
    // 索引访问代价（顺序IO）
    let index_pages = (matching_rows / 10).max(1);
    let index_access_cost = index_pages as f64 * self.config.seq_page_cost 
        + matching_rows as f64 * self.config.cpu_index_tuple_cost;
    
    // 回表代价（随机IO）
    let table_access_cost = matching_rows as f64 * self.config.random_page_cost
        + matching_rows as f64 * self.config.cpu_tuple_cost;
    
    index_access_cost + table_access_cost
}
```

### 2.3 图遍历操作（Traversal Operations）

```rust
/// 扩展操作代价（Expand）
pub fn calculate_expand_cost(
    &self,
    start_nodes: u64,
    edge_type: Option<&str>,
) -> f64 {
    let avg_degree = match edge_type {
        Some(et) => {
            self.stats_manager.get_edge_stats(et)
                .map(|s| s.avg_out_degree)
                .unwrap_or(1.0)
        }
        None => 2.0,
    };
    
    let output_rows = (start_nodes as f64 * avg_degree) as u64;
    let io_cost = output_rows as f64 * self.config.seq_page_cost;
    let cpu_cost = output_rows as f64 * self.config.cpu_tuple_cost;
    
    io_cost + cpu_cost
}

/// 多步遍历代价（Traverse）
pub fn calculate_traverse_cost(
    &self,
    start_nodes: u64,
    edge_type: Option<&str>,
    steps: u32,
) -> f64 {
    let avg_degree = match edge_type {
        Some(et) => {
            self.stats_manager.get_edge_stats(et)
                .map(|s| (s.avg_out_degree + s.avg_in_degree) / 2.0)
                .unwrap_or(1.0)
        }
        None => 2.0,
    };
    
    let mut total_cost = 0.0;
    let mut current_rows = start_nodes as f64;
    
    for _ in 0..steps {
        current_rows *= avg_degree;
        total_cost += current_rows * self.config.cpu_tuple_cost;
    }
    
    total_cost
}
```

### 2.4 过滤操作（Filter）

```rust
/// 过滤代价
/// 公式：输入行数 × 条件数量 × 操作符代价
pub fn calculate_filter_cost(
    &self,
    input_rows: u64,
    condition_count: usize,
) -> f64 {
    input_rows as f64 * condition_count as f64 * self.config.cpu_operator_cost
}
```

### 2.5 连接操作（Join）

```rust
/// 哈希内连接代价
pub fn calculate_hash_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
    let build_cost = left_rows as f64 * self.config.cpu_tuple_cost;
    let probe_cost = right_rows as f64 * self.config.cpu_tuple_cost;
    let hash_overhead = left_rows as f64 * 0.1 * self.config.cpu_operator_cost;
    
    build_cost + probe_cost + hash_overhead
}

/// 嵌套循环连接代价
pub fn calculate_nested_loop_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
    let outer_cost = left_rows as f64 * self.config.cpu_tuple_cost;
    let inner_cost = left_rows as f64 * right_rows as f64 * self.config.cpu_tuple_cost;
    
    outer_cost + inner_cost
}
```

### 2.6 排序和聚合

```rust
/// 排序代价
pub fn calculate_sort_cost(&self, input_rows: u64, sort_columns: usize) -> f64 {
    if input_rows == 0 {
        return 0.0;
    }
    let rows = input_rows as f64;
    let comparisons = rows * rows.log2();
    comparisons * sort_columns as f64 * self.config.cpu_operator_cost
}

/// TopN代价
pub fn calculate_topn_cost(&self, input_rows: u64, limit: i64) -> f64 {
    let n = input_rows as f64;
    let k = limit as f64;
    n * k.log2() * self.config.cpu_operator_cost
}

/// 聚合代价
pub fn calculate_aggregate_cost(&self, input_rows: u64, agg_functions: usize) -> f64 {
    input_rows as f64 * agg_functions as f64 * self.config.cpu_operator_cost
}
```

### 2.7 投影和去重

```rust
/// 投影代价
pub fn calculate_project_cost(&self, input_rows: u64, columns: usize) -> f64 {
    input_rows as f64 * columns as f64 * self.config.cpu_operator_cost
}

/// 去重代价
pub fn calculate_dedup_cost(&self, input_rows: u64) -> f64 {
    input_rows as f64 * self.config.cpu_operator_cost * 2.0
}
```

---

## 三、代价计算与执行计划的集成方案

### 3.1 核心集成架构

```
┌─────────────────────────────────────────────────────────────┐
│                     QueryPlanner                            │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐  │
│  │ PlanGenerator│───▶│ CostAssigner │───▶│ PlanCache    │  │
│  └──────────────┘    └──────────────┘    └──────────────┘  │
│         │                   │                             │
│         ▼                   ▼                             │
│  ┌──────────────┐    ┌──────────────┐                     │
│  │ PlanNodeEnum │    │ CostCalculator│                    │
│  └──────────────┘    └──────────────┘                     │
│                               │                           │
│                               ▼                           │
│                        ┌──────────────┐                   │
│                        │StatisticsMgr │                   │
│                        └──────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 代价赋值器（CostAssigner）设计

`CostAssigner` 负责遍历执行计划并为每个节点计算代价：

1. 后序遍历计划树，先计算子节点代价
2. 根据节点类型调用对应的 `CostCalculator` 方法
3. 设置节点的 `cost` 字段
4. 返回累计代价供父节点使用

### 3.3 集成到 QueryPlanner

修改 `QueryPlanner` 以集成代价赋值：

1. 添加 `CostAssigner` 成员
2. 在 `create_plan()` 中，生成计划后调用 `cost_assigner.assign_costs()`
3. 缓存计划时使用实际计算的代价而非节点数估算

### 3.4 代价模型配置

参考 PostgreSQL 设计可配置的代价参数：

| 参数名 | 默认值 | 说明 |
|--------|--------|------|
| `seq_page_cost` | 1.0 | 顺序页读取代价 |
| `random_page_cost` | 4.0 | 随机页读取代价 |
| `cpu_tuple_cost` | 0.01 | 行处理 CPU 代价 |
| `cpu_index_tuple_cost` | 0.005 | 索引行处理代价 |
| `cpu_operator_cost` | 0.0025 | 操作符计算代价 |

---

## 四、实施步骤和优先级

### 阶段 1：基础框架完善（Week 1）

| 任务 | 文件 | 说明 |
|------|------|------|
| 1. 创建 `CostModelConfig` | `optimizer/cost/config.rs` | 定义可配置的代价参数 |
| 2. 扩展 `CostCalculator` | `optimizer/cost/calculator.rs` | 添加所有节点类型的计算方法 |
| 3. 创建 `CostAssigner` | `optimizer/cost/assigner.rs` | 实现计划遍历和代价赋值 |

### 阶段 2：核心节点支持（Week 2）

| 优先级 | 节点类型 | 说明 |
|--------|----------|------|
| P0 | `ScanVertices` | 最基础的扫描操作 |
| P0 | `IndexScan` | 索引选择的核心 |
| P0 | `Filter` | 最常用的操作 |
| P1 | `HashInnerJoin` | 连接操作 |
| P1 | `Expand` / `Traverse` | 图数据库特有操作 |

### 阶段 3：Planner 集成（Week 3）

1. 修改 `QueryPlanner` 添加 `CostAssigner`
2. 在 `create_plan()` 中调用代价赋值
3. 更新计划缓存使用实际代价

### 阶段 4：优化策略集成（Week 4）

将代价计算集成到现有的 `SeekStrategy` 选择中，使用 `CostCalculator` 获取更精确的代价估算。

### 阶段 5：测试和调优（Week 5-6）

1. 编写单元测试验证代价计算
2. 创建基准测试对比不同策略
3. 调整默认代价参数

---

## 五、关键代码集成点总结

| 集成点 | 当前状态 | 需要修改 |
|--------|----------|----------|
| `PlanNode.cost` | 字段存在，恒为 0.0 | 通过 `CostAssigner` 设置 |
| `CostCalculator` | 已实现基础方法 | 扩展所有节点类型 |
| `QueryPlanner` | 使用节点数估算代价 | 集成 `CostAssigner` |
| `SeekStrategy` | 有 `estimated_cost()` | 使用 `CostCalculator` |
| `PlanCache` | 使用节点数 × 100 | 使用实际计算的代价 |

---

## 六、设计优势

1. **非侵入式**：不需要修改现有的计划节点定义，只需要添加代价赋值步骤
2. **可扩展**：新的节点类型只需要在 `CostAssigner` 中添加匹配分支
3. **可配置**：代价参数可以根据硬件环境调整
4. **渐进式**：可以逐步支持更多节点类型，不影响现有功能

---

*文档生成时间：2026-02-27*
*基于：cost_model_refined_plan.md, cost_model_introduction_analysis.md*
