# Expression和Query模块迁移实施计划

## 概述

本文档详细说明了将Expression模块和Query模块的功能迁移到Core模块的实施计划，旨在消除代码重复，统一类型定义，并优化模块结构。

## 迁移目标

1. **消除类型重复** - 将Expression模块和Query模块中的重复类型定义统一到Core模块
2. **整合功能实现** - 将Expression和Query模块的功能实现迁移到Core模块
3. **优化模块结构** - 建立清晰的模块层次结构，减少循环依赖
4. **保持向后兼容** - 在迁移过程中确保系统稳定性

## 分析结果

### Expression模块与Core模块的重复情况

**完全重复的类型定义：**
- `Expression` 枚举 - 两个文件中的定义完全一致，包含164个变体
- `LiteralValue` 枚举 - 完全相同的5个变体
- `BinaryOperator` 枚举 - 完全相同的操作符定义
- `UnaryOperator` 枚举 - 完全相同的操作符定义
- `AggregateFunction` 枚举 - 完全相同的聚合函数定义
- `DataType` 枚举 - 完全相同的数据类型定义
- `ExpressionType` 枚举 - 完全相同的表达式类型分类

**Expression模块的额外功能：**
- 表达式求值器实现
- 操作实现（二元、一元、函数等）
- 表达式上下文系统
- Cypher查询语言支持
- 存储层集成
- 访问者模式支持

### Query模块与Core模块的重复情况

**上下文类型的重复与差异：**
- `QueryExecutionContext` vs `ExecutionContext` - 功能重叠但实现不同
- `QueryAstContext` vs `QueryContext` - 专门用途 vs 通用用途
- `ExecutionResult` vs `QueryResult` - 执行器返回 vs 查询结果

**重复的数据结构：**
- 顶点、边、路径定义
- 记录和字段值定义

**Query模块的额外功能：**
- 查询规划系统
- 查询执行系统
- 查询验证系统
- 查询调度系统
- 查询访问者模式
- 查询管道管理

## 迁移实施计划

### 阶段一：Expression模块类型统一（高优先级）

**目标文件：**
- `src/expression/expression.rs` - 需要删除重复类型定义
- `src/expression/mod.rs` - 需要更新导出路径
- `src/core/types/expression.rs` - 保留作为权威类型定义

**修改说明：**
1. 删除 `src/expression/expression.rs` 中的所有类型定义
2. 更新 `src/expression/mod.rs` 从Core模块重新导出类型
3. 将所有使用 `crate::expression::Expression` 的代码改为 `crate::core::Expression`

**影响范围：**
- 所有引用Expression类型的文件
- Expression模块的子模块

### 阶段二：Expression模块功能迁移（中优先级）

**目标文件：**
- `src/expression/evaluator.rs` → `src/core/evaluator/expression.rs`
- `src/expression/evaluator_trait.rs` → `src/core/evaluator/traits.rs`
- `src/expression/binary.rs` → `src/core/evaluator/operations/binary.rs`
- `src/expression/unary.rs` → `src/core/evaluator/operations/unary.rs`
- `src/expression/function.rs` → `src/core/evaluator/operations/function.rs`
- `src/expression/context/default.rs` → `src/core/context/expression.rs`

**修改说明：**
1. 移动求值器相关文件到Core模块
2. 更新所有导入路径
3. 整合表达式上下文到Core模块的上下文系统

**影响范围：**
- 所有使用表达式求值器的代码
- Expression模块的上下文用户

### 阶段三：Query模块类型统一（高优先级）

**目标文件：**
- `src/query/context/execution_context.rs` - 需要整合到Core模块
- `src/query/context/ast/query_ast_context.rs` - 需要迁移到Core模块
- `src/core/context/query.rs` - 保留并扩展
- `src/core/context/execution.rs` - 保留并扩展
- `src/core/types/query.rs` - 保留作为权威类型定义

**修改说明：**
1. 将Query模块的上下文功能整合到Core模块
2. 统一查询结果类型定义
3. 更新所有导入路径

**影响范围：**
- 所有使用查询上下文的代码
- Query模块的子模块

