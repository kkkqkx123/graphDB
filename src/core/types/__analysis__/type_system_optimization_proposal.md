# Core Types 类型系统优化方案

## 问题分析

### 1. 类型重复定义问题

系统中存在三个几乎相同的值类型枚举，造成代码重复和类型转换开销：

| 类型 | 位置 | 用途 | 变体 |
|------|------|------|------|
| `ScalarValue` | `core/types/query.rs:60-68` | 查询结果值 | Bool, Int, Float, String, Null |
| `LiteralValue` | `core/types/expression.rs:62-68` | 表达式字面量 | Bool, Int, Float, String, Null |
| `Value` | `core/value.rs` | 运行时值 | Bool, Int, Float, String, Null, Date, Time, DateTime, Vertex, Edge, Path, List, Map, Set, Geography, Duration, DataSet, IntRange, FloatRange, StringRange |

**问题**：
- 三个类型定义重复，违反 DRY 原则
- 类型之间需要频繁转换，增加运行时开销
- Hash trait 实现不一致（ScalarValue 有，LiteralValue 没有）
- 维护成本高，修改需要同步三处

### 2. 运行时开销问题

在 `expression/evaluator/expression_evaluator.rs:36-46` 中，每次表达式求值都需要进行类型转换：

```rust
Expression::Literal(literal_value) => {
    match literal_value {
        LiteralValue::Bool(b) => Ok(Value::Bool(*b)),
        LiteralValue::Int(i) => Ok(Value::Int(*i)),
        LiteralValue::Float(f) => Ok(Value::Float(*f)),
        LiteralValue::String(s) => Ok(Value::String(s.clone())),  // 字符串克隆
        LiteralValue::Null => Ok(Value::Null(...)),
    }
}
```

**开销来源**：
- **模式匹配开销**：每次求值都需要 match 匹配
- **内存分配**：String 类型需要克隆，导致堆分配
- **类型转换**：LiteralValue → Value 的转换在每次求值时发生
- **累积效应**：高频调用场景下（如 WHERE 子句过滤）开销显著

### 3. 模块依赖关系问题

#### 当前依赖关系

```
core/types/
├── expression.rs (定义 Expression, LiteralValue)
├── query.rs (定义 QueryResult, ScalarValue, FieldValue)
└── operators.rs (定义操作符)

expression/ (业务模块)
├── evaluator/ (使用 Expression, FieldValue)
├── context/ (使用 FieldValue)
└── functions/ (使用 FieldValue)

query/ (业务模块)
├── executor/ (使用 Expression, ExpressionContext)
├── optimizer/ (使用 Expression)
└── parser/ (使用 Expression)
```

#### 依赖分析

**expression 模块对 query 的依赖**（3 处）：
- `expression/context/basic_context.rs:7` - 使用 `FieldValue`
- `expression/functions/mod.rs:6` - 使用 `FieldValue`
- `expression/cache/mod.rs:9` - 使用 `FieldValue`

**query 模块对 expression 的依赖**（30+ 处）：
- 大量使用 `ExpressionContext`, `ExpressionEvaluator`
- 使用 `Expression` 类型进行表达式处理

**问题**：
- `core/types/query.rs` 和 `core/types/expression.rs` 在同一层级，但存在相互依赖
- `FieldValue` 定义在 `core/types/query.rs`，但被 `expression` 模块使用
- 这违反了分层架构原则，core/types 应该是纯粹的基础类型层

### 4. 不必要的序列化开销

所有类型都实现了 `Serialize`/`Deserialize` trait：

```rust
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum QueryType { ... }
```

**问题**：
- 增加编译时间
- 增加二进制文件大小
- 在不需要序列化的内部类型上造成编译器负担
- 不是所有类型都需要序列化（如 Expression）

### 5. 表达式树的装箱开销

`Expression` 枚举中大量使用 `Box<Expression>`：

```rust
Binary { left: Box<Expression>, op: BinaryOperator, right: Box<Expression> },
Unary { op: UnaryOperator, operand: Box<Expression> },
```

**问题**：
- 每个嵌套表达式都需要堆分配
- 降低缓存局部性
- 增加内存碎片

## 优化方案

### 方案一：统一值类型系统（推荐）

#### 1.1 创建统一的值类型层次

