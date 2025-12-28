//! 优化的排序执行器
//!
//! 提供高性能排序功能，支持外部排序、并行排序和内存优化

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use std::fs::{File, create_dir_all};
use std::io::{Write, Read, BufWriter, BufReader};
use std::path::PathBuf;

use crate::core::error::{DBError, DBResult};
use crate::expression::{DefaultExpressionContext, ExpressionContext};
use crate::core::{DataSet, Value};
use crate::core::Expression;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::result_processing::traits::{
    BaseResultProcessor, ResultProcessor, ResultProcessorContext,
};
use crate::query::executor::traits::{
    ExecutionResult, Executor, HasStorage,
};
use crate::storage::StorageEngine;

/// 排序顺序枚举
#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// 排序键定义
#[derive(Debug, Clone)]
pub struct SortKey {
    pub expression: Expression,
    pub order: SortOrder,
}

impl SortKey {
    pub fn new(expression: Expression, order: SortOrder) -> Self {
        Self { expression, order }
    }
}

/// 排序算法类型
#[derive(Debug, Clone, PartialEq)]
pub enum SortAlgorithm {
    /// 快速排序（默认）
    QuickSort,
    /// 归并排序（稳定排序）
    MergeSort,
    /// 堆排序（内存高效）
    HeapSort,
    /// 外部排序（大数据集）
    ExternalSort,
    /// 并行快速排序
    ParallelQuickSort,
}

/// 排序配置
#[derive(Debug, Clone)]
pub struct SortConfig {
    /// 排序算法
    pub algorithm: SortAlgorithm,
    /// 内存限制（字节）
    pub memory_limit: usize,
    /// 块大小（用于外部排序）
    pub block_size: usize,
    /// 是否启用并行处理
    pub parallel: bool,
    /// 最大线程数
    pub max_threads: usize,
}

impl Default for SortConfig {
    fn default() -> Self {
        Self {
            algorithm: SortAlgorithm::QuickSort,
            memory_limit: 100 * 1024 * 1024, // 100MB 默认内存限制
            block_size: 64 * 1024, // 64KB 块大小
            parallel: true,
            max_threads: 4,
        }
    }
}

/// 优化的排序执行器
pub struct SortExecutor<S: StorageEngine + Send + 'static> {
    /// 基础处理器
    base: BaseResultProcessor<S>,
    /// 排序键列表
    sort_keys: Vec<SortKey>,
    /// 限制数量
    limit: Option<usize>,
    /// 输入执行器
    input_executor: Option<Box<dyn Executor<S>>>,
    /// 排序配置
    config: SortConfig,
    /// 当前内存使用量
    memory_usage: usize,
    /// 临时目录（用于外部排序）
    temp_dir: PathBuf,
}

