# Core与Common目录职责划分分析

## 概述

本文档分析了当前`src/core`和`src/common`目录的职责划分，并提出了针对Expression和Query模块重构的职责重新分配建议。

## 当前Core目录功能分析

### 现有功能模块

`src/core`目录目前承担了以下核心功能：

1. **基础类型系统**：
   - `value.rs` - 核心Value类型定义（932行），包含所有数据类型和操作
   - `type_utils.rs` - 类型工具函数

2. **访问者模式基础设施**：
   - `visitor.rs` - 统一的访问者模式实现（512行）
   - `visitor_state_enum.rs` - 访问者状态管理

3. **错误处理**：
   - `error.rs` - 统一的错误类型定义（DBError, DBResult, ExpressionError, QueryError）

4. **图数据结构**：
   - `vertex_edge_path.rs` - 顶点、边、路径定义

5. **结果处理**：
   - `result/` - 结果集处理相关功能
   - `result/memory_manager.rs` - 内存管理器
   - `result/result_builder.rs` - 结果构建器
   - `result/result_core.rs` - 结果核心
   - `result/result_iterator.rs` - 结果迭代器

6. **符号系统**：
   - `symbol/` - 符号表和依赖跟踪
   - `symbol/dependency_tracker.rs` - 依赖跟踪器
   - `symbol/plan_node.rs` - 计划节点符号
   - `symbol/symbol_table.rs` - 符号表

7. **其他核心功能**：
   - `allocator.rs` - 分配器
   - `collect_n_succeeded.rs` - 收集成功结果
   - `cord.rs` - Cord数据结构
   - `either.rs` - Either类型
   - `murmur.rs` - Murmur哈希
   - `schema.rs` - 模式定义
   - `signal_handler.rs` - 信号处理器

### Core目录特点

1. **核心业务逻辑**：包含与图数据库核心业务相关的类型和功能
2. **数据结构**：定义了核心的数据结构（Value、Vertex、Edge、Path等）
3. **接口定义**：提供了访问者模式等核心接口
4. **错误处理**：统一的错误类型定义

## 当前Common目录功能分析

### 现有功能模块

`src/common`目录目前主要提供基础设施工具：

1. **系统工具**：
   - `fs.rs` - 文件系统操作
   - `network.rs` - 网络工具
   - `process.rs` - 进程管理
   - `thread.rs` - 线程管理

2. **基础工具**：
   - `base/id.rs` - ID生成器
   - `time.rs` - 时间处理
   - `memory.rs` - 内存管理
   - `log.rs` - 日志系统
   - `charset.rs` - 字符集处理

### Common目录特点

1. **基础设施**：提供系统级的基础设施功能
2. **通用工具**：不特定于图数据库的通用工具
3. **系统交互**：与操作系统交互的功能

## 针对Expression和Query模块重构的职责划分建议

### 问题分析

在Expression和Query模块重构过程中，需要解决以下问题：

1. **循环依赖**：Expression和Query模块之间存在循环依赖
2. **类型定义分散**：表达式和查询相关类型定义分散在多个模块
3. **上下文管理混乱**：多种上下文类型职责重叠

### 解决方案：将共享类型移入Core目录

#### 应移入Core目录的功能

1. **表达式相关类型**：
   ```rust
   // src/core/types/expression.rs
   pub enum Expression { ... }
   pub enum LiteralValue { ... }
   pub enum BinaryOperator { ... }
   pub enum UnaryOperator { ... }
   pub enum AggregateFunction { ... }
   pub enum DataType { ... }
   ```

2. **操作符类型定义**：
   ```rust
   // src/core/types/operators.rs
   pub trait Operator { ... }
   pub struct OperatorRegistry { ... }
   ```

3. **查询相关基础类型**：
   ```rust
   // src/core/types/query.rs
   pub enum QueryType { ... }
   pub enum QueryResult { ... }
   pub struct QueryError { ... }
   ```

4. **统一上下文系统**：
   ```rust
   // src/core/context/query.rs
   pub struct QueryContext { ... }
   
   // src/core/context/execution.rs
   pub struct ExecutionContext { ... }
   
   // src/core/context/session.rs
   pub struct SessionContext { ... }
   
   // src/core/context/expression.rs
   pub trait ExpressionContext { ... }
   ```

5. **表达式求值接口**：
   ```rust
   // src/core/evaluator/
   pub trait ExpressionEvaluator { ... }
   pub struct EvaluationContext { ... }
   ```

#### 保留在Common目录的功能

1. **系统级工具**：
   - 文件系统操作
   - 网络工具
   - 进程和线程管理

2. **基础设施工具**：
   - ID生成器
   - 时间处理
   - 内存管理
   - 日志系统

### 重构后的Core目录结构

