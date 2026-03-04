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

**位置**: `src/query/query_context.rs`, `src/query/query_request_context.rs`

**职责**: 管理查询处理的上下文信息，包括表达式上下文、验证信息、空间信息等。

**核心类型**:

- `QueryContext`: 查询上下文，整合表达式上下文、验证信息、空间信息
- `QueryRequestContext`: 请求上下文，管理单个请求的生命周期

### 4. Validator 模块

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
│   ├── aggregate_strategy.rs
│   ├── alias_strategy.rs
│   ├── clause_strategy.rs
│   ├── expression_operations.rs
│   ├── expression_strategy.rs
│   ├── expression_strategy_test.rs
│   ├── helpers/
│   ├── metadata/
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

### 5. Planner 模块

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
│   │   ├── clause_planner.rs
│   │   ├── limit_pushdown_planner.rs
│   │   ├── mod.rs
│   │   ├── order_by_planner.rs
│   │   ├── pagination_planner.rs
│   │   ├── projection_planner.rs
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

### 6. Optimizer 模块

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

### 7. Executor 模块

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
│   ├── space/             # 空间操作
│   │   ├── alter_space.rs
│   │   ├── clear_space.rs
│   │   ├── create_space.rs
│   │   ├── desc_space.rs
│   │   ├── drop_space.rs
│   │   ├── mod.rs
│   │   ├── show_spaces.rs
│   │   ├── switch_space.rs
│   │   └── tests.rs
│   ├── tag/               # 标签操作
│   │   ├── alter_tag.rs
│   │   ├── create_tag.rs
│   │   ├── desc_tag.rs
│   │   ├── drop_tag.rs
│   │   ├── mod.rs
│   │   ├── show_tags.rs
│   │   └── tests.rs
│   ├── user/              # 用户管理
│   │   ├── alter_user.rs
│   │   ├── change_password.rs
│   │   ├── create_user.rs
│   │   ├── drop_user.rs
│   │   ├── grant_role.rs
│   │   ├── mod.rs
│   │   └── revoke_role.rs
│   ├── analyze.rs
│   └── mod.rs
├── base/                   # 基础类型
│   ├── execution_context.rs
│   ├── execution_result.rs
│   ├── execution_stats.rs
│   ├── executor_base.rs
│   ├── executor_stats.rs
│   ├── mod.rs
│   └── result_processor.rs
├── data_access/           # 数据访问执行器
│   ├── get_vertices.rs
│   ├── get_neighbors.rs
│   ├── get_edges.rs
│   ├── get_prop.rs
│   ├── index_scan.rs
│   └── all_paths.rs
├── data_modification.rs   # 数据修改执行器
├── data_processing/       # 数据处理执行器
│   ├── graph_traversal/   # 图遍历
│   │   ├── algorithms/    # 图算法
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
│   │   └── traversal_utils.rs
│   ├── join/              # 连接操作
│   │   ├── base_join.rs
│   │   ├── cross_join.rs
│   │   ├── full_outer_join.rs
│   │   ├── hash_table.rs
│   │   ├── inner_join.rs
│   │   ├── join_key_evaluator.rs
│   │   ├── left_join.rs
│   │   └── mod.rs
│   ├── set_operations/    # 集合操作
│   │   ├── base.rs
│   │   ├── intersect.rs
│   │   ├── minus.rs
│   │   ├── mod.rs
│   │   ├── union.rs
│   │   └── union_all.rs
│   ├── README.md
│   └── mod.rs
├── expression/            # 表达式执行
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
│   ├── functions/         # 函数实现
│   │   ├── builtin/       # 内置函数
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
│   │   ├── registry.rs    # 函数注册表
│   │   └── signature.rs   # 函数签名
│   └── mod.rs
├── logic/                 # 逻辑控制执行器
│   ├── loops.rs
│   └── mod.rs
├── result_processing/     # 结果处理执行器
│   ├── transformations/   # 转换操作
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
├── executor_enum.rs       # 执行器枚举
├── factory.rs             # 执行器工厂
├── graph_query_executor.rs # 图查询执行器
├── mod.rs
├── object_pool.rs         # 对象池
├── recursion_detector.rs  # 递归检测
├── search_executors.rs    # 搜索执行器
├── special_executors.rs   # 特殊执行器
└── tag_filter.rs
```

**核心类型**:

- `Executor`: 执行器 trait，定义执行器接口
- `ExecutorEnum`: 执行器枚举（静态分发）
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
   - `SetOperationExecutor`: 集合操作
   - `GraphTraversalExecutor`: 图遍历
   - `ExpandExecutor`: 扩展操作

3. **结果处理执行器**（Result Processing）: 处理最终结果
   - `ProjectExecutor`: 投影
   - `FilterExecutor`: 过滤
   - `AggregateExecutor`: 聚合
   - `SortExecutor`: 排序
   - `LimitExecutor`: 限制
   - `DedupExecutor`: 去重

4. **管理执行器**（Admin）: 执行 DDL 和管理操作
   - `CreateSpaceExecutor`: 创建空间
   - `CreateTagExecutor`: 创建标签
   - `CreateEdgeExecutor`: 创建边类型
   - `RebuildIndexExecutor`: 重建索引

5. **表达式执行器**（Expression）: 表达式求值
   - `ExpressionEvaluator`: 表达式求值器
   - `FunctionRegistry`: 函数注册表

**执行器 trait 设计**:

```rust
pub trait Executor: Send {
    fn execute(&mut self) -> DBResult<ExecutionResult>;
    fn open(&mut self) -> DBResult<()>;
    fn close(&mut self) -> DBResult<()>;
    // ...
}
```

**与上下游关系**: Executor 是查询处理的最后一环，接收优化后的执行计划，访问存储层获取数据，生成最终结果。

### 8. QueryManager 模块

**位置**: `src/query/query_manager.rs`

**职责**: 管理查询的生命周期，包括查询的提交、状态跟踪、统计信息收集等。

**核心类型**:

- `QueryManager`: 查询管理器
- `QueryInfo`: 查询信息
- `QueryStats`: 查询统计
- `QueryStatus`: 查询状态

## 数据流转

### 查询处理流程

```
1. 用户输入查询文本
   ↓
