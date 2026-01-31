# 优化器架构分析与重构方案

## 一、概述

本文档对 `src/query/optimizer/plan` 和 `src/query/optimizer/core` 两个核心模块进行了深入分析，对比了原生 Nebula-graph 的实现方案，指出了当前设计中存在的主要问题，并提出了分阶段的改进方案。

优化器是图数据库查询引擎的核心组件，负责将解析后的查询计划转换为高效的执行计划。一个设计良好的优化器需要具备以下能力：等价计划的探索与选择、基于代价的计划评估、完整的控制流与数据流验证、以及高效的对象复用机制。

## 二、当前架构分析

### 2.1 目录结构说明

当前优化器代码组织如下：

```
src/query/optimizer/
├── core/                    # 核心类型模块
│   ├── mod.rs              # 模块导出
│   ├── cost.rs             # 代价模型定义
│   ├── phase.rs            # 优化阶段定义
│   └── config.rs           # 优化配置
├── plan/                   # 计划表示模块
│   ├── mod.rs              # 模块导出
│   ├── context.rs          # 优化上下文
│   ├── group.rs            # 优化组
│   └── node.rs             # 优化节点与规则trait
├── engine/                 # 优化引擎
│   ├── mod.rs              # 模块导出
│   ├── optimizer.rs        # 优化器主逻辑
│   └── exploration.rs      # 探索算法
├── *.rs                    # 各类优化规则（约30个）
└── mod.rs                  # 模块入口
```

### 2.2 核心数据结构使用情况

**plan 目录** 被 16 个规则文件广泛引用，包括消除规则、转换规则、下推规则、合并规则等。这表明 plan 模块作为优化器的核心抽象层已被充分使用。

**core 目录** 仅被 5 个文件引用，包括 plan 模块自身、engine/optimizer.rs 和 rule_enum.rs。这表明 core 模块的抽象程度较高，主要提供基础类型支持。

### 2.3 当前实现的局限性

当前实现虽然基本功能完整，但在架构设计上存在以下局限性：

第一，依赖关系表示不够精确。Nebula-graph 使用对象指针（`OptGroup*`、`OptGroupNode*`）来表示依赖关系，可以直接在运行时进行类型检查和空值验证。而当前实现使用 `usize` 类型的 ID 来引用依赖节点，虽然简化了 Rust 的生命周期管理，但丧失了类型安全性，增加了运行时出错的风险。

第二，对象池机制不完善。虽然项目已提供了 `ObjectPool` 实现（位于 `src/utils/object_pool.rs`），但 OptContext 中的对象池仅用于存储预分配的 OptGroupNode 对象，实际使用时仍然通过 `Vec::push` 和 `HashMap` 进行管理，并未实现真正的对象复用。每次规则应用都可能导致对象的重新分配和 Clone 操作。

第三，数据流验证不完整。Nebula-graph 的优化器在规则匹配时会调用 `checkDataflowDeps` 方法验证数据流依赖是否与控制流依赖一致，确保优化不会破坏查询语义。当前实现虽然提供了 `validate_data_flow` 方法，但验证逻辑过于简单，无法检测复杂的数据流异常。

第四，转换结果语义不完整。Nebula-graph 的 `TransformResult` 支持 `eraseAll`（擦除所有节点）、`eraseCurr`（擦除当前节点）和 `newGroupNodes`（添加新节点）三种操作，当前实现的 `TransformResult` 仅支持返回单个新节点，缺少批量操作能力。

## 三、与 Nebula-graph 的详细对比

### 3.1 OptContext 对比

OptContext 是优化器的全局上下文，管理优化过程中的所有状态信息。

