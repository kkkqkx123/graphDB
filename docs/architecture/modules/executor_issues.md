# Executor 模块问题清单与修改方案

## 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 6.1 | GraphQueryExecutor 中大量语句执行未实现 | 高 | 功能缺失 | 待修复 |
| 6.2 | ExecutorFactory 的 create_executor 方法过长 | 中 | 代码质量问题 | 待修复 |
| 6.3 | 缺乏执行器注册机制 | 中 | 扩展性问题 | 待修复 |
| 6.4 | 错误处理不统一 | 低 | 一致性问题 | 待修复 |
| 6.5 | 缺乏查询超时机制 | 低 | 缺失功能 | 待修复 |
| 6.6 | 执行结果序列化逻辑分散 | 低 | 代码质量问题 | 待修复 |

---

## 详细问题分析

### 问题 6.1: 大量语句执行未实现

**涉及文件**: `src/query/executor/mod.rs`

**当前实现**:
```rust
async fn execute_create(&mut self, _clause: CreateStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "CREATE语句执行未实现".to_string()
    )))
}

async fn execute_delete(&mut self, _clause: DeleteStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "DELETE语句执行未实现".to_string()
    )))
}

async fn execute_update(&mut self, _clause: UpdateStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "UPDATE语句执行未实现".to_string()
    )))
}

async fn execute_insert(&mut self, _clause: InsertStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "INSERT语句执行未实现".to_string()
    )))
}

async fn execute_upsert(&mut self, _clause: UpsertStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "UPSERT语句执行未实现".to_string()
    )))
}

async fn execute_lookup(&mut self, _clause: LookupStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "LOOKUP语句执行未实现".to_string()
    )))
}

async fn execute_show(&mut self, _clause: ShowStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "SHOW语句执行未实现".to_string()
    )))
}

async fn execute_alter(&mut self, _clause: AlterStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "ALTER语句执行未实现".to_string()
    )))
}

async fn execute_fetch_vertices(&mut self, _clause: FetchVerticesStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "FETCH VERTICES语句执行未实现".to_string()
    )))
}

async fn execute_fetch_edges(&mut self, _clause: FetchEdgesStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "FETCH EDGES语句执行未实现".to_string()
    )))
}

async fn execute_find_path(&mut self, _clause: FindPathStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "FIND PATH语句执行未实现".to_string()
    )))
}

async fn execute_get_subgraph(&mut self, _clause: GetSubgraphStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "GET SUBGRAPH语句执行未实现".to_string()
    )))
}

async fn execute_bidirectional_match(&mut self, _clause: BidirectionalMatchStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "双向匹配语句执行未实现".to_string()
    )))
}

async fn execute_multi_label_path(&mut self, _clause: MultiLabelPathStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "多标签路径语句执行未实现".to_string()
    )))
}

async fn execute_with_zone(&mut self, _clause: WithZoneStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "WITH ZONE语句执行未实现".to_string()
    )))
}

async fn execute_commit_bundle(&mut self, _clause: CommitBundleStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "COMMIT BUNDLE语句执行未实现".to_string()
    )))
}

async fn execute_rollback_bundle(&mut self, _clause: RollbackBundleStmt) -> Result<ExecutionResult, DBError> {
    Err(DBError::Query(QueryError::ExecutionError(
        "ROLLBACK BUNDLE语句执行未实现".to_string()
    )))
}
```

**问题分析**:
```
已实现的执行器:
├── ScanVerticesExecutor
├── GetVerticesExecutor
├── GetNeighborsExecutor
└── LimitExecutor

未实现的执行器 (16个):
├── CreateExecutor          (CREATE)
├── DeleteExecutor          (DELETE)
├── UpdateExecutor          (UPDATE)
├── InsertExecutor          (INSERT)
├── UpsertExecutor          (UPSERT)
├── LookupExecutor          (LOOKUP)
├── ShowExecutor            (SHOW)
├── AlterExecutor           (ALTER)
├── FetchVerticesExecutor   (FETCH VERTICES)
├── FetchEdgesExecutor      (FETCH EDGES)
├── FindPathExecutor        (FIND PATH)
├── GetSubgraphExecutor     (GET SUBGRAPH)
├── BidirectionalMatchExecutor
├── MultiLabelPathExecutor
├── WithZoneExecutor
├── CommitBundleExecutor
└── RollbackBundleExecutor
```

