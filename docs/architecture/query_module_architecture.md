# GraphDB Query 模块架构分析

## 概述

本文档描述了 GraphDB 查询引擎的架构设计，分析 `src/query` 目录下各个模块的职责划分、模块间关系以及数据流转过程。查询引擎采用经典的管道（Pipeline）架构，将查询处理分解为解析、验证、规划、优化和执行五个核心阶段，每个阶段由专门的模块负责。

## 整体架构

### 查询处理管道

GraphDB 查询引擎的核心是一个五阶段的处理管道，每个阶段都有明确的职责边界：

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Parser    │ -> │  Validator  │ -> │   Planner   │ -> │  Optimizer  │ -> │  Executor   │
│   (解析)    │    │   (验证)    │    │   (规划)    │    │   (优化)    │    │   (执行)    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
      │                  │                  │                  │                  │
      v                  v                  v                  v                  v
  ┌────────┐        ┌────────┐        ┌────────┐        ┌────────┐        ┌────────┐
  │  AST   │        │  AST   │        │  Plan  │        │  Plan  │        │ Result │
  │ 上下文  │        │ 上下文  │        │  树    │        │  树    │        │  数据  │
  └────────┘        └────────┘        └────────┘        └────────┘        └────────┘
```

**QueryPipelineManager** 是整个管道的协调者，负责管理查询的全生命周期。该类取代了原来的 QueryConverter，现在负责：管理查询处理的全生命周期、协调各个处理阶段（解析→验证→规划→优化→执行）、处理错误和异常、管理查询上下文。通过调用 `execute_query` 方法，依次经过解析、验证、规划、优化、执行五个阶段，最终返回查询结果。

## 模块详解

### 1. Parser 模块

**位置**: `src/query/parser/`

**职责**: 负责将用户输入的查询文本转换为抽象语法树（AST）。这是查询处理的第一道关口。

**子模块结构**:

```
parser/
├── core/          # 核心类型定义
│   ├── error.rs   # 解析错误类型
│   ├── token.rs   # Token 定义
│   └── position.rs # 位置信息
├── lexer/         # 词法分析器
│   ├── lexer.rs   # 主词法分析器
│   └── mod.rs     # 模块导出
├── ast/           # 抽象语法树
│   ├── stmt.rs    # 语句定义
│   ├── types.rs   # 类型定义
│   └── utils.rs   # 工具函数
├── expressions/   # 表达式处理
│   ├── expression_converter.rs
│   └── mod.rs
└── parser/        # 语法分析器
    ├── expr_parser.rs  # 表达式解析
    ├── stmt_parser.rs  # 语句解析
    └── mod.rs