2. Parser 解析为 AST
   - 词法分析：将文本转换为 Token 序列
   - 语法分析：将 Token 序列转换为 AST
   ↓
3. Validator 验证 AST
   - 类型检查
   - 变量引用检查
   - 权限检查
   ↓
4. Planner 生成执行计划
   - 将 AST 转换为计划节点树
   - 应用启发式重写规则
   ↓
5. Optimizer 优化执行计划
   - 收集统计信息
   - 计算代价
   - 选择最优执行策略
   ↓
6. Executor 执行计划
   - 访问存储层获取数据
   - 处理中间结果
   - 生成最终结果
   ↓
7. 返回结果给用户
```

### 关键数据结构流转

```
查询文本 (String)
    ↓
ParserResult { stmt, expr_context }
    ↓
ValidatedStatement { stmt, validation_info }
    ↓
ExecutionPlan { plan_node_tree }
    ↓
ExecutionPlan { optimized_plan_node_tree }
    ↓
ExecutionResult { data }
```

## 模块间关系

### 依赖关系图

```
query_pipeline_manager
    ├── parser
    ├── validator
    ├── planner
    ├── optimizer
    └── executor
        ├── core
        └── expression

query_context
    └── query_request_context

planner
    └── rewrite

optimizer
    ├── stats
    ├── cost
    ├── analysis
    ├── decision
    └── strategy
```

### 设计模式应用

1. **管道模式**（Pipeline）: 查询处理的五个阶段形成管道
2. **工厂模式**（Factory）: `ExecutorFactory` 创建执行器
3. **策略模式**（Strategy）: 验证策略、优化策略
4. **枚举静态分发**: `PlannerEnum`、`Validator`、`ExecutorEnum` 使用枚举替代动态分发
5. **访问者模式**（Visitor）: 计划节点访问器
6. **模板方法模式**: 基础执行器定义执行框架，具体执行器实现细节

## 性能考虑

1. **静态分发**: 使用枚举替代 `dyn trait`，避免动态分发的开销
2. **对象池**: `ObjectPool` 复用执行器对象，减少内存分配
3. **决策缓存**: `DecisionCache` 缓存优化决策，避免重复计算
4. **统计信息**: 基于统计信息的代价模型，选择最优执行计划
5. **编译期断言**: 确保 `PlanNodeEnum` 和 `ExecutorEnum` 变体数量一致

## 扩展性

1. **新语句支持**: 实现 `StatementValidator` trait 和对应的规划器
2. **新优化规则**: 在 `rewrite` 模块添加新的重写规则
3. **新执行器**: 在 `executor` 模块添加新的执行器类型
4. **新函数**: 在 `expression/functions/builtin` 添加新的内置函数
