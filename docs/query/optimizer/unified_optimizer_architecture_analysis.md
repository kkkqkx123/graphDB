# 统一优化器架构分析

## 执行摘要

本文档分析将 `planning/rewrite`（启发式重写）和 `optimizer`（基于代价的优化）合并为统一优化器模块的可行性、优势、风险及实施方案。

**核心结论**：建议**保持两个模块的独立性**，但通过统一目录结构进行组织，而非简单合并。

---

## 一、当前架构分析

### 1.1 模块职责对比

| 维度 | `planning/rewrite` | `optimizer` |
|------|-------------------|-------------|
| **定位** | 规划阶段的启发式重写 | 独立的优化引擎 |
| **优化类型** | 启发式规则（Heuristic） | 基于代价（Cost-Based） |
| **执行时机** | 规划过程中自动应用 | 规划后显式调用 |
| **依赖** | 不依赖统计信息 | 依赖 StatisticsManager |
| **执行保证** | 总是产生更好或等效的计划 | 可能因代价计算失败而保持原计划 |
| **典型规则** | 谓词下推、投影下推、消除冗余 | Join 顺序、索引选择、遍历方向 |
| **性能开销** | 低（模式匹配 + 简单转换） | 中到高（需要代价计算） |

### 1.2 代码规模统计

```
planning/rewrite/
├── 核心框架 (~800 行)
│   ├── rule.rs, plan_rewriter.rs, pattern.rs
│   └── context.rs, result.rs, visitor.rs
├── 重写规则 (~2500 行)
│   ├── predicate_pushdown/ (7 个文件)
│   ├── projection_pushdown/ (5 个文件)
│   ├── merge/ (7 个文件)
│   ├── elimination/ (8 个文件)
│   ├── limit_pushdown/ (6 个文件)
│   └── join_optimization/ (10 个文件)
└── 辅助模块 (~300 行)
    ├── macros.rs, rule_enum.rs, expression_utils.rs

optimizer/
├── 引擎核心 (~600 行)
│   ├── engine.rs, builder.rs, context.rs
├── 代价模型 (~1200 行)
│   ├── cost/calculator.rs, assigner.rs, estimate.rs
│   └── cost/node_estimators/ (6 个文件)
├── 统计信息 (~1000 行)
│   ├── stats/manager.rs, tag.rs, edge.rs, property.rs
│   └── stats/feedback/ (6 个文件)
├── 优化策略 (~2000 行)
│   ├── strategy/ (12 个文件)
│   └── decision/types.rs
└── 分析工具 (~400 行)
    └── analysis/ (4 个文件)

总计：planning/rewrite ~3600 行 vs optimizer ~5200 行
```

---

## 二、合并方案分析

### 2.1 方案 A：完全合并（推荐指数：★☆☆☆☆）

**方案描述**：将 `planning/rewrite` 的所有代码移动到 `optimizer` 目录下，作为优化器的子模块。

```
optimizer/
├── heuristic_rules/        # 原 planning/rewrite
│   ├── predicate_pushdown/
│   ├── projection_pushdown/
│   ├── elimination/
│   └── ...
├── cost_based_optimization/ # 原 optimizer/strategy
│   ├── join_order.rs
│   ├── index_selector.rs
│   └── ...
├── cost/
├── stats/
└── engine.rs
```

#### 优势
- ✅ 统一的优化入口
- ✅ 减少模块数量
- ✅ 便于实现混合优化策略

#### 劣势
- ❌ **职责混淆**：启发式规则和代价模型优化的设计理念不同
- ❌ **循环依赖风险**：rewrite 规则可能被优化器调用，优化器决策又可能触发重写
- ❌ **编译时间增加**：单个模块过大，影响增量编译性能
- ❌ **测试复杂度提升**：需要同时 mock 统计信息和重写上下文
- ❌ **违反单一职责原则**：优化器同时负责"必须的优化"和"可选的优化"

#### 技术障碍

