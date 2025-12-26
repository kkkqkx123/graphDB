# full_outer_join.rs 作为统一外连接实现的分析

## 1. 当前实现分析

### 1.1 full_outer_join.rs 的特点

**优势**：
1. 已实现性能优化：
   - 预计算列映射（`left_col_map` 和 `right_col_map`）
   - 使用 `extract_key_values` 辅助函数
   - 避免重复的 O(n) 列查找

2. 完整的全外连接实现：
   - 构建两个哈希表（左表和右表）
   - 正确处理匹配和未匹配的行
   - 正确填充 NULL 值

3. 代码结构清晰：
   - 单一职责：只处理全外连接
   - 逻辑清晰，易于理解

**劣势**：
1. 只支持全外连接，不支持左连接和右连接
2. 与 `right_join.rs` 存在代码重复（`extract_key_values` 函数）
3. 与 `left_join.rs` 使用不同的架构

### 1.2 right_join.rs 的特点

**优势**：
1. 已实现性能优化：
   - 预计算列映射
   - 使用 `extract_key_values` 辅助函数
   - 避免重复的 O(n) 列查找

2. 完整的右外连接实现：
   - 构建一个哈希表（左表）
   - 正确处理匹配和未匹配的行
   - 正确填充 NULL 值

3. 代码结构清晰

**劣势**：
1. 与 `full_outer_join.rs` 存在代码重复
2. 与 `left_join.rs` 使用不同的架构

### 1.3 left_join.rs 的特点

**优势**：
1. 使用 `HashTableBuilder` 和 `HashTableProbe` 抽象层
2. 代码结构清晰，职责分离
3. 支持单键和多键连接的分离实现

**劣势**：
1. 没有预计算列映射，性能较差
2. 与 `right_join.rs` 和 `full_outer_join.rs` 使用不同的架构
3. 代码重复较多

## 2. 统一方案评估

### 2.1 方案一：新建 unified_outer_join.rs

**优点**：
- 完全独立的实现，不影响现有代码
- 可以设计更灵活的架构
- 易于测试和验证

**缺点**：
- 增加代码复杂度
- 需要维护三套实现（`left_join.rs`、`right_join.rs`、`unified_outer_join.rs`）
- 迁移成本高
- 可能引入新的 bug

### 2.2 方案二：重构现有文件（推荐）

**优点**：
- 减少代码重复
- 统一优化策略
- 保持代码简洁
- 降低维护成本
- 迁移成本较低

**缺点**：
- 需要修改现有代码
- 可能影响现有功能（需要充分测试）

## 3. 推荐方案：重构现有文件

### 3.1 核心思路

**不新建 `unified_outer_join.rs` 文件**，而是：

1. **提取公共代码**：
   - 将 `extract_key_values` 辅助函数提取到 `hash_table.rs`
   - 创建统一的哈希表构建和探测方法

2. **统一优化策略**：
   - 所有外连接执行器都使用预计算列映射
   - 所有外连接执行器都使用 `extract_key_values` 辅助函数
   - 保持架构一致性

3. **保持独立实现**：
   - 保留 `LeftJoinExecutor`、`RightJoinExecutor`、`FullOuterJoinExecutor`
   - 每个执行器专注于自己的连接类型
   - 通过共享的辅助函数实现代码复用

### 3.2 具体实施步骤

#### 步骤1：提取公共代码到 hash_table.rs

```rust
/// 从行中提取连接键的值（优化版本）
pub fn extract_key_values(
    row: &[Value],
    col_names: &[String],
    key_exprs: &[Expression],
    col_map: &HashMap<&str, usize>,
) -> Vec<Value> {
    let mut key_parts = Vec::new();

    for key_expr in key_exprs {
        if let Expression::Variable(key_name) = key_expr {
            if let Some(&key_pos) = col_map.get(key_name.as_str()) {
                if key_pos < row.len() {
                    key_parts.push(row[key_pos].clone());
                } else {
                    key_parts.push(Value::Null(NullType::Null));
                }
            } else if let Ok(key_pos) = key_name.parse::<usize>() {
                if key_pos < row.len() {
                    key_parts.push(row[key_pos].clone());
                } else {
                    key_parts.push(Value::Null(NullType::Null));
                }
            } else {
                key_parts.push(Value::Null(NullType::Null));
            }
        } else {
            key_parts.push(Value::Null(NullType::Null));
        }
    }

    key_parts
}

/// 构建哈希表（优化版本）
pub fn build_hash_table(
    dataset: &DataSet,
    key_exprs: &[Expression],
) -> Result<HashMap<JoinKey, Vec<usize>>, String> {
    let col_map: HashMap<&str, usize> = dataset.col_names.iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i))
        .collect();

    let mut hash_table = HashMap::new();

    for (idx, row) in dataset.rows.iter().enumerate() {
        let key_values = extract_key_values(row, &dataset.col_names, key_exprs, &col_map);
        let key = JoinKey::new(key_values);

        hash_table.entry(key)
            .or_insert_with(Vec::new)
            .push(idx);
    }

    Ok(hash_table)
}
```

