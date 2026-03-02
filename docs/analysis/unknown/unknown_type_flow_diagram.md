# Unknown 类型处理流程详细图解

## 1. 完整执行流程图

```
┌─────────────────────────────────────────────────────────────────┐
│                    UNWIND 语句的完整生命周期                      │
└─────────────────────────────────────────────────────────────────┘

╔═════════════════════════════════════════════════════════════════╗
║                        阶段1: 语法解析                           ║
║                                                                 ║
║  输入：UNWIND variable_expr AS variable_name                   ║
║  处理：将语句分解为结构化表达式                                  ║
║  输出：(Expression, String)                                    ║
╚═════════════════════════════════════════════════════════════════╝
                              ↓
╔═════════════════════════════════════════════════════════════════╗
║                    阶段2: 验证（UnwindValidator）               ║
║                                                                 ║
║  ┌─────────────────────────────────────────────────────────┐  ║
║  │ 1. validate_expression()                                │  ║
║  │    检查：表达式是否为空                                  │  ║
║  │    类型推导：expression.deduce_type()                   │  ║
║  │    验证：结果应为 List 或 Set                           │  ║
║  └─────────────────────────────────────────────────────────┘  ║
║           ↓                                   ↑                 ║
║           ├─ 类型确定（Int/String/List等）  │               ║
║           │   → 接受，继续                   │                ║
║           │                                  │                 ║
║           └─ 类型不确定（DataType::Empty）  │               ║
║               ↓                              │                 ║
║  ┌─────────────────────────────────────────────────────────┐  ║
║  │ 2. validate_variable()                                  │  ║
║  │    检查：变量名是否为空                                  │  ║
║  │    检查：变量名格式是否合法                              │  ║
║  │    检查：变量名是否已定义                                │  ║
║  └─────────────────────────────────────────────────────────┘  ║
║           ↓                                                    ║
║  ┌─────────────────────────────────────────────────────────┐  ║
║  │ 3. validate_type()                                      │  ║
║  │                                                         │  ║
║  │    list_type = deduce_list_element_type(expr)          │  ║
║  │                                                         │  ║
║  │    if list_type == ValueType::Unknown {               │  ║
║  │        // 不报错！延迟到运行时                          │  ║
║  │        // 这是关键点                                    │  ║
║  │    }                                                    │  ║
║  │                                                         │  ║
║  │    ✓ 返回 Ok(())                                       │  ║
║  └─────────────────────────────────────────────────────────┘  ║
║           ↓                                                    ║
║  ┌─────────────────────────────────────────────────────────┐  ║
║  │ 4. validate_aliases()                                   │  ║
║  │    检查：表达式中引用的别名是否都已定义                  │  ║
║  └─────────────────────────────────────────────────────────┘  ║
║           ↓                                                    ║
║  验证结果：                                                    ║
║  ├─ 所有检查通过 → ValidatedUnwind { expression, ..., element_type }
║  │                 注意：element_type 可能是 Unknown              │
║  │                                                         │  ║
║  └─ 部分检查失败 → ValidationError                        │  ║
║                                                                 ║
║  输出：ValidatedUnwind 或 ValidationError                      ║
╚═════════════════════════════════════════════════════════════════╝
                              ↓
╔═════════════════════════════════════════════════════════════════╗
║              阶段3: 执行（UnwindExecutor）                     ║
║                                                                 ║
║  开始: execute_unwind()                                         ║
║                                                                 ║
║  ┌─────────────────────────────────────────────────────────┐  ║
║  │ Step 1: 获取输入数据                                    │  ║
║  │         input_result = context.get_result(input_var)    │  ║
║  │                                                         │  ║
║  │ 可能的输入类型：                                         │  ║
║  │  - ExecutionResult::Values(Vec<Value>)                 │  ║
║  │  - ExecutionResult::Vertices(Vec<Vertex>)              │  ║
║  │  - ExecutionResult::Edges(Vec<Edge>)                   │  ║
║  │  - ExecutionResult::Paths(Vec<Path>)                   │  ║
║  │  - ExecutionResult::DataSet(DataSet)                   │  ║
║  │  - ExecutionResult::Success / Empty / Count / ...      │  ║
║  └─────────────────────────────────────────────────────────┘  ║
║           ↓                                                    ║
║  ┌─────────────────────────────────────────────────────────┐  ║
║  │ Step 2: 对每个输入值迭代                                │  ║
║  │                                                         │  ║
║  │  for each value in input {                             │  ║
║  │      // 这里是运行时类型确定的地方！                    │  ║
║  │  }                                                      │  ║
║  └─────────────────────────────────────────────────────────┘  ║
║           ↓                                                    ║
║  ┌─────────────────────────────────────────────────────────┐  ║
║  │ Step 3: 对每个输入值求值 UNWIND 表达式                 │  ║
║  │                                                         │  ║
║  │  expr_context.set_variable("_", value)                 │  ║
║  │  unwind_value = ExpressionEvaluator::evaluate(expr)    │  ║
║  │                                                         │  ║
║  │  关键：evaluate() 返回实际的 Value 枚举值，而不是抽象类型  │  ║
║  │  - 如果 expr 是列表字面量：Value::List(...)            │  ║
║  │  - 如果 expr 是变量：根据上下文返回实际值               │  ║
║  │  - 如果 expr 是函数调用：根据函数执行结果返回           │  ║
║  └─────────────────────────────────────────────────────────┘  ║
║           ↓                                                    ║
║  ┌─────────────────────────────────────────────────────────┐  ║
║  │ Step 4: 类型处理（extract_list）← 关键处理点           │  ║
║  │                                                         │  ║
║  │  match unwind_value {                                  │  ║
║  │      Value::List(list) => {                            │  ║
║  │          // Value::List 已经确定是列表                  │  ║
║  │          list_values = list.clone().into_vec()         │  ║
║  │      },                                                │  ║
║  │      Value::Null(_) | Value::Empty => {                │  ║
║  │          // 空值 → 0 行                                 │  ║
║  │          list_values = vec![]                          │  ║
║  │      },                                                │  ║
║  │      _ => {                                            │  ║
║  │          // 其他具体值（Int, String, Vertex 等）      │  ║
║  │          // → 包装为单元素列表                           │  ║
║  │          list_values = vec![unwind_value]              │  ║
║  │      }                                                 │  ║
║  │  }                                                      │  ║
║  │                                                         │  ║
║  │  此处 Unknown 类型已消失，所有值都有具体类型！          │  ║
║  └─────────────────────────────────────────────────────────┘  ║
║           ↓                                                    ║
║  ┌─────────────────────────────────────────────────────────┐  ║
║  │ Step 5: 生成输出行                                      │  ║
║  │                                                         │  ║
║  │  for each list_item in list_values {                   │  ║
║  │      row = Vec::new()                                  │  ║
║  │      if !from_pipe {                                   │  ║
║  │          row.push(value.clone())  // 保留原始输入       │  ║
║  │      }                                                 │  ║
║  │      row.push(list_item)  // 添加展开的元素             │  ║
║  │      dataset.rows.push(row)                            │  ║
║  │  }                                                      │  ║
║  └─────────────────────────────────────────────────────────┘  ║
║           ↓                                                    ║
║  输出：DataSet { col_names, rows }                            ║
║  ├─ 所有行都由具体 Value 组成                                ║
║  ├─ 没有任何 Unknown 类型                                    ║
║  └─ 可以被后续操作（RETURN, GROUP BY 等）处理               ║
║                                                                 ║
╚═════════════════════════════════════════════════════════════════╝
```

