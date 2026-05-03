# 查询引擎架构分析与改进方案

## 一、整体架构概览

当前查询引擎采用经典的 **Parse → Validate → Plan → Optimize → Execute** 五阶段流水线架构，由 `QueryPipelineManager` 统一协调。

### 核心设计选择

- **静态分发**（enum-based dispatch）而非动态分发（`dyn` trait），符合 Rust 零成本抽象理念
- **三层枚举映射**：`PlannerEnum` → `PlanNodeEnum` → `ExecutorEnum`
- **宏减少样板代码**：`define_enum_is_methods!`、`define_enum_as_methods!`、`delegate_to_executor!`
- **Builder 模式分层**：`ExecutorFactory` → 各类 Builder（DataAccessBuilder、AdminBuilder 等）
- **Optimizer 独立为 Engine**：`OptimizerEngine` 作为共享实例，与数据库实例同生命周期

### 当前规模

| 组件 | 变体数量 | 代码行数 |
|------|----------|----------|
| PlannerEnum | 28 | ~570 行 |
| PlanNodeEnum | 90+ | ~1500 行 |
| ExecutorEnum | 90+ | ~1000 行 |
| ValidatorEnum | 40+ | ~939 行 |
| QueryPipelineManager | - | ~952 行 |

---

## 二、识别出的架构问题

### 问题 1：枚举爆炸（严重）

#### 现状

- `PlanNodeEnum` 有 90+ 个变体，其中约 40% 是管理节点（Create/Drop/Alter/Show/Desc）
- `ExecutorEnum` 有 90+ 个变体，与 `PlanNodeEnum` 一一对应
- 每次新增节点类型需要修改 6+ 处代码

#### 问题表现

1. **扩展成本高**：新增一个 PlanNode 需要修改 `PlanNodeEnum` 定义、`is_xxx` 宏、`as_xxx` 宏、`ExecutorFactory::create_executor` 的 match、`ExecutorEnum` 定义、`delegate_to_executor` 宏、`delegate_to_executor_mut` 宏、`Debug` 实现等
2. **代码膨胀**：`executor_enum.rs` 已达 1000 行，`plan_node_enum.rs` 超过 1500 行，`planner.rs` 超过 570 行
3. **编译时间长**：巨大的 enum 和宏展开显著增加编译时间
4. **管理节点占比过高**：管理节点结构简单、逻辑同质，但占据了大量枚举空间

#### 改进方案

**方案 A：管理节点合并为参数化枚举（推荐，短期）**

将管理类节点从 40+ 个独立变体合并为按类别分组的参数化变体：

```rust
pub enum PlanNodeEnum {
    // 现有查询节点保持不变（Project, Filter, Sort, Join, Traverse 等）
    // ...

    // 管理节点合并为参数化变体
    SpaceManagement(SpaceManageNode),   // 替代 CreateSpace/DropSpace/AlterSpace/DescSpace/ShowSpaces/ShowCreateSpace/SwitchSpace/ClearSpace
    TagManagement(TagManageNode),       // 替代 CreateTag/AlterTag/DropTag/DescTag/ShowTags/ShowCreateTag
    EdgeManagement(EdgeManageNode),     // 替代 CreateEdge/AlterEdge/DropEdge/DescEdge/ShowEdges/ShowCreateEdge
    IndexManagement(IndexManageNode),   // 替代 CreateTagIndex/DropTagIndex/.../ShowIndexes/ShowCreateIndex
    UserManagement(UserManageNode),     // 替代 CreateUser/DropUser/AlterUser/ChangePassword/GrantRole/RevokeRole/ShowUsers/ShowRoles
    FulltextManagement(FulltextManageNode), // 替代 CreateFulltextIndex/DropFulltextIndex/...
    VectorManagement(VectorManageNode),     // 替代 CreateVectorIndex/DropVectorIndex/...
}

// 每个管理类别定义子枚举
pub enum SpaceManageNode {
    Create(CreateSpaceNode),
    Drop(DropSpaceNode),
    Alter(AlterSpaceNode),
    Desc(DescSpaceNode),
    Show(ShowSpacesNode),
    ShowCreate(ShowCreateSpaceNode),
    Switch(SwitchSpaceNode),
    Clear(ClearSpaceNode),
}
```