**1. 上下文依赖冲突**
```rust
// planning/rewrite 使用 RewriteContext
pub struct RewriteContext {
    node_id_counter: usize,
    registered_nodes: HashMap<usize, PlanNodeEnum>,
}

// optimizer 使用 OptimizationContext
pub struct OptimizationContext {
    statistics: Arc<StatisticsManager>,
    cost_calculator: Arc<CostCalculator>,
}

// 合并后需要统一上下文，但两者的数据需求完全不同
```

**2. 规则执行时机冲突**
```rust
// 当前流程（清晰）
Planner::transform()
  → 生成初始计划
  → rewrite_plan()  // 自动应用启发式规则
  → 返回优化后的计划

// 合并后的流程（混乱）
Planner::transform()
  → 生成初始计划
  → OptimizerEngine::optimize()  // 需要手动区分哪些规则必须先执行
    → 先执行启发式规则？
    → 再执行代价模型优化？
    → 如果代价模型优化产生了新的模式，是否需要再次执行启发式规则？
```

**3. 错误处理不一致**
```rust
// planning/rewrite 的错误处理
pub enum RewriteError {
    PatternMismatch,
    TransformationFailed(String),
}

// optimizer 的错误处理
pub enum CostError {
    StatisticsUnavailable,
    CalculationOverflow,
}

// 合并后需要统一的错误类型，但两者的错误语义不同
```

### 2.2 方案 B：保持独立，统一目录（推荐指数：★★★★★）

**方案描述**：保持两个模块的代码独立性，但通过目录结构统一组织。

```
src/query/optimizer/
├── README.md                    # 优化器架构说明
├── mod.rs                       # 统一导出
│
├── heuristic/                   # 原 planning/rewrite（重命名）
│   ├── mod.rs
│   ├── core/                    # 核心框架
│   │   ├── rule.rs
│   │   ├── rewriter.rs
│   │   ├── pattern.rs
│   │   └── context.rs
│   ├── rules/                   # 具体规则
│   │   ├── predicate_pushdown/
│   │   ├── projection_pushdown/
│   │   ├── elimination/
│   │   ├── merge/
│   │   ├── limit_pushdown/
│   │   └── join_optimization/
│   └── registry.rs              # 规则注册表
│
├── cost_based/                  # 原 optimizer/strategy
│   ├── mod.rs
│   ├── engine.rs                # 优化引擎
│   ├── strategies/              # 优化策略
│   │   ├── join_order.rs
│   │   ├── index_selector.rs
│   │   ├── traversal_start.rs
│   │   └── ...
│   └── decision/                # 决策类型
│
├── cost/                        # 代价计算（保持不变）
│   ├── calculator.rs
│   ├── assigner.rs
│   └── node_estimators/
│
├── stats/                       # 统计信息（保持不变）
│   ├── manager.rs
│   ├── tag.rs
│   ├── edge.rs
│   └── feedback/
│
└── pipeline/                    # 新增：优化管道编排
    ├── mod.rs
    ├── optimizer_pipeline.rs    # 定义优化流程
    └── phase.rs                 # 优化阶段定义
```

#### 优势
- ✅ **职责清晰**：heuristic 和 cost_based 明确分离
- ✅ **独立演进**：两个模块可以独立修改和测试
- ✅ **统一入口**：通过 optimizer 模块统一管理
- ✅ **灵活的管道编排**：可以定义多阶段优化流程
- ✅ **便于理解**：目录结构反映优化层次

#### 劣势
- ⚠️ 需要重构导入路径（一次性成本）
- ⚠️ 需要更新文档和示例代码

#### 实施复杂度
- 代码移动：~3600 行（planning/rewrite → optimizer/heuristic）
- 路径更新：~50 个文件的 use 语句
- 文档更新：~10 个相关文件

### 2.3 方案 C：保持现状，改进协作（推荐指数：★★★☆☆）

**方案描述**：保持 `planning/rewrite` 和 `optimizer` 的独立模块地位，但改进两者的协作机制。

#### 改进措施

