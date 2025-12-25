# Core Types 模块架构分析与重构建议

## 背景

当前项目中，`query` 和 `expression` 模块采用了清晰的分层架构：
- `core/types/` - 提供基础类型定义
- `expression/` - 提供表达式求值等业务逻辑
- `query/` - 提供查询处理等业务逻辑

这种架构实现了类型定义和业务逻辑的分离，但 `core/types/` 内部仍存在一些架构问题需要分析。

## 当前架构分析

### 1. Query 和 Expression 模块的依赖关系

#### 1.1 模块职责

**core/types/** - 基础类型定义层
- `expression.rs` - 表达式类型定义（Expression, LiteralValue, DataType）
- `query.rs` - 查询类型定义（QueryResult, QueryData, ScalarValue, FieldValue）
- `operators.rs` - 操作符类型定义

**expression/** - 表达式业务逻辑层
- `evaluator/` - 表达式求值器
- `context/` - 表达式上下文
- `functions/` - 函数实现
- `cache/` - 表达式缓存

**query/** - 查询业务逻辑层
- `executor/` - 查询执行器
- `optimizer/` - 查询优化器
- `parser/` - 查询解析器
- `planner/` - 查询规划器

#### 1.2 依赖方向

```
expression/ (业务逻辑)
    ↓ 使用
core/types/expression.rs (类型定义)

query/ (业务逻辑)
    ↓ 使用
core/types/expression.rs (类型定义)
core/types/query.rs (类型定义)
expression/ (业务逻辑) ← 单向依赖
```

**关键发现**：
- `query` 模块单向依赖 `expression` 模块（30+ 处）
- `expression` 模块对 `query` 模块的依赖极少（仅 3 处，都是使用 `FieldValue`）
- 这表明 `expression` 是更基础的模块，`query` 是更高层的模块

#### 1.3 循环依赖问题

**core/types 内部的循环依赖**：

```
core/types/expression.rs
    ↓ 定义
Expression { ... }

core/types/query.rs
    ↓ 定义
FieldValue { Scalar(...), Vertex(...), Edge(...), Path(...) }
    ↓ 使用
ScalarValue

expression/context/basic_context.rs
    ↓ 使用
FieldValue (来自 core/types/query.rs)

expression/evaluator/expression_evaluator.rs
    ↓ 使用
Expression (来自 core/types/expression.rs)
    ↓ 转换为
Value (来自 core/value.rs)
```

**问题**：
- `core/types/expression.rs` 和 `core/types/query.rs` 在同一层级，但存在隐式依赖
- `FieldValue` 定义在 `query.rs`，但被 `expression` 模块使用
- 这违反了分层架构原则，core/types 应该是纯粹的基础类型层

### 2. 当前架构的问题

#### 2.1 类型定义混乱

**问题 1：三个值类型重复定义**

```
core/value.rs
    └── Value (完整的运行时值类型)

core/types/expression.rs
    └── LiteralValue (表达式字面量)

core/types/query.rs
    └── ScalarValue (查询结果标量值)
```

这三个类型都包含相同的变体（Bool, Int, Float, String, Null），但各自独立实现。

**问题 2：职责不清**

- `core/value.rs` - 运行时值类型，包含所有可能的值类型
- `core/types/expression.rs` - 表达式类型，但包含字面量类型
- `core/types/query.rs` - 查询结果类型，但包含标量值类型

#### 2.2 模块边界不清

**core/types/** 应该是什么？
- 选项 A：纯粹的基础类型定义（类似 std）
- 选项 B：包含一些业务相关的类型定义（当前状态）

**当前状态**：
- `expression.rs` - 纯粹的类型定义 ✓
- `operators.rs` - 纯粹的类型定义 ✓
- `query.rs` - 包含业务相关的类型（QueryResult, Record, FieldValue）✗

#### 2.3 依赖关系不清晰

**expression 模块对 query 的依赖**（3 处）：
```rust
// expression/context/basic_context.rs
use crate::core::types::query::FieldValue;

// expression/functions/mod.rs
use crate::core::types::query::FieldValue;

// expression/cache/mod.rs
use crate::core::types::query::FieldValue;
```

**问题**：
- `expression` 模块（更基础的模块）依赖 `query` 模块（更高层的模块）
- 这违反了依赖倒置原则（DIP）
- 应该是 `query` 依赖 `expression`，而不是反过来

## 架构重构方案

### 方案 A：完全分离（推荐）

#### A.1 新的模块结构

```
core/
├── value.rs - 统一的值类型定义
├── types/
│   ├── expression.rs - 表达式类型（仅定义）
│   ├── operators.rs - 操作符类型
│   └── mod.rs
├── query_types/ (新模块)
│   ├── result.rs - 查询结果类型
│   ├── record.rs - 记录类型
│   └── mod.rs
└── mod.rs

expression/ - 表达式业务逻辑
    ├── evaluator/
    ├── context/
    ├── functions/
    └── cache/

query/ - 查询业务逻辑
    ├── executor/
    ├── optimizer/
    ├── parser/
    └── planner/
```

#### A.2 依赖关系

```
expression/ (业务逻辑)
    ↓ 使用
core/types/expression.rs (类型定义)
core/value.rs (值类型)

query/ (业务逻辑)
    ↓ 使用
core/types/expression.rs (类型定义)
core/query_types/ (查询类型)
core/value.rs (值类型)
expression/ (业务逻辑) ← 单向依赖
```

**优势**：
- 清晰的分层架构
- 消除循环依赖
- 模块职责明确
- 符合依赖倒置原则

#### A.3 类型迁移

**迁移前**：
```rust
// core/types/query.rs
pub enum QueryData {
    Scalar(ScalarValue),
    Records(Vec<Record>),
    // ...
}

pub enum ScalarValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Null,
}
```

**迁移后**：
```rust
// core/value.rs
pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Null,
    Vertex(Vertex),
    Edge(Edge),
    Path(Path),
    // ...
}

// core/query_types/result.rs
pub enum QueryData {
    Scalar(Value),  // 使用统一的 Value
    Records(Vec<Record>),
    // ...
}

// 删除 ScalarValue 枚举
```

#### A.4 消除 expression 对 query 的依赖

**迁移前**：
```rust
// expression/context/basic_context.rs
use crate::core::types::query::FieldValue;

pub struct BasicExpressionContext {
    pub variables: HashMap<String, FieldValue>,
    // ...
}
```

**迁移后**：
```rust
// expression/context/basic_context.rs
use crate::core::Value;

pub struct BasicExpressionContext {
    pub variables: HashMap<String, Value>,
    // ...
}
```

**修改点**：
- 将 `FieldValue` 改为 `Value`
- 更新所有使用 `FieldValue` 的地方（3 处）

### 方案 B：保持现状，仅优化类型

#### B.1 模块结构

```
core/
├── value.rs - 统一的值类型定义
├── types/
│   ├── expression.rs - 表达式类型
│   ├── query.rs - 查询类型
│   ├── operators.rs - 操作符类型
│   └── mod.rs
└── mod.rs

expression/ - 表达式业务逻辑
query/ - 查询业务逻辑
```

#### B.2 依赖关系

```
expression/ (业务逻辑)
    ↓ 使用
core/types/expression.rs
core/types/query.rs (仅 FieldValue)
core/value.rs

query/ (业务逻辑)
    ↓ 使用
core/types/expression.rs
core/types/query.rs
core/value.rs
expression/ (业务逻辑)
```

**优势**：
- 改动较小
- 风险较低

**劣势**：
- 仍然存在 expression 对 query 的依赖
- 模块边界不够清晰

### 方案 C：创建独立的 value 模块

#### C.1 模块结构

```
core/
├── value/
│   ├── mod.rs - 值类型定义
│   ├── scalar.rs - 标量值
│   ├── composite.rs - 复合值（List, Map, Set）
│   └── graph.rs - 图值（Vertex, Edge, Path）
├── types/
│   ├── expression.rs - 表达式类型
│   ├── query.rs - 查询类型
│   ├── operators.rs - 操作符类型
│   └── mod.rs
└── mod.rs

expression/ - 表达式业务逻辑
query/ - 查询业务逻辑
```

#### C.2 依赖关系

```
expression/ (业务逻辑)
    ↓ 使用
core/types/expression.rs
core/value/ (值类型)

query/ (业务逻辑)
    ↓ 使用
core/types/expression.rs
core/types/query.rs
core/value/ (值类型)
expression/ (业务逻辑)
```

**优势**：
- 更细粒度的模块划分
- 更好的代码组织

**劣势**：
- 模块数量增加
- 可能过度设计

## 推荐方案

### 推荐：方案 A（完全分离）

#### 理由

1. **符合分层架构原则**
   - `core/types/` - 纯粹的基础类型定义
   - `core/query_types/` - 查询相关的类型定义
   - `expression/` - 表达式业务逻辑
   - `query/` - 查询业务逻辑

2. **消除循环依赖**
   - `expression` 不再依赖 `query`
   - 依赖方向清晰：`query` → `expression` → `core/types`

3. **模块职责明确**
   - 每个模块都有明确的职责
   - 便于维护和扩展

4. **符合依赖倒置原则**
   - 高层模块（query）不依赖低层模块（expression）
   - 两者都依赖抽象（core/types）

#### 实施步骤

1. **创建 core/query_types 模块**
   ```bash
   mkdir src/core/query_types
   touch src/core/query_types/mod.rs
   touch src/core/query_types/result.rs
   touch src/core/query_types/record.rs
   ```

2. **迁移查询类型**
   - 将 `QueryResult`, `QueryData` 迁移到 `result.rs`
   - 将 `Record`, `FieldValue` 迁移到 `record.rs`
   - 删除 `ScalarValue` 枚举，使用 `Value`

3. **更新 core/types**
   - 从 `query.rs` 中删除迁移的类型
   - 保留 `QueryType` 枚举（如果需要）

4. **更新 expression 模块**
   - 将 `FieldValue` 改为 `Value`
   - 更新所有使用 `FieldValue` 的地方

5. **更新 core/mod.rs**
   - 添加 `pub mod query_types;`
   - 更新导出

6. **测试验证**
   - 运行所有测试
   - 验证功能正常

## 风险评估

### 高风险

1. **破坏性变更**
   - 迁移类型会影响大量代码
   - 需要全面测试确保功能正确

2. **向后兼容性**
   - 如果有外部 API 使用这些类型，需要提供兼容层

### 中风险

1. **编译错误**
   - 大量代码需要更新导入路径
   - 需要逐步修复编译错误

2. **测试覆盖**
   - 需要确保所有测试用例都能通过

### 低风险

1. **性能影响**
   - 主要是类型定义的迁移，不影响性能

2. **功能影响**
   - 不改变功能，只是重新组织代码

## 预期收益

### 架构收益

1. **清晰的分层架构**
   - 模块职责明确
   - 依赖关系清晰

2. **消除循环依赖**
   - `expression` 不再依赖 `query`
   - 符合依赖倒置原则

3. **更好的可维护性**
   - 模块边界清晰
   - 便于后续扩展

### 代码质量收益

1. **消除类型重复**
   - 统一使用 `Value` 类型
   - 减少代码重复

2. **提高类型安全**
   - 统一的类型系统
   - 减少类型转换错误

### 开发效率收益

1. **更快的编译**
   - 减少不必要的依赖
   - 降低编译时间

2. **更好的 IDE 支持**
   - 清晰的模块结构
   - 更好的代码导航

## 总结

当前 `core/types` 模块存在类型重复、职责不清、循环依赖等问题。通过采用类似 `query` 和 `expression` 模块的分层架构，可以显著改善代码质量。

**推荐方案**：方案 A（完全分离）

**核心思想**：
- `core/types/` - 纯粹的基础类型定义
- `core/query_types/` - 查询相关的类型定义
- `expression/` - 表达式业务逻辑
- `query/` - 查询业务逻辑

**依赖方向**：
```
query/ → expression/ → core/types/
         ↓
    core/query_types/
         ↓
    core/value.rs
```

这种架构符合分层架构原则、依赖倒置原则，能够消除循环依赖，提高代码质量和可维护性。

建议优先实施类型统一（参考 `type_system_optimization_proposal.md`），然后进行模块分离重构。整个重构过程需要充分测试，确保不破坏现有功能。
