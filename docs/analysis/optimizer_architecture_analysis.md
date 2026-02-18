# Optimizer 架构分析报告

**分析日期**: 2026 年 2 月 18 日  
**分析目标**: `src/query/optimizer/engine/optimizer.rs`  
**分析目的**: 理解 Optimizer 的实际作用，识别与枚举 + 注册表架构的重叠问题

---

## 一、执行摘要

当前优化器实现存在**两套并行的规则管理机制**：

1. **枚举 + 注册表机制**（新架构）：`OptimizationRule` 枚举 + `RuleRegistry` 静态注册表
2. **硬编码规则集机制**（旧架构）：`Optimizer::setup_default_rule_sets()` + `get_rule_names_for_phase()`

两套机制功能重叠但互不连通，导致：
- 规则列表维护重复（新增规则需修改多处）
- `RuleConfig` 配置功能实际失效
- 代码可维护性降低

**建议**: 重构 `Optimizer` 统一使用枚举 + 注册表机制。

---

## 二、Optimizer 核心职责

### 2.1 主要功能

`optimizer.rs` 定义了优化器的核心引擎：

| 组件 | 职责 |
|------|------|
| `Optimizer` | 优化流程主控制器 |
| `RuleSet` | 规则集合容器（按阶段分组） |
| `build_initial_opt_group()` | 将 `ExecutionPlan` 转换为 `OptGroup` 结构 |
| `execute_optimization()` | 按阶段执行优化（Rewrite → Logical → Physical） |
| `extract_execution_plan()` | 从优化后的 `OptGroup` 提取最终执行计划 |
| `apply_rule()` / `explore_group()` | 规则匹配和应用的固定点迭代算法 |

### 2.2 优化流程

```
┌─────────────────────────────────────────────────────────────────┐
│                    Optimizer::optimize()                        │
├─────────────────────────────────────────────────────────────────┤
│  1. build_initial_opt_group()                                   │
│     ExecutionPlan → OptGroup (带 OptGroupNode 的图结构)          │
│                                                                 │
│  2. execute_optimization()                                      │
│     ├─ execute_phase_optimization(Rewrite)                      │
│     ├─ execute_phase_optimization(Logical)                      │
│     └─ execute_phase_optimization(Physical)                     │
│                                                                 │
│  3. extract_execution_plan()                                    │
│     OptGroup → ExecutionPlan (选择最低代价的候选计划)            │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 规则应用算法

```rust
// 简化版伪代码
fn execute_phase_optimization(ctx, root_group, phase) {
    let rule_names = get_rule_names_for_phase(phase);  // 获取本阶段规则名
    let mut round = 0;
    
    while round < max_rounds {
        for rule_name in &rule_names {
            let rule = find_rule(rule_name);  // 按名称查找规则
            apply_rule(ctx, root_group, rule);  // 应用规则
        }
        
        if !ctx.changed() && stable_count >= threshold {
            break;  // 达到稳定状态
        }
        round += 1;
    }
}
```

---

## 三、实际使用情况

### 3.1 在查询管道中的集成

```rust
// src/query/query_pipeline_manager.rs
pub struct QueryPipelineManager<S: StorageClient + 'static> {
    validator: Validator,
    planner: StaticConfigurablePlannerRegistry,
    optimizer: Optimizer,  // ← 优化器实例
    executor_factory: ExecutorFactory<S>,
    stats_manager: Arc<StatsManager>,
}

impl<S: StorageClient + 'static> QueryPipelineManager<S> {
    pub fn new(storage: Arc<Mutex<S>>, stats_manager: Arc<StatsManager>) -> Self {
        Self {
            // ...
            optimizer: Optimizer::from_registry(),  // ← 从注册表创建
            // ...
        }
    }
    
    fn optimize_execution_plan(
        &mut self,
        query_context: &mut QueryContext,
        plan: ExecutionPlan,
    ) -> DBResult<ExecutionPlan> {
        self.optimizer
            .find_best_plan(query_context, plan)  // ← 调用优化器
            .map_err(|e| DBError::from(QueryError::pipeline_optimization_error(e)))
    }
}
```

### 3.2 调用链

```
用户查询
  ↓
QueryPipelineManager::execute_query()
  ↓
QueryPipelineManager::optimize_execution_plan()
  ↓
Optimizer::find_best_plan()
  ↓
Optimizer::optimize()
  ↓
