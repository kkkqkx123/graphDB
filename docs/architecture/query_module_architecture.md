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
  │ 上下文  │        │ 验证信息 │        │  树    │        │ 优化树  │        │  数据  │
  └────────┘        └────────┘        └────────┘        └────────┘        └────────┘
```

**QueryPipelineManager** 是整个管道的协调者，负责管理查询的全生命周期。该类负责：
- 管理查询处理的全生命周期
- 协调各个处理阶段（解析→验证→规划→优化→执行）
- 处理错误和异常
- 管理查询上下文和性能监控

通过调用 `execute_query` 方法，依次经过解析、验证、规划、优化、执行五个阶段，最终返回查询结果。

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
│   └── mod.rs     # 模块导出
├── lexer/         # 词法分析器
│   ├── error.rs   # 词法错误
│   ├── lexer.rs   # 主词法分析器
│   └── mod.rs     # 模块导出
├── ast/           # 抽象语法树
│   ├── mod.rs     # 模块导出
│   ├── pattern.rs # 模式定义
│   ├── stmt.rs    # 语句定义
│   ├── types.rs   # 类型定义
│   └── utils.rs   # 工具函数
└── parser/        # 语法分析器
    ├── clause_parser.rs    # 子句解析
    ├── ddl_parser.rs       # DDL 解析
    ├── dml_parser.rs       # DML 解析
    ├── expr_parser.rs      # 表达式解析
    ├── mod.rs              # 模块导出
    ├── parse_context.rs    # 解析上下文
    ├── parser.rs           # 主解析器
    ├── stmt_parser.rs      # 语句解析
    ├── tests.rs            # 测试
    ├── traversal_parser.rs # 遍历解析
    ├── user_parser.rs      # 用户相关解析
    └── util_stmt_parser.rs # 工具语句解析
```

**核心类型**:

- `Parser`: 主解析器入口，提供 `new()` 和 `parse()` 方法
- `ExprParser`: 表达式解析器，处理 WHERE、YIELD 等子句中的表达式
- `StmtParser`: 语句解析器，处理 MATCH、GO、CREATE 等语句
- `Token` / `TokenKind`: 词法单元定义
- `ParseError`: 解析错误类型

**输出**: `ParserResult`，包含解析后的语句（Stmt）和表达式上下文

**与上下游关系**: Parser 的输出（AST）作为 Validator 的输入。

### 2. Core 模块

**位置**: `src/query/core/`

**职责**: 定义查询执行过程中的核心类型和状态管理。

**子模块结构**:

```
core/
├── execution_state.rs  # 执行状态管理
├── mod.rs             # 模块导出
└── node_type.rs       # 节点类型定义
```

**核心类型**:

- `QueryExecutionState`: 查询执行状态
- `ExecutorState`: 执行器状态
- `LoopExecutionState`: 循环执行状态
- `RowStatus`: 行状态

### 3. QueryContext 模块

**位置**: `src/query/query_context.rs`, `src/query/query_request_context.rs`, `src/query/context/`

**职责**: 管理查询处理的上下文信息，采用组合模式将不同职责分离到专门的上下文中。

**核心类型**:

- `QueryContext`: 查询上下文，整合所有查询相关的上下文信息
  - `QueryRequestContext`: 请求上下文，管理单个请求的生命周期
  - `QueryExecutionState`: 执行状态，管理执行计划和终止标志
  - `QueryResourceContext`: 资源上下文，管理对象池、ID生成器、符号表
  - `QuerySpaceContext`: 空间上下文，管理空间信息和字符集

**设计特点**:

- **组合模式**: 将 QueryContext 拆分为多个专门的上下文，每个上下文负责特定功能
- **职责分离**: 执行状态、资源管理、空间信息等职责明确分离
- **无 Clone**: 不实现 Clone，强制使用 Arc<QueryContext> 来共享所有权
- **Builder 模式**: 提供 QueryContextBuilder 来简化复杂对象的创建

**字段访问方法**:

- `space_id()`: 获取空间 ID
- `space_name()`: 获取空间名称
- `sym_table()`: 获取符号表
- `gen_id()`: 生成唯一 ID
- `obj_pool()`: 获取对象池
- `is_killed()`: 检查是否被终止
- `plan()`: 获取执行计划

**辅助方法**:

- `new_for_validation(query_text)`: 创建用于验证的临时上下文
- `new_for_planning(query_text)`: 创建用于规划的临时上下文
- `builder(rctx)`: 创建构建器

**API 层区分**:

