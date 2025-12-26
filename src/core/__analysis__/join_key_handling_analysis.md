# Join Key 处理机制分析与改进方案

## 问题概述

当前 `BaseJoinExecutor` 使用 `Expression` 作为 join key 的实现存在严重的架构问题，包括类型不匹配、性能问题和与 nebula-graph 实现的差异。

## 当前实现分析

### 1. BaseJoinExecutor 的类型声明

**文件**: `src/query/executor/data_processing/join/base_join.rs`

```rust
pub struct BaseJoinExecutor<S: StorageEngine> {
    pub base: BaseExecutor<S>,
    left_var: String,
    right_var: String,
    /// 连接键表达式列表
    hash_keys: Vec<Expression>,
    /// 探测键表达式列表
    probe_keys: Vec<Expression>,
    col_names: Vec<String>,
    description: String,
    exchange: bool,
    rhs_output_col_idxs: Option<Vec<usize>>,
}
```

**问题**: 声明为 `Vec<Expression>`，但实际实现中将其作为列索引使用。

### 2. Expression 类型定义

**文件**: `src/core/types/expression.rs`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Value),
    Variable(String),
    Property {
        object: Box<Expression>,
        property: String,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    // ... 30+ 变体
}
```

**问题**:
- `Expression` 未实现 `Hash` 和 `Eq` trait，无法直接作为 HashMap 的 key
- 表达式求值需要上下文，不能静态计算 hash
- 复杂表达式（如函数调用、聚合）无法预计算

### 3. 实际实现中的类型不匹配

在 `BaseJoinExecutor::execute` 方法中：

```rust
// TODO: 实现 join key 的求值逻辑
// 当前实现将 Expression 当作列索引使用
let hash_key_indices: Vec<usize> = self.hash_keys.iter()
    .filter_map(|expr| {
        // 错误：将 Expression 解析为列索引
        match expr {
            Expression::Variable(name) => name.parse::<usize>().ok(),
            _ => None,
        }
    })
    .collect();