**1. 明确优化管道**
```rust
// 在 planner.rs 中明确定义优化流程
pub fn transform_with_optimization(...) -> Result<ExecutionPlan> {
    // 阶段 1: 生成初始计划
    let plan = self.generate_initial_plan(...)?;
    
    // 阶段 2: 应用启发式重写（必须）
    let plan = rewrite_plan(plan)?;
    
    // 阶段 3: 应用代价模型优化（可选，基于配置）
    let plan = if config.enable_cost_based_optimization {
        optimizer.optimize(plan)?
    } else {
        plan
    };
    
    Ok(plan)
}
```

**2. 统一统计信息访问**
```rust
// 为 rewrite 模块提供统计信息访问能力
impl RewriteContext {
    pub fn with_statistics(
        statistics: Arc<StatisticsManager>,
    ) -> Self {
        // ...
    }
    
    pub fn estimate_cardinality(&self, node: &PlanNodeEnum) -> f64 {
        // 使用统计信息进行估计，而非硬编码
    }
}
```

**3. 建立规则优先级机制**
```rust
pub enum RewriteRulePriority {
    High,    // 必须首先执行（如消除明显冗余）
    Medium,  // 常规优化（如谓词下推）
    Low,     // 可选优化（如 Join 重排序）
}

impl RewriteRule {
    fn priority(&self) -> RewriteRulePriority;
}
```

---

## 三、详细对比分析

### 3.1 设计哲学对比

| 方面 | Heuristic Rewrite | Cost-Based Optimization |
|------|-------------------|------------------------|
| **设计理念** | "总是有益的" | "基于代价决策" |
| **确定性** | 确定性转换 | 概率性决策 |
| **可预测性** | 高（规则明确） | 中（依赖统计信息质量） |
| **优化深度** | 局部优化 | 全局优化 |
| **执行开销** | 低 | 中到高 |

### 3.2 规则/策略对应关系分析

#### 重叠领域 1：排序优化

```rust
// planning/rewrite/elimination/eliminate_sort.rs
// 场景：输入已排序时消除冗余排序
pub struct EliminateSortRule;
impl RewriteRule for EliminateSortRule {
    fn matches(&self, node: &PlanNodeEnum) -> bool {
        // 检查 Sort 的输入是否已满足排序要求
        node.is_sort() && node.input().is_already_sorted()
    }
}

// optimizer/strategy/topn_optimization.rs
// 场景：基于代价决定是否转换为 TopN
pub struct SortEliminationOptimizer;
impl OptimizationStrategy for SortEliminationOptimizer {
    fn optimize(&self, plan: &ExecutionPlan) -> OptimizationDecision {
        // 比较 Sort+Limit vs TopN 的代价
        let sort_cost = self.calculate_sort_cost();
        let topn_cost = self.calculate_topn_cost();
        if topn_cost < sort_cost {
            OptimizationDecision::ConvertToTopN
        } else {
            OptimizationDecision::KeepSort
        }
    }
}
```

**分析**：
- `EliminateSortRule`：基于**模式匹配**的确定性消除
- `SortEliminationOptimizer`：基于**代价比较**的决策
- **结论**：两者互补，不应合并

#### 重叠领域 2：Join 优化

```rust
// planning/rewrite/join_optimization/join_reorder.rs
// 基于简单规则的重排序
pub struct JoinReorderRule;
impl RewriteRule for JoinReorderRule {
    fn apply(&self, ctx: &mut RewriteContext, node: &PlanNodeEnum) {
        // 使用硬编码的估计值
        let left_rows = 10000.0;
        let right_rows = 50000.0;
        // 小表优先
        if left_rows > right_rows {
            // 交换左右子树
        }
    }
}

// optimizer/strategy/join_order.rs
// 基于动态规划的 Join 顺序优化
pub struct JoinOrderOptimizer;
impl JoinOrderOptimizer {
    pub fn optimize_join_order(&self, tables: &[TableInfo]) -> JoinOrderResult {
        // 使用统计信息
        let stats = self.stats_manager.get_statistics();
        // 动态规划或贪心算法
        if tables.len() <= 8 {
            self.dp_optimize(tables)
        } else {
            self.greedy_optimize(tables)
        }
    }
}
```