- `api::core::QueryRequest`: API 层的查询请求结构，用于 API 接口
- `query::QueryContext`: Query 层的查询上下文，用于查询处理内部

两者职责不同，名称区分避免混淆。

### 4. QueryManager 模块

**位置**: `src/query/query_manager.rs`

**职责**: 负责跟踪和管理正在运行的查询，提供查询统计信息。

**核心类型**:

- `QueryManager`: 查询管理器，管理所有查询的生命周期
- `QueryInfo`: 查询信息，包含查询 ID、状态、执行时间等
- `QueryStats`: 查询统计信息，包含总查询数、运行中查询数等
- `QueryStatus`: 查询状态（Running、Finished、Failed、Killed）

### 5. QueryPipelineManager 模块

**位置**: `src/query/query_pipeline_manager.rs`

**职责**: 协调整个查询处理流程，管理查询的全生命周期。

**核心类型**:

- `QueryPipelineManager`: 查询管道管理器
- 通过引用使用 `OptimizerEngine`，而不是直接创建优化器组件

**与 OptimizerEngine 的关系**:

`QueryPipelineManager` 通过引用使用 `OptimizerEngine`，而不是直接创建优化器组件。`OptimizerEngine` 是全局实例，与数据库实例同生命周期，负责所有查询优化相关的功能。

```rust
// 创建方式
let optimizer_engine = Arc::new(OptimizerEngine::default());
let pipeline = QueryPipelineManager::with_optimizer(
    storage,
    stats_manager,
    optimizer_engine,
);
```

### 6. Validator 模块

**位置**: `src/query/validator/`

**职责**: 验证 AST 的语义正确性，包括类型检查、变量引用检查、权限检查等。采用 trait + 枚举模式管理验证器。

**子模块结构**:

```
validator/
├── clauses/           # 子句级验证器
│   ├── group_by_validator.rs
│   ├── limit_validator.rs
│   ├── mod.rs
│   ├── order_by_validator.rs
│   ├── return_validator.rs
│   ├── sequential_validator.rs
│   ├── with_validator.rs
│   └── yield_validator.rs
├── ddl/               # DDL 验证器
│   ├── admin_validator.rs
│   ├── alter_validator.rs
│   ├── drop_validator.rs
│   └── mod.rs
├── dml/               # DML 验证器
│   ├── mod.rs
│   ├── pipe_validator.rs
│   ├── query_validator.rs
│   ├── set_operation_validator.rs
│   └── use_validator.rs
├── helpers/           # 辅助工具
│   ├── expression_checker.rs
│   ├── mod.rs
│   ├── schema_validator.rs
│   ├── type_checker.rs
│   └── variable_checker.rs
├── statements/        # 语句级验证器
│   ├── create_validator.rs
│   ├── delete_validator.rs
│   ├── fetch_edges_validator.rs
│   ├── fetch_vertices_validator.rs
│   ├── find_path_validator.rs
│   ├── get_subgraph_validator.rs
│   ├── go_validator.rs
│   ├── insert_edges_validator.rs
│   ├── insert_vertices_validator.rs
│   ├── lookup_validator.rs
│   ├── match_validator.rs
│   ├── merge_validator.rs
│   ├── mod.rs
│   ├── remove_validator.rs
│   ├── set_validator.rs
│   ├── unwind_validator.rs
│   └── update_validator.rs
├── strategies/        # 验证策略
│   ├── helpers/       # 策略辅助工具
│   │   ├── expression_checker.rs
│   │   ├── mod.rs
│   │   ├── type_checker.rs
│   │   └── variable_checker.rs
│   ├── metadata/      # 元数据
│   │   ├── aggregate_functions.rs
│   │   └── mod.rs
│   ├── agg_functions.rs
│   ├── aggregate_strategy.rs
│   ├── alias_strategy.rs
│   ├── clause_strategy.rs
│   ├── expression_operations.rs
│   ├── expression_strategy.rs
│   ├── expression_strategy_test.rs
│   ├── mod.rs
│   └── pagination_strategy.rs
├── structs/           # 数据结构
│   ├── alias_structs.rs
│   ├── clause_structs.rs
│   ├── common_structs.rs
│   ├── mod.rs
│   ├── path_structs.rs
│   └── validation_info.rs
├── utility/           # 工具验证器
│   ├── acl_validator.rs
│   ├── explain_validator.rs
│   ├── mod.rs
│   └── update_config_validator.rs
├── assignment_validator.rs
├── expression_analyzer.rs
├── mod.rs
├── validator_enum.rs   # 验证器枚举
└── validator_trait.rs  # 验证器 trait
```

