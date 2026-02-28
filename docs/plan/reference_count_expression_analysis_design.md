# 引用计数统计与表达式分析功能设计文档

## 概述

本文档设计两个新功能，用于支持基于代价的优化决策：

1. **引用计数统计** (Reference Counting)：识别计划树中被多次引用的子计划
2. **表达式分析** (Expression Analysis)：分析表达式的确定性、复杂度、涉及属性等

**模块归属**：这两个功能放在 `src/query/optimizer/analysis/` 目录下，与 `cost` 目录同级，在优化阶段按需计算。

---

## 一、模块架构设计

### 1.1 目录结构

```
src/query/optimizer/
├── mod.rs                          # 优化器模块入口
├── engine.rs                       # 优化器引擎
├── stats/                          # 统计信息
├── cost/                           # 代价计算
│   ├── mod.rs
│   ├── calculator.rs
│   └── ...
├── decision/                       # 优化决策
└── analysis/                       # 【新增】计划分析
    ├── mod.rs                      # 模块入口
    ├── reference_count.rs          # 引用计数分析
    ├── expression.rs               # 表达式分析
    └── fingerprint.rs              # 指纹计算
```

### 1.2 设计理由

将分析功能放在 `optimizer/analysis` 而非 `planner/plan/analysis` 的原因：

1. **职责分离**：
   - `planner` 负责**生成**执行计划
   - `optimizer` 负责**优化**执行计划
   - 引用计数和表达式分析是**优化决策**的辅助工具

2. **按需计算**：
   - 计划生成阶段不需要这些分析信息
   - 仅在优化阶段根据策略需要才进行计算
   - 避免在计划生成阶段引入不必要的开销

3. **依赖关系**：
   - `optimizer` 已经依赖 `planner`（可以导入planner模块）
   - 放在 `optimizer` 不会引入循环依赖

4. **与代价计算一致**：
   - 与 `cost` 模块处于同一层级
   - 都是优化决策的支持工具
   - 可以共享统计信息、配置等

---

## 二、引用计数统计功能设计

### 2.1 功能目标

识别执行计划中被多次引用的子计划节点，为物化策略选择提供数据支持。

### 2.2 核心数据结构

```rust
// src/query/optimizer/analysis/reference_count.rs

/// 子计划引用信息
#[derive(Debug, Clone)]
pub struct SubplanReferenceInfo {
    /// 子计划的唯一标识（基于节点ID列表的哈希）
    pub subplan_id: SubplanId,
    /// 子计划根节点ID
    pub root_node_id: i64,
    /// 被引用次数
    pub reference_count: usize,
    /// 引用位置（父节点ID列表）
    pub reference_locations: Vec<i64>,
    /// 估算输出行数
    pub estimated_output_rows: u64,
    /// 子计划包含的节点数量
    pub node_count: usize,
}

/// 子计划唯一标识
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubplanId(u64);

/// 引用计数分析结果
#[derive(Debug, Clone)]
pub struct ReferenceCountAnalysis {
    /// 所有被多次引用的子计划
    pub repeated_subplans: Vec<SubplanReferenceInfo>,
    /// 节点ID到引用信息的映射
    pub node_reference_map: HashMap<i64, SubplanReferenceInfo>,
}

/// 引用计数分析器
pub struct ReferenceCountAnalyzer {
    /// 指纹计算器
    fingerprint_calculator: FingerprintCalculator,
}

impl ReferenceCountAnalyzer {
    /// 分析计划的引用计数
    pub fn analyze(&self, plan: &PlanNodeEnum) -> ReferenceCountAnalysis {
        let mut context = AnalysisContext::new();
        self.analyze_recursive(plan, &mut context);
        context.into_analysis_result()
    }
}
```

### 2.3 算法设计

#### 2.3.1 结构指纹计算

```rust
// src/query/optimizer/analysis/fingerprint.rs

/// 指纹计算器
pub struct FingerprintCalculator;

impl FingerprintCalculator {
    /// 计算计划节点的结构指纹
    /// 
    /// 使用稳定的哈希算法，相同结构的子计划产生相同指纹
    pub fn calculate_fingerprint(&self, node: &PlanNodeEnum) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // 哈希节点类型
        std::mem::discriminant(node).hash(&mut hasher);
        
        // 哈希子节点指纹（递归）
        self.hash_children(node, &mut hasher);
        
        // 哈希关键配置参数
        self.hash_node_config(node, &mut hasher);
        
        hasher.finish()
    }
    
    /// 哈希子节点
    fn hash_children(&self, node: &PlanNodeEnum, hasher: &mut impl Hasher) {
        match node {
            PlanNodeEnum::Filter(n) => {
                if let Some(input) = n.input() {
                    self.calculate_fingerprint(input).hash(hasher);
                }
            }
            PlanNodeEnum::HashInnerJoin(n) => {
                self.calculate_fingerprint(n.left_input()).hash(hasher);
                self.calculate_fingerprint(n.right_input()).hash(hasher);
            }
            // 其他节点类型...
            _ => {}
        }
    }
}
```

