# AST 模块 - 彻底重构版本

## 概述

本模块实现了基于 trait 的抽象语法树（AST）设计，采用访问者模式，具有更好的扩展性、类型安全性和可维护性。相比之前的枚举实现，新的架构提供了更灵活的节点类型系统和更强大的分析能力。

## 架构设计

### 核心特性

1. **基于 Trait 的设计**：所有 AST 节点都实现统一的 trait 接口
2. **访问者模式**：支持遍历、转换和分析操作
3. **类型安全**：编译时类型检查，减少运行时错误
4. **扩展性强**：易于添加新的节点类型和功能
5. **内存高效**：使用 Box<dyn Trait> 避免不必要的内存分配

### 模块结构

```
src/query/parser/ast/
├── mod.rs              # 模块入口和公共接口
├── node.rs             # 基础 AST 节点实现
├── expression.rs       # 表达式 AST 定义
├── statement.rs        # 语句 AST 定义
├── pattern.rs          # 模式 AST 定义（图模式匹配）
├── types.rs            # 类型定义和辅助结构
├── visitor.rs          # 访问者模式实现
├── builder.rs          # AST 构建器
├── compat.rs           # 兼容性适配层
├── span.rs             # 位置信息
└── tests.rs            # 测试用例
```

## 核心 Trait

### AstNode Trait

所有 AST 节点的基类 trait：

```rust
pub trait AstNode: std::fmt::Debug + Clone + PartialEq {
    fn span(&self) -> Span;
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result;
    fn node_type(&self) -> &'static str;
    fn to_string(&self) -> String;
    fn clone_box(&self) -> Box<dyn AstNode>;
}
```

### Expression Trait

所有表达式节点的 trait：

```rust
pub trait Expression: AstNode {
    fn expr_type(&self) -> ExpressionType;
    fn is_constant(&self) -> bool;
    fn children(&self) -> Vec<&dyn Expression>;
}
```

### Statement Trait

所有语句节点的 trait：

```rust
pub trait Statement: AstNode {
    fn stmt_type(&self) -> StatementType;
    fn children(&self) -> Vec<&dyn AstNode>;
}
```

### Pattern Trait

所有模式节点的 trait：

```rust
pub trait Pattern: AstNode {
    fn pattern_type(&self) -> PatternType;
    fn variables(&self) -> Vec<&str>;
}
```

## 表达式类型

### 支持的表达式类型

1. **常量表达式** (`ConstantExpr`)：字面量值
2. **变量表达式** (`VariableExpr`)：变量引用
3. **二元表达式** (`BinaryExpr`)：二元操作符
4. **一元表达式** (`UnaryExpr`)：一元操作符
5. **函数调用** (`FunctionCallExpr`)：函数调用
6. **属性访问** (`PropertyAccessExpr`)：属性访问
7. **列表表达式** (`ListExpr`)：列表字面量
8. **映射表达式** (`MapExpr`)：映射字面量
9. **CASE 表达式** (`CaseExpr`)：条件表达式
10. **下标表达式** (`SubscriptExpr`)：数组/映射访问
11. **谓词表达式** (`PredicateExpr`)：列表谓词

### 操作符类型

#### 二元操作符 (`BinaryOp`)
- 算术：`Add`, `Sub`, `Mul`, `Div`, `Mod`, `Exp`
- 逻辑：`And`, `Or`, `Xor`
- 关系：`Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge`
- 字符串：`Regex`, `In`, `NotIn`, `Contains`, `StartsWith`, `EndsWith`

#### 一元操作符 (`UnaryOp`)
- `Not`, `Plus`, `Minus`
- `IsNull`, `IsNotNull`, `IsEmpty`, `IsNotEmpty`

#### 谓词类型 (`PredicateType`)
- `All`, `Any`, `Single`, `None`, `Exists`

## 语句类型

### 支持的语句类型

1. **查询语句** (`QueryStatement`)：多语句查询
2. **创建语句** (`CreateStatement`)：创建节点/边/标签/索引
3. **匹配语句** (`MatchStatement`)：图模式匹配
4. **删除语句** (`DeleteStatement`)：删除节点/边
5. **更新语句** (`UpdateStatement`)：更新节点/边属性
6. **GO 语句** (`GoStatement`)：图遍历
7. **FETCH 语句** (`FetchStatement`)：获取属性
8. **USE 语句** (`UseStatement`)：切换空间
9. **SHOW 语句** (`ShowStatement`)：显示信息
10. **EXPLAIN 语句** (`ExplainStatement`)：查询解释

## 模式类型

### 支持的模式类型

1. **节点模式** (`NodePattern`)：节点模式匹配
2. **边模式** (`EdgePattern`)：边模式匹配
3. **路径模式** (`PathPattern`)：复杂路径模式
4. **变量模式** (`VariablePattern`)：变量引用

### 路径元素类型

