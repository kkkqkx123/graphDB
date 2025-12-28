# 查询执行器技术实施指南

## 概述

本文档提供查询执行器功能实施的具体技术指导，包括代码结构、实现模式和最佳实践。

## 核心实施原则

### 1.1 Rust最佳实践
- **零成本抽象**：使用泛型和trait，避免运行时开销
- **内存安全**：充分利用借用检查器，避免unsafe代码
- **错误处理**：使用Result类型，避免panic
- **异步优先**：IO操作必须使用async/await

### 1.2 架构一致性
- **模块化设计**：功能相关的代码组织在同一模块
- **trait组合**：使用小trait组合而非大trait继承
- **依赖倒置**：依赖抽象而非具体实现
- **配置驱动**：通过配置而非硬编码控制行为

## 具体实施指南

### 2.1 执行器实现模板

```rust
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::{DBResult, Value, Vertex, Edge};
use crate::query::executor::traits::{Executor, ExecutionResult, HasStorage};
use crate::storage::StorageEngine;

/// 具体执行器实现
pub struct MyExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    // 执行器特定字段
    config: ExecutorConfig,
    state: ExecutionState,
}

#[async_trait]
impl<S: StorageEngine> Executor<S> for MyExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 1. 参数验证
        self.validate_input()?;
        
        // 2. 执行核心逻辑
        let result = self.execute_core().await?;
        
        // 3. 结果处理
        self.process_result(result)
    }
    
    fn open(&mut self) -> DBResult<()> {
        self.base.open()?;
        self.initialize_state()?;
        Ok(())
    }
    
    fn close(&mut self) -> DBResult<()> {
        self.cleanup()?;
        self.base.close()?;
        Ok(())
    }
    
    // 其他必需方法实现...
}

impl<S: StorageEngine> HasStorage<S> for MyExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}
```

### 2.2 错误处理模式

#### 自定义错误类型
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    
    #[error("Memory limit exceeded: {current} > {limit}")]
    MemoryLimitExceeded { current: usize, limit: usize },
    
    #[error("Execution timeout after {duration:?}")]
    Timeout { duration: Duration },
}

// 转换到DBError
impl From<ExecutorError> for DBError {
    fn from(error: ExecutorError) -> Self {
        DBError::QueryExecution(error.to_string())
    }
}
```

#### 错误上下文
```rust
use anyhow::{Context, Result};

async fn execute_query(&mut self) -> Result<ExecutionResult> {
    self.validate_input()
        .context("Failed to validate executor input")?;
    
    let data = self.fetch_data()
        .context("Failed to fetch data from storage")?;
    
    self.process_data(data)
        .context("Failed to process query data")
}
```

### 2.3 内存管理策略

#### 内存限制配置
```rust
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// 单个查询最大内存使用
    pub max_query_memory: usize,
    
    /// 内存检查间隔
    pub check_interval: Duration,
    
    /// 是否启用内存溢出到磁盘
    pub spill_enabled: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_query_memory: 1024 * 1024 * 100, // 100MB
            check_interval: Duration::from_millis(100),
            spill_enabled: true,
        }
    }
}
```

#### 内存监控实现
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MemoryTracker {
    current_usage: AtomicUsize,
    limit: usize,
}

impl MemoryTracker {
    pub fn new(limit: usize) -> Self {
        Self {
            current_usage: AtomicUsize::new(0),
            limit,
        }
    }
    
    pub fn allocate(&self, size: usize) -> Result<()> {
        let current = self.current_usage.fetch_add(size, Ordering::AcqRel);
        if current + size > self.limit {
            self.current_usage.fetch_sub(size, Ordering::AcqRel);
            return Err(ExecutorError::MemoryLimitExceeded {
                current: current + size,
                limit: self.limit,
            }.into());
        }
        Ok(())
    }
    
    pub fn deallocate(&self, size: usize) {
        self.current_usage.fetch_sub(size, Ordering::AcqRel);
    }
}
```

### 2.4 性能优化技术

#### 零拷贝数据处理
```rust
// 使用引用而非克隆
data.iter()
    .filter(|item| self.predicate.matches(item))
    .map(|item| &item.field)
    .collect()
```

#### 批处理优化
```rust
const BATCH_SIZE: usize = 1000;

async fn process_in_batches(&self, data: Vec<Item>) -> Result<Vec<Result>> {
    let mut results = Vec::new();
    
    for batch in data.chunks(BATCH_SIZE) {
        let batch_results = self.process_batch(batch).await?;
        results.extend(batch_results);
    }
    
    Ok(results)
}
```