#### 2.3.2 引用计数分析流程

```rust
impl ReferenceCountAnalyzer {
    /// 递归分析计划树
    fn analyze_recursive(
        &self,
        node: &PlanNodeEnum,
        context: &mut AnalysisContext,
    ) -> u64 {
        // 计算当前节点指纹
        let fingerprint = self.fingerprint_calculator.calculate_fingerprint(node);
        let node_id = node.id();
        
        // 记录引用
        context.record_reference(fingerprint, node_id);
        
        // 递归分析子节点
        match node {
            PlanNodeEnum::Filter(n) => {
                if let Some(input) = n.input() {
                    self.analyze_recursive(input, context);
                }
            }
            PlanNodeEnum::HashInnerJoin(n) => {
                self.analyze_recursive(n.left_input(), context);
                self.analyze_recursive(n.right_input(), context);
            }
            // 其他节点类型...
            _ => {}
        }
        
        fingerprint
    }
}
```

### 2.4 使用场景

#### 2.4.1 CTE物化决策

```rust
// src/query/optimizer/strategy/materialization.rs

use crate::query::optimizer::analysis::ReferenceCountAnalyzer;

/// 基于引用计数的CTE物化决策
pub struct MaterializationStrategy {
    reference_analyzer: ReferenceCountAnalyzer,
}

impl MaterializationStrategy {
    pub fn should_materialize_cte(
        &self,
        cte_plan: &PlanNodeEnum,
    ) -> MaterializationDecision {
        let ref_analysis = self.reference_analyzer.analyze(cte_plan);
        
        // 获取CTE的引用信息
        if let Some(ref_info) = ref_analysis.node_reference_map.get(&cte_plan.id()) {
            // 被引用多次才考虑物化
            if ref_info.reference_count < 2 {
                return MaterializationDecision::DoNotMaterialize;
            }
            
            // 结果集不能太大
            if ref_info.estimated_output_rows > 10000 {
                return MaterializationDecision::DoNotMaterialize;
            }
            
            return MaterializationDecision::Materialize {
                estimated_size: ref_info.estimated_output_rows,
                reference_count: ref_info.reference_count,
            };
        }
        
        MaterializationDecision::DoNotMaterialize
    }
}
```

---

## 三、表达式分析功能设计

### 3.1 功能目标

分析表达式特性，为优化决策提供依据：
- 判断表达式是否确定性（是否包含非确定性函数）
- 分析表达式复杂度
- 提取表达式涉及的数据依赖

### 3.2 核心数据结构

```rust
// src/query/optimizer/analysis/expression.rs

/// 表达式分析结果
#[derive(Debug, Clone, Default)]
pub struct ExpressionAnalysis {
    /// 是否确定性（不含rand()、now()等非确定性函数）
    pub is_deterministic: bool,
    /// 复杂度评分（0-100）
    pub complexity_score: u32,
    /// 引用的属性列表
    pub referenced_properties: Vec<String>,
    /// 引用的变量列表
    pub referenced_variables: Vec<String>,
    /// 调用的函数列表
    pub called_functions: Vec<String>,
    /// 是否包含聚合函数
    pub contains_aggregate: bool,
    /// 是否包含子查询
    pub contains_subquery: bool,
    /// 表达式深度
    pub depth: u32,
    /// 节点数量
    pub node_count: u32,
}

/// 表达式分析选项
#[derive(Debug, Clone)]
pub struct AnalysisOptions {
    /// 分析确定性
    pub check_deterministic: bool,
    /// 分析复杂度
    pub check_complexity: bool,
    /// 提取属性引用
    pub extract_properties: bool,
    /// 提取变量引用
    pub extract_variables: bool,
    /// 统计函数调用
    pub count_functions: bool,
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            check_deterministic: true,
            check_complexity: true,
            extract_properties: true,
            extract_variables: true,
            count_functions: true,
        }
    }
}

/// 表达式分析器
pub struct ExpressionAnalyzer {
    /// 非确定性函数注册表
    nondeterministic_registry: NondeterministicFunctionRegistry,
    /// 分析选项
    options: AnalysisOptions,
}

impl ExpressionAnalyzer {
    /// 创建默认分析器
    pub fn new() -> Self {
        Self {
            nondeterministic_registry: NondeterministicFunctionRegistry::new(),
            options: AnalysisOptions::default(),
        }
    }
    
    /// 创建带选项的分析器
    pub fn with_options(options: AnalysisOptions) -> Self {
        Self {
            nondeterministic_registry: NondeterministicFunctionRegistry::new(),
            options,
        }
    }
    
    /// 分析表达式
    pub fn analyze(&self, expr: &Expression) -> ExpressionAnalysis {
        let mut result = ExpressionAnalysis::default();
        result.is_deterministic = true; // 默认假设是确定性的
        self.analyze_recursive(expr, &mut result, 0);
        result
    }
}
```