```

**问题**:
- 声明为 `Expression`，实际使用时当作列索引
- 通过字符串解析提取索引，效率低下
- 忽略了表达式的语义含义

## Expression 在代码库中的其他使用场景分析

### 1. FilterExecutor - 过滤操作

**文件**: `src/query/executor/result_processing/filter.rs`

```rust
fn apply_filter(&self, dataset: &mut DataSet) -> DBResult<()> {
    let evaluator = ExpressionEvaluator;
    for row in &dataset.rows {
        let mut context = DefaultExpressionContext::new();
        for (i, col_name) in dataset.col_names.iter().enumerate() {
            if i < row.len() {
                context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        let condition_result = evaluator.evaluate(&self.condition, &mut context)?;
        if let Value::Bool(true) = condition_result {
            filtered_rows.push(row.clone());
        }
    }
    dataset.rows = filtered_rows;
    Ok(())
}
```

**特点**:
- 使用 `ExpressionEvaluator` 对每一行进行求值
- 不需要将 Expression 作为 HashMap 的 key
- 求值结果用于布尔判断

### 2. ProjectExecutor - 投影操作

**文件**: `src/query/executor/result_processing/projection.rs`

```rust
fn project_row(&self, row: &[Value], col_names: &[String]) -> DBResult<Vec<Value>> {
    let mut projected_row = Vec::new();
    let evaluator = ExpressionEvaluator;
    let mut context = DefaultExpressionContext::new();

    for (i, col_name) in col_names.iter().enumerate() {
        if i < row.len() {
            context.set_variable(col_name.clone(), row[i].clone());
        }
    }

    for column in &self.columns {
        match evaluator.evaluate(&column.expression, &mut context) {
            Ok(value) => projected_row.push(value),
            Err(e) => return Err(DBError::Expression(...)),
        }
    }

    Ok(projected_row)
}
```

**特点**:
- 对每一行的每个投影列进行求值
- 不需要将 Expression 作为 HashMap 的 key
- 求值结果用于构建新的行

### 3. ExpressionCacheManager - 表达式缓存

**文件**: `src/expression/cache/mod.rs`

```rust
pub struct ExpressionCacheManager {
    function_cache: Arc<StatsCacheWrapper<String, Value, ...>>,
    expression_cache: Arc<StatsCacheWrapper<String, Expression, ...>>,
    variable_cache: Arc<StatsCacheWrapper<String, Value, ...>>,
}
```

**特点**:
- 使用 `String` 作为 key，而不是 `Expression`
- 缓存的是求值结果，而不是表达式本身
- Expression 作为 value 存储在缓存中

### 4. 其他使用场景

通过 grep 分析发现，Expression 主要用于：
- 属性访问：`Property { object, property }`
- 函数调用：`Function { name, args }`
- 二元操作：`Binary { left, op, right }`
- 聚合函数：`Aggregate { func, arg, distinct }`

**结论**: 没有其他场景需要将 Expression 作为 HashMap 的 key。

## Value 类型的 Hash 实现

**文件**: `src/core/value.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Empty,
    Null(NullType),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Date(DateValue),
    Time(TimeValue),
    DateTime(DateTimeValue),
    Vertex(Box<Vertex>),
    Edge(Edge),
    Path(Path),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Set(std::collections::HashSet<Value>),
    Geography(GeographyValue),
    Duration(DurationValue),
    DataSet(DataSet),
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Empty => 0u8.hash(state),
            Value::Null(n) => { 1u8.hash(state); n.hash(state); }
            Value::Bool(b) => { 2u8.hash(state); b.hash(state); }
            Value::Int(i) => { 3u8.hash(state); i.hash(state); }
            Value::Float(f) => {
                4u8.hash(state);
                if f.is_nan() {
                    (0x7ff80000u32 as u64).hash(state);
                } else if *f == 0.0 {
                    0.0_f64.to_bits().hash(state);
                } else {
                    f.to_bits().hash(state);
                }
            }
            // ... 其他变体
        }
    }
}
```

**结论**: Value 已经实现了 Hash 和 Eq，可以作为 HashMap 的 key。

## 为 Expression 添加 Hash trait 的影响评估

### 1. 技术可行性

Expression 包含 30+ 变体，包括：
- 字面量：`Literal(Value)`
- 变量：`Variable(String)`
- 属性访问：`Property { object, property }`
- 二元操作：`Binary { left, op, right }`
- 函数调用：`Function { name, args }`
- 聚合函数：`Aggregate { func, arg, distinct }`
- 容器类型：`List(Vec<Expression>)`, `Map(Vec<(String, Expression)>)`
- 条件表达式：`Case { conditions, default }`
- 类型转换：`TypeCast { expr, target_type }`
- 下标访问：`Subscript { collection, index }`
- 范围访问：`Range { collection, start, end }`
- 路径构建：`Path(Vec<Expression>)`
- 标签：`Label(String)`
- 图数据库特有：`TagProperty`, `EdgeProperty`, `InputProperty`, `VariableProperty`, `SourceProperty`, `DestProperty`

**实现 Hash 的挑战**:
- 递归结构需要递归 hash
- 浮点数需要特殊处理（NaN、+0.0/-0.0）
- 容器类型需要确定性的 hash 顺序
- 函数调用和聚合函数的 hash 语义不明确

### 2. 性能影响

如果为 Expression 添加 Hash：
- **内存开销**: 每次计算 hash 都需要遍历整个表达式树
- **CPU 开销**: 复杂表达式的 hash 计算成本高
- **缓存失效**: Expression 是不可变的，hash 可以缓存，但需要额外的存储

**对比**: nebula-graph 不使用 Expression 作为 HashMap 的 key，而是使用求值后的 Value。

### 3. 语义问题

Expression 的 hash 语义不明确：
- `Variable("x")` 和 `Variable("y")` 应该有不同的 hash，但它们的值可能相同
- `Function("abs", [Literal(Value::Int(-5))])` 和 `Literal(Value::Int(5))` 在求值后相同，但表达式不同
- `Property { object: Variable("n"), property: "name" }` 的 hash 依赖于上下文

**结论**: Expression 的 hash 语义不清晰，不适合作为 HashMap 的 key。

## 与 Nebula-Graph 对比分析

### 1. Nebula-Graph 的实现

**文件**: `nebula-3.8.0/src/graph/executor/query/JoinExecutor.h`

```cpp
class JoinExecutor : public Executor {
 protected:
  void buildHashTable(const std::vector<Expression*>& hashKeys,
                      Iterator* iter,
                      std::unordered_map<List, std::vector<const Row*>>& hashTable);

  void buildSingleKeyHashTable(Expression* hashKey,
                               Iterator* iter,
                               std::unordered_map<Value, std::vector<const Row*>>& hashTable);