## 2. 类型状态变化

```
表达式 UNWIND expr AS var
   ↓
验证阶段：
   ├─ 推导 expr 的类型
   │  └─ 如果推导失败 → ValueType::Unknown ✓
   │                    （允许，不中断）
   ├─ 验证 var 合法性
   └─ ✓ 验证通过

执行阶段：
   ├─ ExpressionEvaluator::evaluate(expr, context)
   │  ├─ 查找变量值
   │  ├─ 执行函数
   │  ├─ 计算表达式
   │  └─ 返回 Value（具体类型）← ✗ 不是 Unknown
   │
   ├─ extract_list(value) 根据实际 Value 类型处理
   │  └─ match value {
   │       Value::List(...) → 提取元素
   │       Value::Int(...) → 包装
   │       Value::Null → 空列表
   │       ... 其他值 → 包装
   │     }
   │
   └─ 输出：Vec<Value>（所有值都有具体类型）

输出阶段：
   └─ DataSet { rows: Vec<Vec<Value>> }
      └─ ✓ 完全确定的类型
```

## 3. 三个关键转换点

### 转换点 1: DataType::Empty → ValueType::Unknown

```
Location: src/query/validator/validator_trait.rs

fn from_data_type(dt: &DataType) -> ValueType {
    match dt {
        DataType::Bool => ValueType::Bool,
        DataType::Int => ValueType::Int,
        DataType::String => ValueType::String,
        DataType::List => ValueType::List,
        ...
        DataType::Empty => ValueType::Unknown,  ← 转换点
        ...
    }
}

说明：
- DataType::Empty 是表达式无法推导的类型
- 在验证器中转换为 ValueType::Unknown
- Unknown 表示"我不知道，但允许"的语义
```