```

**核心类型**:

- `Parser`: 主解析器入口，提供 `new()` 和 `parse()` 方法
- `ExprParser`: 表达式解析器，处理 WHERE、YIELD 等子句中的表达式
- `StmtParser`: 语句解析器，处理 MATCH、GO、CREATE 等语句
- `Token` / `TokenKind`: 词法单元定义
- `Position` / `Span`: 源码位置信息，用于错误定位

**输出**: `QueryAstContext`，包含解析后的语句（Stmt）对象

**与上下游关系**: Parser 的输出（AST）作为 Validator 的输入；Parser 依赖 context 模块中的 `QueryAstContext` 来存储解析结果。

### 2. Context 模块

**位置**: `src/query/context/`

**职责**: 管理查询处理的上下文信息，包括 AST 上下文、执行上下文、运行时上下文等。这是整个查询引擎的数据中心。

**子模块结构**:

```
context/
├── ast/                    # AST 上下文
│   ├── base.rs            # 基础上下文
│   ├── common.rs          # 公共类型
│   ├── cypher_ast_context.rs
│   ├── query_ast_context.rs
│   ├── query_types/       # 查询类型（GO、FETCH、LOOKUP 等）
│   │   ├── go.rs
│   │   ├── fetch_vertices.rs
│   │   ├── fetch_edges.rs
│   │   ├── lookup.rs
│   │   └── ...
│   └── mod.rs
├── execution/             # 执行上下文
│   ├── execution_context.rs
│   ├── query_execution.rs
│   └── mod.rs
├── managers/              # 管理器接口
│   ├── schema_manager.rs  # 模式管理
│   ├── storage_client.rs  # 存储客户端
│   ├── index_manager.rs   # 索引管理
│   ├── meta_client.rs     # 元数据客户端
│   ├── transaction.rs     # 事务管理
│   ├── retry.rs           # 重试机制
│   └── impl/              # 实现
│       ├── schema_manager_impl.rs
│       ├── storage_client_impl.rs
│       ├── index_manager_impl.rs
│       └── meta_client_impl.rs
├── symbol/                # 符号表
│   ├── symbol_table.rs
│   └── mod.rs
├── validate/              # 验证上下文
│   ├── context.rs
│   ├── basic_context.rs
│   ├── schema.rs
│   ├── generators.rs
│   ├── types.rs
│   └── mod.rs
├── core_query_context.rs  # 核心查询上下文
├── components.rs          # 组件访问器
├── request_context.rs     # 请求上下文
├── runtime_context.rs     # 运行时上下文
└── mod.rs
```

**核心类型**:

- `QueryAstContext`: 存储解析后的 AST 信息
- `CoreQueryContext`: 核心查询上下文，整合各个子上下文
- `ExecutionContext`: 执行上下文，包含执行所需的状态信息
- `RequestContext`: 请求上下文，管理单个请求的生命周期
- `RuntimeContext`: 运行时上下文，管理执行时的运行时状态
- `SymbolTable`: 符号表，管理变量别名和类型信息

**管理器接口**:

- `SchemaManager`: 提供 schema 信息查询
- `StorageClient`: 提供数据存取接口
- `IndexManager`: 提供索引管理接口
- `MetaClient`: 提供元数据管理接口

**与上下游关系**: Context 模块被 Parser、Validator、Planner、Optimizer、Executor 所有阶段共同使用，是数据流转的核心载体。

### 3. Validator 模块

**位置**: `src/query/validator/`

**职责**: 验证 AST 的语义正确性，包括类型检查、变量引用检查、权限检查等。采用策略模式和工厂模式实现。

**子模块结构**:

```
validator/
├── base_validator.rs       # 基础验证器接口
├── match_validator.rs      # MATCH 语句验证
├── go_validator.rs         # GO 语句验证
├── fetch_vertices_validator.rs
├── fetch_edges_validator.rs
├── pipe_validator.rs       # Pipe 验证
├── yield_validator.rs      # YIELD 验证
├── order_by_validator.rs   # ORDER BY 验证
├── limit_validator.rs      # LIMIT 验证
├── use_validator.rs        # USE 验证
├── unwind_validator.rs     # UNWIND 验证
├── lookup_validator.rs     # LOOKUP 验证
├── find_path_validator.rs  # FIND PATH 验证
├── get_subgraph_validator.rs
├── set_validator.rs        # SET 验证
├── sequential_validator.rs # 顺序验证
├── validation_factory.rs   # 验证器工厂
├── validation_interface.rs # 验证接口
└── strategies/             # 验证策略
    ├── expression_strategy.rs
    ├── alias_strategy.rs
    ├── type_inference.rs
    ├── expression_operations.rs
    ├── variable_validator.rs
    └── ...
