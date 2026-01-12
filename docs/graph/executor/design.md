# 数据处理执行器设计文档

## 架构概述

数据处理执行器是图数据库查询引擎的核心组件，负责执行查询计划中的数据处理操作。本文档详细描述了各个执行器的设计原理、实现细节和接口规范。

## 执行器基础架构

### 核心 Trait 定义

```rust
#[async_trait]
pub trait Executor<S: StorageEngine>: Send + Sync {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError>;
    fn open(&mut self) -> Result<(), QueryError>;
    fn close(&mut self) -> Result<(), QueryError>;
    fn id(&self) -> usize;
    fn name(&self) -> &str;
}

pub trait InputExecutor<S: StorageEngine> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>);
    fn get_input(&self) -> Option<&Box<dyn Executor<S>>>;
}
```

### 执行结果类型

```rust
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    Values(Vec<Value>),
    Vertices(Vec<Vertex>),
    Edges(Vec<Edge>),
    Paths(Vec<Path>),
    DataSet(DataSet),
    Count(usize),
    Success,
}
```

## 具体执行器设计

### 1. FilterExecutor - 条件过滤执行器

#### 设计原理
FilterExecutor 根据指定的条件表达式对输入数据进行过滤，实现 WHERE 子句的功能。

#### 核心实现

```rust
pub struct FilterExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    condition: Expression,
    input_executor: Option<Box<dyn Executor<S>>>,
    expression_cache: LruCache<String, bool>,  // 表达式结果缓存
}

impl<S: StorageEngine> FilterExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        condition: Expression,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "FilterExecutor".to_string(), storage),
            condition,
            input_executor: None,
            expression_cache: LruCache::new(1000),
        }
    }

    /// 评估条件表达式
    async fn evaluate_condition(&self, context: &ExpressionContext) -> Result<bool, QueryError> {
        // 检查缓存
        let cache_key = format!("{:?}", context);
        if let Some(&result) = self.expression_cache.get(&cache_key) {
            return Ok(result);
        }

        // 评估表达式
        let result = self.condition.evaluate(context)
            .map_err(|e| QueryError::ExpressionError(e.to_string()))?;

        // 转换为布尔值
        let bool_result = match result {
            Value::Bool(b) => b,
            Value::Null(_) => false,
            _ => true,  // 非空值视为 true
        };

        // 缓存结果
        self.expression_cache.put(cache_key, bool_result);
        Ok(bool_result)
    }

    /// 应用过滤条件
    fn apply_filter(&self, input: ExecutionResult) -> Result<ExecutionResult, QueryError> {
        match input {
            ExecutionResult::Values(values) => {
                let filtered_values = values.into_iter()
                    .filter(|value| {
                        let context = ExpressionContext::from_value(value);
                        self.evaluate_condition(&context).unwrap_or(false)
                    })
                    .collect();
                Ok(ExecutionResult::Values(filtered_values))
            },
            ExecutionResult::Vertices(vertices) => {
                let filtered_vertices = vertices.into_iter()
                    .filter(|vertex| {
                        let context = ExpressionContext::from_vertex(vertex);
                        self.evaluate_condition(&context).unwrap_or(false)
                    })
                    .collect();
                Ok(ExecutionResult::Vertices(filtered_vertices))
            },
            ExecutionResult::Edges(edges) => {
                let filtered_edges = edges.into_iter()
                    .filter(|edge| {
                        let context = ExpressionContext::from_edge(edge);
                        self.evaluate_condition(&context).unwrap_or(false)
                    })
                    .collect();
                Ok(ExecutionResult::Edges(filtered_edges))
            },
            _ => Ok(input),
        }
    }
}
```

#### 性能优化
- 表达式结果缓存
- 短路评估优化
- 批量处理优化

### 2. LoopExecutor - 循环控制执行器

#### 设计原理
LoopExecutor 实现循环控制逻辑，支持条件循环和计数循环。