**核心类型**:

- `Validator`: 统一验证器入口（枚举）
- `StatementValidator`: 语句验证器 trait
- `ValidationResult`: 验证结果
- `ValidationInfo`: 验证信息
- `ValidatedStatement`: 验证后的语句

**验证策略**: 采用策略模式将验证逻辑分解为独立的策略类，包括表达式策略、别名策略、聚合策略等。

**与上下游关系**: Validator 接收 Parser 输出的 AST 上下文，验证通过后将验证后的上下文传递给 Planner。

### 7. Planner 模块

**位置**: `src/query/planner/`

**职责**: 将验证后的 AST 转换为执行计划（Execution Plan）。采用静态注册模式管理多种规划器。

**子模块结构**:

```
planner/
├── plan/                   # 计划结构
│   ├── algorithms/         # 算法实现
│   │   ├── index_scan.rs
│   │   ├── mod.rs
│   │   └── path_algorithms.rs
│   ├── core/              # 核心类型
│   │   ├── nodes/         # 计划节点定义
│   │   │   ├── aggregate_node.rs
│   │   │   ├── control_flow_node.rs
│   │   │   ├── data_processing_node.rs
│   │   │   ├── edge_nodes.rs
│   │   │   ├── factory.rs
│   │   │   ├── filter_node.rs
│   │   │   ├── graph_scan_node.rs
│   │   │   ├── index_nodes.rs
│   │   │   ├── insert_nodes.rs
│   │   │   ├── join_node.rs
│   │   │   ├── macros.rs
│   │   │   ├── mod.rs
│   │   │   ├── plan_node_category.rs
│   │   │   ├── plan_node_children.rs
│   │   │   ├── plan_node_enum.rs
│   │   │   ├── plan_node_operations.rs
│   │   │   ├── plan_node_traits.rs
│   │   │   ├── plan_node_traits_impl.rs
│   │   │   ├── plan_node_visitor.rs
│   │   │   ├── project_node.rs
│   │   │   ├── sample_node.rs
│   │   │   ├── set_operations_node.rs
│   │   │   ├── sort_node.rs
│   │   │   ├── space_nodes.rs
│   │   │   ├── start_node.rs
│   │   │   ├── tag_nodes.rs
│   │   │   ├── traversal_node.rs
│   │   │   └── user_nodes.rs
│   │   ├── common.rs
│   │   ├── explain.rs
│   │   ├── mod.rs
│   │   └── node_id_generator.rs
│   ├── execution_plan.rs
│   └── mod.rs
├── statements/             # 语句规划器
│   ├── clauses/           # 子句规划器
│   │   ├── mod.rs
│   │   ├── order_by_planner.rs
│   │   ├── pagination_planner.rs
│   │   ├── return_clause_planner.rs
│   │   ├── unwind_planner.rs
│   │   ├── where_clause_planner.rs
│   │   ├── with_clause_planner.rs
│   │   └── yield_planner.rs
│   ├── paths/             # 路径规划
│   │   ├── match_path_planner.rs
│   │   ├── mod.rs
│   │   └── shortest_path_planner.rs
│   ├── seeks/             # 查找策略
│   │   ├── edge_seek.rs
│   │   ├── index_seek.rs
│   │   ├── mod.rs
│   │   ├── prop_index_seek.rs
│   │   ├── scan_seek.rs
│   │   ├── seek_strategy.rs
│   │   ├── seek_strategy_base.rs
│   │   ├── variable_prop_index_seek.rs
│   │   └── vertex_seek.rs
│   ├── create_planner.rs
│   ├── delete_planner.rs
│   ├── fetch_edges_planner.rs
│   ├── fetch_vertices_planner.rs
│   ├── go_planner.rs
│   ├── group_by_planner.rs
│   ├── insert_planner.rs
│   ├── lookup_planner.rs
│   ├── maintain_planner.rs
│   ├── match_statement_planner.rs
│   ├── mod.rs
│   ├── path_planner.rs
│   ├── set_operation_planner.rs
│   ├── statement_planner.rs
│   ├── subgraph_planner.rs
│   ├── update_planner.rs
│   ├── use_planner.rs
│   └── user_management_planner.rs
├── rewrite/               # 计划重写（启发式优化）
│   ├── aggregate/         # 聚合优化
│   │   ├── mod.rs
│   │   └── push_filter_down_aggregate.rs
│   ├── elimination/       # 消除规则
│   │   ├── dedup_elimination.rs
│   │   ├── eliminate_append_vertices.rs
│   │   ├── eliminate_empty_set_operation.rs
│   │   ├── eliminate_filter.rs
│   │   ├── eliminate_row_collect.rs
│   │   ├── eliminate_sort.rs
│   │   ├── mod.rs
│   │   ├── remove_append_vertices_below_join.rs
│   │   └── remove_noop_project.rs
│   ├── limit_pushdown/    # Limit 下推
│   │   ├── mod.rs
│   │   ├── push_limit_down_get_edges.rs
│   │   ├── push_limit_down_get_vertices.rs
│   │   ├── push_limit_down_index_scan.rs
│   │   ├── push_limit_down_scan_edges.rs
│   │   ├── push_limit_down_scan_vertices.rs
│   │   └── push_topn_down_index_scan.rs
│   ├── merge/             # 合并规则
│   │   ├── collapse_consecutive_project.rs
│   │   ├── collapse_project.rs
│   │   ├── combine_filter.rs
│   │   ├── merge_get_nbrs_and_dedup.rs
│   │   ├── merge_get_nbrs_and_project.rs
│   │   ├── merge_get_vertices_and_dedup.rs
│   │   ├── merge_get_vertices_and_project.rs
│   │   └── mod.rs
│   ├── predicate_pushdown/ # 谓词下推
│   │   ├── mod.rs
│   │   ├── push_efilter_down.rs
│   │   ├── push_filter_down_all_paths.rs
│   │   ├── push_filter_down_cross_join.rs
│   │   ├── push_filter_down_expand_all.rs
│   │   ├── push_filter_down_get_nbrs.rs
│   │   ├── push_filter_down_hash_inner_join.rs
│   │   ├── push_filter_down_hash_left_join.rs
│   │   ├── push_filter_down_inner_join.rs
│   │   ├── push_filter_down_node.rs
│   │   ├── push_filter_down_traverse.rs
│   │   └── push_vfilter_down_scan_vertices.rs
│   ├── projection_pushdown/ # 投影下推
│   │   ├── mod.rs
│   │   ├── projection_pushdown.rs
│   │   └── push_project_down.rs
│   ├── context.rs
│   ├── expression_utils.rs
│   ├── macros.rs
│   ├── mod.rs
│   ├── pattern.rs
│   ├── plan_rewriter.rs
│   ├── result.rs
│   ├── rewrite_rule.rs
│   ├── rule.rs
│   ├── rule_enum.rs
│   └── visitor.rs
├── connector.rs           # 连接器
├── mod.rs
├── planner.rs             # 主规划器
└── template_extractor.rs  # 模板提取器
```

