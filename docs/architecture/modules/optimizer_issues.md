# Optimizer 模块问题清单与修改方案

## 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 5.1 | 优化规则硬编码，无法动态配置 | 高 | 扩展性问题 | 待修复 |
| 5.2 | MAX_EXPLORATION_ROUNDS 硬编码为 128 | 中 | 灵活性问题 | 待修复 |
| 5.3 | 优化阶段规则分配使用字符串匹配 | 中 | 代码质量问题 | 待修复 |
| 5.4 | property_pruning 和 rewrite_arguments 为空实现 | 中 | 功能缺失 | 待修复 |
| 5.5 | 成本模型过于简单 | 低 | 功能不完整 | 待修复 |
| 5.6 | 缺乏优化规则注册机制 | 低 | 扩展性问题 | 待修复 |
| 5.7 | 不支持基于规则的执行计划生成 | 低 | 功能缺失 | 待修复 |

---

## 详细问题分析

### 问题 5.1: 优化规则硬编码

**涉及文件**: `src/query/optimizer/mod.rs`

**当前实现**:
```rust
pub fn default() -> Self {
    let mut logical_rules = RuleSet::new("logical");
    logical_rules.add_rule(Box::new(FilterPushDownRule));
    logical_rules.add_rule(Box::new(PredicatePushDownRule));
    logical_rules.add_rule(Box::new(ProjectionPushDownRule));
    logical_rules.add_rule(Box::new(CombineFilterRule));
    logical_rules.add_rule(Box::new(CollapseProjectRule));
    logical_rules.add_rule(Box::new(DedupEliminationRule));
    logical_rules.add_rule(Box::new(CollapseProjectRule));
    logical_rules.add_rule(Box::new(TopNRule));
    logical_rules.add_rule(Box::new(MergeCursorRule));
    logical_rules.add_rule(Box::new(EquivExprRule));
    logical_rules.add_rule(Box::new(PushLimitDownRule));
    logical_rules.add_rule(Box::new(PredicatePushDownRule));
    logical_rules.add_rule(Box::new(CollapseProjectRule));
    logical_rules.add_rule(Box::new(CombineFilterRule));
    logical_rules.add_rule(Box::new(CollapseProjectRule));
    logical_rules.add_rule(Box::new(DedupEliminationRule));
    logical_rules.add_rule(Box::new(CollapseProjectRule));
    logical_rules.add_rule(Box::new(TopNRule));
    logical_rules.add_rule(Box::new(CollapseProjectRule));
    logical_rules.add_rule(Box::new(CombineFilterRule));
    logical_rules.add_rule(Box::new(EquivExprRule));
    
    let mut physical_rules = RuleSet::new("physical");
    physical_rules.add_rule(Box::new(JoinOptimizationRule));
    
    let mut post_rules = RuleSet::new("post");
    post_rules.add_rule(Box::new(TopNRule));
    
    Self {
        rule_sets: vec![logical_rules, physical_rules, post_rules],
    }
}
```

**问题**:
- 无法在运行时添加/移除规则
- 无法配置规则的执行顺序
- 无法为不同查询类型配置不同规则集
- 无法通过配置文件管理规则

---

### 问题 5.2: 硬编码的 MAX_EXPLORATION_ROUNDS

**涉及文件**: `src/query/optimizer/mod.rs`

**当前实现**:
```rust
fn optimize_subplan(
    &mut self,
    plan_group: &mut OptGroup,
    context: &mut OptContext,
) -> Result<Option<PlanNodeEnum>, OptimizerError> {
    let mut rounds = 0u64;
    let mut best_plan_node = plan_group.best_plan_node().clone();
    let mut best_cost = context.get_cost(&best_plan_node).unwrap_or(Cost::new());

    let mut max_rounds = 128u64;  // 硬编码
    for _ in 0..max_rounds {
        // ...
    }
    best_plan_node
}
```

