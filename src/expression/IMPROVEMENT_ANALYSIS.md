# src/expression 目录改进分析

## 当前问题

### 1. 冗余的 types.rs 模块

`src\expression\types.rs` 只是一个简单的重新导出模块，没有提供任何额外功能：

```rust
pub use crate::core::types::expression::{DataType, Expression, ExpressionType, LiteralValue};
```

这种设计造成了：
- 不必要的间接层
- 增加了代码复杂度
- `mod.rs` 已经重新导出了这些类型，`types.rs` 显得冗余

### 2. 引用路径不一致

当前代码中存在两种引用方式：
- `use crate::expression::types::{DataType, Expression, ExpressionType, LiteralValue}`
- `use crate::expression::{Expression, ExpressionType, ExpressionVisitor, LiteralValue}`

这种不一致性增加了维护成本。

## 改进方案

### 方案一：删除 types.rs（推荐）

**优点**：
- 简化模块结构
- 减少间接层
- 统一引用路径

**步骤**：
1. 删除 `src\expression\types.rs`
2. 更新 `src\expression\mod.rs`，直接从 `core::types::expression` 重新导出
3. 更新所有引用 `types` 模块的文件

**修改后的 `src\expression\mod.rs`**：
```rust
pub mod aggregate_functions;
pub mod storage;
pub mod visitor;

// 重新导出expression模块的访问器
pub use visitor::{ExpressionAcceptor, ExpressionVisitor, ExpressionDepthFirstVisitor, ExpressionTransformer, ExpressionTypeFilter};

// Re-export Core operators directly - no more wrapper types
pub use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};

// Re-export Core expression types
pub use crate::core::types::expression::{DataType, Expression, ExpressionType, LiteralValue};

// Re-export Core evaluator
pub use crate::core::evaluator::ExpressionEvaluator;

// Re-export storage module types for convenience
pub use storage::{ColumnDef, FieldDef, FieldType, RowReaderWrapper, Schema};
```

**修改后的 `src\expression\visitor.rs`**：
```rust
use crate::core::types::expression::{DataType, Expression, ExpressionType, LiteralValue};
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
```

### 方案二：扩展 types.rs 功能

如果保留 `types.rs`，应该为其添加实际功能：

**可能的扩展**：
1. 表达式构建器（Expression Builder）
2. 表达式验证器（Expression Validator）
3. 表达式简化器（Expression Simplifier）
4. 表达式类型推断（Type Inference）

**示例**：
```rust
pub use crate::core::types::expression::{DataType, Expression, ExpressionType, LiteralValue};

/// 表达式构建器
pub struct ExpressionBuilder {
    // ...
}

impl ExpressionBuilder {
    pub fn new() -> Self { ... }
    pub fn literal(value: impl Into<LiteralValue>) -> Expression { ... }
    pub fn variable(name: impl Into<String>) -> Expression { ... }
    pub fn binary(left: Expression, op: BinaryOperator, right: Expression) -> Expression { ... }
    // ...
}

/// 表达式验证器
pub struct ExpressionValidator {
    // ...
}

impl ExpressionValidator {
    pub fn validate(&self, expr: &Expression) -> Result<(), ValidationError> { ... }
}
```

## 推荐方案

**推荐方案一**（删除 `types.rs`），理由：
1. 当前 `types.rs` 没有提供任何额外功能
2. `Expression` 类型已经定义在 `core::types::expression`，不需要在 `expression` 模块中重复定义
3. 直接重新导出可以避免不必要的间接层
4. 统一引用路径，减少维护成本

## 迁移步骤

1. 删除 `src\expression\types.rs`
2. 更新 `src\expression\mod.rs` 中的重新导出语句
3. 更新 `src\expression\visitor.rs` 中的引用语句
4. 运行 `cargo check` 确保没有编译错误
5. 运行测试确保功能正常

## 注意事项

1. 确保所有使用 `use crate::expression::types::...` 的文件都已更新
2. 检查是否有其他模块依赖 `types.rs` 的导出
3. 更新相关文档和注释
