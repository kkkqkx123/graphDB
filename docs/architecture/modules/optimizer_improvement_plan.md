# GraphDB Optimizer 改进计划

## 概述

本计划基于 NebulaGraph 优化器的设计，实现 GraphDB 优化器的系统性改进。

## 问题分析

### 现有问题
1. **规则配置硬编码**：规则名称使用字符串匹配，缺乏类型安全
2. **规则启用控制缺失**：无法动态启用/禁用特定优化规则
3. **迭代控制不灵活**：最大迭代轮数固定，无法自适应提前终止

## 解决方案

### 核心改进

1. **规则枚举化**：使用 `OptimizationRule` 枚举替代字符串匹配
2. **规则配置系统**：支持通过配置文件启用/禁用规则
3. **自适应迭代**：根据优化效果动态调整迭代次数

---

## 实现状态：全部完成 ✅

### 阶段1: 创建规则枚举和配置结构 ✅

**文件修改/创建：**

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/query/optimizer/rule_enum.rs` | 新建 | 定义 `OptimizationRule` 枚举，包含所有优化规则 |
| `src/query/optimizer/rule_config.rs` | 新建 | 实现 `RuleConfig` 结构，管理规则启用状态 |
| `src/query/optimizer/mod.rs` | 修改 | 导出新模块和类型 |
| `src/query/optimizer/core/config.rs` | 修改 | 集成 `RuleConfig` 到 `OptimizationConfig` |

**核心类型：**

```rust
pub enum OptimizationRule {
    FilterPushDown,
    PredicatePushDown,
    ProjectionPushDown,
    // ... 更多规则
}

impl OptimizationRule {
    pub fn phase(&self) -> OptimizationPhase { ... }
    pub fn name(&self) -> &'static str { ... }
    pub fn from_name(name: &str) -> Option<Self> { ... }
}

pub struct RuleConfig {
    enabled_rules: Vec<OptimizationRule>,
    disabled_rules: Vec<OptimizationRule>,
    rule_flags: FxHashMap<&'static str, bool>,
}
```

---

### 阶段2: 创建规则注册机制 ✅

**文件修改/创建：**

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/query/optimizer/rule_registry.rs` | 新建 | 实现静态规则注册表，支持规则发现 |
| `src/query/optimizer/mod.rs` | 修改 | 导出 `RuleRegistry` |

**核心功能：**

```rust
pub struct RuleRegistry;

impl RuleRegistry {
    pub fn register(rule: OptimizationRule, creator: fn() -> Box<dyn OptRule>);
    pub fn create_instance(rule: OptimizationRule) -> Option<Box<dyn OptRule>>;
    pub fn get_all_rules() -> Vec<OptimizationRule>;
    pub fn get_rules_by_phase(phase: OptimizationPhase) -> Vec<OptimizationRule>;
}

#[macro_export]
macro_rules! register_rule {
    ($rule:expr, $creator:expr) => { ... };
}
```

---

### 阶段3: 修改 Optimizer 使用枚举匹配 ✅

**文件修改：**

| 文件 | 说明 |
|------|------|
| `src/query/optimizer/engine/optimizer.rs` | 修改 `get_rules_for_phase` 使用枚举匹配 |

**改进点：**

```rust
fn get_rules_for_phase(&self, phase: &OptimizationPhase) -> Vec<&dyn OptRule> {
    for rule_set in &self.rule_sets {
        for rule in &rule_set.rules {
            let rule_enum = OptimizationRule::from_name(rule.name());
            if let Some(enum_rule) = rule_enum {
                if enum_rule.phase() == *phase && self.is_rule_enabled(enum_rule) {
                    rules.push(rule.as_ref());
                }
            }
        }
    }
    rules
}
```

**新增方法：**

```rust
impl Optimizer {
    pub fn enable_rule(&mut self, rule: OptimizationRule);
    pub fn disable_rule(&mut self, rule: OptimizationRule);
    pub fn is_rule_enabled(&self, rule: OptimizationRule) -> bool;
}
```

