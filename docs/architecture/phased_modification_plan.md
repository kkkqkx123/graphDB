# 分阶段修改计划

## 一、概述

本文档描述 GraphDB 查询模块的分阶段修改计划。基于对查询模块操作类型管理和处理链条完整性的分析，本计划将改进工作划分为四个阶段，每个阶段都有明确的目标、任务和交付物。修改计划遵循"先基础后上层、先核心后外围"的原则，确保每个阶段的修改都能独立验证，降低风险。

分阶段修改的核心原则包括：
1. **渐进式改进**：每次修改都独立可验证，避免大规模一次性重构
2. **向后兼容**：保留现有接口，允许逐步迁移
3. **风险控制**：优先处理高优先级问题，确保系统稳定性
4. **可测试性**：每个阶段都有明确的测试目标

## 二、总体修改计划概览

### 2.1 四个阶段概览

| 阶段 | 名称 | 优先级 | 预计工作量 | 主要目标 |
|------|------|--------|------------|----------|
| 第一阶段 | 基础类型统一 | 高 | 2-3 周 | 统一操作类型枚举，建立类型映射 |
| 第二阶段 | 静态分发改造 | 高 | 2-3 周 | 完成 InputExecutor 静态分发改造 |
| 第三阶段 | 处理链条完善 | 中 | 3-4 周 | 完善 Validator 和 ExecutorFactory 映射 |
| 第四阶段 | 访问者模式与优化 | 中 | 2-3 周 | 实现 PlanNodeVisitor，优化规则 |

### 2.2 阶段依赖关系

```
第一阶段（类型统一）
    ↓
第二阶段（静态分发）
    ↓
第三阶段（链条完善）
    ↓
第四阶段（访问者与优化）
```

第一阶段是后续所有阶段的基础，必须首先完成。

## 三、第一阶段：基础类型统一

### 3.1 阶段目标

本阶段的核心目标是建立统一的操作类型枚举 `CoreOperationKind`，并实现各模块枚举之间的转换。主要任务包括：

1. 创建 `CoreOperationKind` 枚举，涵盖所有查询操作类型
2. 实现 `Stmt` 到 `CoreOperationKind` 的转换
3. 实现 `StatementType` 到 `CoreOperationKind` 的转换
4. 实现 `PlanNodeEnum` 到 `CoreOperationKind` 的转换
5. 更新各模块使用统一的类型枚举

### 3.2 具体任务

**任务 1.1：创建 CoreOperationKind 枚举**

位置：`src/query/core/operation_kind.rs`

```rust
/// 核心操作类型枚举 - 查询系统的类型基础
///
/// 此枚举统一了查询系统中的所有操作类型，贯穿 Parser、Validator、Planner、Optimizer 和 Executor 五个模块。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CoreOperationKind {
    // 数据查询操作
    Match, Go, Lookup, FindPath, GetSubgraph,
    
    // 数据访问操作
    ScanVertices, ScanEdges, GetVertices, GetEdges, GetNeighbors,
    
    // 数据转换操作
    Project, Filter, Sort, Limit, TopN, Sample, Unwind,
    
    // 数据聚合操作
    Aggregate, GroupBy, Having, Dedup,
    
    // 连接操作
    InnerJoin, LeftJoin, CrossJoin, HashJoin,
    
    // 图遍历操作
    Traverse, Expand, ExpandAll, ShortestPath, AllPaths,
    
    // 数据修改操作
    Insert, Update, Delete, Merge,
    
    // 模式匹配操作
    PatternApply, RollUpApply,
    
    // 循环控制操作
    Loop, ForLoop, WhileLoop,
    
    // 空间管理操作
    CreateSpace, DropSpace, DescribeSpace, UseSpace,
    
    // 标签管理操作
    CreateTag, AlterTag, DropTag, DescribeTag,
    
    // 边类型管理操作
    CreateEdge, AlterEdge, DropEdge, DescribeEdge,
    
    // 索引管理操作
    CreateIndex, DropIndex, DescribeIndex, RebuildIndex, FulltextIndexScan,
    
    // 用户管理操作
    CreateUser, AlterUser, DropUser, ChangePassword,
    
    // 其他操作
    Set, Explain, Show, Assignment,
}
```

交付物：
- `src/query/core/operation_kind.rs` 文件
- 完整的 `CoreOperationKind` 枚举定义
- 辅助方法（`category()`、`is_read_only()`、`is_metadata_operation()`）