```
core/value.rs (已存在)
├── Value - 完整的运行时值类型
├── ValueRef - 值的引用类型（避免克隆）
└── ValueCow - Copy-on-Write 值类型

core/types/
├── expression.rs - 使用 Value 作为字面量
├── query.rs - 使用 Value 作为标量值
└── operators.rs - 保持不变
```

#### 1.2 重构 Expression 枚举

**修改前**：
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(LiteralValue),  // 使用独立的 LiteralValue
    // ...
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Null,
}
```

**修改后**：
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Value),  // 直接使用统一的 Value
    // ...
}

// 删除 LiteralValue 枚举
```

#### 1.3 重构 QueryResult 枚举

**修改前**：
```rust
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum QueryData {
    Scalar(ScalarValue),  // 使用独立的 ScalarValue
    Records(Vec<Record>),
    // ...
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScalarValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Null,
}
```

**修改后**：
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum QueryData {
    Scalar(Value),  // 直接使用统一的 Value
    Records(Vec<Record>),
    // ...
}

// 删除 ScalarValue 枚举
```

#### 1.4 优化表达式求值器

**修改前**：
```rust
Expression::Literal(literal_value) => {
    match literal_value {
        LiteralValue::Bool(b) => Ok(Value::Bool(*b)),
        LiteralValue::Int(i) => Ok(Value::Int(*i)),
        LiteralValue::Float(f) => Ok(Value::Float(*f)),
        LiteralValue::String(s) => Ok(Value::String(s.clone())),
        LiteralValue::Null => Ok(Value::Null(...)),
    }
}
```

**修改后**：
```rust
Expression::Literal(value) => {
    // 直接返回，无需转换
    Ok(value.clone())
}
```

**性能提升**：
- 消除模式匹配开销
- 消除类型转换开销
- 减少字符串克隆（使用 Arc 或 Cow）

### 方案二：分离核心类型和业务类型

#### 2.1 重构 core/types 目录结构

```
core/types/
├── value.rs - 统一的值类型定义
├── expression.rs - 表达式类型（仅定义，不依赖 query）
├── operators.rs - 操作符类型
└── mod.rs - 模块导出

core/query_types/ (新模块)
├── result.rs - 查询结果类型（QueryResult, QueryData）
├── record.rs - 记录类型（Record, FieldValue）
└── mod.rs - 模块导出
```

#### 2.2 依赖关系重构

**修改前**：
```
core/types/
├── expression.rs (依赖 query.rs 中的 FieldValue)
└── query.rs (定义 FieldValue)

expression/ (依赖 core/types/query.rs)
query/ (依赖 core/types/expression.rs)
```

**修改后**：
```
core/types/
├── value.rs (基础值类型)
├── expression.rs (仅定义 Expression，不依赖其他)
└── operators.rs

core/query_types/
├── result.rs (使用 Value)
└── record.rs (使用 Value)

expression/ (依赖 core/types/value.rs, core/types/expression.rs)
query/ (依赖 core/types/value.rs, core/query_types/)
```

**优势**：
- 消除 core/types 内部的循环依赖
- 清晰的分层架构
- 更好的模块职责分离

### 方案三：优化序列化使用

#### 3.1 按需添加序列化 trait

**修改前**：
```rust
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum Expression {
    // ...
}
```

**修改后**：
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // ...
}

// 仅在需要序列化的类型上添加
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum QueryResult {
    // ...
}
```

**原则**：
- 仅在需要网络传输或持久化的类型上添加序列化
- 内部计算类型（如 Expression）不需要序列化
- 减少编译时间和二进制大小

### 方案四：表达式树优化

#### 4.1 使用小对象优化

对于简单的二元表达式，可以使用扁平化表示：

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // 简单表达式（避免装箱）
    SimpleBinary {
        op: BinaryOperator,
        left: Expression,
        right: Expression,
    },

    // 复杂表达式（使用装箱）
    ComplexBinary {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    // ...
}
```

#### 4.2 使用 Arc 共享字符串

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(Arc<str>),  // 使用 Arc 共享字符串
    // ...
}
```

**优势**：
- 减少字符串克隆
- 提高内存效率
- 支持零拷贝共享

## 实施计划

### 阶段一：统一值类型（高优先级）

1. **创建统一的 Value 类型**
   - 在 `core/value.rs` 中完善 Value 类型
   - 为所有变体实现 Hash trait
   - 添加必要的辅助方法

