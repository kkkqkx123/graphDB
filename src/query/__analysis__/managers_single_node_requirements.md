# 单节点架构下Managers功能补充分析

## 概述

本文档分析在保持单节点架构的情况下，`src/query/context/managers`目录需要补充的功能，专注于单节点场景下的核心需求。

## 当前实现状态

### 已实现功能

#### IndexManager
- ✅ 获取索引信息 (`get_index`)
- ✅ 列出所有索引 (`list_indexes`)
- ✅ 检查索引存在 (`has_index`)

#### MetaClient
- ✅ 获取集群信息 (`get_cluster_info`)
- ✅ 获取空间信息 (`get_space_info`)
- ✅ 检查连接状态 (`is_connected`)

#### SchemaManager
- ✅ 获取Schema (`get_schema`)
- ✅ 列出所有Schema (`list_schemas`)
- ✅ 检查Schema存在 (`has_schema`)

#### StorageClient
- ✅ 执行存储操作 (`execute`)
- ✅ 检查连接状态 (`is_connected`)

## 单节点场景下的核心需求

### 1. 持久化支持（最高优先级）

#### 需求描述
单节点架构下，数据持久化是最基本的需求。当前所有Manager都是纯内存实现，重启后数据全部丢失。

#### 需要补充的功能

##### 1.1 IndexManager持久化
**功能需求**:
- 索引定义持久化到磁盘
- 启动时从磁盘加载索引
- 索引变更时自动持久化

**实现建议**:
```rust
pub trait IndexManager: Send + Sync + std::fmt::Debug {
    // 现有功能
    fn get_index(&self, name: &str) -> Option<Index>;
    fn list_indexes(&self) -> Vec<String>;
    fn has_index(&self, name: &str) -> bool;
    
    // 新增功能
    fn create_index(&self, index: Index) -> Result<(), String>;
    fn drop_index(&self, name: &str) -> Result<(), String>;
    fn load_from_disk(&self) -> Result<(), String>;
    fn save_to_disk(&self) -> Result<(), String>;
}

#[derive(Debug, Clone)]
pub struct PersistentIndexManager {
    indexes: Arc<RwLock<HashMap<String, Index>>>,
    storage_path: PathBuf,
}

impl PersistentIndexManager {
    pub fn new(storage_path: PathBuf) -> Result<Self, String> {
        let mut manager = Self {
            indexes: Arc::new(RwLock::new(HashMap::new())),
            storage_path,
        };
        manager.load_from_disk()?;
        Ok(manager)
    }
    
    fn load_from_disk(&self) -> Result<(), String> {
        // 从磁盘加载索引定义
    }
    
    fn save_to_disk(&self) -> Result<(), String> {
        // 保存索引定义到磁盘
    }
}
```

##### 1.2 MetaClient持久化
**功能需求**:
- Space定义持久化
- 启动时加载Space信息
- Space变更时自动持久化

**实现建议**:
```rust
pub trait MetaClient: Send + Sync + std::fmt::Debug {
    // 现有功能
    fn get_cluster_info(&self) -> Result<ClusterInfo, String>;
    fn get_space_info(&self, space_id: i32) -> Result<SpaceInfo, String>;
    fn is_connected(&self) -> bool;
    
    // 新增功能
    fn create_space(&self, space: SpaceDesc) -> Result<i32, String>;
    fn drop_space(&self, space_id: i32) -> Result<(), String>;
    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, String>;
    fn load_spaces(&self) -> Result<(), String>;
    fn save_spaces(&self) -> Result<(), String>;
}
```

##### 1.3 SchemaManager持久化
**功能需求**:
- Schema定义持久化
- 启动时加载Schema
- Schema变更时自动持久化

**实现建议**:
```rust
pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    // 现有功能
    fn get_schema(&self, name: &str) -> Option<Schema>;
    fn list_schemas(&self) -> Vec<String>;
    fn has_schema(&self, name: &str) -> bool;
    
    // 新增功能
    fn create_schema(&self, schema: Schema) -> Result<(), String>;
    fn drop_schema(&self, name: &str) -> Result<(), String>;
    fn update_schema(&self, name: &str, schema: Schema) -> Result<(), String>;
    fn load_schemas(&self) -> Result<(), String>;
    fn save_schemas(&self) -> Result<(), String>;
}
```

### 2. Schema管理功能（高优先级）

#### 需求描述
单节点场景下需要完整的Schema管理功能，包括Tag和EdgeType的定义、修改、删除。

#### 需要补充的功能