**影响**:
- 大部分写操作无法执行
- 限制了数据库的实用性
- 无法进行完整的功能测试

---

### 问题 6.2: create_executor 方法过长

**涉及文件**: `src/query/executor/mod.rs`

**当前实现**: `create_executor` 方法包含约 300+ 行代码，使用大型 match 语句处理所有节点类型。

**问题**:
- 代码难以阅读和维护
- 违反单一职责原则
- 添加新节点类型需要修改大量代码
- 无法复用执行器创建逻辑

---

### 问题 6.3: 缺乏执行器注册机制

**当前实现**: 执行器创建是硬编码的

**问题**:
- 无法动态添加新执行器
- 无法替换默认执行器
- 无法配置执行器参数
- 难以进行插件化扩展

---

## 修改方案

### 修改方案 6.1: 实现缺失的执行器

**预估工作量**: 15-20 人天（按功能模块）

**修改策略**: 按优先级分阶段实现

**阶段 1: 高频写操作（5-7 人天）**

```rust
// src/query/executor/statement/insert_executor.rs

pub struct InsertVertexExecutor {
    id: i64,
    storage: Arc<Mutex<S>>,
    space_name: String,
    tag_name: String,
    properties: Vec<(String, Value)>,
    on_duplicate_key: OnDuplicateKeyAction,
}

pub enum OnDuplicateKeyAction {
    Ignore,
    Update,
    Replace,
}

impl<S: StorageEngine> InsertVertexExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        space_name: String,
        tag_name: String,
        properties: Vec<(String, Value)>,
        on_duplicate_key: OnDuplicateKeyAction,
    ) -> Self {
        Self {
            id,
            storage,
            space_name,
            tag_name,
            properties,
            on_duplicate_key,
        }
    }
}

#[async_trait]
impl<S: StorageEngine> Executor<S> for InsertVertexExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let mut storage = self.storage.lock().map_err(|e| {
            DBError::Execution(format!("Failed to acquire storage lock: {}", e))
        })?;
        
        // 生成顶点 ID
        let vertex_id = storage.generate_vertex_id(&self.space_name)?;
        
        // 创建顶点
        let vertex = Vertex::new(
            vertex_id,
            self.tag_name.clone(),
            self.properties.clone(),
        );
        
        // 插入顶点
        storage.insert_vertex(&self.space_name, vertex)?;
        
        // 返回结果
        Ok(ExecutionResult::success_with_count(1))
    }
}

// src/query/executor/statement/delete_executor.rs

pub struct DeleteVertexExecutor {
    id: i64,
    storage: Arc<Mutex<S>>,
    space_name: String,
    vid: Value,
}

pub struct DeleteEdgeExecutor {
    id: i64,
    storage: Arc<Mutex<S>>,
    space_name: String,
    src_vid: Value,
    edge_type: String,
    rank: i64,
    dst_vid: Value,
}

impl<S: StorageEngine> DeleteVertexExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let mut storage = self.storage.lock().map_err(|e| {
            DBError::Execution(format!("Failed to acquire storage lock: {}", e))
        })?;
        
        // 删除顶点
        let count = storage.delete_vertex(&self.space_name, &self.vid)?;
        
        Ok(ExecutionResult::success_with_count(count))
    }
}

impl<S: StorageEngine> DeleteEdgeExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let mut storage = self.storage.lock().map_err(|e| {
            DBError::Execution(format!("Failed to acquire storage lock: {}", e))
        })?;
        
        // 删除边
        let count = storage.delete_edge(
            &self.space_name,
            &self.src_vid,
            &self.edge_type,
            self.rank,
            &self.dst_vid,
        )?;
        
        Ok(ExecutionResult::success_with_count(count))
    }
}
```

**阶段 2: 中频操作（5-7 人天）**

