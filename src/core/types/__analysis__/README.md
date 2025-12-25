# Core Types 优化方案总结

## 文档说明

本目录包含对 `core/types` 模块的详细分析和优化建议：

1. **type_system_optimization_proposal.md** - 类型系统优化方案
2. **module_architecture_analysis.md** - 模块架构分析与重构建议

## 核心问题

### 1. 类型重复定义

系统中存在三个几乎相同的值类型枚举：

| 类型 | 位置 | 用途 |
|------|------|------|
| `ScalarValue` | `core/types/query.rs` | 查询结果值 |
| `LiteralValue` | `core/types/expression.rs` | 表达式字面量 |
| `Value` | `core/value.rs` | 运行时值 |

**问题**：
- 代码重复，违反 DRY 原则
- 类型之间需要频繁转换，增加运行时开销
- Hash trait 实现不一致
- 维护成本高

### 2. 运行时开销

每次表达式求值都需要进行类型转换：

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
- 模式匹配开销
- 内存分配（字符串克隆）
- 类型转换开销
- 累积效应（高频调用场景）

### 3. 模块依赖问题

#### 当前依赖关系

```
expression/ (业务逻辑)
    ↓ 使用 (3 处)
core/types/query.rs (FieldValue)

query/ (业务逻辑)
    ↓ 使用 (30+ 处)
core/types/expression.rs (Expression)
expression/ (业务逻辑) ← 单向依赖
```

**问题**：
- `expression` 模块（更基础）依赖 `query` 模块（更高层）
- 违反依赖倒置原则（DIP）
- `core/types` 内部存在隐式循环依赖

### 4. 不必要的序列化

所有类型都实现了 `Serialize`/`Deserialize` trait：

```rust
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum QueryType { ... }
```

**问题**：
- 增加编译时间
- 增加二进制文件大小
- 在不需要序列化的内部类型上造成编译器负担

### 5. 表达式树装箱

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

#### 核心思想

将 `ScalarValue`、`LiteralValue` 和 `Value` 统一为一个类型系统。

#### 实施步骤

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

#### 预期收益

- 表达式求值性能提升 20-30%
- 内存分配减少 15-25%
- 字符串操作性能提升 10-20%
- 减少约 200 行重复代码

### 方案二：分离核心类型（推荐）

#### 核心思想

参考 `query` 和 `expression` 模块的分层架构，重构 `core/types` 模块。

#### 新的模块结构

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
query/ - 查询业务逻辑
```

#### 依赖关系

```
query/ (业务逻辑)
    ↓ 使用
core/types/expression.rs
core/query_types/
core/value.rs
expression/ (业务逻辑) ← 单向依赖
```

**优势**：
- 清晰的分层架构
- 消除循环依赖
- 模块职责明确
- 符合依赖倒置原则

#### 实施步骤

1. **创建 core/query_types 模块**
2. **迁移查询类型**
3. **更新 core/types**
4. **更新 expression 模块**
5. **更新 core/mod.rs**
6. **测试验证**

### 方案三：优化序列化使用

#### 核心思想

按需添加序列化 trait，仅在需要网络传输或持久化的类型上添加。

#### 实施步骤

1. **审查所有类型的序列化需求**
2. **移除不必要的序列化 trait**
3. **测试验证**

#### 预期收益

- 编译时间减少 5-10%
- 二进制大小减少 3-5%

### 方案四：表达式树优化

#### 核心思想

使用小对象优化和 Arc 共享字符串，减少堆分配。

#### 实施步骤

1. **实现小对象优化**
2. **使用 Arc 优化字符串**
3. **性能测试**

#### 预期收益

- 表达式求值性能提升 5-10%
- 缓存命中率提升

## 实施计划

### 阶段一：统一值类型（高优先级，高收益）

**目标**：消除类型重复，统一值类型系统

**步骤**：
1. 创建统一的 Value 类型
2. 重构 Expression
3. 重构 QueryResult
4. 优化表达式求值器

**预期收益**：
- 性能提升 20-30%
- 减少代码重复

**风险**：高（破坏性变更）

### 阶段二：分离核心类型（中优先级，高收益）

**目标**：消除循环依赖，清晰模块职责

**步骤**：
1. 创建 core/query_types 模块
2. 迁移查询类型
3. 消除循环依赖
4. 更新模块导出

**预期收益**：
- 清晰的架构
- 更好的可维护性

**风险**：中（需要全面测试）

### 阶段三：优化序列化（中优先级，中收益）

**目标**：减少编译时间和二进制大小

**步骤**：
1. 审查序列化需求
2. 移除不必要的序列化 trait
3. 测试验证

**预期收益**：
- 编译时间减少 5-10%
- 二进制大小减少 3-5%

**风险**：低（不影响功能）

### 阶段四：表达式树优化（低优先级，中收益）

**目标**：减少堆分配，提高性能

**步骤**：
1. 实现小对象优化
2. 使用 Arc 优化字符串
3. 性能测试

**预期收益**：
- 性能提升 5-10%
- 缓存命中率提升

**风险**：低（可以逐步实施）

## 建议实施顺序

1. **阶段一**：统一值类型（高优先级，高收益）
2. **阶段二**：分离核心类型（中优先级，高收益）
3. **阶段三**：优化序列化（中优先级，中收益）
4. **阶段四**：表达式树优化（低优先级，中收益）

## 总结

当前 `core/types` 系统存在类型重复、运行时开销、循环依赖等问题。通过统一值类型、分离核心类型、优化序列化和表达式树，可以显著提升性能和代码质量。

**核心建议**：
1. 优先实施阶段一和阶段二，这两个阶段能够解决最核心的问题，收益最大
2. 阶段三和阶段四可以作为后续优化逐步实施
4. 整个重构过程需要充分测试，确保不破坏现有功能
5. 建议采用渐进式重构，分阶段进行，每个阶段都进行充分的测试和验证

**关键收益**：
- 性能提升 20-30%（阶段一）
- 清晰的架构和更好的可维护性（阶段二）
- 编译时间减少 5-10%（阶段三）
- 性能提升 5-10%（阶段四）

**参考文档**：
- `type_system_optimization_proposal.md` - 详细的类型系统优化方案
- `module_architecture_analysis.md` - 模块架构分析与重构建议
