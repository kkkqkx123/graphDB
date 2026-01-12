# 连接执行器表达式处理方案

## 问题分析

### 当前设计的矛盾

1. **PlanNode 层**：使用 `Vec<Expression>` 作为连接键
   - 这是合理的，因为查询计划中的连接条件可以是表达式（如 `a.id = b.id` 或 `a.age > b.age`）

2. **Executor 层**：当前也使用 `Vec<Expression>`
   - 但哈希表实现需要的是列索引（`usize`）或具体的值（`Value`）

3. **HashTable 层**：使用 `key_index: usize` 构建哈希表
   - 这与 Expression 类型不匹配

### 核心问题

连接操作需要在执行时将表达式求值为具体的值，但当前的架构没有提供这个转换机制。

## 解决方案

### 架构设计

```
PlanNode (Expression) → Executor (Expression + Evaluator) → HashTable (Value)
```

### 具体实现

#### 1. Executor 层保持 Expression 类型

```rust
pub struct InnerJoinExecutor<S: StorageEngine> {
    base_executor: BaseJoinExecutor<S>,
    hash_keys: Vec<Expression>,
    probe_keys: Vec<Expression>,
    evaluator: ExpressionEvaluator,
}
```

#### 2. 实现表达式求值逻辑

在执行连接时，对每一行数据求值表达式：

```rust
fn evaluate_expression_on_row(
    expr: &Expression,
    row: &[Value],
    col_names: &[String],
    context: &mut dyn ExpressionContext,
) -> Result<Value, QueryError> {
    // 创建行级上下文
    for (idx, col_name) in col_names.iter().enumerate() {
        if idx < row.len() {
            context.set_variable(col_name, row[idx].clone());
        }
    }
    
    // 求值表达式
    evaluator.evaluate(expr, context)
}
```

#### 3. 修改哈希表构建逻辑

```rust
pub fn build_single_key_table_with_expressions(
    dataset: &DataSet,
    key_expr: &Expression,
    col_names: &[String],
    evaluator: &ExpressionEvaluator,
) -> Result<SingleKeyHashTable, QueryError> {
    let mut hash_table = HashMap::new();
    let mut context = create_row_context();

    for row in &dataset.rows {
        // 求值表达式
        let key_value = evaluate_expression_on_row(
            key_expr,
            row,
            col_names,
            &mut context,
        )?;

        hash_table
            .entry(key_value)
            .or_insert_with(Vec::new)
            .push(row.clone());
    }

    Ok(hash_table)
}
```

### 实现步骤

1. **修改 BaseJoinExecutor**
   - 保持 `hash_keys: Vec<Expression>` 和 `probe_keys: Vec<Expression>`
   - 添加表达式求值器字段

2. **修改 InnerJoinExecutor**
   - 实现表达式求值逻辑
   - 修改哈希表构建方法以使用表达式求值

3. **修改 LeftJoinExecutor**
   - 同样实现表达式求值逻辑

4. **修改 HashTableBuilder**
   - 添加基于表达式的哈希表构建方法
   - 保留基于索引的方法（向后兼容）

5. **实现 ExpressionContext**
   - 创建行级上下文，支持变量绑定
   - 支持表达式求值

### 关键技术点

#### 表达式类型支持

连接表达式通常包括：
- `Variable(name)` - 列引用，如 `id`, `age`
- `Property { object, property }` - 属性访问，如 `person.id`
- `Binary { left, op, right }` - 二元表达式，如 `a.id = b.id`

#### 求值上下文

需要实现一个轻量级的 ExpressionContext，支持：
- 变量绑定（列名 → 值）
- 表达式求值
- 错误处理

#### 性能考虑

- 预编译表达式（如果可能）
- 缓存求值结果
- 批量求值优化

## 迁移计划

### 阶段1：基础架构
1. 修改 BaseJoinExecutor 保持 Expression 类型
2. 实现 ExpressionContext 的行级上下文
3. 添加表达式求值辅助方法

### 阶段2：执行器修改
1. 修改 InnerJoinExecutor 实现表达式求值
2. 修改 LeftJoinExecutor 实现表达式求值
3. 修改 HashTableBuilder 支持表达式

### 阶段3：集成测试
1. 编写单元测试
2. 编写集成测试
3. 性能测试

## 优势

1. **灵活性**：支持复杂的连接条件
2. **类型安全**：使用强类型系统避免错误
3. **可扩展性**：易于添加新的表达式类型
4. **性能**：通过求值器优化提高性能

## 风险与缓解

### 风险1：性能开销
- **缓解**：实现表达式预编译和缓存

### 风险2：复杂性增加
- **缓解**：提供清晰的抽象和文档

### 风险3：向后兼容性
- **缓解**：保留基于索引的方法作为备选