```

**核心类型**:

- `Validator`: 统一验证器入口
- `ValidationFactory`: 验证器工厂，负责创建具体的验证器
- `ValidatorRegistry`: 验证器注册表，支持动态注册
- `ValidationContext`: 验证上下文
- 各具体验证器（如 `MatchValidator`、`GoValidator`）

**验证策略**: 采用策略模式将验证逻辑分解为独立的策略类，包括表达式策略、别名策略、类型推断策略等。

**与上下游关系**: Validator 接收 Parser 输出的 AST 上下文，验证通过后将验证后的上下文传递给 Planner。

### 4. Planner 模块

**位置**: `src/query/planner/`

**职责**: 将验证后的 AST 转换为执行计划（Execution Plan）。采用注册表模式管理多种规划器。

**子模块结构**:

```
planner/
├── plan/                   # 计划结构
│   ├── algorithms/         # 算法实现
│   │   ├── index_scan.rs
│   │   └── path_algorithms.rs
│   ├── core/              # 核心类型
│   │   ├── nodes/         # 计划节点定义
│   │   │   ├── start_node.rs
│   │   │   ├── project_node.rs
│   │   │   ├── filter_node.rs
│   │   │   ├── aggregate_node.rs
│   │   │   ├── join_node.rs
│   │   │   ├── traversal_node.rs
│   │   │   └── ...
│   │   ├── common.rs
│   │   └── explain.rs
│   ├── management/        # 管理操作计划
│   │   ├── admin/         # 管理操作
│   │   ├── ddl/           # DDL 操作
│   │   ├── dml/           # DML 操作
│   │   └── security/      # 安全操作
│   ├── execution_plan.rs
│   └── common.rs
├── statements/             # 语句规划器
│   ├── core/              # 核心规划器
│   │   ├── match_clause_planner.rs
│   │   └── cypher_clause_planner.rs
│   ├── clauses/           # 子句规划器
│   │   ├── clause_planner.rs
│   │   ├── return_clause_planner.rs
│   │   ├── where_clause_planner.rs
│   │   ├── order_by_clause_planner.rs
│   │   ├── projection_planner.rs
│   │   ├── pagination_planner.rs
│   │   ├── unwind_planner.rs
│   │   ├── with_clause_planner.rs
│   │   └── yield_planner.rs
│   ├── seeks/             # 查找策略
│   │   ├── seek_strategy.rs
│   │   ├── seek_strategy_base.rs
│   │   ├── index_seek.rs
│   │   ├── vertex_seek.rs
│   │   ├── scan_seek.rs
│   │   └── mod.rs
│   ├── paths/             # 路径规划
│   │   ├── match_path_planner.rs
│   │   └── shortest_path_planner.rs
│   ├── match_planner.rs
│   ├── go_planner.rs
│   ├── lookup_planner.rs
│   ├── fetch_vertices_planner.rs
│   ├── fetch_edges_planner.rs
│   ├── maintain_planner.rs
│   ├── subgraph_planner.rs
│   └── path_planner.rs
├── planner.rs              # 主规划器
├── connector.rs            # 连接器
└── mod.rs
```

**核心类型**:

- `Planner`: 规划器 trait
- `PlannerRegistry`: 规划器注册表
- `SequentialPlanner`: 顺序规划器
- `ExecutionPlan`: 执行计划
- `SubPlan`: 子计划
- `SentenceKind`: 语句类型枚举（MATCH、GO、LOOKUP 等）

**规划器注册机制**: 采用类型安全的枚举替代字符串匹配，支持优先级排序和动态注册。

**计划节点类型**:

- `StartNode`: 起始节点
- `GetVerticesNode`: 获取顶点
- `GetNeighborsNode`: 获取邻居
- `GetEdgesNode`: 获取边
- `ProjectNode`: 投影节点
- `FilterNode`: 过滤节点
- `AggregateNode`: 聚合节点
- `JoinNode`: 连接节点
- `SortNode`: 排序节点
- `LimitNode`: 限制节点

**与上下游关系**: Planner 接收 Validator 输出的上下文，生成执行计划后传递给 Optimizer。

### 5. Optimizer 模块

**位置**: `src/query/optimizer/`

**职责**: 对执行计划进行优化，包括谓词下推、投影下推、连接优化、索引优化等。采用基于规则的优化（RBO）框架。

**子模块结构**:

```
optimizer/
├── core/                   # 核心类型
│   ├── config.rs          # 优化配置
│   ├── cost.rs            # 成本模型
│   ├── phase.rs           # 优化阶段
│   └── mod.rs
├── engine/                 # 优化引擎
│   ├── exploration.rs     # 探索引擎
│   ├── optimizer.rs       # 主优化器
│   └── mod.rs
├── plan/                   # 优化计划结构
│   ├── context.rs         # 优化上下文
│   ├── group.rs           # 计划组
│   ├── node.rs            # 优化节点
│   └── mod.rs
├── rule_patterns.rs        # 规则模式
├── rule_traits.rs          # 规则 trait 定义
├── elimination_rules.rs    # 消除规则
├── index_optimization.rs   # 索引优化
├── join_optimization.rs    # 连接优化
├── limit_pushdown.rs       # Limit 下推
├── operation_merge.rs      # 操作合并
├── predicate_pushdown.rs   # 谓词下推
├── projection_pushdown.rs  # 投影下推
├── property_tracker.rs     # 属性追踪
├── prune_properties_visitor.rs
├── scan_optimization.rs    # 扫描优化
├── transformation_rules.rs # 转换规则
└── mod.rs
```

**核心类型**:

- `Optimizer`: 主优化器
- `RuleSet`: 规则集
- `OptContext`: 优化上下文
- `OptGroup`: 优化计划组
- `OptGroupNode`: 优化计划节点
- `OptRule`: 优化规则 trait

**优化阶段**:

1. **LogicalOptimization**（逻辑优化）: 谓词下推、投影下推、消除冗余操作
2. **PhysicalOptimization**（物理优化）: 连接优化、Limit 下推、索引选择
3. **PostOptimization**（后优化）: 最终优化和验证

**优化规则分类**:

- **消除规则**（Elimination Rules）: 移除不必要的节点
- **下推规则**（Push Down Rules）: 将操作下推到数据源附近
- **合并规则**（Merge Rules）: 合并相邻的同类操作
- **转换规则**（Transformation Rules）: 转换为更高效的操作

**常用优化规则**:

- `FilterPushDownRule`: 过滤条件下推
- `ProjectionPushDownRule`: 投影列下推
- `JoinOptimizationRule`: 连接优化
- `PushLimitDownRule`: Limit 下推
- `IndexScanRule`: 索引扫描优化
- `CombineFilterRule`: 过滤条件合并

**与上下游关系**: Optimizer 接收 Planner 输出的执行计划，经过多阶段优化后返回优化后的计划给 Executor。

### 6. Executor 模块

**位置**: `src/query/executor/`

**职责**: 执行优化后的查询计划，产生最终结果。采用工厂模式创建具体的执行器。

**子模块结构**:

```
executor/
├── base/                   # 基础类型
│   ├── execution_context.rs
│   ├── execution_result.rs
│   ├── executor_base.rs
│   ├── executor_stats.rs
│   └── mod.rs
├── admin/                  # 管理操作执行器
│   ├── space/             # 空间操作
│   │   ├── create_space.rs
│   │   ├── drop_space.rs
│   │   ├── desc_space.rs
│   │   └── show_spaces.rs
│   ├── tag/               # 标签操作
│   │   ├── create_tag.rs
│   │   ├── alter_tag.rs
│   │   ├── desc_tag.rs
│   │   └── drop_tag.rs
│   ├── edge/              # 边类型操作
│   │   ├── create_edge.rs
│   │   ├── alter_edge.rs
│   │   ├── desc_edge.rs
│   │   └── drop_edge.rs
│   ├── index/             # 索引操作
│   │   ├── tag_index.rs
│   │   ├── edge_index.rs
│   │   └── rebuild_index.rs
│   └── mod.rs
├── data_access/           # 数据访问执行器
│   ├── get_vertices.rs
│   ├── get_neighbors.rs
│   ├── get_edges.rs
│   ├── get_prop.rs
│   ├── index_scan.rs
│   └── all_paths.rs
├── data_processing/       # 数据处理执行器
│   ├── graph_traversal/   # 图遍历
│   │   ├── expand.rs
│   │   ├── expand_all.rs
│   │   ├── shortest_path.rs
│   │   └── traverse.rs
│   ├── join/              # 连接操作
│   │   ├── inner_join.rs
│   │   ├── left_join.rs
│   │   ├── right_join.rs
│   │   ├── full_outer_join.rs
│   │   ├── cross_join.rs
│   │   ├── hash_table.rs
│   │   └── parallel.rs
│   ├── set_operations/    # 集合操作
│   │   ├── intersect.rs
│   │   ├── minus.rs
│   │   ├── union.rs
│   │   └── union_all.rs
│   └── mod.rs
├── result_processing/     # 结果处理执行器
│   ├── projection.rs      # 投影
│   ├── filter.rs          # 过滤
│   ├── aggregation.rs     # 聚合
│   ├── dedup.rs           # 去重
│   ├── sort.rs            # 排序
│   ├── limit.rs           # 限制
│   ├── sample.rs          # 采样
│   ├── topn.rs            # TopN
│   ├── transformations/   # 转换操作
│   │   ├── append_vertices.rs
│   │   ├── assign.rs
│   │   ├── pattern_apply.rs
│   │   ├── rollup_apply.rs
│   │   └── unwind.rs
│   └── traits.rs
├── logic/                 # 逻辑控制执行器
│   ├── loops.rs
│   └── mod.rs
├── factory.rs             # 执行器工厂
├── graph_query_executor.rs # 图查询执行器
├── data_modification.rs   # 数据修改执行器
├── traits.rs              # 执行器 trait
├── object_pool.rs         # 对象池
└── recursion_detector.rs  # 递归检测
```

**核心类型**:

- `Executor`: 执行器 trait，定义执行器接口
- `ExecutionContext`: 执行上下文
- `ExecutionResult`: 执行结果
- `ExecutorFactory`: 执行器工厂
- `BaseExecutor`: 基础执行器

**执行器分类**:

1. **数据访问执行器**（Data Access）: 直接与存储层交互
   - `GetVerticesExecutor`: 获取顶点
   - `GetNeighborsExecutor`: 获取邻居
   - `GetEdgesExecutor`: 获取边
   - `IndexScanExecutor`: 索引扫描

2. **数据处理执行器**（Data Processing）: 处理中间结果
   - `JoinExecutor`: 连接操作
   - `AggregateExecutor`: 聚合操作
   - `SetOperationExecutor`: 集合操作

3. **结果处理执行器**（Result Processing）: 处理最终结果
   - `ProjectExecutor`: 投影
   - `FilterExecutor`: 过滤
   - `SortExecutor`: 排序
   - `LimitExecutor`: 限制

4. **管理执行器**（Admin）: 执行 DDL 和管理操作
   - `CreateSpaceExecutor`
   - `CreateTagExecutor`
   - `RebuildIndexExecutor`

**执行器 trait 设计**:

```rust
pub trait Executor<S: StorageEngine>: Send {
    async fn execute(&mut self) -> DBResult<ExecutionResult>;
    fn open(&mut self) -> DBResult<()>;
    fn close(&mut self) -> DBResult<()>;
    // ...
}
```

**与上下游关系**: Executor 是查询处理的最后一环，接收优化后的执行计划，访问存储层获取数据，生成最终结果。

### 7. Scheduler 模块

**位置**: `src/query/scheduler/`

**职责**: 管理查询的执行调度，支持异步执行和并行处理。

**子模块结构**:

```
scheduler/
├── async_scheduler.rs     # 异步调度器
├── execution_schedule.rs   # 执行调度
├── types.rs               # 类型定义
└── mod.rs
```

**核心类型**:

- `QueryScheduler`: 查询调度器 trait
- `AsyncMsgNotifyBasedScheduler`: 基于消息通知的异步调度器
- `ExecutionSchedule`: 执行调度
- `SchedulerConfig`: 调度配置
- `ExecutorType`: 执行器类型

**功能特点**:

- 支持异步查询执行
- 支持并行查询调度
- 管理执行器生命周期
- 处理执行事件和通知

### 8. Visitor 模块

**位置**: `src/query/visitor/`

**职责**: 提供 AST 遍历和分析的访问者实现，用于静态分析和代码转换。

**子模块结构**:

```
visitor/
├── deduce_type_visitor.rs       # 类型推导
├── deduce_props_visitor.rs      # 属性推导
├── deduce_alias_type_visitor.rs # 别名类型推导
├── evaluable_expr_visitor.rs    # 可求值性检查
├── extract_filter_expr_visitor.rs # 过滤表达式提取
├── extract_prop_expr_visitor.rs  # 属性表达式提取
├── extract_group_suite_visitor.rs # 分组提取
├── find_visitor.rs              # 查找访问者
├── variable_visitor.rs          # 变量访问者
├── vid_extract_visitor.rs       # VID 提取
├── fold_constant_expr_visitor.rs # 常量折叠
├── property_tracker_visitor.rs  # 属性追踪
├── rewrite_visitor.rs           # 重写访问者
└── mod.rs
```

**访问者类型**:

- `DeduceTypeVisitor`: 推导表达式的返回类型
- `DeducePropsVisitor`: 推导表达式所需的属性
- `EvaluableExprVisitor`: 检查表达式是否可求值
- `ExtractFilterExprVisitor`: 从 AST 中提取过滤条件
- `VidExtractVisitor`: 提取顶点 ID

**设计模式**: 访问者模式用于在不修改 AST 结构的情况下，对 AST 进行各种分析和转换操作。

## 模块间依赖关系

### 依赖图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Query Pipeline                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Parser ──┐                                                             │
│           │                                                             │
│  Validator┼──► Context ──┐                                              │
│                   │      │                                              │
│  Planner ──┴──────┼──────┤                                              │
│                   │      │                                              │
│  Optimizer ───────┼──────┤                                              │
│                   │      │                                              │
│  Executor ────────┴──────┘                                              │
│                                                                         │
│  Scheduler ──► (Executor, Context)                                      │
│                                                                         │
│  Visitor ────► (Parser AST, Context)                                    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 依赖说明

**Parser 依赖**:
- `context`: `QueryAstContext` 用于存储解析结果
- `core`: 基础类型定义

**Validator 依赖**:
- `context`: 验证上下文和符号表
- `parser`: AST 结构定义
- `visitor`: 类型推导等访问者

**Planner 依赖**:
- `context`: AST 上下文和符号表
- `validator`: 验证后的上下文信息

**Optimizer 依赖**:
- `context`: 查询上下文
- `planner`: 计划节点定义
- `visitor`: 属性推导访问者

**Executor 依赖**:
- `context`: 执行上下文和存储客户端
- `optimizer`: 优化后的计划
- `storage`: 存储引擎接口

**Scheduler 依赖**:
- `executor`: 执行器实例
- `context`: 执行上下文

**Visitor 依赖**:
- `parser`: AST 结构定义
- `context`: 上下文信息

## 数据流转

### 查询处理流程

```
1. Parser 阶段
   输入: 查询文本
   处理: 词法分析 → 语法分析 → AST 构建
   输出: QueryAstContext (包含 Stmt)

