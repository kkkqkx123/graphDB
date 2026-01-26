# Visitor 统一修改方案

## 一、现状分析

### 1.1 两个独立的 Visitor 实现

当前项目存在两个独立的表达式访问者实现：

#### 1.1.1 Parser 层 Visitor (`parser::ast::visitor::ExprVisitor`)

**文件位置**: `src/query/parser/ast/visitor.rs`

**特点**:
- 基于结构体的表达式变体（`ConstantExpression`, `VariableExpression` 等）
- 方法接收完整的结构体引用
- 提供 `DefaultVisitor` 实现用于遍历
- 仅支持表达式访问，不包含状态管理

**主要方法**:
```rust
pub trait ExprVisitor {
    type Result;
    fn visit_expression(&mut self, expression: &Expression) -> Self::Result;
    fn visit_constant(&mut self, expression: &ConstantExpression) -> Self::Result;
    fn visit_variable(&mut self, expression: &VariableExpression) -> Self::Result;
    fn visit_binary(&mut self, expression: &BinaryExpression) -> Self::Result;
    // ... 其他变体访问方法
}
```

#### 1.1.2 Core 层 Visitor (`core::expression_visitor::ExpressionVisitor`)

**文件位置**: `src/core/expression_visitor.rs`

**特点**:
- 基于枚举的表达式变体（`Literal`, `Variable`, `Binary` 等）
- 方法接收解构后的字段引用
- 包含完整的 `ExpressionVisitorState` 状态管理
- 提供丰富的扩展 trait（深度优先遍历、转换器等）
- 完善的错误处理机制

**主要方法**:
```rust
pub trait ExpressionVisitor: Debug + Send + Sync {
    type Result;
    fn visit_expression(&mut self, expression: &Expression) -> Self::Result;
    fn visit_literal(&mut self, value: &Value) -> Self::Result;
    fn visit_variable(&mut self, name: &str) -> Self::Result;
    fn visit_binary(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression) -> Self::Result;
    // ... 其他变体访问方法
}
```

### 1.2 变体映射关系

两个 Visitor 的表达式变体映射：

| Parser Expression (struct) | Core Expression (enum) | 统一后的变体 |
|---------------------------|----------------------|-------------|
| `ConstantExpression` | `Literal(Value)` | `Literal` |
| `VariableExpression` | `Variable(String)` | `Variable` |
| `BinaryExpression` | `Binary { left, op, right }` | `Binary` |
| `UnaryExpression` | `Unary { op, operand }` | `Unary` |
| `FunctionCallExpression` | `Function { name, args }` | `Function` |
| `PropertyAccessExpression` | `Property { object, property }` | `Property` |
| `ListExpression` | `List(Vec<Expression>)` | `List` |
| `MapExpression` | `Map(Vec<(String, Expression)>)` | `Map` |
| `CaseExpression` | `Case { conditions, default }` | `Case` |
| `SubscriptExpression` | `Subscript { collection, index }` | `Subscript` |
| `TypeCastExpression` | `TypeCast { expression, target_type }` | `TypeCast` |
| `RangeExpression` | `Range { collection, start, end }` | `Range` |
| `PathExpression` | `Path(Vec<Expression>)` | `Path` |
| `LabelExpression` | `Label(String)` | `Label` |

### 1.3 方法签名对比