  std::unordered_map<Value, std::vector<const Row*>> hashTable_;
  std::unordered_map<List, std::vector<const Row*>> listHashTable_;
};
```

**文件**: `nebula-3.8.0/src/graph/executor/query/JoinExecutor.cpp`

```cpp
void JoinExecutor::buildHashTable(const std::vector<Expression*>& hashKeys,
                                  Iterator* iter,
                                  std::unordered_map<List, std::vector<const Row*>>& hashTable) {
  QueryExpressionContext ctx(ectx_);
  for (; iter->valid(); iter->next()) {
    List list;
    list.values.reserve(hashKeys.size());
    for (auto& col : hashKeys) {
      Value val = col->eval(ctx(iter));  // 运行时求值
      list.values.emplace_back(std::move(val));
    }

    auto& vals = hashTable[list];  // 使用 List 作为 key
    vals.emplace_back(iter->row());
  }
}
```

**关键设计**:
- 接收 `Expression*` 作为输入，但在运行时求值
- 使用 `Value`（单键）或 `List`（多键）作为 HashMap 的 key
- `Value` 和 `List` 都实现了 `Hash` 和 `Eq`
- 运行时求值是不可避免的，因为 join key 的值依赖于数据行

### 2. Nebula-Graph 的 Value 和 List Hash 实现

**文件**: `nebula-3.8.0/src/common/datatypes/Value.h`

```cpp
namespace std {
template <>
struct hash<nebula::Value> {
  std::size_t operator()(const nebula::Value& h) const {
    if (h.isInt()) {
      return h.getInt();
    } else if (h.isStr()) {
      return std::hash<std::string>()(h.getStr());
    }
    return h.hash();
  }
};
}
```

**文件**: `nebula-3.8.0/src/common/datatypes/List.h`

```cpp
namespace std {
template <>
struct hash<nebula::List> {
  std::size_t operator()(const nebula::List& h) const {
    if (h.values.size() == 1) {
      return std::hash<nebula::Value>()(h.values[0]);
    }
    size_t seed = 0;
    for (auto& v : h.values) {
      seed ^= hash<nebula::Value>()(v) + 0x9e3779b9 + (seed << 6) + (seed >> 2);
    }
    return seed;
  }
};
}
```

**关键特点**:
- Value 实现了 Hash，可以快速计算 hash
- List 的 hash 基于其包含的 Value 的 hash
- 单元素 List 的 hash 优化为直接使用元素的 hash

### 3. 运行时求值的性能开销分析

Nebula-Graph 在 buildHashTable 时：
1. 遍历每一行
2. 对每个 join key 表达式调用 `Expression::eval`
3. 将求值结果（Value 或 List）作为 HashMap 的 key

**性能特点**:
- **不可避免的开销**: join key 的值依赖于数据行，必须在运行时求值
- **可优化的部分**: 
  - 简单表达式（如列访问）可以优化为直接读取
  - 复杂表达式（如函数调用）必须求值
  - 可以缓存求值结果，但缓存命中率取决于数据分布
- **Hash 计算开销**: Value 和 List 的 hash 计算很快，因为它们是简单的数据结构

**对比**: 如果使用 Expression 作为 HashMap 的 key：
- 需要计算整个表达式树的 hash
- 每次查找都需要重新计算 hash（除非缓存）
- hash 语义不清晰，可能导致错误的匹配

## 最终修改方案

基于以上分析，提出以下修改方案：

### 方案概述

**核心思想**: 采用 nebula-graph 的设计，使用求值后的 `Value`（单键）或 `Vec<Value>`（多键）作为 HashMap 的 key，而不是使用 `Expression`。

### 架构设计

```
Expression (join key 表达式)
    ↓ 运行时求值
Value (单键) 或 Vec<Value> (多键)
    ↓ 作为 HashMap 的 key
HashMap<Value, Vec<Row>> 或 HashMap<Vec<Value>, Vec<Row>>
```

### 实现步骤

#### 步骤 1: 修改 BaseJoinExecutor 的字段定义

```rust
pub struct BaseJoinExecutor<S: StorageEngine> {
    pub base: BaseExecutor<S>,
    left_var: String,
    right_var: String,
    /// 连接键表达式列表（保持不变，用于运行时求值）
    hash_keys: Vec<Expression>,
    /// 探测键表达式列表（保持不变，用于运行时求值）
    probe_keys: Vec<Expression>,
    col_names: Vec<String>,
    description: String,
    exchange: bool,
    rhs_output_col_idxs: Option<Vec<usize>>,
}
```

**说明**: 保持 `Vec<Expression>` 的定义，因为它们用于运行时求值。

#### 步骤 2: 实现 JoinKeyEvaluator

```rust
/// Join Key 求值器
pub struct JoinKeyEvaluator {
    evaluator: ExpressionEvaluator,
}

impl JoinKeyEvaluator {
    pub fn new() -> Self {
        Self {
            evaluator: ExpressionEvaluator,
        }
    }

    /// 评估单个 join key
    pub fn evaluate_key(
        &self,
        expr: &Expression,
        context: &mut DefaultExpressionContext,
    ) -> DBResult<Value> {
        self.evaluator.evaluate(expr, context)
    }

    /// 评估多个 join key（返回 Vec<Value>）
    pub fn evaluate_keys(
        &self,
        exprs: &[Expression],
        context: &mut DefaultExpressionContext,
    ) -> DBResult<Vec<Value>> {
        let mut keys = Vec::with_capacity(exprs.len());
        for expr in exprs {
            keys.push(self.evaluate_key(expr, context)?);
        }
        Ok(keys)
    }
}
```

**说明**: 
- 使用现有的 `ExpressionEvaluator` 进行求值
- 返回 `Value`（单键）或 `Vec<Value>`（多键）
- `Value` 已经实现了 `Hash` 和 `Eq`

#### 步骤 3: 修改 BaseJoinExecutor 的 execute 方法

```rust
impl<S: StorageEngine> BaseJoinExecutor<S> {
    pub fn execute(&mut self) -> DBResult<DataSet> {
        let left_dataset = self.get_input_dataset(&self.left_var)?;
        let right_dataset = self.get_input_dataset(&self.right_var)?;

        let evaluator = JoinKeyEvaluator::new();

        // 构建哈希表
        let hash_table = if self.hash_keys.len() == 1 {
            // 单键：使用 HashMap<Value, Vec<Row>>
            self.build_single_key_hash_table(&left_dataset, &evaluator)?
        } else {
            // 多键：使用 HashMap<Vec<Value>, Vec<Row>>
            self.build_multi_key_hash_table(&left_dataset, &evaluator)?
        };

        // 探测哈希表
        let result = if self.probe_keys.len() == 1 {
            self.probe_single_key_hash_table(&right_dataset, &hash_table, &evaluator)?
        } else {
            self.probe_multi_key_hash_table(&right_dataset, &hash_table, &evaluator)?
        };

        Ok(result)
    }

