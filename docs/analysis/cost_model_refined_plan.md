# GraphDB 轻量级代价模型实施方案（调整版）

## 概述

本文档提供调整后的分阶段实施方案，主要变更：
1. 在 `query` 模块下创建独立的 `optimizer` 层
2. 明确各模块的目录归属

---

## 目录结构设计

### 现有结构

```
src/query/
├── parser/          # 查询解析
├── planner/         # 查询规划
│   ├── plan/        # 执行计划定义
│   ├── rewrite/     # 计划重写规则
│   └── statements/  # 语句规划器
└── executor/        # 执行器
```

### 新增结构

```
src/query/
├── parser/          # 查询解析（不变）
├── planner/         # 查询规划（不变）
├── optimizer/       # 查询优化器（新增）
│   ├── cost/        # 代价计算
│   ├── stats/       # 统计信息
│   └── strategy/    # 优化策略
├── executor/        # 执行器（不变）
└── mod.rs
```

---

## 模块归属说明

| 模块 | 归属目录 | 说明 |
|------|---------|------|
| `StatisticsManager` | `optimizer/stats/` | 统计信息管理 |
| `CostCalculator` | `optimizer/cost/` | 代价计算 |
| `SelectivityEstimator` | `optimizer/cost/` | 选择性估计 |
| `TraversalStartSelector` | `optimizer/strategy/` | 遍历起点选择策略 |
| `IndexSelector` | `optimizer/strategy/` | 索引选择策略 |
| `AnalyzeExecutor` | `executor/admin/` | ANALYZE 命令执行器（已在该目录） |

---

## 阶段 1：统计信息模块（Week 1-2）

### 1.1 目录结构

```
src/query/optimizer/
└── stats/
    ├── mod.rs
    ├── manager.rs          # StatisticsManager
    ├── collector.rs        # StatisticsCollector
    ├── tag.rs              # TagStatistics
    ├── edge.rs             # EdgeTypeStatistics
    └── property.rs         # PropertyStatistics
```

### 1.2 核心组件

#### StatisticsManager（统计信息管理器）

```rust
// src/query/optimizer/stats/manager.rs

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// 统计信息管理器
/// 
/// 统一管理所有统计信息，提供线程安全的访问
pub struct StatisticsManager {
    /// 标签统计信息
    tag_stats: Arc<RwLock<HashMap<String, TagStatistics>>>,
    /// 边类型统计信息
    edge_stats: Arc<RwLock<HashMap<String, EdgeTypeStatistics>>>,
    /// 属性统计信息
    property_stats: Arc<RwLock<HashMap<String, PropertyStatistics>>>,
}

impl StatisticsManager {
    pub fn new() -> Self {
        Self {
            tag_stats: Arc::new(RwLock::new(HashMap::new())),
            edge_stats: Arc::new(RwLock::new(HashMap::new())),
            property_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn get_tag_stats(&self, tag_name: &str) -> Option<TagStatistics> {
        self.tag_stats.read().get(tag_name).cloned()
    }
    
    pub fn update_tag_stats(&self, stats: TagStatistics) {
        self.tag_stats.write().insert(stats.tag_name.clone(), stats);
    }
    
    pub fn get_vertex_count(&self, tag_name: &str) -> u64 {
        self.get_tag_stats(tag_name)
            .map(|s| s.vertex_count)
            .unwrap_or(0)
    }
    
    // ... 其他方法
}

impl Default for StatisticsManager {
    fn default() -> Self {
        Self::new()
    }
}
```

#### TagStatistics（标签统计）

```rust
// src/query/optimizer/stats/tag.rs

use std::time::SystemTime;

/// 标签统计信息
#[derive(Debug, Clone)]
pub struct TagStatistics {
    /// 标签名称
    pub tag_name: String,
    /// 顶点数量
    pub vertex_count: u64,
    /// 平均出度（关键指标：影响遍历代价）
    pub avg_out_degree: f64,
    /// 平均入度
    pub avg_in_degree: f64,
    /// 平均顶点大小（字节）
    pub avg_vertex_size: usize,
    /// 最后更新时间
    pub last_analyzed: SystemTime,
}

impl TagStatistics {
    pub fn new(tag_name: String) -> Self {
        Self {
            tag_name,
            vertex_count: 0,
            avg_out_degree: 0.0,
            avg_in_degree: 0.0,
            avg_vertex_size: 0,
            last_analyzed: SystemTime::now(),
        }
    }
    
    /// 估算遍历代价
    pub fn estimate_traversal_cost(&self, start_nodes: u64, steps: u32) -> f64 {
        let degree = (self.avg_out_degree + self.avg_in_degree) / 2.0;
        start_nodes as f64 * degree.powi(steps as i32)
    }
}
```