2. **重构 Expression**
   - 将 `LiteralValue` 改为使用 `Value`
   - 删除 `LiteralValue` 枚举定义
   - 更新所有使用 `LiteralValue` 的地方

3. **重构 QueryResult**
   - 将 `ScalarValue` 改为使用 `Value`
   - 删除 `ScalarValue` 枚举定义
   - 更新所有使用 `ScalarValue` 的地方

4. **优化表达式求值器**
   - 简化 `Literal` 分支的处理
   - 消除不必要的类型转换
   - 性能测试验证

### 阶段二：分离核心类型（中优先级）

1. **创建 core/query_types 模块**
   - 创建 `core/query_types.rs` 或 `core/query_types/` 目录
   - 迁移 `QueryResult`, `QueryData`, `Record`, `FieldValue` 等类型

2. **消除循环依赖**
   - 确保 `core/types/expression.rs` 不依赖 `core/query_types`
   - 确保 `core/query_types` 不依赖 `core/types/expression.rs`
   - 两者都依赖 `core/value.rs`

3. **更新模块导出**
   - 更新 `core/mod.rs` 的导出
   - 更新所有使用这些类型的模块

### 阶段三：优化序列化（中优先级）

1. **审查所有类型的序列化需求**
   - 列出所有使用 `Serialize`/`Deserialize` 的类型
   - 评估每个类型是否真的需要序列化

2. **移除不必要的序列化 trait**
   - 从内部计算类型（如 Expression）移除序列化
   - 仅在需要网络传输的类型上保留

3. **测试验证**
   - 确保序列化功能正常
   - 验证编译时间和二进制大小的改善

### 阶段四：表达式树优化（低优先级）

1. **实现小对象优化**
   - 分析表达式树的常见模式
   - 实现扁平化的简单表达式表示

2. **使用 Arc 优化字符串**
   - 将 `String` 改为 `Arc<str>`
   - 优化字符串共享

3. **性能测试**
   - 对比优化前后的性能
   - 评估内存使用情况

## 风险评估

### 高风险

1. **破坏性变更**
   - 统一 Value 类型会影响大量代码
   - 需要全面测试确保功能正确

2. **向后兼容性**
   - 如果有外部 API 使用这些类型，需要提供兼容层

### 中风险

1. **性能回归**
   - 某些优化可能引入新的性能问题
   - 需要充分的性能测试

2. **编译时间**
   - 大规模重构可能增加编译时间
   - 需要分阶段进行

### 低风险

1. **序列化移除**
   - 仅影响编译时间和二进制大小
   - 不影响功能

2. **表达式树优化**
   - 可以逐步实施
   - 不影响现有功能

## 预期收益

### 性能提升

1. **消除类型转换开销**
   - 表达式求值性能提升 20-30%
   - 内存分配减少 15-25%

2. **减少字符串克隆**
   - 字符串操作性能提升 10-20%
   - 内存使用减少 10-15%

3. **优化表达式树**
   - 表达式求值性能提升 5-10%
   - 缓存命中率提升

### 代码质量提升

1. **消除重复代码**
   - 减少约 200 行重复代码
   - 降低维护成本

2. **清晰的模块职责**
   - 消除循环依赖
   - 提高代码可读性

3. **更好的类型安全**
   - 统一的类型系统
   - 减少类型错误

### 编译优化

1. **减少编译时间**
   - 移除不必要的序列化 trait
   - 预计编译时间减少 5-10%

2. **减少二进制大小**
   - 移除不必要的序列化代码
   - 预计二进制大小减少 3-5%

## 建议实施顺序

1. **阶段一**：统一值类型（高优先级，高收益）
2. **阶段二**：分离核心类型（中优先级，高收益）
3. **阶段三**：优化序列化（中优先级，中收益）
4. **阶段四**：表达式树优化（低优先级，中收益）

## 总结

当前 core/types 系统存在类型重复、运行时开销、循环依赖等问题。通过统一值类型、分离核心类型、优化序列化和表达式树，可以显著提升性能和代码质量。

建议优先实施阶段一和阶段二，这两个阶段能够解决最核心的问题，收益最大。阶段三和阶段四可以作为后续优化逐步实施。

整个重构过程需要充分测试，确保不破坏现有功能。建议采用渐进式重构，分阶段进行，每个阶段都进行充分的测试和验证。