    fn build_single_key_hash_table(
        &self,
        dataset: &DataSet,
        evaluator: &JoinKeyEvaluator,
    ) -> DBResult<HashMap<Value, Vec<Vec<Value>>>> {
        let mut hash_table = HashMap::new();

        for row in &dataset.rows {
            let mut context = self.create_context(dataset, row);
            let key = evaluator.evaluate_key(&self.hash_keys[0], &mut context)?;

            hash_table.entry(key).or_insert_with(Vec::new).push(row.clone());
        }

        Ok(hash_table)
    }

    fn build_multi_key_hash_table(
        &self,
        dataset: &DataSet,
        evaluator: &JoinKeyEvaluator,
    ) -> DBResult<HashMap<Vec<Value>, Vec<Vec<Value>>>> {
        let mut hash_table = HashMap::new();

        for row in &dataset.rows {
            let mut context = self.create_context(dataset, row);
            let keys = evaluator.evaluate_keys(&self.hash_keys, &mut context)?;

            hash_table.entry(keys).or_insert_with(Vec::new).push(row.clone());
        }

        Ok(hash_table)
    }

    fn probe_single_key_hash_table(
        &self,
        dataset: &DataSet,
        hash_table: &HashMap<Value, Vec<Vec<Value>>>,
        evaluator: &JoinKeyEvaluator,
    ) -> DBResult<DataSet> {
        let mut result = DataSet::new();
        result.col_names = self.col_names.clone();

        for row in &dataset.rows {
            let mut context = self.create_context(dataset, row);
            let key = evaluator.evaluate_key(&self.probe_keys[0], &mut context)?;

            if let Some(matching_rows) = hash_table.get(&key) {
                for left_row in matching_rows {
                    let mut joined_row = left_row.clone();
                    joined_row.extend(row.clone());
                    result.rows.push(joined_row);
                }
            }
        }

        Ok(result)
    }

    fn probe_multi_key_hash_table(
        &self,
        dataset: &DataSet,
        hash_table: &HashMap<Vec<Value>, Vec<Vec<Value>>>,
        evaluator: &JoinKeyEvaluator,
    ) -> DBResult<DataSet> {
        let mut result = DataSet::new();
        result.col_names = self.col_names.clone();

        for row in &dataset.rows {
            let mut context = self.create_context(dataset, row);
            let keys = evaluator.evaluate_keys(&self.probe_keys, &mut context)?;

            if let Some(matching_rows) = hash_table.get(&keys) {
                for left_row in matching_rows {
                    let mut joined_row = left_row.clone();
                    joined_row.extend(row.clone());
                    result.rows.push(joined_row);
                }
            }
        }

        Ok(result)
    }

    fn create_context(&self, dataset: &DataSet, row: &[Value]) -> DefaultExpressionContext {
        let mut context = DefaultExpressionContext::new();
        for (i, col_name) in dataset.col_names.iter().enumerate() {
            if i < row.len() {
                context.set_variable(col_name.clone(), row[i].clone());
            }
        }
        context
    }
}
```

**说明**:
- 使用 `Value`（单键）或 `Vec<Value>`（多键）作为 HashMap 的 key
- `Value` 已经实现了 `Hash` 和 `Eq`，可以直接使用
- `Vec<Value>` 的 hash 可以通过迭代计算（参考 nebula-graph 的实现）

#### 步骤 4: 为 Vec<Value> 实现 Hash（如果需要）

Rust 的 `Vec<T>` 只有在 `T: Hash` 时才能作为 HashMap 的 key。`Value` 已经实现了 `Hash`，所以 `Vec<Value>` 也可以作为 key。

如果需要优化 hash 计算（参考 nebula-graph 的单元素优化），可以实现自定义的 hash：

```rust
use std::hash::{Hash, Hasher};

pub struct JoinKey(Vec<Value>);

impl JoinKey {
    pub fn new(keys: Vec<Value>) -> Self {
        Self(keys)
    }
}