#### EdgeTypeStatistics（边类型统计）

```rust
// src/query/optimizer/stats/edge.rs

use std::time::SystemTime;

/// 边类型统计信息
#[derive(Debug, Clone)]
pub struct EdgeTypeStatistics {
    /// 边类型名称
    pub edge_type: String,
    /// 边总数
    pub edge_count: u64,
    /// 平均出度
    pub avg_out_degree: f64,
    /// 平均入度
    pub avg_in_degree: f64,
    /// 唯一源顶点数
    pub unique_src_vertices: u64,
    /// 唯一目标顶点数
    pub unique_dst_vertices: u64,
    /// 最后更新时间
    pub last_analyzed: SystemTime,
}

impl EdgeTypeStatistics {
    pub fn new(edge_type: String) -> Self {
        Self {
            edge_type,
            edge_count: 0,
            avg_out_degree: 0.0,
            avg_in_degree: 0.0,
            unique_src_vertices: 0,
            unique_dst_vertices: 0,
            last_analyzed: SystemTime::now(),
        }
    }
    
    /// 估算扩展代价
    pub fn estimate_expand_cost(&self, start_nodes: u64) -> f64 {
        start_nodes as f64 * self.avg_out_degree
    }
}
```

#### PropertyStatistics（属性统计）

```rust
// src/query/optimizer/stats/property.rs

use std::time::SystemTime;
use crate::core::Value;

/// 属性统计信息
#[derive(Debug, Clone)]
pub struct PropertyStatistics {
    /// 属性名称
    pub property_name: String,
    /// 所属标签（可选）
    pub tag_name: Option<String>,
    /// 不同值数量
    pub distinct_values: u64,
    /// 空值比例
    pub null_fraction: f64,
    /// 最小值
    pub min_value: Option<Value>,
    /// 最大值
    pub max_value: Option<Value>,
    /// 最后更新时间
    pub last_analyzed: SystemTime,
}

impl PropertyStatistics {
    pub fn new(property_name: String, tag_name: Option<String>) -> Self {
        Self {
            property_name,
            tag_name,
            distinct_values: 0,
            null_fraction: 0.0,
            min_value: None,
            max_value: None,
            last_analyzed: SystemTime::now(),
        }
    }
    
    /// 估算等值条件选择性
    pub fn estimate_equality_selectivity(&self) -> f64 {
        if self.distinct_values == 0 {
            0.1
        } else {
            1.0 / self.distinct_values as f64
        }
    }
}
```

#### StatisticsCollector（统计信息收集器）

```rust
// src/query/optimizer/stats/collector.rs

use crate::storage::StorageClient;
use crate::core::{StorageError, Vertex, Edge};

/// 统计信息收集器
pub struct StatisticsCollector<S: StorageClient> {
    storage: Arc<S>,
}

impl<S: StorageClient> StatisticsCollector<S> {
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }
    
    /// 收集标签统计信息
    pub fn collect_tag_stats(&self, tag_name: &str) -> Result<TagStatistics, StorageError> {
        let mut stats = TagStatistics::new(tag_name.to_string());
        
        let vertices: Vec<Vertex> = self.storage.scan_vertices_by_tag(tag_name)?.collect();
        stats.vertex_count = vertices.len() as u64;
        
        if stats.vertex_count > 0 {
            let total_size: usize = vertices.iter()
                .map(|v| std::mem::size_of_val(v))
                .sum();
            stats.avg_vertex_size = total_size / vertices.len();
            
            let (avg_out, avg_in) = self.calculate_average_degrees(&vertices)?;
            stats.avg_out_degree = avg_out;
            stats.avg_in_degree = avg_in;
        }
        
        Ok(stats)
    }
    
    /// 收集所有统计信息
    pub fn collect_all_stats(&self) -> Result<StatisticsCollection, StorageError> {
        let mut collection = StatisticsCollection::new();
        
        for tag_name in self.storage.get_all_tags()? {
            let stats = self.collect_tag_stats(&tag_name)?;
            collection.tag_stats.push(stats);
        }
        
        for edge_type in self.storage.get_all_edge_types()? {
            let stats = self.collect_edge_stats(&edge_type)?;
            collection.edge_stats.push(stats);
        }
        
        Ok(collection)
    }
    
    // ... 其他方法
}

/// 统计信息集合
pub struct StatisticsCollection {
    pub tag_stats: Vec<TagStatistics>,
    pub edge_stats: Vec<EdgeTypeStatistics>,
    pub property_stats: Vec<PropertyStatistics>,
}

impl StatisticsCollection {
    pub fn new() -> Self {
        Self {
            tag_stats: Vec::new(),
            edge_stats: Vec::new(),
            property_stats: Vec::new(),
        }
    }
}
```