### 阶段四：Query模块功能迁移（中优先级）

**目标文件：**
- `src/query/planner/planner.rs` → `src/core/query/planner/`
- `src/query/planner/plan/execution_plan.rs` → `src/core/query/plan/`
- `src/query/executor/traits.rs` → `src/core/query/executor/traits.rs`
- `src/query/executor/factory.rs` → `src/core/query/executor/factory.rs`
- `src/query/validator/` → `src/core/query/validator/`
- `src/query/scheduler/` → `src/core/query/scheduler/`

**修改说明：**
1. 迁移查询规划系统到Core模块
2. 迁移查询执行系统到Core模块
3. 迁移查询支持系统到Core模块
4. 更新所有导入路径

**影响范围：**
- 所有使用查询规划器的代码
- 所有使用查询执行器的代码
- Query模块的所有子模块

### 阶段五：清理和优化（低优先级）

**目标文件：**
- `src/expression/` - 整个目录可以删除
- `src/query/` - 保留部分文件，删除重复代码
- `src/core/mod.rs` - 更新导出

**修改说明：**
1. 删除Expression模块目录
2. 清理Query模块中的重复代码
3. 优化Core模块的导出结构

**影响范围：**
- 整个项目的导入结构
- 文档和注释

## 风险评估和缓解措施

### 主要风险

1. **循环依赖** - 模块间可能存在循环依赖
2. **编译错误** - 大量导入路径更改可能导致编译错误
3. **功能回归** - 迁移过程中可能丢失某些功能
4. **性能影响** - 迁移可能影响系统性能

### 缓解措施

1. **分阶段验证** - 每个阶段完成后进行完整的测试
2. **保留适配器** - 在迁移期间提供临时适配器确保兼容性
3. **增量迁移** - 一次迁移一个子模块，减少影响范围
4. **性能基准** - 建立性能基准，确保迁移不影响性能

## 时间表

| 阶段 | 预估工作量 | 优先级 | 依赖关系 |
|------|------------|--------|----------|
| 阶段一：Expression类型统一 | 2-3天 | 高 | 无 |
| 阶段二：Expression功能迁移 | 3-5天 | 中 | 阶段一完成 |
| 阶段三：Query类型统一 | 3-4天 | 高 | 无 |
| 阶段四：Query功能迁移 | 5-7天 | 中 | 阶段三完成 |
| 阶段五：清理和优化 | 2-3天 | 低 | 前面阶段完成 |

## 迁移后的模块结构

```
src/core/
├── types/
│   ├── expression.rs    # 表达式类型定义
│   ├── query.rs         # 查询类型定义
│   └── operators.rs     # 操作符定义
├── context/
│   ├── base.rs          # 基础上下文
│   ├── query.rs         # 查询上下文
│   ├── execution.rs     # 执行上下文
│   └── expression.rs    # 表达式上下文
├── evaluator/
│   ├── traits.rs        # 求值器接口
│   ├── expression.rs    # 表达式求值器
│   └── operations/      # 操作实现
├── query/
│   ├── planner/         # 查询规划器
│   ├── executor/        # 查询执行器
│   ├── validator/       # 查询验证器
│   ├── scheduler/       # 查询调度器
│   └── visitor/         # 查询访问者
└── cypher/              # Cypher支持
```

## 验证计划

1. **编译验证** - 每个阶段完成后确保代码能够编译
2. **单元测试** - 运行所有单元测试确保功能正确
3. **集成测试** - 运行集成测试确保模块间协作正常
4. **性能测试** - 运行性能测试确保系统性能不受影响
5. **功能测试** - 运行完整的功能测试确保所有功能正常

## 总结

本迁移计划旨在消除Expression模块和Query模块与Core模块之间的代码重复，统一类型定义，并优化模块结构。通过分阶段的迁移策略，我们可以在保持系统稳定性的同时，实现代码的整合和优化。

迁移完成后，Core模块将成为整个系统的核心，提供统一的类型定义和功能实现，而Expression模块和Query模块将被整合或删除，从而简化整个项目的结构。