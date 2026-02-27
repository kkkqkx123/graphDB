# GraphDB 代价模型引入分析

## 概述

本文档基于查询优化器分析文档，分析当前 GraphDB 项目引入代价模型（Cost-based Optimization）的必要性、可行性和实施方案。同时参考 PostgreSQL 和 SQLite 的代价模型设计，为 GraphDB 提供设计建议。

---

## 一、当前项目状态分析

### 1.1 现有代价模型架构

根据 [current_cost_model_analysis.md](../cost/current_cost_model_analysis.md) 的分析，当前 GraphDB 的代价模型处于**占位符状态**：

**已实现的部分：**
- 所有计划节点都包含 `cost: f64` 字段
- 节点提供 `cost()` 方法返回代价值
- 计划缓存使用节点数估算代价（`estimated_cost = node_count × 100.0`）
- 查找策略（SeekStrategy）有初步的代价估算（如 `ScanSeek` 使用 `estimated_rows`，`IndexSeek` 使用固定值）

**存在的问题：**
1. **代价恒为零**：所有数据查询节点代价初始化为 0.0，运行时不更新
2. **缺乏统计信息集成**：没有与存储层统计信息集成的机制
3. **无选择性估计**：没有实现条件选择性的估计机制
4. **代价模型与优化器脱节**：优化器规则无法基于代价进行比较和选择

### 1.2 查询优化器现状

根据 [01_query_optimizer_analysis.md](../plan/01_query_optimizer_analysis.md)，当前优化器具有以下特点：

**已实现的优化：**
- 计划缓存（LRU 策略，默认 1000 条）
- 计划重写（20+ 条启发式规则，静态分发）
- 索引选择（标签索引、属性索引查找）

**缺失的能力：**
- 无法区分全表扫描和索引扫描的实际成本差异
- 无法基于代价选择连接策略（目前仅支持 Hash Join）
- 无法基于数据分布选择最优执行计划

---

## 二、PostgreSQL 代价模型参考

### 2.1 核心代价参数设计

PostgreSQL 使用一组可配置的代价参数：

| 参数名 | 默认值 | 说明 |
|--------|--------|------|
| `seq_page_cost` | 1.0 | 顺序读取一个磁盘页面的成本 |
| `random_page_cost` | 4.0 | 随机读取一个磁盘页面的成本 |
| `cpu_tuple_cost` | 0.01 | 处理每一行数据的 CPU 成本 |
| `cpu_index_tuple_cost` | 0.005 | 处理每个索引项的 CPU 成本 |
| `cpu_operator_cost` | 0.0025 | 执行每个操作符或函数的 CPU 成本 |

**对 GraphDB 的启示：**
- 将 I/O 代价和 CPU 代价分离，便于针对不同硬件环境调优
- 顺序访问和随机访问的代价区分对索引选择至关重要
- 代价参数可配置，适应不同部署场景（如 SSD 环境可降低 `random_page_cost`）

### 2.2 统计信息体系

PostgreSQL 的统计信息分为多个层次：

**表级统计（pg_class）：**
- `reltuples`：表的估计行数
- `relpages`：表占用的磁盘页数

**列级统计（pg_stats）：**
- `null_frac`：空值比例
- `n_distinct`：不同值数量
- `most_common_vals`（MCV）：最常见值列表
- `most_common_freqs`：对应 MCV 的频率
- `histogram_bounds`：直方图边界（用于范围查询）
- `correlation`：列值与物理存储顺序的相关性

**对 GraphDB 的启示：**
- 需要维护表/标签级别的统计信息（顶点数、边数）
- 需要维护属性级别的统计信息（不同值数量、空值比例）
- MCV + 直方图的组合可以处理常见值和数据分布

### 2.3 选择性估计方法

**等值条件选择性：**
- 如果查询值在 MCV 列表中，直接使用对应频率
- 否则使用均匀分布假设：`(1 - MCV 总频率) / (不同值数量 - MCV 数量)`

**范围条件选择性：**
- 使用直方图确定查询值落在哪个桶
- 计算该桶内的比例，累加前面完整桶的比例

**连接选择性：**
- 等值连接：`1 / max(左表不同值数, 右表不同值数)`
- 考虑空值：`(1 - 左表空值率) × (1 - 右表空值率) × 基础选择性`

