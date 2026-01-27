# Query Planner 模块关系分析

## 概述

`src/query/planner` 目录是 GraphDB 查询引擎的核心组件之一，负责将解析后的 AST（抽象语法树）转换为可执行的查询计划。该模块采用分层架构设计，实现了从语句到子句再到具体执行节点的完整规划流程。

## 顶层架构

```
src/query/planner/
├── mod.rs                 # 模块入口，重新导出主要类型
├── planner.rs             # 规划器核心trait和注册机制
├── connector.rs           # 计划连接器，用于连接子计划
├── statements/            # 语句级规划器
│   ├── mod.rs
│   ├── statement_planner.rs
│   ├── match_statement_planner.rs
│   ├── match_planner.rs
│   ├── go_planner.rs
│   ├── lookup_planner.rs
│   ├── path_planner.rs
│   ├── subgraph_planner.rs
│   ├── fetch_vertices_planner.rs
│   ├── fetch_edges_planner.rs
│   ├── maintain_planner.rs
│   ├── core/              # 核心规划trait和工具
│   ├── clauses/           # 子句级规划器
│   ├── paths/             # 路径规划相关
│   └── seeks/             # 查找策略相关
└── plan/                  # 执行计划和节点定义
    ├── mod.rs
    ├── execution_plan.rs
    ├── common.rs
    ├── algorithms/        # 算法相关节点
    ├── core/              # 核心节点定义
    │   ├── mod.rs
    │   ├── nodes/         # 各种计划节点
    │   ├── common.rs
    │   └── explain.rs
    └── management/        # 管理操作相关
        ├── admin/
        ├── ddl/
        ├── dml/
        └── security/
```

## 模块依赖关系

### 核心模块职责

#### 1. `planner.rs` - 规划器核心

这是整个规划系统的中枢模块，包含以下核心组件：

- **Planner trait**：所有规划器必须实现的基础接口
  ```rust
  pub trait Planner {
      fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
      fn match_planner(&self, ast_ctx: &AstContext) -> bool;
      fn name(&self) -> &'static str;
  }
  ```

- **SentenceKind 枚举**：定义支持的语句类型
  - Match、Go、Lookup、Path、Subgraph、FetchVertices、FetchEdges、Maintain

- **PlannerRegistry**：规划器注册表，管理所有可用规划器
  - 支持按语句类型注册和查找规划器
  - 支持优先级机制，处理匹配冲突

- **PlanCache**：计划缓存，提高重复查询性能
  - 基于 LRU 策略管理缓存
  - 支持按查询文本、空间ID、语句类型生成缓存键

- **SequentialPlanner**：顺序规划器，协调多个子规划器

#### 2. `statements/` - 语句级规划器

语句级规划器负责处理完整的查询语句，每个语句类型对应一个专门的规划器：

**主语句规划器**：
- `match_planner.rs` - MATCH 语句规划器
- `go_planner.rs` - GO 语句规划器（NGQL）
- `lookup_planner.rs` - LOOKUP 语句规划器
- `path_planner.rs` - PATH 语句规划器
- `subgraph_planner.rs` - SUBGRAPH 语句规划器
- `fetch_vertices_planner.rs` - FETCH VERTICES 规划器
- `fetch_edges_planner.rs` - FETCH EDGES 规划器
- `maintain_planner.rs` - MAINTAIN 语句规划器

**语句规划器架构**：
```
StatementPlanner trait
    ↓
BaseStatementPlanner (通用实现)
    ↓
具体语句规划器 (MatchPlanner, GoPlanner, etc.)
```

**StatementPlanner trait** 定义：
```rust
pub trait StatementPlanner: Planner {
    fn statement_type(&self) -> &'static str;
    fn supported_clause_kinds(&self) -> Vec<CypherClauseKind>;
    fn extract_clauses(&self, ast_ctx: &AstContext) -> Vec<CypherClauseKind>;
    fn plan_with_clause_planners(...) -> Result<ExecutionPlan, PlannerError>;
}
```

#### 3. `statements/core/` - 核心规划组件

提供语句规划的基础设施：

