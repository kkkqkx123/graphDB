# TopN 执行器模块设计文档

**文档版本**: 1.0  
**创建日期**: 2025-12-30  
**参考源**: NebulaGraph 3.8.0  
**模块位置**: `src/query/executor/result_processing/topn.rs`

## 1. 模块概述

TopN 执行器模块负责实现高效的 TopN 查询功能，基于堆数据结构对输入数据进行排序并返回前 N 条记录，为查询优化提供高性能的排序和限制操作。

### 1.1 核心职责
- 对输入数据流进行排序
- 维护固定大小的最大堆或最小堆
- 高效处理大规模数据集的 TopN 查询
- 支持多列排序和自定义排序规则

### 1.2 设计原则
- **高效性**: 使用堆数据结构实现 O(N log K) 时间复杂度
- **内存优化**: 仅维护前 N 条记录，减少内存占用
- **可配置性**: 支持升序/降序、多列排序等配置
- **稳定性**: 保证相同排序值记录的相对顺序

## 2. 架构设计

### 2.1 核心数据结构

```rust
// src/query/executor/result_processing/topn.rs
pub struct TopNExecutor<S: StorageEngine> {
    // 执行计划节点
    plan_node: Arc<TopNPlanNode>,
    
    // 存储引擎
    storage: Arc<Mutex<S>>,
    
    // 执行上下文
    context: ExecutionContext,
    
    // 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
    
    // 堆数据结构（最大堆或最小堆）
    heap: BinaryHeap<TopNItem>,
    
    // 配置参数
    limit: usize,
    sort_columns: Vec<SortColumn>,
    sort_direction: SortDirection,
    
    // 状态管理
    is_open: bool,
    is_closed: bool,
}

// TopN 堆项数据结构
pub struct TopNItem {
    // 排序值（用于堆比较）
    sort_value: Vec<Value>,
    
    // 原始索引（用于稳定性保证）
    _original_index: usize,
    
    // 实际数据行
    row: Vec<Value>,
}

// 排序方向枚举
pub enum SortDirection {
    Ascending,
    Descending,
}

// 排序列定义
pub struct SortColumn {
    column_index: usize,
    data_type: DataType,
    nulls_first: bool,
}
```

### 2.2 接口设计

```rust
// TopN 执行器接口
trait TopNExecutorTrait<S: StorageEngine>: Executor<S> {
    // 设置输入执行器
    fn set_input(&mut self, executor: Box<dyn Executor<S>>) -> Result<(), TopNError>;
    
    // 配置排序参数
    fn configure_sorting(
        &mut self,
        sort_columns: Vec<SortColumn>,
        direction: SortDirection,
    ) -> Result<(), TopNError>;
    
    // 堆管理接口
    fn push_to_heap(&mut self, item: TopNItem) -> Result<(), TopNError>;
    fn pop_from_heap(&mut self) -> Option<TopNItem>;
    
    // 性能监控
    fn get_heap_size(&self) -> usize;
    fn get_processed_count(&self) -> usize;
}

// TopN 项比较接口
impl Ord for TopNItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // 多列排序比较逻辑
        for (self_val, other_val) in self.sort_value.iter().zip(other.sort_value.iter()) {
            match self_val.cmp(other_val) {
                Ordering::Equal => continue,
                ordering => return ordering,
            }
        }
        
        // 稳定性保证：使用原始索引
        self._original_index.cmp(&other._original_index)
    }
}
```

## 3. NebulaGraph 对标分析

### 3.1 NebulaGraph 实现参考

NebulaGraph 中的 TopN 功能主要通过以下机制实现：

```cpp
// nebula-3.8.0/src/graph/executor/query/TopNExecutor.h
class TopNExecutor final : public Executor {
public:
    TopNExecutor(const PlanNode* node, QueryContext* qctx);
    
    folly::Future<Status> execute() override;
    
private:
    // 堆管理
    void pushToHeap(Row row);
    void popFromHeap();
    
    // 排序比较
    int compareRows(const Row& lhs, const Row& rhs) const;
    
    // 配置参数
    size_t limit_;
    std::vector<OrderByCol> orderByCols_;
    
    // 堆数据结构
    std::priority_queue<Row, std::vector<Row>, RowComparator> heap_;
};

// 行比较器
struct RowComparator {
    bool operator()(const Row& lhs, const Row& rhs) const {
        // 多列排序逻辑
        for (const auto& col : orderByCols_) {
            auto cmp = compareValues(lhs[col.index], rhs[col.index]);
            if (cmp != 0) return col.desc ? cmp > 0 : cmp < 0;
        }
        return false;
    }
};
```