---

### 阶段4: 实现规则启用开关 ✅

**文件修改/创建：**

| 文件 | 操作 | 说明 |
|------|------|------|
| `src/query/optimizer/optimizer_config.rs` | 新建 | 配置文件解析模块 |
| `src/query/optimizer/mod.rs` | 修改 | 导出配置加载函数 |
| `config.toml` | 修改 | 添加优化器配置节 |

**配置文件示例：**

```toml
[optimizer]
max_iteration_rounds = 5
max_exploration_rounds = 128
enable_cost_model = true
enable_multi_plan = true
enable_property_pruning = true

[optimizer.disabled_rules]
FilterPushDownRule = false
PredicatePushDownRule = false

[optimizer.enabled_rules]
RemoveUselessNodeRule = true
```

**配置加载：**

```rust
pub fn load_optimizer_config(config_path: &Path) -> Result<OptimizerConfigInfo, String>;
```

---

### 阶段5: 实现自适应迭代轮数 ✅

**文件修改：**

| 文件 | 说明 |
|------|------|
| `src/query/optimizer/core/config.rs` | 添加自适应迭代配置字段 |
| `src/query/optimizer/optimizer_config.rs` | 添加自适应配置解析 |
| `src/query/optimizer/engine/optimizer.rs` | 实现自适应迭代逻辑 |

**新增配置字段：**

```rust
pub struct OptimizationConfig {
    pub enable_adaptive_iteration: bool,  // 默认 true
    pub stable_threshold: usize,          // 默认 2
    pub min_iteration_rounds: usize,      // 默认 1
}
```

**自适应迭代逻辑：**

```rust
fn execute_phase_optimization(
    &mut self,
    ctx: &mut OptContext,
    root_group: &mut OptGroup,
    phase: OptimizationPhase,
) -> Result<(), OptimizerError> {
    let min_rounds = self.config.min_iteration_rounds;
    let stable_threshold = self.config.stable_threshold;
    let enable_adaptive = self.config.enable_adaptive_iteration;
    
    let mut stable_count = 0;

    while round < max_rounds {
        // 应用规则...
        
        if ctx.changed {
            stable_count = 0;
        } else {
            stable_count += 1;
        }
        
        if enable_adaptive 
            && round >= min_rounds 
            && stable_count >= stable_threshold 
        {
            break;  // 提前终止
        }
    }
    Ok(())
}
```

---

## 改进效果

### 1. 类型安全
- 规则枚举提供编译时类型检查
- 消除字符串匹配的运行时错误

### 2. 灵活配置
- 支持通过配置文件控制规则启用状态
- 支持运行时动态调整规则配置

### 3. 性能优化
- 自适应迭代减少不必要的优化轮数
- 稳定后提前终止，避免资源浪费

### 4. 可维护性
- 规则枚举集中管理所有优化规则
- 规则注册机制支持插件化扩展

---

## 使用示例

```rust
use graphdb::query::optimizer::{Optimizer, OptimizationRule, RuleConfig};

// 创建配置
let mut rule_config = RuleConfig::default();
rule_config.disable(OptimizationRule::FilterPushDown);
rule_config.enable(OptimizationRule::RemoveUselessNode);

// 使用配置创建优化器
let config = OptimizationConfig::with_rule_config(rule_config);
let optimizer = Optimizer::with_config(rule_sets, config);

// 检查规则状态
if optimizer.is_rule_enabled(OptimizationRule::JoinOptimization) {
    // 规则已启用
}
```

---

## 下一步工作

当前改进已完成。未来可考虑：

1. **规则优先级机制**：为不同规则设置优先级，控制应用顺序
2. **代价模型集成**：利用规则枚举实现更精确的代价估算
3. **优化规则扩展**：通过注册机制支持自定义优化规则
4. **优化统计增强**：收集更多优化效果指标，支持性能调优
