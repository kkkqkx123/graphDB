# 优化器扩展功能分析

## 1. 现有架构分析

### 1.1 核心组件

当前优化器已具备以下基础设施：

```
src/query/optimizer/
├── core/
│   └── cost.rs              # 代价模型（已简化为 f64）
├── plan/
│   ├── context.rs           # OptContext - 优化上下文
│   ├── group.rs             # OptGroup - 优化组管理
│   └── node.rs              # OptGroupNode - 组节点
├── rules/
│   ├── join/
│   │   └── join_optimization.rs  # 连接优化规则
│   ├── predicate_pushdown/       # 谓词下推规则
│   ├── limit_pushdown/           # Limit下推规则
│   ├── elimination/              # 消除规则
│   └── ...
└── engine/
    └── optimizer.rs         # 优化器引擎
```

### 1.2 现有功能

| 功能 | 状态 | 说明 |
|------|------|------|
| 规则框架 | ✅ 完整 | OptRule trait + 规则注册机制 |
| 行数估算 | ✅ 基础 | estimate_row_count() 硬编码默认值 |
| 代价估算 | ✅ 预留 | estimate_subtree_cost() 未使用 |
| 连接优化 | ✅ 简单 | 基于阈值的决策 |
| 规则集 | ✅ 丰富 | 谓词下推、Limit下推、投影下推等 |

### 1.3 关键代码

**连接优化规则** (`src/query/optimizer/rules/join/join_optimization.rs`):

```rust
impl JoinOptimizationRule {
    /// 评估最优的连接策略
    fn evaluate_join_strategy(&self, ctx: &OptContext, left_node: &PlanNodeEnum, right_node: &PlanNodeEnum) 
        -> Result<JoinStrategy, OptimizerError> {
        let left_rows = self.estimate_row_count(ctx, left_node)?;
        let right_rows = self.estimate_row_count(ctx, right_node)?;
        
        // 基于阈值决策
        const SMALL_TABLE_THRESHOLD: u64 = 1000;
        if left_rows < SMALL_TABLE_THRESHOLD || right_rows < SMALL_TABLE_THRESHOLD {
            return Ok(JoinStrategy::HashJoin);
        }
        // ...
    }
    
    /// 估算子树的行数（硬编码）
    fn estimate_row_count(&self, _ctx: &OptContext, node: &PlanNodeEnum) -> Result<u64> {
        let rows = match node {
            PlanNodeEnum::ScanVertices(_) => 10000,
            PlanNodeEnum::IndexScan(_) => 100,
            // ...
        };
        Ok(rows)
    }
}
```

---

## 2. 可扩展功能分析

### 2.1 功能列表与评估

| 功能 | 收益 | 实现复杂度 | 推荐优先级 |
|------|------|-----------|-----------|
| **统计信息缓存** | 避免重复查询，提升优化速度 | 低 | ⭐⭐⭐⭐⭐ |
| **运行时行数反馈** | 逐步校准估算，提升准确性 | 低 | ⭐⭐⭐⭐⭐ |
| **选择性估算改进** | 更准确的行数估算 | 中 | ⭐⭐⭐⭐ |
| **索引选择性提示** | 更智能的索引选择 | 中 | ⭐⭐⭐⭐ |
| **连接顺序启发式** | 多表连接优化 | 中 | ⭐⭐⭐ |
| **代价驱动的规则启用** | 避免有害转换 | 中高 | ⭐⭐ |
| **计划缓存** | 跳过重复优化 | 中 | ⭐⭐ |

### 2.2 高优先级功能详解

#### 2.2.1 统计信息缓存

**目标**：在 `OptContext` 中缓存表级统计信息，避免重复查询存储层。

**实现方案**：

