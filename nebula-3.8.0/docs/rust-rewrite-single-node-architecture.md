# Rust 重写 NebulaGraph 架构修改分析

## 概述

本文档分析了将 NebulaGraph 用 Rust 重新实现，移除分布式实现，并使用 Rust 生态系统替换大多数外部依赖后的架构修改方案。该方案将创建一个轻量级的单机图数据库，专注于个人或小规模使用场景。

## 设计目标

### 核心目标
- 使用 Rust 完全重写，利用其内存安全和并发优势
- 移除分布式功能，专注于单机性能
- 大幅减少外部依赖，使用 Rust 生态系统
- 保持核心图数据库功能完整性
- 提供高性能的单机图处理能力

### 非目标
- 分布式支持
- 集群管理和容错
- 企业级安全和认证
- 复杂的并发控制（多用户场景）

## 架构修改

### 1. 移除分布式相关模块

#### 原有的分布式组件移除
- RAFT 协议实现 (`src/kvstore/raftex`)
- 服务发现和负载均衡
- 跨节点通信协议
- 分区管理 (`src/kvstore/Part`)
- 一致性算法实现

#### 保留的单机功能
- 存储引擎
- 查询引擎
- 事务支持
- Schema 管理

### 2. 新架构设计

```
┌─────────────────┐
│   应用层        │
│                 │
│ - CLI 工具      │
│ - HTTP API      │
│ - Rust 客户端   │
└─────────────────┘
         │
┌─────────────────┐
│   服务层        │
│                 │
│ - 查询引擎      │
│ - 事务管理      │
│ - 缓存系统      │
│ - 权限控制      │
└─────────────────┘
         │
┌─────────────────┐
│   存储层        │
│                 │
│ - 存储引擎      │
│ - 索引系统      │
│ - 文件管理      │
└─────────────────┘
         │
┌─────────────────┐
│   系统层        │
│                 │
│ - 内存管理      │
│ - 错误处理      │
│ - 日志系统      │
└─────────────────┘
```

### 3. 核心模块重构

#### 3.1 存储引擎模块 (`storage`)
```rust
// 单机存储引擎，无需分区和分布式逻辑
pub trait StorageEngine {
    fn insert_node(&mut self, node: Node) -> Result<u64, StorageError>;
    fn get_node(&self, id: u64) -> Result<Option<Node>, StorageError>;
    fn update_node(&mut self, node: Node) -> Result<(), StorageError>;
    fn delete_node(&mut self, id: u64) -> Result<(), StorageError>;
    
    fn insert_edge(&mut self, edge: Edge) -> Result<u64, StorageError>;
    fn get_edge(&self, id: u64) -> Result<Option<Edge>, StorageError>;
    fn get_node_edges(&self, node_id: u64, direction: Direction) -> Result<Vec<Edge>, StorageError>;
    
    // 单机事务支持
    fn begin_transaction(&mut self) -> Result<TransactionId, StorageError>;
    fn commit_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError>;
    fn rollback_transaction(&mut self, tx_id: TransactionId) -> Result<(), StorageError>;
}

// 使用 Rust 原生嵌入式数据库，如 sled 或 redb
pub struct NativeStorage {
    db: sled::Db,  // 或 redb::Database
    node_index: sled::Tree,  // 节点索引
    edge_index: sled::Tree,  // 边索引
    schema: sled::Tree,      // Schema 信息
}
```

#### 3.2 查询引擎模块 (`query`)
```rust
// 简化的查询引擎，无需分布式查询规划
pub struct QueryEngine {
    storage: Arc<RwLock<dyn StorageEngine>>,
    parser: QueryParser,
    optimizer: QueryOptimizer,
}

impl QueryEngine {
    pub fn execute(&self, query: Query) -> Result<QueryResult, QueryError> {
        // 本地查询执行，无需跨节点协调
        match query {
            Query::CreateNode { labels, properties } => {
                let node = Node::new(labels, properties);
                let id = self.storage.write().unwrap().insert_node(node)?;
                Ok(QueryResult::NodeId(id))
            }
            Query::Match { pattern, conditions } => {
                // 本地匹配，无需分布式计算
                self.execute_local_match(pattern, conditions)
            }
            // 其他查询类型
        }
    }
}
```

#### 3.3 事务管理模块 (`transaction`)
```rust
// 简化的事务管理，不再需要分布式事务协议
pub struct TransactionManager {
    storage: Arc<RwLock<dyn StorageEngine>>,
    active_transactions: HashMap<TransactionId, TransactionState>,
}

pub struct Transaction {
    id: TransactionId,
    operations: Vec<Operation>,
    committed: bool,
    // 本地事务控制，替代分布式事务协调
}
```

## Rust 生态依赖替换策略

### 1. 网络和协议层替换
**原 C++ 依赖**：
- Fbthrift, Wangle, Proxygen