**对 GraphDB 的启示：**
- 选择性估计是代价模型的核心
- 需要为图数据库设计特定的选择性估计方法（如邻居节点分布）

### 2.4 代价计算公式

**顺序扫描：**
```
总代价 = (页面数 × seq_page_cost) + (行数 × cpu_tuple_cost)
```

**索引扫描：**
```
总代价 = 索引访问代价 + 回表代价
索引访问代价 = (索引页数 × seq_page_cost) + (索引行数 × cpu_index_tuple_cost)
回表代价 = (选择性 × 表行数 × random_page_cost) + (选择性 × 表行数 × cpu_tuple_cost)
```

**嵌套循环连接：**
```
代价 = 外表扫描代价 + (外表行数 × 内表扫描代价)
```

**哈希连接：**
```
代价 = 构建哈希表代价 + 探测代价
```

---

## 三、SQLite 代价模型参考

### 3.1 SQLite 的简化设计

SQLite 采用更简化的代价模型：

**特点：**
- 主要基于启发式规则，代价估算相对简单
- 使用 `ANALYZE` 命令收集统计信息存储在 `sqlite_stat1`、`sqlite_stat2`、`sqlite_stat4` 表中
- 统计信息包括：表行数、索引深度、平均扇出等

**代价估算：**
- 全表扫描代价 ≈ 表行数
- 索引扫描代价 ≈ 索引深度 + 匹配行数
- 连接顺序通过动态规划枚举，基于代价选择

**对 GraphDB 的启示：**
- 对于嵌入式/单节点数据库，简化的代价模型可能足够
- 统计信息收集可以延迟执行（显式 `ANALYZE` 命令）

### 3.2 SQLite 的查询优化策略

根据 SQLite 官方文档，其优化器主要依赖：
- 索引选择（基于查询条件）
- 连接顺序优化（基于代价估算）
- 子查询扁平化
- 延迟工作优化（Co-routines）

---

## 四、GraphDB 引入代价模型的必要性分析

### 4.1 值得引入的场景

| 场景 | 必要性 | 说明 |
|------|--------|------|
| 多索引选择 | **高** | 当多个索引可用时，需要基于代价选择最优索引 |
| 连接策略选择 | **中** | 未来支持多种 Join 算法（Hash/Nested Loop）时需要 |
| 扫描方式选择 | **高** | 全表扫描 vs 索引扫描的选择需要代价估算 |
| 复杂图遍历 | **中** | 多跳遍历的路径选择可以基于代价优化 |
| 数据分布不均 | **中** | 代价模型可以适应数据倾斜场景 |

### 4.2 当前阶段是否值得引入

**支持引入的理由：**
1. **项目已有基础**：节点代价字段和代价接口已存在，只是未实现计算逻辑
2. **索引选择需求**：当前索引选择是硬编码的优先级，需要基于代价的决策
3. **计划缓存优化**：基于真实代价的缓存淘汰策略比基于节点数更有效
4. **未来扩展性**：为后续 Join 优化、自适应优化奠定基础

**暂缓引入的理由：**
1. **单节点架构**：GraphDB 是单节点数据库，查询复杂度相对可控
2. **当前优化器已较完善**：20+ 条启发式规则已覆盖大部分优化场景
3. **实现复杂度高**：需要完整的统计信息收集机制和维护成本
4. **收益有限**：对于简单查询，代价模型带来的提升可能不明显

### 4.3 建议方案：渐进式引入

考虑到项目现状，建议采用**渐进式引入**策略：

**阶段 1：基础代价计算（P2 优先级）**
- 为不同操作类型定义基础代价（扫描、过滤、连接等）
- 不依赖统计信息，使用固定值或简单估算
- 目标：让优化器能够区分不同操作的成本差异

**阶段 2：统计信息收集（P1 优先级）**
- 实现表级统计（标签顶点数、边类型数）
- 实现属性级统计（不同值数量）
- 提供 `ANALYZE` 命令手动更新统计信息

**阶段 3：选择性估计（P3 优先级）**
- 基于统计信息实现选择性估计
- 支持等值、范围条件的选择性计算
- 集成到索引选择和连接优化中