```rust
// 在 OptContext 中添加统计信息缓存
pub struct OptContext {
    // ... 现有字段
    
    /// 统计信息缓存（表名 -> TableStats）
    stats_cache: RefCell<HashMap<String, TableStats>>,
}

impl OptContext {
    /// 获取表统计信息（带缓存）
    pub fn get_table_stats(&self, table_name: &str) -> Option<TableStats> {
        // 1. 先查缓存
        if let Some(stats) = self.stats_cache.borrow().get(table_name) {
            return Some(stats.clone());
        }
        
        // 2. 从存储层获取（未来实现）
        // let stats = self.storage.get_table_stats(table_name)?;
        
        // 3. 存入缓存
        // self.stats_cache.borrow_mut().insert(table_name.to_string(), stats.clone());
        
        None
    }
    
    /// 手动设置统计信息（用于测试或手动优化）
    pub fn set_table_stats(&self, table_name: &str, stats: TableStats) {
        self.stats_cache.borrow_mut().insert(table_name.to_string(), stats);
    }
    
    /// 清除统计信息缓存
    pub fn clear_stats_cache(&self) {
        self.stats_cache.borrow_mut().clear();
    }
}
```

**收益**：
- 避免重复查询存储层
- 为后续所有统计相关功能打基础
- 支持手动设置统计信息进行测试

---

#### 2.2.2 运行时行数反馈

**目标**：使用实际执行结果的行数来校准估算，逐步提升准确性。

**实现方案**：

```rust
// 在 OptContext 中添加执行反馈
pub struct OptContext {
    // ... 现有字段
    
    /// 实际行数反馈（节点ID -> 实际行数）
    actual_row_counts: RefCell<HashMap<usize, u64>>,
    
    /// 反馈统计（用于计算平均误差）
    feedback_stats: RefCell<FeedbackStats>,
}

#[derive(Debug, Clone, Default)]
pub struct FeedbackStats {
    pub total_estimates: u64,
    pub total_actual: u64,
    pub sample_count: u64,
}

impl OptContext {
    /// 执行后更新实际行数
    pub fn update_actual_row_count(&self, node_id: usize, actual_rows: u64) {
        self.actual_row_counts.borrow_mut().insert(node_id, actual_rows);
        
        // 更新统计
        let mut stats = self.feedback_stats.borrow_mut();
        stats.sample_count += 1;
    }
    
    /// 获取实际行数（如果有反馈）
    pub fn get_actual_row_count(&self, node_id: usize) -> Option<u64> {
        self.actual_row_counts.borrow().get(&node_id).copied()
    }
    
    /// 计算估算误差率
    pub fn get_estimate_error_rate(&self) -> f64 {
        let stats = self.feedback_stats.borrow();
        if stats.sample_count == 0 {
            return 0.0;
        }
        
        let avg_estimate = stats.total_estimates as f64 / stats.sample_count as f64;
        let avg_actual = stats.total_actual as f64 / stats.sample_count as f64;
        
        if avg_actual == 0.0 {
            return 0.0;
        }
        
        ((avg_estimate - avg_actual).abs() / avg_actual).min(1.0)
    }
    
    /// 获取校准后的行数估算
    pub fn get_calibrated_row_estimate(&self, node_id: usize, estimated_rows: u64) -> u64 {
        // 如果有实际值，优先使用
        if let Some(actual) = self.get_actual_row_count(node_id) {
            return actual;
        }
        
        // 否则根据历史误差率校准
        let error_rate = self.get_estimate_error_rate();
        if error_rate > 0.0 {
            // 如果历史估算偏高，调低估算
            (estimated_rows as f64 * (1.0 - error_rate)) as u64
        } else {
            estimated_rows
        }
    }
}
```

**收益**：
- 逐步校准行数估算
- 提升重复查询的优化质量
- 支持估算准确性监控

---

## 3. 实施路线图

### 3.1 第一阶段：基础设施（立即实施）

1. **统计信息缓存**
   - 修改 `OptContext` 添加 `stats_cache` 字段
   - 实现 `get_table_stats()` 和 `set_table_stats()` 方法
   - 预计工作量：1-2 小时

2. **运行时行数反馈**
   - 添加 `actual_row_counts` 和 `feedback_stats` 字段
   - 实现 `update_actual_row_count()` 和 `get_calibrated_row_estimate()` 方法
   - 预计工作量：2-3 小时

### 3.2 第二阶段：估算改进（短期）

3. **选择性估算改进**
   - 扩展 `estimate_selectivity()` 方法
   - 添加对唯一列、范围条件的处理
   - 预计工作量：1-2 天

4. **索引选择性提示**
   - 修改索引扫描规则
   - 基于选择性决定是否使用索引
   - 预计工作量：1-2 天

