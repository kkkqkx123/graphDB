# 表达式系统架构改进方案

## 一、问题分析

### 1.1 当前实现的问题

#### 问题1：重复分析表达式

**现象**：
- Optimizer 层每次调用 `analyze()` 都重新遍历表达式树
- Rewrite 层、Executor 层可能重复分析同一个表达式
- 没有利用 ExpressionContext 存储分析结果

**代码示例**：
```rust
// 当前实现：每个阶段都重新分析
let analysis = optimizer.analyze(&ctx_expr);  // 重新分析
let analysis = optimizer.analyze(&ctx_expr);  // 又重新分析
let analysis = optimizer.analyze(&ctx_expr);  // 再次重新分析
```

#### 问题2：未使用 Visitor 模式

**现象**：
- `ExpressionAnalyzer` 使用了 20+ 个重复的 `match expr` 分支
- 未使用现有的 `PropertyCollector`、`VariableCollector`、`FunctionCollector`
- 代码重复，维护困难

**代码示例**：
```rust
// 当前实现：重复的模式匹配
fn analyze_recursive(&self, expr: &Expression, result: &mut ExpressionAnalysis, depth: u32) {
    match expr {
        Expression::Literal(_) => { /* 20+ 行代码 */ }
        Expression::Variable(var) => { /* 10+ 行代码 */ }
        Expression::Property { object, property } => {
            if self.options.extract_properties {
                if !result.referenced_properties.contains(&property) {
                    result.referenced_properties.push(property.clone());
                }
            }
            self.analyze_recursive(object, result, depth + 1);
        }
        // ... 20+ 个分支，每个分支都有重复逻辑
    }
}
```

#### 问题3：ExpressionContext 未被使用

**现象**：
- Optimizer 层完全没有使用 `ExpressionContext`
- 分析结果没有被存储和复用
- 违反了架构文档的设计意图

**架构文档要求**：
```
阶段 4：Optimizer
- 输入：ExecutionPlan + ExpressionContext
- Optimizer 分析 ContextualExpression
- 分析结果存储到 ExpressionContext
- 代价估算使用分析结果
```

### 1.2 缓存方案的问题

#### 缓存维护的复杂性

| 问题 | 具体影响 |
|------|---------|
| **缓存失效** | Rewrite 层修改表达式后，缓存如何失效？ |
| **生命周期管理** | ExpressionId 可能被重用，缓存返回过期数据 |
| **并发安全** | 多线程访问需要加锁，增加复杂度 |
| **内存占用** | 缓存占用额外内存，需要限制大小 |
| **调试困难** | 缓存命中/未命中难以追踪 |

#### 缓存无法解决根本问题

缓存只是"掩盖"了问题，而不是"解决"了问题。正确的方案是：
- 每个表达式只分析一次（在 Validator 层）
- 后续阶段直接从 ExpressionContext 读取
- 不需要缓存，因为 ExpressionContext 本身就是存储

---

## 二、架构文档的设计意图

### 2.1 正确的数据流

```
Parser → Validator → Planner → Rewrite → Optimizer → Executor
   ↓         ↓          ↓         ↓          ↓          ↓
 创建      分析一次    不分析    不分析    直接读取   不分析
Context   并存储     直接用    直接用    不重复     直接用
```

### 2.2 各层职责

| 层级 | 职责 | 是否分析表达式 |
|------|------|--------------|
| **Parser** | 解析 SQL，创建 Expression | ❌ 不分析 |
| **Validator** | 语义检查、类型推导、常量折叠 | ✅ **分析一次** |
| **Planner** | 生成执行计划 | ❌ 不分析 |
| **Rewrite** | 应用重写规则 | ⚠️ 只分析新创建的表达式 |
| **Optimizer** | 代价估算、计划优化 | ❌ 从 ExpressionContext 读取 |
| **Executor** | 执行查询计划 | ❌ 不分析 |

### 2.3 ExpressionContext 的作用