**收益**：
- `PlanNodeEnum` 变体从 90+ 降至约 50
- 新增管理操作只需修改对应的子枚举和 Builder，无需修改主枚举
- `ExecutorFactory` 的 match 分支大幅减少

**方案 B：引入宏自动生成 dispatch 代码（中期）**

创建 `define_dispatch!` 宏，自动生成 enum 的 `transform`/`matches`/`name` 等方法：

```rust
define_planner_dispatch! {
    (Match, MatchStatementPlanner, "MatchPlanner"),
    (Go, GoPlanner, "GoPlanner"),
    (Lookup, LookupPlanner, "LookupPlanner"),
    // ...
}
```

**方案 C：对管理节点使用注册表模式（长期，不推荐）**

```rust
pub struct AdminExecutorRegistry<S: StorageClient> {
    creators: HashMap<AdminNodeType, Box<dyn Fn(...) -> Result<ExecutorEnum<S>, QueryError>>>,
}
```

引入了 `dyn`，与项目规范（避免动态分发）冲突。

---

### 问题 2：三层枚举映射缺乏类型安全（中等）

#### 现状

- `PlannerEnum` → `PlanNodeEnum` → `ExecutorEnum` 三层之间的映射完全靠手写 match
- 没有编译期保证三层的变体是对齐的
- 如果在 `PlanNodeEnum` 新增了变体但忘记在 `ExecutorFactory` 中处理，编译器不会报错（因为 match 有 `_ =>` 兜底或新变体在已有分支中）

#### 改进方案

**利用 `#[non_exhaustive]` + 强制穷尽匹配**

在 `PlanNodeEnum` 上标记 `#[non_exhaustive]`，在 `ExecutorFactory::create_executor` 中不使用 `_ =>` 兜底，强制每个新变体都必须显式处理：

```rust
#[non_exhaustive]
pub enum PlanNodeEnum { ... }

// ExecutorFactory 中
match plan_node {
    PlanNodeEnum::Start(node) => { ... },
    PlanNodeEnum::GetVertices(node) => { ... },
    // 必须列出所有变体，新增变体会导致编译错误
}
```

**收益**：新增 PlanNode 变体时，编译器会强制要求在所有 match 中处理，避免遗漏。

---

### 问题 3：Validator 和 Planner 之间的数据传递方式不一致（中等）

#### 现状

- `ValidatedStatement` 包含 `ValidationInfo`，但不同 planner 从中提取数据的方式不同
- 有些 planner 直接从 AST 重新解析（如 `MatchStatementPlanner`），有些使用 `ValidationInfo`
- `ValidationInfo` 中的信息与 planner 实际需要的信息不匹配

#### 改进方案

**定义 PlannerInput 类型**

```rust
pub struct PlannerInput {
    pub stmt: Arc<Stmt>,
    pub validation_info: ValidationInfo,
    pub metadata_context: Option<MetadataContext>,
    pub expression_context: Arc<ExpressionAnalysisContext>,
}
```

让 `Planner::transform` 接收 `PlannerInput` 而非分别接收 `ValidatedStatement` 和 `Arc<QueryContext>`，明确数据来源，避免 planner 重复解析 AST。

---

### 问题 4：QueryPipelineManager 职责过重（中等）

#### 现状

- `QueryPipelineManager` 有 952 行，承担了：解析调度、验证调度、计划生成、优化调度、执行调度、缓存管理、EXPLAIN/PROFILE 处理、指标收集
- `execute_query_with_profile` 方法中混合了业务逻辑和性能指标收集
- 多个 `execute_query_*` 变体方法有大量重复代码

#### 改进方案

**拆分为 Pipeline Stages + Orchestrator**

```rust
pub struct QueryOrchestrator<S: StorageClient> {
    parse_stage: ParseStage,
    validate_stage: ValidateStage,
    plan_stage: PlanStage,
    optimize_stage: OptimizeStage,
    execute_stage: ExecuteStage,
    cache: Arc<QueryPlanCache>,
}

// 每个阶段有明确的输入/输出
trait PipelineStage {
    type Input;
    type Output;
    type Error;
    fn process(&mut self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}
```

