# Expression类型重构修改计划

## 修改概述
简化`Expression`枚举，移除冗余的变体，将多种属性访问统一为`Property`表达式，将一元操作统一为`Unary`表达式。

## 已完成的修改

### 1. 核心类型文件
- [x] `src/core/types/expression.rs` - 移除冗余变体
- [x] `src/core/expression_visitor.rs` - 移除对应的visit方法
- [x] `src/core/expression_utils.rs` - 移除使用已删除变体的函数

## 待修复的文件（按优先级排序）

### 第一优先级：核心查询模块
1. [ ] `src/query/visitor/vid_extract_visitor.rs` - 修复VariableProperty引用
2. [ ] `src/query/visitor/rewrite_visitor.rs` - 修复ExpressionType引用
3. [ ] `src/query/validator/go_validator.rs` - 修复一元操作和属性访问
4. [ ] `src/query/validator/order_by_validator.rs` - 修复属性访问

### 第二优先级：执行器模块
5. [ ] `src/query/executor/` - 修复各类执行器中的表达式处理

### 第三优先级：规划器模块
6. [ ] `src/query/planner/` - 修复规划器中的表达式处理

### 第四优先级：其他模块
7. [ ] `src/storage/` - 存储层
8. [ ] `src/common/` - 公共模块
9. [ ] `src/services/` - 服务层

## 具体修改内容

### 需要移除的Expression变体（已移除）
1. 一元操作变体（8种）：
   - `UnaryPlus` → 合并到 `Unary { op: UnaryOperator::Plus }`
   - `UnaryNegate` → 合并到 `Unary { op: UnaryOperator::Negate }`
   - `UnaryNot` → 合并到 `Unary { op: UnaryOperator::Not }`
   - `UnaryIncr` → 合并到 `Unary { op: UnaryOperator::Increment }`
   - `UnaryDecr` → 合并到 `Unary { op: UnaryOperator::Decrement }`
   - `IsNull` → 合并到 `Unary { op: UnaryOperator::IsNull }`
   - `IsNotNull` → 合并到 `Unary { op: UnaryOperator::IsNotNull }`
   - `IsEmpty` → 合并到 `Unary { op: UnaryOperator::IsEmpty }`
   - `IsNotEmpty` → 合并到 `Unary { op: UnaryOperator::IsNotEmpty }`

2. 属性访问变体（6种）：
   - `TagProperty` → 合并到 `Property`
   - `EdgeProperty` → 合并到 `Property`
   - `InputProperty` → 合并到 `Property`
   - `VariableProperty` → 合并到 `Property`
   - `SourceProperty` → 合并到 `Property`
   - `DestinationProperty` → 合并到 `Property`

3. 占位符变体（8种）：
   - `ListComprehension` → 移除
   - `Predicate` → 移除
   - `Reduce` → 移除
   - `MatchPathPattern` → 移除
   - `Extract` → 移除
   - `Find` → 移除
   - `ESQuery` → 移除
   - `UUID` → 移除

## 修改策略

### 阶段1：修复核心查询模块
1. 修复`vid_extract_visitor.rs`
   - 将`visit_variable_property`替换为使用`Property`表达式
   - 更新属性访问模式

2. 修复`rewrite_visitor.rs`
   - 更新`ExpressionType`枚举的使用
   - 移除对已删除变体的检查

3. 修复验证器
   - 更新一元操作的检查逻辑
   - 统一属性访问的处理方式

### 阶段2：修复执行器模块
- 更新表达式求值逻辑
- 修复属性访问的运行时处理

### 阶段3：修复规划器和其他模块
- 更新查询规划逻辑
- 修复存储层的表达式处理

## 验证标准
每完成一个文件的修改，运行`cargo check`确保：
- 编译错误数量减少
- 没有引入新的错误
- 所有警告在可控范围内