##### 2.1 Tag管理
**功能需求**:
- 创建Tag定义
- 修改Tag定义（添加字段）
- 删除Tag定义
- 获取Tag定义

**实现建议**:
```rust
#[derive(Debug, Clone)]
pub struct TagSchema {
    pub name: String,
    pub fields: Vec<Field>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default_value: Option<Value>,
}

pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    // Tag管理
    fn create_tag(&self, space_id: i32, tag: TagSchema) -> Result<i32, String>;
    fn alter_tag(&self, space_id: i32, tag_id: i32, add_fields: Vec<Field>) -> Result<(), String>;
    fn drop_tag(&self, space_id: i32, tag_id: i32) -> Result<(), String>;
    fn get_tag(&self, space_id: i32, tag_id: i32) -> Option<TagSchema>;
    fn list_tags(&self, space_id: i32) -> Result<Vec<TagSchema>, String>;
}
```

##### 2.2 EdgeType管理
**功能需求**:
- 创建EdgeType定义
- 修改EdgeType定义（添加字段）
- 删除EdgeType定义
- 获取EdgeType定义

**实现建议**:
```rust
#[derive(Debug, Clone)]
pub struct EdgeTypeSchema {
    pub name: String,
    pub fields: Vec<Field>,
    pub comment: Option<String>,
}

pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    // EdgeType管理
    fn create_edge_type(&self, space_id: i32, edge_type: EdgeTypeSchema) -> Result<i32, String>;
    fn alter_edge_type(&self, space_id: i32, edge_type_id: i32, add_fields: Vec<Field>) -> Result<(), String>;
    fn drop_edge_type(&self, space_id: i32, edge_type_id: i32) -> Result<(), String>;
    fn get_edge_type(&self, space_id: i32, edge_type_id: i32) -> Option<EdgeTypeSchema>;
    fn list_edge_types(&self, space_id: i32) -> Result<Vec<EdgeTypeSchema>, String>;
}
```

### 3. 索引管理功能（高优先级）

#### 需求描述
单节点场景下需要完整的索引管理功能，包括索引的创建、删除、状态跟踪。

#### 需要补充的功能

##### 3.1 索引生命周期管理
**功能需求**:
- 创建索引
- 删除索引
- 查询索引状态
- 列出所有索引

**实现建议**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum IndexStatus {
    Creating,
    Building,
    Active,
    Dropped,
}

#[derive(Debug, Clone)]
pub struct Index {
    pub id: i32,
    pub name: String,
    pub space_id: i32,
    pub schema_name: String,  // Tag或EdgeType名称
    pub fields: Vec<String>,  // 索引字段
    pub index_type: IndexType,
    pub status: IndexStatus,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IndexType {
    TagIndex,
    EdgeIndex,
    FulltextIndex,
}

pub trait IndexManager: Send + Sync + std::fmt::Debug {
    // 现有功能
    fn get_index(&self, name: &str) -> Option<Index>;
    fn list_indexes(&self) -> Vec<String>;
    fn has_index(&self, name: &str) -> bool;
    
    // 新增功能
    fn create_index(&self, space_id: i32, index: Index) -> Result<i32, String>;
    fn drop_index(&self, space_id: i32, index_id: i32) -> Result<(), String>;
    fn get_index_status(&self, space_id: i32, index_id: i32) -> Option<IndexStatus>;
    fn list_indexes_by_space(&self, space_id: i32) -> Result<Vec<Index>, String>;
}
```

##### 3.2 索引构建功能
**功能需求**:
- 异步构建索引
- 查询索引构建进度
- 取消索引构建

**实现建议**:
```rust
#[derive(Debug, Clone)]
pub struct IndexBuildProgress {
    pub index_id: i32,
    pub total_vertices: u64,
    pub processed_vertices: u64,
    pub percentage: f64,
    pub status: IndexBuildStatus,
}

#[derive(Debug, Clone)]
pub enum IndexBuildStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
}

pub trait IndexManager: Send + Sync + std::fmt::Debug {
    fn build_index(&self, space_id: i32, index_id: i32) -> Result<(), String>;
    fn get_build_progress(&self, space_id: i32, index_id: i32) -> Option<IndexBuildProgress>;
    fn cancel_build(&self, space_id: i32, index_id: i32) -> Result<(), String>;
}
```

### 4. 数据操作功能（最高优先级）

#### 需求描述
单节点场景下需要完整的数据操作功能，包括Vertex和Edge的增删改查。

#### 需要补充的功能

##### 4.1 Vertex操作
**功能需求**:
- 添加Vertex
- 删除Vertex
- 更新Vertex属性
- 查询Vertex

**实现建议**:
```rust
#[derive(Debug, Clone)]
pub struct Vertex {
    pub id: VertexID,
    pub tags: Vec<TagData>,
}

