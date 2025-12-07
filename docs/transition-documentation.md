# NebulaGraph 到 GraphDB 的架构迁移文档

## 概述

本文档详细描述了从旧的 NebulaGraph C++ 实现到新的 Rust 实现（GraphDB）的架构迁移过程。GraphDB 是一个轻量级的单节点图数据库，专注于个人和小规模应用场景，通过使用 Rust 语言和消除分布式功能来显著减少外部依赖。

## 架构迁移目标

### 核心目标
- 使用 Rust 语言重写，利用其内存安全和并发优势
- 移除分布式功能，专注于单机性能
- 大幅减少外部依赖，使用 Rust 生态系统
- 保持核心图数据库功能完整性
- 提供高性能的单机图处理能力
- 实现单一可执行文件部署

### 非目标
- 分布式支持
- 集群管理和容错
- 企业级安全和认证
- 复杂的多用户并发控制

## 模块对应关系

### 旧架构模块映射

| 旧模块 (NebulaGraph C++) | 新模块 (GraphDB Rust) | 说明 |
|------------------------|----------------------|------|
| `src/clients` | `graphDB/src/api` | 客户端功能集成到 API 层，提供 CLI、HTTP API 和 Rust 客户端 |
| `src/common` | `graphDB/src/utils`, `graphDB/src/core` | 通用工具函数移至 utils 模块；核心数据类型定义移至 core 模块 |
| `src/graph` | `graphDB/src/query` | 图查询引擎功能整合到查询模块，包含查询解析和执行 |
| `src/storage` | `graphDB/src/storage` | 存储引擎功能保留，但简化为单机实现 |
| `src/meta` | `graphDB/src/storage`, `graphDB/src/core` | 元数据管理集成到存储和核心模块 |
| `src/kvstore` | `graphDB/src/storage` | 键值存储功能整合到新的存储模块 |
| `src/daemons` | 集成在二进制文件中 | 服务守护进程功能嵌入到单一可执行文件中 |
| `src/parser` | `graphDB/src/query` | 查询解析器与查询引擎深度集成 |
| `src/webservice` | `graphDB/src/api` | Web 服务功能作为 API 层的一部分 |

## 架构差异分析

### 1. 编程语言差异
- **旧架构**: C++ 实现，需要手动内存管理，存在内存安全风险
- **新架构**: Rust 实现，利用所有权系统保证内存安全和并发安全

### 2. 系统架构差异
- **旧架构**: 分布式系统，包含多个独立服务（graphd、metad、storaged）
- **新架构**: 单节点架构，所有功能集成在单一进程中

### 3. 部署模型差异
- **旧架构**: 需要 Docker 容器编排，多个服务实例
- **新架构**: 单一可执行文件，简化部署和分发

### 4. 外部依赖差异
- **旧架构**: 依赖大量外部库（如 Facebook Thrift 生态、RocksDB、Boost 等）
- **新架构**: 极少外部依赖，使用 Rust 生态系统（sled、tokio、serde 等）

### 5. 通信模型差异
- **旧架构**: 网络通信，服务间通过 Thrift/RPC 协议通信
- **新架构**: 进程内通信，使用内存共享和消息传递

## 新架构详细设计

### 核心模块设计

#### 1. 核心数据结构模块 (`core`)
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

#### 2. 存储引擎模块 (`storage`)
```rust
// 存储引擎接口
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

#### 3. 查询引擎模块 (`query`)
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

#### 4. 事务管理模块 (`transaction`)
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

## 开发实施路线图

### Phase 1: 项目设置和核心基础设施 (Weeks 1-2)
1. 创建 Rust 项目结构
2. 定义核心数据结构
3. 设置基本配置管理

### Phase 2: 存储层实现 (Weeks 3-5)
1. 实现存储引擎 trait
2. 实现基本 CRUD 操作
3. 添加事务支持

### Phase 3: 查询和解析引擎 (Weeks 6-8)
1. 实现查询语言解析器
2. 实现查询执行引擎
3. 添加查询优化

### Phase 4: 索引系统 (Weeks 9-10)
1. 实现索引机制
2. 集成存储和查询层

### Phase 5: API 和服务层 (Weeks 11-12)
1. 实现 API 层
2. 实现服务管理

### Phase 6: 事务管理 (Weeks 13-14)
1. 增强事务系统
2. 实现事务性查询

### Phase 7: 性能优化和测试 (Weeks 15-16)
1. 性能优化
2. 全面测试

### Phase 8: 文档和打包 (Week 17)
1. 创建文档
2. 打包和部署

## 依赖项分析

### Rust 生态依赖策略

#### 1. 网络和协议层替换
- `tokio` - 异步运行时（替代 Wangle）
- `hyper/axum` - HTTP 实现（替代 Proxygen）
- `tonic` - gRPC 实现（可选）

#### 2. 存储引擎替换
- `sled` 或 `redb` - Rust 原生嵌入式数据库（替代 RocksDB）

#### 3. 日志和错误处理替换
- `log/env_logger` - 日志记录（替代 Glog）
- `clap` - 命令行参数解析（替代 Gflags）
- `anyhow/thiserror` - 错误处理

#### 4. 安全功能替换
- `ring` - 加密库（替代 OpenSSL）
- `rustls` - TLS 实现

## 性能和安全性优势

### 内存安全
- Rust 的所有权系统在编译时消除内存安全问题
- 无需外部工具检测内存错误

### 并发安全
- 无需外部库防止数据竞争
- 静态保证线程安全

### 性能优势
- 零成本抽象，编译期优化
- 无垃圾回收停顿
- 高效的内存管理

## 结论

通过使用 Rust 重写并移除分布式功能，GraphDB 实现了以下架构改进：

1. **架构简化** - 移除复杂的分布式组件，专注于单机性能
2. **依赖减少** - 外部依赖减少约 70-80%，提高可维护性
3. **性能提升** - 消除网络延迟，优化本地访问
4. **安全性增强** - 利用 Rust 的内存安全特性
5. **开发效率** - 利用 Rust 的现代语言特性

这种架构特别适合需要高性能单机图数据库的场景，如个人项目、开发测试环境或边缘计算场景。