| Parser 方法签名 | Core 方法签名 | 统一方法签名 |
|---------------|--------------|-------------|
| `visit_constant(&ConstantExpression)` | `visit_literal(&Value)` | `visit_literal(&Value)` |
| `visit_variable(&VariableExpression)` | `visit_variable(&str)` | `visit_variable(&str)` |
| `visit_binary(&BinaryExpression)` | `visit_binary(&Expression, &BinaryOperator, &Expression)` | `visit_binary(&Expression, &BinaryOperator, &Expression)` |
| `visit_function_call(&FunctionCallExpression)` | `visit_function(&str, &[Expression])` | `visit_function(&str, &[Expression])` |
| `visit_property_access(&PropertyAccessExpression)` | `visit_property(&Expression, &str)` | `visit_property(&Expression, &str)` |
| `visit_list(&ListExpression)` | `visit_list(&[Expression])` | `visit_list(&[Expression])` |
| `visit_map(&MapExpression)` | `visit_map(&[(String, Expression)])` | `visit_map(&[(String, Expression)])` |
| `visit_case(&CaseExpression)` | `visit_case(&[(Expression, Expression)], &Option<Box<Expression>>)` | `visit_case(&[(Expression, Expression)], Option<&Expression>)` |
| `visit_subscript(&SubscriptExpression)` | `visit_subscript(&Expression, &Expression)` | `visit_subscript(&Expression, &Expression)` |
| `visit_type_cast(&TypeCastExpression)` | `visit_type_cast(&Expression, &DataType)` | `visit_type_cast(&Expression, &DataType)` |
| `visit_range(&RangeExpression)` | `visit_range(&Expression, &Option<Box<Expression>>, &Option<Box<Expression>>)` | `visit_range(&Expression, Option<&Expression>, Option<&Expression>)` |
| `visit_path(&PathExpression)` | `visit_path(&[Expression])` | `visit_path(&[Expression])` |
| `visit_label(&LabelExpression)` | `visit_label(&str)` | `visit_label(&str)` |

## 二、统一 Visitor 设计

### 2.1 设计目标

1. **统一接口**: 创建一个统一的 `ExpressionVisitor` trait，兼容 Core 层的 `core::types::Expression`
2. **状态管理**: 保留 Core 层 Visitor 的状态管理功能
3. **错误处理**: 保留 Core 层 Visitor 的错误处理机制
4. **扩展支持**: 保留 Core 层 Visitor 的扩展 trait（遍历、转换等）
5. **向后兼容**: 提供适配器模式支持现有 Parser 层代码

### 2.2 统一 Visitor 接口设计