impl<S: StorageEngine + Send + 'static> SortExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        sort_keys: Vec<SortKey>,
        limit: Option<usize>,
        config: SortConfig,
    ) -> DBResult<Self> {
        let base = BaseResultProcessor::new(
            id,
            "SortExecutor".to_string(),
            "High-performance sorting with external sort and parallel processing".to_string(),
            storage,
        );

        let temp_dir = std::env::temp_dir().join("graphdb_sort");
        create_dir_all(&temp_dir)?;

        Ok(Self {
            base,
            sort_keys,
            limit,
            input_executor: None,
            config,
            memory_usage: 0,
            temp_dir,
        })
    }

    /// 处理输入数据并排序
    async fn process_input(&mut self) -> DBResult<DataSet> {
        if let Some(ref mut input_exec) = self.input_executor {
            let input_result = input_exec.execute().await?;

            match input_result {
                ExecutionResult::DataSet(mut data_set) => {
                    // 根据数据集大小和配置选择合适的排序算法
                    self.select_and_execute_sort(&mut data_set)?;
                    Ok(data_set)
                }
                _ => Err(DBError::Query(
                    crate::core::error::QueryError::ExecutionError(
                        "Sort executor expects DataSet input".to_string(),
                    ),
                )),
            }
        } else {
            Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "Sort executor requires input executor".to_string(),
                ),
            ))
        }
    }

    /// 根据数据集特征选择并执行合适的排序算法
    fn select_and_execute_sort(&mut self, data_set: &mut DataSet) -> DBResult<()> {
        if self.sort_keys.is_empty() || data_set.rows.is_empty() {
            return Ok(());
        }

        // 估算内存使用
        let estimated_memory = self.estimate_memory_usage(data_set);
        self.memory_usage = estimated_memory;

        // 如果有LIMIT且数据量很大，使用Top-N算法
        if let Some(limit) = self.limit {
            if data_set.rows.len() > limit * 10 {
                return self.execute_top_n_sort(data_set, limit);
            }
        }

        // 根据内存使用和配置选择排序算法
        if estimated_memory > self.config.memory_limit {
            // 大数据集使用外部排序
            self.execute_external_sort(data_set)
        } else if self.config.parallel && data_set.rows.len() > 10000 {
            // 中等大小数据集使用并行排序
            self.execute_parallel_sort(data_set)
        } else if self.config.algorithm == SortAlgorithm::HeapSort {
            // 内存受限时使用堆排序
            self.execute_heap_sort(data_set)
        } else {
            // 默认使用快速排序
            self.execute_quick_sort(data_set)
        }
    }

    /// 估算数据集内存使用
    fn estimate_memory_usage(&self, data_set: &DataSet) -> usize {
        let row_size = std::mem::size_of::<Vec<Value>>() + 
                      data_set.col_names.len() * std::mem::size_of::<Value>();
        let sort_key_size = self.sort_keys.len() * std::mem::size_of::<Value>();
        
        data_set.rows.len() * (row_size + sort_key_size)
    }

    /// 执行Top-N排序（使用最小堆/最大堆）
    fn execute_top_n_sort(&mut self, data_set: &mut DataSet, n: usize) -> DBResult<()> {
        if n == 0 || data_set.rows.is_empty() {
            return Ok(());
        }

        let evaluator = ExpressionEvaluator;
        let mut heap = BinaryHeap::with_capacity(n);

        // 为每行计算排序键值并维护堆
        for row in &data_set.rows {
            let sort_values = self.calculate_sort_values(row, &data_set.col_names, &evaluator)?;
            
            // 获取排序方向（假设所有排序键方向相同）
            let sort_order = if !self.sort_keys.is_empty() {
                self.sort_keys[0].order.clone()
            } else {
                SortOrder::Asc // 默认升序
            };
            
            if heap.len() < n {
                    heap.push(SortItem::new(sort_values, row.clone(), sort_order));
                } else {
                    // 检查是否需要替换堆顶元素
                    if let Some(mut top) = heap.peek_mut() {
                        let new_item = SortItem::new(sort_values.clone(), row.clone(), sort_order.clone());
                        let comparison = new_item.cmp(&top);
                        // 对于最大堆，如果新元素比堆顶小（在堆的排序意义上），则替换
                        if comparison == Ordering::Less {
                            *top = new_item;
                        }
                    }
                }
        }

        // 提取排序结果
        let mut sorted_items: Vec<_> = heap.into_iter().collect();
        sorted_items.sort_by(|a, b| self.compare_sort_items(a, b));

        data_set.rows = sorted_items.into_iter().map(|item| item.row).collect();
        Ok(())
    }

    /// 执行快速排序
    fn execute_quick_sort(&mut self, data_set: &mut DataSet) -> DBResult<()> {
        let evaluator = ExpressionEvaluator;
        let mut rows_with_keys: Vec<(Vec<Value>, Vec<Value>)> = Vec::new();

        // 为每行计算排序键值
        for row in &data_set.rows {
            let sort_values = self.calculate_sort_values(row, &data_set.col_names, &evaluator)?;
            rows_with_keys.push((sort_values, row.clone()));
        }

        // 执行快速排序
        let len = rows_with_keys.len();
        if len > 0 {
            self.quick_sort_recursive(&mut rows_with_keys, 0, len - 1);
        }

        // 提取排序后的行
        data_set.rows = rows_with_keys.into_iter().map(|(_, row)| row).collect();
        Ok(())
    }

    /// 快速排序递归实现
    fn quick_sort_recursive(&self, items: &mut [(Vec<Value>, Vec<Value>)], low: usize, high: usize) {
        if low < high {
            let pivot = self.partition(items, low, high);
            if pivot > 0 {
                self.quick_sort_recursive(items, low, pivot - 1);
            }
            self.quick_sort_recursive(items, pivot + 1, high);
        }
    }

    /// 快速排序分区函数
    fn partition(&self, items: &mut [(Vec<Value>, Vec<Value>)], low: usize, high: usize) -> usize {
        let pivot = high;
        let mut i = low;

        for j in low..high {
            if self.compare_sort_items_vec(&items[j].0, &items[pivot].0) != Ordering::Greater {
                items.swap(i, j);
                i += 1;
            }
        }

        items.swap(i, pivot);
        i
    }

    /// 执行并行快速排序
    fn execute_parallel_sort(&mut self, data_set: &mut DataSet) -> DBResult<()> {
        let evaluator = ExpressionEvaluator;
        let mut rows_with_keys: Vec<(Vec<Value>, Vec<Value>)> = Vec::new();

        // 为每行计算排序键值
        for row in &data_set.rows {
            let sort_values = self.calculate_sort_values(row, &data_set.col_names, &evaluator)?;
            rows_with_keys.push((sort_values, row.clone()));
        }

        // 并行快速排序实现
        self.parallel_quick_sort(&mut rows_with_keys);

        // 提取排序后的行
        data_set.rows = rows_with_keys.into_iter().map(|(_, row)| row).collect();
        Ok(())
    }

    /// 并行快速排序
    fn parallel_quick_sort(&self, items: &mut [(Vec<Value>, Vec<Value>)]) {
        if items.len() <= 1000 {
            // 小数组使用普通快速排序
            self.quick_sort_recursive(items, 0, items.len().saturating_sub(1));
        } else {
            // 大数组使用并行处理
            let pivot = self.partition(items, 0, items.len().saturating_sub(1));
            
            // 并行处理左右两部分
            let (left, right) = items.split_at_mut(pivot);
            
            if !left.is_empty() && !right.is_empty() {
                // 这里可以使用rayon::join进行真正的并行处理
                // 目前使用顺序处理作为简化实现
                self.parallel_quick_sort(left);
                self.parallel_quick_sort(&mut right[1..]); // 跳过pivot
            } else if !left.is_empty() {
                self.parallel_quick_sort(left);
            } else if right.len() > 1 {
                self.parallel_quick_sort(&mut right[1..]);
            }
        }
    }

    /// 执行堆排序
    fn execute_heap_sort(&mut self, data_set: &mut DataSet) -> DBResult<()> {
        let evaluator = ExpressionEvaluator;
        let mut rows_with_keys: Vec<(Vec<Value>, Vec<Value>)> = Vec::new();

        // 为每行计算排序键值
        for row in &data_set.rows {
            let sort_values = self.calculate_sort_values(row, &data_set.col_names, &evaluator)?;
            rows_with_keys.push((sort_values, row.clone()));
        }

        // 执行堆排序
        self.heap_sort(&mut rows_with_keys);

        // 提取排序后的行
        data_set.rows = rows_with_keys.into_iter().map(|(_, row)| row).collect();
        Ok(())
    }

    /// 堆排序实现
    fn heap_sort(&self, items: &mut [(Vec<Value>, Vec<Value>)]) {
        let n = items.len();
        
        // 构建最大堆
        for i in (0..n / 2).rev() {
            self.heapify(items, n, i);
        }
        
        // 一个个从堆中取出元素
        for i in (1..n).rev() {
            items.swap(0, i);
            self.heapify(items, i, 0);
        }
    }

    /// 堆化函数
    fn heapify(&self, items: &mut [(Vec<Value>, Vec<Value>)], n: usize, i: usize) {
        let mut largest = i;
        let left = 2 * i + 1;
        let right = 2 * i + 2;

        if left < n && self.compare_sort_items_vec(&items[left].0, &items[largest].0) == Ordering::Greater {
            largest = left;
        }

        if right < n && self.compare_sort_items_vec(&items[right].0, &items[largest].0) == Ordering::Greater {
            largest = right;
        }

        if largest != i {
            items.swap(i, largest);
            self.heapify(items, n, largest);
        }
    }

    /// 执行外部排序（完整实现）
    fn execute_external_sort(&mut self, data_set: &mut DataSet) -> DBResult<()> {
        if data_set.rows.is_empty() {
            return Ok(());
        }

        let evaluator = ExpressionEvaluator;
        let mut temp_files = Vec::new();
        let mut current_block = Vec::new();
        let mut current_block_size = 0;
        let block_size = self.config.block_size;

        // 第一阶段：将数据分块并排序写入临时文件
        for row in &data_set.rows {
            let sort_values = self.calculate_sort_values(row, &data_set.col_names, &evaluator)?;
            let row_size = std::mem::size_of_val(row) + std::mem::size_of_val(&sort_values);
            
            current_block.push((sort_values, row.clone()));
            current_block_size += row_size;

            // 当块大小达到限制时，排序并写入临时文件
            if current_block_size >= block_size {
                self.sort_and_write_block(&mut current_block, &mut temp_files)?;
                current_block.clear();
                current_block_size = 0;
            }
        }

        // 处理剩余的块
        if !current_block.is_empty() {
            self.sort_and_write_block(&mut current_block, &mut temp_files)?;
        }

        // 第二阶段：多路归并
        if temp_files.is_empty() {
            return Ok(());
        } else if temp_files.len() == 1 {
            // 只有一个块，直接读取结果
            self.read_sorted_block(&temp_files[0], data_set)?;
        } else {
            // 多个块，执行多路归并
            self.multiway_merge(&temp_files, data_set)?;
        }

        // 清理临时文件
        for temp_file in &temp_files {
            std::fs::remove_file(temp_file).ok();
        }

        Ok(())
    }

    /// 对块进行排序并写入临时文件
    fn sort_and_write_block(&self, block: &mut [(Vec<Value>, Vec<Value>)], temp_files: &mut Vec<PathBuf>) -> DBResult<()> {
        // 对块进行排序
        block.sort_by(|a, b| self.compare_sort_items_vec(&a.0, &b.0));

        // 写入临时文件
        let temp_file = self.temp_dir.join(format!("sort_block_{}.tmp", temp_files.len()));
        let file = File::create(&temp_file)?;
        let mut writer = BufWriter::new(file);

        // 序列化并写入数据 - 克隆数据以避免借用问题
        let block_clone: Vec<(Vec<Value>, Vec<Value>)> = block.to_vec();
        let encoded = bincode::encode_to_vec(&block_clone, bincode::config::standard())
            .map_err(|e| DBError::Serialization(format!("Failed to encode sort block: {}", e)))?;
        writer.write_all(&encoded)?;
        writer.flush()?;

        temp_files.push(temp_file);
        Ok(())
    }

    /// 读取排序后的块
    fn read_sorted_block(&self, temp_file: &PathBuf, data_set: &mut DataSet) -> DBResult<()> {
        let file = File::open(temp_file)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let (block, _): (Vec<(Vec<Value>, Vec<Value>)>, _) = 
            bincode::decode_from_slice(&buffer, bincode::config::standard())
                .map_err(|e| DBError::Serialization(format!("Failed to decode sort block: {}", e)))?;

        data_set.rows = block.into_iter().map(|(_, row)| row).collect();
        Ok(())
    }

    /// 多路归并
    fn multiway_merge(&self, temp_files: &[PathBuf], data_set: &mut DataSet) -> DBResult<()> {
        // 读取所有块的第一部分到内存
        let mut readers = Vec::new();
        let mut current_items = Vec::new();

        for (i, temp_file) in temp_files.iter().enumerate() {
            let file = File::open(temp_file)?;
            let mut reader = BufReader::new(file);
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer)?;

            let (block, _): (Vec<(Vec<Value>, Vec<Value>)>, _) = 
                bincode::decode_from_slice(&buffer, bincode::config::standard())
                    .map_err(|e| DBError::Serialization(format!("Failed to decode sort block in merge: {}", e)))?;
            if !block.is_empty() {
                // 先克隆第一个元素，再将 block 移动到 readers 中
                let first_item = block[0].clone();
                readers.push((block, 0)); // (数据块, 当前索引)
                current_items.push((i, first_item.0, first_item.1));
            }
        }

        // 使用最小堆进行多路归并
        let mut result = Vec::new();
        let mut heap = BinaryHeap::new();

        // 初始化堆
        for (i, sort_values, row) in current_items {
            heap.push(Reverse(SortItem::new_with_source(sort_values, row, i, SortOrder::Asc)));
        }

        // 归并过程
        while let Some(Reverse(item)) = heap.pop() {
            result.push(item.row);
            
            // 从同一个源读取下一个元素
            let source_index = item.source_index;
            if let Some(ref mut reader) = readers.get_mut(source_index) {
                reader.1 += 1; // 移动到下一个元素
                if reader.1 < reader.0.len() {
                    let next_item = &reader.0[reader.1];
                    heap.push(Reverse(SortItem::new_with_source(
                        next_item.0.clone(),
                        next_item.1.clone(),
                        source_index,
                        SortOrder::Asc,
                    )));
                }
            }
        }

        data_set.rows = result;
        Ok(())
    }

    /// 执行归并排序
    fn execute_merge_sort(&mut self, data_set: &mut DataSet) -> DBResult<()> {
        let evaluator = ExpressionEvaluator;
        let mut rows_with_keys: Vec<(Vec<Value>, Vec<Value>)> = Vec::new();

        // 为每行计算排序键值
        for row in &data_set.rows {
            let sort_values = self.calculate_sort_values(row, &data_set.col_names, &evaluator)?;
            rows_with_keys.push((sort_values, row.clone()));
        }

        // 执行归并排序
        self.merge_sort(&mut rows_with_keys);

        // 提取排序后的行
        data_set.rows = rows_with_keys.into_iter().map(|(_, row)| row).collect();
        Ok(())
    }

    /// 归并排序实现
    fn merge_sort(&self, items: &mut [(Vec<Value>, Vec<Value>)]) {
        if items.len() <= 1 {
            return;
        }

        let mid = items.len() / 2;
        
        // 递归排序左右两部分
        let mut left: Vec<(Vec<Value>, Vec<Value>)> = items[..mid].to_vec();
        let mut right: Vec<(Vec<Value>, Vec<Value>)> = items[mid..].to_vec();
        
        self.merge_sort(&mut left);
        self.merge_sort(&mut right);
        
        // 合并结果
        let mut merged = self.merge_impl(&left, &right);
        
        // 将结果复制回原数组
        for (i, item) in merged.drain(..).enumerate() {
            items[i] = item;
        }
    }

    /// 合并两个已排序的数组
    fn merge_impl(&self, left: &[(Vec<Value>, Vec<Value>)], right: &[(Vec<Value>, Vec<Value>)]) -> Vec<(Vec<Value>, Vec<Value>)> {
        let mut temp = Vec::with_capacity(left.len() + right.len());
        
        let mut i = 0;
        let mut j = 0;
        
        while i < left.len() && j < right.len() {
            if self.compare_sort_items_vec(&left[i].0, &right[j].0) != Ordering::Greater {
                temp.push(left[i].clone());
                i += 1;
            } else {
                temp.push(right[j].clone());
                j += 1;
            }
        }
        
        while i < left.len() {
            temp.push(left[i].clone());
            i += 1;
        }
        
        while j < right.len() {
            temp.push(right[j].clone());
            j += 1;
        }
        
        temp
    }

    /// 计算行的排序键值
    fn calculate_sort_values(
        &self,
        row: &[Value],
        col_names: &[String],
        evaluator: &ExpressionEvaluator,
    ) -> DBResult<Vec<Value>> {
        let mut expr_context = DefaultExpressionContext::new();
        for (i, col_name) in col_names.iter().enumerate() {
            if i < row.len() {
                expr_context.set_variable(col_name.clone(), row[i].clone());
            }
        }

        let mut sort_values = Vec::new();
        for sort_key in &self.sort_keys {
            let sort_value = evaluator
                .evaluate(&sort_key.expression, &mut expr_context)
                .map_err(|e| {
                    DBError::Query(crate::core::error::QueryError::ExecutionError(
                        e.to_string(),
                    ))
                })?;
            sort_values.push(sort_value);
        }

        Ok(sort_values)
    }

    /// 比较两个排序项目
    fn compare_sort_items(&self, a: &SortItem, b: &SortItem) -> Ordering {
        self.compare_sort_items_vec(&a.sort_values, &b.sort_values)
    }

    /// 比较两个排序值向量
    fn compare_sort_items_vec(&self, a: &[Value], b: &[Value]) -> Ordering {
        for ((idx, sort_val_a), sort_val_b) in a.iter().enumerate().zip(b.iter()) {
            let comparison = self.compare_values(sort_val_a, sort_val_b, &self.sort_keys[idx].order);
            if !comparison.is_eq() {
                return comparison;
            }
        }
        Ordering::Equal
    }

    /// 比较两个值，根据排序方向
    fn compare_values(&self, a: &Value, b: &Value, order: &SortOrder) -> Ordering {
        let comparison = a.partial_cmp(b).unwrap_or(Ordering::Equal);

        match order {
            SortOrder::Asc => comparison,
            SortOrder::Desc => comparison.reverse(),
        }
    }
}