**任务 1.2：创建类型转换 trait**

位置：`src/query/core/operation_kind.rs`

```rust
/// 类型转换 trait
pub trait IntoOperationKind {
    fn into_operation_kind(&self) -> CoreOperationKind;
}

/// 类型转换实现
impl IntoOperationKind for Stmt {
    fn into_operation_kind(&self) -> CoreOperationKind {
        // 实现转换逻辑
    }
}

impl IntoOperationKind for PlanNodeEnum {
    fn into_operation_kind(&self) -> CoreOperationKind {
        // 实现转换逻辑
    }
}
```

交付物：
- `IntoOperationKind` trait 定义
- 各模块枚举的转换实现

**任务 1.3：更新 Parser 模块**

位置：`src/query/parser/mod.rs`

```rust
pub mod operation_kind_support;

pub use operation_kind_support::{CoreOperationKind, IntoOperationKind};
```

交付物：
- Parser 模块导出类型转换功能
- Parser 模块的单元测试

**任务 1.4：更新 Validator 模块**

位置：`src/query/validator/mod.rs`

```rust
pub use crate::query::core::operation_kind::{CoreOperationKind, IntoOperationKind};

impl From<CoreOperationKind> for StatementType {
    fn from(kind: CoreOperationKind) -> Self {
        // 实现转换逻辑
    }
}
```

交付物：
- Validator 模块的类型转换功能
- Validator 模块的单元测试

**任务 1.5：更新 Planner 模块**

位置：`src/query/planner/mod.rs`

```rust
pub use crate::query::core::operation_kind::{CoreOperationKind, IntoOperationKind};

impl From<&PlanNodeEnum> for CoreOperationKind {
    fn from(node: &PlanNodeEnum) -> Self {
        // 实现转换逻辑
    }
}
```

交付物：
- Planner 模块的类型转换功能
- Planner 模块的单元测试

**任务 1.6：更新 Executor 模块**

位置：`src/query/executor/mod.rs`

```rust
pub use crate::query::core::operation_kind::{CoreOperationKind, IntoOperationKind};

impl<S: StorageEngine + Send + 'static> From<&ExecutorEnum<S>> for CoreOperationKind {
    fn from(exec: &ExecutorEnum<S>) -> Self {
        // 实现转换逻辑
    }
}
```

交付物：
- Executor 模块的类型转换功能
- Executor 模块的单元测试

### 3.3 验收标准

1. `CoreOperationKind` 枚举包含所有查询操作类型
2. 各模块枚举到 `CoreOperationKind` 的转换正确
3. 所有模块编译通过
4. 单元测试覆盖率不低于 80%
5. 集成测试覆盖主要查询路径

### 3.4 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 枚举变体遗漏 | 功能不完整 | 对照 NebulaGraph 文档逐一检查 |
| 转换逻辑错误 | 类型映射错误 | 编写完整的单元测试 |
| 编译错误 | 无法构建 | 逐步编译，及时修复 |

## 四、第二阶段：静态分发改造

### 4.1 阶段目标

本阶段的核心目标是完成 `InputExecutor` 和 `ChainableExecutor` 的静态分发改造，消除 `Box<dyn Executor<S>>` 的使用。主要任务包括：

1. 更新 `InputExecutor` trait 使用 `ExecutorEnum<S>` 替代 `Box<dyn Executor<S>>`
2. 更新所有执行器的 `InputExecutor` 实现
3. 实现 `ExecutorEnum` 的 `InputExecutor` 实现
4. 移除 `Box<dyn Executor<S>>` 的相关代码

### 4.2 具体任务

**任务 2.1：更新 InputExecutor trait**

位置：`src/query/executor/base/executor_base.rs`

```rust
/// 输入执行器 trait - 统一输入处理机制
///
/// 需要访问输入数据的执行器应实现此 trait。
/// 使用 ExecutorEnum 替代 Box<dyn Executor<S>>，实现静态分发。
pub trait InputExecutor<S: StorageEngine> {
    /// 设置输入数据
    fn set_input(&mut self, input: ExecutorEnum<S>);
    
    /// 获取输入数据
    fn get_input(&self) -> Option<&ExecutorEnum<S>>;
    
    /// 获取可变的输入数据
    fn get_input_mut(&mut self) -> Option<&mut ExecutorEnum<S>>;
}
```

交付物：
- 更新后的 `InputExecutor` trait 定义
- 向后兼容的别名（可选）