```rust
// src/query/executor/statement/update_executor.rs

pub struct UpdateVertexExecutor {
    id: i64,
    storage: Arc<Mutex<S>>,
    space_name: String,
    vid: Value,
    properties: Vec<(String, Expression)>,
}

pub struct UpdateEdgeExecutor {
    id: i64,
    storage: Arc<Mutex<S>>,
    space_name: String,
    src_vid: Value,
    edge_type: String,
    rank: i64,
    dst_vid: Value,
    properties: Vec<(String, Expression)>,
}

impl<S: StorageEngine> UpdateVertexExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let mut storage = self.storage.lock().map_err(|e| {
            DBError::Execution(format!("Failed to acquire storage lock: {}", e))
        })?;
        
        // 评估属性值
        let mut evaluated_props = Vec::new();
        for (key, expr) in &self.properties {
            let value = self.evaluate_expression(&mut *storage, expr).await?;
            evaluated_props.push((key.clone(), value));
        }
        
        // 更新顶点属性
        storage.update_vertex(&self.space_name, &self.vid, &evaluated_props)?;
        
        Ok(ExecutionResult::success_with_count(1))
    }
}

// src/query/executor/statement/fetch_executor.rs

pub struct FetchVerticesExecutor {
    id: i64,
    storage: Arc<Mutex<S>>,
    space_name: String,
    vids: Vec<Value>,
    properties: Vec<String>,
}

pub struct FetchEdgesExecutor {
    id: i64,
    storage: Arc<Mutex<S>>,
    space_name: String,
    edge_refs: Vec<EdgeRef>,
}

impl<S: StorageEngine> FetchVerticesExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = self.storage.lock().map_err(|e| {
            DBError::Execution(format!("Failed to acquire storage lock: {}", e))
        })?;
        
        let mut rows = Vec::new();
        for vid in &self.vids {
            if let Some(vertex) = storage.get_vertex(&self.space_name, vid)? {
                let row = self.vertex_to_row(&vertex);
                rows.push(row);
            }
        }
        
        Ok(ExecutionResult::new(rows, self.get_columns()))
    }
}
```

**阶段 3: 复杂查询（5-6 人天）**

```rust
// src/query/executor/statement/find_path_executor.rs

pub struct FindPathExecutor {
    id: i64,
    storage: Arc<Mutex<S>>,
    space_name: String,
    src_vid: Option<Value>,
    dst_vid: Option<Value>,
    pattern: PathPattern,
    limit: usize,
    with_props: bool,
}

impl<S: StorageEngine> FindPathExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let storage = self.storage.lock().map_err(|e| {
            DBError::Execution(format!("Failed to acquire storage lock: {}", e))
        })?;
        
        // 使用 BFS 或 DFS 查找路径
        let paths = match &self.pattern {
            PathPattern::Shortest(n) => {
                self.find_shortest_path(&mut *storage, *n)?
            }
            PathPattern::All(n) => {
                self.find_all_paths(&mut *storage, *n)?
            }
            PathPattern::NoLoop(n) => {
                self.find_no_loop_paths(&mut *storage, *n)?
            }
        };
        
        let rows: Vec<Row> = paths
            .into_iter()
            .take(self.limit)
            .map(|path| self.path_to_row(path))
            .collect();
        
        Ok(ExecutionResult::new(rows, self.get_columns()))
    }
    
    fn find_shortest_path(&self, storage: &mut S, max_hops: usize) -> DBResult<Vec<Path>> {
        // BFS 实现
        let mut queue = VecDeque::new();
        let mut visited = HashMap::new();
        
        if let Some(src) = &self.src_vid {
            queue.push_back((vec![src.clone()], 0));
            visited.insert(src.clone(), vec![src.clone()]);
        }
        
        let mut shortest_paths = Vec::new();
        
        while let Some((path, hops)) = queue.pop_front() {
            if hops > max_hops {
                continue;
            }
            
            let current = path.last().unwrap();
            
            if let Some(dst) = &self.dst_vid {
                if current == dst {
                    shortest_paths.push(Path::new(path));
                    continue;
                }
            }
            
            // 遍历邻居
            let neighbors = storage.get_neighbors(&self.space_name, current, None, None)?;
            for neighbor in neighbors {
                if !path.contains(&neighbor.dst_id) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor.dst_id);
                    
                    if let Some(existing_paths) = visited.get(&neighbor.dst_id) {
                        if existing_paths.len() <= hops + 1 {
                            continue;
                        }
                    }
                    
                    visited.insert(neighbor.dst_id, new_path.clone());
                    queue.push_back((new_path, hops + 1));
                }
            }
        }
        
        Ok(shortest_paths)
    }
}
```

---

### 修改方案 6.2: 重构 ExecutorFactory

**预估工作量**: 3-4 人天

**修改代码**:

```rust
// src/query/executor/mod.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 执行器创建者 trait
pub trait ExecutorCreator<S: StorageEngine>: Send {
    fn create(&self, node: &PlanNodeEnum, storage: Arc<Mutex<S>>) -> Result<Box<dyn Executor<S>>, QueryError>;
    fn can_handle(&self, node: &PlanNodeEnum) -> bool;
}

/// 执行器工厂
pub struct ExecutorFactory<S: StorageEngine + 'static> {
    storage: Option<Arc<Mutex<S>>>,
    executors: HashMap<&'static str, Box<dyn ExecutorCreator<S>>>,
    default_creator: Option<Box<dyn Fn(&PlanNodeEnum, Arc<Mutex<S>>) -> Result<Box<dyn Executor<S>>, QueryError>>>,
}

impl<S: StorageEngine + 'static> ExecutorFactory<S> {
    pub fn new() -> Self {
        let mut factory = Self {
            storage: None,
            executors: HashMap::new(),
            default_creator: None,
        };
        
        // 注册默认执行器
        factory.register("Start", Box::new(StartExecutorCreator));
        factory.register("ScanVertices", Box::new(ScanVerticesExecutorCreator));
        factory.register("GetVertices", Box::new(GetVerticesExecutorCreator));
        factory.register("GetNeighbors", Box::new(GetNeighborsExecutorCreator));
        factory.register("Filter", Box::new(FilterExecutorCreator));
        factory.register("Project", Box::new(ProjectExecutorCreator));
        factory.register("Aggregate", Box::new(AggregateExecutorCreator));
        factory.register("Sort", Box::new(SortExecutorCreator));
        factory.register("Limit", Box::new(LimitExecutorCreator));
        factory.register("TopN", Box::new(TopNExecutorCreator));
        factory.register("Dedup", Box::new(DedupExecutorCreator));
        
        factory
    }
    
    pub fn register(&mut self, node_type: &'static str, creator: Box<dyn ExecutorCreator<S>>) {
        self.executors.insert(node_type, creator);
    }
    
    pub fn set_storage(&mut self, storage: Arc<Mutex<S>>) {
        self.storage = Some(storage);
    }
    
    pub fn create_executor(
        &self,
        plan_node: &PlanNodeEnum,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        let storage = self.storage.as_ref()
            .ok_or_else(|| QueryError::ExecutionError(
                "Storage not set in ExecutorFactory".to_string()
            ))?;
        
        let node_type = plan_node.name();
        
        if let Some(creator) = self.executors.get(node_type) {
            creator.create(plan_node, storage.clone())
        } else if let Some(default_creator) = &self.default_creator {
            default_creator(plan_node, storage.clone())
        } else {
            Err(QueryError::ExecutionError(format!(
                "No executor registered for node type: {}", node_type
            )))
        }
    }
}

// 具体的执行器创建者实现

pub struct GetNeighborsExecutorCreator;

impl<S: StorageEngine> ExecutorCreator<S> for GetNeighborsExecutorCreator {
    fn create(&self, node: &PlanNodeEnum, storage: Arc<Mutex<S>>) -> Result<Box<dyn Executor<S>>, QueryError> {
        if let PlanNodeEnum::GetNeighbors(n) = node {
            let executor = GetNeighborsExecutor::new(
                n.id(),
                storage,
                Some(vec![crate::core::Value::String(n.src_vids().to_string())]),
                None,
                n.expression().and_then(|e| parse_expression_safe(e)),
                n.step_limit().map(|l| l as usize),
            );
            Ok(Box::new(executor))
        } else {
            Err(QueryError::ExecutionError(
                "GetNeighborsExecutorCreator can only create for GetNeighbors node".to_string()
            ))
        }
    }
    
    fn can_handle(&self, node: &PlanNodeEnum) -> bool {
        matches!(node, PlanNodeEnum::GetNeighbors(_))
    }
}

pub struct ProjectExecutorCreator;

impl<S: StorageEngine> ExecutorCreator<S> for ProjectExecutorCreator {
    fn create(&self, node: &PlanNodeEnum, storage: Arc<Mutex<S>>) -> Result<Box<dyn Executor<S>>, QueryError> {
        if let PlanNodeEnum::Project(n) = node {
            let executor = ProjectExecutor::new(
                n.id(),
                storage,
                n.columns().to_vec(),
            );
            Ok(Box::new(executor))
        } else {
            Err(QueryError::ExecutionError(
                "ProjectExecutorCreator can only create for Project node".to_string()
            ))
        }
    }
    
    fn can_handle(&self, node: &PlanNodeEnum) -> bool {
        matches!(node, PlanNodeEnum::Project(_))
    }
}
```