execute_optimization()
  ├─ execute_phase_optimization(Rewrite)
  ├─ execute_phase_optimization(Logical)
  └─ execute_phase_optimization(Physical)
      ↓
  get_rule_names_for_phase()  // ⚠️ 硬编码规则名列表
      ↓
  find_rule()  // ⚠️ 字符串匹配查找
      ↓
  apply_rule() → explore_group() → 规则应用
```

---

## 四、架构重叠分析

### 4.1 两套规则发现机制

#### 机制 A：枚举 + 注册表（新架构）

```rust
// src/query/optimizer/rule_enum.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationRule {
    // 逻辑优化规则
    ProjectionPushDown,
    CombineFilter,
    CollapseProject,
    // ... 共 34 个规则
    
    // 物理优化规则
    JoinOptimization,
    PushLimitDownGetVertices,
    // ...
}

impl OptimizationRule {
    pub fn phase(&self) -> OptimizationPhase {
        // 每个规则知道自己属于哪个阶段
        match self {
            Self::ProjectionPushDown => OptimizationPhase::Logical,
            Self::IndexScan => OptimizationPhase::Physical,
            // ...
        }
    }
    
    pub fn create_instance(&self) -> Option<Rc<dyn OptRule>> {
        // 直接实例化规则
        match self {
            Self::ProjectionPushDown => Some(Rc::new(ProjectionPushDownRule)),
            // ...
        }
    }
}

// src/query/optimizer/rule_registry.rs
pub fn get_rules_by_phase(phase: OptimizationPhase) -> Result<Vec<OptimizationRule>, DBError> {
    // 从注册表按阶段过滤规则
    Ok(reader.keys().filter(|r| r.phase() == phase).copied().collect())
}
```

#### 机制 B：Optimizer 内部硬编码（旧架构）

```rust
// src/query/optimizer/engine/optimizer.rs:637-667
fn get_rule_names_for_phase(&self, phase: &OptimizationPhase) -> Vec<&'static str> {
    match phase {
        OptimizationPhase::Rewrite => vec![
            "ExpandGetNeighborsRule",
            "AddVertexIdRule",
            "PushFilterDownAggregateRule",
            "LimitPushDownRule",
            "PredicatePushDownRule",
        ],
        OptimizationPhase::Logical => vec![
            "UnionEdgeTypeGroupRule",
            "GetNodeRule",
            "GetEdgeRule",
            "DedupNodeRule",
            "SortRule",
            "CollapseProjectRule",
            "CollapseFilterRule",
            "BinaryJoinRule",
        ],
        OptimizationPhase::Physical => vec![
            "IndexScanRule",
            "VertexIndexScanRule",
            "EdgeIndexScanRule",
            "HashJoinRule",
            "SortRule",
            "LimitRule",
        ],
        _ => Vec::new(),
    }
}

fn find_rule(&self, name: &str) -> Option<Rc<dyn OptRule>> {
    // 遍历所有规则集进行字符串匹配
    for rs in &self.rule_sets {
        for rule in &rs.rules {
            if rule.name() == name {
                return Some(Rc::clone(rule));
            }
        }
    }
    None
}
```

### 4.2 规则初始化重复

#### 方式 A：注册表自动注册

```rust
// src/query/optimizer/rule_registrar.rs
pub fn register_all_rules() {
    register_logical_rules();   // 16 个逻辑规则
    register_physical_rules();  // 17 个物理规则
}

fn register_logical_rules() {
    let _ = RuleRegistry::register(OptimizationRule::ProjectionPushDown, 
        || Box::new(ProjectionPushDownRule));
    let _ = RuleRegistry::register(OptimizationRule::CombineFilter, 
        || Box::new(CombineFilterRule));
    // ... 自动注册所有规则
}
```

#### 方式 B：Optimizer 手动添加

```rust
// src/query/optimizer/engine/optimizer.rs:97-137
fn setup_default_rule_sets(&mut self) {
    let mut rewrite_rules = RuleSet::new("rewrite");
    if let Some(rule) = OptimizationRule::PushFilterDownAggregate.create_instance() {
        rewrite_rules.add_rule(rule);
    }
    self.rule_sets.push(rewrite_rules);

    let mut logical_rules = RuleSet::new("logical");
    if let Some(rule) = OptimizationRule::CollapseProject.create_instance() {
        logical_rules.add_rule(rule);
    }
    if let Some(rule) = OptimizationRule::CombineFilter.create_instance() {
        logical_rules.add_rule(rule);
    }
    // ... 手动添加每个规则
    self.rule_sets.push(logical_rules);
    
    let mut physical_rules = RuleSet::new("physical");
    if let Some(rule) = OptimizationRule::IndexScan.create_instance() {
        physical_rules.add_rule(rule);
    }
    // ...
    self.rule_sets.push(physical_rules);
}
```

### 4.3 数据流向断裂

```
┌─────────────────────────────────────────────────────────────┐
│  RuleRegistry (全局静态注册表)                               │
│  - 存储所有 OptimizationRule 枚举 → RuleCreator             │
│  - 支持按 phase 过滤                                         │
│  - 支持动态启用/禁用规则 (RuleConfig)                        │
│  - 单例模式，线程安全                                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
                            │ ⚠️ 未被 Optimizer 使用
                            ✗