#### 异步并发控制
```rust
use futures::stream::{self, StreamExt};
use tokio::sync::Semaphore;

const MAX_CONCURRENT: usize = 10;

async fn concurrent_processing(&self, items: Vec<Item>) -> Result<Vec<Result>> {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    
    let results = stream::iter(items)
        .map(|item| {
            let permit = semaphore.clone().acquire_owned();
            async move {
                let _permit = permit.await?;
                self.process_item(item).await
            }
        })
        .buffer_unordered(MAX_CONCURRENT)
        .collect::<Vec<_>>()
        .await;
    
    results.into_iter().collect::<Result<Vec<_>>>()
}
```

## 具体功能实施

### 3.1 JOIN算法优化实施

#### 当前问题分析
```rust
// src/query/executor/data_processing/join/hash_table.rs
pub struct HashJoinTable {
    table: HashMap<Vec<u8>, Vec<Row>>,
}
```

**问题**：
- 简单哈希表实现，无内存管理
- 缺少溢出处理机制
- 性能未经优化

#### 优化实现
```rust
pub struct OptimizedHashJoinTable {
    /// 主哈希表
    main_table: HashMap<JoinKey, Vec<Row>>,
    
    /// 内存使用跟踪
    memory_tracker: Arc<MemoryTracker>,
    
    /// 溢出配置
    spill_config: SpillConfig,
    
    /// 溢出文件句柄（可选）
    spill_file: Option<SpillFile>,
}

impl OptimizedHashJoinTable {
    pub fn new(memory_limit: usize, spill_enabled: bool) -> Self {
        Self {
            main_table: HashMap::new(),
            memory_tracker: Arc::new(MemoryTracker::new(memory_limit)),
            spill_config: SpillConfig::new(spill_enabled),
            spill_file: None,
        }
    }
    
    pub fn insert(&mut self, key: JoinKey, row: Row) -> Result<()> {
        // 估算内存使用
        let estimated_size = key.size() + row.size();
        self.memory_tracker.allocate(estimated_size)?;
        
        // 检查是否需要溢出
        if self.should_spill() {
            self.spill_to_disk()?;
        }
        
        self.main_table.entry(key).or_insert_with(Vec::new).push(row);
        Ok(())
    }
    
    fn should_spill(&self) -> bool {
        self.spill_config.enabled && 
        self.memory_tracker.usage_ratio() > self.spill_config.threshold
    }
}
```

### 3.2 最短路径算法实施

#### 算法选择
```rust
pub enum ShortestPathAlgorithm {
    /// Dijkstra算法（带权图）
    Dijkstra,
    /// BFS（无权图）
    Bfs,
    /// A*（启发式搜索）
    AStar { heuristic: Box<dyn Heuristic> },
}

pub struct ShortestPathExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    algorithm: ShortestPathAlgorithm,
    max_depth: Option<usize>,
    weight_property: Option<String>,
}
```

#### Dijkstra实现
```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Eq, PartialEq)]
struct SearchNode {
    vertex: VertexId,
    distance: u64,
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance) // 最小堆
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

async fn dijkstra(
    &self,
    start: VertexId,
    end: VertexId,
    weight_prop: Option<&str>,
) -> Result<Option<Vec<VertexId>>> {
    let mut distances = HashMap::new();
    let mut previous = HashMap::new();
    let mut heap = BinaryHeap::new();
    
    // 初始化
    distances.insert(start, 0);
    heap.push(SearchNode { vertex: start, distance: 0 });
    
    while let Some(SearchNode { vertex, distance }) = heap.pop() {
        if vertex == end {
            return Ok(self.reconstruct_path(&previous, start, end));
        }
        
        if distance > *distances.get(&vertex).unwrap_or(&u64::MAX) {
            continue;
        }
        
        // 获取邻居节点
        let neighbors = self.get_neighbors(vertex).await?;
        
        for (neighbor, edge_weight) in neighbors {
            let weight = self.get_edge_weight(edge_weight, weight_prop)?;
            let new_distance = distance + weight;
            
            if new_distance < *distances.get(&neighbor).unwrap_or(&u64::MAX) {
                distances.insert(neighbor, new_distance);
                previous.insert(neighbor, vertex);
                heap.push(SearchNode { vertex: neighbor, distance: new_distance });
            }
        }
    }
    
    Ok(None) // 无路径
}
```

### 3.3 全文搜索集成

#### 轻量级方案选择
```rust
// 使用Tantivy作为全文搜索引擎
tantivy = "0.21"
```