#### 核心实现

```rust
pub struct LoopExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    condition: Option<Expression>,  // 循环条件，None 表示无限循环
    body_executor: Box<dyn Executor<S>>,
    max_iterations: Option<usize>,
    current_iteration: usize,
    loop_state: LoopState,
}

#[derive(Debug)]
enum LoopState {
    NotStarted,
    Running,
    Finished,
    Error(String),
}

impl<S: StorageEngine> LoopExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        condition: Option<Expression>,
        body_executor: Box<dyn Executor<S>>,
        max_iterations: Option<usize>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "LoopExecutor".to_string(), storage),
            condition,
            body_executor,
            max_iterations,
            current_iteration: 0,
            loop_state: LoopState::NotStarted,
        }
    }

    /// 评估循环条件
    async fn evaluate_condition(&self) -> Result<bool, QueryError> {
        match &self.condition {
            Some(expr) => {
                let context = ExpressionContext::new();
                let result = expr.evaluate(&context)
                    .map_err(|e| QueryError::ExpressionError(e.to_string()))?;
                
                match result {
                    Value::Bool(b) => Ok(b),
                    Value::Null(_) => Ok(false),
                    _ => Ok(true),
                }
            },
            None => Ok(true),  // 无条件循环
        }
    }

    /// 检查是否应该继续循环
    fn should_continue(&self) -> bool {
        if let LoopState::Error(_) = self.loop_state {
            return false;
        }

        if let Some(max_iter) = self.max_iterations {
            if self.current_iteration >= max_iter {
                return false;
            }
        }

        true
    }

    /// 执行单次循环
    async fn execute_iteration(&mut self) -> Result<ExecutionResult, QueryError> {
        self.current_iteration += 1;
        
        // 执行循环体
        let result = self.body_executor.execute().await?;
        
        // 重置循环体状态
        self.body_executor.close()?;
        self.body_executor.open()?;
        
        Ok(result)
    }

    /// 收集所有循环结果
    fn collect_results(&self, results: Vec<ExecutionResult>) -> ExecutionResult {
        let mut all_values = Vec::new();
        let mut all_vertices = Vec::new();
        let mut all_edges = Vec::new();
        let mut all_paths = Vec::new();

        for result in results {
            match result {
                ExecutionResult::Values(values) => all_values.extend(values),
                ExecutionResult::Vertices(vertices) => all_vertices.extend(vertices),
                ExecutionResult::Edges(edges) => all_edges.extend(edges),
                ExecutionResult::Paths(paths) => all_paths.extend(paths),
                _ => {}
            }
        }

        // 根据内容返回最合适的结果类型
        if !all_values.is_empty() {
            ExecutionResult::Values(all_values)
        } else if !all_vertices.is_empty() {
            ExecutionResult::Vertices(all_vertices)
        } else if !all_edges.is_empty() {
            ExecutionResult::Edges(all_edges)
        } else if !all_paths.is_empty() {
            ExecutionResult::Paths(all_paths)
        } else {
            ExecutionResult::Success
        }
    }
}
```

#### 安全机制
- 最大迭代次数限制
- 内存使用监控
- 异常处理和恢复

### 3. TraverseExecutor - 图遍历执行器

#### 设计原理
TraverseExecutor 实现多步图遍历，支持路径构建和属性过滤。

#### 核心实现