impl Hash for JoinKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.len() == 1 {
            // 单键优化：直接使用元素的 hash
            self.0[0].hash(state);
        } else {
            // 多键：组合 hash
            for (i, key) in self.0.iter().enumerate() {
                key.hash(state);
                if i < self.0.len() - 1 {
                    0x9e3779b9u64.hash(state);
                }
            }
        }
    }
}

impl PartialEq for JoinKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for JoinKey {}
```

**说明**: 
- 单键优化：直接使用元素的 hash，避免额外的包装
- 多键：组合 hash，使用类似于 nebula-graph 的算法
- 实现 `Eq` 以满足 HashMap 的要求

### 性能优化建议

#### 1. 简单表达式优化

对于简单的表达式（如 `Variable("name")`），可以优化为直接读取列值：

```rust
impl JoinKeyEvaluator {
    pub fn evaluate_key_optimized(
        &self,
        expr: &Expression,
        context: &mut DefaultExpressionContext,
        dataset: &DataSet,
        row: &[Value],
    ) -> DBResult<Value> {
        match expr {
            Expression::Variable(name) => {
                if let Some(idx) = dataset.col_names.iter().position(|n| n == name) {
                    if idx < row.len() {
                        return Ok(row[idx].clone());
                    }
                }
                self.evaluator.evaluate(expr, context)
            }
            _ => self.evaluator.evaluate(expr, context),
        }
    }
}
```

#### 2. 表达式缓存

对于重复的表达式求值，可以使用缓存：

```rust
use std::collections::HashMap;

pub struct CachedJoinKeyEvaluator {
    evaluator: ExpressionEvaluator,
    cache: HashMap<String, Value>,
}

impl CachedJoinKeyEvaluator {
    pub fn evaluate_key_cached(
        &mut self,
        expr: &Expression,
        context: &mut DefaultExpressionContext,
    ) -> DBResult<Value> {
        let cache_key = format!("{:?}", expr);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        let result = self.evaluator.evaluate(expr, context)?;
        self.cache.insert(cache_key, result.clone());
        Ok(result)
    }
}
```

**注意**: 缓存只在表达式不依赖于行数据时有效（如常量表达式）。

#### 3. 批量求值优化

对于多键 join，可以一次性求值所有键：

```rust
impl JoinKeyEvaluator {
    pub fn evaluate_keys_batch(
        &self,
        exprs: &[Expression],
        context: &mut DefaultExpressionContext,
    ) -> DBResult<Vec<Value>> {
        exprs.iter()
            .map(|expr| self.evaluate_key(expr, context))
            .collect()
    }
}
```

### 方案优势

1. **符合 nebula-graph 的设计**: 使用求值后的 Value 作为 HashMap 的 key
2. **避免 Expression 的 Hash 问题**: 不需要为 Expression 实现 Hash
3. **性能优化**: Value 的 hash 计算很快，可以快速查找
4. **语义清晰**: join key 的值是运行时求值的结果，而不是表达式本身
5. **向后兼容**: 保持 `Vec<Expression>` 的定义，不影响其他代码

### 方案风险

1. **运行时求值开销**: 每一行都需要求值，这是不可避免的
2. **内存开销**: 需要存储求值后的 Value 作为 HashMap 的 key
3. **复杂度增加**: 需要实现 JoinKeyEvaluator 和相关的哈希表构建逻辑

### 对比其他方案

#### 方案 A: 为 Expression 实现 Hash

**优点**:
- 可以直接使用 Expression 作为 HashMap 的 key
- 不需要运行时求值

**缺点**:
- Expression 的 hash 语义不清晰
- hash 计算成本高（需要遍历整个表达式树）
- 可能导致错误的匹配（如 `Variable("x")` 和 `Variable("y")` 的值可能相同）
- 影响其他使用 Expression 的代码

#### 方案 B: 使用列索引

**优点**:
- 实现简单
- 性能高（直接读取列值）

**缺点**:
- 不支持复杂表达式（如函数调用、属性访问）
- 与 nebula-graph 的设计不一致
- 限制了 join key 的表达能力

#### 方案 C: 当前方案（推荐）

**优点**:
- 符合 nebula-graph 的设计
- 支持任意表达式作为 join key
- Value 的 hash 计算快
- 语义清晰

**缺点**:
- 运行时求值开销（不可避免）
- 实现复杂度较高

## 实施计划

### 阶段 1: 基础实现（1-2 周）

1. 实现 `JoinKeyEvaluator`
2. 修改 `BaseJoinExecutor` 的 `execute` 方法
3. 实现单键和多键的哈希表构建和探测

### 阶段 2: 性能优化（1 周）

1. 实现简单表达式优化
2. 实现表达式缓存（可选）
3. 性能测试和调优

### 阶段 3: 测试和验证（1 周）

1. 单元测试
2. 集成测试
3. 性能测试

## 总结

本方案采用 nebula-graph 的设计，使用求值后的 `Value`（单键）或 `Vec<Value>`（多键）作为 HashMap 的 key，而不是使用 `Expression`。这样可以避免 Expression 的 Hash 问题，同时支持任意表达式作为 join key。

关键点：
- 保持 `Vec<Expression>` 的定义，用于运行时求值
- 使用 `Value`（单键）或 `Vec<Value>`（多键）作为 HashMap 的 key
- `Value` 已经实现了 `Hash` 和 `Eq`，可以直接使用
- 运行时求值是不可避免的，但可以通过优化减少开销

这个方案符合 nebula-graph 的设计，避免了 Expression 的 Hash 问题，同时保持了 join key 的表达能力。
```