#### 索引管理
```rust
pub struct FulltextIndexManager {
    index_directory: PathBuf,
    schema: Schema,
    index: Index,
    writer: IndexWriter,
}

impl FulltextIndexManager {
    pub fn new(index_path: PathBuf) -> Result<Self> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("content", TEXT | STORED);
        schema_builder.add_text_field("vertex_id", STRING | STORED);
        let schema = schema_builder.build();
        
        let index = Index::create_in_dir(&index_path, schema.clone())?;
        let writer = index.writer(50_000_000)?; // 50MB堆大小
        
        Ok(Self {
            index_directory: index_path,
            schema,
            index,
            writer,
        })
    }
    
    pub fn index_vertex(&mut self, vertex_id: VertexId, content: &str) -> Result<()> {
        let content_field = self.schema.get_field("content")?;
        let vertex_id_field = self.schema.get_field("vertex_id")?;
        
        let mut document = Document::default();
        document.add_text(content_field, content);
        document.add_text(vertex_id_field, &vertex_id.to_string());
        
        self.writer.add_document(document)?;
        Ok(())
    }
    
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<VertexId>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        
        let query_parser = QueryParser::for_index(&self.index, vec![
            self.schema.get_field("content")?
        ]);
        
        let query = query_parser.parse_query(query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        
        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            if let Some(vertex_id_value) = retrieved_doc.get_first(self.schema.get_field("vertex_id")?) {
                if let Some(vertex_id_str) = vertex_id_value.as_text() {
                    if let Ok(vertex_id) = vertex_id_str.parse() {
                        results.push(vertex_id);
                    }
                }
            }
        }
        
        Ok(results)
    }
}
```

## 测试策略

### 4.1 单元测试
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_join_memory_tracking() {
        let memory_limit = 1024 * 1024; // 1MB
        let mut table = OptimizedHashJoinTable::new(memory_limit, true);
        
        // 测试正常插入
        let key = JoinKey::from("test_key");
        let row = Row::from(vec![Value::Int(42)]);
        assert!(table.insert(key, row).is_ok());
        
        // 测试内存限制
        let large_key = JoinKey::from("x".repeat(1024 * 1024));
        let large_row = Row::from(vec![Value::String("x".repeat(1024 * 1024))]);
        assert!(table.insert(large_key, large_row).is_err());
    }
    
    #[tokio::test]
    async fn test_dijkstra_algorithm() {
        let graph = create_test_graph().await;
        let executor = ShortestPathExecutor::new(graph);
        
        let path = executor.dijkstra(1, 5, None).await.unwrap();
        assert!(path.is_some());
        assert_eq!(path.unwrap(), vec![1, 2, 3, 5]);
    }
}
```

### 4.2 集成测试
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_query_execution() {
        // 1. 创建测试数据库
        let db = create_test_database().await;
        
        // 2. 插入测试数据
        insert_test_data(&db).await.unwrap();
        
        // 3. 执行复杂查询
        let query = r#"
            MATCH (p:Person)-[:KNOWS]->(friend:Person)
            WHERE p.name = 'Alice'
            RETURN friend.name, friend.age
            ORDER BY friend.age DESC
            LIMIT 10
        "#;
        
        let result = db.execute_query(query).await.unwrap();
        
        // 4. 验证结果
        assert_eq!(result.len(), 2); // Alice有2个朋友
        assert_eq!(result[0].get("friend.name"), Some(&Value::String("Bob".to_string())));
    }
}
```

### 4.3 性能基准测试
```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_hash_join(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_join");
    
    for size in [1000, 10000, 100000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let left_table = generate_test_table(size);
                let right_table = generate_test_table(size);
                hash_join(&left_table, &right_table)
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, benchmark_hash_join);
criterion_main!(benches);
```

## 部署和监控

### 5.1 性能监控
```rust
use metrics::{counter, histogram, gauge};

pub struct ExecutorMetrics {
    query_counter: Counter,
    execution_time: Histogram,
    memory_usage: Gauge,
}

impl ExecutorMetrics {
    pub fn record_query(&self, duration: Duration, memory_used: usize) {
        self.query_counter.increment(1);
        self.execution_time.record(duration.as_secs_f64());
        self.memory_usage.set(memory_used as f64);
    }
}
```

### 5.2 日志记录
```rust
use tracing::{info, warn, error, debug};

async fn execute_with_logging(&mut self) -> Result<ExecutionResult> {
    let query_id = self.id();
    info!("Starting query execution: {}", query_id);
    
    let start_time = Instant::now();
    
    match self.execute_core().await {
        Ok(result) => {
            let duration = start_time.elapsed();
            info!("Query {} completed in {:?}: {} results", 
                  query_id, duration, result.count());
            Ok(result)
        }
        Err(error) => {
            let duration = start_time.elapsed();
            error!("Query {} failed after {:?}: {}", 
                   query_id, duration, error);
            Err(error)
        }
    }
}
```

## 结论

本技术实施指南提供了：

1. **标准化的执行器实现模式**
2. **详细的性能优化策略**
3. **完整的错误处理机制**
4. **全面的测试策略**
5. **生产环境的监控方案**

通过遵循这些指南，可以确保查询执行器的高质量实现，同时保持与项目整体架构的一致性。