**任务 2.2：更新各执行器的 InputExecutor 实现**

位置：各执行器文件（如 `filter.rs`、`project.rs` 等）

```rust
impl<S: StorageEngine + Send + 'static> InputExecutor<S> for FilterExecutor<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        self.input = Some(input);
    }
    
    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        self.input.as_ref()
    }
    
    fn get_input_mut(&mut self) -> Option<&mut ExecutorEnum<S>> {
        self.input.as_mut()
    }
}
```

需要更新的执行器：
- `FilterExecutor`
- `ProjectExecutor`
- `LimitExecutor`
- `SortExecutor`
- `TopNExecutor`
- `SampleExecutor`
- `AggregateExecutor`
- `GroupByExecutor`
- `HavingExecutor`
- `DedupExecutor`
- `UnwindExecutor`
- `AssignExecutor`
- `AppendVerticesExecutor`
- `PatternApplyExecutor`
- `RollUpApplyExecutor`
- `LoopExecutor`
- `ForLoopExecutor`
- `WhileLoopExecutor`
- `TraverseExecutor`
- `ExpandExecutor`
- `ExpandAllExecutor`
- `ShortestPathExecutor`
- `MultiShortestPathExecutor`
- `AllPathsExecutor`
- `InnerJoinExecutor`
- `HashInnerJoinExecutor`
- `LeftJoinExecutor`
- `HashLeftJoinExecutor`
- `CrossJoinExecutor`

交付物：
- 所有执行器的 `InputExecutor` 实现更新
- 每个执行器的单元测试

**任务 2.3：实现 ExecutorEnum 的 InputExecutor 实现**

位置：`src/query/executor/executor_enum.rs`

```rust
#[async_trait]
impl<S: StorageEngine + Send + 'static> InputExecutor<S> for ExecutorEnum<S> {
    fn set_input(&mut self, input: ExecutorEnum<S>) {
        match self {
            ExecutorEnum::Filter(exec) => exec.set_input(input),
            ExecutorEnum::Project(exec) => exec.set_input(input),
            // ... 所有分支
        }
    }
    
    fn get_input(&self) -> Option<&ExecutorEnum<S>> {
        match self {
            ExecutorEnum::Filter(exec) => exec.get_input(),
            ExecutorEnum::Project(exec) => exec.get_input(),
            // ... 所有分支
        }
    }
    
    fn get_input_mut(&mut self) -> Option<&mut ExecutorEnum<S>> {
        match self {
            ExecutorEnum::Filter(exec) => exec.get_input_mut(),
            ExecutorEnum::Project(exec) => exec.get_input_mut(),
            // ... 所有分支
        }
    }
}
```

交付物：
- `ExecutorEnum` 的 `InputExecutor` 实现
- 编译器验证所有分支正确处理

**任务 2.4：移除 Box<dyn Executor<S>> 的使用**

搜索并移除 `Box<dyn Executor<S>>` 的使用：

```bash
# 搜索使用位置
grep -r "Box<dyn Executor" src/query/executor/

# 逐个移除或替换
```

需要处理的文件：
- `src/query/executor/base/executor_base.rs`（原始定义）
- `src/query/scheduler/async_scheduler.rs`（调度器使用）
- `src/query/scheduler/execution_schedule.rs`（执行计划使用）

交付物：
- `Box<dyn Executor<S>>` 使用完全移除
- 相关代码更新为使用 `ExecutorEnum<S>`

### 4.3 验收标准

1. `InputExecutor` trait 使用 `ExecutorEnum<S>` 类型参数
2. 所有执行器的 `InputExecutor` 实现更新
3. `ExecutorEnum` 实现 `InputExecutor` trait
4. `Box<dyn Executor<S>>` 不再出现在代码中
5. 所有执行器编译通过
6. 执行器相关测试全部通过

### 4.4 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| match 分支遗漏 | 编译错误 | 使用编译器错误定位遗漏分支 |
| 类型不兼容 | 编译错误 | 逐步编译，及时修复 |
| 性能回归 | 执行性能下降 | 性能测试验证 |

## 五、第三阶段：处理链条完善

### 5.1 阶段目标

本阶段的核心目标是完善处理链条中缺失的环节，确保所有查询类型都能正确处理。主要任务包括：