**收益**：
- 每个阶段职责单一，易于测试和维护
- 指标收集通过中间件/装饰器模式实现，不侵入业务逻辑
- 消除 `execute_query_*` 变体方法之间的代码重复

---

### 问题 5：错误类型转换链过长，信息丢失（中等）

#### 现状

- `PlannerError` → `DBError::Query(QueryError::ExecutionError(msg))` → `QueryError` → `DBError`
- 转换过程中原始错误类型信息丢失，只剩 `to_string()` 的字符串
- `ValidationError` 和 `PlannerError` 之间没有直接转换
- `DBError` 有 20+ 个变体，其中多个是 `String` 类型（如 `Validation(String)`、`Io(String)`），缺乏结构化

#### 改进方案

**1. 让 `PlannerError` 直接嵌入 `QueryError`**

```rust
pub enum QueryError {
    // 现有变体...
    PlanningError(PlannerError),  // 替代 PlanningError(String)
}
```

**2. 减少字符串化错误变体**

```rust
pub enum DBError {
    Validation(ValidationError),  // 替代 Validation(String)
    Io(std::io::Error),          // 替代 Io(String)
    // ...
}
```

**收益**：错误链保留完整类型信息，便于上层精确匹配和处理。

---

### 问题 6：ExecutorFactory 的 Builder 模式不够统一（轻微）

#### 现状

- `ExecutorFactory::create_executor` 使用 match 分发到各类 Builder
- 但 `build_loop_executor` 和 `build_select_executor` 是 `ExecutorFactory` 自身的方法（因为需要创建临时 `ExecutorFactory`）
- `SelectExecutor` 的构建逻辑特殊：需要创建临时 factory 来避免借用冲突

#### 改进方案

将 `build_loop_executor` 和 `build_select_executor` 移入 `ControlFlowBuilder`，通过传入 `&mut ExecutorFactory` 引用来解决借用问题：

```rust
impl ControlFlowBuilder {
    pub fn build_select<S: StorageClient>(
        factory: &mut ExecutorFactory<S>,
        node: &SelectNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // ...
    }
}
```

---

### 问题 7：PlanNode 的 children 访问模式不够统一（轻微）

#### 现状

- `PlanNode` trait 定义了 `children()` 方法返回 `Vec<&PlanNodeEnum>`
- 不同节点的输入数量不同（0/1/2），每次调用都要分配 Vec
- `PlanExecutor::build_executor_chain` 通过 `children.len()` 判断节点类型（0=叶子，1=单输入，2=双输入），这是隐式约定

#### 改进方案

**引入 NodeArity 枚举**

```rust
pub enum NodeArity {
    Leaf,
    Unary,
    Binary,
}

// 在 PlanNode trait 中
fn arity(&self) -> NodeArity;
fn left_child(&self) -> Option<&PlanNodeEnum>;
fn right_child(&self) -> Option<&PlanNodeEnum>;
```

**收益**：避免 Vec 分配，语义更清晰，消除隐式约定。

---

### 问题 8：Optimizer 与 Planner 的边界模糊（轻微）

#### 现状

- `MatchStatementPlanner` 内部做了部分优化（如 seek strategy 选择）
- Optimizer 的启发式规则也做类似的事情（如 predicate pushdown）
- 存在重复优化的风险

#### 改进方案

明确约定：**Planner 只负责生成逻辑正确的初始计划，不做任何优化。所有优化决策交给 Optimizer。**

Seek strategy 选择应作为 Optimizer 的一个规则来实现，而非在 Planner 中硬编码。

---

## 三、改进优先级排序