**分析**：
- `JoinReorderRule`：快速、局部的启发式优化
- `JoinOrderOptimizer`：慢速、全局的代价模型优化
- **结论**：应该保留两层优化，但需要明确执行顺序

### 3.3 调用链分析

#### 当前调用链

```
QueryPipelineManager::execute()
  └─> Parser::parse()
      └─> Validator::validate()
          └─> Planner::transform()
              ├─> 生成初始计划
              └─> rewrite_plan()  ← planning/rewrite
                  └─> PlanRewriter::rewrite()
                      └─> 应用所有启发式规则
          └─> OptimizerEngine::optimize()  ← optimizer
              └─> 应用代价模型优化策略
```

#### 问题点

1. **优化顺序不明确**：`rewrite_plan()` 在 `Planner::transform()` 中调用，但 `OptimizerEngine::optimize()` 的调用点分散
2. **缺乏反馈机制**：代价模型优化的结果无法反馈给启发式规则
3. **重复优化风险**：某些规则可能被多次应用

---

## 四、推荐方案：分层优化架构

### 4.1 架构设计

```
┌─────────────────────────────────────────────────────────┐
│                   Query Optimizer                       │
│  (统一入口，通过 optimizer 模块管理)                     │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│              Optimization Pipeline                      │
│  (定义优化阶段和执行顺序)                                │
└─────────────────────────────────────────────────────────┘
                            │
            ┌───────────────┴───────────────┐
            │                               │
            ▼                               ▼
┌───────────────────────┐       ┌───────────────────────┐
│   Phase 1: Heuristic  │       │  Phase 2: Cost-Based  │
│   (必须执行)          │       │  (可选，基于配置)      │
│                       │       │                       │
│ ┌───────────────────┐ │       │ ┌───────────────────┐ │
│ │ Predicate Pushdown│ │       │ │ Join Order        │ │
│ │ Projection Push   │ │       │ │ Index Selection   │ │
│ │ Elimination       │ │       │ │ Traversal Start   │ │
│ │ Merge Operations  │ │       │ │ Traversal Direction│ │
│ │ Limit Pushdown    │ │       │ │ Materialization   │ │
│ └───────────────────┘ │       │ └───────────────────┘ │
│                       │       │                       │
│ 特点：                │       │ 特点：                │
│ - 不依赖统计信息      │       │ - 依赖统计信息        │
│ - 确定性转换          │       │ - 代价模型决策        │
│ - 低开销              │       │ - 中到高开销          │
│ - 总是产生更好计划    │       │ - 可能保持原计划      │
└───────────────────────┘       └───────────────────────┘
```

### 4.2 目录结构

