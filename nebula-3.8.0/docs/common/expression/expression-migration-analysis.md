# NebulaGraph 表达式系统迁移分析

## 概述

本文档分析了 NebulaGraph 3.8.0 版本中的表达式系统，该系统位于 `src/common/expression` 目录。该系统是查询引擎的核心组件，负责处理各种表达式类型的评估、序列化和反序列化。

## 表达式类型概览

### 1. 基础表达式类型

#### ConstantExpression
- **功能**：表示常量值，如数字、字符串、布尔值、null 等
- **关键方法**：`eval()`、`toString()`、`clone()`
- **用途**：用于表示查询中的字面常量值

#### UnaryExpression
- **功能**：表示一元运算符，如正号、负号、逻辑非、递增、递减等
- **操作类型**：`kUnaryPlus`、`kUnaryNegate`、`kUnaryNot`、`kUnaryIncr`、`kUnaryDecr`

#### BinaryExpression（基类）
- **功能**：所有二元运算表达式的基类
- **子类包括**：
  - ArithmeticExpression：算术运算（+、-、*、/、%）
  - RelationalExpression：关系运算（==、!=、<、>、<=、>=、IN、REG 等）
  - LogicalExpression：逻辑运算（AND、OR、XOR）
  - AttributeExpression：属性访问（obj.attr）
  - SubscriptExpression：下标运算（array[index]、map[key]）

### 2. 属性表达式类型

#### PropertyExpression（基类）
- **功能**：处理属性访问的抽象基类
- **子类包括**：
  - EdgePropertyExpression：边属性访问（edge_name.prop）
  - TagPropertyExpression：标签属性访问（tag_name.prop）
  - LabelTagPropertyExpression：标签-标签属性访问（label.tag_name.prop）
  - InputPropertyExpression：输入属性访问（$-.prop）
  - VariablePropertyExpression：变量属性访问（$var.prop）
  - SourcePropertyExpression：源顶点属性访问（$^.tag.prop）
  - DestPropertyExpression：目标顶点属性访问（$$.tag.prop）
  - EdgeSrcIdExpression：边源顶点ID（EdgeName._src）
  - EdgeTypeExpression：边类型（EdgeName._type）
  - EdgeRankExpression：边排名（EdgeName._rank）
  - EdgeDstIdExpression：边目标顶点ID（EdgeName._dst）

### 3. 容器表达式类型

#### ContainerExpression（基类）
- **功能**：处理容器类型表达式的基类
- **子类包括**：
  - ListExpression：列表表达式
  - SetExpression：集合表达式
  - MapExpression：映射表达式

### 4. 函数和聚合表达式类型

#### FunctionCallExpression
- **功能**：表示函数调用，包含函数名和参数列表
- **关键组件**：
  - ArgumentList：参数列表管理
  - FunctionManager：函数管理器，缓存函数指针

#### AggregateExpression
- **功能**：聚合函数表达式（COUNT、SUM、AVG 等）

### 5. 控制流和特殊表达式类型

#### CaseExpression
- **功能**：实现 CASE-WHEN-THEN-ELSE-END 逻辑

#### ListComprehensionExpression
- **功能**：列表推导式表达式

#### PredicateExpression
- **功能**：谓词表达式，用于过滤操作

#### ReduceExpression
- **功能**：归约表达式操作

#### MatchPathPatternExpression
- **功能**：路径模式匹配表达式

#### TextSearchExpression
- **功能**：文本搜索表达式（ESQUERY）

### 6. 图特定表达式类型

#### VertexExpression / EdgeExpression
- **功能**：表示顶点和边值

#### PathBuildExpression
- **功能**：路径构建表达式，用于图遍历

#### LabelExpression
- **功能**：图查询中的标签表达式

#### UUIDExpression
- **功能**：UUID 生成表达式

#### TypeCastingExpression
- **功能**：类型转换表达式

#### ColumnExpression
- **功能**：查询中的列引用

#### VariableExpression
- **功能**：变量引用表达式

## 关键系统组件

### 表达式评估系统
- **ExpressionContext**：表达式评估时的上下文接口
  - 变量访问：`getVar()`, `setVar()`
  - 属性访问：`getTagProp()`, `getEdgeProp()`, `getSrcProp()`, `getDstProp()`
  - 输入访问：`getInputProp()`
  - 其他：`getVertex()`, `getEdge()`, `getColumn()`

### 访问者模式（Visitor Pattern）
- **ExprVisitor**：抽象访问者接口，定义对各种表达式的访问方法
- **ExprVisitorImpl**：访问者接口的默认实现

### 序列化系统
- **Encoder/Decoder**：用于表达式的编码和解码
- **encode/decode**：静态方法用于表达式的序列化和反序列化

## Rust 迁移考虑因素

### 1. 内存管理
- 原 C++ 实现使用 ObjectPool 进行内存管理
- Rust 实现应利用所有权系统，避免手动内存管理
- 使用 Box<T>、Vec<T>、Rc<T>、Arc<T> 等智能指针

### 2. 类型系统
- 原 C++ 实现使用运行时类型检查（kind() 方法）
- Rust 实现应使用枚举（enum）和模式匹配（pattern matching）
- 利用 Rust 的类型系统在编译时捕获更多错误

### 3. 继承 vs 枚举
- 原 C++ 实现使用继承层次结构
- Rust 实现应使用枚举变体（enum variants）来表示不同的表达式类型
- 使用 trait 来实现共享行为

### 4. 访问者模式
- 原 C++ 实现使用虚函数和访问者模式
- Rust 实现可以使用模式匹配替代，或者实现类似于 `Visit` trait

### 5. 错误处理
- 原 C++ 实现使用 NullType 来表示错误
- Rust 实现应使用 Result<T, E> 和 Option<T> 类型进行错误处理

## 迁移建议

### 1. 基础结构设计
```rust
#[derive(Debug, Clone)]
pub enum Expression {
    Constant(Value),
    Arithmetic { 
        left: Box<Expression>, 
        op: ArithmeticOp, 
        right: Box<Expression> 
    },
    Relational { 
        left: Box<Expression>, 
        op: RelationalOp, 
        right: Box<Expression> 
    },
    // ... 其他变体
}

#[derive(Debug, Clone)]
pub enum Value {
    Null(NullType),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    // ... 其他类型
}
```

### 2. 评估方法
```rust
impl Expression {
    pub fn eval(&self, context: &dyn ExpressionContext) -> Result<Value, EvaluationError> {
        match self {
            Expression::Constant(value) => Ok(value.clone()),
            Expression::Arithmetic { left, op, right } => {
                eval_arithmetic(&**left, *op, &**right, context)
            },
            // ... 其他情况
        }
    }
}
```

### 3. 上下文接口
```rust
pub trait ExpressionContext {
    fn get_variable(&self, var: &str) -> Result<Value, ContextError>;
    fn get_tag_property(&self, tag: &str, prop: &str) -> Result<Value, ContextError>;
    // ... 其他方法
}
```

## 总结

NebulaGraph 的表达式系统是一个复杂的层次结构，支持 SQL 和图查询语言中的各种表达式类型。在迁移到 Rust 时，应利用 Rust 的类型安全、内存安全和并发安全特性，使用枚举和模式匹配替代继承层次结构，并利用 Rust 的错误处理机制。

关键迁移点包括：
1. 从继承层次结构迁移到枚举变体
2. 从手动内存管理迁移到所有权系统
3. 从虚函数分派迁移到模式匹配
4. 从运行时类型检查迁移到编译时类型检查
5. 从手动错误处理迁移到 Result/Option 类型系统