**关键特点**:
- 使用 `Expression*` 指针，在运行时求值
- 使用 `Value` 和 `List` 作为 hash table 的 key（已实现 Hash）
- 分别处理单键和多键情况
- 通过迭代器逐行求值

### 2. Nebula-Graph 的表达式求值

**文件**: `nebula-3.8.0/src/graph/context/QueryExpressionContext.h`

```cpp
class QueryExpressionContext {
 public:
  explicit QueryExpressionContext(Iterator* iter) : iter_(iter) {}

  const Value& getValue(const Expression* expr) const {
    return expr->eval(*this);
  }

  Iterator* iter() const { return iter_; }

 private:
  Iterator* iter_;
};
```

**关键特点**:
- 表达式求值需要上下文（包含迭代器）
- 表达式实现 `eval()` 方法
- 返回 `Value` 类型（已实现 Hash）

### 3. Nebula-Graph 的构建过程

```cpp
void JoinExecutor::buildHashTable(
    const std::vector<Expression*>& hashKeys,
    Iterator* iter,
    std::unordered_map<List, std::vector<const Row*>>& hashTable) {
  QueryExpressionContext ctx(iter);
  for (; iter->valid(); iter->next()) {
    List key;
    for (auto* expr : hashKeys) {
      key.values.emplace_back(expr->eval(ctx));
    }
    hashTable[key].emplace_back(iter->row());
  }
}
```

**关键特点**:
- 为每一行创建表达式求值上下文
- 逐行求值表达式
- 将求值结果（Value）作为 hash key
- 存储行指针到 hash table

## 当前 Expression Evaluator 分析

### 1. Expression Evaluator Traits

**文件**: `src/expression/evaluator/traits.rs`

```rust
pub trait Evaluator<C: ExpressionContext> {
    fn evaluate(&self, expr: &Expression, context: &mut C) -> Result<Value, ExpressionError>;
    
    fn evaluate_batch(
        &self,
        expressions: &[Expression],
        context: &mut C,
    ) -> Result<Vec<Value>, ExpressionError>;
    
    fn can_evaluate(&self, _expr: &Expression, _context: &C) -> bool {
        true
    }
    
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
}
```

**优点**:
- 定义了清晰的表达式求值接口
- 支持批量求值
- 支持求值能力检查

**问题**:
- 未在 join executor 中使用
- 缺少专门的 join key 求值优化

### 2. Expression Context Traits

**文件**: `src/core/context/traits.rs`

```rust
pub trait ExpressionContext {
    fn get_variable(&self, name: &str) -> Option<&Value>;
    fn get_property(&self, object: &Value, property: &str) -> Option<Value>;
    fn set_variable(&mut self, name: String, value: Value);
}
```

**优点**:
- 定义了表达式求值所需的上下文接口
- 支持变量和属性访问

**问题**:
- 缺少行迭代器支持
- 无法直接用于 join 中的表达式求值

## 核心问题总结

### 1. 类型设计问题

| 问题 | 描述 | 影响 |
|------|------|------|
| Expression 未实现 Hash | 无法直接作为 HashMap key | 编译错误 |
| Expression 求值需要上下文 | 无法静态计算 hash | 运行时开销 |
| 声明与实现不匹配 | 声明为 Expression，实际当作索引 | 代码混乱 |

### 2. 架构设计问题

| 问题 | 描述 | 影响 |
|------|------|------|
| 缺少 JoinKeyEvaluator | 没有专门的 join key 求值器 | 性能差 |
| 表达式求值未集成 | ExpressionEvaluator 未在 join 中使用 | 功能缺失 |
| 上下文不完整 | ExpressionContext 缺少行迭代器 | 无法求值 |

### 3. 性能问题

| 问题 | 描述 | 影响 |
|------|------|------|
| 字符串解析提取索引 | 通过字符串解析获取列索引 | 性能差 |
| 无缓存机制 | 每次都重新求值表达式 | 性能差 |
| 无预计算 | 不区分简单和复杂表达式 | 性能差 |

## 改进方案

### 方案 1: 实现完整的 JoinKeyEvaluator（推荐）

#### 1.1 设计 JoinKeyEvaluator