#### mod.rs

```rust
// src/query/optimizer/stats/mod.rs

pub mod manager;
pub mod collector;
pub mod tag;
pub mod edge;
pub mod property;

pub use manager::StatisticsManager;
pub use collector::{StatisticsCollector, StatisticsCollection};
pub use tag::TagStatistics;
pub use edge::EdgeTypeStatistics;
pub use property::PropertyStatistics;
```

---

## 阶段 2：代价计算模块（Week 2-3）

### 2.1 目录结构

```
src/query/optimizer/
└── cost/
    ├── mod.rs
    ├── calculator.rs       # CostCalculator
    └── selectivity.rs      # SelectivityEstimator
```

### 2.2 核心组件

#### CostCalculator（代价计算器）

```rust
// src/query/optimizer/cost/calculator.rs

use crate::query::optimizer::stats::StatisticsManager;

/// 代价计算器
/// 
/// 针对图数据库特性设计的轻量级代价计算
pub struct CostCalculator {
    stats_manager: Arc<StatisticsManager>,
}

impl CostCalculator {
    pub fn new(stats_manager: Arc<StatisticsManager>) -> Self {
        Self { stats_manager }
    }
    
    /// 计算全表扫描代价
    pub fn calculate_scan_cost(&self, tag_name: &str) -> f64 {
        let row_count = self.stats_manager.get_vertex_count(tag_name);
        row_count as f64
    }
    
    /// 计算索引扫描代价
    pub fn calculate_index_scan_cost(
        &self,
        tag_name: &str,
        selectivity: f64,
    ) -> f64 {
        let table_rows = self.stats_manager.get_vertex_count(tag_name);
        let matching_rows = (selectivity * table_rows as f64) as u64;
        let index_pages = (matching_rows / 10).max(1);
        
        index_pages as f64 * 0.1 + matching_rows as f64
    }
    
    /// 计算单步扩展代价
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
        
        start_nodes as f64 * avg_degree
    }
    
    /// 计算多步遍历代价
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
        
        start_nodes as f64 * avg_degree.powi(steps as i32)
    }
    
    /// 计算过滤代价
    pub fn calculate_filter_cost(&self, input_rows: u64, condition_count: usize) -> f64 {
        input_rows as f64 * condition_count as f64 * 0.01
    }
    
    /// 计算哈希连接代价
    pub fn calculate_hash_join_cost(&self, left_rows: u64, right_rows: u64) -> f64 {
        let build_cost = left_rows as f64;
        let probe_cost = right_rows as f64;
        let hash_overhead = left_rows as f64 * 0.1;
        
        build_cost + probe_cost + hash_overhead
    }
    
    pub fn statistics_manager(&self) -> Arc<StatisticsManager> {
        self.stats_manager.clone()
    }
}
```

#### SelectivityEstimator（选择性估计器）