### 转换点 2: ValueType::Unknown → Value（执行时）

```
Location: src/query/executor/result_processing/transformations/unwind.rs

fn execute_unwind() {
    // 执行表达式获得实际值
    let unwind_value = ExpressionEvaluator::evaluate(&expr, &context)?;
    //                                        ↓
    //                    返回的是 Value 枚举，比如 Value::List(...) 或 Value::Int(...)
    //
    // 此时不再是 Unknown，而是具体的值和类型
}

说明：
- 从 Unknown 类型跳跃到具体 Value
- ExpressionEvaluator 通过实际求值确定了类型
- extract_list 根据实际值进行处理
```

### 转换点 3: Value → DataSet（行生成时）

```
Location: src/query/executor/result_processing/transformations/unwind.rs

fn execute_unwind() {
    for list_item in list_values {  // 这些都是具体的 Value
        row.push(list_item);         // 推入行向量
        dataset.rows.push(row);      // 推入数据集
    }
    //
    // 结果：DataSet 的每个元素都是具体的 Value
    //      后续操作可以直接访问类型信息
}

说明：
- 每个 list_item 都有确定的类型
- DataSet 中没有任何 Unknown
- 可被后续查询操作安全处理
```

## 4. 不同输入表达式的处理路径

```
输入表达式示例及其类型推导

1. 字面量列表
   ────────────────────────
   UNWIND [1, 2, 3] AS x
   
   验证阶段：
   - Expression::List(...) → deduce_type() = DataType::List
   - ValueType::List（已知）
   - element_type = Integer（可推导）
   
   执行阶段：
   - evaluate([1, 2, 3]) → Value::List([Int(1), Int(2), Int(3)])
   - extract_list → [Int(1), Int(2), Int(3)]
   - 输出：3 行，每行 Int 类型
   
   结论：编译期和运行期类型一致 ✓


2. 变量引用
   ────────────────────────
   UNWIND variable AS x
   
   验证阶段：
   - Expression::Variable("variable") → deduce_type() = DataType::Empty
   - ValueType::Unknown（编译期无法知道）
   - element_type = Unknown
   
   执行阶段：
   - 表达式上下文查找 variable
   - variable 可能是 [1,2,3]、5、"hello" 或任何值
   - evaluate(variable) → 返回实际的 Value
   - extract_list(actual_value) → 根据实际值类型处理
   
   示例1：variable = [a, b, c]
   - extract_list(Value::List(...)) → [a, b, c]
   - 输出：3 行
   
   示例2：variable = 42
   - extract_list(Value::Int(42)) → [42]
   - 输出：1 行
   
   示例3：variable = null
   - extract_list(Value::Null) → []
   - 输出：0 行
   
   结论：运行时才能确定真实类型 ✓


3. 函数调用
   ────────────────────────
   UNWIND range(1, 10) AS x
   
   验证阶段：
   - Expression::Function("range", [1, 10])
   - deduce_function_type("range", ...) = DataType::List
   - 元素类型 = Unknown（range 返回整数列表但编译期未记录）
   
   执行阶段：
   - evaluate(range(1, 10)) → Value::List([Int(1), ..., Int(9)])
   - extract_list → [Int(1), ..., Int(9)]
   - 输出：9 行，每行 Int 类型
   
   结论：函数执行时获得具体值 ✓


4. 属性访问
   ────────────────────────
   UNWIND vertex.tags AS tag
   
   验证阶段：
   - Expression::Property { object: vertex, prop: "tags" }
   - deduce_type() = DataType::Empty
   - element_type = Unknown（属性值编译期无法确定）
   
   执行阶段：
   - evaluate(vertex.tags, context)
   - 根据 vertex 的实际数据库记录，获取 tags 属性
   - 可能返回 List、Null 或单个值
   - extract_list 处理该实际值
   
   结论：数据库值运行时才能确定 ✓
```