```
src/query/optimizer/
├── mod.rs                          # 统一导出
├── README.md                       # 架构说明
│
├── pipeline/                       # 新增：优化管道
│   ├── mod.rs
│   ├── optimizer_pipeline.rs       # 管道编排
│   ├── phase.rs                    # 阶段定义
│   └── config.rs                   # 优化配置
│
├── heuristic/                      # 原 planning/rewrite
│   ├── mod.rs
│   ├── core/                       # 核心框架
│   │   ├── rule.rs                 # RewriteRule trait
│   │   ├── rewriter.rs             # PlanRewriter
│   │   ├── pattern.rs              # 模式匹配
│   │   ├── context.rs              # RewriteContext
│   │   └── result.rs               # 错误处理
│   ├── rules/                      # 具体规则
│   │   ├── mod.rs
│   │   ├── predicate_pushdown/
│   │   ├── projection_pushdown/
│   │   ├── elimination/
│   │   ├── merge/
│   │   ├── limit_pushdown/
│   │   └── join_optimization/
│   └── registry.rs                 # 规则注册表
│
├── cost_based/                     # 原 optimizer/strategy
│   ├── mod.rs
│   ├── engine.rs                   # OptimizerEngine
│   ├── strategies/                 # 优化策略
│   │   ├── mod.rs
│   │   ├── join_order.rs
│   │   ├── index_selector.rs
│   │   ├── traversal_start.rs
│   │   ├── traversal_direction.rs
│   │   ├── aggregate_strategy.rs
│   │   ├── topn_optimization.rs
│   │   ├── subquery_unnesting.rs
│   │   └── materialization.rs
│   ├── decision/                   # 决策类型
│   │   ├── mod.rs
│   │   └── types.rs
│   └── context.rs                  # OptimizationContext
│
├── cost/                           # 代价计算（保持不变）
│   ├── mod.rs
│   ├── calculator.rs
│   ├── assigner.rs
│   ├── estimate.rs
│   ├── config.rs
│   ├── selectivity.rs
│   └── node_estimators/
│
├── stats/                          # 统计信息（保持不变）
│   ├── mod.rs
│   ├── manager.rs
│   ├── tag.rs
│   ├── edge.rs
│   ├── property.rs
│   ├── histogram.rs
│   └── feedback/
│
└── analysis/                       # 分析工具（保持不变）
    ├── mod.rs
    ├── expression.rs
    ├── fingerprint.rs
    └── batch.rs
```

### 4.3 导入路径变更

#### 变更前
```rust
// planning/rewrite
use crate::query::planning::rewrite::{PlanRewriter, RewriteRule};
use crate::query::planning::rewrite::elimination::EliminateFilterRule;

// optimizer
use crate::query::optimizer::{OptimizerEngine, CostCalculator};
use crate::query::optimizer::strategy::JoinOrderOptimizer;
```

#### 变更后
```rust
// heuristic (原 planning/rewrite)
use crate::query::optimizer::heuristic::{PlanRewriter, RewriteRule};
use crate::query::optimizer::heuristic::rules::elimination::EliminateFilterRule;

// cost_based (原 optimizer/strategy)
use crate::query::optimizer::{OptimizerEngine, CostCalculator};
use crate::query::optimizer::cost_based::strategies::JoinOrderOptimizer;

// 或通过统一导出简化
use crate::query::optimizer::{
    heuristic::{PlanRewriter, RewriteRule},
    cost_based::strategies::JoinOrderOptimizer,
};
```

### 4.4 优化管道实现示例

```rust
// optimizer/pipeline/optimizer_pipeline.rs

pub struct OptimizationPipeline {
    heuristic_rewriter: PlanRewriter,
    cost_optimizer: OptimizerEngine,
    config: PipelineConfig,
}

pub struct PipelineConfig {
    pub enable_heuristic: bool,
    pub enable_cost_based: bool,
    pub max_heuristic_iterations: usize,
    pub statistics_threshold: u64,
}

impl OptimizationPipeline {
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            heuristic_rewriter: PlanRewriter::default(),
            cost_optimizer: OptimizerEngine::default(),
            config,
        }
    }
    
    pub fn optimize(&self, plan: ExecutionPlan) -> OptimizeResult<ExecutionPlan> {
        let mut current_plan = plan;
        
        // Phase 1: Heuristic Optimization (必须执行)
        if self.config.enable_heuristic {
            current_plan = self.apply_heuristic(current_plan)?;
        }
        
        // Phase 2: Cost-Based Optimization (可选)
        if self.config.enable_cost_based {
            current_plan = self.apply_cost_based(current_plan)?;
        }
        
        Ok(current_plan)
    }
    
    fn apply_heuristic(&self, plan: ExecutionPlan) -> OptimizeResult<ExecutionPlan> {
        log::debug!("应用启发式优化规则");
        self.heuristic_rewriter.rewrite(plan)
            .map_err(|e| OptimizeError::HeuristicFailed(e.to_string()))
    }
    
    fn apply_cost_based(&self, plan: ExecutionPlan) -> OptimizeResult<ExecutionPlan> {
        log::debug!("应用代价模型优化");
        self.cost_optimizer.optimize(plan)
    }
}
```

