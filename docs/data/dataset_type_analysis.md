# DataSet 类型作用及使用情况分析

## 1. 概述

`DataSet` 是图数据库查询系统中的核心数据结构，用于表示**结构化表格数据**。它类似于关系数据库中的结果集（ResultSet），是查询执行过程中数据传递和结果输出的主要载体。

## 2. 类型定义

### 2.1 基本结构

```rust
pub struct DataSet {
    pub col_names: Vec<String>,      // 列名列表
    pub rows: Vec<Vec<Value>>,       // 行数据，每行是 Value 的集合
}
```

**位置**: `src/core/value/dataset.rs`

### 2.2 在类型系统中的位置

`DataSet` 是 `Value` 枚举的一个变体，属于高级复合类型：

```rust
pub enum Value {
    // ... 基础类型
    DataSet(super::dataset::DataSet),
}
```

**数据类型标识**: `DataType::DataSet`

**类型优先级**: 170（较高优先级，仅低于部分复杂类型）

## 3. 核心功能

### 3.1 基本操作

| 方法 | 功能 | 返回值 |
|------|------|--------|
| `new()` | 创建空数据集 | `DataSet` |
| `with_columns()` | 创建带列名的数据集 | `DataSet` |
| `add_row()` | 添加行 | `()` |
| `row_count()` | 获取行数 | `usize` |
| `col_count()` | 获取列数 | `usize` |
| `is_empty()` | 检查是否为空 | `bool` |
| `get_col_index()` | 获取列索引 | `Option<usize>` |
| `get_column()` | 获取列所有值 | `Option<Vec<Value>>` |

### 3.2 数据转换操作

| 方法 | 功能 | 说明 |
|------|------|------|
| `filter()` | 过滤 | 根据谓词函数过滤行 |
| `map()` | 映射 | 转换每行数据 |
| `sort_by()` | 排序 | 根据比较函数排序 |
| `limit()` | 限制行数 | 取前 N 行 |
| `skip()` | 跳过行数 | 跳过前 N 行 |
| `transpose()` | 转置 | 行列互换 |
| `distinct()` | 去重 | 获取某列的唯一值 |

### 3.3 集合操作

| 方法 | 功能 | SQL 对应 |
|------|------|---------|
| `union()` | 并集 | `UNION` |
| `intersect()` | 交集 | `INTERSECT` |
| `except()` | 差集 | `EXCEPT` |

### 3.4 高级操作

| 方法 | 功能 | 说明 |
|------|------|------|
| `join()` | 连接 | 基于指定列进行等值连接 |
| `group_by()` | 分组 | 按键函数分组，返回 `(K, DataSet)` 对 |
| `aggregate()` | 聚合 | 应用聚合函数 |

### 3.5 内存管理

```rust
pub fn estimated_size(&self) -> usize
```

估算 DataSet 的内存占用，包括：
- 结构体本身大小
- 列名向量的容量和数据
- 行向量的容量和每行的 Value 大小

## 4. 在查询执行中的使用

### 4.1 执行结果类型

`DataSet` 是 `ExecutionResult` 枚举的主要变体之一：

```rust
pub enum ExecutionResult {
    Values(Vec<Value>),
    Vertices(Vec<Vertex>),
    Edges(Vec<Edge>),
    DataSet(DataSet),      // 结构化数据集结果
    Result(CoreResult),
    Empty,
    Success,
    Error(String),
    Count(usize),
    Paths(Vec<Path>),
}
```

**位置**: `src/query/executor/base/execution_result.rs`

### 4.2 查询执行器中的使用

#### 4.2.1 投影操作 (Projection)

```rust
// src/query/executor/result_processing/projection.rs
ExecutionResult::DataSet(dataset) => {
    // 对数据集进行列投影
    let projected_dataset = ...;
    ExecutionResult::DataSet(projected_dataset)
}
```

#### 4.2.2 过滤操作 (Filter)

```rust
// src/query/executor/result_processing/filter.rs
ExecutionResult::DataSet(mut dataset) => {
    // 根据条件过滤数据集
    dataset.rows.retain(|row| predicate(row));
    Ok(ExecutionResult::DataSet(dataset))
}
```

#### 4.2.3 限制操作 (Limit)

```rust
// src/query/executor/result_processing/limit.rs
ExecutionResult::DataSet(mut data_set) => {
    // 应用 LIMIT 和 OFFSET
    data_set.rows = data_set.rows.iter().skip(offset).take(limit).cloned().collect();
    Ok(ExecutionResult::DataSet(data_set))
}
```

#### 4.2.4 排序操作 (Sort)

```rust
// src/query/executor/result_processing/sort.rs
ExecutionResult::DataSet(mut data_set) => {
    // 根据排序键排序
    data_set.sort_by(comparator);
    Ok(ExecutionResult::DataSet(dataset))
}
```

#### 4.2.5 聚合操作 (Aggregation)

```rust
// src/query/executor/result_processing/aggregation.rs
// 分组聚合后构造结果数据集
let result_dataset = DataSet {
    col_names: output_columns,
    rows: aggregated_rows,
};
```