```rust
// src/core/types/expression/visitor.rs

use crate::core::types::expression::{DataType, Expression};
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;
use std::collections::HashMap;

/// 统一的表达式访问者 trait
///
/// 基于 `core::types::Expression` 的统一访问者接口，
/// 提供完整的表达式遍历、转换和状态管理功能。
pub trait ExpressionVisitor: Send + Sync {
    /// 访问者结果类型
    type Result;

    /// 主入口点 - 访问表达式
    fn visit_expression(&mut self, expression: &Expression) -> Self::Result {
        match expression {
            Expression::Literal(value) => self.visit_literal(value),
            Expression::Variable(name) => self.visit_variable(name),
            Expression::Property { object, property } => self.visit_property(object, property),
            Expression::Binary { left, op, right } => self.visit_binary(left, op, right),
            Expression::Unary { op, operand } => self.visit_unary(op, operand),
            Expression::Function { name, args } => self.visit_function(name, args),
            Expression::Aggregate { func, arg, distinct } => {
                self.visit_aggregate(func, arg, *distinct)
            }
            Expression::List(items) => self.visit_list(items),
            Expression::Map(pairs) => self.visit_map(pairs),
            Expression::Case { conditions, default } => self.visit_case(conditions, default.as_deref()),
            Expression::TypeCast { expression, target_type } => {
                self.visit_type_cast(expression, target_type)
            }
            Expression::Subscript { collection, index } => {
                self.visit_subscript(collection, index)
            }
            Expression::Range { collection, start, end } => {
                self.visit_range(collection, start.as_deref(), end.as_deref())
            }
            Expression::Path(items) => self.visit_path(items),
            Expression::Label(name) => self.visit_label(name),
        }
    }

    /// 访问字面量
    fn visit_literal(&mut self, value: &Value) -> Self::Result;

    /// 访问变量
    fn visit_variable(&mut self, name: &str) -> Self::Result;

    /// 访问属性访问
    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result;

    /// 访问二元运算
    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result;

    /// 访问一元运算
    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Self::Result;

    /// 访问函数调用
    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result;

    /// 访问聚合函数
    fn visit_aggregate(
        &mut self,
        func: &AggregateFunction,
        arg: &Expression,
        distinct: bool,
    ) -> Self::Result;

    /// 访问列表
    fn visit_list(&mut self, items: &[Expression]) -> Self::Result;

    /// 访问映射
    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result;

    /// 访问 CASE 表达式
    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result;

    /// 访问类型转换
    fn visit_type_cast(&mut self, expression: &Expression, target_type: &DataType) -> Self::Result;

    /// 访问下标访问
    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result;

    /// 访问范围表达式
    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) -> Self::Result;

    /// 访问路径表达式
    fn visit_path(&mut self, items: &[Expression]) -> Self::Result;

    /// 访问标签表达式
    fn visit_label(&mut self, name: &str) -> Self::Result;
}

/// 表达式访问者状态
#[derive(Debug, Clone)]
pub struct ExpressionVisitorState {
    /// 是否继续访问
    pub continue_visiting: bool,
    /// 当前访问深度
    pub depth: usize,
    /// 最大达到的深度
    pub max_depth_reached: usize,
    /// 访问计数
    pub visit_count: usize,
    /// 最大深度限制
    pub max_depth: Option<usize>,
    /// 自定义状态数据
    pub custom_data: HashMap<String, Value>,
}

impl ExpressionVisitorState {
    /// 创建新的访问者状态
    pub fn new() -> Self {
        Self {
            continue_visiting: true,
            depth: 0,
            max_depth_reached: 0,
            visit_count: 0,
            max_depth: None,
            custom_data: HashMap::new(),
        }
    }

    /// 增加访问深度
    pub fn increment_depth(&mut self) {
        self.depth += 1;
        self.max_depth_reached = self.max_depth_reached.max(self.depth);
    }

    /// 减少访问深度
    pub fn decrement_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// 增加访问计数
    pub fn increment_visit_count(&mut self) {
        self.visit_count += 1;
    }

    /// 检查是否超过最大深度
    pub fn exceeds_max_depth(&self) -> bool {
        self.max_depth.map_or(false, |max| self.depth > max)
    }
}

impl Default for ExpressionVisitorState {
    fn default() -> Self {
        Self::new()
    }
}

/// 表达式访问者结果类型
pub type VisitorResult<T> = Result<T, VisitorError>;

/// 表达式访问者错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisitorError {
    /// 超过最大深度限制
    MaxDepthExceeded,
    /// 访问被停止
    VisitationStopped,
    /// 类型不匹配
    TypeMismatch(String),
    /// 自定义错误
    Custom(String),
}

impl std::fmt::Display for VisitorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisitorError::MaxDepthExceeded => write!(f, "超过最大深度限制"),
            VisitorError::VisitationStopped => write!(f, "访问被停止"),
            VisitorError::TypeMismatch(msg) => write!(f, "类型不匹配: {}", msg),
            VisitorError::Custom(msg) => write!(f, "自定义错误: {}", msg),
        }
    }
}

impl std::error::Error for VisitorError {}
```

### 2.3 扩展 Trait 设计

```rust
/// 深度优先遍历器 trait
pub trait ExpressionDepthFirstVisitor: ExpressionVisitor {
    /// 访问子表达式
    fn visit_children(&mut self, expression: &Expression) -> Self::Result;

    /// 默认结果
    fn default_result(&self) -> Self::Result;
}

/// 表达式转换器 trait
pub trait ExpressionTransformer: ExpressionVisitor<Result = Expression> {
    /// 转换表达式
    fn transform(&mut self, expression: &Expression) -> Expression {
        self.visit_expression(expression)
    }

    /// 转换子表达式
    fn transform_children(&mut self, expression: &Expression) -> Expression;
}

/// 访问者辅助 trait
pub trait ExpressionVisitorExt: ExpressionVisitor {
    /// 获取表达式树的最大深度
    fn max_depth(&mut self, expression: &Expression) -> usize;

    /// 获取表达式树中的所有变量名
    fn collect_variables(&mut self, expression: &Expression) -> Vec<String>;
}
```