### 3.2 关键差异与改进

| 特性 | NebulaGraph | GraphDB 设计 | 改进点 |
|------|-------------|--------------|--------|
| 堆实现 | std::priority_queue | BinaryHeap | 更灵活的堆操作 |
| 稳定性 | 不保证稳定性 | 使用原始索引保证 | 更好的结果一致性 |
| 内存管理 | 手动管理 | RAII 自动管理 | 更安全的内存使用 |
| 错误处理 | 异常机制 | Result 类型 | 更安全的错误处理 |

## 4. 核心功能实现

### 4.1 执行器初始化

```rust
impl<S: StorageEngine> TopNExecutor<S> {
    pub fn new(
        plan_node: Arc<TopNPlanNode>,
        storage: Arc<Mutex<S>>,
        context: ExecutionContext,
    ) -> Self {
        let limit = plan_node.limit;
        let sort_columns = plan_node.sort_columns.clone();
        let sort_direction = plan_node.sort_direction;
        
        TopNExecutor {
            plan_node,
            storage,
            context,
            input_executor: None,
            heap: BinaryHeap::with_capacity(limit + 1), // 预留一个位置
            limit,
            sort_columns,
            sort_direction,
            is_open: false,
            is_closed: false,
        }
    }
    
    // 配置输入执行器
    pub fn set_input(&mut self, executor: Box<dyn Executor<S>>) -> Result<(), TopNError> {
        if self.is_open {
            return Err(TopNError::ExecutorAlreadyOpen);
        }
        
        self.input_executor = Some(executor);
        Ok(())
    }
}
```

### 4.2 执行流程实现

```rust
impl<S: StorageEngine> Executor<S> for TopNExecutor<S> {
    async fn execute(&mut self) -> Result<QueryResult, QueryError> {
        // 1. 打开执行器
        self.open().await?;
        
        // 2. 处理输入数据
        let input_result = self.process_input().await?;
        
        // 3. 构建最终结果
        let final_result = self.build_final_result().await?;
        
        // 4. 关闭执行器
        self.close().await?;
        
        Ok(final_result)
    }
    
    async fn open(&mut self) -> Result<(), QueryError> {
        if self.is_open {
            return Err(QueryError::ExecutorAlreadyOpen);
        }
        
        // 验证输入执行器
        if self.input_executor.is_none() {
            return Err(QueryError::MissingInputExecutor);
        }
        
        // 打开输入执行器
        self.input_executor.as_mut().unwrap().open().await?;
        
        self.is_open = true;
        Ok(())
    }
    
    async fn close(&mut self) -> Result<(), QueryError> {
        if !self.is_open || self.is_closed {
            return Ok(());
        }
        
        // 关闭输入执行器
        if let Some(ref mut input) = self.input_executor {
            input.close().await?;
        }
        
        // 清理堆数据
        self.heap.clear();
        
        self.is_closed = true;
        Ok(())
    }
}
```

### 4.3 数据处理逻辑