#### 步骤2：重构 left_join.rs

```rust
use crate::query::executor::data_processing::join::hash_table::{
    extract_key_values, build_hash_table, JoinKey,
};

impl<S: StorageEngine> LeftJoinExecutor<S> {
    fn execute_single_key_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> DBResult<DataSet> {
        // 预计算列映射
        let left_col_map: HashMap<&str, usize> = left_dataset.col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let right_col_map: HashMap<&str, usize> = right_dataset.col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        // 左外连接：右表构建哈希表，左表探测
        let right_hash_table = build_hash_table(right_dataset, self.base_executor.probe_keys())
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "构建哈希表失败: {}",
                    e
                )))
            })?;

        // 构建结果集
        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();

        let mut matched_rows = std::collections::HashSet::new();

        // 探测哈希表
        for left_row in &left_dataset.rows {
            let key_values = extract_key_values(
                left_row,
                &left_dataset.col_names,
                self.base_executor.hash_keys(),
                &left_col_map,
            );

            let key = JoinKey::new(key_values);

            if let Some(right_indices) = right_hash_table.get(&key) {
                matched_rows.insert(left_row.clone());

                for right_idx in right_indices {
                    if *right_idx < right_dataset.rows.len() {
                        let right_row = &right_dataset.rows[*right_idx];
                        let new_row = self.base_executor.new_row(left_row.clone(), right_row.clone());
                        result.rows.push(new_row);
                    }
                }
            }
        }

        // 处理未匹配的左表行（填充NULL）
        for left_row in &left_dataset.rows {
            if !matched_rows.contains(left_row) {
                let mut new_row = left_row.clone();
                for _ in 0..right_dataset.col_names.len() {
                    new_row.push(Value::Null(NullType::Null));
                }
                result.rows.push(new_row);
            }
        }

        Ok(result)
    }
}
```

#### 步骤3：重构 right_join.rs

```rust
use crate::query::executor::data_processing::join::hash_table::{
    extract_key_values, build_hash_table, JoinKey,
};

impl<S: StorageEngine> RightJoinExecutor<S> {
    async fn execute_right_join(&mut self) -> DBResult<ExecutionResult> {
        // ... 获取输入数据集 ...

        // 预计算列映射
        let left_col_map: HashMap<&str, usize> = left_dataset.col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let right_col_map: HashMap<&str, usize> = right_dataset.col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        // 右外连接：左表构建哈希表，右表探测
        let left_hash_table = build_hash_table(left_dataset, self.base.hash_keys())
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "构建哈希表失败: {}",
                    e
                )))
            })?;

        // 构建结果集
        let mut result_dataset = DataSet {
            col_names: self.base.col_names().clone(),
            rows: Vec::new(),
        };

        let mut matched_rows = std::collections::HashSet::new();

        // 探测哈希表
        for right_row in &right_dataset.rows {
            let key_values = extract_key_values(
                right_row,
                &right_dataset.col_names,
                self.base.probe_keys(),
                &right_col_map,
            );

            let key = JoinKey::new(key_values);

            if let Some(left_indices) = left_hash_table.get(&key) {
                matched_rows.insert(right_row.clone());

                for left_idx in left_indices {
                    if *left_idx < left_dataset.rows.len() {
                        let left_row = &left_dataset.rows[*left_idx];
                        let mut joined_row = left_row.clone();
                        joined_row.extend_from_slice(right_row);
                        result_dataset.rows.push(joined_row);
                    }
                }
            }
        }

        // 处理未匹配的右表行（填充NULL）
        for right_row in &right_dataset.rows {
            if !matched_rows.contains(right_row) {
                let mut null_left_row = Vec::new();
                for _ in 0..left_dataset.col_names.len() {
                    null_left_row.push(Value::Null(NullType::Null));
                }

                let mut joined_row = null_left_row;
                joined_row.extend_from_slice(right_row);
                result_dataset.rows.push(joined_row);
            }
        }

        Ok(ExecutionResult::DataSet(result_dataset))
    }
}
```

