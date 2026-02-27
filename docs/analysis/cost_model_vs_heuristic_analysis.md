# GraphDB 代价模型 vs 启发式规则深度分析

## 问题背景

用户提出关键问题：由于图数据库的 Join 需求比传统 RDBMS 更小，是否应该效仿 SQLite 删除代价体系，仅保留启发式规则？

本文档基于对 GraphDB 代码的深入分析，重新评估代价模型的必要性。

---

## 一、图数据库 vs 传统 RDBMS 的 Join 差异分析

### 1.1 传统 RDBMS 的 Join 特点

在关系型数据库中，Join 是核心操作：
- **多表连接是常态**：业务逻辑通常需要连接多个表（用户表、订单表、商品表等）
- **连接顺序至关重要**：N 个表连接有 N! 种顺序，不同顺序性能差异巨大
- **连接算法选择复杂**：Hash Join、Nested Loop Join、Sort-Merge Join 需要根据数据量选择
- **代价模型是刚需**：没有代价模型无法确定最优连接顺序和算法

### 1.2 图数据库的 Join 特点

通过分析 GraphDB 代码，发现图数据库的 Join 使用场景确实更少：

**代码分析发现：**

1. **MATCH 查询使用 CrossJoin 处理多路径**（[match_statement_planner.rs](../../src/query/planner/statements/match_statement_planner.rs)）：
   ```rust
   // 处理额外的路径模式（使用交叉连接）
   for pattern in match_stmt.patterns.iter().skip(1) {
       let path_plan = self.plan_path_pattern(pattern, space_id, sym_table)?;
       plan = self.cross_join_plans(plan, path_plan)?;
   }
   ```
   多路径 MATCH 查询使用 CrossJoin，但这种情况相对较少。

2. **GO 查询使用 HashInnerJoin**（[go_planner.rs](../../src/query/planner/statements/go_planner.rs)）：
   ```rust
   // GO 查询主要使用 ExpandAll + Filter + Project
   // 没有复杂的 Join 逻辑
   ```

3. **Join 节点类型**（[join_node.rs](../../src/query/planner/plan/core/nodes/join_node.rs)）：
   - `InnerJoinNode`、`LeftJoinNode`、`CrossJoinNode`
   - `HashInnerJoinNode`、`HashLeftJoinNode`、`FullOuterJoinNode`
   虽然定义了多种 Join 类型，但实际使用场景有限。

### 1.3 图数据库的核心操作：遍历而非 Join

图数据库的核心是**遍历操作**而非 Join：

**遍历节点类型**（[traversal_node.rs](../../src/query/planner/plan/core/nodes/traversal_node.rs)）：
- `ExpandNode`：单步扩展
- `ExpandAllNode`：全路径扩展（GO 查询主要使用）
- `TraverseNode`：多步遍历（支持 min/max steps）
- `GetNeighborsNode`：获取邻居节点

**关键区别：**
| 特性 | 传统 RDBMS | 图数据库 |
|------|-----------|---------|
| 核心操作 | Join | 遍历（Traverse/Expand） |
| 连接复杂度 | 高（多表笛卡尔积） | 低（基于边关系的局部扩展） |
| 数据访问模式 | 随机访问多个表 | 沿着边关系顺序访问 |
| 优化重点 | 连接顺序和算法 | 遍历起点选择和剪枝 |

---

## 二、SQLite 简化策略分析

### 2.1 SQLite 的优化器特点

SQLite 采用极简的查询优化策略：

1. **代价模型简化**：
   - 全表扫描代价 ≈ 表行数
   - 索引扫描代价 ≈ 索引深度 + 匹配行数
   - 没有复杂的 CPU/I/O 分离

2. **统计信息简化**：
   - 存储在 `sqlite_stat1`、`sqlite_stat2`、`sqlite_stat4` 表中
   - 仅包含：表行数、索引深度、平均扇出等基础信息
   - 没有 MCV、直方图等复杂统计

3. **优化策略**：
   - 主要依赖启发式规则
   - 索引选择基于简单代价比较
   - 连接顺序通过动态规划，但代价估算简单

### 2.2 SQLite 为什么可以简化？

1. **嵌入式场景**：
   - 单用户、单连接
   - 查询通常简单
   - 数据量相对较小

2. **Join 场景有限**：
   - 嵌入式应用通常表数量少
   - 复杂查询在应用层处理

3. **资源受限环境**：
   - 移动设备、IoT 设备
   - 优化器本身需要轻量级

---

## 三、GraphDB 是否应该效仿 SQLite？

### 3.1 支持效仿 SQLite 的理由

**1. 单节点架构**
- GraphDB 是单节点图数据库，非分布式
- 查询复杂度相对可控
- 不需要处理跨节点 Join 的代价估算