```rust
pub struct JoinKeyEvaluator<S: StorageEngine> {
    evaluator: ExpressionEvaluator,
    cache: HashMap<Expression, CachedJoinKey>,
}

enum CachedJoinKey {
    DirectColumn(usize),           // 直接列访问
    Constant(Value),               // 常量表达式
    Computed(Expression),          // 需要运行时计算
}

impl<S: StorageEngine> JoinKeyEvaluator<S> {
    pub fn new() -> Self {
        Self {
            evaluator: ExpressionEvaluator::new(),
            cache: HashMap::new(),
        }
    }

    /// 预处理 join key 表达式
    pub fn preprocess(&mut self, expr: &Expression) -> Result<CachedJoinKey, JoinError> {
        if let Some(cached) = self.cache.get(expr) {
            return Ok(cached.clone());
        }

        let key = match expr {
            Expression::Variable(name) => {
                // 尝试解析为列索引
                if let Ok(idx) = name.parse::<usize>() {
                    CachedJoinKey::DirectColumn(idx)
                } else {
                    CachedJoinKey::Computed(expr.clone())
                }
            }
            Expression::Literal(value) => {
                CachedJoinKey::Constant(value.clone())
            }
            _ => CachedJoinKey::Computed(expr.clone()),
        };

        self.cache.insert(expr.clone(), key.clone());
        Ok(key)
    }

    /// 求值 join key
    pub fn evaluate(
        &self,
        key: &CachedJoinKey,
        row: &Row,
        context: &mut ExecutionContext,
    ) -> Result<Value, JoinError> {
        match key {
            CachedJoinKey::DirectColumn(idx) => {
                Ok(row.values[*idx].clone())
            }
            CachedJoinKey::Constant(value) => {
                Ok(value.clone())
            }
            CachedJoinKey::Computed(expr) => {
                self.evaluator.evaluate(expr, context)
            }
        }
    }
}
```

#### 1.2 修改 BaseJoinExecutor

```rust
pub struct BaseJoinExecutor<S: StorageEngine> {
    pub base: BaseExecutor<S>,
    left_var: String,
    right_var: String,
    /// 预处理的连接键
    hash_keys: Vec<CachedJoinKey>,
    /// 预处理的探测键
    probe_keys: Vec<CachedJoinKey>,
    col_names: Vec<String>,
    description: String,
    exchange: bool,
    rhs_output_col_idxs: Option<Vec<usize>>,
    /// Join key 求值器
    key_evaluator: JoinKeyEvaluator<S>,
}

impl<S: StorageEngine> BaseJoinExecutor<S> {
    pub fn new(
        left_var: String,
        right_var: String,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
        col_names: Vec<String>,
        description: String,
        exchange: bool,
        rhs_output_col_idxs: Option<Vec<usize>>,
        base: BaseExecutor<S>,
    ) -> Result<Self, JoinError> {
        let mut key_evaluator = JoinKeyEvaluator::new();
        
        // 预处理 hash keys
        let processed_hash_keys = hash_keys.iter()
            .map(|expr| key_evaluator.preprocess(expr))
            .collect::<Result<Vec<_>, _>>()?;
        
        // 预处理 probe keys
        let processed_probe_keys = probe_keys.iter()
            .map(|expr| key_evaluator.preprocess(expr))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            base,
            left_var,
            right_var,
            hash_keys: processed_hash_keys,
            probe_keys: processed_probe_keys,
            col_names,
            description,
            exchange,
            rhs_output_col_idxs,
            key_evaluator,
        })
    }

    fn build_hash_table(&self, rows: &[Row]) -> Result<HashMap<JoinKey, Vec<usize>>, JoinError> {
        let mut hash_table = HashMap::new();
        let mut context = ExecutionContext::new();

        for (idx, row) in rows.iter().enumerate() {
            let key_values: Vec<Value> = self.hash_keys.iter()
                .map(|key| self.key_evaluator.evaluate(key, row, &mut context))
                .collect::<Result<Vec<_>, _>>()?;

            let join_key = JoinKey::new(key_values);
            hash_table.entry(join_key).or_insert_with(Vec::new).push(idx);
        }

        Ok(hash_table)
    }
}
```

### 方案 2: 简化实现（快速修复）

如果暂时无法实现完整的 JoinKeyEvaluator，可以采用简化方案：

```rust
pub struct BaseJoinExecutor<S: StorageEngine> {
    pub base: BaseExecutor<S>,
    left_var: String,
    right_var: String,
    /// 连接键列索引列表
    hash_key_indices: Vec<usize>,
    /// 探测键列索引列表
    probe_key_indices: Vec<usize>,
    col_names: Vec<String>,
    description: String,
    exchange: bool,
    rhs_output_col_idxs: Option<Vec<usize>>,
}

impl<S: StorageEngine> BaseJoinExecutor<S> {
    pub fn new(
        left_var: String,
        right_var: String,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
        col_names: Vec<String>,
        description: String,
        exchange: bool,
        rhs_output_col_idxs: Option<Vec<usize>>,
        base: BaseExecutor<S>,
    ) -> Result<Self, JoinError> {
        // 提取列索引
        let hash_key_indices = hash_keys.iter()
            .map(|expr| Self::extract_column_index(expr))
            .collect::<Result<Vec<_>, _>>()?;
        
        let probe_key_indices = probe_keys.iter()
            .map(|expr| Self::extract_column_index(expr))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            base,
            left_var,
            right_var,
            hash_key_indices,
            probe_key_indices,
            col_names,
            description,
            exchange,
            rhs_output_col_idxs,
        })
    }

    fn extract_column_index(expr: &Expression) -> Result<usize, JoinError> {
        match expr {
            Expression::Variable(name) => {
                name.parse::<usize>()
                    .map_err(|_| JoinError::InvalidJoinKey(format!("Invalid column index: {}", name)))
            }
            _ => Err(JoinError::InvalidJoinKey("Only variable expressions are supported".to_string())),
        }
    }
}
```