```rust
// src/query/optimizer/cost/selectivity.rs

use crate::query::optimizer::stats::StatisticsManager;
use crate::core::Expression;

/// 选择性估计器
pub struct SelectivityEstimator {
    stats_manager: Arc<StatisticsManager>,
}

impl SelectivityEstimator {
    pub fn new(stats_manager: Arc<StatisticsManager>) -> Self {
        Self { stats_manager }
    }
    
    /// 估计等值条件选择性
    pub fn estimate_equality_selectivity(
        &self,
        tag_name: Option<&str>,
        property_name: &str,
    ) -> f64 {
        let stats = self.stats_manager.get_property_stats(tag_name, property_name);
        
        match stats {
            Some(s) if s.distinct_values > 0 => {
                1.0 / s.distinct_values as f64
            }
            _ => 0.1,
        }
    }
    
    /// 估计范围条件选择性
    pub fn estimate_range_selectivity(&self) -> f64 {
        0.333
    }
    
    /// 从表达式估计选择性
    pub fn estimate_from_expression(
        &self,
        expr: &Expression,
        tag_name: Option<&str>,
    ) -> f64 {
        // 实现细节...
        0.1
    }
}
```

#### mod.rs

```rust
// src/query/optimizer/cost/mod.rs

pub mod calculator;
pub mod selectivity;

pub use calculator::CostCalculator;
pub use selectivity::SelectivityEstimator;
```

---

## 阶段 3：优化策略模块（Week 3-5）

### 3.1 目录结构

```
src/query/optimizer/
└── strategy/
    ├── mod.rs
    ├── traversal_start.rs   # TraversalStartSelector
    └── index.rs             # IndexSelector
```

### 3.2 核心组件

#### TraversalStartSelector（遍历起点选择器）

```rust
// src/query/optimizer/strategy/traversal_start.rs

use crate::query::optimizer::cost::{CostCalculator, SelectivityEstimator};
use crate::query::parser::ast::pattern::{Pattern, NodePattern};

/// 遍历起点选择器
pub struct TraversalStartSelector {
    cost_calculator: Arc<CostCalculator>,
    selectivity_estimator: Arc<SelectivityEstimator>,
}

/// 候选起点信息
#[derive(Debug, Clone)]
pub struct CandidateStart {
    pub node_pattern: NodePattern,
    pub estimated_start_nodes: u64,
    pub estimated_cost: f64,
    pub reason: SelectionReason,
}

#[derive(Debug, Clone)]
pub enum SelectionReason {
    ExplicitVid,
    HighSelectivityIndex { selectivity: f64 },
    TagIndex { vertex_count: u64 },
    FullScan { vertex_count: u64 },
}

impl TraversalStartSelector {
    pub fn new(
        cost_calculator: Arc<CostCalculator>,
        selectivity_estimator: Arc<SelectivityEstimator>,
    ) -> Self {
        Self {
            cost_calculator,
            selectivity_estimator,
        }
    }
    
    /// 选择最优遍历起点
    pub fn select_start_node(&self, pattern: &Pattern) -> CandidateStart {
        let candidates: Vec<CandidateStart> = pattern.nodes()
            .iter()
            .map(|node| self.evaluate_node(node))
            .collect();
        
        candidates.into_iter()
            .min_by(|a, b| a.estimated_cost.partial_cmp(&b.estimated_cost).unwrap())
            .expect("Pattern should have at least one node")
    }
    
    fn evaluate_node(&self, node: &NodePattern) -> CandidateStart {
        // 实现细节...
        todo!()
    }
}
```

#### IndexSelector（索引选择器）

```rust
// src/query/optimizer/strategy/index.rs

use crate::query::optimizer::cost::{CostCalculator, SelectivityEstimator};
use crate::index::IndexMetadata;

/// 索引选择器
pub struct IndexSelector {
    cost_calculator: Arc<CostCalculator>,
    selectivity_estimator: Arc<SelectivityEstimator>,
}

/// 索引选择结果
#[derive(Debug, Clone)]
pub enum IndexSelection {
    PropertyIndex {
        index_name: String,
        property_name: String,
        estimated_cost: f64,
        selectivity: f64,
    },
    TagIndex {
        estimated_cost: f64,
        vertex_count: u64,
    },
    FullScan {
        estimated_cost: f64,
        vertex_count: u64,
    },
}

impl IndexSelector {
    pub fn new(
        cost_calculator: Arc<CostCalculator>,
        selectivity_estimator: Arc<SelectivityEstimator>,
    ) -> Self {
        Self {
            cost_calculator,
            selectivity_estimator,
        }
    }
    
    /// 为查询选择最优索引
    pub fn select_index(
        &self,
        tag_name: &str,
        predicates: &[PropertyPredicate],
        available_indexes: &[IndexMetadata],
    ) -> IndexSelection {
        // 实现细节...
        todo!()
    }
}

impl IndexSelection {
    pub fn estimated_cost(&self) -> f64 {
        match self {
            IndexSelection::PropertyIndex { estimated_cost, .. } => *estimated_cost,
            IndexSelection::TagIndex { estimated_cost, .. } => *estimated_cost,
            IndexSelection::FullScan { estimated_cost, .. } => *estimated_cost,
        }
    }
}
```

