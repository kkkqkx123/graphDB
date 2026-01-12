# FoldConstantExprVisitor 迁移分析

## NebulaGraph FoldConstantExprVisitor 功能概述

### 核心功能
1. **遍历表达式树**：使用访问者模式遍历各种类型的表达式
2. **判断可折叠性**：判断表达式是否可以被折叠为常量
3. **执行常量折叠**：计算可折叠表达式的值，并替换为常量表达式
4. **错误处理**：处理除零、溢出、类型错误等异常情况

### 支持的表达式类型
- 常量表达式 (ConstantExpression)
- 一元表达式 (UnaryExpression)
- 二元表达式 (ArithmeticExpression, RelationalExpression, LogicalExpression)
- 类型转换表达式 (TypeCastingExpression)
- 函数调用表达式 (FunctionCallExpression)
- 聚合表达式 (AggregateExpression)
- 容器表达式 (ListExpression, SetExpression, MapExpression)
- Case表达式 (CaseExpression)
- 属性访问表达式 (PropertyExpression, TagPropertyExpression, EdgePropertyExpression等)
- 路径构建表达式 (PathBuildExpression)
- 列表推导式 (ListComprehensionExpression)
- 归约表达式 (ReduceExpression)
- 下标表达式 (SubscriptExpression, SubscriptRangeExpression)
- 变量表达式 (VariableExpression, VersionedVariableExpression)

### 关键实现细节
```cpp
// 判断表达式是否为常量
bool isConstant(Expression *expr) const {
    return expr->kind() == Expression::Kind::kConstant;
}

// 折叠表达式
Expression *fold(Expression *expr) {
    QueryExpressionContext ctx;
    auto value = expr->eval(ctx(nullptr));
    // 处理各种错误情况
    return ConstantExpression::make(pool_, value);
}
```

## 新架构中的现有实现

### 1. CypherExpressionOptimizer
**位置**: `src/query/parser/cypher/expression_optimizer.rs`

**功能**: 提供简单的常量折叠功能

**限制**:
- 只针对Cypher AST，不适用于统一Expression类型
- 只实现了加法和乘法的常量折叠
- 没有完整的表达式类型覆盖
- 没有错误处理机制

**示例代码**:
```rust
// 只实现了加法和乘法的常量折叠
match (&*optimized_left, &*optimized_right, bin_expr.operator) {
    (
        Expression::Literal(Literal::Integer(left_val)),
        Expression::Literal(Literal::Integer(right_val)),
        BinaryOperator::Add
    ) => {
        Expression::Literal(Literal::Integer(left_val + right_val))
    },
    // ... 其他情况
}
```

### 2. ExpressionEvaluator
**位置**: `src/expression/evaluator/expression_evaluator.rs`

**功能**: 提供表达式求值功能

**限制**:
- 只能求值表达式，不能修改表达式树
- 没有自动折叠机制
- 需要外部调用者判断是否应该折叠

**示例代码**:
```rust
pub fn evaluate(
    &self,
    expr: &Expression,
    context: &mut dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    match expr {
        Expression::Literal(value) => Ok(value.clone()),
        Expression::Binary { left, op, right } => {
            let left_value = self.evaluate(left, context)?;
            let right_value = self.evaluate(right, context)?;
            self.eval_binary_operation(&left_value, op, &right_value)
        }
        // ...
    }
}
```

### 3. EvaluableExprVisitor
**位置**: `src/query/visitor/evaluable_expr_visitor.rs`

**功能**: 判断表达式是否可求值（是否为常量表达式）

**限制**:
- 只能判断，不能折叠
- 没有实际执行折叠操作

**示例代码**:
```rust
pub fn is_evaluable(&mut self, expr: &Expression) -> bool {
    self.evaluable = true;
    self.error = None;

    if let Err(e) = self.visit(expr) {
        self.evaluable = false;
        self.error = Some(e);
    }

    self.evaluable
}
```

## 功能对比分析

| 功能 | NebulaGraph | 新架构 | 状态 |
|------|-------------|--------|------|
| 遍历表达式树 | ✅ | ✅ (ExpressionVisitor) | 已实现 |
| 判断可折叠性 | ✅ | ✅ (EvaluableExprVisitor) | 已实现 |
| 执行常量折叠 | ✅ | ⚠️ (部分实现) | 部分实现 |
| 二元表达式折叠 | ✅ | ⚠️ (仅加法/乘法) | 需要完善 |
| 一元表达式折叠 | ✅ | ❌ | 未实现 |
| 函数调用折叠 | ✅ | ❌ | 未实现 |
| 容器表达式折叠 | ✅ | ❌ | 未实现 |
| Case表达式折叠 | ✅ | ❌ | 未实现 |
| 错误处理 | ✅ | ❌ | 未实现 |
| 表达式替换 | ✅ | ❌ | 未实现 |

## 迁移建议