### 方案 3: 参考 Nebula-Graph 实现

完全参考 nebula-graph 的实现方式：

```rust
pub struct BaseJoinExecutor<S: StorageEngine> {
    pub base: BaseExecutor<S>,
    left_var: String,
    right_var: String,
    /// 连接键表达式
    hash_keys: Vec<Expression>,
    /// 探测键表达式
    probe_keys: Vec<Expression>,
    col_names: Vec<String>,
    description: String,
    exchange: bool,
    rhs_output_col_idxs: Option<Vec<usize>>,
    /// Hash table: Value -> Vec<Row>
    hash_table: HashMap<Value, Vec<Row>>,
    /// Hash table for multiple keys: List -> Vec<Row>
    list_hash_table: HashMap<Vec<Value>, Vec<Row>>,
}

impl<S: StorageEngine> BaseJoinExecutor<S> {
    fn build_hash_table(&mut self, rows: Vec<Row>) -> Result<(), JoinError> {
        let mut context = ExecutionContext::new();
        let evaluator = ExpressionEvaluator::new();

        if self.hash_keys.len() == 1 {
            // 单键情况
            for row in rows {
                let key = evaluator.evaluate(&self.hash_keys[0], &mut context)?;
                self.hash_table.entry(key).or_insert_with(Vec::new).push(row);
            }
        } else {
            // 多键情况
            for row in rows {
                let mut key = Vec::with_capacity(self.hash_keys.len());
                for expr in &self.hash_keys {
                    key.push(evaluator.evaluate(expr, &mut context)?);
                }
                self.list_hash_table.entry(key).or_insert_with(Vec::new).push(row);
            }
        }

        Ok(())
    }
}
```

## 推荐方案

### 短期方案（1-2周）
采用**方案 2（简化实现）**，快速修复当前问题：
- 将 `Vec<Expression>` 改为 `Vec<usize>`
- 在构造函数中提取列索引
- 移除字符串解析逻辑

### 中期方案（2-4周）
实现**方案 1（JoinKeyEvaluator）**，提供更好的性能和扩展性：
- 实现 `JoinKeyEvaluator` 预处理和求值
- 支持直接列访问、常量和计算表达式
- 添加缓存机制

### 长期方案（4-8周）
参考**方案 3（Nebula-Graph）**，实现完整的表达式求值：
- 完善表达式求值器
- 支持复杂表达式
- 优化性能

## 实施计划

### 第一阶段：快速修复（1周）
1. 修改 `BaseJoinExecutor` 的类型定义
2. 在构造函数中提取列索引
3. 移除字符串解析逻辑
4. 运行测试验证

### 第二阶段：实现 JoinKeyEvaluator（2-3周）
1. 设计 `JoinKeyEvaluator` 接口
2. 实现预处理和求值逻辑
3. 集成到 `BaseJoinExecutor`
4. 添加性能测试

### 第三阶段：完善表达式求值（2-3周）
1. 完善 `ExpressionEvaluator`
2. 支持复杂表达式
3. 优化性能
4. 添加更多测试

## 预期收益

### 性能提升
- **直接列访问**: 避免字符串解析，提升 10-100 倍性能
- **缓存机制**: 避免重复求值，提升 2-5 倍性能
- **预计算**: 区分简单和复杂表达式，优化关键路径

### 代码质量
- **类型安全**: 消除类型不匹配问题
- **职责清晰**: 明确 join key 的处理逻辑
- **易于维护**: 代码结构清晰，易于理解和修改

### 扩展性
- **支持复杂表达式**: 可以支持任意表达式作为 join key
- **易于优化**: 可以添加更多优化策略
- **兼容性好**: 与 nebula-graph 实现方式一致

## 总结

当前 `BaseJoinExecutor` 使用 `Expression` 作为 join key 的实现存在严重的架构问题。通过与 nebula-graph 的对比分析，我们发现了关键差异：

1. **Nebula-Graph**: 在运行时求值表达式，使用 `Value` 作为 hash key
2. **当前实现**: 声明为 `Expression`，实际当作列索引使用

推荐采用三阶段实施方案：
1. 短期：快速修复类型不匹配问题
2. 中期：实现 `JoinKeyEvaluator` 提供更好的性能
3. 长期：参考 nebula-graph 实现完整的表达式求值

这将显著提升性能、代码质量和扩展性。