1. 完善 Validator 注册表，添加缺失的验证器
2. 完善 ExecutorFactory 映射，添加缺失的执行器
3. 完善 PlanNodeEnum 的 `is_*` 和 `as_*` 方法
4. 完善各模块的错误处理

### 5.2 具体任务

**任务 3.1：完善 Validator 注册表**

位置：`src/query/validator/validation_factory.rs`

```rust
fn register_default_validators(&mut self) {
    // 现有的验证器
    self.register("MATCH", || Validator::new());
    self.register("GO", || Validator::new());
    self.register("LOOKUP", || Validator::new());
    self.register("FETCH_VERTICES", || Validator::new());
    self.register("FETCH_EDGES", || Validator::new());
    self.register("USE", || Validator::new());
    self.register("PIPE", || Validator::new());
    self.register("YIELD", || Validator::new());
    self.register("ORDER_BY", || Validator::new());
    self.register("LIMIT", || Validator::new());
    self.register("UNWIND", || Validator::new());
    self.register("FIND_PATH", || Validator::new());
    self.register("GET_SUBGRAPH", || Validator::new());
    self.register("SET", || Validator::new());
    
    // 新增验证器
    self.register("INSERT_VERTICES", || Validator::new());
    self.register("INSERT_EDGES", || Validator::new());
    self.register("UPDATE", || Validator::new());
    self.register("DELETE", || Validator::new());
    self.register("CREATE_SPACE", || Validator::new());
    self.register("DROP_SPACE", || Validator::new());
    self.register("CREATE_TAG", || Validator::new());
    self.register("ALTER_TAG", || Validator::new());
    self.register("DROP_TAG", || Validator::new());
    self.register("CREATE_EDGE", || Validator::new());
    self.register("ALTER_EDGE", || Validator::new());
    self.register("DROP_EDGE", || Validator::new());
    self.register("SHOW_SPACES", || Validator::new());
    self.register("SHOW_TAGS", || Validator::new());
    self.register("SHOW_EDGES", || Validator::new());
}
```

交付物：
- 完整的 Validator 注册表
- 所有语句类型的验证器注册

**任务 3.2：添加缺失的执行器**

需要添加的执行器：

| 执行器 | 优先级 | 依赖 |
|--------|--------|------|
| FulltextIndexScanExecutor | 高 | 全文索引支持 |
| DataCollectExecutor | 中 | 数据收集节点 |
| ArgumentExecutor | 中 | 参数传递节点 |
| PassThroughExecutor | 低 | 直通节点 |
| BFSShortestExecutor | 中 | BFS 最短路径 |

以 `FulltextIndexScanExecutor` 为例：

```rust
/// 全文索引扫描执行器
pub struct FulltextIndexScanExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    index_name: String,
    pattern: String,
    limit: Option<usize>,
}

impl<S: StorageEngine + Send + 'static> FulltextIndexScanExecutor<S> {
    pub fn new(node: &FulltextIndexScanNode, storage: Arc<Mutex<S>>) -> Result<Self, QueryError> {
        Ok(Self {
            base: BaseExecutor::new(node.id(), "FulltextIndexScan".to_string(), storage),
            index_name: node.index_name().to_string(),
            pattern: node.pattern().to_string(),
            limit: node.limit(),
        })
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for FulltextIndexScanExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 实现全文索引扫描逻辑
        todo!()
    }
}
```

交付物：
- 所有缺失执行器的实现
- 执行器工厂的更新

**任务 3.3：完善 ExecutorFactory 映射**

位置：`src/query/executor/factory.rs`

```rust
fn analyze_plan_node(&mut self, node: &PlanNodeEnum, loop_layers: usize) -> Result<(), QueryError> {
    match node {
        // 已处理的节点
        PlanNodeEnum::Filter(n) => self.analyze_plan_node(n.input(), loop_layers)?,
        PlanNodeEnum::Project(n) => self.analyze_plan_node(n.input(), loop_layers)?,
        PlanNodeEnum::Limit(n) => self.analyze_plan_node(n.input(), loop_layers)?,
        PlanNodeEnum::Sort(n) => self.analyze_plan_node(n.input(), loop_layers)?,
        
        // 新增处理的节点
        PlanNodeEnum::Loop(n) => {
            if let Some(body) = n.body() {
                self.analyze_plan_node(body, loop_layers + 1)?;
            }
        }
        PlanNodeEnum::ForLoop(n) => {
            if let Some(body) = n.body() {
                self.analyze_plan_node(body, loop_layers + 1)?;
            }
        }
        PlanNodeEnum::WhileLoop(n) => {
            if let Some(body) = n.body() {
                self.analyze_plan_node(body, loop_layers + 1)?;
            }
        }
        PlanNodeEnum::FulltextIndexScan(n) => {
            // 全文索引扫描，无子节点
        }
        
        // 避免默认错误
        _ => {
            log::warn!("未处理的计划节点类型: {:?}", node.type_name());
        }
    }
    Ok(())
}
```