#### 4.2.6 TopN 操作

```rust
// src/query/executor/result_processing/topn.rs
fn execute_topn_dataset(&self, dataset: DataSet) -> DBResult<DataSet> {
    // 并行或串行执行 TopN 选择
}
```

#### 4.2.7 采样操作 (Sample)

```rust
// src/query/executor/result_processing/sample.rs
ExecutionResult::DataSet(dataset) => {
    // 随机采样
    let sampled_dataset = ...;
    Ok(ExecutionResult::DataSet(sampled_dataset))
}
```

### 4.3 数据转换操作

#### 4.3.1 UNWIND 转换

```rust
// src/query/executor/result_processing/transformations/unwind.rs
// 将列表展开为多行
let mut dataset = DataSet {
    col_names: output_columns,
    rows: expanded_rows,
};
```

#### 4.3.2 顶点追加 (AppendVertices)

```rust
// src/query/executor/result_processing/transformations/append_vertices.rs
// 构建请求数据集并追加顶点信息
let mut dataset = DataSet {
    col_names: output_columns,
    rows: vertex_rows,
};
```

#### 4.3.3 模式应用 (PatternApply)

```rust
// src/query/executor/result_processing/transformations/pattern_apply.rs
// 应用图模式匹配
let mut dataset = DataSet {
    col_names: output_columns,
    rows: matched_rows,
};
```

#### 4.3.4 RollupApply

```rust
// src/query/executor/result_processing/transformations/rollup_apply.rs
// 执行 ROLLUP 操作
let mut dataset = DataSet {
    col_names: output_columns,
    rows: rollup_rows,
};
```

### 4.4 集合操作

#### 4.4.1 并集 (Union)

```rust
// src/query/executor/data_processing/set_operations/union.rs
fn execute_union(&mut self) -> Result<DataSet, QueryError> {
    // 1. 获取左右输入数据集
    // 2. 验证列名一致性
    // 3. 合并并去重
    let result_dataset = DataSet {
        col_names: self.set_executor.get_col_names().clone(),
        rows: deduped_rows,
    };
}
```

#### 4.4.2 交集 (Intersect) 和 差集 (Minus)

类似 Union，使用 `DataSet` 作为输入和输出。

### 4.5 连接操作 (Join)

#### 4.5.1 哈希表构建

```rust
// src/query/executor/data_processing/join/hash_table.rs
pub fn build_from_dataset(
    dataset: &DataSet,
    key_indices: &[usize],
    initial_capacity: usize,
) -> DBResult<HashTable> {
    // 从数据集构建哈希表用于连接
    for (idx, row) in dataset.rows.iter().enumerate() {
        let key_values = ...;
        let key = JoinKey::new(key_values);
        let entry = HashTableEntry::new(row.clone(), idx);
        hash_table.insert(key, entry)?;
    }
}
```

#### 4.5.2 全外连接 (FullOuterJoin)

```rust
// src/query/executor/data_processing/join/full_outer_join.rs
// 构造连接结果数据集
let result_dataset = DataSet {
    col_names: combined_columns,
    rows: joined_rows,
};
```

## 5. 在 API 层的使用

### 5.1 HTTP API

#### 5.1.1 查询处理

```rust
// src/api/server/http/handlers/query.rs
ExecutionResult::DataSet(dataset) => {
    let columns: Vec<String> = dataset.col_names.clone();
    let rows: Vec<HashMap<String, serde_json::Value>> = dataset
        .rows
        .iter()
        .map(|row| column_names.iter().zip(row).collect())
        .collect();
    // 序列化为 JSON 响应
}
```

#### 5.1.2 流式处理

```rust
// src/api/server/http/handlers/stream.rs
ExecutionResult::DataSet(dataset) => {
    let columns = dataset.col_names.clone();
    let rows: Vec<serde_json::Value> = dataset
        .rows
        .into_iter()
        .map(|row| ... )
        .collect();
    // 流式返回
}
```

### 5.2 图服务层

```rust
// src/api/server/graph_service.rs
// 通用情况：返回 DataSet
ExecutionResult::DataSet(DataSet {
    col_names: columns,
    rows: data_rows,
}) => {
    // 处理并返回
}
```

### 5.3 嵌入式 API

```rust
// src/api/core/query_api.rs
crate::query::executor::base::ExecutionResult::DataSet(data) => {
    // 处理数据集结果
    // DataSet 使用 `col_names` 而非 `columns`
}
```

## 6. 在测试中的使用

### 6.1 测试辅助函数

#### 6.1.1 结果验证

```rust
// tests/common/validation_helpers.rs
ExecutionResult::DataSet(ds) => {
    // 验证数据集结构和内容
}
```

#### 6.1.2 调试输出

```rust
// tests/common/debug_helpers.rs
/// 将 DataSet 格式化为人类可读的表格
pub fn format_dataset(dataset: &DataSet) -> String {
    let mut output = String::new();
    output.push_str(&dataset.col_names.join(", "));
    output.push('\n');
    for (i, row) in dataset.rows.iter().enumerate() {
        // 格式化每行
    }
    output.push_str(&format!("Total rows: {}\n", dataset.rows.len()));
    output
}

/// 打印数据集用于调试
pub fn print_dataset(dataset: &DataSet) {
    eprintln!("\n{}", format_dataset(dataset));
}
```