**2. Join 需求确实较少**
- 核心操作是图遍历（Expand/Traverse）
- Join 主要用于多路径 MATCH 和特定查询
- 遍历操作的代价估算相对简单

**3. 当前启发式规则已较完善**
- 20+ 条重写规则覆盖常见优化场景
- 索引选择已有基础逻辑
- 计划缓存机制有效

**4. 维护成本考虑**
- 完整代价模型需要统计信息收集、维护
- 增加系统复杂度和资源消耗
- 对于简单查询，收益可能不明显

### 3.2 反对完全删除代价体系的理由

**1. 图数据库特有的优化需求**

虽然 Join 少，但图数据库有其他需要代价估算的场景：

- **遍历起点选择**：
  ```cypher
  MATCH (a:Person {name: 'Alice'})-[:KNOWS]->(b:Person)
  ```
  如果 `:Person {name: 'Alice'}` 匹配 1 个节点，而 `:Person` 有 100 万个节点，
  从 'Alice' 开始遍历比全表扫描后过滤高效得多。

- **多跳遍历策略**：
  ```cypher
  MATCH (a)-[:KNOWS*1..3]->(b)
  ```
  需要估算每跳的平均度数，决定是否使用双向遍历或剪枝。

- **索引 vs 全表扫描**：
  即使 Join 少，单表/单标签的索引选择仍然需要代价估算。

**2. 项目已有代价框架**

当前代码已有代价模型的基础：
- 所有节点有 `cost` 字段
- 查找策略有 `estimated_cost` 方法
- 计划缓存使用代价进行淘汰

完全删除意味着：
- 需要重构现有代码
- 计划缓存的代价淘汰策略需要重新设计

**3. 未来扩展性**

即使当前 Join 少，未来可能需要：
- 复杂的图模式匹配（多个子图连接）
- 图算法与关系操作的混合查询
- 子查询优化

**4. 与 SQLite 的场景差异**

| 特性 | SQLite | GraphDB |
|------|--------|---------|
| 目标场景 | 嵌入式、移动设备 | 服务端、个人/小型应用 |
| 数据模型 | 关系型 | 图模型 |
| 核心操作 | SQL 查询 | 图遍历 + Cypher 查询 |
| 查询复杂度 | 通常简单 | 可能复杂（多跳遍历） |
| 资源限制 | 严格 | 相对宽松 |

---

## 四、重新评估：折中方案

### 4.1 核心结论

**不建议完全删除代价体系，但建议大幅简化，采用"轻量级代价模型"。**

理由：
1. 图数据库的核心优化需求是**遍历起点选择**和**索引选择**，不是 Join 优化
2. 这些场景需要简单的代价比较，不需要 PostgreSQL 级别的复杂模型
3. 项目已有代价框架，完全删除成本较高

### 4.2 推荐的轻量级方案

**方案：保留简化版代价体系，聚焦图数据库核心需求**

#### 阶段 1：基础统计信息（必须）

```rust
/// 简化的标签统计信息
pub struct TagStatistics {
    pub tag_name: String,
    pub vertex_count: u64,        // 顶点数量
    pub avg_out_degree: f64,      // 平均出度（对遍历优化至关重要）
    pub avg_in_degree: f64,       // 平均入度
}

/// 简化的属性统计信息
pub struct PropertyStatistics {
    pub property_name: String,
    pub distinct_values: u64,     // 不同值数量（用于选择性估计）
    pub null_fraction: f64,       // 空值比例
}
```

**收集方式：**
- 后台异步收集（不阻塞查询）
- 提供 `ANALYZE` 命令手动触发
- 统计信息可以不那么实时（图数据通常变化较慢）

#### 阶段 2：简化代价计算

```rust
/// 简化的代价计算器
pub struct SimpleCostCalculator;

impl SimpleCostCalculator {
    /// 扫描代价 = 行数
    pub fn scan_cost(row_count: u64) -> f64 {
        row_count as f64
    }
    
    /// 索引扫描代价 = 索引页数 + 匹配行数
    pub fn index_scan_cost(index_pages: u64, matching_rows: u64) -> f64 {
        index_pages as f64 * 0.1 + matching_rows as f64
    }
    
    /// 遍历代价 = 起始节点数 × 平均度数^跳数
    pub fn traverse_cost(
        start_nodes: u64,
        avg_degree: f64,
        steps: u32,
    ) -> f64 {
        start_nodes as f64 * avg_degree.powi(steps as i32)
    }
    
    /// 过滤代价 = 输入行数 × 条件复杂度
    pub fn filter_cost(input_rows: u64, condition_count: usize) -> f64 {
        input_rows as f64 * condition_count as f64 * 0.01
    }
}
```

