# 查询处理流程 AST 转换与数据传递分析

## 概述

本文档深入分析 GraphDB 查询处理流程中各个阶段的 AST 转换和数据传递情况。通过追踪数据在各阶段之间的流转，识别哪些步骤对数据结构进行了实质性转换，哪些步骤仅作为数据传递的通道。这一分析对于理解系统架构、定位性能瓶颈以及优化数据流具有重要意义。

## 查询处理管道总览

GraphDB 查询引擎采用五阶段管道架构，每个阶段都有明确的职责边界。QueryPipelineManager 作为协调者，按顺序调用各个阶段：

```rust
pub async fn execute_query(&mut self, query_text: &str) -> DBResult<ExecutionResult> {
    // 1. 创建查询上下文
    let mut query_context = self.create_query_context(query_text)?;

    // 2. 解析查询并生成 AST 上下文
    let ast = self.parse_into_context(query_text)?;

    // 3. 验证查询
    self.validate_query(&mut query_context, &ast)?;

    // 4. 生成执行计划
    let execution_plan = self.generate_execution_plan(&mut query_context, &ast)?;

    // 5. 优化执行计划
    let optimized_plan = self.optimize_execution_plan(&mut query_context, execution_plan)?;

    // 6. 执行计划
    self.execute_plan(&mut query_context, optimized_plan).await
}
```

## 阶段详细分析

### 第一阶段：Parser（解析阶段）

**位置**: `src/query/parser/`

**输入**: 查询文本字符串

**处理过程**:
1. 词法分析（Lexer）：将查询文本分解为 Token 序列
2. 语法分析（Parser）：根据语法规则将 Token 序列构建为 AST

**核心代码**:

```rust
fn parse_into_context(
    &mut self,
    query_text: &str,
) -> DBResult<crate::query::context::ast::QueryAstContext> {
    let mut parser = Parser::new(query_text);
    match parser.parse() {
        Ok(stmt) => {
            let mut ast = crate::query::context::ast::QueryAstContext::new(query_text);
            ast.set_statement(stmt);  // 将解析出的 Stmt 设置到上下文中
            Ok(ast)
        }
        Err(e) => Err(DBError::Query(...)),
    }
}
```

**AST 转换**: **是**
- 转换类型：`String` → `QueryAstContext`（包含 `Stmt`）
- 数据结构发生变化：从扁平的文本变为树状的抽象语法树
- 关键产出：`Stmt` 枚举，包含语句类型信息

**输出数据结构**:

```
QueryAstContext
├── base: AstContext
│   ├── query_text: String
│   ├── statement_type: String  // "MATCH", "GO" 等
│   └── sentence: Option<Stmt>  // 解析后的语句 AST
├── dependencies: HashMap<String, Vec<String>>
├── query_variables: HashMap<String, VariableInfo>
└── expression_contexts: Vec<ExpressionContext>
```

**数据传递**: 创建新的上下文对象，未使用传入的 query_context

---

### 第二阶段：Validator（验证阶段）

**位置**: `src/query/validator/`

**输入**: `QueryAstContext`（包含解析后的 Stmt）

**处理过程**:
1. 生命周期检查（space_chosen）
2. 具体验证逻辑（validate_impl）
3. 权限检查（check_permission）
4. 生成执行计划（to_plan）

**核心代码**:

```rust
fn validate_query(
    &mut self,
    _query_context: &mut QueryContext,
    ast: &crate::query::context::ast::QueryAstContext,
) -> DBResult<()> {
    let _stmt = ast.base_context().sentence().ok_or_else(|| {
        DBError::Query(...)
    })?;
    self.validator.validate_unified().map_err(|e| {
        DBError::Query(...)
    })
}
```

**AST 转换**: **部分转换**
- **问题识别**: Validator 接收 QueryAstContext，但实际使用的是内部创建的 `ValidationContext`
- 输入的 `ast` 仅用于提取语句检查是否存在，未进行深度转换
- 验证结果存储在独立的 `ValidationContext` 中，而非增强后的 `QueryAstContext`

