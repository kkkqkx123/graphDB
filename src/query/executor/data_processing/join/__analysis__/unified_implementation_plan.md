# Join实现统一方案

## 1. 问题分析

### 1.1 当前实现差异

**左连接 (left_join.rs):**
- 使用 `HashTableBuilder` 和 `HashTableProbe` 抽象层
- 依赖专门的哈希表类型 (`SingleKeyHashTable`, `MultiKeyHashTable`)
- 支持单键和多键连接的分离实现
- 架构清晰，职责分离

**右连接 (right_join.rs):**
- 手动实现哈希表构建逻辑
- 使用标准 `HashMap<Value, Vec<Vec<Value>>>`
- 通过 `extract_key_values()` 辅助函数优化性能
- 单函数处理所有键类型

### 1.2 性能对比

**左连接优势：**
- 专业化实现，针对不同键数优化
- 架构抽象可能带来更好的扩展性

**右连接优势：**
- 预计算列映射，避免重复查找
- 辅助函数减少代码重复
- 更直接的控制流

### 1.3 统一必要性评估

**需要统一的理由：**
1. **维护性**：两种完全不同的架构增加维护复杂度
2. **一致性**：相似的算法应该有一致的实现方式
3. **性能优化**：可以结合两种实现的优势
4. **代码复用**：减少重复逻辑

**不需要统一的理由：**
1. **功能正确**：当前实现都能正确工作
2. **性能差异**：右连接的性能优化已经实现
3. **开发成本**：统一需要大量重构工作

**结论**：需要统一，但采用渐进式重构策略

## 2. 统一方案设计

### 2.1 设计原则

1. **向后兼容**：保持现有 API 不变
2. **渐进式重构**：分阶段实施，降低风险
3. **性能优先**：保留右连接的性能优化
4. **架构清晰**：保持抽象层的优势

### 2.2 统一架构

```
统一架构层次：
┌─────────────────────────────────────┐
│   JoinExecutor (统一接口)            │
├─────────────────────────────────────┤
│   OuterJoinExecutor (外连接实现)     │
│   InnerJoinExecutor (内连接实现)     │
├─────────────────────────────────────┤
│   OptimizedHashTable (优化哈希表)    │
│   JoinKeyEvaluator (键评估器)       │
├─────────────────────────────────────┤
│   BaseJoinExecutor (基础功能)        │
└─────────────────────────────────────┘
```

### 2.3 核心组件设计

#### 2.3.1 优化的哈希表构建器

```rust
/// 优化的哈希表构建器
pub struct OptimizedHashTableBuilder;

impl OptimizedHashTableBuilder {
    /// 构建哈希表（统一接口）
    pub fn build_table(
        dataset: &DataSet,
        key_exprs: &[Expression],
        col_map: &HashMap<&str, usize>,
    ) -> DBResult<UnifiedHashTable> {
        let mut hash_table = HashMap::new();

        for row in &dataset.rows {
            let key_values = extract_key_values(row, &dataset.col_names, key_exprs, col_map);
            let key = JoinKey::new(key_values);

            hash_table.entry(key)
                .or_insert_with(Vec::new)
                .push(row.clone());
        }

        Ok(UnifiedHashTable::new(hash_table))
    }
}

/// 统一的哈希表
pub struct UnifiedHashTable {
    inner: HashMap<JoinKey, Vec<Vec<Value>>>,
}

impl UnifiedHashTable {
    pub fn new(inner: HashMap<JoinKey, Vec<Vec<Value>>>) -> Self {
        Self { inner }
    }

    pub fn get(&self, key: &JoinKey) -> Option<&Vec<Vec<Value>>> {
        self.inner.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&JoinKey, &Vec<Vec<Value>>)> {
        self.inner.iter()
    }
}
```

#### 2.3.2 统一的外连接执行器

