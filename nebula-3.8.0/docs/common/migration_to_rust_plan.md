# 从 NebulaGraph Common 迁移到 Rust 架构的规划

## 概述

本文档提供了从 NebulaGraph 的 Common 模块迁移到 Rust 图数据库架构的详细规划。重点是确定需要迁移的模块及其实现优先级。

## 迁移策略

### Rust 中的等效实现

1. **基础类型和错误处理**:
   - `base` 模块中的 `Status` 和 `StatusOr` 类型可以使用 Rust 的 `Result<T, E>` 类型替代
   - `ErrorOr` 可以映射到 Rust 的错误处理机制

2. **数据类型**:
   - `datatypes` 模块中的 `Value` 类型需要用 Rust 的枚举来重新实现
   - `DataSet`, `Vertex`, `Edge`, `Path` 等核心数据结构需要使用 Rust 的结构体和枚举实现

3. **表达式解析器**:
   - `expression` 模块的表达式系统需要用 Rust 实现
   - 需要重新实现访问者模式以适应 Rust 的特质（trait）

4. **并发和线程**:
   - `thread` 模块的线程池可以用 Rust 的 `std::thread` 或 `tokio` 等异步运行时替代

## 迁移路线图

### 阶段 1: 核心数据类型和基础功能 (高优先级)

#### 1. 实现基础类型
- [ ] `datatypes` 模块: `Value` 类型（Rust 枚举）
- [ ] `datatypes` 模块: 其他数据类型（`DataSet`, `Vertex`, `Edge`, `Path`, `Date`, `Time` 等）
- [ ] `base` 模块: 错误处理类型（`Status`, `StatusOr`，映射到 Rust Result<T, E>）

#### 2. 实现表达式系统
- [ ] `expression` 模块: 表达式基类（使用 Rust 特质和枚举）
- [ ] `expression` 模块: 各种表达式类型（算术、逻辑、函数调用等）

#### 3. 实现通用工具
- [ ] `utils` 模块: 通用工具函数
- [ ] `conf` 模块: 配置管理系统

### 阶段 2: 业务功能 (中优先级)

#### 1. 算法和数据结构
- [ ] `algorithm` 模块: 图算法实现
- [ ] `memory` 模块: 内存管理（可利用 Rust 的所有权机制）

#### 2. 网络和通信
- [ ] `network` 模块: 网络通信层
- [ ] `session` 模块: 会话管理系统

#### 3. 存储和文件系统
- [ ] `fs` 模块: 文件系统操作
- [ ] `id` 模块: ID 生成和管理

### 阶段 3: 完善和扩展 (低优先级)

#### 1. 扩展功能
- [ ] `http` 模块: HTTP 协议支持
- [ ] `geo` 模块: 地理位置处理（可选）

#### 2. 监控和日志
- [ ] `log` 模块: 日志系统
- [ ] `stats` 模块: 统计信息收集

## Rust 实现示例

### Value 类型的实现示例

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Set(std::collections::HashSet<Value>),
    Map(std::collections::HashMap<String, Value>),
    // 其他类型...
}
```

### Status 类型的实现示例

```rust
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    OK,
    Error(String),
    // 其他状态...
}

pub type StatusOr<T> = Result<T, Status>;

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::OK => write!(f, "OK"),
            Status::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}
```

### 表达式系统的实现示例

```rust
trait ExprVisitor {
    fn visit_arithmetic_expr(&mut self, expr: &ArithmeticExpression) -> Result<()>;
    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpression) -> Result<()>;
    // 其他表达式类型...
}

#[derive(Debug, Clone)]
pub enum Expression {
    Arithmetic(ArithmeticExpression),
    FunctionCall(FunctionCallExpression),
    // 其他表达式类型...
}

impl Expression {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> Result<()> {
        match self {
            Expression::Arithmetic(expr) => visitor.visit_arithmetic_expr(expr),
            Expression::FunctionCall(expr) => visitor.visit_function_call_expr(expr),
            // 其他表达式类型...
        }
    }
}
```

## 潜在挑战和解决方案

### 1. 内存管理
- **挑战**: 从 C++ 的手动内存管理迁移到 Rust 的所有权系统
- **解决方案**: 利用 Rust 的所有权、借用和生命周期机制，避免垃圾回收的性能开销

### 2. 并发处理
- **挑战**: 重新实现 C++ 的并发模式
- **解决方案**: 使用 Rust 的线程、消息传递和异步编程模型

### 3. 外部库依赖
- **挑战**: 替换 C++ 的第三方库
- **解决方案**: 选择适当的 Rust 库或自己实现

## 总结

迁移计划应该优先考虑核心数据类型和基础功能，然后逐步扩展到业务功能和扩展功能。Rust 的类型系统、内存安全和并发模型将为系统提供更好的安全性和性能。

在迁移过程中，应特别关注表达式系统和错误处理机制的重构，因为这些是查询引擎的核心组件。