```rust
pub struct TraverseExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    step_range: (usize, usize),
    edge_types: Option<Vec<String>>,
    edge_direction: EdgeDirection,
    vertex_filter: Option<Expression>,
    edge_filter: Option<Expression>,
    generate_path: bool,
    dedup_strategy: DedupStrategy,
}

#[derive(Debug, Clone)]
pub enum DedupStrategy {
    None,
    Vertex,
    Edge,
    Path,
}

#[derive(Debug)]
struct TraversalState {
    current_step: usize,
    frontier: Vec<TraversalNode>,
    visited: HashSet<Value>,
    paths: Vec<Path>,
    results: Vec<TraversalResult>,
}

#[derive(Debug, Clone)]
struct TraversalNode {
    vertex: Value,
    path: Option<Path>,
    depth: usize,
}

impl<S: StorageEngine> TraverseExecutor<S> {
    /// 执行多步遍历
    async fn multi_step_traverse(&mut self, start_vertices: Vec<Value>) -> Result<ExecutionResult, QueryError> {
        let mut state = TraversalState {
            current_step: 0,
            frontier: start_vertices.into_iter().map(|v| TraversalNode {
                vertex: v,
                path: None,
                depth: 0,
            }).collect(),
            visited: HashSet::new(),
            paths: Vec::new(),
            results: Vec::new(),
        };

        // 初始化访问集合
        for node in &state.frontier {
            state.visited.insert(node.vertex.clone());
        }

        // 执行遍历
        while state.current_step < self.step_range.1 && !state.frontier.is_empty() {
            self.execute_traversal_step(&mut state).await?;
            state.current_step += 1;
        }

        // 构建结果
        self.build_traversal_result(state)
    }

    /// 执行单步遍历
    async fn execute_traversal_step(&self, state: &mut TraversalState) -> Result<(), QueryError> {
        let mut next_frontier = Vec::new();
        let storage = self.base.storage.lock().unwrap();

        for node in &state.frontier {
            // 获取邻居节点
            let neighbors = self.get_neighbors(&storage, &node.vertex).await?;
            
            for neighbor in neighbors {
                // 检查是否已访问
                if state.visited.contains(&neighbor) {
                    continue;
                }

                // 应用顶点过滤
                if let Some(ref filter) = self.vertex_filter {
                    let context = ExpressionContext::from_value(&neighbor);
                    if !self.evaluate_filter(filter, &context)? {
                        continue;
                    }
                }

                // 创建新的遍历节点
                let mut new_path = None;
                if self.generate_path {
                    new_path = self.extend_path(&node.path, &node.vertex, &neighbor);
                }

                let new_node = TraversalNode {
                    vertex: neighbor.clone(),
                    path: new_path,
                    depth: node.depth + 1,
                };

                next_frontier.push(new_node);
                state.visited.insert(neighbor);
            }
        }

        state.frontier = next_frontier;
        Ok(())
    }

    /// 获取邻居节点
    async fn get_neighbors(&self, storage: &S, vertex: &Value) -> Result<Vec<Value>, QueryError> {
        let edges = storage.get_node_edges(vertex, self.edge_direction.into())
            .map_err(|e| QueryError::StorageError(e))?;

        let mut neighbors = Vec::new();

        for edge in edges {
            // 应用边类型过滤
            if let Some(ref edge_types) = self.edge_types {
                if !edge_types.contains(&edge.edge_type) {
                    continue;
                }
            }

            // 应用边过滤
            if let Some(ref filter) = self.edge_filter {
                let context = ExpressionContext::from_edge(&edge);
                if !self.evaluate_filter(filter, &context)? {
                    continue;
                }
            }

            // 根据方向提取邻居节点
            let neighbor = match self.edge_direction {
                EdgeDirection::Out => edge.dst,
                EdgeDirection::In => edge.src,
                EdgeDirection::Both => {
                    if edge.src == *vertex {
                        edge.dst
                    } else {
                        edge.src
                    }
                }
            };

            neighbors.push(*neighbor);
        }

        Ok(neighbors)
    }

    /// 扩展路径
    fn extend_path(&self, current_path: &Option<Path>, from: &Value, to: &Value) -> Option<Path> {
        match current_path {
            Some(path) => {
                let mut new_path = path.clone();
                new_path.add_step(from.clone(), to.clone());
                Some(new_path)
            },
            None => {
                let mut path = Path::new();
                path.add_step(from.clone(), to.clone());
                Some(path)
            }
        }
    }

    /// 构建遍历结果
    fn build_traversal_result(&self, state: TraversalState) -> Result<ExecutionResult, QueryError> {
        if self.generate_path {
            Ok(ExecutionResult::Paths(state.paths))
        } else {
            let vertices: Vec<Value> = state.frontier.into_iter()
                .map(|node| node.vertex)
                .collect();
            Ok(ExecutionResult::Values(vertices))
        }
    }
}
```