交付物：
- 完整的 `analyze_plan_node` 函数
- 警告日志替代错误抛出

**任务 3.4：完善 PlanNodeEnum 的辅助方法**

位置：`src/query/planner/plan/core/nodes/plan_node_enum.rs`

```rust
impl PlanNodeEnum {
    // 现有的方法
    pub fn is_start(&self) -> bool { /* ... */ }
    pub fn is_project(&self) -> bool { /* ... */ }
    
    // 新增的方法
    pub fn is_loop(&self) -> bool {
        matches!(self, PlanNodeEnum::Loop(_))
    }
    
    pub fn is_for_loop(&self) -> bool {
        matches!(self, PlanNodeEnum::ForLoop(_))
    }
    
    pub fn is_while_loop(&self) -> bool {
        matches!(self, PlanNodeEnum::WhileLoop(_))
    }
    
    pub fn is_fulltext_index_scan(&self) -> bool {
        matches!(self, PlanNodeEnum::FulltextIndexScan(_))
    }
    
    pub fn as_loop(&self) -> Option<&LoopNode> {
        match self {
            PlanNodeEnum::Loop(n) => Some(n),
            _ => None,
        }
    }
    
    pub fn as_fulltext_index_scan(&self) -> Option<&FulltextIndexScanNode> {
        match self {
            PlanNodeEnum::FulltextIndexScan(n) => Some(n),
            _ => None,
        }
    }
}
```

交付物：
- 完整的 `is_*` 和 `as_*` 方法集
- 所有节点类型的辅助方法

### 5.3 验收标准

1. Validator 注册表包含所有语句类型的验证器
2. ExecutorFactory 能处理所有 PlanNodeEnum 变体
3. 所有缺失的执行器已添加
4. PlanNodeEnum 的辅助方法完整
5. 所有查询类型都能通过处理链条
6. 集成测试覆盖所有查询类型

### 5.4 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 执行器实现复杂 | 开发周期延长 | 优先实现高频使用的执行器 |
| 验证逻辑不完整 | 错误到执行阶段才发现 | 编写完整的验证测试 |
| 映射遗漏 | 查询处理失败 | 自动化测试覆盖所有路径 |

## 六、第四阶段：访问者模式与优化

### 6.1 阶段目标

本阶段的核心目标是引入 `PlanNodeVisitor` 访问者模式，简化优化规则的实现，并完成剩余的优化工作。主要任务包括：

1. 实现 `PlanNodeVisitor` trait 和默认实现
2. 更新优化规则使用访问者模式
3. 添加缺失的优化规则
4. 完成访问者模式在 Scheduler 中的应用

### 6.2 具体任务

**任务 4.1：实现 PlanNodeVisitor**

位置：`src/query/visitor/plan_node_visitor.rs`

```rust
/// PlanNode 访问者 trait
///
/// 提供统一的 PlanNode 遍历接口，简化优化规则和数据转换的实现。
pub trait PlanNodeVisitor {
    /// 访问结果的类型
    type Result;
    
    /// 访问开始节点
    fn visit_start(&mut self, node: &StartNode) -> Self::Result;
    
    /// 访问项目节点
    fn visit_project(&mut self, node: &ProjectNode) -> Self::Result;
    
    // ... 其他节点类型的访问方法
    
    /// 统一的访问入口
    fn visit(&mut self, node: &PlanNodeEnum) -> Self::Result;
}

/// 默认的 PlanNode 访问者实现
pub struct DefaultPlanNodeVisitor;

impl PlanNodeVisitor for DefaultPlanNodeVisitor {
    type Result = ();
    
    fn visit_start(&mut self, _node: &StartNode) {}
    fn visit_project(&mut self, _node: &ProjectNode) {}
    // ... 其他默认实现
}
```

交付物：
- `PlanNodeVisitor` trait 定义
- `DefaultPlanNodeVisitor` 实现
- 完整的访问者接口文档

**任务 4.2：更新 visitor 模块**