### 3.3 非确定性函数注册表

```rust
/// 非确定性函数注册表
pub struct NondeterministicFunctionRegistry {
    /// 非确定性函数名集合
    functions: HashSet<String>,
}

impl NondeterministicFunctionRegistry {
    pub fn new() -> Self {
        let mut functions = HashSet::new();
        
        // 时间相关函数
        functions.insert("now".to_string());
        functions.insert("current_time".to_string());
        functions.insert("current_date".to_string());
        functions.insert("current_timestamp".to_string());
        
        // 随机数函数
        functions.insert("rand".to_string());
        functions.insert("random".to_string());
        functions.insert("uuid".to_string());
        
        // 其他非确定性函数
        functions.insert("row_number".to_string());
        
        Self { functions }
    }
    
    pub fn is_nondeterministic(&self, func_name: &str) -> bool {
        self.functions.contains(func_name)
    }
    
    /// 注册自定义非确定性函数
    pub fn register(&mut self, func_name: String) {
        self.functions.insert(func_name);
    }
}
```

### 3.4 递归分析实现

```rust
impl ExpressionAnalyzer {
    fn analyze_recursive(
        &self,
        expr: &Expression,
        result: &mut ExpressionAnalysis,
        depth: u32,
    ) {
        result.depth = result.depth.max(depth);
        result.node_count += 1;
        
        match expr {
            Expression::Literal(_) => {
                // 字面量是确定性的，复杂度低
                if self.options.check_complexity {
                    result.complexity_score += 1;
                }
            }
            
            Expression::Variable(var) => {
                if self.options.extract_variables {
                    if !result.referenced_variables.contains(var) {
                        result.referenced_variables.push(var.clone());
                    }
                }
                if self.options.check_complexity {
                    result.complexity_score += 2;
                }
            }
            
            Expression::Property { property, .. } => {
                if self.options.extract_properties {
                    if !result.referenced_properties.contains(property) {
                        result.referenced_properties.push(property.clone());
                    }
                }
                if self.options.check_complexity {
                    result.complexity_score += 5;
                }
            }
            
            Expression::Function { name, args } => {
                if self.options.count_functions {
                    result.called_functions.push(name.clone());
                }
                
                // 检查是否非确定性
                if self.options.check_deterministic {
                    if self.nondeterministic_registry.is_nondeterministic(name) {
                        result.is_deterministic = false;
                    }
                }
                
                // 函数调用增加复杂度
                if self.options.check_complexity {
                    result.complexity_score += 10;
                    result.complexity_score += args.len() as u32 * 2;
                }
                
                // 递归分析参数
                for arg in args {
                    self.analyze_recursive(arg, result, depth + 1);
                }
            }
            
            Expression::Aggregate { .. } => {
                result.contains_aggregate = true;
                if self.options.check_complexity {
                    result.complexity_score += 20;
                }
            }
            
            Expression::Binary { left, right, .. } => {
                if self.options.check_complexity {
                    result.complexity_score += 2;
                }
                self.analyze_recursive(left, result, depth + 1);
                self.analyze_recursive(right, result, depth + 1);
            }
            
            Expression::Unary { operand, .. } => {
                if self.options.check_complexity {
                    result.complexity_score += 1;
                }
                self.analyze_recursive(operand, result, depth + 1);
            }
            
            Expression::Case { conditions, default, .. } => {
                if self.options.check_complexity {
                    result.complexity_score += conditions.len() as u32 * 5;
                }
                
                for (when, then) in conditions {
                    self.analyze_recursive(when, result, depth + 1);
                    self.analyze_recursive(then, result, depth + 1);
                }
                
                if let Some(default_expr) = default {
                    self.analyze_recursive(default_expr, result, depth + 1);
                }
            }
            
            Expression::ListComprehension { .. } => {
                result.contains_subquery = true;
                if self.options.check_complexity {
                    result.complexity_score += 30;
                }
            }
            
            _ => {}
        }
    }
}
```