**问题**:
- 无法根据查询复杂度动态调整
- 简单查询可能过度优化
- 复杂查询可能迭代不足

---

### 问题 5.3: 规则分配使用字符串匹配

**涉及文件**: `src/query/optimizer/mod.rs`

**当前实现**:
```rust
fn get_rules_for_phase(&self, phase: &OptimizationPhase) -> Vec<&dyn OptRule> {
    for rule_set in &self.rule_sets {
        for rule in &rule_set.rules {
            let rule_name = rule.name();  // 返回字符串
            let matches_phase = match phase {
                OptimizationPhase::LogicalOptimization => {
                    matches!(rule_name,
                        "FilterPushDownRule" | "PredicatePushDownRule" |
                        "ProjectionPushDownRule" | "CombineFilterRule" |
                        "CollapseProjectRule" | "DedupEliminationRule" |
                        "TopNRule" | "MergeCursorRule" | "EquivExprRule" |
                        "PushLimitDownRule"
                    )
                }
                OptimizationPhase::PhysicalOptimization => {
                    matches!(rule_name, "JoinOptimizationRule")
                }
                OptimizationPhase::PostOptimization => {
                    matches!(rule_name, "TopNRule")
                }
            };
            if matches_phase {
                results.push(rule.as_ref());
            }
        }
    }
    results
}
```

**问题**:
- 容易出错（字符串拼写错误）
- 难以维护（添加新规则需要修改多处代码）
- 编译时无法检查

---

### 问题 5.4: property_pruning 和 rewrite_arguments 为空实现

**涉及文件**: `src/query/optimizer/mod.rs`

**当前实现**:
```rust
fn property_pruning(
    &self,
    _ctx: &mut OptContext,
    _root_group: &mut OptGroup,
) -> Result<(), OptimizerError> {
    // 空实现
    Ok(())
}

fn rewrite_arguments(
    &self,
    _ctx: &mut OptContext,
    _root_group: &mut OptGroup,
) -> Result<(), OptimizerError> {
    // 空实现
    Ok(())
}
```

**问题**:
- 属性裁剪功能未实现
- 参数重写功能未实现
- 无法优化查询中未使用的列/属性

---

### 问题 5.5: 成本模型过于简单

**当前实现**: 只有简单的 Cost 结构体，无实际成本估算

**缺失功能**:
- 基于统计信息的成本估算
- 不同算子的成本模型
- 基数估算（Cardinality Estimation）
- 选择率估算（Selectivity Estimation）

---

## 修改方案

### 修改方案 5.1-5.3: 配置化优化规则

**预估工作量**: 4-5 人天

**修改目标**:
- 使用枚举替代字符串匹配规则
- 支持配置化的规则管理
- 灵活配置优化参数

**修改步骤**:

**步骤 1**: 定义优化规则枚举