**Nebula-graph 实现**（[OptContext.cpp](file:///d:/项目/database/graphDB/nebula-3.8.0/src/graph/optimizer/OptContext.cpp)）：

```cpp
class OptContext {
    graph::QueryContext *qctx_;  // 查询上下文指针
    std::unique_ptr<ObjectPool> objPool_;  // 对象池
    std::unordered_map<int64_t, const OptGroupNode *> planNodeToOptGroupNodeMap_;  // 映射表
    bool changed_{false};  // 计划是否改变
};
```

**当前实现**（[context.rs](file:///d:/项目/database/graphDB/src/query/optimizer/plan/context.rs)）：

```rust
pub struct OptContext {
    pub query_context: QueryContext,
    pub stats: OptimizationStats,
    pub changed: bool,
    pub visited_groups: HashSet<usize>,
    pub plan_node_to_group_node: HashMap<usize, OptGroupNode>,  // 存储对象而非指针
    pub group_map: HashMap<usize, OptGroup>,  // 存储对象而非指针
    pub statistics: Statistics,
    object_pool: ObjectPool<OptGroupNode>,  // 对象池
}
```

**关键差异分析**：

Nebula-graph 使用对象池的 `makeAndAdd` 方法创建对象，确保对象生命周期由对象池统一管理。当调用 `OptGroupNode::create(ctx, node, group)` 时，对象池会自动将新对象添加到池中，并建立 planNodeId 到 OptGroupNode 的映射。这种设计确保了对象的唯一性和可追溯性。

当前实现虽然在 OptContext 中包含了 ObjectPool 字段，但在实际使用中并未充分利用对象池的能力。`plan_node_to_group_node` 存储的是完整的 `OptGroupNode` 对象而非从对象池获取的引用，这导致对象池形同虚设。

### 3.2 OptGroup 对比

OptGroup 表示一组等价的执行计划节点，是优化器进行计划空间搜索的基本单位。

**Nebula-graph 实现**（[OptGroup.cpp](file:///d:/项目/database/graphDB/nebula-3.8.0/src/graph/optimizer/OptGroup.cpp)）：

```cpp
class OptGroup {
    OptContext *ctx_;
    std::vector<OptGroupNode *> groupNodes_;  // 节点指针列表
    std::unordered_set<const OptGroupNode *> groupNodesReferenced_;  // 被引用计数
    std::string outputVar_;  // 输出变量名
    std::vector<const OptRule *> exploredRules_;  // 已探索规则
    
    Status explore(const OptRule *rule);
    Status exploreUntilMaxRound(const OptRule *rule);
    std::pair<double, const OptGroupNode *> findMinCostGroupNode() const;
};
```

**当前实现**（[group.rs](file:///d:/项目/database/graphDB/src/query/optimizer/plan/group.rs)）：

```rust
pub struct OptGroup {
    pub id: usize,
    pub nodes: Vec<OptGroupNode>,  // 存储对象而非指针
    pub logical: bool,
    pub explored_rules: Vec<String>,  // 使用字符串而非指针
    pub root_group: bool,
    pub output_var: Option<String>,
    pub bodies: Vec<OptGroup>,  // 内联存储body而非引用
    pub group_nodes_referenced: HashSet<usize>,
    pub candidates: Vec<PlanCandidate>,
    pub phase: OptimizationPhase,
}
```

**关键差异分析**：

Nebula-graph 的 OptGroup 使用指针引用 OptGroupNode，这允许在优化过程中动态添加和删除节点，而不会导致所有权的复杂性。`exploredRules_` 存储规则指针，可以直接调用规则的方法进行状态检查。

当前实现使用 `Vec<OptGroupNode>` 存储节点对象，这意味着每次添加新节点都需要 Clone 操作。`bodies: Vec<OptGroup>` 将 Loop 和 Select 的 body 直接内联存储，这虽然简化了生命周期管理，但导致内存布局不够灵活，无法支持复杂的嵌套结构。

### 3.3 OptRule 机制对比

OptRule 是优化规则的核心抽象，定义了规则的匹配和转换逻辑。

**Nebula-graph 实现**（[OptRule.cpp](file:///d:/项目/database/graphDB/nebula-3.8.0/src/graph/optimizer/OptRule.cpp)）：

```cpp
class OptRule {
    virtual StatusOr<MatchedResult> match(OptContext *ctx, const OptGroupNode *groupNode) const;
    virtual bool match(OptContext *ctx, const MatchedResult &matched) const;
    virtual StatusOr<TransformResult> transform(OptContext *ctx, const MatchedResult &matched) const = 0;
    bool checkDataflowDeps(...) const;
    const Pattern &pattern() const { return pattern_; }
};
```

**当前实现**（[node.rs](file:///d:/项目/database/graphDB/src/query/optimizer/plan/node.rs)）：

```rust
pub trait OptRule: std::fmt::Debug {
    fn name(&self) -> &str;
    fn apply(&self, ctx: &mut OptContext, group_node: &OptGroupNode) 
        -> Result<Option<OptGroupNode>, OptimizerError>;
    fn pattern(&self) -> Pattern;
    fn match_pattern(&self, ctx: &mut OptContext, group_node: &OptGroupNode) 
        -> Result<Option<MatchedResult>, OptimizerError>;
}
```

**关键差异分析**：

Nebula-graph 的 `transform` 方法返回 `StatusOr<TransformResult>`，支持返回多个新节点。`TransformResult` 包含 `newGroupNodes`（新节点列表）、`eraseAll`（是否擦除所有旧节点）和 `eraseCurr`（是否擦除当前节点），支持复杂的转换模式。

当前实现的 `apply` 方法仅返回 `Option<OptGroupNode>`，只能返回单个新节点，限制了规则的能力。例如，合并两个节点的规则需要返回两个节点合并后的新节点，同时可能需要删除原来的两个节点，这在当前实现中难以优雅地表达。

### 3.4 优化器引擎对比

优化器引擎负责协调整个优化过程，包括计划转换、规则应用和最终计划选择。

**Nebula-graph 实现**（[Optimizer.cpp](file:///d:/项目/database/graphDB/nebula-3.8.0/src/graph/optimizer/Optimizer.cpp)）：

```cpp
StatusOr<const PlanNode *> Optimizer::findBestPlan(QueryContext *qctx) {
    auto optCtx = std::make_unique<OptContext>(qctx);
    auto ret = prepare(optCtx.get(), root);  // 转换为 group
    NG_RETURN_IF_ERROR(ret);
    auto rootGroup = std::move(ret).value();
    
    NG_RETURN_IF_ERROR(doExploration(optCtx.get(), rootGroup));  // 执行优化
    auto *newRoot = rootGroup->getPlan();  // 获取最佳 plan
    
    NG_RETURN_IF_ERROR(postprocess(newRoot, qctx, spaceID));  // 后处理
    return newRoot;
}
```

**当前实现**（[optimizer.rs](file:///d:/项目/database/graphDB/src/query/optimizer/engine/optimizer.rs)）：

```rust
pub fn find_best_plan(&mut self, qctx: &mut QueryContext, plan: ExecutionPlan) 
    -> Result<ExecutionPlan, OptimizerError> {
    let mut opt_ctx = OptContext::new(qctx.clone());
    let mut root_group = self.plan_to_group(&plan)?;
    root_group.root_group = true;
    
    self.execute_phase_optimization(&mut opt_ctx, &mut root_group, 
        OptimizationPhase::LogicalOptimization)?;
    self.execute_phase_optimization(&mut opt_ctx, &mut root_group, 
        OptimizationPhase::PhysicalOptimization)?;
    self.execute_phase_optimization(&mut opt_ctx, &mut root_group, 
        OptimizationPhase::PostOptimization)?;
    
    self.post_process(&mut opt_ctx, &mut root_group)?;
    self.group_to_plan(&root_group)
}
```

**关键差异分析**：

Nebula-graph 的 `prepare` 方法通过递归转换构建 OptGroup 结构，同时处理 Loop 和 Select 的 body。`doExploration` 方法使用 `visited_` 集合追踪已访问的组，避免重复探索。`postprocess` 方法调用 `PrunePropertiesVisitor` 进行属性剪枝，确保最终计划只输出必要的属性。

当前实现的 `plan_to_group` 方法过于简单，只是将 PlanNode 递归转换为 OptGroupNode 存入同一个 OptGroup，忽略了 Loop 和 Select 的特殊处理。`post_process` 中的 `prune_properties` 和 `rewrite_arguments` 方法都是空实现。

## 四、主要问题总结

### 4.1 架构完整性问题

当前实现存在以下架构层面的缺陷：

**依赖关系不精确**：使用 `usize` ID 代替对象引用，虽然简化了 Rust 的借用检查，但丧失了类型安全性。运行时可能出现无效 ID 导致的查找失败，且难以进行依赖关系的静态检查。

**缺少 Memo 结构**：Nebula-graph 的优化器使用 Memo 结构存储等价计划，当前实现仅使用简单的 Vec 存储，无法支持真正的多计划探索和代价比较。

**数据流验证缺失**：无法验证数据流依赖与控制流依赖的一致性，可能导致优化后的计划语义不正确。

### 4.2 优化能力缺失

**多计划探索受限**：当前实现只能生成单一的执行计划，无法并行探索多个等价计划并选择代价最低的方案。

**规则能力受限**：`apply` 方法仅返回单个节点，限制了可以表达的优化模式。例如，节点合并规则需要同时删除多个旧节点并添加新节点，当前实现难以优雅地表达。

**迭代控制不完善**：缺少 `visited_` 追踪机制，可能导致规则在同一个组上重复应用。

### 4.3 性能问题

**频繁的对象分配**：每次规则应用都创建新的 OptGroupNode 对象并通过 Clone 复制状态，没有利用对象池进行复用。

**低效的查找操作**：`HashMap<usize, OptGroup>` 的查找需要计算哈希值，而指针比较通常更快。

**不必要的内存拷贝**：大量使用 `clone()` 方法复制对象，增加了内存开销。

### 4.4 代码质量问题

**类型安全缺失**：使用字符串存储规则名称而非规则指针，运行时需要进行字符串比较。

**空值处理不完善**：多处使用 `unwrap()` 和 `expect()`，在异常情况下可能导致 panic。

**验证逻辑薄弱**：`validate_data_flow` 方法的实现过于简单，无法检测复杂的计划异常。

## 五、分阶段修改方案

### 5.1 第一阶段：核心数据结构完善

**目标**：完善 plan 目录中的核心数据结构，为后续优化奠定基础。

**修改内容**：

修改 `context.rs`，完善 OptContext 的对象池使用方式，实现真正的对象复用。添加 `OptGroup` 和 `OptGroupNode` 的引用类型，替换现有的 ID 引用方式。增强数据流验证逻辑。

修改 `group.rs`，实现更完善的组管理机制。添加 `visited` 追踪防止重复探索。修改 `bodies` 字段为引用类型。

修改 `node.rs`，扩展 `TransformResult` 以支持多节点转换。添加 `eraseAll` 和 `eraseCurr` 语义。完善 `OptRule` trait 的定义。

**预期效果**：核心数据结构更加健全，类型安全性提高。

### 5.2 第二阶段：优化器引擎增强

**目标**：增强 engine/optimizer.rs 的功能，实现更完整的优化流程。

**修改内容**：

完善 `plan_to_group` 方法，正确处理 Loop 和 Select 节点的 body 转换。实现真正的对象池复用机制。添加 `visited` 追踪机制。

完善 `post_process` 方法，实现属性剪枝和参数重写。添加计划深度检查。

实现代价估算模块，支持更精确的代价计算。

**预期效果**：优化器功能更完整，生成的执行计划质量更高。

### 5.3 第三阶段：规则系统增强

**目标**：增强规则表达能力，支持更复杂的优化模式。

**修改内容**：

修改现有规则以利用新的 `TransformResult` 语义。添加新的转换规则，支持节点合并、分解等操作。

添加数据流验证规则，确保优化后的计划语义正确。

**预期效果**：规则系统更强大，能够表达更多的优化模式。

### 5.4 第四阶段：性能优化

**目标**：提高优化器的运行性能，减少内存开销。

**修改内容**：

实现真正的对象池机制，复用 OptGroupNode 对象。

优化数据结构，减少 Clone 操作。

添加并行探索支持（如果适用）。

**预期效果**：优化器运行速度更快，内存占用更少。

## 六、文件依赖关系

### 6.1 plan 模块内部依赖

```
context.rs
  ├── group.rs（OptGroup 类型）
  ├── node.rs（OptGroupNode、PlanNodeProperties 类型）
  └── utils/object_pool.rs（ObjectPool 类型）

group.rs
  ├── node.rs（OptGroupNode 类型）
  └── core/phase.rs（OptimizationPhase 类型）

node.rs
  ├── context.rs（OptContext 类型）
  ├── group.rs（OptGroup 类型）
  └── core/cost.rs（Cost 类型）
```

### 6.2 外部依赖

plan 模块被以下模块引用：

```
engine/optimizer.rs
  ├── plan/context.rs（OptContext 类型）
  ├── plan/group.rs（OptGroup 类型）
  └── plan/node.rs（OptGroupNode、OptRule 类型）

*.rs（各优化规则）
  └── plan/context.rs、plan/node.rs
```

## 七、修改注意事项

### 7.1 兼容性考虑

修改核心数据结构时需要确保与现有规则的兼容性。建议采用渐进式修改策略，先修改底层结构，再逐步调整上层代码。

### 7.2 测试策略

每次修改后需要运行现有测试确保功能正常。建议添加针对数据流验证、对象池复用等新功能的单元测试。

### 7.3 文档更新

修改代码的同时需要更新相关文档，确保文档与代码一致。

## 八、结语

当前优化器实现虽然基本功能完整，但在架构设计上存在一些局限性。通过本文档提出的分阶段修改方案，可以逐步完善优化器的架构，提高其功能和性能。修改过程需要谨慎进行，确保每次修改后系统仍能正常工作。

建议优先实施第一阶段的修改，完善核心数据结构，为后续优化奠定基础。在此过程中，需要密切关注代码质量和性能变化，确保修改朝着预期方向发展。