┌─────────────────────────────────────────────────────────────┐
│  Optimizer::rule_sets (实例级规则集)                         │
│  - 在 setup_default_rule_sets() 中硬编码                     │
│  - 通过 find_rule() 按名称查找（字符串匹配）                 │
│  - 与 RuleRegistry 完全隔离                                  │
│  - RuleConfig 配置无法生效                                   │
└─────────────────────────────────────────────────────────────┘
```

---

## 五、具体问题列表

| # | 问题 | 描述 | 影响 | 严重性 |
|---|------|------|------|--------|
| 1 | **规则发现冗余** | `get_rule_names_for_phase()` 硬编码规则名 vs `RuleRegistry::get_rules_by_phase()` | 维护两份规则列表，容易不同步 | 🔴 高 |
| 2 | **规则实例化重复** | `setup_default_rule_sets()` 手动调用 `create_instance()` vs 注册表自动创建 | 代码重复，新增规则需修改多处 | 🔴 高 |
| 3 | **配置失效** | `RuleConfig` 的启用/禁用规则功能未实际使用 | 用户无法通过配置控制规则行为 | 🔴 高 |
| 4 | **字符串匹配低效** | `find_rule()` 遍历所有规则集进行字符串比较 | 性能开销，类型安全性低 | 🟡 中 |
| 5 | **测试不一致** | `Optimizer::default()` 使用硬编码规则集 | 测试可能无法反映真实行为 | 🟡 中 |
| 6 | **规则名硬编码** | 规则名散落在多处（枚举、注册表、optimizer.rs） | 重构规则名时需同步修改多处 | 🟡 中 |

### 5.1 RuleConfig 失效示例

```rust
// src/query/optimizer/rule_config.rs
pub struct RuleConfig {
    enabled_rules: HashSet<OptimizationRule>,
    disabled_rules: HashSet<OptimizationRule>,
}

impl RuleConfig {
    pub fn enable(&mut self, rule: OptimizationRule) { ... }
    pub fn disable(&mut self, rule: OptimizationRule) { ... }
    pub fn is_enabled(&self, rule: &OptimizationRule) -> bool { ... }
}

// ❌ 但在 optimizer.rs 中从未使用：
fn setup_default_rule_sets(&mut self) {
    // 没有检查 self.config.rule_config
    // 直接添加所有规则，无视启用/禁用配置
    if let Some(rule) = OptimizationRule::CollapseProject.create_instance() {
        logical_rules.add_rule(rule);
    }
}
```

---

## 六、重构建议

### 6.1 推荐方案：统一使用枚举 + 注册表

```rust
// 重构后的 Optimizer
impl Optimizer {
    /// 从注册表加载规则，应用 RuleConfig 过滤
    pub fn from_registry_with_config(config: OptimizationConfig) -> Self {
        let mut optimizer = Self {
            config,
            rule_sets: Vec::new(),
            enable_cost_model: true,
            enable_rule_based: true,
        };
        
        // 从注册表动态加载规则
        optimizer.setup_rule_sets_from_registry();
        optimizer
    }
    