```rust
// src/query/optimizer/rules.rs

use strum_macros::{Display, EnumString};

/// 优化规则枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
pub enum OptimizationRule {
    // 逻辑优化规则
    #[strum(serialize = "FilterPushDownRule")]
    FilterPushDown,
    
    #[strum(serialize = "PredicatePushDownRule")]
    PredicatePushDown,
    
    #[strum(serialize = "ProjectionPushDownRule")]
    ProjectionPushDown,
    
    #[strum(serialize = "CombineFilterRule")]
    CombineFilter,
    
    #[strum(serialize = "CollapseProjectRule")]
    CollapseProject,
    
    #[strum(serialize = "DedupEliminationRule")]
    DedupElimination,
    
    #[strum(serialize = "TopNRule")]
    TopN,
    
    #[strum(serialize = "MergeCursorRule")]
    MergeCursor,
    
    #[strum(serialize = "EquivExprRule")]
    EquivExpr,
    
    #[strum(serialize = "PushLimitDownRule")]
    PushLimitDown,
    
    // 物理优化规则
    #[strum(serialize = "JoinOptimizationRule")]
    JoinOptimization,
    
    #[strum(serialize = "IndexScanRule")]
    IndexScan,
    
    #[strum(serialize = "HashJoinRule")]
    HashJoin,
    
    #[strum(serialize = "SortMergeJoinRule")]
    SortMergeJoin,
    
    // 后优化规则
    #[strum(serialize = "RemoveUselessNodeRule")]
    RemoveUselessNode,
}

impl OptimizationRule {
    /// 获取规则所属的优化阶段
    pub fn belongs_to(&self) -> OptimizationPhase {
        match self {
            // 逻辑优化
            Self::FilterPushDown | Self::PredicatePushDown | Self::ProjectionPushDown |
            Self::CombineFilter | Self::CollapseProject | Self::DedupElimination |
            Self::TopN | Self::MergeCursor | Self::EquivExpr | Self::PushLimitDown => {
                OptimizationPhase::LogicalOptimization
            }
            
            // 物理优化
            Self::JoinOptimization | Self::IndexScan | Self::HashJoin | Self::SortMergeJoin => {
                OptimizationPhase::PhysicalOptimization
            }
            
            // 后优化
            Self::RemoveUselessNode => {
                OptimizationPhase::PostOptimization
            }
        }
    }
    
    /// 创建规则实例
    pub fn create_instance(&self) -> Box<dyn OptRule> {
        match self {
            Self::FilterPushDown => Box::new(FilterPushDownRule),
            Self::PredicatePushDown => Box::new(PredicatePushDownRule),
            Self::ProjectionPushDown => Box::new(ProjectionPushDownRule),
            Self::CombineFilter => Box::new(CombineFilterRule),
            Self::CollapseProject => Box::new(CollapseProjectRule),
            Self::DedupElimination => Box::new(DedupEliminationRule),
            Self::TopN => Box::new(TopNRule),
            Self::MergeCursor => Box::new(MergeCursorRule),
            Self::EquivExpr => Box::new(EquivExprRule),
            Self::PushLimitDown => Box::new(PushLimitDownRule),
            Self::JoinOptimization => Box::new(JoinOptimizationRule),
            Self::IndexScan => Box::new(IndexScanRule),
            Self::HashJoin => Box::new(HashJoinRule),
            Self::SortMergeJoin => Box::new(SortMergeJoinRule),
            Self::RemoveUselessNode => Box::new(RemoveUselessNodeRule),
        }
    }
}
```

**步骤 2**: 创建优化配置

```rust
// src/query/optimizer/config.rs

/// 优化配置
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// 启用的逻辑优化规则
    pub logical_rules: Vec<OptimizationRule>,
    /// 启用的物理优化规则
    pub physical_rules: Vec<OptimizationRule>,
    /// 启用的后优化规则
    pub post_rules: Vec<OptimizationRule>,
    /// 最大迭代轮数
    pub max_iteration_rounds: usize,
    /// 最大探索轮数
    pub max_exploration_rounds: usize,
    /// 是否启用基于成本的优化
    pub enable_cost_based_optimization: bool,
    /// 默认行数（用于估算）
    pub default_row_count: usize,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            logical_rules: vec![
                OptimizationRule::FilterPushDown,
                OptimizationRule::PredicatePushDown,
                OptimizationRule::ProjectionPushDown,
                OptimizationRule::CombineFilter,
                OptimizationRule::CollapseProject,
                OptimizationRule::DedupElimination,
                OptimizationRule::TopN,
                OptimizationRule::PushLimitDown,
                OptimizationRule::EquivExpr,
            ],
            physical_rules: vec![
                OptimizationRule::JoinOptimization,
                OptimizationRule::IndexScan,
            ],
            post_rules: vec![
                OptimizationRule::RemoveUselessNode,
                OptimizationRule::TopN,
            ],
            max_iteration_rounds: 10,
            max_exploration_rounds: 128,
            enable_cost_based_optimization: true,
            default_row_count: 1000,
        }
    }
}

impl OptimizationConfig {
    /// 从配置文件加载
    pub fn from_toml(config_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(config_str)
    }
    
    /// 从环境变量覆盖
    pub fn apply_env_overrides(&mut self) {
        if let Ok(max_rounds) = std::env::var("OPTIMIZER_MAX_ROUNDS") {
            if let Ok(rounds) = max_rounds.parse() {
                self.max_iteration_rounds = rounds;
            }
        }
        
        if let Ok(enable_cbo) = std::env::var("OPTIMIZER_ENABLE_CBO") {
            self.enable_cost_based_optimization = enable_cbo.to_lowercase() == "true";
        }
    }
}
```