#### mod.rs

```rust
// src/query/optimizer/strategy/mod.rs

pub mod traversal_start;
pub mod index;

pub use traversal_start::{TraversalStartSelector, CandidateStart, SelectionReason};
pub use index::{IndexSelector, IndexSelection};
```

---

## 阶段 4：ANALYZE 命令（Week 5-6）

### 4.1 目录归属

`AnalyzeExecutor` 属于执行器层，放在 `src/query/executor/admin/` 目录（已存在）。

### 4.2 核心组件

```rust
// src/query/executor/admin/analyze.rs（新增文件）

use crate::query::optimizer::stats::{StatisticsCollector, StatisticsManager};

/// ANALYZE 命令执行器
pub struct AnalyzeExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    target: AnalyzeTarget,
}

#[derive(Debug, Clone)]
pub enum AnalyzeTarget {
    All,
    Tag(String),
    EdgeType(String),
    Property { tag: Option<String>, property: String },
}

impl<S: StorageClient> AnalyzeExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "AnalyzeExecutor".to_string(), storage),
            target: AnalyzeTarget::All,
        }
    }
    
    // ... 实现细节
}
```

---

## 完整目录结构

```
src/query/
├── mod.rs
├── parser/
│   └── ...
├── planner/
│   ├── mod.rs
│   ├── plan/
│   ├── rewrite/
│   ├── statements/
│   ├── connector.rs
│   ├── planner.rs
│   └── template_extractor.rs
├── optimizer/                    # 新增：查询优化器层
│   ├── mod.rs
│   ├── stats/                    # 统计信息模块
│   │   ├── mod.rs
│   │   ├── manager.rs            # StatisticsManager
│   │   ├── collector.rs          # StatisticsCollector
│   │   ├── tag.rs                # TagStatistics
│   │   ├── edge.rs               # EdgeTypeStatistics
│   │   └── property.rs           # PropertyStatistics
│   ├── cost/                     # 代价计算模块
│   │   ├── mod.rs
│   │   ├── calculator.rs         # CostCalculator
│   │   └── selectivity.rs        # SelectivityEstimator
│   └── strategy/                 # 优化策略模块
│       ├── mod.rs
│       ├── traversal_start.rs    # TraversalStartSelector
│       └── index.rs              # IndexSelector
└── executor/
    ├── mod.rs
    ├── base/
    ├── admin/
    │   ├── mod.rs
    │   └── analyze.rs            # AnalyzeExecutor（新增）
    └── ...
```

---

## 实施时间表（调整版）

| 阶段 | 内容 | 时间 | 主要文件 |
|------|------|------|---------|
| 1 | 统计信息模块 | Week 1-2 | `optimizer/stats/*.rs` |
| 2 | 代价计算模块 | Week 2-3 | `optimizer/cost/*.rs` |
| 3 | 优化策略模块 | Week 3-5 | `optimizer/strategy/*.rs` |
| 4 | ANALYZE 命令 | Week 5-6 | `executor/admin/analyze.rs` |
| 5 | 集成与测试 | Week 6-7 | 修改 `planner/*.rs` |

---

## 关键设计原则

1. **命名简洁**：移除 "simplified"、"simple" 等修饰词，使用直接命名
2. **目录清晰**：`optimizer` 层独立，内部按功能分为 `stats`、`cost`、`strategy`
3. **职责分明**：
   - `stats`：统计信息收集和管理
   - `cost`：代价计算和选择性估计
   - `strategy`：基于代价的优化决策
   - `executor/admin`：ANALYZE 命令执行
4. **渐进式集成**：每个阶段完成后可独立测试，最后集成到 planner