**Rust 替代方案**：
- `tokio` - 异步运行时
- `hyper` - HTTP 实现
- `tonic` - gRPC 实现（如果需要）
- `axum` - Web 框架

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
hyper = { version = "0.14", features = ["full"] }
tonic = "0.8"  # 可选
axum = "0.6"
```

### 2. 存储引擎替换
**原 C++ 依赖**：
- RocksDB

**Rust 替代方案**：
- `sled` - 高性能嵌入式数据库
- `redb` - Rust 原生嵌入式数据库
- `lmdb` - 如果需要 LMDB

```toml
[dependencies]
sled = "0.34"  # 或使用 redb = "0.11"
```

### 3. 日志和错误处理替换
**原 C++ 依赖**：
- Glog, Gflags

**Rust 替代方案**：
- `log` / `env_logger` - 日志记录
- `clap` - 命令行参数解析
- `anyhow` / `thiserror` - 错误处理

```toml
[dependencies]
log = "0.4"
env_logger = "0.10"
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
```

### 4. 加密和安全替换
**原 C++ 依赖**：
- OpenSSL, Sodium

**Rust 替代方案**：
- `ring` - 加密库
- `rustls` - TLS 实现
- `argon2` - 密码哈希

```toml
[dependencies]
ring = "0.16"
rustls = "0.20"
argon2 = "0.4"  # 可选
```

### 5. 数据处理和序列化替换
**原 C++ 依赖**：
- DoubleConversion, various string utilities

**Rust 替代方案**：
- `serde` - 序列化/反序列化
- `serde_json` - JSON 处理
- 内置的字符串处理

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 6. 压缩功能替换
**原 C++ 依赖**：
- Snappy, Zstd, Bzip2

**Rust 替代方案**：
- `flate2` - zlib/deflate
- `snap` - Snappy 兼容
- `zstd` - Zstd 实现

```toml
[dependencies]
flate2 = "1.0"
snap = "1.0"  # 如果需要 Snappy 兼容
zstd = "0.12"
```

## 架构组件详细设计

### 1. 图数据库核心 (`graph`)
```rust
pub struct GraphDatabase {
    storage: Arc<RwLock<NativeStorage>>,
    query_engine: Arc<QueryEngine>,
    transaction_manager: Arc<Mutex<TransactionManager>>,
    // 单机环境下的缓存和索引
    node_cache: Arc<Mutex<LruCache<u64, Node>>>,
    index_manager: Arc<IndexManager>,
}

impl GraphDatabase {
    pub fn new(config: Config) -> Result<Self, DbError> {
        let db = sled::open(&config.data_dir)?;
        let storage = NativeStorage::new(db)?;
        
        Ok(GraphDatabase {
            storage: Arc::new(RwLock::new(storage)),
            query_engine: Arc::new(QueryEngine::new()),
            transaction_manager: Arc::new(Mutex::new(TransactionManager::new())),
            node_cache: Arc::new(Mutex::new(LruCache::new(CACHE_SIZE))),
            index_manager: Arc::new(IndexManager::new()),
        })
    }
}
```

### 2. 查询语言和解析器 (`parser`)
```rust
// 简化的解析器，不再需要复杂的分布式查询解析
pub struct QueryParser;

impl QueryParser {
    pub fn parse(&self, query_string: &str) -> Result<Query, ParseError> {
        // 简化的语法解析，专注于本地执行的查询
        // 移除了分布式查询规划相关的复杂性
        todo!()
    }
}
```

### 3. 索引系统 (`index`)
```rust
// 简化的索引系统，无需分布式索引
pub struct IndexManager {
    label_index: HashMap<String, BTreeSet<u64>>,      // 标签到节点ID
    property_index: HashMap<String, HashMap<Value, BTreeSet<u64>>>,  // 属性索引
    edge_index: HashMap<u64, Vec<u64>>,               // 节点到边的索引
}
```

## 性能优化策略

### 1. 内存管理
- 利用 Rust 的所有权系统实现零拷贝数据访问
- 使用内存映射文件优化大文件访问
- 实现对象池减少分配/释放开销

### 2. 并发控制
- 使用 `Arc` 和 `RwLock` 实现高效的读写分离
- 避免全局锁，使用分段锁策略

### 3. 缓存策略
- 多层缓存策略（L1: CPU缓存, L2: 内存缓存, L3: SSD缓存）
- 智能预取算法

## 移除分布式功能后的架构优势

### 1. 简化设计
- 消除了复杂的分布式一致性问题
- 简化的事务模型（单机事务 vs 分布式事务）
- 无需分区和路由逻辑

### 2. 性能提升
- 消除网络延迟
- 减少序列化/反序列化开销
- 更直接的数据访问路径

### 3. 降低依赖
- 移除所有分布式协调库
- 减少网络协议栈依赖
- 消除集群管理依赖

## 依赖项减少评估

### 预期减少的依赖（约 70-80%）
1. **完全消除**：
   - RAFT 实现相关库
   - 分布式协调库
   - 复杂的 Thrift 生态栈

2. **简化替代**：
   - RocksDB -> sled/redb
   - OpenSSL -> ring/rustls
   - Boost -> Rust 标准库

3. **内置功能**：
   - 内存安全（无需外部工具）
   - 简化错误处理
   - 内置并发安全

### 保留的必要依赖
1. **存储后端**：sled 或 redb
2. **异步运行时**：tokio
3. **序列化**：serde
4. **日志**：log + env_logger

## 结论

通过使用 Rust 重写并移除分布式功能，可以实现以下架构改进：

1. **架构简化**：移除复杂的分布式组件，专注于单机性能
2. **依赖减少**：外部依赖减少约 70-80%，提高可维护性
3. **性能提升**：消除网络延迟，优化本地访问
4. **安全性增强**：利用 Rust 的内存安全特性
5. **开发效率**：利用 Rust 的现代语言特性，如模式匹配、错误处理等

这种架构特别适合需要高性能单机图数据库的场景，如个人项目、开发测试环境或边缘计算场景。虽然牺牲了分布式扩展能力，但获得了更简单、更高效、更安全的架构。