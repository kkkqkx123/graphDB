# 代价体系与查询阶段集成设计

## 1. 当前查询处理流程分析

### 1.1 查询处理流水线

```
查询字符串 → Parser → AST → Validator → Planner → ExecutionPlan → Optimizer → 执行
```

### 1.2 各阶段关键类型

| 阶段 | 主要类型 | 职责 |
|------|----------|------|
| **Parser** | `AST`, `Statement` | 语法解析，生成抽象语法树 |
| **Validator** | `ValidationContext` | 语义验证，类型检查 |
| **Planner** | `ExecutionPlan`, `PlanNodeEnum` | 生成执行计划 |
| **Optimizer** | `OptContext`, `OptGroup` | 基于代价的优化 |
| **Executor** | `ExecutionResult` | 执行计划 |

### 1.3 当前代价相关实现

#### PlanNode trait（已有基础）

```rust
pub trait PlanNode {
    fn cost(&self) -> f64;  // 只有单个 f64 值
    // ...
}
```

**问题**：代价信息过于简单，只有单个 f64 值，缺少行数估计、宽度估计等关键信息。

#### ExecutionPlan 结构

```rust
pub struct ExecutionPlan {
    pub root: Option<PlanNodeEnum>,
    pub id: i64,
    pub optimize_time_in_us: u64,
    pub format: String,
    // 缺少：total_cost, estimated_rows, plan_properties
}
```

**问题**：计划级别缺少代价信息字段。

#### QueryContext 结构

```rust
pub struct QueryContext {
    schema_manager: Option<Arc<dyn SchemaManager>>,
    index_metadata_manager: Option<Arc<dyn IndexMetadataManager>>,
    storage_client: Option<Arc<dyn StorageClient>>,
    // 缺少: statistics_provider
}
```

**问题**：没有集成统计信息提供者，无法获取表/列统计信息。

## 2. 代价体系集成方案

### 2.1 集成原则

1. **Planner 阶段集成代价计算**：Planner 生成 `ExecutionPlan` 时计算初始代价
2. **统计信息通过 QueryContext 传递**：统一访问存储层统计信息
3. **Optimizer 基于代价进行计划变换**：使用代价信息选择最优计划
4. **与现有架构兼容**：逐步迁移，保持向后兼容

### 2.2 扩展 PlanNode trait

```rust
pub trait PlanNode {
    /// 获取节点代价（已有）
    fn cost(&self) -> f64;
    
    /// 获取估计输出行数（新增）
    fn estimated_rows(&self) -> u64;
    
    /// 获取估计输出行宽度（新增）
    fn estimated_width(&self) -> u64;
    
    /// 获取完整代价结构（新增）
    fn plan_node_cost(&self) -> PlanNodeCost;
    
    /// 设置代价（新增）
    fn set_cost(&mut self, cost: PlanNodeCost);
    
    // ... 其他方法
}
```

### 2.3 扩展 ExecutionPlan 结构

```rust
pub struct ExecutionPlan {
    pub root: Option<PlanNodeEnum>,
    pub id: i64,
    pub optimize_time_in_us: u64,
    pub format: String,
    
    // 新增：代价信息
    /// 计划总代价
    pub total_cost: f64,
    /// 估计输出行数
    pub estimated_rows: u64,
    /// 计划属性（包含代价、选择性等）
    pub plan_properties: PlanNodeProperties,
}
```

### 2.4 在 QueryContext 中集成统计信息

```rust
pub struct QueryContext {
    // ... 现有字段
    
    /// 统计信息提供者（新增）
    statistics_provider: Option<Arc<dyn StatisticsProvider>>,
}

impl QueryContext {
    /// 获取统计信息提供者
    pub fn statistics_provider(&self) -> Option<Arc<dyn StatisticsProvider>> {
        self.statistics_provider.clone()
    }
    
    /// 设置统计信息提供者
    pub fn set_statistics_provider(&mut self, provider: Arc<dyn StatisticsProvider>) {
        self.statistics_provider = Some(provider);
    }
}
```

### 2.5 创建代价计算上下文

```rust
/// 代价计算上下文
/// 
/// 封装代价计算所需的所有依赖
pub struct CostCalculationContext<'a> {
    /// 统计信息提供者
    pub statistics: &'a dyn StatisticsProvider,
    /// 代价模型配置
    pub config: &'a CostModelConfig,
    /// 查询上下文
    pub query_context: &'a QueryContext,
}

impl<'a> CostCalculationContext<'a> {
    pub fn new(
        statistics: &'a dyn StatisticsProvider,
        config: &'a CostModelConfig,
        query_context: &'a QueryContext,
    ) -> Self {
        Self {
            statistics,
            config,
            query_context,
        }
    }
}
```

