# 表达式类型简化修改计划

## 修改目标
移除Expression枚举中的冗余类型，统一使用核心表达式类型。

## 需要移除的表达式变体

### 第一阶段：移除重复的一元操作（9种）
从Expression枚举中移除以下变体，改用`Expression::Unary`：
- `UnaryPlus(Box<Expression>)` → 使用 `Unary(UnaryOperator::Plus, expr)`
- `UnaryNegate(Box<Expression>)` → 使用 `Unary(UnaryOperator::Minus, expr)`
- `UnaryNot(Box<Expression>)` → 使用 `Unary(UnaryOperator::Not, expr)`
- `UnaryIncr(Box<Expression>)` → 使用 `Unary(UnaryOperator::Increment, expr)`
- `UnaryDecr(Box<Expression>)` → 使用 `Unary(UnaryOperator::Decrement, expr)`
- `IsNull(Box<Expression>)` → 使用 `Unary(UnaryOperator::IsNull, expr)`
- `IsNotNull(Box<Expression>)` → 使用 `Unary(UnaryOperator::IsNotNull, expr)`
- `IsEmpty(Box<Expression>)` → 使用 `Unary(UnaryOperator::IsEmpty, expr)`
- `IsNotEmpty(Box<Expression>)` → 使用 `Unary(UnaryOperator::IsNotEmpty, expr)`

### 第二阶段：移除过度设计的属性访问（6种）
移除以下变体，改用通用的`Expression::Property`：
- `TagProperty { tag: String, prop: String }`
- `EdgeProperty { edge: String, prop: String }`
- `InputProperty(String)`
- `VariableProperty { var: String, prop: String }`
- `SourceProperty { tag: String, prop: String }`
- `DestinationProperty { tag: String, prop: String }`

### 第三阶段：移除缺乏实现的高级功能（6种）
- `ListComprehension { generator, condition }`
- `Predicate { list, condition }`
- `Reduce { list, var, initial, expr }`
- `ESQuery(String)`
- `UUID`
- `MatchPathPattern { path_alias, patterns }`

## 需要同步修改的ExpressionType变体
- 移除：TagProperty, EdgeProperty, InputProperty, VariableProperty, SourceProperty, DestinationProperty

## 需要修改的文件清单

### 核心文件（必须修改）
1. `src/core/types/expression.rs` - 主要修改目标
2. `src/core/types/mod.rs` - 重新导出

### 验证器（必须修改）
3. `src/query/validator/go_validator.rs` - 9处一元操作 + 6处属性访问 + 6处占位符
4. `src/query/validator/order_by_validator.rs` - 9处一元操作 + 6处属性访问 + 4处占位符
5. `src/query/validator/strategies/type_inference.rs` - 6处属性访问 + 3处占位符
6. `src/query/validator/strategies/aggregate_strategy.rs` - 9处一元操作
7. `src/query/validator/strategies/alias_strategy.rs` - 9处一元操作
8. `src/query/validator/strategies/expression_strategy.rs` - 5处一元操作

### 优化器（必须修改）
9. `src/query/optimizer/optimizer.rs` - 9处一元操作
10. `src/query/optimizer/plan_validator.rs` - 3处占位符
11. `src/query/optimizer/predicate_pushdown.rs` - 6处属性访问

### 访问者（必须修改）
12. `src/core/expression_visitor.rs` - 9处一元操作 + 6处属性访问 + 6处占位符
13. `src/query/visitor/find_visitor.rs` - 9处一元操作 + 6处属性访问 + 6处占位符
14. `src/query/visitor/extract_group_suite_visitor.rs` - 6处属性访问
15. `src/query/visitor/deduce_props_visitor.rs` - 3处属性访问

### 工具类（必须修改）
16. `src/core/expression_utils.rs` - 9处一元操作 + 6处属性访问 + 6处占位符

### 求值器（可能需要修改）
17. `src/expression/evaluator/expression_evaluator.rs` - 1处InputProperty

## 修改步骤

### 步骤1：修改核心expression.rs
- [ ] 移除9个一元操作变体
- [ ] 移除6个属性访问变体
- [ ] 移除6个占位符类型
- [ ] 更新children()方法
- [ ] 更新expression_type()方法
- [ ] 更新ExpressionType枚举

### 步骤2：修改核心访问者
- [ ] 修改expression_visitor.rs

### 步骤3：修改工具类
- [ ] 修改expression_utils.rs

### 步骤4：修改验证器
- [ ] 修改go_validator.rs
- [ ] 修改order_by_validator.rs
- [ ] 修改type_inference.rs
- [ ] 修改aggregate_strategy.rs
- [ ] 修改alias_strategy.rs
- [ ] 修改expression_strategy.rs

### 步骤5：修改优化器
- [ ] 修改optimizer.rs
- [ ] 修改plan_validator.rs
- [ ] 修改predicate_pushdown.rs

### 步骤6：修改访问者
- [ ] 修改find_visitor.rs
- [ ] 修改extract_group_suite_visitor.rs
- [ ] 修改deduce_props_visitor.rs

### 步骤7：修改求值器
- [ ] 修改expression_evaluator.rs

### 步骤8：类型检查
- [ ] 运行cargo check确保无错误

## 注意事项
1. 每完成一个文件的修改，运行cargo check确保编译通过
2. 如果遇到编译错误，及时修复
3. 保持代码风格一致
4. 不要引入新的警告