### 4. DedupExecutor - 去重执行器

#### 设计原理
DedupExecutor 实现数据去重功能，支持基于指定键的去重策略。

#### 核心实现

```rust
pub struct DedupExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    input_var: String,
    dedup_keys: Option<Vec<String>>,
    strategy: DedupStrategy,
    memory_limit: usize,
}

impl<S: StorageEngine> DedupExecutor<S> {
    /// 执行去重操作
    fn execute_dedup(&self, input: ExecutionResult) -> Result<ExecutionResult, QueryError> {
        match input {
            ExecutionResult::Values(values) => {
                let deduped_values = self.dedup_values(values)?;
                Ok(ExecutionResult::Values(deduped_values))
            },
            ExecutionResult::Vertices(vertices) => {
                let deduped_vertices = self.dedup_vertices(vertices)?;
                Ok(ExecutionResult::Vertices(deduped_vertices))
            },
            ExecutionResult::Edges(edges) => {
                let deduped_edges = self.dedup_edges(edges)?;
                Ok(ExecutionResult::Edges(deduped_edges))
            },
            _ => Ok(input),
        }
    }

    /// 基于哈希的去重
    fn hash_based_dedup<T>(&self, items: Vec<T>, key_extractor: impl Fn(&T) -> String) -> Result<Vec<T>, QueryError> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        let mut memory_usage = 0;

        for item in items {
            let key = key_extractor(&item);
            
            if !seen.contains(&key) {
                seen.insert(key);
                result.push(item);
                
                // 检查内存使用
                memory_usage += std::mem::size_of::<T>();
                if memory_usage > self.memory_limit {
                    return Err(QueryError::MemoryLimitExceeded);
                }
            }
        }

        Ok(result)
    }

    /// 值去重
    fn dedup_values(&self, values: Vec<Value>) -> Result<Vec<Value>, QueryError> {
        match &self.dedup_keys {
            Some(keys) => {
                // 基于指定键去重
                self.hash_based_dedup(values, |value| {
                    self.extract_keys_from_value(value, keys)
                })
            },
            None => {
                // 完全去重
                self.hash_based_dedup(values, |value| format!("{:?}", value))
            }
        }
    }

    /// 从值中提取键
    fn extract_keys_from_value(&self, value: &Value, keys: &[String]) -> String {
        match value {
            Value::Map(map) => {
                keys.iter()
                    .filter_map(|key| map.get(key))
                    .map(|v| format!("{:?}", v))
                    .collect::<Vec<_>>()
                    .join("|")
            },
            _ => format!("{:?}", value),
        }
    }
}
```

## 性能优化策略

### 1. 内存管理

```rust
pub struct MemoryManager {
    limit: usize,
    used: usize,
    allocations: HashMap<usize, usize>,
}

impl MemoryManager {
    pub fn allocate(&mut self, size: usize) -> Result<usize, QueryError> {
        if self.used + size > self.limit {
            return Err(QueryError::MemoryLimitExceeded);
        }
        
        let id = self.allocations.len() + 1;
        self.allocations.insert(id, size);
        self.used += size;
        Ok(id)
    }
    
    pub fn deallocate(&mut self, id: usize) {
        if let Some(size) = self.allocations.remove(&id) {
            self.used -= size;
        }
    }
}
```

### 2. 并行处理