### 2.6 在 Planner 中集成代价计算

```rust
impl PlannerEnum {
    pub fn transform_with_cost(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
        cost_ctx: &CostCalculationContext,
    ) -> Result<SubPlan, PlannerError> {
        // 1. 生成基础计划
        let mut sub_plan = self.transform(stmt, qctx)?;
        
        // 2. 计算计划代价
        if let Some(ref mut root) = sub_plan.root {
            Self::calculate_plan_cost(root, cost_ctx)?;
        }
        
        Ok(sub_plan)
    }
    
    /// 递归计算计划代价
    fn calculate_plan_cost(
        node: &mut PlanNodeEnum,
        ctx: &CostCalculationContext,
    ) -> Result<PlanNodeCost, PlannerError> {
        // 递归计算子节点代价
        let child_costs: Vec<PlanNodeCost> = node
            .children_mut()
            .iter_mut()
            .map(|child| Self::calculate_plan_cost(child, ctx))
            .collect::<Result<Vec<_>, _>>()?;
        
        // 根据节点类型计算代价
        let cost = match node {
            PlanNodeEnum::ScanVertices(scan) => {
                let stats = ctx.statistics
                    .get_table_stats(scan.table_name())
                    .ok_or_else(|| PlannerError::StatisticsNotFound(scan.table_name().to_string()))?;
                CostCalculator::calculate_scan_cost(stats, ctx.config)
            }
            PlanNodeEnum::Filter(filter) => {
                let input_cost = child_costs.first()
                    .ok_or(PlannerError::InvalidPlan("Filter 节点缺少输入".to_string()))?;
                CostCalculator::calculate_filter_cost(input_cost, &filter.condition(), ctx.config)
            }
            PlanNodeEnum::HashJoin(join) => {
                let left_cost = child_costs.get(0)
                    .ok_or(PlannerError::InvalidPlan("HashJoin 缺少左输入".to_string()))?;
                let right_cost = child_costs.get(1)
                    .ok_or(PlannerError::InvalidPlan("HashJoin 缺少右输入".to_string()))?;
                CostCalculator::calculate_hash_join_cost(left_cost, right_cost, ctx.config)
            }
            // ... 其他节点类型
            _ => PlanNodeCost::default(),
        };
        
        // 设置节点代价
        node.set_cost(cost.clone());
        
        Ok(cost)
    }
}
```

## 3. 优化规则迁移分析

### 3.1 当前优化规则结构

```
src/query/optimizer/rules/
├── predicate_pushdown/    # 谓词下推规则
├── merge/                 # 操作合并规则
├── limit_pushdown/        # LIMIT 下推规则
├── projection_pushdown/   # 投影下推规则
├── elimination/           # 消除规则
├── index/                 # 索引优化规则
├── scan/                  # 扫描优化规则
├── join/                  # 连接优化规则
├── aggregate/             # 聚合优化规则
└── transformation/        # 转换规则
```

### 3.2 规则分类分析

根据规则是否需要代价信息，可以分为两类：

#### A. 与代价无关的规则（启发式规则）

这些规则基于固定的模式匹配，不需要代价计算：

| 规则类别 | 具体规则 | 说明 |
|----------|----------|------|
| **谓词下推** | `PushFilterDownScanVerticesRule` | 将过滤条件下推到扫描节点 |
| | `PushFilterDownJoinRule` | 将过滤条件下推到连接操作 |
| | `PushEFilterDownRule` | 边过滤条件下推 |
| **操作合并** | `CombineFilterRule` | 合并相邻的过滤条件 |
| | `CollapseConsecutiveProjectRule` | 合并连续的投影操作 |
| | `MergeGetVerticesAndProjectRule` | 合并获取顶点和投影 |
| **投影下推** | `PushProjectDownRule` | 将投影操作下推 |
| **消除规则** | `EliminateRedundantProjectRule` | 消除冗余投影 |
| | `EliminateEmptyFilterRule` | 消除空过滤条件 |

**特点**：
- 基于固定的模式匹配
- 不依赖统计信息
- 总是产生更优或等价的计划
- 可以在 Planner 阶段直接应用

#### B. 与代价相关的规则（基于代价的规则）

这些规则需要代价信息来选择最优方案：

| 规则类别 | 具体规则 | 说明 |
|----------|----------|------|
| **索引选择** | `IndexScanRule` | 选择最优索引 |
| **连接算法选择** | `HashJoinRule` | 选择哈希连接 |
| | `NestedLoopJoinRule` | 选择嵌套循环连接 |
| **扫描方式选择** | `IndexFullScanRule` | 选择全索引扫描 vs 全表扫描 |
| **重排序** | `JoinReorderRule` | 多表连接顺序优化 |