- **CypherClausePlanner trait**：子句规划器的核心接口
- **ClauseType 枚举**：定义子句类型（Match、Where、Return、With、OrderBy、Limit、Unwind、Yield）
- **PlanningContext**：规划过程中的上下文管理
- **DataFlowManager**：数据流验证和管理
- **ContextPropagator**：上下文传播工具

#### 4. `statements/clauses/` - 子句级规划器

子句级规划器处理查询的单个组成部分：

| 文件 | 子句类型 | 职责 |
|------|---------|------|
| `clause_planner.rs` | 基类 | 定义 ClausePlanner trait 和 BaseClausePlanner |
| `return_clause_planner.rs` | RETURN | 处理 RETURN 子句的投影逻辑 |
| `where_clause_planner.rs` | WHERE | 处理 WHERE 子句的过滤逻辑 |
| `with_clause_planner.rs` | WITH | 处理 WITH 子句的中间投影 |
| `order_by_planner.rs` | ORDER BY | 处理排序逻辑 |
| `pagination_planner.rs` | LIMIT/SKIP | 处理分页逻辑 |
| `projection_planner.rs` | 投影 | 通用投影逻辑 |
| `unwind_planner.rs` | UNWIND | 处理列表展开 |
| `yield_planner.rs` | YIELD | 处理 YIELD 子句 |

**子句规划器层次**：
```
ClausePlanner trait (扩展 CypherClausePlanner)
    ↓
BaseClausePlanner (基类实现)
    ↓
具体子句规划器 (ReturnClausePlanner, WhereClausePlanner, etc.)
```

#### 5. `statements/paths/` - 路径规划

处理 MATCH 查询中的路径模式：

- **match_path_planner.rs**：匹配路径模式规划器
  - 支持节点路径和边路径
  - 路径模式解析和验证

- **shortest_path_planner.rs**：最短路径规划器
  - BFS 算法实现
  - 支持配置起始点来源

#### 6. `statements/seeks/` - 查找策略

定义顶点查找策略和选择器：

- **seek_strategy.rs**：查找策略 trait 和实现
- **seek_strategy_base.rs**：查找策略基类和上下文
- **vertex_seek.rs**：顶点查找策略
- **index_seek.rs**：索引查找策略
- **scan_seek.rs**：全表扫描策略

**查找策略层次**：
```
SeekStrategy trait
    ↓
SeekStrategyBase (基类)
    ↓
具体策略 (VertexSeek, IndexSeek, ScanSeek)
```

#### 7. `plan/` - 执行计划结构

定义查询执行计划的物理表示：

**核心数据结构**：
- **ExecutionPlan**：完整的可执行计划
  - 包含根节点和计划ID
  - 支持优化时间跟踪

- **SubPlan**：子计划，用于复杂查询的分段规划
  - 包含 root 和 tail 节点
  - 支持计划的连接和合并

**计划节点枚举 (PlanNodeEnum)**：
- **StartNode**：起始节点，查询的入口点
- **ScanVerticesNode**：顶点扫描节点
- **ScanEdgesNode**：边扫描节点
- **GetVerticesNode**：获取顶点节点
- **GetEdgesNode**：获取边节点
- **GetNeighborsNode**：获取邻居节点
- **ExpandAllNode**：全量扩展节点
- **ExpandNode**：扩展节点
- **AppendVerticesNode**：追加顶点节点
- **TraverseNode**：遍历节点
- **FilterNode**：过滤节点
- **ProjectNode**：投影节点
- **SortNode**：排序节点
- **LimitNode**：限制节点
- **AggregateNode**：聚合节点
- **DedupNode**：去重节点
- **UnwindNode**：展开节点
- **ArgumentNode**：参数节点
- **JoinNode 系列**：各类连接节点（InnerJoin、LeftJoin、HashInnerJoin 等）

#### 8. `plan/management/` - 管理操作

处理数据库管理相关的操作：