```rust
use tokio::task::JoinSet;

pub struct ParallelExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    worker_count: usize,
}

impl<S: StorageEngine> ParallelExecutor<S> {
    async fn execute_parallel<T, F>(&self, items: Vec<T>, processor: F) -> Result<Vec<T::Output>, QueryError>
    where
        T: Send + 'static,
        T::Output: Send + 'static,
        F: Fn(T) -> T::Output + Send + Sync + 'static,
    {
        let chunk_size = (items.len() + self.worker_count - 1) / self.worker_count;
        let mut join_set = JoinSet::new();
        
        for chunk in items.chunks(chunk_size) {
            let chunk = chunk.to_vec();
            let processor = &processor;
            
            join_set.spawn(async move {
                chunk.into_iter().map(processor).collect::<Vec<_>>()
            });
        }
        
        let mut results = Vec::new();
        while let Some(result) = join_set.join_next().await {
            results.extend(result?);
        }
        
        Ok(results)
    }
}
```

## 错误处理

### 统一错误类型

```rust
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Expression evaluation error: {0}")]
    ExpressionError(String),
    
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),
    
    #[error("Memory limit exceeded: used {used}, limit {limit}")]
    MemoryLimitExceeded { used: usize, limit: usize },
    
    #[error("Timeout error: operation took longer than {0}ms")]
    TimeoutError(u64),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}
```

### 错误恢复策略

```rust
pub trait ErrorRecovery {
    fn can_recover(&self, error: &ExecutorError) -> bool;
    fn recover(&mut self, error: ExecutorError) -> Result<(), ExecutorError>;
}

pub struct DefaultErrorRecovery;

impl ErrorRecovery for DefaultErrorRecovery {
    fn can_recover(&self, error: &ExecutorError) -> bool {
        match error {
            ExecutorError::MemoryLimitExceeded { .. } => true,
            ExecutorError::TimeoutError(_) => true,
            _ => false,
        }
    }
    
    fn recover(&mut self, error: ExecutorError) -> Result<(), ExecutorError> {
        match error {
            ExecutorError::MemoryLimitExceeded { .. } => {
                // 尝试垃圾回收
                self.garbage_collect()?;
                Ok(())
            },
            ExecutorError::TimeoutError(_) => {
                // 重试操作
                self.retry_operation()?;
                Ok(())
            },
            _ => Err(error),
        }
    }
}
```

## 测试框架

### 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MockStorage;
    
    #[tokio::test]
    async fn test_filter_executor() {
        let storage = Arc::new(Mutex::new(MockStorage::new()));
        let condition = Expression::GreaterThan(
            Box::new(Expression::Variable("age".to_string())),
            Box::new(Expression::Literal(Value::Int(18))),
        );
        
        let mut executor = FilterExecutor::new(1, storage, condition);
        
        // 设置测试数据
        let test_data = vec![
            Value::Map(maplit::hashmap! {
                "name".to_string() => Value::String("Alice".to_string()),
                "age".to_string() => Value::Int(20),
            }),
            Value::Map(maplit::hashmap! {
                "name".to_string() => Value::String("Bob".to_string()),
                "age".to_string() => Value::Int(16),
            }),
        ];
        
        executor.set_input(Box::new(MockExecutor::new(test_data)));
        
        // 执行过滤
        let result = executor.execute().await.unwrap();
        
        // 验证结果
        match result {
            ExecutionResult::Values(values) => {
                assert_eq!(values.len(), 1);
                assert_eq!(values[0], Value::Map(maplit::hashmap! {
                    "name".to_string() => Value::String("Alice".to_string()),
                    "age".to_string() => Value::Int(20),
                }));
            },
            _ => panic!("Expected Values result"),
        }
    }
}
```

## 总结

本文档详细描述了数据处理执行器的设计和实现方案，包括核心执行器的具体实现、性能优化策略、错误处理机制和测试框架。通过这些设计，可以构建一个高效、可靠、可扩展的图数据库查询执行引擎。