### 6.2 测试场景

```rust
// tests/common/test_scenario.rs
ExecutionResult::DataSet(ds) => ds.col_names.clone(),
ExecutionResult::DataSet(ds) => ds.rows.clone(),
```

### 6.3 数据夹具

```rust
// tests/common/data_fixtures.rs
/// 社交网络测试数据集
pub fn social_network_dataset() -> (Vec<Vertex>, Vec<Edge>) {
    // 构建测试用的顶点和边数据
}
```

### 6.4 集成测试

```rust
// tests/integration_core.rs
let mut dataset = DataSet::new();
dataset.col_names.push("name".to_string());
dataset.col_names.push("age".to_string());
dataset.rows.push(vec![
    Value::String("Alice".to_string()),
    Value::Int(30),
]);
```

## 7. 序列化与反序列化

`DataSet` 支持多种序列化格式：

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash, Encode, Decode)]
pub struct DataSet {
    pub col_names: Vec<String>,
    pub rows: Vec<Vec<super::Value>>,
}
```

- **Serde**: JSON 等格式序列化
- **Bincode**: 二进制编码/解码（用于存储和网络传输）

## 8. 值比较

```rust
// src/core/value/value_compare.rs
fn cmp_dataset(a: &DataSet, b: &DataSet) -> Ordering {
    // 比较列名
    // 比较行数
    // 逐行比较
}
```

在类型优先级中，`DataSet` 的优先级为 20，用于确定不同 `Value` 类型之间的比较顺序。

## 9. 优化相关

### 9.1 聚合策略配置

```rust
// src/query/optimizer/cost/config.rs
/// 小数据集阈值
pub small_dataset_threshold: u64,  // 默认值 1000
```

当数据行数低于此阈值时，使用简单聚合策略。

### 9.2 聚合策略选择

```rust
// src/query/optimizer/strategy/aggregate_strategy.rs
enum SelectionReason {
    SmallDataSet,    // 小数据集
    LargeDataSet,    // 大数据集
}
```

### 9.3 连接顺序优化

```rust
// src/query/optimizer/strategy/join_order.rs
dp_threshold: 8,  // 默认使用动态规划处理少于 8 个表的数据集
```

### 9.4 成本计算

```rust
// src/query/optimizer/cost/calculator.rs
Value::DataSet(_) => self.config.complex_type_cost_factor * 1.25,
```

`DataSet` 类型的成本因子较高，反映其处理复杂度。

## 10. 使用场景总结

| 场景 | 说明 |
|------|------|
| **查询结果返回** | 作为 SELECT/MATCH 等查询的最终结果格式 |
| **中间结果传递** | 在执行计划各阶段之间传递结构化数据 |
| **集合运算** | UNION/INTERSECT/EXCEPT 操作的输入输出 |
| **连接操作** | Join 操作的构建和输出 |
| **聚合分组** | GROUP BY 和聚合函数的处理对象 |
| **数据转换** | UNWIND、APPLY 等转换操作的结果 |
| **排序限制** | ORDER BY、LIMIT、TOPN 的处理对象 |
| **API 响应** | HTTP API 和嵌入式 API 的结果格式 |

## 11. 设计特点

1. **简单直观**: 采用类似表格的行列结构，易于理解和使用
2. **功能丰富**: 提供过滤、映射、排序、分组、聚合等多种操作
3. **类型安全**: 作为 `Value` 的变体，享受 Rust 类型系统的安全保障
4. **可序列化**: 支持 JSON 和二进制序列化，便于存储和传输
5. **内存估算**: 提供 `estimated_size()` 方法用于内存管理
6. **不可变性**: 大多数操作返回新 `DataSet`，保持原数据不变

## 12. 相关文件索引

| 文件路径 | 说明 |
|---------|------|
| `src/core/value/dataset.rs` | DataSet 核心定义和实现 |
| `src/core/value/value_def.rs` | Value 枚举定义，包含 DataSet 变体 |
| `src/core/types/mod.rs` | DataType 枚举定义 |
| `src/core/type_system.rs` | 类型系统工具 |
| `src/core/value/value_compare.rs` | 值比较实现 |
| `src/query/executor/base/execution_result.rs` | 执行结果类型定义 |
| `src/query/executor/result_processing/` | 各种结果处理执行器 |
| `src/query/executor/data_processing/` | 数据处理执行器 |
| `src/api/server/http/handlers/` | HTTP API 处理器 |
| `tests/common/` | 测试辅助工具 |

## 13. 总结

`DataSet` 是图数据库查询系统的核心数据结构，承担着：
- **数据载体**: 在执行计划各阶段传递结构化数据
- **结果格式**: 作为查询的最终输出格式
- **操作对象**: 支持丰富的数据操作和转换

其设计借鉴了关系数据库的结果集概念，同时保持了 Rust 的类型安全和内存安全特性，是连接存储层、查询引擎和 API 层的关键纽带。
