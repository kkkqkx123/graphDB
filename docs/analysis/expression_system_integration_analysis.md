# 表达式系统集成分析报告

## 概述

本报告基于 `docs/architecture/expression_system_integration.md` 文档，对 `src/query/optimizer` 和 `src/query/executor` 目录的表达式系统集成情况进行详细分析。

## 分析日期

2026-03-03

## 分析范围

- `src/query/optimizer/` - 查询优化器层
- `src/query/executor/` - 查询执行器层

## 问题汇总

### 严重问题（5个）

| 序号 | 问题 | 位置 | 违反原则 |
|------|------|------|----------|
| 1 | OptimizerEngine 创建自己的 ExpressionContext | engine.rs:50 | 3.2 跨阶段共享 |
| 2 | ExpressionAnalyzer 接受 Expression | expression.rs:89 | 3.3 类型安全 |
| 3 | SubqueryUnnestingOptimizer 直接操作 Expression | subquery_unnesting.rs:189-191 | 5.2 禁止重复创建 |
| 4 | AggregateStrategySelector 持有独立 ExpressionContext | aggregate_strategy.rs:117 | 3.2 跨阶段共享 |
| 5 | ExecutionContext 缺少 ExpressionContext | execution_context.rs:13-14 | 3.4 信息完整 |

### 中等问题（6个）

| 序号 | 问题 | 位置 | 违反原则 |
|------|------|------|----------|
| 1 | JoinCondition 使用 Expression | join_order.rs:62 | 3.3 类型安全 |
| 2 | FilterExecutor 提取 Expression 方式不够优雅 | filter.rs:55-58 | 1.6 提取方式 |
| 3 | ProjectColumn 直接包含 Expression | projection.rs:26 | 3.3 类型安全 |
| 4 | InnerJoinExecutor 接受 Vec<Expression> | inner_join.rs:54 | 3.3 类型安全 |
| 5 | ExecutorFactory 提取 Expression 方式不够清晰 | factory.rs:398 | 1.6 提取方式 |
| 6 | AggregateFunctionSpec 使用 AggregateFunction | aggregation.rs:42 | 3.3 类型安全 |

---

## 详细问题分析

### 一、Optimizer 层问题

#### 1.1 OptimizerEngine 创建自己的 ExpressionContext ⚠️ 严重