**阶段 4：完整代价模型（P4 优先级）**
- 实现类似 PostgreSQL 的分层代价参数
- 支持自适应统计更新
- 考虑 Join 策略的代价比较

---

## 五、GraphDB 代价模型设计方案

### 5.1 核心数据结构

```rust
/// 代价模型配置
pub struct CostModelConfig {
    /// 顺序页读取代价
    pub seq_page_cost: f64,
    /// 随机页读取代价
    pub random_page_cost: f64,
    /// 行处理 CPU 代价
    pub cpu_tuple_cost: f64,
    /// 索引行处理代价
    pub cpu_index_tuple_cost: f64,
    /// 操作符计算代价
    pub cpu_operator_cost: f64,
}

impl Default for CostModelConfig {
    fn default() -> Self {
        Self {
            seq_page_cost: 1.0,
            random_page_cost: 4.0,
            cpu_tuple_cost: 0.01,
            cpu_index_tuple_cost: 0.005,
            cpu_operator_cost: 0.0025,
        }
    }
}

/// 标签统计信息
pub struct TagStatistics {
    pub tag_name: String,
    pub vertex_count: u64,
    pub avg_vertex_size: usize,
    pub last_analyzed: SystemTime,
}

/// 边类型统计信息
pub struct EdgeTypeStatistics {
    pub edge_type: String,
    pub edge_count: u64,
    pub avg_out_degree: f64,
    pub avg_in_degree: f64,
}

/// 属性统计信息
pub struct PropertyStatistics {
    pub property_name: String,
    pub null_fraction: f64,
    pub distinct_values: u64,
    pub most_common_vals: Vec<Value>,
    pub most_common_freqs: Vec<f64>,
    pub histogram_bounds: Option<Vec<Value>>,
}

/// 统计信息管理器
pub struct StatisticsManager {
    tag_stats: HashMap<String, TagStatistics>,
    edge_type_stats: HashMap<String, EdgeTypeStatistics>,
    property_stats: HashMap<String, PropertyStatistics>,
}
```

### 5.2 节点代价计算

```rust
/// 代价计算器
pub struct CostCalculator {
    config: CostModelConfig,
    stats_manager: Arc<StatisticsManager>,
}

impl CostCalculator {
    /// 计算顺序扫描代价
    pub fn calculate_seq_scan_cost(&self, tag_name: &str) -> f64 {
        let stats = self.stats_manager.get_tag_stats(tag_name);
        let page_count = stats.estimate_page_count();
        let row_count = stats.vertex_count;
        
        page_count as f64 * self.config.seq_page_cost
            + row_count as f64 * self.config.cpu_tuple_cost
    }
    
    /// 计算索引扫描代价
    pub fn calculate_index_scan_cost(
        &self,
        tag_name: &str,
        property_name: &str,
        selectivity: f64,
    ) -> f64 {
        let table_stats = self.stats_manager.get_tag_stats(tag_name);
        let index_stats = self.stats_manager.get_property_stats(property_name);
        
        // 索引访问代价
        let index_pages = index_stats.estimate_index_pages();
        let index_rows = table_stats.vertex_count;
        let index_cost = index_pages as f64 * self.config.seq_page_cost
            + index_rows as f64 * self.config.cpu_index_tuple_cost;
        
        // 回表代价
        let matching_rows = (selectivity * table_stats.vertex_count as f64) as u64;
        let table_access_cost = matching_rows as f64 * self.config.random_page_cost
            + matching_rows as f64 * self.config.cpu_tuple_cost;
        
        index_cost + table_access_cost
    }
    
    /// 计算过滤代价
    pub fn calculate_filter_cost(&self, input_rows: u64, condition_complexity: usize) -> f64 {
        input_rows as f64 * self.config.cpu_operator_cost * condition_complexity as f64
    }
    
    /// 计算哈希连接代价
    pub fn calculate_hash_join_cost(
        &self,
        left_rows: u64,
        right_rows: u64,
    ) -> f64 {
        let build_cost = left_rows as f64 * self.config.cpu_tuple_cost;
        let probe_cost = right_rows as f64 * self.config.cpu_tuple_cost;
        let hash_overhead = left_rows as f64 * 0.001; // 哈希构建开销
        
        build_cost + probe_cost + hash_overhead
    }
}
```

