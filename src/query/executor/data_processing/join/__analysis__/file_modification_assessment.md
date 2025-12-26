# 文件修改评估

## 1. 需要修改的文件

### 1.1 hash_table.rs

**修改原因**：
- 需要添加 `UnifiedHashTable` 类型以支持统一的哈希表实现
- 需要添加性能优化的构建方法，集成 `extract_key_values` 辅助函数
- 需要保持现有的 `SingleKeyHashTable` 和 `MultiKeyHashTable` 以确保向后兼容性

**具体修改内容**：

1. 添加 `UnifiedHashTable` 类型：
```rust
/// 统一的哈希表
pub struct UnifiedHashTable {
    inner: HashMap<JoinKey, Vec<Vec<Value>>>,
    col_map: HashMap<String, usize>,
}

impl UnifiedHashTable {
    pub fn new(inner: HashMap<JoinKey, Vec<Vec<Value>>>, col_map: HashMap<String, usize>) -> Self {
        Self { inner, col_map }
    }

    pub fn get(&self, key: &JoinKey) -> Option<&Vec<Vec<Value>>> {
        self.inner.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&JoinKey, &Vec<Vec<Value>>)> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
```

2. 添加优化的构建方法：
```rust
impl HashTableBuilder {
    /// 构建统一的哈希表（优化版本）
    pub fn build_unified_table(
        dataset: &crate::core::DataSet,
        key_exprs: &[Expression],
    ) -> Result<UnifiedHashTable, String> {
        // 预计算列映射
        let col_map: HashMap<&str, usize> = dataset.col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let mut hash_table = HashMap::new();

        for row in &dataset.rows {
            let key_values = extract_key_values(row, &dataset.col_names, key_exprs, &col_map);
            let key = JoinKey::new(key_values);

            hash_table.entry(key)
                .or_insert_with(Vec::new)
                .push(row.clone());
        }

        let col_map_owned: HashMap<String, usize> = col_map.into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect();

        Ok(UnifiedHashTable::new(hash_table, col_map_owned))
    }
}
```

3. 添加辅助函数：
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
                    key_parts.push(Value::Null(crate::core::NullType::Null));
                }
            } else if let Ok(key_pos) = key_name.parse::<usize>() {
                if key_pos < row.len() {
                    key_parts.push(row[key_pos].clone());
                } else {
                    key_parts.push(Value::Null(crate::core::NullType::Null));
                }
            } else {
                key_parts.push(Value::Null(crate::core::NullType::Null));
            }
        } else {
            key_parts.push(Value::Null(crate::core::NullType::Null));
        }
    }

    key_parts
}
```

**向后兼容性**：
- 保留所有现有的公共 API
- 保留 `SingleKeyHashTable` 和 `MultiKeyHashTable` 类型别名
- 保留 `HashTableBuilder` 和 `HashTableProbe` 的所有现有方法

---

### 1.2 mod.rs

**修改原因**：
- 需要添加新模块 `unified_outer_join` 的导出
- 需要更新 `JoinExecutorFactory` 以支持统一实现
- 需要保持向后兼容性

**具体修改内容**：

1. 添加新模块声明：
```rust
pub mod unified_outer_join;
```

2. 添加新类型导出：
```rust
pub use unified_outer_join::UnifiedOuterJoinExecutor;
pub use hash_table::{UnifiedHashTable, extract_key_values};
```

3. 更新 `JoinExecutorFactory`（可选，用于渐进式迁移）：
```rust
impl JoinExecutorFactory {
    /// 根据配置创建相应的join执行器
    pub fn create_executor<S: crate::storage::StorageEngine + Send + 'static>(
        id: i64,
        storage: std::sync::Arc<std::sync::Mutex<S>>,
        config: JoinConfig,
    ) -> Result<Box<dyn crate::query::executor::traits::Executor<S>>, crate::query::QueryError>
    {
        match config.join_type {
            JoinType::Inner => {
                // 保持现有实现
                let hash_keys: Vec<Expression> = config.left_keys.into_iter().map(Expression::Variable).collect();
                let probe_keys: Vec<Expression> = config.right_keys.into_iter().map(Expression::Variable).collect();
                
                if config.enable_parallel {
                    Ok(Box::new(HashInnerJoinExecutor::new(
                        id, storage, config.left_var, config.right_var,
                        hash_keys, probe_keys, config.output_columns,
                    )))
                } else {
                    Ok(Box::new(InnerJoinExecutor::new(
                        id, storage, config.left_var, config.right_var,
                        hash_keys, probe_keys, config.output_columns,
                    )))
                }
            }
            JoinType::Left => {
                // 可以选择使用统一实现或保持现有实现
                // 渐进式迁移：先保持现有实现
                let hash_keys: Vec<Expression> = config.left_keys.into_iter().map(Expression::Variable).collect();
                let probe_keys: Vec<Expression> = config.right_keys.into_iter().map(Expression::Variable).collect();
                
                if config.enable_parallel {
                    Ok(Box::new(HashLeftJoinExecutor::new(
                        id, storage, config.left_var, config.right_var,
                        hash_keys, probe_keys, config.output_columns,
                    )))
                } else {
                    Ok(Box::new(LeftJoinExecutor::new(
                        id, storage, config.left_var, config.right_var,
                        hash_keys, probe_keys, config.output_columns,
                    )))
                }
            }
            JoinType::Right => {
                // 可以选择使用统一实现或保持现有实现
                // 渐进式迁移：先保持现有实现
                Ok(Box::new(RightJoinExecutor::new(
                    id, storage, config.left_var, config.right_var,
                    config.left_keys, config.right_keys, config.output_columns,
                )))
            }
            JoinType::Full => {
                // 可以选择使用统一实现或保持现有实现
                // 渐进式迁移：先保持现有实现
                Ok(Box::new(FullOuterJoinExecutor::new(
                    id, storage, config.left_var, config.right_var,
                    config.left_keys, config.right_keys, config.output_columns,
                )))
            }
            JoinType::Cross => {
                Ok(Box::new(CrossJoinExecutor::new(
                    id, storage, vec![config.left_var, config.right_var],
                    config.output_columns,
                )))
            }
        }
    }
}
```

**向后兼容性**：
- 保持所有现有的导出
- 保持 `JoinExecutorFactory` 的现有行为
- 新功能通过新类型添加，不影响现有代码

---

### 1.3 base_join.rs

**修改原因**：
- 可能需要添加一些辅助方法以支持统一实现
- 需要保持向后兼容性

**具体修改内容**：

1. 添加列映射预计算方法：
```rust
impl<S: StorageEngine> BaseJoinExecutor<S> {
    /// 预计算列名到索引的映射
    pub fn build_column_map(col_names: &[String]) -> HashMap<String, usize> {
        col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect()
    }