**步骤 3**: 修改 Optimizer 实现

```rust
// src/query/optimizer/mod.rs

impl Optimizer {
    /// 使用配置创建优化器
    pub fn with_config(config: OptimizationConfig) -> Self {
        let mut logical_rules = RuleSet::new("logical");
        for rule in &config.logical_rules {
            logical_rules.add_rule(rule.create_instance());
        }
        
        let mut physical_rules = RuleSet::new("physical");
        for rule in &config.physical_rules {
            physical_rules.add_rule(rule.create_instance());
        }
        
        let mut post_rules = RuleSet::new("post");
        for rule in &config.post_rules {
            post_rules.add_rule(rule.create_instance());
        }
        
        Self {
            rule_sets: vec![logical_rules, physical_rules, post_rules],
            config,
        }
    }
    
    fn get_rules_for_phase(&self, phase: &OptimizationPhase) -> Vec<&dyn OptRule> {
        let mut results = Vec::new();
        
        for rule_set in &self.rule_sets {
            for rule in &rule_set.rules {
                // 使用枚举匹配替代字符串匹配
                let rule_enum = rule.as_rule_enum();
                if rule_enum.belongs_to() == *phase {
                    results.push(rule.as_ref());
                }
            }
        }
        
        results
    }
}
```

---

### 修改方案 5.2: 动态调整优化轮数

**预估工作量**: 0.5 人天

**修改代码**:

```rust
impl Optimizer {
    fn optimize_subplan(
        &mut self,
        plan_group: &mut OptGroup,
        context: &mut OptContext,
    ) -> Result<Option<PlanNodeEnum>, OptimizerError> {
        let mut rounds = 0u64;
        let mut best_plan_node = plan_group.best_plan_node().clone();
        let mut best_cost = context.get_cost(&best_plan_node).unwrap_or(Cost::new());

        // 使用配置中的值
        let mut max_rounds = self.config.max_exploration_rounds as u64;
        
        // 自适应调整：根据计划复杂度动态调整
        let plan_complexity = self.estimate_plan_complexity(plan_group);
        if plan_complexity < 10 {
            max_rounds = max_rounds / 4;  // 简单计划，少迭代
        } else if plan_complexity > 100 {
            max_rounds = max_rounds * 2;  // 复杂计划，多迭代
        }

        for _ in 0..max_rounds {
            // 检查是否收敛
            if rounds > 0 && best_cost.is_zero() {
                break;
            }

            // 执行优化步骤
            let changes = self.explore_plan(context, plan_group)?;
            
            if changes == 0 {
                break;  // 无变化，提前终止
            }

            rounds += 1;
        }

        // 记录优化统计
        context.add_optimization_stats(rounds, best_cost.clone());

        best_plan_node
    }
    
    fn estimate_plan_complexity(&self, plan_group: &OptGroup) -> usize {
        // 基于节点数和深度估算复杂度
        let node_count = plan_group.nodes.len();
        let max_depth = plan_group.nodes.iter()
            .map(|n| n.dependencies.len())
            .max()
            .unwrap_or(0);
        
        node_count * (max_depth + 1)
    }
}
```

---

### 修改方案 5.4: 实现属性裁剪和参数重写

**预估工作量**: 3-4 人天

**修改代码**:

```rust
impl Optimizer {
    /// 属性裁剪：移除未被使用的属性
    fn property_pruning(
        &self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        // 1. 收集所有需要的属性
        let mut required_props = HashSet::new();
        self.collect_required_properties(ctx, root_group, &mut required_props)?;
        
        // 2. 对每个节点应用属性裁剪
        for node in &mut root_group.nodes {
            self.prune_node_properties(node, &required_props)?;
            
            // 递归处理依赖节点
            for &dep_id in &node.dependencies {
                if let Some(dep_group) = ctx.group_map.get_mut(&dep_id) {
                    self.property_pruning(ctx, dep_group)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn collect_required_properties(
        &self,
        ctx: &mut OptContext,
        group: &OptGroup,
        required_props: &mut HashSet<String>,
    ) -> Result<(), OptimizerError> {
        for node in &group.nodes {
            match node.plan_node.name() {
                "Project" => {
                    if let Some(project_node) = node.plan_node.as_project() {
                        // 收集 Project 需要的列
                        for column in project_node.columns() {
                            self.collect_expr_properties(column.expression(), required_props);
                        }
                    }
                }
                "Filter" => {
                    if let Some(filter_node) = node.plan_node.as_filter() {
                        self.collect_expr_properties(filter_node.condition(), required_props);
                    }
                }
                "Aggregate" => {
                    // Aggregate 需要 group by 的列
                    if let Some(agg_node) = node.plan_node.as_aggregate() {
                        for group_by in agg_node.group_keys() {
                            self.collect_expr_properties(group_by, required_props);
                        }
                    }
                }
                _ => {}
            }
            
            // 递归处理依赖
            for &dep_id in &node.dependencies {
                if let Some(dep_group) = ctx.group_map.get(&dep_id) {
                    self.collect_required_properties(ctx, dep_group, required_props)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn prune_node_properties(
        &self,
        node: &mut OptGroupNode,
        required_props: &HashSet<String>,
    ) -> Result<(), OptimizerError> {
        match node.plan_node.name() {
            "GetNeighbors" => {
                if let Some(gn_node) = node.plan_node.as_get_neighbors_mut() {
                    let current_props = gn_node.properties();
                    let pruned_props: Vec<String> = current_props
                        .iter()
                        .filter(|prop| required_props.contains(*prop))
                        .cloned()
                        .collect();
                    
                    gn_node.set_properties(pruned_props);
                }
            }
            "GetVertices" => {
                if let Some(gv_node) = node.plan_node.as_get_vertices_mut() {
                    let current_props = gv_node.properties();
                    let pruned_props: Vec<String> = current_props
                        .iter()
                        .filter(|prop| required_props.contains(*prop))
                        .cloned()
                        .collect();
                    
                    gv_node.set_properties(pruned_props);
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// 参数重写：将参数引用替换为实际值
    fn rewrite_arguments(
        &self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        let arg_map = self.build_argument_mapping(ctx, root_group)?;
        
        for node in &mut root_group.nodes {
            self.rewrite_node_arguments(node, &arg_map)?;
            
            // 递归处理依赖节点
            for &dep_id in &node.dependencies {
                if let Some(dep_group) = ctx.group_map.get_mut(&dep_id) {
                    self.rewrite_arguments(ctx, dep_group)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn build_argument_mapping(
        &self,
        ctx: &OptContext,
        group: &OptGroup,
    ) -> Result<HashMap<usize, HashMap<String, Expression>>, OptimizerError> {
        let mut mapping = HashMap::new();
        
        for node in &group.nodes {
            if let Some(outputs) = self.get_node_outputs(node) {
                mapping.insert(node.id, outputs);
            }
        }
        
        Ok(mapping)
    }
    
    fn rewrite_node_arguments(
        &self,
        node: &mut OptGroupNode,
        arg_map: &HashMap<usize, HashMap<String, Expression>>,
    ) -> Result<(), OptimizerError> {
        match node.plan_node.name() {
            "Project" => {
                if let Some(project_node) = node.plan_node.as_project_mut() {
                    for column in project_node.columns_mut() {
                        if let Some(arg_mapping) = arg_map.get(&node.id) {
                            self.rewrite_expr_arguments(column.expression_mut(), arg_mapping);
                        }
                    }
                }
            }
            "Filter" => {
                if let Some(filter_node) = node.plan_node.as_filter_mut() {
                    if let Some(arg_mapping) = arg_map.get(&node.id) {
                        self.rewrite_expr_arguments(filter_node.condition_mut(), arg_mapping);
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
}
```