### 5.3 选择性估计

```rust
/// 选择性估计器
pub struct SelectivityEstimator {
    stats_manager: Arc<StatisticsManager>,
}

impl SelectivityEstimator {
    /// 估计等值条件选择性
    pub fn estimate_equality_selectivity(
        &self,
        property_name: &str,
        value: &Value,
    ) -> f64 {
        let stats = self.stats_manager.get_property_stats(property_name);
        
        // 检查 MCV
        if let Some(idx) = stats.most_common_vals.iter().position(|v| v == value) {
            return stats.most_common_freqs[idx];
        }
        
        // 使用均匀分布假设
        let mcv_total_freq: f64 = stats.most_common_freqs.iter().sum();
        let remaining_distinct = stats.distinct_values - stats.most_common_vals.len() as u64;
        
        if remaining_distinct > 0 {
            (1.0 - mcv_total_freq) / remaining_distinct as f64
        } else {
            0.0
        }
    }
    
    /// 估计范围条件选择性
    pub fn estimate_range_selectivity(
        &self,
        property_name: &str,
        lower_bound: Option<&Value>,
        upper_bound: Option<&Value>,
    ) -> f64 {
        let stats = self.stats_manager.get_property_stats(property_name);
        
        if let Some(histogram) = &stats.histogram_bounds {
            // 使用直方图估计
            self.estimate_from_histogram(histogram, lower_bound, upper_bound)
        } else {
            // 默认估计：范围条件通常选择 1/3 的数据
            0.333
        }
    }
    
    /// 估计连接选择性
    pub fn estimate_join_selectivity(
        &self,
        left_property: &str,
        right_property: &str,
    ) -> f64 {
        let left_stats = self.stats_manager.get_property_stats(left_property);
        let right_stats = self.stats_manager.get_property_stats(right_property);
        
        let max_distinct = std::cmp::max(left_stats.distinct_values, right_stats.distinct_values);
        
        if max_distinct > 0 {
            1.0 / max_distinct as f64
        } else {
            0.1 // 默认值
        }
    }
}
```

### 5.4 集成到现有架构

```rust
/// 扩展 QueryPlanner 支持代价计算
pub struct QueryPlanner {
    plan_cache: Arc<Mutex<LruCache<PlanCacheKey, Arc<ExecutionPlan>>>>,
    cost_calculator: CostCalculator,
    selectivity_estimator: SelectivityEstimator,
}

impl QueryPlanner {
    /// 生成执行计划时计算代价
    pub fn create_plan(&self, stmt: &Stmt, context: &QueryContext) -> Result<ExecutionPlan, PlannerError> {
        // ... 现有计划生成逻辑 ...
        
        // 计算并设置节点代价
        self.calculate_plan_costs(&mut plan)?;
        
        Ok(plan)
    }
    
    /// 递归计算计划代价
    fn calculate_plan_costs(&self, plan: &mut ExecutionPlan) -> Result<(), PlannerError> {
        for node in plan.nodes_mut() {
            let cost = match node {
                PlanNodeEnum::ScanVertices(n) => {
                    self.cost_calculator.calculate_seq_scan_cost(&n.tag_name)
                }
                PlanNodeEnum::IndexScan(n) => {
                    let selectivity = self.selectivity_estimator
                        .estimate_equality_selectivity(&n.property_name, &n.value);
                    self.cost_calculator.calculate_index_scan_cost(
                        &n.tag_name, &n.property_name, selectivity
                    )
                }
                PlanNodeEnum::Filter(n) => {
                    let input_rows = self.estimate_input_rows(n.input());
                    self.cost_calculator.calculate_filter_cost(input_rows, n.condition_complexity())
                }
                PlanNodeEnum::HashInnerJoin(n) => {
                    let left_rows = self.estimate_input_rows(n.left_input());
                    let right_rows = self.estimate_input_rows(n.right_input());
                    self.cost_calculator.calculate_hash_join_cost(left_rows, right_rows)
                }
                // ... 其他节点类型 ...
                _ => 0.0,
            };
            
            node.set_cost(cost);
        }
        
        Ok(())
    }
}
```

---

## 六、实施建议

### 6.1 优先级排序