**核心类型**:

- `Planner`: 规划器 trait
- `PlannerEnum`: 规划器枚举（静态分发）
- `ExecutionPlan`: 执行计划
- `SubPlan`: 子计划
- `SentenceKind`: 语句类型枚举（MATCH、GO、LOOKUP 等）
- `PlanNodeEnum`: 计划节点枚举
- `PlanRewriter`: 计划重写器

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
- `TraversalNode`: 遍历节点
- `IndexScanNode`: 索引扫描节点

**重写规则分类**:

- **消除规则**（Elimination）: 移除不必要的节点
- **下推规则**（Push Down）: 将操作下推到数据源附近
- **合并规则**（Merge）: 合并相邻的同类操作

**与上下游关系**: Planner 接收 Validator 输出的上下文，生成执行计划后传递给 Optimizer。

### 8. Optimizer 模块

**位置**: `src/query/optimizer/`

**职责**: 对执行计划进行优化，包括统计信息管理、代价计算、优化策略等。采用基于代价的优化（CBO）框架。

**子模块结构**:

```
optimizer/
├── analysis/              # 计划分析
│   ├── expression.rs      # 表达式分析
│   ├── fingerprint.rs     # 指纹生成
│   ├── mod.rs
│   └── reference_count.rs # 引用计数分析
├── cost/                  # 代价计算
│   ├── node_estimators/   # 节点代价估算器
│   │   ├── control_flow.rs
│   │   ├── data_processing.rs
│   │   ├── graph_algorithm.rs
│   │   ├── graph_traversal.rs
│   │   ├── join.rs
│   │   ├── mod.rs
│   │   ├── scan.rs
│   │   └── sort_limit.rs
│   ├── assigner.rs        # 代价分配器
│   ├── calculator.rs      # 代价计算器
│   ├── child_accessor.rs  # 子节点访问器
│   ├── config.rs          # 代价模型配置
│   ├── estimate.rs        # 代价估算
│   ├── expression_parser.rs # 表达式解析器
│   ├── mod.rs
│   └── selectivity.rs     # 选择性估算
├── decision/              # 优化决策
│   ├── cache.rs           # 决策缓存
│   ├── mod.rs
│   └── types.rs           # 决策类型
├── stats/                 # 统计信息
│   ├── collector.rs       # 统计信息收集器
│   ├── edge.rs            # 边类型统计
│   ├── manager.rs         # 统计信息管理器
│   ├── mod.rs
│   ├── property.rs        # 属性统计
│   └── tag.rs             # 标签统计
├── strategy/              # 优化策略
│   ├── aggregate_strategy.rs    # 聚合策略
│   ├── index.rs                 # 索引选择
│   ├── join_order.rs            # 连接顺序
│   ├── materialization.rs       # 物化策略
│   ├── mod.rs
│   ├── subquery_unnesting.rs    # 子查询展开
│   ├── topn_optimization.rs     # TopN 优化
│   ├── traversal_direction.rs   # 遍历方向
│   └── traversal_start.rs       # 遍历起点
├── engine.rs              # 优化器引擎
└── mod.rs
```