```rust
/// 统一的外连接执行器
pub struct UnifiedOuterJoinExecutor<S: StorageEngine> {
    base_executor: BaseJoinExecutor<S>,
    /// 连接类型（左/右/全）
    join_type: JoinType,
    /// 左侧数据集列数
    left_col_size: usize,
    /// 右侧数据集列数
    right_col_size: usize,
    /// 哈希表
    hash_table: Option<UnifiedHashTable>,
    /// 列名到索引的映射
    left_col_map: HashMap<String, usize>,
    right_col_map: HashMap<String, usize>,
}

impl<S: StorageEngine> UnifiedOuterJoinExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        left_var: String,
        right_var: String,
        hash_keys: Vec<Expression>,
        probe_keys: Vec<Expression>,
        col_names: Vec<String>,
        join_type: JoinType,
    ) -> Self {
        Self {
            base_executor: BaseJoinExecutor::new(
                id, storage, left_var, right_var, hash_keys, probe_keys, col_names,
            ),
            join_type,
            left_col_size: 0,
            right_col_size: 0,
            hash_table: None,
            left_col_map: HashMap::new(),
            right_col_map: HashMap::new(),
        }
    }

    /// 执行外连接（统一实现）
    fn execute_outer_join(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) -> DBResult<DataSet> {
        // 预计算列映射
        self.precompute_column_maps(left_dataset, right_dataset);

        // 根据连接类型确定构建表和探测表
        let (build_dataset, probe_dataset, build_keys, probe_keys) = 
            self.determine_build_probe_tables(left_dataset, right_dataset)?;

        // 构建哈希表
        let hash_table = self.build_hash_table(build_dataset, build_keys)?;

        // 执行连接
        self.perform_join(&hash_table, probe_dataset, probe_keys)
    }

    /// 预计算列映射
    fn precompute_column_maps(
        &mut self,
        left_dataset: &DataSet,
        right_dataset: &DataSet,
    ) {
        self.left_col_map = left_dataset.col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect();

        self.right_col_map = right_dataset.col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.clone(), i))
            .collect();

        self.left_col_size = left_dataset.col_names.len();
        self.right_col_size = right_dataset.col_names.len();
    }

    /// 确定构建表和探测表
    fn determine_build_probe_tables<'a>(
        &self,
        left_dataset: &'a DataSet,
        right_dataset: &'a DataSet,
    ) -> DBResult<(&'a DataSet, &'a DataSet, &[Expression], &[Expression])> {
        match self.join_type {
            JoinType::Left => {
                // 左连接：右表构建，左表探测
                Ok((right_dataset, left_dataset, 
                    &self.base_executor.probe_keys, 
                    &self.base_executor.hash_keys))
            }
            JoinType::Right => {
                // 右连接：左表构建，右表探测
                Ok((left_dataset, right_dataset,
                    &self.base_executor.hash_keys,
                    &self.base_executor.probe_keys))
            }
            JoinType::Full => {
                // 全外连接：右表构建，左表探测
                Ok((right_dataset, left_dataset,
                    &self.base_executor.probe_keys,
                    &self.base_executor.hash_keys))
            }
            _ => Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "不支持的外连接类型".to_string()
                )
            ))
        }
    }

    /// 构建哈希表
    fn build_hash_table(
        &self,
        dataset: &DataSet,
        key_exprs: &[Expression],
    ) -> DBResult<UnifiedHashTable> {
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

        Ok(UnifiedHashTable::new(hash_table))
    }

    /// 执行连接
    fn perform_join(
        &self,
        hash_table: &UnifiedHashTable,
        probe_dataset: &DataSet,
        probe_keys: &[Expression],
    ) -> DBResult<DataSet> {
        let mut result = DataSet::new();
        result.col_names = self.base_executor.get_col_names().clone();

        let col_map: HashMap<&str, usize> = probe_dataset.col_names.iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let mut matched_probe_rows = std::collections::HashSet::new();

        // 探测哈希表
        for probe_row in &probe_dataset.rows {
            let key_values = extract_key_values(probe_row, &probe_dataset.col_names, probe_keys, &col_map);
            let key = JoinKey::new(key_values);

            if let Some(matching_rows) = hash_table.get(&key) {
                matched_probe_rows.insert(probe_row.clone());

                for build_row in matching_rows {
                    let new_row = match self.join_type {
                        JoinType::Left | JoinType::Right => {
                            self.base_executor.new_row(probe_row.clone(), build_row.clone())
                        }
                        JoinType::Full => {
                            self.base_executor.new_row(probe_row.clone(), build_row.clone())
                        }
                        _ => unreachable!(),
                    };
                    result.rows.push(new_row);
                }
            }
        }

        // 处理未匹配的行
        self.handle_unmatched_rows(&mut result, probe_dataset, &matched_probe_rows);

        Ok(result)
    }

    /// 处理未匹配的行
    fn handle_unmatched_rows(
        &self,
        result: &mut DataSet,
        probe_dataset: &DataSet,
        matched_rows: &std::collections::HashSet<Vec<Value>>,
    ) {
        match self.join_type {
            JoinType::Left => {
                // 左连接：未匹配的左表行填充右表NULL
                for probe_row in &probe_dataset.rows {
                    if !matched_rows.contains(probe_row) {
                        let mut new_row = probe_row.clone();
                        for _ in 0..self.right_col_size {
                            new_row.push(Value::Null(NullType::Null));
                        }
                        result.rows.push(new_row);
                    }
                }
            }
            JoinType::Right => {
                // 右连接：未匹配的右表行填充左表NULL
                for probe_row in &probe_dataset.rows {
                    if !matched_rows.contains(probe_row) {
                        let mut new_row = Vec::new();
                        for _ in 0..self.left_col_size {
                            new_row.push(Value::Null(NullType::Null));
                        }
                        new_row.extend(probe_row.clone());
                        result.rows.push(new_row);
                    }
                }
            }
            JoinType::Full => {
                // 全外连接：需要额外处理构建表未匹配的行
                self.handle_full_outer_unmatched(result, probe_dataset, matched_rows);
            }
            _ => {}
        }
    }

    /// 处理全外连接未匹配的行
    fn handle_full_outer_unmatched(
        &self,
        result: &mut DataSet,
        probe_dataset: &DataSet,
        matched_probe_rows: &std::collections::HashSet<Vec<Value>>,
    ) {
        // 未匹配的探测表行
        for probe_row in &probe_dataset.rows {
            if !matched_probe_rows.contains(probe_row) {
                let mut new_row = probe_row.clone();
                for _ in 0..self.right_col_size {
                    new_row.push(Value::Null(NullType::Null));
                }
                result.rows.push(new_row);
            }
        }

        // 未匹配的构建表行（需要额外遍历哈希表）
        if let Some(ref hash_table) = self.hash_table {
            for (key, build_rows) in hash_table.iter() {
                let mut all_matched = true;
                for build_row in build_rows {
                    if !self.is_build_row_matched(build_row, matched_probe_rows) {
                        all_matched = false;
                        break;
                    }
                }

                if !all_matched {
                    for build_row in build_rows {
                        let mut new_row = Vec::new();
                        for _ in 0..self.left_col_size {
                            new_row.push(Value::Null(NullType::Null));
                        }
                        new_row.extend(build_row.clone());
                        result.rows.push(new_row);
                    }
                }
            }
        }
    }

    /// 检查构建表行是否已匹配
    fn is_build_row_matched(
        &self,
        _build_row: &[Value],
        _matched_probe_rows: &std::collections::HashSet<Vec<Value>>,
    ) -> bool {
        // 实现需要跟踪匹配的构建表行
        // 这里简化处理，实际实现需要更复杂的逻辑
        false
    }
}
```