/// 排序项，用于堆排序
#[derive(Debug, Clone, PartialEq, Eq)]
struct SortItem {
    sort_values: Vec<Value>,
    row: Vec<Value>,
    source_index: usize, // 用于多路归并时标识数据来源
    sort_order: SortOrder, // 排序方向
}

impl SortItem {
    fn new(sort_values: Vec<Value>, row: Vec<Value>, sort_order: SortOrder) -> Self {
        Self { sort_values, row, source_index: 0, sort_order }
    }

    fn new_with_source(sort_values: Vec<Value>, row: Vec<Value>, source_index: usize, sort_order: SortOrder) -> Self {
        Self { sort_values, row, source_index, sort_order }
    }
}

impl PartialOrd for SortItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // 比较排序值，考虑排序方向
        for (a, b) in self.sort_values.iter().zip(other.sort_values.iter()) {
            let comparison = a.partial_cmp(b).unwrap_or(Ordering::Equal);
            if !comparison.is_eq() {
                // 根据排序方向调整比较结果
                return match self.sort_order {
                    SortOrder::Asc => comparison,
                    SortOrder::Desc => comparison.reverse(),
                };
            }
        }
        Ordering::Equal
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> ResultProcessor<S> for SortExecutor<S> {
    async fn process(&mut self, input: ExecutionResult) -> DBResult<ExecutionResult> {
        ResultProcessor::set_input(self, input);
        let dataset = self.process_input().await?;
        Ok(ExecutionResult::DataSet(dataset))
    }

    fn set_input(&mut self, input: ExecutionResult) {
        self.base.input = Some(input);
    }

    fn get_input(&self) -> Option<&ExecutionResult> {
        self.base.input.as_ref()
    }

    fn context(&self) -> &ResultProcessorContext {
        &self.base.context
    }

    fn set_context(&mut self, context: ResultProcessorContext) {
        self.base.context = context;
    }

    fn memory_usage(&self) -> usize {
        self.memory_usage
    }

    fn reset(&mut self) {
        self.base.memory_usage = 0;
        self.base.input = None;
        self.memory_usage = 0;
    }
}