```rust
impl<S: StorageEngine> TopNExecutor<S> {
    async fn process_input(&mut self) -> Result<(), TopNError> {
        let mut input = self.input_executor.as_mut().unwrap();
        let mut processed_count = 0;
        
        // 处理输入数据流
        while let Some(row) = input.next().await? {
            processed_count += 1;
            
            // 提取排序值
            let sort_value = self.extract_sort_values(&row)?;
            
            // 创建 TopN 项
            let item = TopNItem {
                sort_value,
                _original_index: processed_count, // 用于稳定性保证
                row,
            };
            
            // 推入堆中
            self.push_to_heap(item)?;
            
            // 内存限制检查
            if self.exceeds_memory_limit() {
                return Err(TopNError::MemoryLimitExceeded);
            }
        }
        
        Ok(())
    }
    
    fn push_to_heap(&mut self, item: TopNItem) -> Result<(), TopNError> {
        // 根据排序方向调整比较逻辑
        let adjusted_item = match self.sort_direction {
            SortDirection::Ascending => item, // 最小堆
            SortDirection::Descending => {
                // 对于最大堆，需要反转比较逻辑
                TopNItem {
                    sort_value: self.invert_sort_values(item.sort_value)?,
                    _original_index: item._original_index,
                    row: item.row,
                }
            }
        };
        
        // 推入堆中
        self.heap.push(adjusted_item);
        
        // 如果堆大小超过限制，弹出最小/最大项
        if self.heap.len() > self.limit {
            self.heap.pop();
        }
        
        Ok(())
    }
    
    async fn build_final_result(&mut self) -> Result<QueryResult, QueryError> {
        // 从堆中提取结果（需要反转顺序）
        let mut results: Vec<Vec<Value>> = Vec::with_capacity(self.heap.len());
        
        while let Some(item) = self.heap.pop() {
            // 根据排序方向恢复原始顺序
            let final_row = match self.sort_direction {
                SortDirection::Ascending => item.row,
                SortDirection::Descending => item.row, // 已经是正确顺序
            };
            
            results.push(final_row);
        }
        
        // 反转结果以获得正确顺序
        if self.sort_direction == SortDirection::Ascending {
            results.reverse();
        }
        
        Ok(QueryResult::new(results))
    }
}
```

### 4.4 排序值处理

```rust
impl<S: StorageEngine> TopNExecutor<S> {
    fn extract_sort_values(&self, row: &[Value]) -> Result<Vec<Value>, TopNError> {
        let mut sort_values = Vec::with_capacity(self.sort_columns.len());
        
        for sort_col in &self.sort_columns {
            if sort_col.column_index >= row.len() {
                return Err(TopNError::InvalidColumnIndex(sort_col.column_index));
            }
            
            let value = &row[sort_col.column_index];
            
            // 处理 NULL 值排序
            let adjusted_value = if value.is_null() {
                if sort_col.nulls_first {
                    Value::min_value() // NULL 排在最前
                } else {
                    Value::max_value() // NULL 排在最后
                }
            } else {
                value.clone()
            };
            
            sort_values.push(adjusted_value);
        }
        
        Ok(sort_values)
    }
    
    fn invert_sort_values(&self, mut sort_values: Vec<Value>) -> Result<Vec<Value>, TopNError> {
        for value in &mut sort_values {
            if !value.is_null() {
                // 反转值的比较逻辑（用于最大堆）
                *value = value.invert_for_sorting()?;
            }
        }
        
        Ok(sort_values)
    }
}
```

## 5. 性能优化

### 5.1 堆大小优化

```rust
impl<S: StorageEngine> TopNExecutor<S> {
    // 动态调整堆容量
    fn optimize_heap_capacity(&mut self) {
        let current_capacity = self.heap.capacity();
        let ideal_capacity = self.limit + 10; // 预留一些空间
        
        if current_capacity > ideal_capacity * 2 {
            // 堆容量过大，进行收缩
            let mut new_heap = BinaryHeap::with_capacity(ideal_capacity);
            while let Some(item) = self.heap.pop() {
                new_heap.push(item);
                if new_heap.len() >= self.limit {
                    break;
                }
            }
            self.heap = new_heap;
        }
    }
    
    // 内存使用监控
    fn exceeds_memory_limit(&self) -> bool {
        let estimated_memory = self.heap.len() * 100; // 估算每项100字节
        estimated_memory > 100 * 1024 * 1024 // 100MB 限制
    }
}
```

### 5.2 批量处理优化

```rust
impl<S: StorageEngine> TopNExecutor<S> {
    // 批量处理输入数据
    async fn process_input_batch(&mut self, batch_size: usize) -> Result<(), TopNError> {
        let mut input = self.input_executor.as_mut().unwrap();
        let mut batch = Vec::with_capacity(batch_size);
        
        while let Some(row) = input.next().await? {
            batch.push(row);
            
            if batch.len() >= batch_size {
                self.process_batch(&batch).await?;
                batch.clear();
            }
        }
        
        // 处理剩余数据
        if !batch.is_empty() {
            self.process_batch(&batch).await?;
        }
        
        Ok(())
    }
    
    async fn process_batch(&mut self, batch: &[Vec<Value>]) -> Result<(), TopNError> {
        for row in batch {
            let sort_value = self.extract_sort_values(row)?;
            let item = TopNItem {
                sort_value,
                _original_index: 0, // 批量处理时索引不重要
                row: row.clone(),
            };
            
            self.push_to_heap(item)?;
        }
        
        Ok(())
    }
}
```