    fn setup_rule_sets_from_registry(&mut self) {
        // 按阶段从注册表获取规则
        for phase in [OptimizationPhase::Rewrite, OptimizationPhase::Logical, OptimizationPhase::Physical] {
            // 1. 从注册表获取本阶段所有规则
            let rules = RuleRegistry::get_rules_by_phase(phase)
                .unwrap_or_default();
            
            // 2. 应用 RuleConfig 过滤
            let filtered_rules: Vec<_> = rules
                .into_iter()
                .filter(|rule| {
                    // 检查启用/禁用配置
                    self.config.rule_config
                        .as_ref()
                        .map(|c| c.is_enabled(rule))
                        .unwrap_or(true)
                })
                .collect();
            
            // 3. 实例化规则并添加到规则集
            let mut rule_set = RuleSet::new(phase.name());
            for rule_enum in filtered_rules {
                if let Some(rule) = rule_enum.create_instance() {
                    rule_set.add_rule(rule);
                }
            }
            
            if !rule_set.is_empty() {
                self.rule_sets.push(rule_set);
            }
        }
    }
    
    /// 从注册表创建优化器（支持配置）
    pub fn from_registry() -> Self {
        let config = OptimizationConfig::default();
        Self::from_registry_with_config(config)
    }
    
    /// 使用自定义配置创建优化器
    pub fn with_config(rule_sets: Vec<RuleSet>, config: OptimizationConfig) -> Self {
        let mut optimizer = Self {
            config,
            rule_sets,
            enable_cost_model: true,
            enable_rule_based: true,
        };

        // 如果未提供规则集，从注册表加载
        if optimizer.rule_sets.is_empty() {
            optimizer.setup_rule_sets_from_registry();
        }

        optimizer
    }
}
```

### 6.2 需要删除的代码

```rust
// ❌ 删除：硬编码规则名列表
fn get_rule_names_for_phase(&self, phase: &OptimizationPhase) -> Vec<&'static str> {
    // 整个方法删除
}

// ❌ 删除：字符串匹配查找
fn find_rule(&self, name: &str) -> Option<Rc<dyn OptRule>> {
    // 整个方法删除
}

// ✅ 保留：但改为从注册表加载
fn setup_default_rule_sets(&mut self) {
    // 重构为 setup_rule_sets_from_registry()
}
```

### 6.3 迁移步骤

| 步骤 | 任务 | 预计工作量 |
|------|------|------------|
| 1 | 实现 `setup_rule_sets_from_registry()` | 1 小时 |
| 2 | 更新 `from_registry()` 和 `with_config()` 调用新方法 | 0.5 小时 |
| 3 | 删除 `get_rule_names_for_phase()` 和 `find_rule()` | 0.5 小时 |
| 4 | 确保 `RuleConfig` 在规则加载时被应用 | 1 小时 |
| 5 | 更新现有测试用例 | 2 小时 |
| 6 | 运行全量测试验证功能 | 1 小时 |
| **总计** | | **6 小时** |

### 6.4 重构后的数据流

```
┌─────────────────────────────────────────────────────────────┐
│  RuleRegistry (全局静态注册表)                               │
│  - OptimizationRule 枚举 → RuleCreator                      │
│  - get_rules_by_phase() 按阶段过滤                          │
└─────────────────────────────────────────────────────────────┘
                            ↓
                            │ 使用
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  Optimizer::setup_rule_sets_from_registry()                 │
│  1. RuleRegistry::get_rules_by_phase(phase)                 │
│  2. 应用 RuleConfig 过滤 (is_enabled())                     │
│  3. rule.create_instance() 实例化                           │
│  4. 添加到 Optimizer::rule_sets                             │
└─────────────────────────────────────────────────────────────┘
                            ↓
                            │ 执行优化
                            ↓
┌─────────────────────────────────────────────────────────────┐
│  execute_phase_optimization()                               │
│  - 直接遍历 self.rule_sets[phase].rules                     │
│  - 无需字符串匹配，直接应用规则                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 七、相关文件清单

### 7.1 核心文件

| 文件路径 | 作用 | 是否需要修改 |
|----------|------|--------------|
| `src/query/optimizer/engine/optimizer.rs` | 优化器引擎 | ✅ 是（主要重构目标） |
| `src/query/optimizer/rule_enum.rs` | 规则枚举定义 | ❌ 否（保持不变） |
| `src/query/optimizer/rule_registry.rs` | 规则注册表 | ❌ 否（保持不变） |
| `src/query/optimizer/rule_registrar.rs` | 规则注册初始化 | ⚠️ 可选（可简化） |
| `src/query/optimizer/rule_config.rs` | 规则配置 | ❌ 否（保持不变） |
| `src/query/optimizer/core/config.rs` | 优化器配置 | ❌ 否（保持不变） |

### 7.2 使用 Optimizer 的文件