位置：`src/query/visitor/mod.rs`

```rust
mod plan_node_visitor;

pub use plan_node_visitor::{PlanNodeVisitor, DefaultPlanNodeVisitor};
```

交付物：
- visitor 模块的更新
- 模块导出正确的类型

**任务 4.3：更新优化规则使用访问者模式**

以 `TopNRule` 为例：

```rust
impl OptRule for TopNRule {
    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 使用访问者模式简化逻辑
        let visitor = TopNRuleVisitor {
            ctx,
            converted: false,
        };
        
        let result = visitor.visit(&node.plan_node);
        if result.converted {
            Ok(Some(result.new_node))
        } else {
            Ok(None)
        }
    }
}

struct TopNRuleVisitor<'a> {
    ctx: &'a mut OptContext,
    converted: bool,
    new_node: OptGroupNode,
}

impl<'a> PlanNodeVisitor for TopNRuleVisitor<'a> {
    type Result = TopNRuleVisitor<'a>;
    
    fn visit_limit(&mut self, node: &LimitNode) -> Self::Result {
        // 检查 Limit 下是否是 Sort
        self.ctx.find_group_node_by_plan_node_id(node.input())
            .map(|child| {
                if child.plan_node.is_sort() {
                    // 转换为 TopN
                    self.converted = true;
                    // ...
                }
            });
        self
    }
    
    fn visit(&mut self, node: &PlanNodeEnum) -> Self::Result {
        // 使用统一的访问入口
        PlanNodeVisitor::visit(self, node);
        self
    }
}
```

需要更新的规则：
- `TopNRule`
- `LimitPushdown`
- `PredicatePushdown`
- `ProjectionPushdown`

交付物：
- 更新后的优化规则
- 使用访问者模式的规则实现示例

**任务 4.4：添加缺失的优化规则**

缺失的优化规则：

| 规则名称 | 类型 | 优先级 | 描述 |
|----------|------|--------|------|
| PredicateReorder | 重排序 | 中 | 重新排列谓词顺序 |
| ConstantFolding | 常量折叠 | 高 | 折叠常量表达式 |
| SubQueryOptimization | 子查询优化 | 中 | 优化子查询执行 |
| LoopUnrolling | 循环展开 | 低 | 展开简单循环 |

以 `ConstantFoldingRule` 为例：

```rust
/// 常量折叠规则
///
/// 将可以在编译时计算的常量表达式折叠为常量值。
pub struct ConstantFoldingRule;

impl OptRule for ConstantFoldingRule {
    fn name(&self) -> &str {
        "ConstantFoldingRule"
    }
    
    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        let visitor = ConstantFoldingVisitor { folded: false };
        let result = visitor.visit(&node.plan_node);
        
        if result.folded {
            Ok(Some(node.clone()))
        } else {
            Ok(None)
        }
    }
}
```

交付物：
- 缺失优化规则的实现
- 规则的单元测试

**任务 4.5：在 Scheduler 中应用访问者模式**

位置：`src/query/scheduler/execution_schedule.rs`

```rust
/// 执行计划分析访问者
struct ExecutionPlanAnalyzer {
    executors: Vec<ExecutorEnum<S>>,
    dependencies: HashMap<i64, Vec<i64>>,
}

impl<S: StorageEngine + Send + 'static> PlanNodeVisitor for ExecutionPlanAnalyzer {
    type Result = ();
    
    fn visit_start(&mut self, node: &StartNode) {
        let executor = ExecutorEnum::Start(StartExecutor::new(node, self.storage.clone()));
        self.executors.push(executor);
    }
    
    fn visit_project(&mut self, node: &ProjectNode) {
        let executor = ExecutorEnum::Project(ProjectExecutor::new(node, self.storage.clone()));
        self.executors.push(executor);
    }
    
    // ... 其他节点类型的处理
    
    fn visit(&mut self, node: &PlanNodeEnum) {
        PlanNodeVisitor::visit(self, node);
    }
}
```

交付物：
- 使用访问者模式的执行计划分析
- 简化的 Scheduler 实现

### 6.3 验收标准

1. `PlanNodeVisitor` 接口定义完整
2. 所有优化规则支持使用访问者模式
3. 缺失的优化规则已添加
4. Scheduler 使用访问者模式分析执行计划
5. 优化规则的单元测试通过
6. 性能测试验证优化效果