---

### 修改方案 6.3: 添加执行器注册机制

**预估工作量**: 2 人天

**修改代码**:

```rust
// src/query/executor/registry.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 执行器注册表
pub struct ExecutorRegistry<S: StorageEngine + 'static> {
    creators: HashMap<&'static str, Box<dyn ExecutorCreator<S>>>,
    middlewares: Vec<Box<dyn ExecutorMiddleware>>,
    default_creator: Option<Box<dyn ExecutorCreator<S>>>,
}

impl<S: StorageEngine + 'static> ExecutorRegistry<S> {
    pub fn new() -> Self {
        Self {
            creators: HashMap::new(),
            middlewares: Vec::new(),
            default_creator: None,
        }
    }
    
    pub fn register<C>(&mut self, node_type: &'static str, creator: C)
    where
        C: ExecutorCreator<S> + 'static,
    {
        self.creators.insert(node_type, Box::new(creator));
    }
    
    pub fn register_middleware<M>(&mut self, middleware: M)
    where
        M: ExecutorMiddleware + 'static,
    {
        self.middlewares.push(Box::new(middleware));
    }
    
    pub fn set_default<C>(&mut self, creator: C)
    where
        C: ExecutorCreator<S> + 'static,
    {
        self.default_creator = Some(Box::new(creator));
    }
    
    pub fn create(
        &self,
        node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        let node_type = node.name();
        
        let creator = self.creators.get(node_type)
            .or(self.default_creator.as_ref())
            .ok_or_else(|| QueryError::ExecutionError(format!(
                "No executor found for node type: {}", node_type
            )))?;
        
        let executor = creator.create(node, storage)?;
        
        // 应用中间件
        let mut wrapped = executor;
        for middleware in &self.middlewares {
            wrapped = middleware.wrap(wrapped);
        }
        
        Ok(wrapped)
    }
}

/// 执行器中间件
pub trait ExecutorMiddleware: Send {
    fn wrap(&self, executor: Box<dyn Executor<S>>) -> Box<dyn Executor<S>>;
}

/// 日志中间件
pub struct LoggingMiddleware;

impl<S: StorageEngine> ExecutorMiddleware for LoggingMiddleware {
    fn wrap(&self, executor: Box<dyn Executor<S>>) -> Box<dyn Executor<S>> {
        Box::new(LoggingExecutorWrapper { inner: executor })
    }
}

struct LoggingExecutorWrapper<S: StorageEngine> {
    inner: Box<dyn Executor<S>>,
}

#[async_trait]
impl<S: StorageEngine> Executor<S> for LoggingExecutorWrapper<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = std::time::Instant::now();
        let result = self.inner.execute().await;
        let duration = start.elapsed();
        
        tracing::info!(
            executor = std::any::type_name_of_val(&*self.inner),
            duration_ms = duration.as_millis(),
            result = ?result,
            "Executor completed"
        );
        
        result
    }
}

/// 超时中间件
pub struct TimeoutMiddleware {
    duration: std::time::Duration,
}

impl<S: StorageEngine> ExecutorMiddleware for TimeoutMiddleware {
    fn wrap(&self, executor: Box<dyn Executor<S>>) -> Box<dyn Executor<S>> {
        Box::new(TimeoutExecutorWrapper {
            inner: executor,
            duration: self.duration,
        })
    }
}

struct TimeoutExecutorWrapper<S: StorageEngine> {
    inner: Box<dyn Executor<S>>,
    duration: std::time::Duration,
}

#[async_trait]
impl<S: StorageEngine> Executor<S> for TimeoutExecutorWrapper<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        match tokio::time::timeout(self.duration, self.inner.execute()).await {
            Ok(result) => result,
            Err(_) => Err(DBError::Execution("Query timeout".to_string())),
        }
    }
}
```

---

### 修改方案 6.5: 添加查询超时机制

**预估工作量**: 2 人天

**修改代码**:

```rust
// src/query/executor/context.rs

use std::time::{Duration, Instant};

/// 执行配置
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub timeout: Duration,
    pub max_rows: usize,
    pub enable_profiling: bool,
    pub max_memory: usize,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_rows: 1_000_000,
            enable_profiling: false,
            max_memory: 1024 * 1024 * 1024, // 1GB
        }
    }
}

/// 执行上下文
pub struct ExecutionContext {
    pub config: ExecutionConfig,
    pub start_time: Instant,
    pub row_count: usize,
    pub memory_used: usize,
    pub profile_data: Option<ProfileData>,
}

#[derive(Debug, Default)]
pub struct ProfileData {
    pub node_executions: Vec<NodeExecutionStats>,
}

#[derive(Debug, Clone)]
pub struct NodeExecutionStats {
    pub node_id: i64,
    pub node_name: String,
    pub duration: Duration,
    pub rows_processed: usize,
    pub memory_used: usize,
}

impl ExecutionContext {
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            config,
            start_time: Instant::now(),
            row_count: 0,
            memory_used: 0,
            profile_data: None,
        }
    }
    
    pub fn check_timeout(&self) -> Result<(), QueryError> {
        if self.start_time.elapsed() > self.config.timeout {
            Err(QueryError::ExecutionError("Query timeout".to_string()))
        } else {
            Ok(())
        }
    }
    
    pub fn check_row_limit(&self) -> Result<(), QueryError> {
        if self.row_count > self.config.max_rows {
            Err(QueryError::ExecutionError(
                format!("Row limit exceeded: {}", self.config.max_rows)
            ))
        } else {
            Ok(())
        }
    }
    
    pub fn check_memory_limit(&self) -> Result<(), QueryError> {
        if self.memory_used > self.config.max_memory {
            Err(QueryError::ExecutionError(
                format!("Memory limit exceeded: {} bytes", self.config.max_memory)
            ))
        } else {
            Ok(())
        }
    }
    
    pub fn increment_row_count(&mut self, count: usize) {
        self.row_count += count;
    }
}

// 修改执行器以使用执行上下文
#[async_trait]
impl<S: StorageEngine> Executor<S> for FilterExecutor<S> {
    async fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 检查超时
        self.context.check_timeout()?;
        
        let input_result = self.input.execute().await?;
        let mut output_rows = Vec::new();
        
        for row in input_result.rows() {
            // 检查超时和行数限制
            self.context.check_timeout()?;
            self.context.check_row_limit()?;
            
            if self.evaluate_condition(&row)? {
                output_rows.push(row.clone());
                self.context.increment_row_count(1);
            }
        }
        
        Ok(ExecutionResult::new(output_rows, input_result.columns()))
    }
}
```

---

## 修改优先级

| 序号 | 修改方案 | 优先级 | 预估工作量 | 依赖 |
|------|----------|--------|------------|------|
| 6.1 | 实现缺失的执行器 | 高 | 15-20 人天 | 无 |
| 6.2 | 重构 ExecutorFactory | 中 | 3-4 人天 | 无 |
| 6.3 | 添加执行器注册机制 | 中 | 2 人天 | 6.2 |
| 6.5 | 添加查询超时机制 | 低 | 2 人天 | 无 |

---

## 测试建议

### 测试用例 1: InsertExecutor

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_insert_vertex() {
        let storage = Arc::new(Mutex::new(TestStorage::new()));
        let mut executor = InsertVertexExecutor::new(
            1,
            storage,
            "test_space".to_string(),
            "Player".to_string(),
            vec![
                ("name".to_string(), Value::String("John".to_string())),
                ("age".to_string(), Value::Int(25)),
            ],
            OnDuplicateKeyAction::Ignore,
        );
        
        let result = executor.execute().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().affected_rows(), 1);
    }
    
    #[tokio::test]
    async fn test_delete_vertex() {
        let storage = Arc::new(Mutex::new(TestStorage::new()));
        // 插入测试数据
        // ...
        
        let mut executor = DeleteVertexExecutor::new(
            1,
            storage,
            "test_space".to_string(),
            Value::Int(1),
        );
        
        let result = executor.execute().await;
        assert!(result.is_ok());
    }
}
```

---

## 风险与注意事项

### 风险 1: 执行器实现复杂度

- **风险**: 某些执行器（如 FindPath）实现复杂
- **缓解措施**: 先实现基本版本，再优化
- **实现**: 使用 BFS/DFS 等标准算法

### 风险 2: 存储引擎集成

- **风险**: 执行器需要与存储引擎正确集成
- **缓解措施**: 统一存储接口定义
- **实现**: 定义清晰的 StorageEngine trait

### 风险 3: 性能影响

- **风险**: 新功能可能影响查询性能
- **缓解措施**: 充分的性能测试
- **实现**: 添加性能基准测试