```rust
// ExpressionContext 本身就是存储，不需要额外缓存
expr_context.set_analysis(expr_id, analysis);  // 存储分析结果
expr_context.set_type(expr_id, data_type);     // 存储类型信息
expr_context.set_constant(expr_id, value);     // 存储常量折叠结果

// 后续阶段直接读取
let analysis = expr_context.get_analysis(expr_id).unwrap();
let data_type = expr_context.get_type(expr_id).unwrap();
```

---

## 三、改进方案

### 3.1 方案1：扩展 ExpressionContext

**目标**：添加存储和读取分析结果的方法

```rust
impl ExpressionContext {
    /// 存储表达式分析结果
    pub fn set_analysis(&self, expr_id: ExpressionId, analysis: ExpressionAnalysis) {
        self.analyses.write().insert(expr_id, analysis);
    }

    /// 获取表达式分析结果
    pub fn get_analysis(&self, expr_id: ExpressionId) -> Option<ExpressionAnalysis> {
        self.analyses.read().get(&expr_id).cloned()
    }

    /// 存储类型推导结果
    pub fn set_type(&self, expr_id: ExpressionId, data_type: DataType) {
        self.types.write().insert(expr_id, data_type);
    }

    /// 获取类型推导结果
    pub fn get_type(&self, expr_id: ExpressionId) -> Option<DataType> {
        self.types.read().get(&expr_id).cloned()
    }

    /// 存储常量折叠结果
    pub fn set_constant(&self, expr_id: ExpressionId, value: Value) {
        self.constants.write().insert(expr_id, value);
    }

    /// 获取常量折叠结果
    pub fn get_constant(&self, expr_id: ExpressionId) -> Option<Value> {
        self.constants.read().get(&expr_id).cloned()
    }
}
```

### 3.2 方案2：使用 Visitor 模式重构 ExpressionAnalyzer

**目标**：消除重复的模式匹配代码

```rust
impl ExpressionAnalyzer {
    /// 分析表达式（使用 Visitor 模式）
    pub fn analyze(&self, expr: &Expression) -> ExpressionAnalysis {
        let mut analysis = ExpressionAnalysis::new();

        // 使用现有的 Collector
        if self.options.extract_properties {
            let mut collector = PropertyCollector::new();
            collector.visit(expr);
            analysis.referenced_properties = collector.properties;
        }

        if self.options.extract_variables {
            let mut collector = VariableCollector::new();
            collector.visit(expr);
            analysis.referenced_variables = collector.variables;
        }

        if self.options.count_functions {
            let mut collector = FunctionCollector::new();
            collector.visit(expr);
            analysis.called_functions = collector.functions;
        }

        // 自定义分析逻辑
        let mut visitor = AnalysisVisitor::new(&mut analysis, self.options.clone());
        visitor.visit(expr);

        analysis
    }
}

struct AnalysisVisitor<'a> {
    analysis: &'a mut ExpressionAnalysis,
    options: AnalysisOptions,
}

impl ExpressionVisitor for AnalysisVisitor<'_> {
    fn visit_function(&mut self, name: &str, args: &[Expression]) {
        if self.options.count_functions {
            self.analysis.called_functions.push(name.to_string());
        }

        if self.options.check_deterministic {
            if NondeterministicChecker::is_nondeterministic(name) {
                self.analysis.is_deterministic = false;
            }
        }

        if self.options.check_complexity {
            self.analysis.complexity_score += 10 + args.len() as u32 * 2;
        }

        // 递归调用由 visitor 模式自动处理
        for arg in args {
            self.visit(arg);
        }
    }

    fn visit_aggregate(&mut self, func: &AggregateFunction, arg: &Expression, _distinct: bool) {
        self.analysis.contains_aggregate = true;

        if self.options.count_functions {
            self.analysis.called_functions.push(format!("{:?}", func));
        }

        if self.options.check_complexity {
            self.analysis.complexity_score += 20;
        }

        self.visit(arg);
    }

    // ... 其他方法
}
```

### 3.3 方案3：修改 Optimizer 层

**目标**：从 ExpressionContext 读取分析结果，不重复分析