### 3.3 第三阶段：高级功能（中期）

5. **连接顺序启发式**
   - 实现多表连接的贪心排序
   - 预计工作量：2-3 天

6. **计划缓存**
   - 实现查询计划缓存机制
   - 预计工作量：3-5 天

---

## 4. 与 Nebula-Graph 对比

| 功能 | Nebula-Graph | 当前项目（建议） |
|------|-------------|----------------|
| 统计信息 | Meta Service 存储 | 本地缓存 + 运行时反馈 |
| 代价模型 | 单一代价值 | 简化的单一代价值 |
| 行数估算 | 表级统计 | 表级 + 运行时反馈校准 |
| 选择性估算 | 无 | 简单启发式 |
| 连接优化 | RBO 为主 | RBO + 简单行数估算 |
| 计划缓存 | 无 | 可考虑实现 |

**结论**：当前项目建议采用更轻量级但实用的策略，避免 Nebula-Graph 中 CBO 实现不完整的问题。

---

## 5. 关键设计决策

### 5.1 为什么优先选择统计信息缓存和运行时反馈？

1. **实现简单**：仅需在 `OptContext` 中添加字段和方法，不修改现有规则逻辑
2. **收益明显**：立即提升优化器性能和准确性
3. **基础性强**：为后续更复杂的功能奠定基础
4. **风险低**：不影响现有功能，可逐步启用

### 5.2 为什么不立即实现完整 CBO？

1. **单节点架构**：不需要考虑分布式统计和网络代价
2. **图数据库特性**：查询模式相对固定（点查、邻域遍历），简单估算已能满足大部分场景
3. **避免过度设计**：在缺乏真实性能数据的情况下，简单方案更可靠
4. **渐进式改进**：先让优化器"能用"，再让它"好用"

---

## 6. 代码实现参考

### 6.1 修改 OptContext

文件：`src/query/optimizer/plan/context.rs`

```rust
use crate::query::optimizer::core::cost::{TableStats, FeedbackStats};

pub struct OptContext {
    // ... 现有字段
    
    /// 统计信息缓存
    stats_cache: RefCell<HashMap<String, TableStats>>,
    
    /// 实际行数反馈
    actual_row_counts: RefCell<HashMap<usize, u64>>,
    
    /// 反馈统计
    feedback_stats: RefCell<FeedbackStats>,
}

impl OptContext {
    pub fn new(query_context: QueryContext) -> Self {
        Self {
            // ... 现有初始化
            stats_cache: RefCell::new(HashMap::new()),
            actual_row_counts: RefCell::new(HashMap::new()),
            feedback_stats: RefCell::new(FeedbackStats::default()),
        }
    }
    
    // ... 新增方法（见上文）
}
```

### 6.2 修改 Cost 模块

文件：`src/query/optimizer/core/cost.rs`

```rust
/// 反馈统计
#[derive(Debug, Clone, Default)]
pub struct FeedbackStats {
    pub total_estimates: u64,
    pub total_actual: u64,
    pub sample_count: u64,
}

impl FeedbackStats {
    pub fn record(&mut self, estimated: u64, actual: u64) {
        self.total_estimates += estimated;
        self.total_actual += actual;
        self.sample_count += 1;
    }
    
    pub fn average_error_rate(&self) -> f64 {
        if self.sample_count == 0 {
            return 0.0;
        }
        
        let avg_estimate = self.total_estimates as f64 / self.sample_count as f64;
        let avg_actual = self.total_actual as f64 / self.sample_count as f64;
        
        if avg_actual == 0.0 {
            return 0.0;
        }
        
        ((avg_estimate - avg_actual).abs() / avg_actual).min(1.0)
    }
}
```

---

## 7. 总结

基于现有架构，**统计信息缓存**和**运行时行数反馈**是最值得优先实现的功能：

- **实现简单**：仅需在 `OptContext` 中添加字段和方法
- **收益明显**：立即提升优化器性能和准确性
- **基础性强**：为后续更复杂的功能奠定基础
- **风险低**：不影响现有功能，可逐步启用

这两个功能可以在不修改现有规则逻辑的情况下，显著提升优化器的实用性和准确性，是性价比最高的改进方案。