| 文件路径 | 使用方式 |
|----------|----------|
| `src/query/query_pipeline_manager.rs` | `Optimizer::from_registry()`, `Optimizer::with_config()` |
| `src/api/service/query_processor.rs` | 通过 `QueryPipelineManager` 间接使用 |
| `src/api/service/graph_service.rs` | 通过 `QueryPipelineManager` 间接使用 |

### 7.3 规则实现文件

所有规则实现在 `src/query/optimizer/rules/` 目录下，共 40+ 个规则文件，无需修改。

---

## 八、风险评估

### 8.1 重构风险

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 规则加载顺序变化 | 低 | 中 | 确保按阶段顺序加载（Rewrite → Logical → Physical） |
| RuleConfig 行为变化 | 中 | 高 | 添加单元测试验证启用/禁用功能 |
| 现有测试失败 | 高 | 低 | 预期内的测试失败，需同步更新测试 |
| 性能回归 | 低 | 中 | 重构后规则查找从 O(n) 降为 O(1)，性能应提升 |

### 8.2 不回退措施

1. **分支开发**: 在独立分支上进行重构
2. **增量提交**: 每完成一个步骤就提交
3. **测试覆盖**: 确保关键路径有测试覆盖
4. **回滚计划**: 保留原 `setup_default_rule_sets()` 作为 fallback（标记为 `#[deprecated]`）

---

## 九、结论

### 9.1 核心发现

1. **`optimizer.rs` 是优化器的执行引擎**，负责优化流程编排和规则应用
2. **存在两套并行的规则管理机制**，功能重叠但互不连通
3. **枚举 + 注册表架构更优**，但未被充分利用
4. **`RuleConfig` 配置功能实际失效**，用户无法控制规则行为

### 9.2 建议行动

**优先级：高**

建议尽快重构 `optimizer.rs` 统一使用枚举 + 注册表机制：
- 消除代码重复
- 启用规则配置功能
- 提高可维护性
- 改善类型安全性

预计工作量：**6 小时**  
风险等级：**低**（有完善的测试覆盖）

---

## 附录 A：关键代码对比

### A.1 重构前

```rust
// optimizer.rs
fn setup_default_rule_sets(&mut self) {
    let mut logical_rules = RuleSet::new("logical");
    if let Some(rule) = OptimizationRule::CollapseProject.create_instance() {
        logical_rules.add_rule(rule);
    }
    // ... 手动添加每个规则
    self.rule_sets.push(logical_rules);
}

fn get_rule_names_for_phase(&self, phase: &OptimizationPhase) -> Vec<&'static str> {
    match phase {
        OptimizationPhase::Logical => vec![
            "CollapseProjectRule", "CombineFilterRule", ...
        ],
        // ...
    }
}

fn find_rule(&self, name: &str) -> Option<Rc<dyn OptRule>> {
    for rs in &self.rule_sets {
        for rule in &rs.rules {
            if rule.name() == name {
                return Some(Rc::clone(rule));
            }
        }
    }
    None
}
```

### A.2 重构后

```rust
// optimizer.rs
fn setup_rule_sets_from_registry(&mut self) {
    for phase in [OptimizationPhase::Rewrite, OptimizationPhase::Logical, OptimizationPhase::Physical] {
        let rules = RuleRegistry::get_rules_by_phase(phase)
            .unwrap_or_default()
            .into_iter()
            .filter(|rule| {
                self.config.rule_config
                    .as_ref()
                    .map(|c| c.is_enabled(rule))
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        
        let mut rule_set = RuleSet::new(phase.name());
        for rule_enum in rules {
            if let Some(rule) = rule_enum.create_instance() {
                rule_set.add_rule(rule);
            }
        }
        
        if !rule_set.is_empty() {
            self.rule_sets.push(rule_set);
        }
    }
}

// get_rule_names_for_phase() 和 find_rule() 已删除
// execute_phase_optimization() 直接遍历规则集
```

---

## 附录 B：测试验证清单

重构后需验证以下测试：

- [ ] `test_optimizer_creation()` - 验证优化器创建
- [ ] `test_rule_set_creation()` - 验证规则集创建
- [ ] `QueryPipelineManager` 相关测试 - 验证查询管道集成
- [ ] 各优化规则单元测试 - 验证规则行为不变
- [ ] `RuleConfig` 启用/禁用功能测试 - 验证配置生效
- [ ] 端到端查询测试 - 验证完整查询流程