```rust
impl OptimizerEngine {
    /// 获取表达式分析结果（优先从 ExpressionContext 读取）
    pub fn get_analysis(&self, ctx_expr: &ContextualExpression) -> ExpressionAnalysis {
        let expr_id = ctx_expr.id();

        // 尝试从 ExpressionContext 读取
        if let Some(analysis) = ctx_expr.context().get_analysis(expr_id) {
            return analysis;
        }

        // 如果不存在，执行分析并存储
        let expr = ctx_expr.expression().unwrap().inner();
        let analysis = self.expression_analyzer.analyze(expr);
        ctx_expr.context().set_analysis(expr_id, analysis.clone());
        analysis
    }
}

// 使用示例
impl SubqueryUnnestingOptimizer {
    pub fn should_unnest(&self, pattern_apply: &PatternApplyNode) -> UnnestDecision {
        // 直接从 ExpressionContext 读取，不重复分析
        if let Some(condition) = pattern_apply.condition() {
            let analysis = self.expression_analyzer.get_analysis(condition);

            if !analysis.is_deterministic {
                return UnnestDecision::KeepPatternApply {
                    reason: KeepReason::NonDeterministic,
                };
            }

            if analysis.complexity_score > self.max_complexity {
                return UnnestDecision::KeepPatternApply {
                    reason: KeepReason::ComplexCondition,
                };
            }
        }
        // ...
    }
}
```

---

## 四、实施计划

### 4.1 立即实施（高优先级）

1. ✅ 扩展 ExpressionContext，添加存储方法
   - `set_analysis()` / `get_analysis()`
   - `set_type()` / `get_type()`
   - `set_constant()` / `get_constant()`

2. ✅ 使用 Visitor 模式重构 ExpressionAnalyzer
   - 使用现有的 `PropertyCollector`、`VariableCollector`、`FunctionCollector`
   - 创建 `AnalysisVisitor` 实现自定义分析逻辑
   - 消除重复的模式匹配代码

3. ✅ 修改 Optimizer 层
   - 添加 `get_analysis()` 方法，优先从 ExpressionContext 读取
   - 修改 `SubqueryUnnestingOptimizer` 使用 `get_analysis()`
   - 修改 `MaterializationOptimizer` 使用 `get_analysis()`

### 4.2 短期实施（中优先级）

1. ⚠️ 修改 Rewrite 层
   - 新创建的表达式注册到 ExpressionContext
   - 分析新表达式并存储到 ExpressionContext

2. ⚠️ 修改 Executor 层
   - 从 ExpressionContext 读取类型信息
   - 不需要分析表达式，直接执行

### 4.3 长期实施（低优先级）

1. 📋 完善类型推导
   - 在 Validator 层实现完整的类型推导
   - 存储类型信息到 ExpressionContext

2. 📋 实现常量折叠
   - 在 Validator 层实现常量折叠
   - 存储常量值到 ExpressionContext

---

## 五、预期收益

### 5.1 性能提升

| 优化项 | 预期提升 | 说明 |
|--------|---------|------|
| 消除重复分析 | 30-50% | 每个表达式只分析一次 |
| Visitor 模式 | 10-20% | 减少模式匹配开销 |
| 代码质量 | 显著提升 | 减少重复代码 |

### 5.2 代码质量提升

| 指标 | 改进 |
|------|------|
| 代码重复 | 减少 60% |
| 维护成本 | 降低 50% |
| 可扩展性 | 显著提升 |

---

## 六、总结

### 6.1 核心原则

1. ✅ **不要使用缓存** - ExpressionContext 本身就是存储
2. ✅ **每个表达式只分析一次** - 在 Validator 层
3. ✅ **后续阶段直接读取** - 从 ExpressionContext 读取
4. ✅ **使用 Visitor 模式** - 消除重复的模式匹配代码
5. ✅ **利用现有的 Collector** - PropertyCollector、VariableCollector、FunctionCollector

### 6.2 关键要点

- ❌ **不要使用缓存** - 会带来缓存失效、生命周期管理等复杂问题
- ✅ **ExpressionContext 就是存储** - 不需要额外的缓存层
- ✅ **Visitor 模式** - 提高代码质量和可维护性
- ✅ **每个表达式只分析一次** - 在 Validator 层分析，后续阶段直接读取