#[derive(Debug, Clone)]
pub struct TagData {
    pub tag_id: i32,
    pub props: HashMap<String, Value>,
}

pub type VertexID = i64;

pub trait StorageClient: Send + Sync + std::fmt::Debug {
    // Vertex操作
    fn add_vertex(&self, space_id: i32, vertex: Vertex) -> Result<(), String>;
    fn add_vertices(&self, space_id: i32, vertices: Vec<Vertex>) -> Result<(), String>;
    fn delete_vertex(&self, space_id: i32, vertex_id: VertexID) -> Result<(), String>;
    fn delete_vertices(&self, space_id: i32, vertex_ids: Vec<VertexID>) -> Result<(), String>;
    fn update_vertex(&self, space_id: i32, vertex_id: VertexID, tag_id: i32, props: HashMap<String, Value>) -> Result<(), String>;
    fn get_vertex(&self, space_id: i32, vertex_id: VertexID) -> Result<Option<Vertex>, String>;
    fn get_vertices(&self, space_id: i32, vertex_ids: Vec<VertexID>) -> Result<Vec<Vertex>, String>;
}
```

##### 4.2 Edge操作
**功能需求**:
- 添加Edge
- 删除Edge
- 更新Edge属性
- 查询Edge

**实现建议**:
```rust
#[derive(Debug, Clone)]
pub struct Edge {
    pub src: VertexID,
    pub dst: VertexID,
    pub edge_type: i32,
    pub rank: i64,
    pub props: HashMap<String, Value>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EdgeKey {
    pub src: VertexID,
    pub edge_type: i32,
    pub rank: i64,
    pub dst: VertexID,
}

pub trait StorageClient: Send + Sync + std::fmt::Debug {
    // Edge操作
    fn add_edge(&self, space_id: i32, edge: Edge) -> Result<(), String>;
    fn add_edges(&self, space_id: i32, edges: Vec<Edge>) -> Result<(), String>;
    fn delete_edge(&self, space_id: i32, edge_key: EdgeKey) -> Result<(), String>;
    fn delete_edges(&self, space_id: i32, edge_keys: Vec<EdgeKey>) -> Result<(), String>;
    fn update_edge(&self, space_id: i32, edge_key: EdgeKey, props: HashMap<String, Value>) -> Result<(), String>;
    fn get_edge(&self, space_id: i32, edge_key: EdgeKey) -> Result<Option<Edge>, String>;
    fn get_edges(&self, space_id: i32, edge_keys: Vec<EdgeKey>) -> Result<Vec<Edge>, String>;
}
```

##### 4.3 扫描操作
**功能需求**:
- 扫描所有Vertex
- 扫描指定Tag的Vertex
- 扫描指定EdgeType的Edge

**实现建议**:
```rust
pub trait StorageClient: Send + Sync + std::fmt::Debug {
    // 扫描操作
    fn scan_vertices(&self, space_id: i32, limit: i32) -> Result<Vec<Vertex>, String>;
    fn scan_vertices_by_tag(&self, space_id: i32, tag_id: i32, limit: i32) -> Result<Vec<Vertex>, String>;
    fn scan_edges(&self, space_id: i32, limit: i32) -> Result<Vec<Edge>, String>;
    fn scan_edges_by_type(&self, space_id: i32, edge_type: i32, limit: i32) -> Result<Vec<Edge>, String>;
}
```

### 5. 索引查询功能（中优先级）

#### 需求描述
单节点场景下需要支持基于索引的查询，提高查询性能。

#### 需要补充的功能

##### 5.1 索引查询
**功能需求**:
- 基于索引查询Vertex
- 基于索引查询Edge
- 支持等值查询
- 支持范围查询

**实现建议**:
```rust
#[derive(Debug, Clone)]
pub enum IndexCondition {
    Equal { field: String, value: Value },
    NotEqual { field: String, value: Value },
    LessThan { field: String, value: Value },
    LessThanOrEqual { field: String, value: Value },
    GreaterThan { field: String, value: Value },
    GreaterThanOrEqual { field: String, value: Value },
    In { field: String, values: Vec<Value> },
}

pub trait StorageClient: Send + Sync + std::fmt::Debug {
    // 索引查询
    fn lookup_vertices(&self, space_id: i32, index_id: i32, condition: IndexCondition) -> Result<Vec<VertexID>, String>;
    fn lookup_edges(&self, space_id: i32, index_id: i32, condition: IndexCondition) -> Result<Vec<EdgeKey>, String>;
    fn lookup_vertices_range(&self, space_id: i32, index_id: i32, start: Value, end: Value) -> Result<Vec<VertexID>, String>;
}
```

### 6. Space管理功能（中优先级）

#### 需求描述
单节点场景下需要支持多个Space的逻辑隔离。

#### 需要补充的功能

##### 6.1 Space生命周期管理
**功能需求**:
- 创建Space
- 删除Space
- 列出所有Space
- 获取Space信息

**实现建议**:
```rust
#[derive(Debug, Clone)]
pub struct SpaceDesc {
    pub name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
    pub vid_type: VidType,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub enum VidType {
    Int64,
    FixedString(i32),
}

#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub id: i32,
    pub name: String,
    pub partition_num: i32,
    pub replica_factor: i32,
    pub vid_type: VidType,
}

pub trait MetaClient: Send + Sync + std::fmt::Debug {
    // Space管理
    fn create_space(&self, space: SpaceDesc) -> Result<i32, String>;
    fn drop_space(&self, space_id: i32) -> Result<(), String>;
    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, String>;
    fn get_space_by_name(&self, name: &str) -> Option<SpaceInfo>;
}
```

### 7. 版本控制功能（中优先级）

#### 需求描述
单节点场景下需要支持Schema版本控制，便于Schema演进和回滚。

#### 需要补充的功能

##### 7.1 Schema版本管理
**功能需求**:
- 为Schema分配版本号
- 获取指定版本的Schema
- 获取Schema历史版本
- 支持多版本Schema共存

**实现建议**:
```rust
#[derive(Debug, Clone)]
pub struct VersionedSchema {
    pub schema: Schema,
    pub version: i64,
    pub created_at: i64,
    pub comment: Option<String>,
}

pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    // 版本管理
    fn get_schema_version(&self, space_id: i32, schema_name: &str, version: i64) -> Option<VersionedSchema>;
    fn get_latest_schema_version(&self, space_id: i32, schema_name: &str) -> Option<i64>;
    fn get_schema_history(&self, space_id: i32, schema_name: &str) -> Vec<VersionedSchema>;
    fn rollback_schema(&self, space_id: i32, schema_name: &str, version: i64) -> Result<(), String>;
}
```

### 8. 事务支持（中优先级）

#### 需求描述
单节点场景下需要支持基本的事务功能，保证数据一致性。

#### 需要补充的功能

##### 8.1 基本事务操作
**功能需求**:
- 开始事务
- 提交事务
- 回滚事务

**实现建议**:
```rust
pub trait StorageClient: Send + Sync + std::fmt::Debug {
    // 事务操作
    fn begin_transaction(&self) -> Result<TransactionID, String>;
    fn commit_transaction(&self, tx_id: TransactionID) -> Result<(), String>;
    fn rollback_transaction(&self, tx_id: TransactionID) -> Result<(), String>;
}