- `Node`：节点
- `Edge`：边
- `Alternative`：替代模式
- `Optional`：可选模式
- `Repeated`：重复模式

## 访问者模式

### 默认访问者 (`DefaultVisitor`)

提供基础的 AST 遍历功能，递归访问所有子节点。

### 类型检查访问者 (`TypeChecker`)

检查表达式类型兼容性，报告类型错误和警告。

### 语义分析访问者 (`SemanticAnalyzer`)

进行语义分析，检查变量定义、作用域等。

### AST 转换器 (`AstTransformer`)

执行 AST 优化和转换，如常量折叠。

### AST 格式化器 (`AstFormatter`)

生成格式化的 AST 字符串表示，用于调试和显示。

## AST 构建器

### AstBuilder

提供流畅的 API 来构建复杂的 AST 结构：

```rust
let builder = AstBuilder::new(span);
let expr = builder.binary(
    builder.constant(Value::Int(5)),
    BinaryOp::Add,
    builder.constant(Value::Int(3))
);
```

### ExpressionBuilder

专门的表达式构建器，提供便捷的操作符方法：

```rust
let builder = ExpressionBuilder::new(span);
let expr = builder.add(left_expr, right_expr);
```

### StatementBuilder

专门的语句构建器，简化语句创建：

```rust
let builder = StatementBuilder::new(span);
let stmt = builder.match_pattern(pattern);
```

## 兼容性适配

### 向后兼容

`compat.rs` 模块提供了与旧 AST 结构的兼容性：

- 类型别名保持 API 一致性
- 转换函数支持新旧类型互转
- 兼容性包装器简化迁移

### 迁移指南

1. **逐步迁移**：先使用兼容性层，再逐步替换
2. **测试验证**：确保功能正确性
3. **性能优化**：利用新架构的优势

## 使用示例

### 构建简单表达式

```rust
use crate::query::parser::ast::*;

let span = Span::default();
let builder = AstBuilder::new(span);

// 构建：x + 5
let expr = builder.binary(
    builder.variable("x"),
    BinaryOp::Add,
    builder.constant(Value::Int(5))
);
```

### 构建复杂查询

```rust
use crate::query::parser::ast::*;

let span = Span::default();
let builder = AstBuilder::new(span);

// 构建：MATCH (n:Person) WHERE n.age > 30 RETURN n.name
let pattern = builder.node_pattern(
    Some("n".to_string()),
    vec!["Person".to_string()]
);

let where_clause = builder.binary(
    builder.property_access(
        builder.variable("n"),
        "age"
    ),
    BinaryOp::Gt,
    builder.constant(Value::Int(30))
);

let return_item = builder.property_access(
    builder.variable("n"),
    "name"
);

let match_stmt = builder.match_(vec![pattern])
    .with_where_clause(where_clause)
    .with_return_clause(ReturnClause {
        distinct: false,
        items: vec![ReturnItem::Expression(return_item, None)]
    });
```

### 使用访问者模式

```rust
use crate::query::parser::ast::visitor::*;

let mut type_checker = TypeChecker::new();
expr.accept(&mut type_checker);

if type_checker.has_errors() {
    for error in &type_checker.errors {
        println!("Type error: {}", error);
    }
}
```

## 性能考虑

### 内存优化

- 使用 `Box<dyn Trait>` 减少内存占用
- 避免不必要的克隆操作
- 利用 Rust 的零成本抽象

### 编译时优化

- trait 对象在编译时解析
- 内联小函数减少调用开销
- 模式匹配优化

## 扩展性

### 添加新表达式类型

1. 实现 `Expression` trait
2. 在 `ExpressionType` 枚举中添加新类型
3. 实现相应的访问者方法
4. 添加构建器支持

### 添加新语句类型

1. 实现 `Statement` trait
2. 在 `StatementType` 枚举中添加新类型
3. 实现相应的访问者方法
4. 添加构建器支持

### 添加新访问者

1. 实现 `Visitor` trait
2. 定义结果类型
3. 实现访问逻辑
4. 注册到访问者系统中

## 测试

运行所有测试：

```bash
cargo test -p graphdb --lib query::parser::ast::tests
```

## 未来改进

1. **性能优化**：进一步优化内存使用和访问速度
2. **错误恢复**：增强错误处理和恢复能力
3. **增量解析**：支持增量式 AST 构建
4. **序列化**：添加 AST 的序列化和反序列化支持
5. **可视化**：提供 AST 可视化工具

## 总结

新的 AST 架构提供了：

- **更好的类型安全**：编译时检查，减少运行时错误
- **更强的扩展性**：易于添加新功能和节点类型
- **更清晰的架构**：职责分离，代码更易维护
- **更丰富的功能**：访问者模式支持复杂的分析和转换
- **更好的性能**：内存优化和零成本抽象

这个重构为整个 parser 模块提供了坚实的基础，支持未来更复杂的功能实现和性能优化。