**验证上下文转换**:

```
输入: QueryAstContext
      └── AstContext
          └── Stmt (原始解析结果)

内部创建: ValidationContext
      ├── space: Option<SpaceInfo>
      ├── inputs: Vec<ColumnDef>
      ├── outputs: Vec<ColumnDef>
      ├── expr_props: ExpressionProps
      └── validation_errors: Vec<ValidationError>

输出: ValidationResult (通过/失败)
```

**数据传递**: **仅传递指针**
- `query_context` 参数未被使用（标注为 `_query_context`）
- `ast` 参数仅用于存在性检查
- 验证状态未回流到 `QueryAstContext`

---

### 第三阶段：Planner（规划阶段）

**位置**: `src/query/planner/`

**输入**: `QueryAstContext`（验证后的）

**处理过程**:
1. 从 AST 提取语句类型（SentenceKind）
2. 根据语句类型选择对应的规划器
3. 调用具体规划器生成执行计划

**核心代码**:

```rust
fn generate_execution_plan(
    &mut self,
    _query_context: &mut QueryContext,
    ast: &crate::query::context::ast::QueryAstContext,
) -> DBResult<crate::query::planner::plan::ExecutionPlan> {
    let ast_ctx = ast.base_context();  // 提取 AstContext
    match self.planner.transform(ast_ctx) {  // 使用 AstContext 而非 QueryAstContext
        Ok(sub_plan) => {
            let mut plan = ExecutionPlan::new(sub_plan.root().clone());
            // ... 设置计划 ID
            Ok(plan)
        }
        Err(e) => Err(DBError::Query(...)),
    }
}
```

**AST 转换**: **是**
- 转换类型：`Stmt` → `SubPlan` → `ExecutionPlan`
- 从语句 AST 转换为计划节点树
- 关键方法：`MatchPlanner::transform()`、`GoPlanner::transform()` 等

**规划器注册与选择**:

```rust
pub struct PlannerRegistry {
    planners: HashMap<SentenceKind, Vec<MatchAndInstantiate>>,
}

pub enum SentenceKind {
    Match, Go, Lookup, Path, Subgraph, FetchVertices, FetchEdges, Maintain,
}

pub trait Planner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
}
```

**计划节点转换示例**（以 MatchPlanner 为例）:

```
输入: MATCH (n:Tag) WHERE n.age > 20 RETURN n.name
      └── MatchStmt
          ├── pattern: (n:Tag)
          └── where: Some(n.age > 20)
          └── return: Some(...)

输出: ExecutionPlan
      └── root: PlanNodeEnum::Project
              └── input: PlanNodeEnum::Filter
                      └── condition: n.age > 20
                      └── input: PlanNodeEnum::ScanVertices
                              └── tag_filter: Some(Tag)
```

**数据传递**: **部分传递**
- `query_context` 参数未被使用
- 使用 `ast.base_context()` 提取 `AstContext`，而非直接使用 `QueryAstContext`

---

### 第四阶段：Optimizer（优化阶段）

**位置**: `src/query/optimizer/`

**输入**: `ExecutionPlan`（规划后的原始计划）

**处理过程**:
1. 计划转换：将 `ExecutionPlan` 转换为 `OptGroup`
2. 多阶段优化：Logical → Physical → PostOptimization
3. 应用优化规则：谓词下推、投影下推、连接优化等
4. 计划转换回：将 `OptGroup` 转换回 `ExecutionPlan`

**核心代码**:

```rust
pub fn find_best_plan(
    &mut self,
    qctx: &mut QueryContext,
    plan: ExecutionPlan,
) -> Result<ExecutionPlan, OptimizerError> {
    let mut opt_ctx = OptContext::new(qctx.clone());  // 克隆 QueryContext

    let mut root_group = self.plan_to_group(&plan)?;  // 计划转换
    root_group.root_group = true;

    self.execute_phase_optimization(&mut opt_ctx, &mut root_group, OptimizationPhase::LogicalOptimization)?;
    self.execute_phase_optimization(&mut opt_ctx, &mut root_group, OptimizationPhase::PhysicalOptimization)?;
    self.execute_phase_optimization(&mut opt_ctx, &mut root_group, OptimizationPhase::PostOptimization)?;

    self.post_process(&mut opt_ctx, &mut root_group)?;

    let optimized_plan = self.group_to_plan(&root_group)?;  // 转换回计划

    Ok(optimized_plan)
}
```

**AST 转换**: **是（针对计划节点）**
- 转换类型：`ExecutionPlan` → `OptGroup` → 优化后 `OptGroup` → `ExecutionPlan`
- 数据结构从树形计划变为优化器的分组表示（OptGroup）

**计划转换方法**:

```rust
fn plan_to_group(&self, plan: &ExecutionPlan) -> Result<OptGroup, OptimizerError> {
    if let Some(root_node) = &plan.root {
        let mut group = OptGroup::new(0, false);
        self.convert_node_to_group(root_node, &mut group, 0)?;
        Ok(group)
    } else {
        Err(OptimizerError::PlanConversionError(...))
    }
}

fn convert_node_to_group(
    &self,
    node: &PlanNodeEnum,
    group: &mut OptGroup,
    node_id: usize,
) -> Result<(), OptimizerError> {
    let opt_node = OptGroupNode::new(node_id, node.clone());
    group.nodes.push(opt_node);

    for (i, dep) in node.dependencies().iter().enumerate() {
        self.convert_node_to_group(dep, group, node_id + i + 1)?;
    }
    Ok(())
}
```

**优化阶段详情**:

```
阶段 1: LogicalOptimization (逻辑优化)
├── FilterPushDownRule      - 过滤条件下推
├── PredicatePushDownRule   - 谓词下推
├── ProjectionPushDownRule  - 投影下推
├── CombineFilterRule       - 过滤条件合并
└── DedupEliminationRule    - 去重消除

阶段 2: PhysicalOptimization (物理优化)
├── JoinOptimizationRule    - 连接优化
├── PushLimitDownRule       - Limit 下推
├── IndexScanRule           - 索引扫描优化
└── ScanWithFilterOptimizationRule

阶段 3: PostOptimization (后优化)
└── TopNRule
```

**数据传递**: **完整传递**
- 使用 `qctx.clone()` 传递 QueryContext
- `opt_ctx` 持有 QueryContext 的克隆

---

### 第五阶段：Executor（执行阶段）

**位置**: `src/query/executor/`

**输入**: 优化后的 `ExecutionPlan`

**处理过程**:
1. 从计划根节点开始
2. 递归创建执行器链
3. 按依赖顺序执行

**核心代码**:

```rust
async fn execute_plan(
    &mut self,
    query_context: &mut QueryContext,
    plan: crate::query::planner::plan::ExecutionPlan,
) -> DBResult<ExecutionResult> {
    self.executor_factory
        .execute_plan(query_context, plan)
        .await
}
```

**执行器工厂创建执行器**:

```rust
pub fn create_executor(
    &self,
    plan_node: &PlanNodeEnum,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<Box<dyn Executor<S>>, QueryError> {
    match plan_node {
        PlanNodeEnum::Start(node) => Ok(Box::new(StartExecutor::new(node.id()))),
        PlanNodeEnum::ScanVertices(node) => {
            let executor = GetVerticesExecutor::new(node.id(), storage, ...);
            Ok(Box::new(executor))
        }
        PlanNodeEnum::Filter(node) => {
            let executor = FilterExecutor::new(node.id(), storage, node.condition().clone());
            Ok(Box::new(executor))
        }
        // ... 更多节点类型
    }
}
```

**AST 转换**: **是（计划到结果）**
- 转换类型：`ExecutionPlan` → `ExecutionResult`
- 计划节点被实例化为执行器
- 执行器产生最终结果

