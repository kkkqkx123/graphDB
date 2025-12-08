# NebulaGraph src/graph 模块到 Rust 架构迁移计划

## 概述

本文档详细分析了将 NebulaGraph 的 `src/graph` 模块迁移到新的 Rust 架构的计划。`src/graph` 是 NebulaGraph 图数据库的查询处理引擎核心，包含了从查询解析、验证、规划、优化到执行的完整流程。通过将该模块迁移到 Rust 架构，我们能够利用 Rust 的内存安全、并发安全和零成本抽象等优势，构建一个更轻量、高性能的单节点图数据库。

## 迁移必要性

### 1. 架构简化
- NebulaGraph 原本是为分布式场景设计的，包含复杂的分布式逻辑
- 新的 Rust 架构专注于单节点部署，可显著简化代码和降低外部依赖

### 2. 内存安全和并发安全
- C++ 实现存在内存泄漏和并发安全风险
- Rust 的所有权系统可从根本上解决这些问题

### 3. 性能优化
- Rust 提供零成本抽象，可优化执行性能
- 更好的内存管理减少垃圾回收开销

### 4. 单节点部署
- 为个人用户和小规模应用场景提供轻量级解决方案
- 生成单个可执行文件，便于部署和分发

## 迁移范围

根据 `graph_modules_analysis.md` 的分析，`src/graph` 模块包含以下核心组件，均需迁移到 Rust 架构：

1. `context/` - 执行上下文管理
2. `validator/` - 查询验证器
3. `planner/` - 查询计划生成器
4. `optimizer/` - 查询优化器
5. `executor/` - 执行引擎
6. `scheduler/` - 执行调度器
7. `service/` - 服务层
8. `session/` - 会话管理
9. `visitor/` - 表达式访问器
10. `util/` - 工具函数库
11. `stats/` - 统计信息
12. `gc/` - 垃圾回收

## 迁移顺序

### 第一阶段：核心基础设施迁移

#### 1. `util/` - 工具函数库 (高优先级)
- **原因**: 为其他模块提供基础工具函数
- **内容**: 
  - `ExpressionUtils`、`SchemaUtil`、`IndexUtil` 等
  - `IdGenerator`、`AnonVarGenerator`、`AnonColGenerator`
  - `AstUtils`、`Constants`、`Utils`
- **Rust 对应**: `src/utils/` 目录，包含对应的工具模块

#### 2. `context/` - 执行上下文管理 (高优先级)  
- **原因**: 为整个查询处理流程提供上下文管理
- **内容**:
  - `QueryContext`、`ExecutionContext`
  - `Iterator`、`Symbols`、`ValidateContext`、`QueryExpressionContext`
  - `Result`、AST 上下文等
- **Rust 对应**: `src/core/` 目录，包含上下文相关的数据结构定义

### 第二阶段：解析与验证层迁移

#### 3. `visitor/` - 表达式访问器 (高优先级)
- **原因**: 验证器和优化器依赖表达式分析
- **内容**:
  - `DeduceTypeVisitor`、`DeducePropsVisitor`、`ExtractFilterExprVisitor`
  - `FoldConstantExprVisitor`、`EvaluableExprVisitor`、`FindVisitor`
  - 各种表达式分析和转换访问器
- **Rust 对应**: `src/query/visitor/` 目录，实现表达式访问模式

#### 4. `validator/` - 查询验证器 (高优先级)
- **原因**: 查询处理流程的第二步，验证 AST 的合法性
- **内容**:
  - `MatchValidator`、`GoValidator`、`FetchVerticesValidator`
  - `LookupValidator`、`MaintainValidator`、`MutateValidator`
  - 各类语句和子句的验证器
- **Rust 对应**: `src/query/validator/` 目录，实现验证逻辑

### 第三阶段：计划与执行层迁移

#### 5. `planner/` - 查询计划生成器 (中优先级)
- **原因**: 基于验证后的 AST 生成执行计划
- **内容**:
  - `Planner`、`SequentialPlanner` 及各类子规划器
  - `ExecutionPlan` 及各类 PlanNode
  - `MatchPlanner`、`GoPlanner` 等特定语句规划器
- **Rust 对应**: `src/query/planner/` 目录，包含计划生成器和执行计划定义

#### 6. `optimizer/` - 查询优化器 (中优先级)
- **原因**: 优化执行计划，提升查询性能
- **内容**:
  - `Optimizer`、`OptContext`、`OptGroup`、`OptRule`
  - 各类优化规则（推下、索引优化、消除冗余、合并等）