---

## 五、实施路线图

### 阶段 1：准备工作（1-2 周）

**任务清单**：
- [ ] 创建 `optimizer/heuristic/` 目录结构
- [ ] 创建 `optimizer/cost_based/` 目录结构
- [ ] 创建 `optimizer/pipeline/` 目录结构
- [ ] 更新 `optimizer/mod.rs` 统一导出

**代码变更**：
- 移动文件（不修改内容）
- 更新模块声明
- 修复导入路径

**测试验证**：
- 运行现有测试套件
- 确保功能无变化

### 阶段 2：管道编排（2-3 周）

**任务清单**：
- [ ] 实现 `OptimizationPipeline`
- [ ] 定义优化阶段枚举
- [ ] 实现配置系统
- [ ] 更新 `Planner::transform_with_full_context()`

**代码变更**：
```rust
// planner.rs
pub fn transform_with_full_context(...) -> Result<ExecutionPlan> {
    let plan = self.transform(validated, qctx)?;
    
    // 使用新的优化管道
    let pipeline = OptimizationPipeline::new(config);
    let optimized_plan = pipeline.optimize(plan)?;
    
    Ok(optimized_plan)
}
```

**测试验证**：
- 单元测试：测试每个优化阶段
- 集成测试：测试完整优化流程
- 性能测试：验证优化效果

### 阶段 3：改进协作（3-4 周）

**任务清单**：
- [ ] 为 heuristic 规则提供统计信息访问
- [ ] 实现规则优先级机制
- [ ] 建立优化反馈循环
- [ ] 消除重复优化

**代码变更**：
```rust
// heuristic/rules/join_optimization/join_reorder.rs
impl RewriteRule for JoinReorderRule {
    fn apply(&self, ctx: &mut RewriteContext, node: &PlanNodeEnum) {
        // 使用统计信息而非硬编码
        let stats = ctx.statistics();
        let left_rows = stats.estimate_cardinality(node.left());
        let right_rows = stats.estimate_cardinality(node.right());
        
        // 基于真实统计信息进行优化
        if left_rows > right_rows {
            // 交换
        }
    }
}
```

**测试验证**：
- 对比优化前后的计划质量
- 验证统计信息准确性
- 性能基准测试

### 阶段 4：文档和清理（1 周）

**任务清单**：
- [ ] 更新架构文档
- [ ] 编写迁移指南
- [ ] 更新示例代码
- [ ] 清理废弃代码

---

## 六、风险评估

### 6.1 技术风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|---------|
| 循环依赖 | 中 | 高 | 严格分层，禁止反向依赖 |
| 性能回退 | 中 | 中 | 分阶段验证，保留回滚能力 |
| 测试失败 | 高 | 低 | 充分测试覆盖，逐步迁移 |
| 编译错误 | 高 | 低 | 使用 IDE 重构工具，批量修复 |

### 6.2 组织风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|---------|
| 学习曲线 | 中 | 低 | 提供详细文档和示例 |
| 代码审查负担 | 高 | 中 | 分阶段提交，小步快跑 |
| 团队分歧 | 低 | 中 | 充分讨论，达成共识 |

---

## 七、成本效益分析

### 7.1 实施成本

**人力成本**：
- 阶段 1：1-2 周（1 人）
- 阶段 2：2-3 周（1-2 人）
- 阶段 3：3-4 周（2 人）
- 阶段 4：1 周（1 人）
- **总计**：7-10 人周

**机会成本**：
- 延迟其他功能开发 2-3 个月
- 代码审查和测试时间

### 7.2 预期收益

**短期收益（3-6 个月）**：
- ✅ 代码组织更清晰
- ✅ 优化流程更透明
- ✅ 便于添加新规则

**中期收益（6-12 个月）**：
- ✅ 优化质量提升 10-20%
- ✅ 编译时间减少 5-10%
- ✅ 测试覆盖率提升