#[async_trait]
impl<S: StorageEngine + Send + Sync + 'static> Executor<S> for SortExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            self.base
                .input
                .clone()
                .unwrap_or(ExecutionResult::DataSet(DataSet::new()))
        };

        self.process(input_result).await
    }

    fn open(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.base.input.is_some()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> &str {
        &self.base.description
    }
}

impl<S: StorageEngine + Send + 'static> HasStorage<S> for SortExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        &self.base.storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::{DataSet, Value};
    use crate::config::test_config::test_config;

    fn create_test_dataset() -> DataSet {
        let mut data_set = DataSet::new();
        data_set.col_names = vec!["name".to_string(), "age".to_string(), "score".to_string()];
        
        // 添加测试数据
        data_set.rows = vec![
            vec![Value::String("Alice".to_string()), Value::Int(25), Value::Float(85.5)],
            vec![Value::String("Bob".to_string()), Value::Int(30), Value::Float(92.0)],
            vec![Value::String("Charlie".to_string()), Value::Int(22), Value::Float(78.5)],
            vec![Value::String("David".to_string()), Value::Int(28), Value::Float(88.0)],
            vec![Value::String("Eve".to_string()), Value::Int(26), Value::Float(95.5)],
        ];
        
        data_set
    }

    #[test]
    fn test_quick_sort() {
        let mut data_set = create_test_dataset();
        let sort_keys = vec![
            SortKey {
                expression: Expression::InputProperty("score".to_string()), // 按score排序
                order: SortOrder::Asc,
            }
        ];
        
        let config = SortConfig {
            algorithm: SortAlgorithm::QuickSort,
            memory_limit: 1024 * 1024, // 1MB
            block_size: 64 * 1024, // 64KB
            parallel: false,
            max_threads: 1,
        };
        
        // 创建模拟存储引擎，使用test_config提供的路径
        use crate::storage::native_storage::NativeStorage;
        let test_config = test_config();
        let db_path = test_config.test_db_path("test_db_sort_1");
        let storage = Arc::new(Mutex::new(NativeStorage::new(db_path.to_str().unwrap()).unwrap()));
        
        let mut executor = SortExecutor::new(1, storage, sort_keys, None, config).unwrap();
        
        // 手动调用排序方法进行测试
        executor.select_and_execute_sort(&mut data_set).unwrap();
        
        // 验证排序结果
        assert_eq!(data_set.rows.len(), 5);
        assert_eq!(data_set.rows[0][2], Value::Float(78.5)); // Charlie
        assert_eq!(data_set.rows[4][2], Value::Float(95.5)); // Eve
    }

    #[test]
    fn test_top_n_sort() {
        let mut data_set = create_test_dataset();
        let sort_keys = vec![
            SortKey {
                expression: Expression::InputProperty("score".to_string()), // 按score排序
                order: SortOrder::Desc, // 降序
            }
        ];
        
        let config = SortConfig {
            algorithm: SortAlgorithm::HeapSort,
            memory_limit: 1024 * 1024, // 1MB
            block_size: 64 * 1024, // 64KB
            parallel: false,
            max_threads: 1,
        };
        
        // 创建模拟存储引擎，使用test_config提供的路径
        use crate::storage::native_storage::NativeStorage;
        let test_config = test_config();
        let db_path = test_config.test_db_path("test_db_sort_2");
        let storage = Arc::new(Mutex::new(NativeStorage::new(db_path.to_str().unwrap()).unwrap()));
        
        let mut executor = SortExecutor::new(1, storage, sort_keys, Some(3), config).unwrap();
        
        // 手动调用排序方法进行测试
        executor.select_and_execute_sort(&mut data_set).unwrap();
        
        // 验证Top-N结果
        assert_eq!(data_set.rows.len(), 3); // 只保留前3个
        assert_eq!(data_set.rows[0][2], Value::Float(95.5)); // Eve (最高分)
        assert_eq!(data_set.rows[2][2], Value::Float(88.0)); // David (第三高分)
    }
}