**核心类型**:

- `OptimizerEngine`: 优化器引擎（全局实例）
- `StatisticsManager`: 统计信息管理器
- `CostCalculator`: 代价计算器
- `CostAssigner`: 代价分配器
- `SelectivityEstimator`: 选择性估算器

**优化策略**:

- `TraversalStartSelector`: 遍历起点选择
- `IndexSelector`: 索引选择
- `JoinOrderOptimizer`: 连接顺序优化
- `AggregateStrategy`: 聚合策略
- `TraversalDirectionOptimizer`: 遍历方向优化
- `SubqueryUnnestingOptimizer`: 子查询展开优化
- `MaterializationOptimizer`: 物化优化
- `TopNOptimization`: TopN 优化

**优化决策**:

- `OptimizationDecision`: 优化决策
- `DecisionCache`: 决策缓存
- `TraversalStartDecision`: 遍历起点决策
- `IndexSelectionDecision`: 索引选择决策
- `JoinOrderDecision`: 连接顺序决策

**与上下游关系**: Optimizer 接收 Planner 输出的执行计划，经过优化后返回优化后的计划给 Executor。

### 9. Executor 模块

**位置**: `src/query/executor/`

**职责**: 执行优化后的查询计划，产生最终结果。采用工厂模式创建具体的执行器。

**子模块结构**:

```
executor/
├── admin/                  # 管理操作执行器
│   ├── edge/              # 边类型操作
│   │   ├── alter_edge.rs
│   │   ├── create_edge.rs
│   │   ├── desc_edge.rs
│   │   ├── drop_edge.rs
│   │   ├── mod.rs
│   │   ├── show_edges.rs
│   │   └── tests.rs
│   ├── index/             # 索引操作
│   │   ├── edge_index.rs
│   │   ├── mod.rs
│   │   ├── rebuild_index.rs
│   │   ├── show_edge_index_status.rs
│   │   ├── show_tag_index_status.rs
│   │   ├── tag_index.rs
│   │   └── tests.rs
│   ├── query_management/  # 查询管理
│   │   ├── mod.rs
│   │   └── show_stats.rs
│   ├── space/            # 空间操作
│   │   ├── alter_space.rs
│   │   ├── clear_space.rs
│   │   ├── create_space.rs
│   │   ├── desc_space.rs
│   │   ├── drop_space.rs
│   │   ├── mod.rs
│   │   ├── show_spaces.rs
│   │   ├── switch_space.rs
│   │   └── tests.rs
│   ├── tag/              # 标签操作
│   │   ├── alter_tag.rs
│   │   ├── create_tag.rs
│   │   ├── desc_tag.rs
│   │   ├── drop_tag.rs
│   │   ├── mod.rs
│   │   ├── show_tags.rs
│   │   └── tests.rs
│   ├── user/             # 用户操作
│   │   ├── alter_user.rs
│   │   ├── change_password.rs
│   │   ├── create_user.rs
│   │   ├── drop_user.rs
│   │   ├── grant_role.rs
│   │   ├── mod.rs
│   │   └── revoke_role.rs
│   ├── analyze.rs
│   └── mod.rs
├── base/                  # 基础执行器
│   ├── execution_context.rs
│   ├── execution_result.rs
│   ├── execution_stats.rs
│   ├── executor_base.rs
│   ├── executor_stats.rs
│   ├── mod.rs
│   └── result_processor.rs
├── data_processing/       # 数据处理执行器
│   ├── graph_traversal/  # 图遍历执行器
│   │   ├── algorithms/   # 算法实现
│   │   │   ├── a_star.rs
│   │   │   ├── bidirectional_bfs.rs
│   │   │   ├── dijkstra.rs
│   │   │   ├── mod.rs
│   │   │   ├── multi_shortest_path.rs
│   │   │   ├── subgraph_executor.rs
│   │   │   ├── traits.rs
│   │   │   └── types.rs
│   │   ├── all_paths.rs
│   │   ├── expand.rs
│   │   ├── expand_all.rs
│   │   ├── factory.rs
│   │   ├── impls.rs
│   │   ├── mod.rs
│   │   ├── shortest_path.rs
│   │   ├── tests.rs
│   │   ├── traits.rs
│   │   ├── traversal_utils.rs
│   │   └── traverse.rs
│   ├── join/             # 连接执行器
│   │   ├── base_join.rs
│   │   ├── cross_join.rs
│   │   ├── full_outer_join.rs
│   │   ├── hash_table.rs
│   │   ├── inner_join.rs
│   │   ├── join_key_evaluator.rs
│   │   ├── left_join.rs
│   │   └── mod.rs
│   ├── set_operations/   # 集合操作执行器
│   │   ├── base.rs
│   │   ├── intersect.rs
│   │   ├── minus.rs
│   │   ├── mod.rs
│   │   ├── union.rs
│   │   └── union_all.rs
│   └── mod.rs
├── expression/            # 表达式求值
│   ├── evaluation_context/ # 求值上下文
│   │   ├── cache_manager.rs
│   │   ├── default_context.rs
│   │   ├── mod.rs
│   │   └── row_context.rs
│   ├── evaluator/         # 表达式求值器
│   │   ├── collection_operations.rs
│   │   ├── expression_evaluator.rs
│   │   ├── functions.rs
│   │   ├── mod.rs
│   │   ├── operations.rs
│   │   └── traits.rs
│   ├── functions/         # 函数
│   │   ├── builtin/      # 内置函数
│   │   │   ├── aggregate.rs
│   │   │   ├── container.rs
│   │   │   ├── conversion.rs
│   │   │   ├── datetime.rs
│   │   │   ├── geography.rs
│   │   │   ├── graph.rs
│   │   │   ├── macros.rs
│   │   │   ├── math.rs
│   │   │   ├── mod.rs
│   │   │   ├── path.rs
│   │   │   ├── regex.rs
│   │   │   ├── string.rs
│   │   │   └── utility.rs
│   │   ├── mod.rs
│   │   ├── registry.rs
│   │   └── signature.rs
│   └── mod.rs
├── logic/                 # 循环控制执行器
│   ├── loops.rs
│   └── mod.rs
├── result_processing/      # 结果处理执行器
│   ├── transformations/   # 数据转换执行器
│   │   ├── append_vertices.rs
│   │   ├── assign.rs
│   │   ├── mod.rs
│   │   ├── pattern_apply.rs
│   │   ├── rollup_apply.rs
│   │   └── unwind.rs
│   ├── agg_data.rs
│   ├── agg_function_manager.rs
│   ├── aggregation.rs
│   ├── dedup.rs
│   ├── filter.rs
│   ├── limit.rs
│   ├── mod.rs
│   ├── projection.rs
│   ├── sample.rs
│   ├── sort.rs
│   └── topn.rs
├── aggregation.rs
├── aggregation_benchmark.rs
├── data_access.rs
├── data_modification.rs
├── executor_enum.rs
├── factory.rs
├── graph_query_executor.rs
├── mod.rs
├── object_pool.rs
├── recursion_detector.rs
├── search_executors.rs
├── special_executors.rs
└── tag_filter.rs
```

**核心类型**:

- `Executor`: 执行器 trait
- `ExecutorEnum`: 执行器枚举（静态分发）
- `ExecutorFactory`: 执行器工厂
- `ExecutionContext`: 执行上下文
- `ExecutionResult`: 执行结果
- `GraphQueryExecutor`: 图查询执行器

**执行器分类**:

1. **基础执行器**（base/）:
   - `BaseExecutor`: 基础执行器 trait
   - `StartExecutor`: 起始执行器
   - `InputExecutor`: 输入执行器
   - `ResultProcessor`: 结果处理器

