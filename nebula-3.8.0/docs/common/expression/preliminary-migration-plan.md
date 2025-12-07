# 表达式系统初步迁移方案

## 1. 概述

本文档提出了将 NebulaGraph 的表达式系统从 C++ 迁移到 Rust 的初步方案。该方案旨在利用 Rust 的类型安全、内存安全和零成本抽象等特性，同时保持原始系统的所有核心功能。

## 2. 设计原则

### 2.1 类型安全优先
- 使用 Rust 枚举替代 C++ 继承层次结构
- 利用 Rust 的类型系统在编译时捕获更多错误
- 实现强类型表达式评估

### 2.2 性能优化
- 保持或改善原始系统的性能
- 利用 Rust 的零成本抽象特性
- 避免不必要的堆分配

### 2.3 内存安全
- 使用 Rust 所有权系统替代手动内存管理
- 消除内存泄漏和悬空指针
- 支持高效的并发访问

## 3. 核心数据结构设计

### 3.1 基础值类型
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Date(Date),
    Time(Time),
    DateTime(DateTime),
    Vertex(Vertex),
    Edge(Edge),
    Path(Path),
    List(Vec<Value>),
    Map(std::collections::HashMap<String, Value>),
    Set(std::collections::HashSet<Value>),
    // 其他类型...
}
```

### 3.2 表达式枚举
```rust
#[derive(Debug, Clone)]
pub enum Expression {
    // 常量表达式
    Constant(Value),
    
    // 一元表达式
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
    },
    
    // 二元表达式
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    
    // 属性访问表达式
    Property {
        entity: Box<Expression>,
        property: String,
    },
    
    // 函数调用表达式
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    
    // 变量引用表达式
    Variable {
        name: String,
    },
    
    // 容器表达式
    List(Vec<Expression>),
    Map(Vec<(Expression, Expression)>),
    Set(Vec<Expression>),
    
    // 条件表达式
    Case {
        conditions: Vec<(Expression, Expression)>, // (条件, 结果)
        default: Option<Box<Expression>>,
    },
    
    // 其他表达式类型...
}
```

### 3.3 操作枚举
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
    Increment,
    Decrement,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    // 算术运算
    Add, Sub, Mul, Div, Mod,
    // 关系运算
    Eq, Ne, Lt, Le, Gt, Ge,
    // 逻辑运算
    And, Or, Xor,
    // 其他运算
    In, NotIn, Subscript, Attribute,
}
```

## 4. 评估系统设计

### 4.1 表达式上下文
```rust
pub trait ExpressionContext {
    fn get_variable(&self, name: &str) -> Result<Value, EvaluationError>;
    fn get_tag_property(&self, tag: &str, property: &str) -> Result<Value, EvaluationError>;
    fn get_edge_property(&self, edge: &str, property: &str) -> Result<Value, EvaluationError>;
    fn get_vertex(&self) -> Result<Value, EvaluationError>;
    fn get_edge(&self) -> Result<Value, EvaluationError>;
    // 其他获取方法...
}

// 默认实现示例
#[derive(Default)]
pub struct DefaultExpressionContext {
    variables: std::collections::HashMap<String, Value>,
    tag_properties: std::collections::HashMap<String, std::collections::HashMap<String, Value>>,
    // 其他字段...
}
```

### 4.2 评估实现
```rust
impl Expression {
    pub fn eval(&self, context: &dyn ExpressionContext) -> Result<Value, EvaluationError> {
        match self {
            Expression::Constant(value) => Ok(value.clone()),
            
            Expression::Unary { op, operand } => {
                let value = operand.eval(context)?;
                eval_unary(*op, value)
            },
            
            Expression::Binary { op, left, right } => {
                let left_val = left.eval(context)?;
                let right_val = right.eval(context)?;
                eval_binary(*op, left_val, right_val)
            },
            
            Expression::Variable { name } => context.get_variable(name),
            
            Expression::FunctionCall { name, args } => {
                let evaluated_args: Result<Vec<Value>, _> = 
                    args.iter().map(|arg| arg.eval(context)).collect();
                let args = evaluated_args?;
                call_function(name, args)
            },
            
            // 其他表达式类型的评估...
        }
    }
}
```

## 5. 迁移步骤

### 阶段 1: 基础结构实现
1. 实现 `Value` 枚举和相关方法
2. 实现基本的表达式类型（常量、一元、二元）
3. 实现基础的评估逻辑

### 阶段 2: 核心功能实现
1. 实现变量和属性访问表达式
2. 实现函数调用表达式
3. 实现容器表达式（列表、映射、集合）

### 阶段 3: 高级功能实现
1. 实现图特定表达式（顶点、边、路径）
2. 实现聚合表达式
3. 实现控制流表达式（条件、列表推导等）

### 阶段 4: 优化和测试
1. 性能优化和基准测试
2. 全面的单元测试
3. 集成测试

## 6. 预期改进

### 6.1 类型安全
- 在编译时捕获更多类型错误
- 更严格的类型检查

### 6.2 内存安全性
- 消除内存泄漏和悬空指针风险
- 零成本的内存管理

### 6.3 并发安全性
- 原生的线程安全保证
- 更容易实现并发表达式评估

### 6.4 维护性
- 更清晰的代码结构
- 更容易理解和修改

## 7. 风险和挑战

### 7.1 性能风险
- Rust 的零成本抽象可能不如原始 C++ 实现优化
- 需要仔细基准测试性能

### 7.2 迁移复杂性
- 所有表达式类型都需要重新实现
- 需要确保与现有查询引擎的兼容性

### 7.3 学习曲线
- 团队需要适应 Rust 的所有权和借用系统
- 需要时间来掌握 Rust 的惯用法

## 8. 结论

此迁移方案提供了一个全面的方法，将 NebulaGraph 的表达式系统迁移到 Rust。通过利用 Rust 的语言特性，我们可以构建一个更安全、更高效、更易维护的表达式评估系统。虽然迁移过程具有挑战性，但长期收益（类型安全、内存安全、并发安全）使得这个投资是值得的。