2. Validator 阶段
   输入: QueryAstContext
   处理: 类型检查 → 变量解析 → 权限验证
   输出: 验证后的 QueryAstContext

3. Planner 阶段
   输入: 验证后的 QueryAstContext
   处理: AST → 计划节点树 → 执行计划
   输出: ExecutionPlan

4. Optimizer 阶段
   输入: ExecutionPlan
   处理: 规则匹配 → 计划转换 → 成本估算
   输出: 优化后的 ExecutionPlan

5. Executor 阶段
   输入: 优化后的 ExecutionPlan
   处理: 计划执行 → 数据获取 → 结果处理
   输出: ExecutionResult
```

### 上下文传递

```
RequestContext
    │
    ├── QueryAstContext (Parser 输出)
    │       │
    │       └── Stmt (语句 AST)
    │
    ├── CoreQueryContext
    │       ├── SymbolTable (符号表)
    │       ├── ValidationContext (验证上下文)
    │       ├── ExecutionContext (执行上下文)
    │       └── Managers (各管理器)
    │               ├── SchemaManager
    │               ├── StorageClient
    │               ├── IndexManager
    │               └── MetaClient
    │
    └── RuntimeContext (运行时状态)
```

## 关键接口设计

### Planner Trait

```rust
pub trait Planner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError>;
    fn match_planner(&self, ast_ctx: &AstContext) -> bool;
}
```

### Optimizer Trait

```rust
pub trait OptRule: std::fmt::Debug {
    fn apply(&self, ctx: &mut OptContext, group_node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError>;
    fn pattern(&self) -> Pattern;
}
```

### Executor Trait

```rust
#[async_trait]
pub trait Executor<S: StorageEngine>: Send {
    async fn execute(&mut self) -> DBResult<ExecutionResult>;
    fn open(&mut self) -> DBResult<()>;
    fn close(&mut self) -> DBResult<()>;
    // ...
}
```

## 设计模式应用

### 1. 管道模式（Pipeline）
查询处理采用管道模式，每个阶段独立处理，通过上下文传递数据。

### 2. 工厂模式（Factory）
- `ExecutorFactory`: 创建具体的执行器
- `ValidationFactory`: 创建具体的验证器

### 3. 注册表模式（Registry）
- `PlannerRegistry`: 管理规划器注册
- `ValidatorRegistry`: 管理验证器注册

### 4. 策略模式（Strategy）
- 验证策略
- 优化规则

### 5. 访问者模式（Visitor）
- AST 分析和转换

### 6. 享元模式（Flyweight）
- `ObjectPool`: 对象池管理

## 架构特点

### 优势

1. **职责分离**: 每个模块有明确的职责边界
2. **可扩展性**: 支持动态注册新的规划器和验证器
3. **可测试性**: 每个阶段可以独立测试
4. **灵活性**: 支持多种查询语言和执行策略

### 挑战

1. **上下文传递复杂**: 需要在多个阶段之间传递上下文信息
2. **性能开销**: 多阶段处理带来额外的开销
3. **模块耦合**: 部分模块之间存在循环依赖

## 总结

GraphDB 查询引擎采用经典的五阶段管道架构，通过清晰的职责划分和模块化设计，实现了高效、可扩展的查询处理能力。Parser 负责解析、Validator 负责验证、Planner 负责规划、Optimizer 负责优化、Executor 负责执行，各阶段通过 Context 模块传递数据。Visitor 模块提供 AST 分析能力，Scheduler 模块负责执行调度，构成了完整的查询处理框架。

这种架构设计借鉴了 NebulaGraph 的成熟经验，同时针对单节点场景进行了简化和优化，在保证功能完整性的同时，提升了系统的可维护性和性能。