**执行器执行流程**:

```
ExecutionPlan (优化后的计划)
    └── root: Project
            └── input: Filter
                    └── condition: n.age > 20
                    └── input: GetNeighbors
                            └── src_vids: ["1"]
                            └── direction: Both

执行器链:
    GetNeighborsExecutor.execute()
        ↓ (产生中间结果)
    FilterExecutor.execute()
        ↓ (过滤数据)
    ProjectExecutor.execute()
        ↓ (投影列)
    ExecutionResult
```

**数据传递**: **完整传递**
- `query_context` 传递给执行器工厂
- 存储引擎通过 `storage` 参数传递

---

## 数据传递矩阵

### 参数使用情况

| 阶段 | query_context | ast/plan | 使用情况 |
|------|---------------|----------|----------|
| Parser | 创建新对象 | - | 未使用传入参数 |
| Validator | `_query_context` (未使用) | `ast` (仅检查存在性) | 仅部分使用 |
| Planner | `_query_context` (未使用) | `ast` (提取 AstContext) | 部分使用 |
| Optimizer | `qctx.clone()` (完整使用) | `plan` (完整转换) | 完整使用 |
| Executor | `query_context` (传递) | `plan` (完整使用) | 完整使用 |

### 上下文对象流转

```
┌──────────────────────────────────────────────────────────────────────┐
│                        QueryPipelineManager                          │
└──────────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
┌───────────────┐   ┌─────────────────┐   ┌─────────────────┐
│ QueryContext  │   │ QueryAstContext │   │ ExecutionPlan   │
│ (创建)        │   │ (parse)         │   │ (plan)          │
└───────────────┘   └─────────────────┘   └─────────────────┘
        │                     │                     │
        ├─────────────────────┤                     │
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌─────────────────┐   ┌─────────────────┐
│ QueryContext  │   │ ValidationContext│  │ ExecutionPlan   │
│ (未修改)      │   │ (内部创建)      │   │ (优化)          │
└───────────────┘   └─────────────────┘   └─────────────────┘
        │                     │                     │
        │                     ▼                     │
        │           ┌─────────────────┐             │
        │           │ AstContext      │             │
        │           │ (提取)          │             │
        │           └─────────────────┘             │
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌─────────────────┐   ┌─────────────────┐
│ QueryContext  │   │ SubPlan         │   │ ExecutionResult │
│ (传递)        │   │ (生成)          │   │ (最终结果)      │
└───────────────┘   └─────────────────┘   └─────────────────┘
```

## AST 转换点总结

### 完整 AST 转换阶段

1. **Parser 阶段**
   - 输入：`String`（查询文本）
   - 输出：`QueryAstContext`（包含 `Stmt`）
   - 转换性质：文本 → 树形结构

2. **Planner 阶段**
   - 输入：`Stmt`
   - 输出：`ExecutionPlan`（计划节点树）
   - 转换性质：语句 → 执行操作序列

3. **Optimizer 阶段**
   - 输入：`ExecutionPlan`
   - 输出：`ExecutionPlan`（优化后的）
   - 转换性质：原始计划 → 优化后的计划

4. **Executor 阶段**
   - 输入：`ExecutionPlan`
   - 输出：`ExecutionResult`
   - 转换性质：计划 → 可迭代结果集

### 部分转换阶段

**Validator 阶段**
- 输入：`QueryAstContext`
- 输出：验证结果（通过/失败）+ 内部 `ValidationContext`
- 问题：未增强原始 `QueryAstContext`，验证状态独立存储

### 数据传递阶段

| 阶段 | 传递的数据 | 是否转换 | 说明 |
|------|-----------|----------|------|
| Parser → Validator | `QueryAstContext` | 部分 | 仅传递指针，验证器使用内部状态 |
| Validator → Planner | 隐式状态 | 无 | 验证状态未显式传递 |
| Planner → Optimizer | `ExecutionPlan` | 是 | 计划作为整体传递 |
| Optimizer → Executor | `ExecutionPlan` | 是 | 优化后的计划 |

