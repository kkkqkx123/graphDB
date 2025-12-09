# Context 模块快速参考

## 核心概念速览

### 1. Result (执行结果)

**职责**: 包装执行结果，包括值、状态和迭代器

```rust
// 创建成功的结果
let result = Result::new(Value::Int(42));

// 创建带消息的部分成功结果
let result = ResultBuilder::new()
    .value(Value::DataSet(...))
    .state(ResultState::PartialSuccess)
    .message("有警告".to_string())
    .build();

// 访问
result.state();           // ResultState::Success
result.value();           // &Value
result.message();         // &str
```

### 2. Iterator (迭代器)

**职责**: 遍历数据（单值、DataSet 行、图结果）

```rust
// 获取大小
let size = iter.size();

// 遍历
while iter.valid() {
    let row = iter.row();
    iter.next();
}

// 删除当前行
iter.erase();            // 有序删除
iter.unstable_erase();   // 快速删除（破坏顺序）

// 选择范围
iter.select(0, 10);      // 保留 [0, 10) 范围

// 列访问
let val = iter.get_column("name")?;
let idx = iter.get_column_index("age")?;

// 拷贝迭代器（保存状态）
let backup = iter.copy();
```

### 3. ExecutionContext (执行上下文)

**职责**: 管理查询执行期间的变量值

```rust
let ctx = ExecutionContext::new();

// 设置值
ctx.set_value("x", Value::Int(42))?;

// 获取值
let val = ctx.get_value("x")?;

// 版本管理
ctx.set_value("x", Value::Int(100))?;  // 版本 1
let v0 = ctx.get_versioned_value("x", 0)?;    // 最新
let v_prev = ctx.get_versioned_value("x", -1)?; // 前一版本

// 历史管理
let hist = ctx.get_history("x")?;       // 所有版本
ctx.trunc_history("x", 5)?;             // 只保留最近 5 个版本
```

### 4. QueryExpressionContext (表达式上下文)

**职责**: 为表达式求值提供变量和行数据访问

```rust
let qctx = QueryExpressionContext::new(Arc::new(ectx));

// 设置当前行迭代器
qctx = qctx.with_iterator(Box::new(iter));

// 访问变量
let val = qctx.get_var("x")?;

// 访问列数据
let name = qctx.get_column("name")?;

// 访问属性
let tag_prop = qctx.get_tag_prop("person", "age")?;
let edge_prop = qctx.get_edge_prop("knows", "weight")?;

// 表达式内部变量
qctx.set_inner_var("temp", Value::Int(100));
if let Some(val) = qctx.get_inner_var("temp") { ... }
```

### 5. SymbolTable (符号表)

**职责**: 跟踪变量定义和读写依赖

```rust
let st = SymbolTable::new();

// 创建变量
st.new_variable("result")?;

// 检查变量
if st.has_variable("result") { ... }

// 读写依赖追踪
st.read_by("result", "node_1")?;       // result 被 node_1 读取
st.written_by("result", "node_0")?;    // result 被 node_0 写入

// 删除依赖
st.delete_read_by("result", "node_1")?;

// 获取变量信息
let var = st.get_var("result")?;
println!("读取者: {:?}", var.read_by);
println!("写入者: {:?}", var.written_by);
```

### 6. QueryContext (顶级查询上下文)

**职责**: 整合所有子上下文

```rust
let ctx = QueryContext::new();

// 访问子上下文
let ectx = ctx.ectx();              // 执行上下文
let vctx = ctx.vctx();              // 验证上下文
let st = ctx.sym_table();           // 符号表

// 生成唯一 ID
let id = ctx.gen_id();

// 标记终止
ctx.mark_killed();
if ctx.is_killed() { ... }
```

---

## Iterator 类型对比

| 类型 | 用途 | 特点 | 关键方法 |
|------|------|------|---------|
| **DefaultIter** | 单个常量值 | 大小固定为 1 | next()、reset() |
| **SequentialIter** | DataSet 行 | 行级操作、范围操作 | erase()、select()、eraseRange() |
| **GetNeighborsIter** | 图邻居结果 | 树状结构、属性访问 | getVertex()、getEdgeProp() |
| **PropIter** | 属性查询 | 属性优化 | getTagProp()、getEdgeProp() |

---

## 常见使用模式

### 模式 1: 迭代 DataSet

```rust
let dataset = /* ... */;
let mut iter = SequentialIter::new(Arc::new(dataset))?;

while iter.valid() {
    // 获取当前行
    if let Some(row) = iter.row() {
        // 处理行数据
        let name = iter.get_column("name")?;
        let age = iter.get_column("age")?;
        println!("{}，年龄 {}", name, age);
    }
    
    // 进入下一行
    iter.next();
}
```

### 模式 2: 表达式求值