**特点**：
- 需要统计信息（表大小、选择性等）
- 需要计算不同方案的代价
- 选择代价最低的方案
- 必须在 Optimizer 阶段应用

### 3.3 规则迁移方案

#### 方案：分离启发式规则和基于代价的规则

```
src/query/
├── planner/
│   └── plan/
│       ├── rewriter/              # 新增：计划重写器（启发式规则）
│       │   ├── predicate_pushdown/
│       │   ├── merge/
│       │   ├── projection_pushdown/
│       │   └── elimination/
│       └── execution_plan.rs
├── optimizer/
│   └── rules/                     # 保留：基于代价的规则
│       ├── index/
│       ├── join/
│       ├── scan/
│       └── transformation/
```

#### 迁移理由

1. **职责分离**
   - Planner 负责生成"合理"的计划（应用启发式规则）
   - Optimizer 负责选择"最优"的计划（基于代价选择）

2. **简化架构**
   - 启发式规则在 Planner 阶段直接应用，无需代价计算
   - Optimizer 只处理需要代价比较的决策

3. **性能优化**
   - 启发式规则在计划生成时立即应用，减少 Optimizer 搜索空间
   - 避免在 Optimizer 中进行无意义的代价计算

4. **与 PostgreSQL 设计一致**
   - PostgreSQL 的 Planner 也包含启发式规则（如谓词下推）
   - 只有连接顺序、索引选择等才使用代价模型

#### 迁移后的处理流程

```
查询字符串 
    ↓
Parser → AST
    ↓
Validator → 验证后的 AST
    ↓
Planner → 生成初始计划
    ↓
PlanRewriter（启发式规则）→ 重写后的计划
    │   - 谓词下推
    │   - 操作合并
    │   - 投影下推
    │   - 消除冗余
    ↓
CostCalculator → 计算计划代价
    ↓
Optimizer（基于代价的规则）→ 最优计划
    │   - 索引选择
    │   - 连接算法选择
    │   - 连接顺序优化
    ↓
Executor → 执行
```

## 4. 实现步骤

### 阶段 1：扩展基础类型

1. 扩展 `PlanNode` trait，添加代价相关方法
2. 扩展 `ExecutionPlan`，添加计划级代价字段
3. 在 `QueryContext` 中集成 `StatisticsProvider`

### 阶段 2：创建代价计算基础设施

1. 创建 `CostCalculationContext`
2. 实现 `CostCalculator`，支持主要计划节点类型
3. 在 `Planner` 中集成代价计算

### 阶段 3：迁移启发式规则

1. 创建 `src/query/planner/plan/rewriter/` 目录
2. 将谓词下推、操作合并、投影下推、消除规则迁移到 rewriter
3. 在 Planner 中集成 PlanRewriter

### 阶段 4：重构 Optimizer

1. 清理 Optimizer 中的启发式规则
2. 保留基于代价的规则（索引选择、连接优化等）
3. 更新 Optimizer 使用新的代价体系

### 阶段 5：集成测试

1. 验证代价计算准确性
2. 测试规则迁移后的正确性
3. 性能测试对比

## 5. 关键设计决策

### 5.1 为什么将启发式规则迁移到 Planner？

1. **语义清晰**：Planner 生成计划，Rewriter 优化计划结构
2. **减少重复**：避免 Optimizer 对明显更优的计划进行代价计算
3. **简化 Optimizer**：Optimizer 只关注需要代价决策的场景
4. **行业标准**：PostgreSQL、MySQL 等都采用类似设计

### 5.2 为什么保留基于代价的规则在 Optimizer？

1. **需要统计信息**：索引选择、连接顺序等需要表大小、选择性等信息
2. **需要比较代价**：不同方案需要计算代价后比较
3. **搜索空间**：需要考虑多种可能的计划变体

### 5.3 代价计算的时机

- **Planner 阶段**：计算初始计划的代价，用于快速决策
- **Optimizer 阶段**：重新计算变换后计划的代价，用于比较
- **执行阶段**：可选，收集实际执行统计信息用于反馈优化

## 6. 与现有代码的兼容性

### 6.1 向后兼容策略

1. **PlanNode trait**：保留 `cost()` 方法，新增方法提供默认实现
2. **ExecutionPlan**：新增字段使用 `Option` 包装，允许渐进式迁移
3. **规则迁移**：保留原规则模块的导出，标记为 deprecated

### 6.2 渐进式迁移路径

```
阶段 1：添加新接口，保持旧接口
阶段 2：迁移内部实现使用新接口
阶段 3：更新调用方使用新接口
阶段 4：移除旧接口
```