- **Rust 对应**: `src/query/optimizer/` 目录，实现优化规则引擎

### 第四阶段：执行层迁移

#### 7. `executor/` - 执行引擎 (中优先级)
- **原因**: 执行优化后的计划，与存储层交互
- **内容**:
  - `GetVerticesExecutor`、`GetEdgesExecutor`、`GetNeighborsExecutor`
  - `FilterExecutor`、`ProjectExecutor`、`AggregateExecutor`
  - 各类查询、修改、维护、管理、算法、逻辑执行器
- **Rust 对应**: `src/query/executor/` 目录，实现各类执行器

#### 8. `scheduler/` - 执行调度器 (中优先级)
- **原因**: 协调和调度执行器的执行顺序
- **内容**:
  - `Scheduler`、`AsyncMsgNotifyBasedScheduler`
  - 执行器依赖关系管理和执行顺序调度
- **Rust 对应**: `src/query/scheduler/` 目录，实现执行调度逻辑

### 第五阶段：服务与管理层迁移

#### 9. `session/` - 会话管理 (低优先级)
- **原因**: 管理客户端连接和会话状态
- **内容**:
  - `ClientSession`、`GraphSessionManager`
  - 会话生命周期管理和连接状态维护
- **Rust 对应**: `src/api/session/` 目录，实现会话管理功能

#### 10. `service/` - 服务层 (低优先级)
- **原因**: 提供图数据库服务接口和请求处理
- **内容**:
  - `GraphService`、`QueryEngine`、`QueryInstance`
  - 身份验证、权限管理、请求处理
- **Rust 对应**: `src/api/service/` 目录，实现服务入口

#### 11. `stats/` - 统计信息 (低优先级)
- **原因**: 收集和管理查询执行的统计信息
- **内容**:
  - `GraphStats` 统计信息类
  - 性能监控和执行分析数据
- **Rust 对应**: `src/stats/` 目录，实现统计功能

#### 12. `gc/` - 垃圾回收 (低优先级)
- **原因**: Rust 有内置的内存管理机制，可能需要重新设计
- **内容**:
  - `GC` 垃圾回收管理
  - 对象生命周期管理（在 Rust 中可能需要重新考虑）
- **Rust 对应**: 依赖 Rust 的所有权和生命周期机制，可能不需要显式 GC

## 迁移策略

### 1. 渐进式迁移
- 按照上述顺序逐步迁移各个模块
- 在迁移过程中保持系统的可用性
- 逐步替换 C++ 实现为 Rust 实现

### 2. 接口兼容性
- 在迁移过程中保持与存储层、解析器等其他模块的接口兼容
- 设计清晰的 Rust 和 C++ 边界接口

### 3. 重构优化
- 利用 Rust 的特性对原有架构进行优化
- 简化分布式相关的复杂逻辑，专注于单节点场景

### 4. 测试保证
- 为每个迁移的模块编写充分的单元测试和集成测试
- 确保迁移后的功能与原有功能保持一致

## 风险与挑战

### 1. 性能对比
- 需要确保 Rust 实现在性能上不低于原有 C++ 实现
- 进行充分的性能测试和调优

### 2. 依赖管理
- Rust 生态系统的依赖可能与原有 C++ 依赖不兼容
- 需要寻找合适的 Rust 替代方案或实现

### 3. 学习曲线
- 团队需要熟悉 Rust 语言和生态系统
- 可能需要额外的培训和学习时间

### 4. 互操作性
- 在迁移过程中需要处理 Rust 和 C++ 代码的互操作性
- 可能需要使用 FFI（Foreign Function Interface）

## 成功指标

1. **功能完整性**: 迁移后的系统支持原有所有核心功能
2. **性能**: 在单节点场景下性能不低于原有系统
3. **稳定性**: 系统在长时间运行下保持稳定
4. **可维护性**: 代码结构清晰，易于维护和扩展
5. **内存安全**: 消除内存泄漏和并发安全问题
6. **部署便捷**: 生成单个可执行文件，便于部署

## 总结

通过按上述顺序和策略进行迁移，我们可以将 NebulaGraph 的 `src/graph` 模块成功迁移到 Rust 架构，实现一个更轻量、高性能、内存安全的单节点图数据库。这种渐进式迁移方式可以最大限度地降低风险，并确保在迁移过程中保持系统的可用性。