```
src/core/
├── mod.rs
├── types/              # 核心类型系统
│   ├── mod.rs
│   ├── value.rs        # Value类型（现有）
│   ├── expression.rs   # 表达式类型（新增）
│   ├── operators.rs    # 操作符类型（新增）
│   ├── query.rs        # 查询类型（新增）
│   └── type_utils.rs   # 类型工具（现有）
├── context/            # 上下文系统
│   ├── mod.rs
│   ├── query.rs        # 查询上下文（新增）
│   ├── execution.rs    # 执行上下文（新增）
│   ├── session.rs      # 会话上下文（新增）
│   └── expression.rs   # 表达式上下文（新增）
├── evaluator/          # 求值器系统
│   ├── mod.rs
│   ├── traits.rs       # 求值器接口（新增）
│   └── context.rs      # 求值上下文（新增）
├── visitor/            # 访问者模式（现有）
│   ├── mod.rs
│   └── visitor_state_enum.rs
├── error/              # 错误处理（现有）
│   └── mod.rs
├── graph/              # 图数据结构（现有）
│   ├── mod.rs
│   └── vertex_edge_path.rs
├── result/             # 结果处理（现有）
│   └── mod.rs
├── symbol/             # 符号系统（现有）
│   └── mod.rs
└── 其他现有模块...
```

### 重构后的Common目录结构

```
src/common/
├── mod.rs
├── base/               # 基础工具（现有）
│   ├── mod.rs
│   └── id.rs
├── system/             # 系统工具（重组）
│   ├── mod.rs
│   ├── fs.rs           # 文件系统（现有）
│   ├── network.rs      # 网络（现有）
│   ├── process.rs      # 进程（现有）
│   └── thread.rs       # 线程（现有）
├── infrastructure/     # 基础设施（重组）
│   ├── mod.rs
│   ├── time.rs         # 时间（现有）
│   ├── memory.rs       # 内存（现有）
│   ├── log.rs          # 日志（现有）
│   └── charset.rs      # 字符集（现有）
```

## 重构实施计划

### 第一阶段：创建Core子模块（1周）

1. **创建新的子模块结构**：
   - 创建`src/core/types/`目录
   - 创建`src/core/context/`目录
   - 创建`src/core/evaluator/`目录

2. **迁移现有类型**：
   - 将`src/expression/expression.rs`中的类型定义移到`src/core/types/expression.rs`
   - 将操作符定义移到`src/core/types/operators.rs`

### 第二阶段：统一上下文系统（1-2周）

1. **设计统一上下文接口**：
   - 在`src/core/context/`中定义统一的上下文接口
   - 实现上下文层次结构

2. **迁移上下文实现**：
   - 将Expression模块的上下文移到Core
   - 将Query模块的上下文移到Core

### 第三阶段：更新依赖关系（1-2周）

1. **更新Expression模块**：
   - 修改Expression模块使用Core中的类型
   - 删除重复的类型定义

2. **更新Query模块**：
   - 修改Query模块使用Core中的类型
   - 删除重复的类型定义

### 第四阶段：验证和测试（1周）

1. **编译验证**：
   - 确保所有模块正确编译
   - 解决编译错误

2. **功能测试**：
   - 运行现有测试
   - 确保功能正常

## 预期收益

### 1. 解决循环依赖

通过将共享类型移入Core目录，消除Expression和Query模块间的循环依赖：

```
Expression → Core/Types ← Query
```

### 2. 统一类型系统

- 所有表达式相关类型集中在Core中
- 所有查询相关类型集中在Core中
- 避免类型定义的重复和分散

### 3. 清晰的职责划分

- **Core**：核心业务逻辑和数据结构
- **Common**：通用基础设施和系统工具
- **Expression**：表达式处理逻辑
- **Query**：查询处理逻辑

### 4. 更好的可维护性

- 类型定义集中，易于维护
- 依赖关系清晰，易于理解
- 模块职责明确，易于扩展

## 风险与缓解

### 1. 迁移风险

- **风险**：大规模类型迁移可能引入错误
- **缓解**：分阶段迁移，保持向后兼容

### 2. 性能风险

- **风险**：新的模块结构可能影响性能
- **缓解**：性能基准测试，优化关键路径

### 3. 兼容性风险

- **风险**：破坏现有API兼容性
- **缓解**：提供类型别名，渐进式迁移

## 结论

通过将Expression和Query模块的共享类型移入Core目录，可以：

1. **解决循环依赖问题**
2. **统一类型系统**
3. **明确模块职责**
4. **提高代码可维护性**

这种重构方案既保持了Core目录作为核心业务逻辑的定位，又避免了Common目录承担过多业务逻辑，是一个平衡的解决方案。

---

*文档生成日期：2025-06-17*
*分析工具：Roo Architect Mode*