#### 2.3.3 辅助函数

```rust
/// 从行中提取连接键的值（统一实现）
fn extract_key_values(
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
```

## 3. 实施计划

### 3.1 阶段一：基础设施（第1-2周）

**目标**：创建统一的基础组件

1. 创建 `optimized_hash_table.rs`
   - 实现 `OptimizedHashTableBuilder`
   - 实现 `UnifiedHashTable`
   - 添加单元测试

2. 创建 `unified_outer_join.rs`
   - 实现 `UnifiedOuterJoinExecutor`
   - 实现辅助函数
   - 添加单元测试

3. 更新 `mod.rs`
   - 添加新模块导出
   - 保持向后兼容

### 3.2 阶段二：迁移右连接（第3-4周）

**目标**：将右连接迁移到统一实现

1. 修改 `RightJoinExecutor`
   - 内部使用 `UnifiedOuterJoinExecutor`
   - 保持外部 API 不变
   - 添加兼容性测试

2. 性能验证
   - 对比迁移前后的性能
   - 确保性能不下降
   - 优化热点代码

### 3.3 阶段三：迁移左连接（第5-6周）

**目标**：将左连接迁移到统一实现

1. 修改 `LeftJoinExecutor`
   - 内部使用 `UnifiedOuterJoinExecutor`
   - 保持外部 API 不变
   - 添加兼容性测试