### 方案1: 完整迁移
**优点**:
- 功能完整，与NebulaGraph保持一致
- 可以处理所有表达式类型
- 有完善的错误处理

**缺点**:
- 工作量大
- 需要实现所有表达式类型的折叠逻辑
- 需要处理Expression的可变性

**实现步骤**:
1. 创建新的`FoldConstantExprVisitor`，基于`ExpressionVisitor` trait
2. 实现所有表达式类型的折叠逻辑
3. 集成`ExpressionEvaluator`来计算表达式值
4. 实现错误处理机制
5. 添加测试用例

### 方案2: 渐进式迁移
**优点**:
- 可以分阶段实现
- 优先实现常用功能
- 降低风险

**缺点**:
- 功能不完整
- 可能需要多次迭代

**实现步骤**:
1. 第一阶段：实现二元表达式的常量折叠
2. 第二阶段：实现一元表达式和函数调用的折叠
3. 第三阶段：实现容器表达式和Case表达式的折叠
4. 第四阶段：添加错误处理和边界情况处理

### 方案3: 增强现有实现
**优点**:
- 利用现有代码
- 减少重复工作
- 保持架构一致性

**缺点**:
- 可能需要重构现有代码
- 需要协调多个模块

**实现步骤**:
1. 增强`CypherExpressionOptimizer`，支持更多表达式类型
2. 创建统一Expression版本的优化器
3. 在`ExpressionEvaluator`中添加折叠辅助方法
4. 创建`FoldConstantExprVisitor`，调用上述组件

## 推荐方案

**推荐方案3：增强现有实现**

理由：
1. 新架构已经有了`ExpressionEvaluator`可以计算表达式值
2. `EvaluableExprVisitor`可以判断表达式是否可折叠
3. 只需要创建一个协调器来整合这些功能
4. 可以逐步扩展功能，降低风险

## 实现计划

### Phase 1: 基础框架
- [ ] 创建`FoldConstantExprVisitor`结构体
- [ ] 实现`ExpressionVisitor` trait
- [ ] 集成`ExpressionEvaluator`和`EvaluableExprVisitor`
- [ ] 实现基本的二元表达式折叠

### Phase 2: 完整二元表达式
- [ ] 实现所有二元运算符的折叠
- [ ] 添加错误处理（除零、溢出等）
- [ ] 添加测试用例

### Phase 3: 一元和函数调用
- [ ] 实现一元表达式折叠
- [ ] 实现纯函数调用的折叠
- [ ] 添加函数纯度检查

### Phase 4: 容器和复杂表达式
- [ ] 实现List/Set/Map表达式折叠
- [ ] 实现Case表达式折叠
- [ ] 实现其他复杂表达式折叠

### Phase 5: 优化和测试
- [ ] 性能优化
- [ ] 完善测试覆盖
- [ ] 文档完善

## 技术细节

### 表达式可变性处理
由于Rust的借用规则，需要处理表达式的可变性：

```rust
// 方案1: 使用内部可变性
pub struct FoldConstantExprVisitor {
    evaluator: ExpressionEvaluator,
    evaluable_visitor: EvaluableExprVisitor,
}

impl ExpressionVisitor for FoldConstantExprVisitor {
    type Result = Result<Expression, String>;

    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        // 递归折叠子表达式
        let folded_left = self.visit(left)?;
        let folded_right = self.visit(right)?;

        // 判断是否可以折叠
        if self.evaluable_visitor.is_evaluable(&folded_left) &&
           self.evaluable_visitor.is_evaluable(&folded_right) {
            // 计算值并返回常量表达式
            let mut context = DefaultExpressionContext::new();
            let value = self.evaluator.evaluate(&folded_left, &mut context)?;
            Ok(Expression::Literal(value))
        } else {
            // 返回折叠后的子表达式
            Ok(Expression::Binary {
                left: Box::new(folded_left),
                op: op.clone(),
                right: Box::new(folded_right),
            })
        }
    }
}
```

### 错误处理
需要处理各种错误情况：

```rust
pub enum FoldError {
    DivisionByZero,
    Overflow,
    TypeError(String),
    InvalidOperation(String),
}

impl FoldConstantExprVisitor {
    fn fold_binary(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Result<Value, FoldError> {
        match op {
            BinaryOperator::Divide => {
                if right.is_zero() {
                    return Err(FoldError::DivisionByZero);
                }
                left.div(right).map_err(|e| FoldError::InvalidOperation(e))
            }
            // ... 其他运算符
        }
    }
}
```

## 结论

FoldConstantExprVisitor的功能在新架构中只是部分实现，需要迁移。推荐采用**方案3：增强现有实现**，通过整合现有的`ExpressionEvaluator`和`EvaluableExprVisitor`来创建一个完整的常量折叠访问器。

迁移应该分阶段进行，优先实现常用功能，逐步扩展到所有表达式类型。
