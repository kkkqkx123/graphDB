# 核心类型系统 (Core Types System)

本目录包含图数据库的核心类型定义，是整个系统的基础数据结构。这些类型被查询引擎、表达式求值器、存储引擎等多个核心组件广泛使用。

## 目录结构

- [`expression.rs`](./expression.rs) - 表达式类型系统
- [`operators.rs`](./operators.rs) - 操作符类型定义
- [`query.rs`](./query.rs) - 查询相关类型定义

## 文件详细说明

### expression.rs - 表达式类型系统

**作用**：定义了图数据库查询语言中的表达式类型，是查询解析和执行的核心数据结构。

**主要类型**：
- `Expression` - 表达式枚举，包含字面量、变量、属性访问、二元/一元操作、函数调用等
- `LiteralValue` - 字面量值类型（布尔、整数、浮点、字符串、空值）
- `AggregateFunction` - 聚合函数类型（Count、Sum、Avg、Min、Max等）
- `DataType` - 数据类型枚举
- `ExpressionType` - 表达式类型分类

**使用情况**：
- 被查询解析器广泛使用，用于构建抽象语法树(AST)
- 被表达式求值器使用，进行表达式计算
- 被查询优化器使用，进行查询优化
- 被查询验证器使用，进行查询验证

**特点**：
- 使用枚举变体减少装箱，优化内存使用和性能
- 支持图数据库特有的表达式类型（如TagProperty、EdgeProperty等）
- 提供丰富的构建器方法，方便表达式构造

### operators.rs - 操作符类型定义

**作用**：定义了图数据库中支持的各种操作符，包括二元操作符、一元操作符和聚合函数。

**主要类型**：
- `Operator` - 操作符特征定义
- `OperatorRegistry` - 操作符注册表
- `OperatorInstance` - 操作符实例枚举
- `BinaryOperator` - 二元操作符（算术、比较、逻辑、字符串等）
- `UnaryOperator` - 一元操作符（算术、逻辑、存在性检查等）
- `AggregateFunction` - 聚合函数
- `OperatorCategory` - 操作符类别

**使用情况**：
- 被表达式解析器使用，识别和解析操作符
- 被表达式求值器使用，执行操作符对应的计算
- 被查询优化器使用，进行操作符重排序和优化

**特点**：
- 定义了操作符的优先级和结合性
- 使用枚举避免动态分发，提高性能
- 提供操作符注册表，支持动态扩展

### query.rs - 查询相关类型定义

**作用**：定义了查询执行过程中的数据结构和结果类型。

**主要类型**：
- `QueryType` - 查询类型枚举（数据查询、数据操作、数据定义等）
- `QueryResult` - 查询结果类型
- `QueryData` - 查询数据（标量值、记录集合、图数据、路径集合等）
- `Record` - 记录类型
- `FieldValue` - 字段值类型
- `Vertex` - 顶点类型
- `Edge` - 边类型
- `Path` - 路径类型
- `GraphData` - 图数据类型
- `Statistics` - 统计信息类型
- `QueryError` - 查询错误类型

**使用情况**：
- 被查询执行器使用，构建查询结果
- 被表达式求值器使用，进行值类型转换
- 被上下文管理器使用，存储查询状态和结果
- 被API层使用，返回查询结果给客户端

**特点**：
- 支持序列化和反序列化，便于数据传输和持久化
- 为f64类型实现了自定义Hash trait，处理NaN值的特殊情况
- 提供丰富的辅助方法，方便数据操作

## 类型关系图

```
expression.rs
    ├── Expression (使用)
    │   ├── BinaryOperator (来自 operators.rs)
    │   ├── UnaryOperator (来自 operators.rs)
    │   └── AggregateFunction (来自 operators.rs)
    └── LiteralValue

operators.rs
    ├── BinaryOperator
    ├── UnaryOperator
    └── AggregateFunction

query.rs
    ├── QueryResult
    │   └── QueryData
    │       ├── Record
    │       │   └── FieldValue
    │       │       ├── Vertex
    │       │       ├── Edge
    │       │       └── Path
    │       └── GraphData
    └── QueryError
```

## 设计原则

1. **类型安全**：使用Rust的类型系统确保编译时安全
2. **性能优化**：使用枚举避免动态分发，减少内存分配
3. **可扩展性**：通过特征和枚举设计支持未来扩展
4. **一致性**：所有类型都实现了序列化/反序列化，保持数据一致性
5. **文档化**：所有公共类型和方法都有详细的文档注释

## 使用指南

### 创建表达式

```rust
use crate::core::types::expression::{Expression, LiteralValue};
use crate::core::types::operators::{BinaryOperator, UnaryOperator};

// 创建字面量表达式
let expr = Expression::literal(42);

// 创建二元操作表达式
let add_expr = Expression::binary(
    Expression::literal(1),
    BinaryOperator::Add,
    Expression::literal(2)
);

// 创建一元操作表达式
let neg_expr = Expression::unary(
    UnaryOperator::Minus,
    Expression::literal(5)
);
```

### 使用操作符

```rust
use crate::core::types::operators::{OperatorRegistry, BinaryOperator};

let mut registry = OperatorRegistry::new();
registry.register_binary(BinaryOperator::Add);

// 查找操作符
if let Some(op) = registry.find_by_name("+") {
    println!("找到操作符: {:?}", op);
}
```

### 处理查询结果

```rust
use crate::core::types::query::{QueryResult, QueryData, Record, FieldValue};

// 创建成功结果
let result = QueryResult::success(
    1,
    Some(QueryData::Records(vec![Record::new()])),
    100
);

// 检查结果
if result.is_success() {
    if let Some(data) = result.get_success_data() {
        println!("查询成功，返回数据: {:?}", data);
    }
}
```

## 注意事项

1. **类型转换**：在不同模块间传递数据时，注意类型转换的正确性
2. **内存管理**：虽然使用了优化设计，但仍需注意大型查询结果的内存使用
3. **序列化**：所有公共类型都支持序列化，但自定义类型需要手动实现
4. **错误处理**：使用QueryError统一处理查询过程中的错误

## 未来扩展

1. **更多表达式类型**：根据查询语言的发展，可能需要添加新的表达式类型
2. **自定义操作符**：支持用户自定义操作符
3. **类型系统增强**：添加更丰富的类型检查和推导功能
4. **性能优化**：进一步优化内存使用和计算性能