---

### 修改方案 5.5: 增强成本模型

**预估工作量**: 5-7 人天

**修改代码**:

```rust
// src/query/optimizer/cost.rs

use std::collections::HashMap;

/// 成本模型
#[derive(Debug, Clone)]
pub struct Cost {
    pub cpu_cost: f64,
    pub memory_cost: f64,
    pub io_cost: f64,
    pub network_cost: f64,
    pub row_count: usize,
}

impl Cost {
    pub fn new() -> Self {
        Self {
            cpu_cost: 0.0,
            memory_cost: 0.0,
            io_cost: 0.0,
            network_cost: 0.0,
            row_count: 0,
        }
    }
    
    pub fn zero() -> Self {
        Self::new()
    }
    
    pub fn is_zero(&self) -> bool {
        self.cpu_cost == 0.0 && self.memory_cost == 0.0 && 
        self.io_cost == 0.0 && self.network_cost == 0.0
    }
    
    pub fn add(&mut self, other: &Cost) {
        self.cpu_cost += other.cpu_cost;
        self.memory_cost += other.memory_cost;
        self.io_cost += other.io_cost;
        self.network_cost += other.network_cost;
        self.row_count = self.row_count.max(other.row_count);
    }
    
    pub fn total(&self) -> f64 {
        // 加权总成本
        self.cpu_cost * 1.0 + 
        self.memory_cost * 0.5 + 
        self.io_cost * 2.0 + 
        self.network_cost * 3.0
    }
}

/// 成本估算器
pub trait CostEstimator {
    fn estimate_cost(&self, node: &PlanNodeEnum, input_cardinality: usize) -> Cost;
}

/// 默认成本估算器
pub struct DefaultCostEstimator {
    config: OptimizationConfig,
    statistics: HashMap<String, TableStatistics>,
}

#[derive(Debug, Clone)]
pub struct TableStatistics {
    pub row_count: usize,
    pub column_stats: HashMap<String, ColumnStatistics>,
}

#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    pub distinct_count: usize,
    pub null_count: usize,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub histogram: Option<Histogram>,
}

impl DefaultCostEstimator {
    pub fn new(config: OptimizationConfig) -> Self {
        Self {
            config,
            statistics: HashMap::new(),
        }
    }
    
    pub fn update_statistics(&mut self, table: &str, stats: TableStatistics) {
        self.statistics.insert(table.to_string(), stats);
    }
}

impl CostEstimator for DefaultCostEstimator {
    fn estimate_cost(&self, node: &PlanNodeEnum, input_cardinality: usize) -> Cost {
        let mut cost = Cost::new();
        cost.row_count = input_cardinality;
        
        match node {
            PlanNodeEnum::ScanVertices(n) => {
                // 全表扫描成本
                cost.io_cost = input_cardinality as f64;
                cost.cpu_cost = input_cardinality as f64 * 0.1;
                cost.row_count = self.config.default_row_count;
            }
            PlanNodeEnum::GetNeighbors(n) => {
                // 获取邻居成本
                let expand_factor = n.step_limit().unwrap_or(10) as f64;
                cost.io_cost = input_cardinality as f64 * expand_factor;
                cost.cpu_cost = input_cardinality as f64 * expand_factor * 0.5;
                cost.row_count = input_cardinality * expand_factor as usize;
            }
            PlanNodeEnum::Filter(n) => {
                // 过滤成本
                let selectivity = self.estimate_selectivity(n);
                cost.cpu_cost = input_cardinality as f64 * 0.1;
                cost.row_count = (input_cardinality as f64 * selectivity) as usize;
            }
            PlanNodeEnum::Project(n) => {
                // 投影成本
                cost.cpu_cost = input_cardinality as f64 * 0.05 * n.columns().len() as f64;
                cost.row_count = input_cardinality;
            }
            PlanNodeEnum::Join(n) => {
                // Join 成本
                let join_type = n.join_type();
                let join_factor = match join_type {
                    JoinType::HashJoin => 0.8,
                    JoinType::SortMergeJoin => 1.2,
                    _ => 1.0,
                };
                cost.cpu_cost = input_cardinality as f64 * input_cardinality as f64 * 0.01 * join_factor;
                cost.memory_cost = input_cardinality as f64 * 0.5;
                cost.row_count = input_cardinality;
            }
            PlanNodeEnum::Aggregate(n) => {
                // 聚合成本
                cost.cpu_cost = input_cardinality as f64 * 0.5;
                cost.row_count = input_cardinality / 10;  // 估算
            }
            PlanNodeEnum::Sort(n) => {
                // 排序成本
                cost.cpu_cost = input_cardinality as f64 * input_cardinality as f64 * 0.001;
                cost.memory_cost = input_cardinality as f64 * 0.1;
                cost.row_count = input_cardinality;
            }
            PlanNodeEnum::Limit(n) => {
                cost.cpu_cost = input_cardinality as f64 * 0.01;
                cost.row_count = n.limit().min(input_cardinality);
            }
            _ => {
                cost.cpu_cost = input_cardinality as f64 * 0.01;
                cost.row_count = input_cardinality;
            }
        }
        
        cost
    }
    
    fn estimate_selectivity(&self, filter_node: &FilterNode) -> f64 {
        // 估算过滤条件的 selectivity
        // 这是一个简化实现，实际应该基于统计信息
        0.5  // 默认 50%
    }
}
```