### 2.4 Parser 适配器设计

为保持向后兼容，创建 Parser 层到 Core 层的适配器：

```rust
/// Parser Expression 到 Core Expression 的适配器
pub struct ParserExprAdapter<'a> {
    parser_expr: &'a parser::ast::Expression,
}

impl<'a> ParserExprAdapter<'a> {
    pub fn new(parser_expr: &'a parser::ast::Expression) -> Self {
        Self { parser_expr }
    }

    /// 将 Parser Expression 转换为 Core Expression
    pub fn to_core_expression(&self) -> Expression {
        use parser::ast::Expression::*;
        match self.parser_expr {
            Constant(e) => Expression::Literal(e.value.clone()),
            Variable(e) => Expression::Variable(e.name.clone()),
            Binary(e) => Expression::Binary {
                left: Box::new(Self::new(&e.left).to_core_expression()),
                op: e.op.clone(),
                right: Box::new(Self::new(&e.right).to_core_expression()),
            },
            // ... 其他变体转换
            _ => unimplemented!("Unsupported expression variant"),
        }
    }
}
```

## 三、实施计划

### 3.1 阶段一：创建统一 Visitor 接口（已完成）

- [x] 分析现有两个 Visitor 实现
- [x] 设计统一 Visitor 接口
- [x] 创建 `src/core/types/expression/visitor.rs`
- [x] 移动 `ExpressionVisitorState` 和 `VisitorError` 到新模块
- [x] 保留 Core 层扩展 trait（深度优先遍历、转换器等）

### 3.2 阶段二：更新 Core 层 Visitor 引用

**任务**:
1. 更新 `src/core/expression_visitor.rs` 使用新的统一接口
2. 重构扩展 trait 实现
3. 运行测试验证功能

### 3.3 阶段三：创建 Parser 适配器

**任务**:
1. 在 `src/query/parser/ast/` 创建适配器模块
2. 实现 `ParserExprAdapter` 结构体
3. 提供便捷的转换方法
4. 更新现有 Parser 层代码使用适配器

### 3.4 阶段四：清理旧代码

**任务**:
1. 评估是否需要保留 `parser::ast::visitor::ExprVisitor`
2. 如果不需要，标记为废弃（deprecated）
3. 更新项目文档

## 四、文件变更清单

### 4.1 新建文件

| 文件路径 | 描述 |
|---------|------|
| `src/core/types/expression/visitor.rs` | 统一的 Visitor 接口 |

### 4.2 修改文件

| 文件路径 | 修改内容 |
|---------|----------|
| `src/core/expression_visitor.rs` | 重构为使用统一接口，移除重复定义 |
| `src/core/types/expression/mod.rs` | 导出新的 visitor 模块 |

### 4.3 可能废弃的文件

| 文件路径 | 建议操作 |
|---------|----------|
| `src/query/parser/ast/visitor.rs` | 保留用于向后兼容，或标记为废弃 |

## 五、风险评估

### 5.1 高风险项

1. **API 变更影响范围大**
   - 影响所有实现 `ExpressionVisitor` 的代码
   - 需要更新多个文件

2. **方法签名变更**
   - `visit_case` 和 `visit_range` 的参数从 `&Option<Box<Expression>>` 改为 `Option<&Expression>`
   - 需要更新所有实现者

### 5.2 缓解措施

1. **渐进式迁移**
   - 先创建新接口，再逐步迁移
   - 保持向后兼容性

2. **充分测试**
   - 为每个变更编写测试用例
   - 运行完整测试套件验证

## 六、验收标准

1. [ ] 统一 Visitor 接口创建完成
2. [ ] 所有现有测试通过
3. [ ] Core 层代码迁移完成
4. [ ] Parser 适配器可用
5. [ ] 文档更新完成

## 七、版本信息

| 版本 | 日期 | 作者 | 描述 |
|-----|------|-----|-----|
| 1.0 | 2025-01-26 | GraphDB Team | 初始版本 |