根据 [01_query_optimizer_analysis.md](../plan/01_query_optimizer_analysis.md) 的建议优先级，结合代价模型引入的复杂度，建议调整如下：

| 优先级 | 优化项 | 预期收益 | 实现复杂度 | 依赖项 |
|-------|--------|---------|-----------|--------|
| P1 | 统计信息收集机制 | 高 | 中 | 无 |
| P2 | 基础代价计算 | 中 | 低 | 无 |
| P3 | 选择性估计 | 中 | 高 | P1 |
| P4 | 完整代价模型 | 中 | 高 | P1, P2, P3 |
| P5 | Join 策略扩展 | 中 | 高 | P4 |
| P6 | 自适应优化 | 中 | 高 | P4 |

### 6.2 实施步骤

**步骤 1：统计信息收集（2-3 周）**
1. 定义统计信息数据结构
2. 实现统计信息存储（内存 + 持久化）
3. 实现 `ANALYZE` 命令
4. 集成到存储层，收集基础统计

**步骤 2：基础代价计算（1-2 周）**
1. 定义 `CostModelConfig` 和 `CostCalculator`
2. 为各节点类型实现基础代价计算
3. 集成到 `QueryPlanner`
4. 更新计划缓存的代价估算

**步骤 3：选择性估计（2-3 周）**
1. 实现 `SelectivityEstimator`
2. 支持等值、范围条件的选择性估计
3. 集成到索引选择逻辑
4. 添加单元测试和验证

**步骤 4：集成测试与调优（2 周）**
1. 编写基准测试验证优化效果
2. 调整代价参数默认值
3. 处理边界情况和异常

### 6.3 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 统计信息不准确导致计划劣化 | 中 | 高 | 提供手动指定计划的方式；添加计划回退机制 |
| 统计信息维护开销过大 | 中 | 中 | 支持异步统计更新；提供配置关闭自动更新 |
| 代价模型复杂度增加维护成本 | 高 | 低 | 模块化设计；完善的文档和测试 |
| 与现有优化规则冲突 | 低 | 中 | 渐进式集成；充分测试 |

---

## 七、结论

### 7.1 是否值得引入代价模型

**结论：值得引入，但建议采用渐进式策略**

理由：
1. **项目已有良好基础**：节点代价框架已存在，只需填充计算逻辑
2. **索引选择是刚需**：当前硬编码的索引选择优先级需要被代价驱动的决策取代
3. **未来扩展需要**：Join 优化、自适应优化等高级特性依赖代价模型
4. **风险可控**：渐进式引入可以逐步验证效果，降低风险

### 7.2 与 PostgreSQL/SQLite 的对比

| 特性 | PostgreSQL | SQLite | 建议的 GraphDB 方案 |
|------|------------|--------|-------------------|
| 代价参数 | 丰富可配置 | 简化 | 中等复杂度，可配置关键参数 |
| 统计信息 | MCV + 直方图 | 简化统计 | 先实现基础统计，再考虑 MCV |
| 选择性估计 | 精确 | 简化 | 中等精度，满足常见场景 |
| 自动更新 | 支持 | 手动 ANALYZE | 先支持手动，再考虑自动 |
| 适用场景 | 企业级 | 嵌入式 | 单节点图数据库 |

### 7.3 下一步行动

1. **短期（1-2 周）**：
   - 评审本分析文档
   - 确定是否启动阶段 1（统计信息收集）
   - 设计统计信息存储方案

2. **中期（1-2 月）**：
   - 实现基础统计信息收集
   - 实现基础代价计算
   - 集成到现有优化器

3. **长期（3-6 月）**：
   - 完善选择性估计
   - 支持更多优化场景
   - 性能调优和验证

---

## 参考文档

- [PostgreSQL 16 - Planner/Optimizer](https://www.postgresql.org/docs/16/planner-optimizer.html)
- [PostgreSQL 16 - Statistics Used by the Planner](https://www.postgresql.org/docs/16/planner-stats.html)
- [SQLite Query Optimizer Overview](https://sqlite.org/optoverview.html)
- [SQLite Next Generation Query Planner](https://sqlite.org/queryplanner-ng.html)
- [GraphDB 当前代价模型分析](../cost/current_cost_model_analysis.md)
- [GraphDB 查询优化器分析](../plan/01_query_optimizer_analysis.md)