pub type TransactionID = u64;
```

### 9. 错误处理和重试（中优先级）

#### 需求描述
单节点场景下需要完善的错误处理和重试机制，提高系统可靠性。

#### 需要补充的功能

##### 9.1 统一错误类型
**功能需求**:
- 定义统一的错误类型
- 提供详细的错误信息
- 支持错误分类

**实现建议**:
```rust
#[derive(Debug)]
pub enum ManagerError {
    NotFound(String),
    AlreadyExists(String),
    InvalidInput(String),
    StorageError(String),
    SchemaError(String),
    IndexError(String),
    TransactionError(String),
}

impl std::fmt::Display for ManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManagerError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ManagerError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            ManagerError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ManagerError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            ManagerError::SchemaError(msg) => write!(f, "Schema error: {}", msg),
            ManagerError::IndexError(msg) => write!(f, "Index error: {}", msg),
            ManagerError::TransactionError(msg) => write!(f, "Transaction error: {}", msg),
        }
    }
}

impl std::error::Error for ManagerError {}
```

##### 9.2 重试机制
**功能需求**:
- 自动重试失败的操作
- 可配置的重试策略
- 指数退避算法

**实现建议**:
```rust
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

pub fn retry_with_backoff<F, T, E>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut delay = config.initial_delay_ms;
    
    for attempt in 0..config.max_attempts {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if attempt < config.max_attempts - 1 => {
                std::thread::sleep(Duration::from_millis(delay));
                delay = (delay as f64 * config.backoff_multiplier) as u64;
                delay = delay.min(config.max_delay_ms);
            }
            Err(e) => return Err(e),
        }
    }
    
    unreachable!()
}
```

### 10. 统计信息收集（低优先级）

#### 需求描述
单节点场景下需要收集统计信息，便于监控和优化。

#### 需要补充的功能

##### 10.1 索引统计信息
**功能需求**:
- 收集索引使用统计
- 收集索引大小统计
- 收集索引查询性能统计

**实现建议**:
```rust
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub index_id: i32,
    pub index_name: String,
    pub total_rows: u64,
    pub distinct_values: u64,
    pub null_count: u64,
    pub avg_size_bytes: u64,
    pub query_count: u64,
    pub last_query_time: Option<i64>,
}