| 优先级 | 问题 | 改进方案 | 影响范围 | 风险 |
|--------|------|----------|----------|------|
| P0 | 枚举爆炸 | 方案A：管理节点参数化 | PlanNodeEnum, ExecutorEnum, ExecutorFactory | 中（需修改大量 match） |
| P1 | 三层映射缺乏类型安全 | non_exhaustive + 强制穷尽匹配 | ExecutorFactory, executor_enum.rs | 低 |
| P1 | 错误转换链信息丢失 | 结构化错误嵌入 | core/error, planner.rs | 低 |
| P2 | PipelineManager 职责过重 | 拆分为 Pipeline Stages | query_pipeline_manager.rs | 中（接口变更） |
| P2 | Validator/Planner 数据传递不一致 | 定义 PlannerInput | planner.rs, 所有 planner | 中（接口变更） |
| P3 | ExecutorFactory Builder 不统一 | 移入 ControlFlowBuilder | ExecutorFactory | 低 |
| P3 | PlanNode children 访问不统一 | 引入 NodeArity | PlanNode trait, PlanExecutor | 低 |
| P3 | Optimizer/Planner 边界模糊 | 明确职责约定 | MatchStatementPlanner, Optimizer | 低 |

---

## 四、架构优点（应保持）

1. **静态分发选择正确**：enum-based dispatch 避免了 vtable 开销，符合 Rust 零成本抽象理念
2. **宏减少样板代码**：`define_enum_is_methods!`、`define_enum_as_methods!`、`delegate_to_executor!` 等宏有效减少了手写代码
3. **Builder 模式分层合理**：`ExecutorFactory` → 各类 Builder 的分层让代码组织清晰
4. **Optimizer 独立为 Engine**：`OptimizerEngine` 作为共享实例的设计是正确的，避免重复创建
5. **完整的五阶段流水线**：Parse → Validate → Plan → Optimize → Execute 的分离是数据库领域的标准做法
6. **Validator 枚举化**：`Validator` enum 同样采用静态分发，与 Planner/Executor 保持一致

---

## 五、P0 方案详细实施计划

### 阶段 1：管理节点参数化（PlanNodeEnum 侧）

1. 定义管理节点子枚举：
   - `SpaceManageNode`（8 个变体）
   - `TagManageNode`（6 个变体）
   - `EdgeManageNode`（6 个变体）
   - `IndexManageNode`（12 个变体）
   - `UserManageNode`（8 个变体）
   - `FulltextManageNode`（5 个变体）
   - `VectorManageNode`（5 个变体）

2. 修改 `PlanNodeEnum`，将 40+ 个管理变体替换为 7 个参数化变体

3. 更新 `is_xxx` / `as_xxx` 宏和分类方法（`is_management` 等）

4. 更新所有使用 `PlanNodeEnum::CreateSpace(...)` 等模式的代码

### 阶段 2：管理执行器参数化（ExecutorEnum 侧）

1. 定义对应的执行器子枚举：
   - `SpaceManageExecutor<S>`
   - `TagManageExecutor<S>`
   - 等等

2. 修改 `ExecutorEnum`，将 40+ 个管理变体替换为 7 个参数化变体

3. 更新 `delegate_to_executor!` / `delegate_to_executor_mut!` 宏

4. 更新 `Debug` 实现

### 阶段 3：ExecutorFactory 适配

1. 修改 `ExecutorFactory::create_executor`，管理节点分支委托给 `AdminBuilder`

2. 在 `AdminBuilder` 中添加子枚举的分发逻辑

### 阶段 4：验证与测试

1. 运行 `cargo clippy --all-targets --all-features` 确保编译通过

2. 运行 `cargo test --lib` 确保单元测试通过

3. 运行集成测试确保端到端功能正确

---

## 六、总结

当前架构的核心问题是**枚举变体数量失控**，导致每次扩展都需要修改大量代码。这本质上是管理类操作（DDL/DCL）和查询类操作（DQL/DML）混在同一个枚举中的结果。

管理类操作虽然数量多，但逻辑简单且同质，适合参数化合并；而查询类操作逻辑复杂且差异大，适合保持独立变体。

最优先的改进是**管理节点参数化**，它能在不改变整体架构的情况下，将枚举变体数量减少约 40%，同时让新增管理操作的成本从"修改 6+ 处代码"降为"修改 1-2 处代码"。