#### 步骤4：重构 full_outer_join.rs

```rust
use crate::query::executor::data_processing::join::hash_table::{
    extract_key_values, build_hash_table, JoinKey,
};

impl<S: StorageEngine + Send + 'static> FullOuterJoinExecutor<S> {
    async fn execute_full_outer_join(&mut self) -> DBResult<ExecutionResult> {
        // ... 获取输入数据集 ...

        // 预计算列映射
        let left_col_map: HashMap<&str, usize> = left_dataset.col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let right_col_map: HashMap<&str, usize> = right_dataset.col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        // 构建左表哈希表
        let left_hash_table = build_hash_table(left_dataset, self.base.hash_keys())
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "构建左表哈希表失败: {}",
                    e
                )))
            })?;

        // 构建右表哈希表
        let right_hash_table = build_hash_table(right_dataset, self.base.probe_keys())
            .map_err(|e| {
                DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                    "构建右表哈希表失败: {}",
                    e
                )))
            })?;

        // 构建结果数据集
        let mut result_dataset = DataSet {
            col_names: self.base.col_names().clone(),
            rows: Vec::new(),
        };

        let mut matched_left_rows = std::collections::HashSet::new();
        let mut matched_right_rows = std::collections::HashSet::new();

        // 处理左表的每一行
        for left_row in &left_dataset.rows {
            let key_values = extract_key_values(
                left_row,
                &left_dataset.col_names,
                self.base.hash_keys(),
                &left_col_map,
            );

            let key = JoinKey::new(key_values);

            if let Some(right_indices) = right_hash_table.get(&key) {
                matched_left_rows.insert(left_row.clone());

                for right_idx in right_indices {
                    matched_right_rows.insert(*right_idx);

                    if *right_idx < right_dataset.rows.len() {
                        let right_row = &right_dataset.rows[*right_idx];
                        let mut joined_row = left_row.clone();
                        joined_row.extend_from_slice(right_row);
                        result_dataset.rows.push(joined_row);
                    }
                }
            } else {
                // 没有匹配的右表行，用NULL填充右表部分
                let mut null_right_row = Vec::new();
                for _ in 0..right_dataset.col_names.len() {
                    null_right_row.push(Value::Null(NullType::Null));
                }

                let mut joined_row = left_row.clone();
                joined_row.extend_from_slice(&null_right_row);
                result_dataset.rows.push(joined_row);
            }
        }

        // 添加右表中没有匹配的行
        for (right_idx, right_row) in right_dataset.rows.iter().enumerate() {
            if !matched_right_rows.contains(&right_idx) {
                // 用NULL填充左表部分
                let mut null_left_row = Vec::new();
                for _ in 0..left_dataset.col_names.len() {
                    null_left_row.push(Value::Null(NullType::Null));
                }

                let mut joined_row = null_left_row;
                joined_row.extend_from_slice(right_row);
                result_dataset.rows.push(joined_row);
            }
        }

        Ok(ExecutionResult::DataSet(result_dataset))
    }
}
```

## 4. 优势总结

### 4.1 代码复用
- 提取公共的 `extract_key_values` 函数
- 提取公共的 `build_hash_table` 函数
- 减少约 100 行重复代码

### 4.2 性能统一
- 所有外连接都使用预计算列映射
- 所有外连接都使用优化的键提取逻辑
- 统一的性能优化策略

### 4.3 架构一致
- 所有外连接执行器使用相同的架构
- 代码风格一致
- 更容易维护

### 4.4 降低风险
- 不需要新建文件
- 逐步重构，风险可控
- 保持现有 API 不变

## 5. 结论

**不新建 `unified_outer_join.rs` 文件**，而是：

1. 将公共代码提取到 `hash_table.rs`
2. 重构 `left_join.rs`、`right_join.rs`、`full_outer_join.rs`
3. 统一优化策略和架构
4. 保持三个独立的执行器

这种方案可以：
- 减少代码重复
- 统一性能优化
- 保持代码简洁
- 降低维护成本
- 降低迁移风险