### 3.5 使用场景

#### 3.5.1 物化决策中的确定性检查

```rust
// src/query/optimizer/strategy/materialization.rs

use crate::query::optimizer::analysis::ExpressionAnalyzer;

impl MaterializationStrategy {
    /// 检查子计划是否适合物化（要求所有表达式都是确定性的）
    pub fn check_materializable(
        &self,
        plan: &PlanNodeEnum,
        expression_analyzer: &ExpressionAnalyzer,
    ) -> bool {
        let mut checker = MaterializationChecker {
            analyzer: expression_analyzer,
            is_materializable: true,
        };
        
        self.visit_plan(plan, &mut checker);
        checker.is_materializable
    }
}

struct MaterializationChecker<'a> {
    analyzer: &'a ExpressionAnalyzer,
    is_materializable: bool,
}

impl<'a> PlanNodeVisitor for MaterializationChecker<'a> {
    type Result = ();
    
    fn visit_filter(&mut self, node: &FilterNode) -> Self::Result {
        if let Some(condition) = node.condition() {
            let analysis = self.analyzer.analyze(condition);
            if !analysis.is_deterministic {
                self.is_materializable = false;
            }
        }
    }
}
```

#### 3.5.2 索引选择中的复杂度评估

```rust
// src/query/optimizer/strategy/index.rs

use crate::query::optimizer::analysis::ExpressionAnalyzer;

impl IndexSelector {
    /// 评估过滤条件是否适合使用索引
    pub fn evaluate_index_suitability(
        &self,
        condition: &Expression,
        expression_analyzer: &ExpressionAnalyzer,
    ) -> IndexSuitability {
        let analysis = expression_analyzer.analyze(condition);
        
        // 复杂度过高的条件不适合索引
        if analysis.complexity_score > 50 {
            return IndexSuitability::TooComplex;
        }
        
        // 包含非确定性函数的条件不能使用索引
        if !analysis.is_deterministic {
            return IndexSuitability::Nondeterministic;
        }
        
        // 包含多个属性的条件选择性可能不好
        if analysis.referenced_properties.len() > 2 {
            return IndexSuitability::LowSelectivity;
        }
        
        IndexSuitability::Suitable
    }
}
```

---

## 四、模块导出与集成

### 4.1 模块入口

```rust
// src/query/optimizer/analysis/mod.rs

//! 计划分析模块
//!
//! 提供查询计划分析功能，支持优化决策：
//! - 引用计数分析：识别被多次引用的子计划
//! - 表达式分析：分析表达式特性（确定性、复杂度等）

pub mod reference_count;
pub mod expression;
pub mod fingerprint;

pub use reference_count::{
    ReferenceCountAnalyzer,
    ReferenceCountAnalysis,
    SubplanReferenceInfo,
    SubplanId,
};

pub use expression::{
    ExpressionAnalyzer,
    ExpressionAnalysis,
    AnalysisOptions,
    NondeterministicFunctionRegistry,
};

pub use fingerprint::{
    FingerprintCalculator,
    PlanFingerprint,
};
```

### 4.2 优化器模块入口更新

```rust
// src/query/optimizer/mod.rs

pub mod engine;
pub mod stats;
pub mod cost;
pub mod strategy;
pub mod decision;
pub mod analysis;  // 【新增】

// 重新导出分析类型
pub use analysis::{
    ReferenceCountAnalyzer,
    ExpressionAnalyzer,
    ExpressionAnalysis,
};
```

### 4.3 与优化器引擎集成