2. **数据处理执行器**（data_processing/）:
   - **图遍历执行器**（graph_traversal/）:
     - `AllPathsExecutor`: 所有路径执行器
     - `ExpandExecutor`: 扩展执行器
     - `ExpandAllExecutor`: 全部扩展执行器
     - `ShortestPathExecutor`: 最短路径执行器
     - `TraverseExecutor`: 遍历执行器
   - **连接执行器**（join/）:
     - `InnerJoinExecutor`: 内连接执行器
     - `LeftJoinExecutor`: 左连接执行器
     - `CrossJoinExecutor`: 交叉连接执行器
     - `FullOuterJoinExecutor`: 全外连接执行器
   - **集合操作执行器**（set_operations/）:
     - `UnionExecutor`: 并集执行器
     - `UnionAllExecutor`: 并集所有执行器
     - `IntersectExecutor`: 交集执行器
     - `MinusExecutor`: 差集执行器

3. **表达式求值**（expression/）:
   - **求值上下文**（evaluation_context/）:
     - `DefaultContext`: 默认求值上下文
     - `RowContext`: 行上下文
     - `CacheManager`: 缓存管理器
   - **表达式求值器**（evaluator/）:
     - `ExpressionEvaluator`: 表达式求值器
   - **函数**（functions/）:
     - **内置函数**（builtin/）:
       - 聚合函数、容器函数、转换函数、日期时间函数、地理函数、图函数、数学函数、路径函数、正则函数、字符串函数、工具函数

4. **循环控制执行器**（logic/）:
   - `ForLoopExecutor`: For 循环执行器
   - `WhileLoopExecutor`: While 循环执行器
   - `LoopExecutor`: 循环执行器

5. **结果处理执行器**（result_processing/）:
   - **数据转换执行器**（transformations/）:
     - `AppendVerticesExecutor`: 追加顶点执行器
     - `AssignExecutor`: 赋值执行器
     - `PatternApplyExecutor`: 模式应用执行器
     - `RollUpApplyExecutor`: 汇总应用执行器
     - `UnwindExecutor`: 展开执行器
   - `AggregateExecutor`: 聚合执行器
   - `DedupExecutor`: 去重执行器
   - `FilterExecutor`: 过滤执行器
   - `LimitExecutor`: 限制执行器
   - `ProjectExecutor`: 投影执行器
   - `SampleExecutor`: 采样执行器
   - `SortExecutor`: 排序执行器
   - `TopNExecutor`: TopN 执行器

6. **管理执行器**（admin/）:
   - **边类型操作**（edge/）:
     - `CreateEdgeExecutor`: 创建边类型执行器
     - `AlterEdgeExecutor`: 修改边类型执行器
     - `DropEdgeExecutor`: 删除边类型执行器
     - `DescEdgeExecutor`: 描述边类型执行器
     - `ShowEdgesExecutor`: 显示边类型执行器
   - **索引操作**（index/）:
     - `CreateEdgeIndexExecutor`: 创建边索引执行器
     - `CreateTagIndexExecutor`: 创建标签索引执行器
     - `DropEdgeIndexExecutor`: 删除边索引执行器
     - `DropTagIndexExecutor`: 删除标签索引执行器
     - `RebuildEdgeIndexExecutor`: 重建边索引执行器
     - `RebuildTagIndexExecutor`: 重建标签索引执行器
     - `ShowEdgeIndexesExecutor`: 显示边索引执行器
     - `ShowTagIndexesExecutor`: 显示标签索引执行器
   - **空间操作**（space/）:
     - `CreateSpaceExecutor`: 创建空间执行器
     - `AlterSpaceExecutor`: 修改空间执行器
     - `DropSpaceExecutor`: 删除空间执行器
     - `DescSpaceExecutor`: 描述空间执行器
     - `ShowSpacesExecutor`: 显示空间执行器
     - `SwitchSpaceExecutor`: 切换空间执行器
     - `ClearSpaceExecutor`: 清空空间执行器
   - **标签操作**（tag/）:
     - `CreateTagExecutor`: 创建标签执行器
     - `AlterTagExecutor`: 修改标签执行器
     - `DropTagExecutor`: 删除标签执行器
     - `DescTagExecutor`: 描述标签执行器
     - `ShowTagsExecutor`: 显示标签执行器
   - **用户操作**（user/）:
     - `CreateUserExecutor`: 创建用户执行器
     - `AlterUserExecutor`: 修改用户执行器
     - `DropUserExecutor`: 删除用户执行器
     - `ChangePasswordExecutor`: 修改密码执行器
     - `GrantRoleExecutor`: 授予角色执行器
     - `RevokeRoleExecutor`: 撤销角色执行器