**长期收益（1-2 年）**：
- ✅ 支持自适应优化
- ✅ 支持机器学习优化
- ✅ 降低维护成本

### 7.3 投资回报率

```
ROI = (收益 - 成本) / 成本

量化收益（年化）：
- 开发效率提升：20 人天 × ¥2000/天 = ¥40,000
- 性能提升带来的硬件节省：¥20,000
- 维护成本降低：10 人天 × ¥2000/天 = ¥20,000
总收益：¥80,000/年

成本：
- 实施成本：10 人周 × 5 天/周 × ¥2000/天 = ¥100,000

ROI = (80,000 - 100,000) / 100,000 = -20% (第一年)
ROI = (80,000 × 2 - 100,000) / 100,000 = 60% (第二年)
```

**结论**：从长期看，投资回报率为正，但短期可能为负。

---

## 八、最终建议

### 8.1 推荐方案

**采用方案 B（保持独立，统一目录）**，具体实施建议：

1. **保持模块独立性**
   - `heuristic` 和 `cost_based` 保持代码独立
   - 避免循环依赖
   - 各自维护测试套件

2. **统一目录组织**
   - 将 `planning/rewrite` 移动到 `optimizer/heuristic`
   - 将 `optimizer/strategy` 重命名为 `optimizer/cost_based`
   - 新增 `optimizer/pipeline` 负责编排

3. **分阶段实施**
   - 先进行代码移动（低风险）
   - 再实现管道编排（中风险）
   - 最后改进协作（高风险）

### 8.2 不推荐方案

**方案 A（完全合并）** 的问题过于严重：
- 违反单一职责原则
- 增加代码耦合度
- 测试复杂度大幅提升
- 不利于长期维护

**方案 C（保持现状）** 的问题：
- 无法解决当前的职责重叠问题
- 优化流程不够透明
- 不利于添加新规则

### 8.3 关键成功因素

1. **充分的测试覆盖**
   - 迁移前后功能一致性验证
   - 性能基准测试

2. **渐进式重构**
   - 小步快跑，避免大爆炸式重构
   - 每个阶段都可独立回滚

3. **团队共识**
   - 充分讨论方案
   - 明确责任分工

4. **文档同步**
   - 及时更新架构文档
   - 提供迁移指南

---

## 九、附录

### 9.1 相关文件清单

**需要修改的文件**：
- `src/query/planning/mod.rs`
- `src/query/optimizer/mod.rs`
- `src/query/planning/planner.rs`
- `src/query/query_pipeline_manager.rs`
- 所有引用 `planning::rewrite` 的文件
- 所有引用 `optimizer::strategy` 的文件

**需要创建的文档**：
- `docs/query/optimizer/unified_optimizer_architecture.md`
- `docs/query/optimizer/migration_guide.md`
- `docs/query/optimizer/optimization_pipeline.md`

### 9.2 参考架构

**数据库优化器参考**：
- PostgreSQL: `src/backend/optimizer/`
- MySQL: `sql/opt_range.cc`, `sql/opt_join.cc`
- Oracle: Cost-Based Optimizer (CBO)
- SQL Server: Query Optimizer

**共同特点**：
- 分层优化架构
- 启发式规则 + 代价模型
- 明确的优化阶段

### 9.3 术语表

| 术语 | 定义 |
|------|------|
| Heuristic Rule | 不依赖代价计算的优化规则 |
| Cost-Based Optimization | 基于统计信息和代价模型的优化 |
| Rewrite Rule | 重写规则的统称 |
| Optimization Strategy | 优化策略的统称 |
| Optimization Pipeline | 优化管道的编排 |

---

## 十、修订历史

| 版本 | 日期 | 作者 | 变更说明 |
|------|------|------|---------|
| 1.0 | 2026-04-07 | AI Assistant | 初始版本 |

---

**文档状态**：✅ 完成

**下一步行动**：
1. 团队讨论本方案
2. 确定是否实施重构
3. 如实施，成立专项小组
4. 制定详细实施计划