**文件位置**：[src/query/optimizer/engine.rs:50](file:///d:/项目/database/graphDB/src/query/optimizer/engine.rs#L50)

**问题代码**：
```rust
pub fn new(cost_config: CostModelConfig) -> Self {
    // 创建表达式上下文
    let expression_context = Arc::new(ExpressionContext::new());
    // ...
}
```

**违反原则**：
- 文档 3.2 节："ExpressionContext 跨阶段共享"
- 文档 4.2 节："Validator → Planner：Planner 接收：ValidatedStatement + QueryContext (包含 ExpressionContext)"

**问题描述**：
OptimizerEngine 在构造函数中创建了自己的 ExpressionContext，而不是从 QueryContext 接收。这导致：
1. 不同查询使用不同的 ExpressionContext
2. 无法跨阶段共享表达式信息
3. 违反了单一数据源原则

**影响**：
- 无法利用 Parser 和 Validator 层的分析结果
- 可能导致重复分析表达式
- 无法实现跨阶段信息传递

**改进建议**：
```rust
impl OptimizerEngine {
    /// 使用共享的 ExpressionContext 创建优化器引擎
    pub fn with_expression_context(
        expression_context: Arc<ExpressionContext>,
        cost_config: CostModelConfig,
    ) -> Self {
        // 使用传入的 expression_context，而不是创建新的
        let stats_manager = Arc::new(StatisticsManager::new());
        let cost_calculator = Arc::new(CostCalculator::with_config(
            stats_manager.clone(),
            cost_config,
        ));
        // ... 其他初始化逻辑

        Self {
            expression_context,
            // ...
        }
    }
}
```

---

#### 1.2 ExpressionAnalyzer 接受 Expression 而非 ContextualExpression ⚠️ 严重

**文件位置**：[src/query/optimizer/analysis/expression.rs:89](file:///d:/项目/database/graphDB/src/query/optimizer/analysis/expression.rs#L89)

**问题代码**：
```rust
pub fn analyze(&self, expr: &Expression) -> ExpressionAnalysis {
    let mut analysis = ExpressionAnalysis::new();
    // ...
}
```

**违反原则**：
- 文档 3.1 节："Parser 层是唯一创建 Expression 的地方"
- 文档 3.3 节："Planner 层只能使用 ContextualExpression"

**问题描述**：
ExpressionAnalyzer 的 analyze 方法接受 `&Expression` 参数，导致：
1. 优化器层需要从 ContextualExpression 提取 Expression
2. 可能导致重复创建 Expression
3. 违反了类型安全原则

**影响**：
- 类型安全无法保证
- 可能导致重复解析
- 无法利用 ExpressionContext 的缓存

**改进建议**：
```rust
impl ExpressionAnalyzer {
    /// 分析表达式（接受 ContextualExpression）
    pub fn analyze(&self, ctx_expr: &ContextualExpression) -> ExpressionAnalysis {
        let mut analysis = ExpressionAnalysis::new();

        // 通过 ContextualExpression 获取 Expression
        if let Some(expr_meta) = ctx_expr.expression() {
            let expr = expr_meta.inner();

            // 使用现有的 Collector 收集信息
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

            // 使用自定义 Visitor 进行复杂度和确定性分析
            let mut visitor = AnalysisVisitor::new(&mut analysis, self.options.clone());
            visitor.visit(expr);
        }

        analysis
    }
}
```

---

#### 1.3 SubqueryUnnestingOptimizer 直接操作 Expression ⚠️ 严重

**文件位置**：[src/query/optimizer/strategy/subquery_unnesting.rs:189-191](file:///d:/项目/database/graphDB/src/query/optimizer/strategy/subquery_unnesting.rs#L189-L191)

**问题代码**：
```rust
for key_col in pattern_apply.key_cols() {
    if let Some(expr_meta) = key_col.expression() {
        let analysis = self.expression_analyzer.analyze(expr_meta.inner());
        // ...
    }
}
```

**违反原则**：
- 文档 5.2 节："禁止重复创建 Expression"
- 文档 5.4 节："所有层只使用 ContextualExpression"

**问题描述**：
SubqueryUnnestingOptimizer 直接从 ContextualExpression 提取 Expression 并传递给 ExpressionAnalyzer，这违反了"禁止重复创建 Expression"的原则。

**影响**：
- 违反类型安全
- 可能导致重复分析
- 代码不够清晰

**改进建议**：
```rust
for key_col in pattern_apply.key_cols() {
    // 直接传递 ContextualExpression 给 ExpressionAnalyzer
    let analysis = self.expression_analyzer.analyze(key_col);

    // 检查确定性
    if !analysis.is_deterministic {
        return UnnestDecision::KeepPatternApply {
            reason: KeepReason::NonDeterministic,
        };
    }

    // 检查复杂度
    if analysis.complexity_score > self.max_complexity {
        return UnnestDecision::KeepPatternApply {
            reason: KeepReason::ComplexCondition,
        };
    }
}
```

---

#### 1.4 AggregateStrategySelector 持有独立 ExpressionContext ⚠️ 严重

**文件位置**：[src/query/optimizer/strategy/aggregate_strategy.rs:117](file:///d:/项目/database/graphDB/src/query/optimizer/strategy/aggregate_strategy.rs#L117)

**问题代码**：
```rust
pub struct AggregateStrategySelector {
    cost_calculator: Arc<CostCalculator>,
    expression_analyzer: ExpressionAnalyzer,
    expression_context: Arc<ExpressionContext>,
}
```

**违反原则**：
- 文档 3.2 节："ExpressionContext 跨阶段共享"

**问题描述**：
AggregateStrategySelector 持有自己的 ExpressionContext，而不是共享全局的 ExpressionContext。

**影响**：
- 无法跨阶段共享表达式信息
- 可能导致重复分析
- 无法利用缓存

**改进建议**：
```rust
impl AggregateStrategySelector {
    /// 创建带共享表达式上下文的聚合策略选择器
    pub fn with_shared_context(
        cost_calculator: Arc<CostCalculator>,
        expression_analyzer: ExpressionAnalyzer,
        expression_context: Arc<ExpressionContext>,
    ) -> Self {
        Self {
            cost_calculator,
            expression_analyzer,
            expression_context,
        }
    }
}
```

---

#### 1.5 JoinCondition 使用 Expression ⚠️ 中等

**文件位置**：[src/query/optimizer/strategy/join_order.rs:62](file:///d:/项目/database/graphDB/src/query/optimizer/strategy/join_order.rs#L62)

**问题代码**：
```rust
pub struct JoinCondition {
    pub left_table: String,
    pub right_table: String,
    pub selectivity: f64,
    pub expression: Option<Expression>,
}
```

**违反原则**：
- 文档 3.3 节："Planner 层只能使用 ContextualExpression"

**问题描述**：
JoinCondition 结构体直接使用 Expression，应该使用 ContextualExpression。

**影响**：
- 类型安全无法保证
- 无法利用 ExpressionContext 的缓存

**改进建议**：
```rust
pub struct JoinCondition {
    pub left_table: String,
    pub right_table: String,
    pub selectivity: f64,
    pub expression: Option<ContextualExpression>,
}
```

---

### 二、Executor 层问题

#### 2.1 FilterExecutor 提取 Expression 方式不够优雅 ⚠️ 中等

**文件位置**：[src/query/executor/result_processing/filter.rs:55-58](file:///d:/项目/database/graphDB/src/query/executor/result_processing/filter.rs#L55-L58)

**问题代码**：
```rust
pub fn new(id: i64, storage: Arc<Mutex<S>>, condition: ContextualExpression) -> Self {
    // ...
    let expr = match condition.expression() {
        Some(meta) => meta.inner().clone(),
        None => Expression::Literal(Value::Null(NullType::Null)),
    };
    // ...
}
```

**违反原则**：
- 文档 1.6 节："Executor 层：从 ContextualExpression 提取 Expression"

**问题描述**：
虽然文档允许 Executor 层提取 Expression，但实现方式不够优雅，且没有利用 ExpressionContext 的缓存功能。

**影响**：
- 代码可读性差
- 无法利用缓存

**改进建议**：
```rust
impl<S: StorageClient + Send + 'static> FilterExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>, condition: ContextualExpression) -> Self {
        let base = BaseResultProcessor::new(
            id,
            "FilterExecutor".to_string(),
            "Filters query results based on specified conditions".to_string(),
            storage,
        );

        // 提取 Expression（文档允许）
        let expr = Self::extract_expression(&condition);

        Self {
            base,
            condition: expr,
            input_executor: None,
            parallel_config: ParallelConfig::default(),
        }
    }

    /// 从 ContextualExpression 提取 Expression 的辅助方法
    fn extract_expression(ctx_expr: &ContextualExpression) -> Expression {
        match ctx_expr.expression() {
            Some(meta) => meta.inner().clone(),
            None => Expression::Literal(Value::Null(NullType::Null)),
        }
    }
}
```

---

#### 2.2 ProjectColumn 直接包含 Expression ⚠️ 中等

**文件位置**：[src/query/executor/result_processing/projection.rs:26](file:///d:/项目/database/graphDB/src/query/executor/result_processing/projection.rs#L26)

**问题代码**：
```rust
pub struct ProjectionColumn {
    pub name: String,
    pub expression: Expression,
}
```

**违反原则**：
- 文档 1.6 节："Executor 层：从 ContextualExpression 提取 Expression"

**问题描述**：
ProjectionColumn 直接使用 Expression，应该使用 ContextualExpression 以保持类型安全。

**影响**：
- 类型安全无法保证
- 无法利用 ExpressionContext 的缓存

**改进建议**：
```rust
pub struct ProjectionColumn {
    pub name: String,
    pub expression: ContextualExpression,
}

impl ProjectionColumn {
    pub fn new(name: String, expression: ContextualExpression) -> Self {
        Self { name, expression }
    }
}
```

---

#### 2.3 InnerJoinExecutor 接受 Vec<Expression> ⚠️ 中等

**文件位置**：[src/query/executor/data_processing/join/inner_join.rs:54](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/inner_join.rs#L54)

**问题代码**：
```rust
pub struct InnerJoinExecutor<S: StorageClient> {
    base_executor: BaseJoinExecutor<S>,
    single_key_hash_table: Option<HashMap<Value, Vec<Vec<Value>>>>,
    multi_key_hash_table: Option<HashMap<Vec<Value>, Vec<Vec<Value>>>>,
    use_multi_key: bool,
}

impl<S: StorageClient> InnerJoinExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
        col_names: Vec<String>,
    ) -> Self {
        // ...
    }
}
```

**违反原则**：
- 文档 1.6 节："Executor 层：从 ContextualExpression 提取 Expression"

**问题描述**：
InnerJoinExecutor 构造函数直接接受 Vec<Expression>，应该接受 Vec<ContextualExpression>。

**影响**：
- 类型安全无法保证
- 无法利用 ExpressionContext 的缓存

**改进建议**：
```rust
impl<S: StorageClient> InnerJoinExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<ContextualExpression>,
        probe_keys: Vec<ContextualExpression>,
        col_names: Vec<String>,
    ) -> Self {
        let use_multi_key = hash_keys.len() > 1;

        // 提取 Expression（文档允许）
        let hash_exprs = Self::extract_expressions(&hash_keys);
        let probe_exprs = Self::extract_expressions(&probe_keys);

        Self {
            base_executor: BaseJoinExecutor::new(
                id, storage, left_var, right_var, hash_exprs, probe_exprs, col_names,
            ),
            single_key_hash_table: None,
            multi_key_hash_table: None,
            use_multi_key,
        }
    }

    /// 从 ContextualExpression 列表提取 Expression 列表的辅助方法
    fn extract_expressions(ctx_exprs: &[ContextualExpression]) -> Vec<Expression> {
        ctx_exprs
            .iter()
            .filter_map(|ctx_expr| ctx_expr.expression().map(|meta| meta.inner().clone()))
            .collect()
    }
}
```

---

#### 2.4 ExecutorFactory 提取 Expression 方式不够清晰 ⚠️ 中等

**文件位置**：[src/query/executor/factory.rs:398](file:///d:/项目/database/graphDB/src/query/executor/factory.rs#L398)

**问题代码**：
```rust
fn create_inner_join_executor<N>(
    &self,
    node: &N,
    storage: Arc<Mutex<S>>,
) -> Result<ExecutorEnum<S>, QueryError>
where
    N: JoinNode,
{
    let (left_var, right_var) = Self::extract_join_vars(node);

    let hash_keys: Vec<crate::core::Expression> = node
        .hash_keys()
        .iter()
        .filter_map(|ctx_expr| ctx_expr.get_expression())
        .collect();

    let probe_keys: Vec<crate::core::Expression> = node
        .probe_keys()
        .iter()
        .filter_map(|ctx_expr| ctx_expr.get_expression())
        .collect();

    // ...
}
```

**违反原则**：
- 文档 1.6 节："Executor 层：从 ContextualExpression 提取 Expression"

**问题描述**：
虽然文档允许 Executor 层提取 Expression，但使用 `filter_map` 和 `get_expression()` 的方式不够清晰。

**影响**：
- 代码可读性差
- 容易出错

**改进建议**：
```rust
impl<S: StorageClient + 'static> ExecutorFactory<S> {
    /// 从 ContextualExpression 列表提取 Expression 列表的辅助方法
    fn extract_expressions(ctx_exprs: &[ContextualExpression]) -> Vec<Expression> {
        ctx_exprs
            .iter()
            .filter_map(|ctx_expr| ctx_expr.expression().map(|meta| meta.inner().clone()))
            .collect()
    }

    /// 创建内连接执行器（通用方法）
    fn create_inner_join_executor<N>(
        &self,
        node: &N,
        storage: Arc<Mutex<S>>,
    ) -> Result<ExecutorEnum<S>, QueryError>
    where
        N: JoinNode,
    {
        let (left_var, right_var) = Self::extract_join_vars(node);

        // 使用辅助方法提取表达式
        let hash_keys = Self::extract_expressions(node.hash_keys());
        let probe_keys = Self::extract_expressions(node.probe_keys());

        let executor = InnerJoinExecutor::new(
            node.id(),
            storage,
            left_var,
            right_var,
            hash_keys,
            probe_keys,
            node.col_names().to_vec(),
        );
        Ok(ExecutorEnum::InnerJoin(executor))
    }
}
```

---

#### 2.5 ExecutionContext 缺少 ExpressionContext ⚠️ 严重

**文件位置**：[src/query/executor/base/execution_context.rs:13-14](file:///d:/项目/database/graphDB/src/query/executor/base/execution_context.rs#L13-L14)

**问题代码**：
```rust
pub struct ExecutionContext {
    pub results: HashMap<String, ExecutionResult>,
    pub variables: HashMap<String, crate::core::Value>,
}
```

**违反原则**：
- 文档 1.6 节："Executor 层：从 ContextualExpression 提取 Expression"
- 文档 3.4 节："ContextualExpression 保留所有上下文信息"

**问题描述**：
ExecutionContext 没有持有 ExpressionContext，导致：
1. 无法利用 ExpressionContext 的缓存功能
2. 表达式执行时缺少完整的上下文信息
3. 无法支持跨阶段信息传递

**影响**：
- 无法利用缓存
- 无法支持完整上下文
- 无法跨阶段信息传递

**改进建议**：
```rust
use std::collections::HashMap;
use std::sync::Arc;
use crate::core::types::expression::context::ExpressionContext;
use super::execution_result::ExecutionResult;

/// 执行上下文
///
/// 用于在执行器执行过程中存储中间结果和变量，支持执行器之间的数据传递。
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 中间结果存储
    pub results: HashMap<String, ExecutionResult>,
    /// 变量存储
    pub variables: HashMap<String, crate::core::Value>,
    /// 表达式上下文，用于跨阶段共享表达式信息和缓存
    pub expression_context: Arc<ExpressionContext>,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    pub fn new(expression_context: Arc<ExpressionContext>) -> Self {
        Self {
            results: HashMap::new(),
            variables: HashMap::new(),
            expression_context,
        }
    }

    /// 设置中间结果
    pub fn set_result(&mut self, name: String, result: ExecutionResult) {
        self.results.insert(name, result);
    }

    /// 获取中间结果
    pub fn get_result(&self, name: &str) -> Option<&ExecutionResult> {
        self.results.get(name)
    }

    /// 设置变量
    pub fn set_variable(&mut self, name: String, value: crate::core::Value) {
        self.variables.insert(name, value);
    }

    /// 获取变量
    pub fn get_variable(&self, name: &str) -> Option<&crate::core::Value> {
        self.variables.get(name)
    }

    /// 获取表达式上下文
    pub fn expression_context(&self) -> &Arc<ExpressionContext> {
        &self.expression_context
    }
}
```

---

#### 2.6 AggregateFunctionSpec 使用 AggregateFunction ⚠️ 中等

**文件位置**：[src/query/executor/result_processing/aggregation.rs:42](file:///d:/项目/database/graphDB/src/query/executor/result_processing/aggregation.rs#L42)

**问题代码**：
```rust
pub struct AggregateFunctionSpec {
    pub function: AggregateFunction,
    pub field: Option<String>,
    pub distinct: bool,
}
```

**违反原则**：
- 文档 1.6 节："Executor 层：从 ContextualExpression 提取 Expression"

**问题描述**：
AggregateFunctionSpec 直接使用 AggregateFunction，应该使用 ContextualExpression。

**影响**：
- 类型安全无法保证
- 无法利用 ExpressionContext 的缓存

**改进建议**：
```rust
pub struct AggregateFunctionSpec {
    pub expression: ContextualExpression,
    pub distinct: bool,
}

impl AggregateFunctionSpec {
    pub fn new(expression: ContextualExpression) -> Self {
        Self {
            expression,
            distinct: false,
        }
    }

    pub fn with_distinct(mut self) -> Self {
        self.distinct = true;
        self
    }
}
```

---

## 修复优先级

### 第一优先级（严重问题，必须修复）

1. **OptimizerEngine 创建自己的 ExpressionContext**
   - 文件：`src/query/optimizer/engine.rs`
   - 修改：从 QueryContext 接收 ExpressionContext

2. **ExpressionAnalyzer 接受 Expression**
   - 文件：`src/query/optimizer/analysis/expression.rs`
   - 修改：接受 ContextualExpression

3. **SubqueryUnnestingOptimizer 直接操作 Expression**
   - 文件：`src/query/optimizer/strategy/subquery_unnesting.rs`
   - 修改：直接传递 ContextualExpression

4. **AggregateStrategySelector 持有独立 ExpressionContext**
   - 文件：`src/query/optimizer/strategy/aggregate_strategy.rs`
   - 修改：使用共享的 ExpressionContext

5. **ExecutionContext 缺少 ExpressionContext**
   - 文件：`src/query/executor/base/execution_context.rs`
   - 修改：添加 ExpressionContext 字段

### 第二优先级（中等问题，建议修复）

1. **JoinCondition 使用 Expression**
   - 文件：`src/query/optimizer/strategy/join_order.rs`
   - 修改：使用 ContextualExpression

2. **ProjectColumn 直接包含 Expression**
   - 文件：`src/query/executor/result_processing/projection.rs`
   - 修改：使用 ContextualExpression

3. **InnerJoinExecutor 接受 Vec<Expression>**
   - 文件：`src/query/executor/data_processing/join/inner_join.rs`
   - 修改：接受 Vec<ContextualExpression>

4. **AggregateFunctionSpec 使用 AggregateFunction**
   - 文件：`src/query/executor/result_processing/aggregation.rs`
   - 修改：使用 ContextualExpression

5. **FilterExecutor 提取 Expression 方式不够优雅**
   - 文件：`src/query/executor/result_processing/filter.rs`
   - 修改：提供辅助方法

6. **ExecutorFactory 提取 Expression 方式不够清晰**
   - 文件：`src/query/executor/factory.rs`
   - 修改：提供辅助方法

---

## 核心问题根源

1. **ExpressionContext 未跨阶段共享**
   - OptimizerEngine 和 AggregateStrategySelector 都创建了自己的 ExpressionContext
   - 应该从 QueryContext 接收共享的 ExpressionContext

2. **直接使用 Expression**
   - 多个地方直接使用 Expression 而不是 ContextualExpression
   - 应该使用 ContextualExpression 以保证类型安全

3. **ExpressionAnalyzer 接口不当**
   - 接受 Expression 而不是 ContextualExpression
   - 应该修改为接受 ContextualExpression

4. **Executor 层缺少 ExpressionContext**
   - ExecutionContext 没有持有 ExpressionContext
   - 应该添加 ExpressionContext 字段

---

## 修复建议

### 短期修复（高优先级）

1. 修改 OptimizerEngine，从 QueryContext 接收 ExpressionContext
2. 修改 ExpressionAnalyzer，接受 ContextualExpression
3. 修改 ExecutionContext，添加 ExpressionContext 字段
4. 修改 SubqueryUnnestingOptimizer，直接传递 ContextualExpression
5. 修改 AggregateStrategySelector，使用共享的 ExpressionContext

### 中期修复（中优先级）

1. 修改所有使用 Expression 的地方为 ContextualExpression
2. 统一 Expression 提取方式，提供辅助方法
3. 更新所有优化器组件，使用共享的 ExpressionContext

### 长期优化（低优先级）

1. 利用 ExpressionContext 的缓存功能，提高性能
2. 添加类型检查，确保编译时类型安全
3. 完善文档，明确各层职责

---

## 总结

本次分析发现了 12 个主要问题，其中 5 个严重问题、6 个中等问题。这些问题如果不修复，将导致：

- 无法跨阶段共享表达式信息
- 可能重复解析和创建 Expression
- 类型安全无法保证
- 无法利用 ExpressionContext 的缓存功能

建议按照优先级逐步修复这些问题，以确保表达式系统的正确集成。

---

## 修复进度

### 已完成修复（高优先级）

#### 1. ✅ OptimizerEngine 的 ExpressionContext 问题
- **文件**：[src/query/optimizer/engine.rs](file:///d:/项目/database/graphDB/src/query/optimizer/engine.rs)
- **修复**：添加了 `with_expression_context` 方法，接受共享的 ExpressionContext
- **状态**：已完成

#### 2. ✅ ExpressionAnalyzer 接受 ContextualExpression
- **文件**：[src/query/optimizer/analysis/expression.rs](file:///d:/项目/database/graphDB/src/query/optimizer/analysis/expression.rs)
- **修复**：
  - 修改 `analyze` 方法接受 `&ContextualExpression`
  - 删除了 `analyze_expression` 向后兼容方法
  - 更新了所有测试用例
- **状态**：已完成

#### 3. ✅ SubqueryUnnestingOptimizer 直接操作 Expression
- **文件**：[src/query/optimizer/strategy/subquery_unnesting.rs](file:///d:/项目/database/graphDB/src/query/optimizer/strategy/subquery_unnesting.rs)
- **修复**：直接传递 ContextualExpression 给 ExpressionAnalyzer
- **状态**：已完成

#### 4. ✅ AggregateStrategySelector 的 ExpressionContext
- **文件**：[src/query/optimizer/strategy/aggregate_strategy.rs](file:///d:/项目/database/graphDB/src/query/optimizer/strategy/aggregate_strategy.rs)
- **修复**：
  - 修改 `analyze_and_create_context` 方法接受 `&[ContextualExpression]`
  - 删除了 `analyze_and_create_context_with_expressions` 向后兼容方法
- **状态**：已完成

#### 5. ✅ ExecutionContext 添加 ExpressionContext
- **文件**：[src/query/executor/base/execution_context.rs](file:///d:/项目/database/graphDB/src/query/executor/base/execution_context.rs)
- **修复**：
  - 添加了 `expression_context: Arc<ExpressionContext>` 字段
  - 修改 `new` 方法接受 `Arc<ExpressionContext>` 参数
  - 添加了 `expression_context()` 访问方法
- **状态**：已完成

#### 6. ✅ BaseExecutor 构造函数修改
- **文件**：[src/query/executor/base/executor_base.rs](file:///d:/项目/database/graphDB/src/query/executor/base/executor_base.rs)
- **修复**：
  - 修改 `new` 方法接受 `Arc<ExpressionContext>` 参数
  - 修改 `without_storage` 方法接受 `Arc<ExpressionContext>` 参数
- **状态**：已完成

#### 7. ✅ 数据访问执行器修改
- **文件**：[src/query/executor/data_access.rs](file:///d:/项目/database/graphDB/src/query/executor/data_access.rs)
- **修复**：修改了以下执行器的 `new` 方法，添加 `expr_context: Arc<ExpressionContext>` 参数：
  - GetVerticesExecutor
  - GetEdgesExecutor
  - ScanEdgesExecutor
  - GetNeighborsExecutor
  - GetPropExecutor
  - IndexScanExecutor
  - AllPathsExecutor
  - ScanVerticesExecutor
- **状态**：已完成

#### 8. ✅ ExecutorFactory 修改
- **文件**：[src/query/executor/factory.rs](file:///d:/项目/database/graphDB/src/query/executor/factory.rs)
- **修复**：
  - 修改 `execute_plan` 方法，使用 `query_context.expr_context().clone()` 创建 ExecutionContext
  - 修改 `create_executor` 方法中所有执行器创建调用，传递 `context.expression_context().clone()`
- **状态**：已完成

#### 9. ✅ 结果处理执行器修改
- **文件**：
  - [src/query/executor/result_processing/transformations/rollup_apply.rs](file:///d:/项目/database/graphDB/src/query/executor/result_processing/transformations/rollup_apply.rs)
  - [src/query/executor/result_processing/transformations/unwind.rs](file:///d:/项目/database/graphDB/src/query/executor/result_processing/transformations/unwind.rs)
- **修复**：修改了以下执行器的 `new` 方法，添加 `expr_context: Arc<ExpressionContext>` 参数：
  - RollUpApplyExecutor
  - UnwindExecutor
- **状态**：已完成

#### 10. ✅ 图遍历执行器修改
- **文件**：[src/query/executor/data_processing/graph_traversal/expand_all.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/graph_traversal/expand_all.rs)
- **修复**：修改了以下执行器的 `new` 方法，添加 `expr_context: Arc<ExpressionContext>` 参数：
  - ExpandAllExecutor
- **状态**：已完成

#### 11. ✅ 特殊执行器修改
- **文件**：[src/query/executor/special_executors.rs](file:///d:/项目/database/graphDB/src/query/executor/special_executors.rs)
- **修复**：修改了以下执行器的 `new` 方法，添加 `expr_context: Arc<ExpressionContext>` 参数：
  - ArgumentExecutor
  - PassThroughExecutor
  - DataCollectExecutor
- **状态**：已完成

#### 12. ✅ 搜索执行器修改
- **文件**：[src/query/executor/search_executors.rs](file:///d:/项目/database/graphDB/src/query/executor/search_executors.rs)
- **修复**：修改了以下执行器的 `new` 方法，添加 `expr_context: Arc<ExpressionContext>` 参数：
  - BFSShortestExecutor
  - IndexScanExecutor (search_executors.rs 中的版本)
- **状态**：已完成

#### 13. ✅ 结果处理执行器（续）
- **文件**：
  - [src/query/executor/result_processing/transformations/pattern_apply.rs](file:///d:/项目/database/graphDB/src/query/executor/result_processing/transformations/pattern_apply.rs)
  - [src/query/executor/result_processing/transformations/append_vertices.rs](file:///d:/项目/database/graphDB/src/query/executor/result_processing/transformations/append_vertices.rs)
  - [src/query/executor/result_processing/projection.rs](file:///d:/项目/database/graphDB/src/query/executor/result_processing/projection.rs)
- **修复**：修改了以下执行器的 `new` 方法，添加 `expr_context: Arc<ExpressionContext>` 参数：
  - PatternApplyExecutor
  - AppendVerticesExecutor
  - ProjectExecutor
- **状态**：已完成

#### 14. ✅ 连接执行器修改
- **文件**：
  - [src/query/executor/data_processing/join/base_join.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/base_join.rs)
  - [src/query/executor/data_processing/join/inner_join.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/inner_join.rs)
  - [src/query/executor/data_processing/join/cross_join.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/cross_join.rs)
  - [src/query/executor/data_processing/join/left_join.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/left_join.rs)
  - [src/query/executor/data_processing/join/full_outer_join.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/full_outer_join.rs)
- **修复**：修改了以下执行器的 `new` 方法，添加 `expr_context: Arc<ExpressionContext>` 参数：
  - BaseJoinExecutor
  - InnerJoinExecutor
  - CrossJoinExecutor
  - LeftJoinExecutor
  - FullOuterJoinExecutor
- **状态**：已完成

#### 15. ✅ 集合操作执行器修改
- **文件**：
  - [src/query/executor/data_processing/set_operations/base.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/set_operations/base.rs)
  - [src/query/executor/data_processing/set_operations/union.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/set_operations/union.rs)
  - [src/query/executor/data_processing/set_operations/union_all.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/set_operations/union_all.rs)
  - [src/query/executor/data_processing/set_operations/intersect.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/set_operations/intersect.rs)
  - [src/query/executor/data_processing/set_operations/minus.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/set_operations/minus.rs)
- **修复**：修改了以下执行器的 `new` 方法，添加 `expr_context: Arc<ExpressionContext>` 参数：
  - SetExecutor (基类)
  - UnionExecutor
  - UnionAllExecutor
  - IntersectExecutor
  - MinusExecutor
- **状态**：已完成

#### 16. ✅ 类型冲突修复
- **文件**：
  - [src/query/executor/result_processing/transformations/rollup_apply.rs](file:///d:/项目/database/graphDB/src/query/executor/result_processing/transformations/rollup_apply.rs)
  - [src/query/executor/data_processing/join/base_join.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/base_join.rs)
  - [src/query/executor/data_processing/join/inner_join.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/inner_join.rs)
  - [src/query/executor/data_processing/join/cross_join.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/cross_join.rs)
  - [src/query/executor/data_processing/join/left_join.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/left_join.rs)
  - [src/query/executor/data_processing/join/full_outer_join.rs](file:///d:/项目/database/graphDB/src/query/executor/data_processing/join/full_outer_join.rs)
  - [src/query/executor/result_processing/transformations/pattern_apply.rs](file:///d:/项目/database/graphDB/src/query/executor/result_processing/transformations/pattern_apply.rs)
  - [src/query/executor/result_processing/filter.rs](file:///d:/项目/database/graphDB/src/query/executor/result_processing/filter.rs)
- **修复**：解决了 ExpressionContext trait 和 ExpressionContext struct 的命名冲突：
  - 使用 `use crate::core::types::expression::context::ExpressionContext as ExpressionContextStruct;` 导入 struct
  - 使用 `use crate::expression::evaluator::traits::ExpressionContext;` 导入 trait
  - 更新所有函数签名使用正确的类型
- **状态**：已完成

### 编译状态

- ✅ 当前代码编译通过（无错误）
- ⚠️ 有一些未使用的导入警告，不影响功能

### 已修复的执行器总数

- 数据访问执行器：8个
- 结果处理执行器：6个
- 图遍历执行器：1个
- 特殊执行器：3个
- 搜索执行器：2个
- 连接执行器：5个
- 集合操作执行器：5个
- **总计：30个执行器**

### 待修复（中优先级）

1. **JoinCondition 使用 ContextualExpression** - 需要修改连接条件类型
2. **ProjectColumn 使用 ContextualExpression** - 需要修改投影列类型
3. **InnerJoinExecutor 接受 ContextualExpression** - 需要修改内连接执行器
4. **AggregateFunctionSpec 使用 ContextualExpression** - 需要修改聚合函数规范
5. **其他执行器修改** - 还有约 80+ 个执行器需要修改 new 方法签名

### 编译状态

- ✅ 当前代码编译通过
- ⚠️ 仍有大量执行器需要修改以支持 ExpressionContext

### 下一步工作

1. 继续修改剩余的执行器，使其支持 ExpressionContext
2. 修改 JoinCondition、ProjectColumn 等类型，使用 ContextualExpression
3. 更新所有相关的测试用例
4. 运行完整的测试套件，确保没有破坏现有功能