```rust
let ectx = Arc::new(ExecutionContext::new());
ectx.set_value("x", Value::Int(10))?;
ectx.set_value("y", Value::Int(20))?;

let mut qctx = QueryExpressionContext::new(ectx);
qctx = qctx.with_iterator(Box::new(iter));

// 在执行 WHERE/SELECT 时评估表达式
let x_val = qctx.get_var("x")?;  // Value::Int(10)
let col_val = qctx.get_column("amount")?;
```

### 模式 3: 过滤行

```rust
let mut iter = SequentialIter::new(Arc::new(dataset))?;

while iter.valid() {
    let age = iter.get_column_by_index(1)?;
    
    // 删除不符合条件的行
    if age < 18 {
        iter.unstable_erase();  // 快速删除
    } else {
        iter.next();
    }
}
```

### 模式 4: 变量版本管理

```rust
let ectx = ExecutionContext::new();

// 初始化
ectx.set_value("count", Value::Int(0))?;

// 循环迭代中不断更新
for i in 1..=5 {
    ectx.set_value("count", Value::Int(i))?;
}

// 查看所有版本历史
let hist = ectx.get_history("count")?;
assert_eq!(hist.len(), 6);  // 初始值 + 5 次更新

// 获取前一版本
let prev = ectx.get_versioned_value("count", -1)?;  // Value::Int(4)
```

---

## 与其他模块的集成点

### Parser → QueryContext

Parser 在解析查询时使用 ValidateContext:
```rust
// Parser 设置当前空间
query_ctx.vctx_mut().switch_to_space(space_info);

// 注册变量
query_ctx.vctx_mut().register_variable("n", cols);
```

### Planner → SymbolTable

Planner 在规划时构建计划节点依赖:
```rust
// 标记变量的生产者和消费者
sym_table.written_by("result", node)?;
sym_table.read_by("result", next_node)?;
```

### Executor → QueryExpressionContext

Executor 评估表达式时使用表达式上下文:
```rust
let qectx = QueryExpressionContext::new(ectx.clone())
    .with_iterator(Box::new(current_iter));

// 评估过滤条件
let cond_result = evaluate_expr(condition, &qectx)?;
```

---

## 错误处理

### 常见错误

```rust
// ❌ 访问不存在的变量
ctx.get_value("undefined")?;  // Err: 变量 undefined 不存在

// ❌ 列不存在
iter.get_column("missing_col")?;  // Err: 列 missing_col 不存在

// ❌ 没有设置迭代器而访问列
qctx.get_column("col")?;  // Err: 没有设置迭代器

// ❌ 无效迭代状态
iter.next();
iter.next();
iter.row()?;  // None: 已到达末尾
```

### 最佳实践

```rust
// ✅ 检查迭代器有效性
while iter.valid() {
    // 安全访问
    iter.next();
}

// ✅ 使用 Result 进行错误传播
fn process(ctx: &QueryContext) -> Result<Value, String> {
    let val = ctx.ectx().get_value("x")?;
    Ok(val)
}

// ✅ 检查变量存在
if ctx.ectx().exist("x") {
    let val = ctx.ectx().get_value("x")?;
}
```

---

## 性能考虑

### 1. 迭代器选择

- **DefaultIter**: O(1) 内存，用于常量
- **SequentialIter**: O(n) 内存，用于 DataSet
- **GetNeighborsIter**: O(n*m) 内存，用于邻居结果
- **PropIter**: 优化后的属性访问

### 2. 删除操作

```rust
// 快速删除（推荐用于大量删除）
iter.unstable_erase();  // O(1)

// 有序删除（保持顺序）
iter.erase();           // O(n)

// 删除范围
iter.erase_range(0, 1000);  // O(n)
```

### 3. 拷贝优化

```rust
// ❌ 频繁拷贝
let copy = iter.copy();
let copy = iter.copy();
let copy = iter.copy();

// ✅ 必要时才拷贝
let backup = iter.copy();  // 保存还原点
```

---

## 调试技巧

### 1. 打印迭代器状态

```rust
println!("迭代器类型: {:?}", iter.kind());
println!("大小: {}", iter.size());
println!("有效: {}", iter.valid());
println!("列名: {:?}", iter.get_col_names());
```

### 2. 打印执行上下文

```rust
println!("所有变量:");
for (var_name, values) in ectx.all_variables() {
    println!("  {}: {:?}", var_name, values);
}
```

### 3. 打印符号表

```rust
println!("{}", st.to_string());
// 输出: SymTable: [
//   result: name: result, type: Dataset, readBy: <node_1>, writtenBy: <node_0>
//   ...
// ]
```

---

## 相关文档

- [完整功能分析](./context_module_missing_features.md)
- [实现路线图](./context_implementation_roadmap.md)
- [API 参考](#) (编写中)