## 关键发现

### 1. 上下文对象冗余

系统存在多个上下文对象，但它们的职责和关系不够清晰：

- `QueryContext`：查询执行上下文
- `QueryAstContext`：解析后的 AST 上下文
- `AstContext`：`QueryAstContext` 的子对象
- `ValidationContext`：验证上下文
- `ExecutionContext`：执行上下文
- `RuntimeContext`：运行时上下文
- `OptContext`：优化上下文

这些上下文之间存在数据冗余和同步问题。

### 2. 数据传递不完整

在当前实现中：

```rust
// Validator 阶段
fn validate_query(
    &mut self,
    _query_context: &mut QueryContext,  // 未使用
    ast: &QueryAstContext,               // 仅检查存在性
) -> DBResult<()> {
    self.validator.validate_unified()...  // 使用内部 ValidationContext
}

// Planner 阶段
fn generate_execution_plan(
    &mut self,
    _query_context: &mut QueryContext,   // 未使用
    ast: &QueryAstContext,
) -> DBResult<ExecutionPlan> {
    let ast_ctx = ast.base_context();    // 提取子对象
    self.planner.transform(ast_ctx)...  // 使用 AstContext 而非完整 QueryAstContext
}
```

验证阶段和规划阶段创建的上下文信息未能有效传递给后续阶段。

### 3. 类型不一致

Planner 使用 `AstContext` 而非完整的 `QueryAstContext`：

```rust
// 使用 .base_context() 提取
let ast_ctx = ast.base_context();
match self.planner.transform(ast_ctx) { ... }
```

这导致 `QueryAstContext` 中存储的变量信息、表达式上下文等未能被规划器利用。

### 4. 优化阶段的上下文使用

Optimizer 是唯一完整使用 QueryContext 的阶段：

```rust
let mut opt_ctx = OptContext::new(qctx.clone());
```

但这是通过克隆实现的，而非引用传递，说明上下文设计可能存在可变性需求。

## 改进建议

### 1. 统一上下文传递

建议设计统一的查询上下文接口，让验证结果能够被后续阶段访问：

```rust
// 增强后的设计
struct UnifiedQueryContext {
    ast: QueryAstContext,
    validation: ValidationResult,
    plan: Option<ExecutionPlan>,
    // ... 其他阶段信息
}
```

### 2. 消除上下文冗余

合并职责相近的上下文对象：
- 移除 `AstContext`，直接使用 `QueryAstContext`
- 将 `ValidationContext` 的关键信息合并到 `QueryAstContext`
- 统一使用 `QueryContext` 作为主上下文

### 3. 明确数据所有权

明确每个阶段对上下文的所有权和修改权限：
- Parser：对 `QueryAstContext` 有所有权
- Validator：对 `ValidationContext` 有所有权，并应更新 `QueryAstContext`
- Planner：对 `ExecutionPlan` 有所有权
- Optimizer：对优化后的 `ExecutionPlan` 有所有权
- Executor：对 `ExecutionResult` 有所有权

### 4. 添加数据流追踪

在关键转换点添加日志，便于调试和性能分析：

```rust
fn generate_execution_plan(&mut self, ...) -> DBResult<ExecutionPlan> {
    let start = Instant::now();
    let result = /* 规划逻辑 */;
    let duration = start.elapsed();
    tracing::info!(?duration, "计划生成耗时");
    result
}
```

## 结论

GraphDB 查询引擎的五阶段管道架构设计清晰，但数据传递和上下文管理存在改进空间。当前实现中：

- **Parser、Planner、Optimizer、Executor** 四个实质阶段进行了性的 AST/计划转换
- **Validator** 阶段仅进行了部分转换，验证状态独立存储
- **query_context** 在多个阶段未被有效利用
- 多个上下文对象存在职责重叠和数据冗余

建议通过统一上下文传递、消除冗余、明确数据所有权等措施，优化数据流效率，提高代码可维护性。