```rust
// src/query/optimizer/engine.rs

use crate::query::optimizer::analysis::{
    ReferenceCountAnalyzer,
    ExpressionAnalyzer,
};

pub struct OptimizerEngine {
    // 现有字段...
    stats_manager: Arc<StatisticsManager>,
    cost_calculator: CostCalculator,
    
    // 【新增】分析器
    reference_analyzer: ReferenceCountAnalyzer,
    expression_analyzer: ExpressionAnalyzer,
}

impl OptimizerEngine {
    pub fn new(stats_manager: Arc<StatisticsManager>) -> Self {
        Self {
            stats_manager: stats_manager.clone(),
            cost_calculator: CostCalculator::new(stats_manager.clone()),
            reference_analyzer: ReferenceCountAnalyzer::new(),
            expression_analyzer: ExpressionAnalyzer::new(),
        }
    }
    
    /// 计算优化决策（增强版）
    pub fn compute_decision(
        &self,
        plan: &ExecutionPlan,
        sentence_kind: SentenceKind,
    ) -> OptimizationDecision {
        // 1. 分析引用计数
        let ref_analysis = self.reference_analyzer.analyze(plan.root());
        
        // 2. 基于引用计数的优化决策
        let materialization_decisions = self.decide_materializations(
            plan,
            &ref_analysis,
        );
        
        // 3. 原有优化决策逻辑...
        let traversal_start = self.select_traversal_start(plan);
        let index_selection = self.select_indexes(plan);
        let join_order = self.optimize_join_order(plan);
        
        OptimizationDecision {
            traversal_start,
            index_selection,
            join_order,
            materialization_decisions,
            rewrite_rules: Vec::new(),
            stats_version: self.stats_manager.version(),
            index_version: 0,
            created_at: Instant::now(),
        }
    }
    
    /// 决定物化策略
    fn decide_materializations(
        &self,
        plan: &ExecutionPlan,
        ref_analysis: &ReferenceCountAnalysis,
    ) -> Vec<MaterializationDecision> {
        let mut decisions = Vec::new();
        
        for subplan_info in &ref_analysis.repeated_subplans {
            // 被引用次数不足，跳过
            if subplan_info.reference_count < 2 {
                continue;
            }
            
            // 结果集太大，跳过
            if subplan_info.estimated_output_rows > 10000 {
                continue;
            }
            
            decisions.push(MaterializationDecision::Materialize {
                subplan_id: subplan_info.subplan_id.clone(),
                estimated_rows: subplan_info.estimated_output_rows,
            });
        }
        
        decisions
    }
}
```

---

## 五、性能考虑

### 5.1 时间复杂度

| 功能 | 时间复杂度 | 说明 |
|------|-----------|------|
| 引用计数分析 | O(n) | n为计划节点数量，单次后序遍历 |
| 表达式分析 | O(m) | m为表达式节点数量 |
| 指纹计算 | O(1) | 基于节点类型和子节点哈希 |

### 5.2 空间复杂度

| 功能 | 空间复杂度 | 说明 |
|------|-----------|------|
| 引用计数分析 | O(n) | 存储指纹和引用信息 |
| 表达式分析 | O(k) | k为提取的属性/变量/函数数量 |

### 5.3 缓存策略

```rust
/// 分析结果缓存
pub struct AnalysisCache {
    /// 计划模板哈希到引用计数分析的映射
    reference_count_cache: LruCache<u64, ReferenceCountAnalysis>,
    /// 表达式哈希到分析结果的映射
    expression_cache: LruCache<u64, ExpressionAnalysis>,
}

impl AnalysisCache {
    /// 获取或计算引用计数分析
    pub fn get_or_compute_reference_count<F>(
        &mut self,
        plan: &PlanNodeEnum,
        compute: F,
    ) -> ReferenceCountAnalysis
    where
        F: FnOnce(&PlanNodeEnum) -> ReferenceCountAnalysis,
    {
        let plan_hash = self.hash_plan(plan);
        
        if let Some(result) = self.reference_count_cache.get(&plan_hash) {
            return result.clone();
        }
        
        let result = compute(plan);
        self.reference_count_cache.put(plan_hash, result.clone());
        result
    }
}
```

---

## 六、总结

### 6.1 模块归属确认

| 功能 | 建议位置 | 理由 |
|------|---------|------|
| 引用计数统计 | `src/query/optimizer/analysis/` | 优化阶段按需计算，与cost模块同级 |
| 表达式分析 | `src/query/optimizer/analysis/` | 优化阶段按需计算，与cost模块同级 |

### 6.2 与现有架构的关系

```
src/query/
├── planner/                    # 计划生成（不使用analysis）
│   ├── plan/
│   ├── rewrite/               # 启发式重写规则
│   └── ...
└── optimizer/                  # 计划优化（使用analysis）
    ├── stats/                 # 统计信息
    ├── cost/                  # 代价计算
    ├── analysis/              # 【新增】计划分析
    ├── strategy/              # 优化策略（依赖analysis）
    └── decision/              # 优化决策
```

### 6.3 对第3阶段任务的支持

引入这两个功能后，可以实现以下优化策略：

| 原任务 | 新策略 | 依赖功能 |
|--------|--------|----------|
| 物化策略选择 | 基于引用计数+确定性分析的CTE物化 | 引用计数 + 表达式分析 |
| 子查询去关联化 | 基于引用计数+复杂度分析的转换 | 引用计数 + 表达式分析 |

### 6.4 实现优先级

1. **高优先级**：表达式分析
   - 实现简单，立即可用
   - 支持多种优化决策

2. **中优先级**：引用计数统计
   - 需要指纹计算基础设施
   - 主要用于CTE物化场景

3. **低优先级**：分析结果缓存
   - 性能优化，可以后续添加