## 5. 错误处理流程

```
可能的错误场景及处理位置

1. 验证阶段错误（早期检测）
   ──────────────────────────
   
   a) 空表达式
      UNWIND  AS x
      └─ validate_expression() → Err("UNWIND 表达式无效")
      └─ 位置：unwind_validator.rs:175-178
      └─ 严重度：严重错误，阻止执行
   
   b) 类型不匹配（期望 List/Set）
      UNWIND 123 AS x
      └─ validate_expression_internal() 检查类型
      └─ 注意：字面量 123 会推导为 Int，但单值可在运行时包装
      └─ 实际可能通过验证（取决于推导结果）
   
   c) 变量名无效
      UNWIND [1, 2] AS 1x
      └─ validate_variable() → Err("变量名不能以数字开头")
      └─ 位置：unwind_validator.rs:214-222
      └─ 严重度：语法错误
   
   d) 变量重复
      UNWIND [1, 2] AS x  // 而 x 已存在
      └─ validate_variable() → Err("变量已定义")
      └─ 位置：unwind_validator.rs:224-232
      └─ 严重度：语义错误


2. 运行时错误（延迟检测）
   ──────────────────────────
   
   a) 输入值变量未找到
      evaluate(variable) 查找失败
      └─ ExpressionEvaluator::evaluate() → Err
      └─ catch 在 execute_unwind 处
      └─ 位置：unwind.rs:110-115
      └─ 严重度：执行错误
      └─ 提示：变量名拼写错误
   
   b) 表达式求值异常
      UNWIND func_that_crashes() AS x
      └─ ExpressionEvaluator::evaluate() → Err
      └─ 位置：unwind.rs:110-115
      └─ 严重度：执行错误
      └─ 提示：函数调用失败
   
   c) 后续操作类型冲突
      UNWIND [1, 2] AS x
      RETURN x + "string"
      └─ 此时不在 UNWIND 阶段
      └─ 在 RETURN 表达式求值时检测
      └─ 位置：expression_evaluator.rs（二元运算）
      └─ 严重度：类型错误
      └─ 提示：Int 和 String 无法相加


3. Unknown 类型本身不是错误
   ──────────────────────
   
   设计原理：
   - Unknown 表示"我不知道，但有可能"
   - 只要运行时能确定，就是有效的
   - 例如：UNWIND variable AS x
            ↓
            variable 在验证时是 Unknown
            ↓
            variable 在运行时可能是任何值
            ↓
            extract_list 处理任何可能的值
            ↓
            ✓ 通过（不是错误）
```

## 6. 性能和优化考虑

```
延迟类型推导的性能影响

1. 优点
   ────
   - 避免过度的编译期静态分析
   - 允许更灵活的动态查询
   - 支持在查询构建时未完全确定的数据

2. 缺点
   ────
   - 错误发现延迟到运行时
   - 可能需要额外的类型检查

3. 优化方向
   ────────
   - 在执行器中缓存类型推导结果
   - 对重复执行的查询使用 prepared statement 时，
     可在第一次执行时确定类型
   - 在后续操作中利用已知类型信息
```