    /// 从行中提取指定列的值
    pub fn extract_row_values(row: &[Value], indices: &[usize]) -> Vec<Value> {
        indices.iter()
            .filter_map(|&idx| row.get(idx).cloned())
            .collect()
    }
}
```

**向后兼容性**：
- 只添加新的公共方法，不修改现有方法
- 保持所有现有的公共 API

---

## 2. 不需要修改的文件

### 2.1 inner_join.rs

**不需要修改的原因**：
- 内连接实现相对独立，与外连接的实现方式不同
- 内连接不涉及 NULL 填充逻辑，架构差异较大
- 统一方案主要针对外连接（左、右、全外连接）
- 内连接的性能优化可以独立进行

**未来考虑**：
- 如果需要统一所有连接类型，可以在后续阶段考虑
- 当前阶段专注于外连接的统一

---

### 2.2 cross_join.rs

**不需要修改的原因**：
- 笛卡尔积实现完全不同，不涉及键匹配
- 不需要哈希表，直接进行笛卡尔积运算
- 与外连接的实现方式完全独立

---

### 2.3 parallel.rs

**不需要修改的原因**：
- 并行处理模块是独立的优化层
- 可以与任何连接实现配合使用
- 不涉及连接算法的核心逻辑

---

### 2.4 join_key_evaluator.rs

**不需要修改的原因**：
- 键求值器已经足够通用，可以支持各种连接类型
- 提供了必要的表达式求值功能
- 统一实现可以直接使用现有的 `JoinKeyEvaluator`

---

## 3. 需要新建的文件

### 3.1 unified_outer_join.rs

**文件路径**：
`src/query/executor/data_processing/join/unified_outer_join.rs`

**文件内容**：
包含 `UnifiedOuterJoinExecutor` 的完整实现，详见 `unified_implementation_plan.md` 中的设计。

**主要组件**：
- `UnifiedOuterJoinExecutor` 结构体
- `execute_outer_join` 方法
- `precompute_column_maps` 方法
- `determine_build_probe_tables` 方法
- `build_hash_table` 方法
- `perform_join` 方法
- `handle_unmatched_rows` 方法
- `handle_full_outer_unmatched` 方法
- `is_build_row_matched` 方法

---

## 4. 修改优先级

### 高优先级（必须修改）
1. **hash_table.rs** - 核心基础设施，必须首先修改
2. **unified_outer_join.rs** - 新建文件，实现统一的外连接执行器

### 中优先级（建议修改）
3. **mod.rs** - 添加新模块导出，保持向后兼容
4. **base_join.rs** - 添加辅助方法，提高代码复用

### 低优先级（可选修改）
5. **left_join.rs** - 在迁移阶段修改，内部使用统一实现
6. **right_join.rs** - 在迁移阶段修改，内部使用统一实现
7. **full_outer_join.rs** - 在迁移阶段修改，内部使用统一实现

---

## 5. 修改风险评估

### 5.1 高风险修改
- **hash_table.rs**：核心基础设施，修改可能影响所有连接实现
  - 缓解措施：保持向后兼容性，添加充分的单元测试

### 5.2 中风险修改
- **mod.rs**：模块导出修改，可能影响外部依赖
  - 缓解措施：保持现有导出，只添加新导出

### 5.3 低风险修改
- **base_join.rs**：只添加新方法，不修改现有方法
- **新建文件**：不影响现有代码

---

## 6. 测试策略

### 6.1 单元测试
- 为 `hash_table.rs` 的新方法添加单元测试
- 为 `unified_outer_join.rs` 添加完整的单元测试
- 测试覆盖率达到 80% 以上

### 6.2 集成测试
- 测试统一实现与现有实现的兼容性
- 测试不同连接类型的正确性
- 测试性能基准

### 6.3 回归测试
- 确保现有功能不受影响
- 对比修改前后的结果
- 验证性能不下降

---

## 7. 总结

本评估确定了统一实现方案所需的文件修改：

**必须修改的文件**：
1. `hash_table.rs` - 添加 `UnifiedHashTable` 和优化方法
2. 新建 `unified_outer_join.rs` - 实现统一的外连接执行器

**建议修改的文件**：
3. `mod.rs` - 添加新模块导出
4. `base_join.rs` - 添加辅助方法

**不需要修改的文件**：
- `inner_join.rs` - 内连接独立实现
- `cross_join.rs` - 笛卡尔积完全不同
- `parallel.rs` - 并行处理独立模块
- `join_key_evaluator.rs` - 已足够通用

**迁移阶段修改的文件**：
- `left_join.rs` - 内部使用统一实现
- `right_join.rs` - 内部使用统一实现
- `full_outer_join.rs` - 内部使用统一实现

所有修改都遵循向后兼容性原则，确保现有代码不受影响。