**子模块结构**：
- **admin/**：系统管理操作
  - `config_ops.rs`：配置操作
  - `host_ops.rs`：主机操作
  - `index_ops.rs`：索引操作
  - `system_ops.rs`：系统操作

- **ddl/**：数据定义语言
  - `space_ops.rs`：图空间操作
  - `tag_ops.rs`：标签操作
  - `edge_ops.rs`：边类型操作

- **dml/**：数据操作语言
  - `insert_ops.rs`：插入操作
  - `update_ops.rs`：更新操作
  - `delete_ops.rs`：删除操作
  - `data_constructors.rs`：数据构造器

- **security/**：安全管理
  - `user_ops.rs`：用户操作
  - `role_ops.rs`：角色操作

#### 9. `connector.rs` - 计划连接器

提供计划节点之间的连接功能：

- **JoinType 枚举**：连接类型（Inner、Left、Right、Full）
- **SegmentsConnector 结构体**：
  - `inner_join()`：内连接
  - `left_join()`：左连接
  - `cross_join()`：交叉连接
  - `add_input()`：添加输入

## 数据流分析

### 规划流程

```
1. 查询输入
   ↓
2. Parser → AST
   ↓
3. Validator → Validated AST
   ↓
4. Planner (planner.rs)
   ├─ 确定语句类型 (SentenceKind)
   └─ 选择合适的语句规划器
   ↓
5. StatementPlanner (statements/*.rs)
   ├─ 提取子句列表
   └─ 按顺序调用子句规划器
   ↓
6. ClausePlanner (statements/clauses/*.rs)
   ├─ 验证子句上下文
   ├─ 估算执行成本
   └─ 生成子计划 (SubPlan)
   ↓
7. 计划连接 (connector.rs)
   ├─ 连接多个子计划
   └─ 生成最终执行计划
   ↓
8. ExecutionPlan
   ├─ 设置根节点
   └─ 设置计划ID
   ↓
9. 执行引擎
```

### 计划构建示例：MATCH 查询

以 `MATCH (n:Person) WHERE n.age > 25 RETURN n.name ORDER BY n.name LIMIT 10` 为例：

```
StartNode (起始点)
    ↓
ScanVerticesNode (扫描顶点，匹配 Person 标签)
    ↓
FilterNode (过滤 n.age > 25)
    ↓
ProjectNode (投影 n.name)
    ↓
SortNode (按 n.name 排序)
    ↓
LimitNode (限制 10 条)
    ↓
ExecutionPlan
```

## 模块间依赖图

```
planner.rs
    ├── depends on: AstContext, QueryContext
    ├── depends on: ExecutionPlan, SubPlan
    ├── depends on: PlanNodeEnum
    └── provides: Planner, PlannerRegistry, PlanCache

statements/mod.rs
    ├── depends on: planner.rs
    ├── depends on: plan/ (ExecutionPlan, SubPlan)
    ├── re-exports: core/, clauses/, paths/, seeks/
    └── provides: StatementPlanner trait

statements/core/
    ├── depends on: plan/ (PlanNodeEnum)
    ├── depends on: validator/ (CypherClauseKind)
    └── provides: CypherClausePlanner, PlanningContext

statements/clauses/
    ├── depends on: statements/core/
    ├── depends on: plan/ (SubPlan, PlanNodeEnum)
    └── provides: ClausePlanner trait implementations

statements/paths/
    ├── depends on: statements/core/
    ├── depends on: plan/ (nodes/)
    └── provides: MatchPathPlanner, ShortestPathPlanner

statements/seeks/
    ├── depends on: statements/core/
    ├── depends on: index/ (IndexInfo)
    └── provides: SeekStrategy implementations

plan/mod.rs
    ├── provides: ExecutionPlan, SubPlan
    ├── re-exports: algorithms/, core/, management/
    └── depends on: core/nodes/ (PlanNodeEnum)

plan/core/nodes/
    ├── provides: All plan node types
    ├── depends on: plan_node_traits.rs
    └── provides: PlanNodeFactory

plan/management/
    ├── depends on: plan/core/nodes/
    └── provides: ManagementNodeEnum and operations

connector.rs
    ├── depends on: plan/ (SubPlan, PlanNodeEnum)
    └── provides: SegmentsConnector
```

## 设计模式应用

### 1. 策略模式

**查找策略 (SeekStrategy)**：
```rust
pub trait SeekStrategy {
    fn seek(&self, context: &SeekStrategyContext) -> SeekResult;
}
```

**实现**：
- `VertexSeek`：顶点查找策略
- `IndexSeek`：索引查找策略
- `ScanSeek`：全表扫描策略

### 2. 模板方法模式

**ClausePlanner**：
```rust
pub trait ClausePlanner: CypherClausePlanner {
    fn name(&self) -> &'static str;
    fn supported_clause_kind(&self) -> CypherClauseKind;
    fn validate_context(&self, clause_ctx: &CypherClauseContext) -> Result<(), PlannerError>;
    fn estimate_cost(&self, clause_ctx: &CypherClauseContext) -> f64;
    // 子类实现具体逻辑
}
```

### 3. 工厂模式

**PlanNodeFactory**：
```rust
pub struct PlanNodeFactory;
impl PlanNodeFactory {
    pub fn create_scan_node(...) -> Result<ScanVerticesNode, PlanError>;
    pub fn create_filter_node(...) -> Result<FilterNode, PlanError>;
    // ...
}
```

### 4. 注册表模式

**PlannerRegistry**：
```rust
pub struct PlannerRegistry {
    planners: HashMap<SentenceKind, Vec<MatchAndInstantiate>>,
}
```

允许动态注册和选择规划器。

### 5. 组合模式

**SubPlan 和 ExecutionPlan**：
- `ExecutionPlan` 包含 `PlanNodeEnum`（根节点）
- `SubPlan` 包含 `PlanNodeEnum`（根节点和尾节点）
- 计划节点可以包含子节点，形成树形结构

## 关键接口契约

### Planner trait

```rust
pub trait Planner: std::fmt::Debug {
    // 核心转换方法
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
    
    // 检查是否匹配
    fn match_planner(&self, ast_ctx: &AstContext) -> bool;
    
    // 带完整上下文转换
    fn transform_with_full_context(
        &mut self,
        query_context: &mut QueryContext,
        ast_ctx: &AstContext,
    ) -> Result<ExecutionPlan, PlannerError>;
    
    // 获取名称
    fn name(&self) -> &'static str;
}
```

### StatementPlanner trait

```rust
pub trait StatementPlanner: Planner {
    fn statement_type(&self) -> &'static str;
    fn supported_clause_kinds(&self) -> Vec<CypherClauseKind>;
    fn extract_clauses(&self, ast_ctx: &AstContext) -> Vec<CypherClauseKind>;
    fn plan_with_clause_planners(...) -> Result<ExecutionPlan, PlannerError>;
}
```

### ClausePlanner trait

```rust
pub trait ClausePlanner: CypherClausePlanner {
    fn name(&self) -> &'static str;
    fn supported_clause_kind(&self) -> CypherClauseKind;
    fn validate_context(&self, clause_ctx: &CypherClauseContext) -> Result<(), PlannerError>;
    fn transform_clause(...) -> Result<SubPlan, PlannerError>;
}
```

## 扩展点

### 1. 添加新语句类型

1. 在 `SentenceKind` 枚举中添加新类型
2. 创建对应的语句规划器（实现 `Planner` trait）
3. 在 `SequentialPlanner::register_planners()` 中注册

### 2. 添加新子句类型

1. 在 `CypherClauseKind` 枚举中添加新类型
2. 在 `ClauseType` 枚举中添加映射
3. 创建对应的子句规划器
4. 注册到 `PlannerRegistry`

### 3. 添加新查找策略

1. 实现 `SeekStrategy` trait
2. 在 `SeekStrategySelector` 中注册策略
3. 实现选择逻辑

### 4. 添加新计划节点

1. 在 `PlanNodeEnum` 中添加变体
2. 实现 `PlanNode` trait
3. 在必要节点中实现 `accept()` 方法支持遍历

## 总结

Query Planner 模块采用了清晰的分层架构：

1. **顶层**：规划器注册和协调（planner.rs）
2. **语句层**：各类语句的规划器（statements/*.rs）
3. **子句层**：各子句的处理逻辑（statements/clauses/*.rs）
4. **执行层**：具体的物理执行节点（plan/*.rs）

这种设计使得：
- 各层职责清晰，耦合度低
- 易于扩展新语句和子句类型
- 支持灵活的查找策略和连接策略
- 计划缓存提高重复查询性能