---

## 修改优先级

| 序号 | 修改方案 | 优先级 | 预估工作量 | 依赖 |
|------|----------|--------|------------|------|
| 5.1-5.3 | 配置化优化规则 | 高 | 4-5 人天 | 无 |
| 5.4 | 实现属性裁剪和参数重写 | 中 | 3-4 人天 | 无 |
| 5.2 | 动态调整优化轮数 | 低 | 0.5 人天 | 无 |
| 5.5 | 增强成本模型 | 低 | 5-7 人天 | 无 |

---

## 测试建议

### 测试用例 1: 配置化规则

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_custom_rules_config() {
        let config = OptimizationConfig {
            logical_rules: vec![
                OptimizationRule::FilterPushDown,
                OptimizationRule::ProjectionPushDown,
            ],
            physical_rules: vec![
                OptimizationRule::IndexScan,
            ],
            ..Default::default()
        };
        
        let optimizer = Optimizer::with_config(config);
        
        // 验证规则被正确配置
        let logical_rules = optimizer.get_rules_for_phase(&OptimizationPhase::LogicalOptimization);
        assert_eq!(logical_rules.len(), 2);
    }
    
    #[test]
    fn test_property_pruning() {
        let optimizer = Optimizer::default();
        let mut ctx = OptContext::new();
        let mut group = create_test_group();
        
        optimizer.property_pruning(&mut ctx, &mut group).unwrap();
        
        // 验证属性被正确裁剪
    }
}
```

---

## 风险与注意事项

### 风险 1: 成本模型准确性

- **风险**: 简化的成本模型可能导致次优计划
- **缓解措施**: 提供多种成本模型选择
- **实现**: 可插拔的成本估算器接口

### 风险 2: 优化规则冲突

- **风险**: 某些规则组合可能导致问题
- **缓解措施**: 提供规则验证和冲突检测
- **实现**: 在配置时检查规则兼容性