2. 性能验证
   - 对比迁移前后的性能
   - 确保性能不下降
   - 优化热点代码

### 3.4 阶段四：迁移全外连接（第7-8周）

**目标**：将全外连接迁移到统一实现

1. 修改 `FullOuterJoinExecutor`
   - 内部使用 `UnifiedOuterJoinExecutor`
   - 保持外部 API 不变
   - 添加兼容性测试

2. 性能验证
   - 对比迁移前后的性能
   - 确保性能不下降
   - 优化热点代码

### 3.5 阶段五：清理和优化（第9-10周）

**目标**：清理旧代码，优化性能

1. 删除重复代码
   - 移除旧的哈希表构建逻辑
   - 统一辅助函数
   - 清理未使用的代码

2. 性能优化
   - 优化内存使用
   - 减少不必要的拷贝
   - 优化热点路径

3. 文档更新
   - 更新架构文档
   - 添加性能基准测试
   - 更新使用示例

## 4. 风险评估

### 4.1 技术风险

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 性能下降 | 高 | 中 | 性能基准测试，分阶段验证 |
| 兼容性问题 | 中 | 低 | 保持 API 不变，全面测试 |
| 引入新 bug | 高 | 中 | 充分的单元测试和集成测试 |

### 4.2 项目风险

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 开发周期延长 | 中 | 中 | 分阶段实施，及时调整计划 |
| 资源不足 | 中 | 低 | 优先级管理，必要时调整范围 |

## 5. 预期收益

### 5.1 代码质量

- 减少约 40% 的重复代码
- 统一的架构风格
- 更好的可维护性

### 5.2 性能

- 统一的性能优化
- 减少不必要的拷贝
- 更好的内存使用

### 5.3 可扩展性

- 更容易添加新的连接类型
- 更好的抽象层次
- 更清晰的职责分离

## 6. 其他文件修改建议

### 6.1 需要修改的文件

1. **hash_table.rs**
   - 添加 `UnifiedHashTable` 类型
   - 保留现有的 `SingleKeyHashTable` 和 `MultiKeyHashTable` 以保持兼容性
   - 添加性能优化的构建方法

2. **mod.rs**
   - 添加新模块导出
   - 更新 `JoinExecutorFactory` 以支持统一实现
   - 保持向后兼容的 API

3. **base_join.rs**
   - 可能需要添加一些辅助方法
   - 保持现有接口不变

### 6.2 不需要修改的文件

1. **inner_join.rs** - 内连接实现相对独立，暂不涉及
2. **cross_join.rs** - 笛卡尔积实现完全不同，不涉及
3. **parallel.rs** - 并行处理模块，可以独立优化

## 7. 测试策略

### 7.1 单元测试

- 为每个新组件编写单元测试
- 测试覆盖率达到 80% 以上
- 包括边界条件和错误处理

### 7.2 集成测试

- 测试不同连接类型的正确性
- 测试性能基准
- 测试大数据集处理

### 7.3 回归测试

- 确保现有功能不受影响
- 对比迁移前后的结果
- 验证性能不下降

## 8. 总结

本方案提供了一个渐进式的统一实现策略，旨在：

1. 保持向后兼容性
2. 统一架构风格
3. 保留性能优化
4. 提高代码质量

通过分阶段实施，可以降低风险，确保每个阶段都有明确的验收标准。预计整个重构周期为 10 周，最终将实现一个统一、高效、易维护的 Join 实现架构。