## 6. 错误处理设计

### 6.1 错误类型定义

```rust
#[derive(Debug, thiserror::Error)]
pub enum TopNError {
    #[error("执行器已打开")]
    ExecutorAlreadyOpen,
    
    #[error("内存限制超出")]
    MemoryLimitExceeded,
    
    #[error("无效的列索引: {0}")]
    InvalidColumnIndex(usize),
    
    #[error("排序值提取失败: {0}")]
    SortValueExtractionFailed(String),
    
    #[error("堆操作失败: {0}")]
    HeapOperationFailed(String),
    
    #[error("输入执行器错误: {0}")]
    InputExecutorError(#[from] QueryError),
}
```

### 6.2 错误恢复策略

```rust
impl<S: StorageEngine> TopNExecutor<S> {
    // 优雅的错误恢复
    async fn execute_with_recovery(&mut self) -> Result<QueryResult, TopNError> {
        match self.execute().await {
            Ok(result) => Ok(result),
            Err(TopNError::MemoryLimitExceeded) => {
                // 内存不足时，使用外部排序
                self.fallback_to_external_sort().await
            }
            Err(e) => Err(e),
        }
    }
    
    // 外部排序降级方案
    async fn fallback_to_external_sort(&mut self) -> Result<QueryResult, TopNError> {
        // 实现外部排序逻辑
        // 将数据写入临时文件，然后进行归并排序
        // 返回前 N 条记录
        
        // 简化实现：直接返回错误，提示用户调整查询
        Err(TopNError::MemoryLimitExceeded)
    }
}
```

## 7. 测试策略

### 7.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_topn_basic_operation() {
        let executor = create_test_topn_executor(5, SortDirection::Descending);
        let input_data = generate_test_data(100);
        
        let result = executor.process_data(input_data).await.unwrap();
        assert_eq!(result.len(), 5);
        assert!(is_sorted_descending(&result));
    }
    
    #[test]
    fn test_heap_management() {
        let mut executor = create_test_topn_executor(3, SortDirection::Ascending);
        
        // 测试堆的插入和弹出
        for i in 0..10 {
            let item = create_test_item(i);
            executor.push_to_heap(item).unwrap();
        }
        
        assert_eq!(executor.heap.len(), 3);
    }
    
    #[test]
    fn test_sort_value_extraction() {
        let executor = create_test_topn_executor(5, SortDirection::Ascending);
        let row = vec![Value::Int(42), Value::String("test".to_string())];
        
        let sort_values = executor.extract_sort_values(&row).unwrap();
        assert_eq!(sort_values.len(), 2);
    }
}
```

### 7.2 性能测试

```rust
#[cfg(test)]
mod bench {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_topn_large_dataset(c: &mut Criterion) {
        c.bench_function("topn_100k_records", |b| {
            b.iter(|| {
                let mut executor = create_test_topn_executor(100, SortDirection::Descending);
                let data = generate_large_test_data(100_000);
                
                // 异步执行需要特殊处理
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime.block_on(async {
                    executor.process_data(black_box(data)).await.unwrap()
                })
            })
        });
    }
    
    criterion_group!(benches, bench_topn_large_dataset);
    criterion_main!(benches);
}
```

## 8. 未来扩展

### 8.1 计划功能
- 支持并行 TopN 处理
- 添加增量更新支持
- 支持分布式 TopN 查询

### 8.2 优化方向
- 自适应批处理大小
- 更高效的内存管理
- 支持流式 TopN 处理

## 9. 总结

TopN 执行器模块为 GraphDB 提供了 NebulaGraph 级别的高效 TopN 查询能力，通过优化的堆算法实现了 O(N log K) 的时间复杂度。该设计充分利用了 Rust 的所有权系统和类型安全特性，同时保持了与 NebulaGraph 架构的兼容性。通过稳定性保证和内存优化，该模块能够高效处理大规模数据集的排序和限制操作。