### 6.4 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 访问者模式学习成本 | 开发效率下降 | 提供详细的使用文档和示例 |
| 规则迁移工作量 | 开发周期延长 | 分批迁移，逐步验证 |
| 性能影响 | 优化效果下降 | 性能测试验证 |

## 七、测试计划

### 7.1 单元测试

每个阶段都应包含相应的单元测试：

**第一阶段测试**

```rust
#[cfg(test)]
mod operation_kind_tests {
    use super::*;
    
    #[test]
    fn test_stmt_to_operation_kind() {
        let match_stmt = /* 创建 MatchStmt */;
        assert_eq!(CoreOperationKind::from(&match_stmt), CoreOperationKind::Match);
    }
    
    #[test]
    fn test_plan_node_to_operation_kind() {
        let filter_node = /* 创建 FilterNode */;
        assert_eq!(CoreOperationKind::from(&filter_node), CoreOperationKind::Filter);
    }
}
```

**第二阶段测试**

```rust
#[cfg(test)]
mod input_executor_tests {
    #[test]
    fn test_input_executor_set_get() {
        let mut filter: FilterExecutor<MockStorage> = /* 创建 */;
        let input: ExecutorEnum<MockStorage> = /* 创建 */;
        
        filter.set_input(input);
        assert!(filter.get_input().is_some());
    }
}
```

### 7.2 集成测试

每个阶段完成后应运行集成测试：

```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_query_pipeline() {
        let pipeline = QueryPipeline::new();
        let result = pipeline.process("MATCH (n) RETURN n LIMIT 10").await;
        assert!(result.is_ok());
    }
}
```

### 7.3 性能测试

第二阶段完成后应进行性能测试：

```rust
#[cfg(test)]
mod performance_tests {
    #[tokio::test]
    async fn test_executor_performance() {
        let iterations = 1000;
        let start = Instant::now();
        for _ in 0..iterations {
            /* 执行查询 */
        }
        let duration = start.elapsed();
        assert!(duration.as_millis() < 10000); // 10 秒内完成 1000 次
    }
}
```

## 八、时间安排

### 8.1 各阶段时间估算

| 阶段 | 任务数 | 预计工作量 | 缓冲时间 |
|------|--------|------------|----------|
| 第一阶段 | 6 | 2-3 周 | 3 天 |
| 第二阶段 | 4 | 2-3 周 | 3 天 |
| 第三阶段 | 4 | 3-4 周 | 1 周 |
| 第四阶段 | 5 | 2-3 周 | 3 天 |

### 8.2 里程碑

| 里程碑 | 时间 | 验收标准 |
|--------|------|----------|
| M1：第一阶段完成 | 第 3 周 | CoreOperationKind 枚举完整，类型转换正确 |
| M2：第二阶段完成 | 第 6 周 | 静态分发改造完成，无 Box<dyn Executor> |
| M3：第三阶段完成 | 第 10 周 | Validator 和 ExecutorFactory 完整 |
| M4：第四阶段完成 | 第 13 周 | 访问者模式实现，优化规则完整 |

## 九、资源需求

### 9.1 开发资源

| 角色 | 人数 | 职责 |
|------|------|------|
| 高级开发人员 | 1 | 架构设计、核心代码编写 |
| 开发人员 | 2 | 各模块实现、测试编写 |

### 9.2 测试资源

| 资源 | 需求 |
|------|------|
| 单元测试环境 | 本地开发环境 |
| 集成测试环境 | 测试服务器 |
| 性能测试环境 | 专用测试机器 |

## 十、总结

本文档描述了 GraphDB 查询模块的分阶段修改计划。修改计划分为四个阶段：

1. **第一阶段**：基础类型统一，建立 `CoreOperationKind` 枚举
2. **第二阶段**：静态分发改造，消除 `Box<dyn Executor<S>>` 的使用
3. **第三阶段**：处理链条完善，补充缺失的验证器和执行器
4. **第四阶段**：访问者模式与优化，实现 `PlanNodeVisitor` 和缺失的优化规则

每个阶段都有明确的目标、任务和验收标准。通过分阶段的渐进式改进，可以有效降低重构风险，确保系统稳定性。

## 十一、参考文档

- [查询模块操作类型分析](query_operation_type_analysis.md)
- [处理链条完整性分析](processing_chain_integrity_analysis.md)
- [改进后的架构设计文档](improved_architecture_design.md)
- [模块问题与解决方案](modules_issues_and_solutions.md)