7. **搜索执行器**（search_executors.rs）:
   - `BFSShortestExecutor`: BFS 最短路径执行器

8. **特殊执行器**（special_executors.rs）:
   - `ArgumentExecutor`: 参数执行器
   - `DataCollectExecutor`: 数据收集执行器
   - `PassThroughExecutor`: 传递执行器

9. **其他执行器**:
   - `GetVerticesExecutor`: 获取顶点执行器
   - `GetNeighborsExecutor`: 获取邻居执行器
   - `GetEdgesExecutor`: 获取边执行器
   - `GetPropExecutor`: 获取属性执行器
   - `IndexScanExecutor`: 索引扫描执行器
   - `ScanVerticesExecutor`: 扫描顶点执行器

**与上下游关系**: Executor 接收 Optimizer 输出的优化后的执行计划，执行查询并返回结果。

## 模块间关系

### 数据流转

```
用户查询
   ↓
QueryPipelineManager (协调器)
   ↓
Parser → AST
   ↓
Validator → Validated AST
   ↓
Planner → Execution Plan
   ↓
Optimizer → Optimized Plan
   ↓
Executor → Result
   ↓
返回给用户
```

### 模块依赖关系

```
QueryPipelineManager
    ├─→ Parser
    ├─→ Validator
    ├─→ Planner
    ├─→ Optimizer (引用)
    └─→ Executor

Parser
    └─→ AST (输出)

Validator
    ├─→ AST (输入)
    └─→ Validated AST (输出)

Planner
    ├─→ Validated AST (输入)
    ├─→ Optimizer (用于优化决策)
    └─→ Execution Plan (输出)

Optimizer
    ├─→ Execution Plan (输入)
    ├─→ StatisticsManager (统计信息)
    ├─→ CostCalculator (代价计算)
    └─→ Optimized Plan (输出)

Executor
    ├─→ Optimized Plan (输入)
    ├─→ StorageClient (存储访问)
    └─→ ExecutionResult (输出)

QueryManager
    └─→ 查询生命周期管理
```

## 设计模式

### 1. 管道模式（Pipeline Pattern）

查询处理采用管道模式，将查询处理分解为多个独立的阶段，每个阶段负责特定的处理任务。

### 2. 工厂模式（Factory Pattern）

- `ExecutorFactory`: 创建具体的执行器实例
- `PlannerFactory`: 创建具体的规划器实例

### 3. 策略模式（Strategy Pattern）

- `Validator`: 不同的验证策略
- `Optimizer`: 不同的优化策略

### 4. 访问者模式（Visitor Pattern）

- `PlanNodeVisitor`: 访问计划节点
- `PlanRewriter`: 重写计划节点

### 5. 枚举模式（Enum Pattern）

- `ValidatorEnum`: 静态分发的验证器枚举
- `PlannerEnum`: 静态分发的规划器枚举
- `ExecutorEnum`: 静态分发的执行器枚举

## 性能优化

### 1. 静态分发

使用枚举代替 trait 对象，实现静态分发，避免动态分发的开销。

### 2. 编译期检查

使用常量断言确保 `PlanNodeEnum` 和 `ExecutorEnum` 的变体数量一致，在编译期捕获错误。

### 3. 对象池

使用对象池管理执行器实例，减少内存分配和回收的开销。

### 4. 缓存

- `DecisionCache`: 缓存优化决策
- `CacheManager`: 缓存表达式求值结果

### 5. 统计信息

使用统计信息指导优化决策，提高查询性能。

## 错误处理

所有模块都使用统一的错误类型 `DBError`，提供详细的错误信息和堆栈跟踪。

## 总结

GraphDB 查询引擎采用经典的管道架构，将查询处理分解为解析、验证、规划、优化和执行五个阶段。每个阶段都有明确的职责边界，通过精心设计的模块间接口进行协作。系统采用多种设计模式和性能优化技术，确保查询处理的高效性和可维护性。

主要特点：

1. **模块化设计**: 每个模块都有明确的职责，易于理解和维护
2. **静态分发**: 使用枚举代替 trait 对象，提高性能
3. **编译期检查**: 在编译期捕获错误，提高代码质量
4. **性能优化**: 使用对象池、缓存等技术提高性能
5. **可扩展性**: 易于添加新的语句类型、优化策略和执行器