pub trait IndexManager: Send + Sync + std::fmt::Debug {
    fn get_index_stats(&self, space_id: i32, index_id: i32) -> Option<IndexStats>;
    fn update_index_stats(&self, space_id: i32, index_id: i32) -> Result<(), String>;
}
```

##### 10.2 Schema统计信息
**功能需求**:
- 收集Vertex数量统计
- 收集Edge数量统计
- 收集数据分布统计

**实现建议**:
```rust
#[derive(Debug, Clone)]
pub struct SchemaStats {
    pub schema_name: String,
    pub vertex_count: u64,
    pub edge_count: u64,
    pub avg_degree: f64,
}

pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    fn get_schema_stats(&self, space_id: i32, schema_name: &str) -> Option<SchemaStats>;
}
```

## 不需要实现的功能（分布式相关）

以下功能是分布式场景特有的，单节点场景下不需要实现：

### ❌ 不需要的功能

1. **分布式元数据同步**
   - Leader选举
   - Raft协议
   - 元数据复制

2. **分区管理**
   - Partition分配
   - Partition迁移
   - Partition负载均衡

3. **分布式存储**
   - 多副本管理
   - 数据分片
   - 跨节点查询

4. **集群管理**
   - 节点管理
   - 集群拓扑
   - 故障检测

5. **分布式事务**
   - 两阶段提交
   - 分布式锁
   - 跨节点事务

6. **连接池和负载均衡**
   - 多节点连接池
   - 负载均衡策略
   - 故障转移

## 实施优先级

### 第一阶段（核心功能，立即实施）

1. **持久化支持** ⭐⭐⭐
   - IndexManager持久化
   - MetaClient持久化
   - SchemaManager持久化

2. **数据操作功能** ⭐⭐⭐
   - Vertex操作（增删改查）
   - Edge操作（增删改查）
   - 扫描操作

3. **Schema管理功能** ⭐⭐⭐
   - Tag管理
   - EdgeType管理

### 第二阶段（增强功能，近期实施）

4. **索引管理功能** ⭐⭐
   - 索引生命周期管理
   - 索引构建功能

5. **索引查询功能** ⭐⭐
   - 基于索引的查询
   - 范围查询

6. **Space管理功能** ⭐⭐
   - Space生命周期管理

### 第三阶段（优化功能，中期实施）

7. **版本控制功能** ⭐
   - Schema版本管理

8. **事务支持** ⭐
   - 基本事务操作

9. **错误处理和重试** ⭐
   - 统一错误类型
   - 重试机制

### 第四阶段（监控功能，长期规划）

10. **统计信息收集** ⭐
    - 索引统计信息
    - Schema统计信息

## 实施建议

### 1. 渐进式实施

按照优先级分阶段实施，每个阶段完成后进行测试和验证。

### 2. 保持接口稳定

在实施过程中保持接口的稳定性，避免频繁的接口变更。

### 3. 充分测试

每个功能实现后都要进行充分的单元测试和集成测试。

### 4. 文档完善

及时更新文档，记录接口设计和使用方法。

### 5. 性能优化

在功能完善的基础上，逐步进行性能优化。

## 总结

在保持单节点架构的情况下，managers目录需要补充的核心功能包括：

1. **持久化支持** - 保证数据不丢失
2. **数据操作功能** - 支持基本的Vertex和Edge操作
3. **Schema管理功能** - 支持Tag和EdgeType管理
4. **索引管理功能** - 支持索引的创建和管理
5. **索引查询功能** - 支持基于索引的查询
6. **Space管理功能** - 支持多Space隔离
7. **版本控制功能** - 支持Schema版本管理
8. **事务支持** - 支持基本的事务操作
9. **错误处理和重试** - 提高系统可靠性
10. **统计信息收集** - 支持监控和优化

建议按照上述优先级分阶段实施，优先完成核心功能，然后逐步增强和优化。
