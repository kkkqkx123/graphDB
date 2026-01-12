# Rust 图数据库基本功能初步架构设计

## 概述
本文档描述了使用 Rust 实现基本图数据库功能的初步架构设计，专注于个人使用场景，以最小化外部依赖为目标。

## 设计目标

### 核心目标
- 实现基本的图数据库功能（节点、边、属性）
- 最小化外部依赖（基于前面的分析）
- 优化个人使用场景
- 确保内存安全和线程安全

### 非目标
- 分布式支持
- 高性能硬件优化
- 企业级安全和认证
- 复杂的并发控制

## 整体架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   应用层        │    │   服务层        │    │   存储层        │
│                 │    │                 │    │                 │
│ - CLI 工具      │◄──►│ - 查询引擎      │◄──►│ - 数据存储      │
│ - 简单 API      │    │ - 事务管理      │    │ - 索引系统      │
│ - 导入/导出     │    │ - 缓存管理      │    │ - 文件管理      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 模块设计

### 1. 核心数据结构模块 (`core`)
```rust
// 节点结构
pub struct Node {
    id: u64,
    labels: Vec<String>,
    properties: HashMap<String, Value>,
}

// 边结构
pub struct Edge {
    id: u64,
    from_node: u64,  // 起始节点ID
    to_node: u64,    // 终止节点ID
    edge_type: String,
    properties: HashMap<String, Value>,
}

// 值类型枚举
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
}
```

### 2. 存储引擎模块 (`storage`)
```rust
// 简化存储引擎接口
pub trait StorageEngine {
    fn insert_node(&mut self, node: Node) -> Result<u64, StorageError>;
    fn get_node(&self, id: u64) -> Result<Option<Node>, StorageError>;
    fn update_node(&mut self, node: Node) -> Result<(), StorageError>;
    fn delete_node(&mut self, id: u64) -> Result<(), StorageError>;
    
    fn insert_edge(&mut self, edge: Edge) -> Result<u64, StorageError>;
    fn get_edge(&self, id: u64) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(&self, node_id: u64, direction: Direction) -> Result<Vec<Edge>, StorageError>;
}

// 内存存储实现
pub struct MemoryStorage {
    nodes: HashMap<u64, Node>,
    edges: HashMap<u64, Edge>,
    // 节点到边的索引
    node_edge_index: HashMap<u64, Vec<u64>>,
}

// 可选的文件存储实现
pub struct FileStorage {
    data_dir: PathBuf,
    memory_cache: MemoryStorage,
}
```

### 3. 查询引擎模块 (`query`)
```rust
// 简化的查询语言
pub enum Query {
    CreateNode { labels: Vec<String>, properties: HashMap<String, Value> },
    CreateEdge { from: u64, to: u64, edge_type: String, properties: HashMap<String, Value> },
    MatchNodes { labels: Option<Vec<String>>, conditions: Vec<Condition> },
    DeleteNode { id: u64 },
    UpdateNode { id: u64, updates: HashMap<String, Value> },
}

// 查询执行器
pub struct QueryExecutor {
    storage: Box<dyn StorageEngine>,
}

impl QueryExecutor {
    pub fn execute(&mut self, query: Query) -> Result<QueryResult, QueryError> {
        match query {
            Query::CreateNode { labels, properties } => {
                let node = Node::new(labels, properties);
                let id = self.storage.insert_node(node)?;
                Ok(QueryResult::NodeId(id))
            }
            // 其他查询类型的实现
        }
    }
}
```

### 4. 事务管理模块 (`transaction`)
```rust
pub struct Transaction {
    id: u64,
    operations: Vec<Operation>,
    committed: bool,
}

pub enum Operation {
    InsertNode(Node),
    UpdateNode(Node),
    DeleteNode(u64),
    InsertEdge(Edge),
    DeleteEdge(u64),
}

// 简化的事务管理器
pub struct TransactionManager {
    current_tx_id: AtomicU64,
    pending_transactions: HashMap<u64, Transaction>,
}
```

## 核心组件设计

### 1. 图数据库主类
```rust
pub struct GraphDatabase {
    storage: Box<dyn StorageEngine>,
    query_executor: QueryExecutor,
    transaction_manager: TransactionManager,
    cache: Option<LruCache<u64, Node>>, // 可选缓存
}

impl GraphDatabase {
    pub fn new(config: Config) -> Result<Self, DbError> {
        let storage: Box<dyn StorageEngine> = match config.storage_type {
            StorageType::Memory => Box::new(MemoryStorage::new()),
            StorageType::File => Box::new(FileStorage::new(config.data_dir)?),
        };
        
        Ok(GraphDatabase {
            storage,
            query_executor: QueryExecutor::new(storage),
            transaction_manager: TransactionManager::new(),
            cache: if config.enable_cache { 
                Some(LruCache::new(config.cache_size)) 
            } else { None },
        })
    }
    
    pub fn execute_query(&mut self, query: Query) -> Result<QueryResult, DbError> {
        self.query_executor.execute(query)
    }
}
```