**特点：**
- 没有复杂的 I/O/CPU 分离
- 没有 MCV、直方图
- 使用均匀分布假设进行选择性估计

#### 阶段 3：聚焦核心优化场景

**场景 1：遍历起点选择**
```rust
/// 选择最优遍历起点
fn select_traversal_start(
    patterns: &[NodePattern],
    stats: &StatisticsManager,
) -> NodePattern {
    patterns.iter()
        .min_by_key(|p| {
            let estimated_rows = estimate_pattern_rows(p, stats);
            estimated_rows
        })
        .cloned()
        .unwrap()
}
```

**场景 2：索引选择**
```rust
/// 选择最优索引
fn select_index(
    tag_name: &str,
    predicates: &[PropertyPredicate],
    stats: &StatisticsManager,
) -> IndexSelection {
    let scan_cost = SimpleCostCalculator::scan_cost(
        stats.get_vertex_count(tag_name)
    );
    
    let index_cost = predicates.iter()
        .filter_map(|p| {
            let selectivity = 1.0 / stats.get_distinct_values(&p.property);
            let matching_rows = (selectivity * stats.get_vertex_count(tag_name) as f64) as u64;
            Some(SimpleCostCalculator::index_scan_cost(10, matching_rows))
        })
        .min_by(|a, b| a.partial_cmp(b).unwrap());
    
    match index_cost {
        Some(cost) if cost < scan_cost => IndexSelection::Index(index_name),
        _ => IndexSelection::Scan,
    }
}
```

### 4.3 与 SQLite 策略的对比

| 特性 | SQLite | 推荐的 GraphDB 方案 |
|------|--------|-------------------|
| 代价模型 | 极简（行数为主） | 简化（行数 + 度数） |
| 统计信息 | 基础（行数、索引深度） | 图特化（顶点数、平均度数） |
| 优化重点 | 索引选择 | 遍历起点 + 索引选择 |
| 选择性估计 | 简单假设 | 均匀分布假设 |
| 维护成本 | 极低 | 低 |

---

## 五、实施建议

### 5.1 优先级调整

基于以上分析，建议调整优先级：

| 优先级 | 项目 | 说明 |
|-------|------|------|
| P0 | 简化版统计信息 | 顶点数、平均度数（对遍历优化至关重要） |
| P1 | 遍历起点选择优化 | 基于统计信息选择最优起点 |
| P2 | 基础索引选择 | 简单的代价比较 |
| P3 | 选择性估计 | 均匀分布假设即可 |
| P4 | 完整代价模型 | 暂不实施，视需求而定 |

### 5.2 不实施的项目

以下项目建议暂不实施：
- ❌ 复杂的 I/O/CPU 分离代价参数
- ❌ MCV（最常见值）统计
- ❌ 直方图
- ❌ Join 策略选择（Nested Loop vs Hash vs Sort-Merge）
- ❌ 自适应统计更新

### 5.3 代码层面的调整

**保留的部分：**
- 节点的 `cost` 字段
- `cost()` 方法接口
- 计划缓存的代价淘汰策略

**简化的部分：**
- 代价计算逻辑（使用 SimpleCostCalculator）
- 统计信息结构（移除复杂字段）
- 选择性估计（使用均匀分布假设）

---

## 六、总结

### 6.1 核心结论

**不建议完全效仿 SQLite 删除代价体系，但建议采用"图数据库特化的轻量级代价模型"。**

理由：
1. 图数据库虽然 Join 少，但**遍历起点选择**需要代价估算
2. 项目已有代价框架，完全删除重构成本高
3. 简化的代价模型（行数 + 度数）足以满足核心需求
4. 维护成本低，收益明显

### 6.2 关键差异

| 数据库 | 优化重点 | 代价模型复杂度 |
|--------|---------|--------------|
| PostgreSQL | Join 优化 | 高（MCV + 直方图 + 多参数） |
| SQLite | 简单查询 | 极低（行数为主） |
| **GraphDB（推荐）** | 遍历起点 + 索引选择 | **低（行数 + 度数）** |

### 6.3 下一步行动

1. **确认方案**：评审本文档，确认采用轻量级代价模型方案
2. **设计统计信息**：定义 `TagStatistics` 和 `PropertyStatistics` 结构
3. **实现统计收集**：异步收集顶点数、平均度数等核心统计
4. **简化代价计算**：实现 `SimpleCostCalculator`
5. **集成到优化器**：在遍历起点选择和索引选择中使用简化代价模型

---

## 参考文档

- [cost_model_introduction_analysis.md](./cost_model_introduction_analysis.md) - 原始代价模型分析
- [SQLite Query Optimizer Overview](https://sqlite.org/optoverview.html)
- [GraphDB 当前代价模型分析](../cost/current_cost_model_analysis.md)