### 2. 查询解析器
```rust
pub struct QueryParser;

impl QueryParser {
    pub fn parse(&self, query_string: &str) -> Result<Query, ParseError> {
        // 简化的语法解析，例如：
        // "CREATE NODE (name='John', age=30)"
        // "MATCH (person:Person) WHERE age > 25"
        
        let tokens: Vec<&str> = query_string.split_whitespace().collect();
        
        match tokens[0].to_uppercase().as_str() {
            "CREATE" => self.parse_create(tokens),
            "MATCH" => self.parse_match(tokens),
            "DELETE" => self.parse_delete(tokens),
            _ => Err(ParseError::UnknownCommand),
        }
    }
}
```

## 内存管理策略

### 1. 对象池
```rust
// 用于频繁创建/销毁的对象的对象池
pub struct ObjectPool<T> {
    pool: Vec<T>,
    max_size: usize,
}

impl<T: Default + Clone> ObjectPool<T> {
    pub fn get(&mut self) -> T {
        self.pool.pop().unwrap_or_default()
    }
    
    pub fn put(&mut self, obj: T) {
        if self.pool.len() < self.max_size {
            self.pool.push(obj);
        }
    }
}
```

### 2. 内存池
```rust
// 预分配内存块以减少分配器调用
pub struct MemoryPool {
    blocks: Vec<Vec<u8>>,
    current_block_index: usize,
    current_offset: usize,
}
```

## 性能优化考虑

### 1. 索引系统
```rust
// 简化的节点标签索引
pub struct LabelIndex {
    indices: HashMap<String, BTreeSet<u64>>, // 标签到节点ID的映射
}

// 属性值索引
pub struct PropertyIndex {
    indices: HashMap<String, HashMap<Value, BTreeSet<u64>>>, // 属性名 -> (值 -> 节点ID集合)
}
```

### 2. 缓存策略
```rust
// LRU 缓存用于频繁访问的节点
pub struct NodeCache {
    cache: LruCache<u64, Node>,
    hits: AtomicU64,
    misses: AtomicU64,
}
```

## 配置管理

### 1. 配置结构
```rust
#[derive(Deserialize)]
pub struct Config {
    pub storage_type: StorageType,
    pub data_dir: Option<PathBuf>,
    pub cache_size: usize,
    pub enable_cache: bool,
    pub max_connections: usize,
    pub transaction_timeout: Duration,
}

pub enum StorageType {
    Memory,
    File,
}
```

### 2. 配置文件 (config.toml)
```toml
[database]
storage_type = "Memory"  # 或 "File"
data_dir = "./data"
enable_cache = true
cache_size = 1000

[performance]
max_connections = 1
transaction_timeout = 30
```

## 错误处理设计

```rust
#[derive(Debug)]
pub enum DbError {
    StorageError(StorageError),
    QueryError(QueryError),
    ParseError(ParseError),
    TransactionError(TransactionError),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DbError::StorageError(e) => write!(f, "Storage error: {}", e),
            DbError::QueryError(e) => write!(f, "Query error: {}", e),
            // ...
        }
    }
}
```

## 安全考虑

### 1. 内存安全
- 依赖 Rust 的所有权系统保证内存安全
- 无需外部内存安全工具

### 2. 简化认证
- 个人使用场景可选的简单认证
- 配置文件中存储固定密码（可选）

### 3. 数据验证
- 查询参数的类型验证
- 防止简单的注入攻击

## 构建和部署

### Cargo.toml (最小依赖)
```toml
[package]
name = "simple-graph-db"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
lru = "0.12"  # 可选依赖，用于缓存
toml = "0.8"  # 配置文件解析

[dev-dependencies]
tokio-test = "0.4"
```

## 总结

该架构设计专注于：

1. **最小化依赖**：仅使用必要的 Rust crate
2. **内存安全**：利用 Rust 的所有权系统
3. **简单性**：个人使用场景的简化功能
4. **模块化**：清晰的模块边界便于维护

此架构可作为实现轻量级个人图数据库的基础，相比原始的 C++ 实现，依赖项将大幅减少，同时保持核心图数据库功能。