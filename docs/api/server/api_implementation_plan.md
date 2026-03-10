# Server API 分阶段实现方案

## 概述

本文档基于 `src/api/embedded` 和 `src/api/embedded/c_api` 的功能分析，制定 `src/api/server` 模块的补充计划。

## 当前状态分析

### Embedded API 功能清单

| 模块 | 功能描述 | 对应文件 |
|------|---------|---------|
| database | 数据库打开、关闭、配置管理 | `embedded/database.rs` |
| session | 会话管理、图空间切换、查询执行 | `embedded/session.rs` |
| transaction | 事务管理、保存点支持 | `embedded/transaction.rs` |
| batch | 批量数据插入 | `embedded/batch.rs` |
| statement | 预编译语句、参数绑定 | `embedded/statement/` |
| result | 查询结果封装 | `embedded/result.rs` |
| statistics | 会话统计信息 | `embedded/statistics.rs` |
| config | 数据库配置 | `embedded/config.rs` |
| busy_handler | 并发控制 | `embedded/busy_handler.rs` |

### C API 功能清单

| 模块 | 功能描述 | 对应文件 |
|------|---------|---------|
| database | C语言数据库接口 | `c_api/database.rs` |
| session | C语言会话接口 | `c_api/session.rs` |
| query | C语言查询接口 | `c_api/query.rs` |
| statement | C语言预编译语句接口 | `c_api/statement.rs` |
| transaction | C语言事务接口 | `c_api/transaction.rs` |
| batch | C语言批量操作接口 | `c_api/batch.rs` |
| result | C语言结果集接口 | `c_api/result.rs` |
| function | 自定义函数注册 | `c_api/function.rs` |
| value | 值类型转换 | `c_api/value.rs` |
| types | C类型定义 | `c_api/types.rs` |
| error | 错误处理 | `c_api/error.rs` |

### Server API 当前功能

| 模块 | 功能描述 | 对应文件 |
|------|---------|---------|
| graph_service | 核心服务 | `server/graph_service.rs` |
| http/server | HTTP服务器 | `server/http/server.rs` |
| http/router | 路由配置 | `server/http/router.rs` |
| http/handlers/query | 查询执行 | `server/http/handlers/query.rs` |
| http/handlers/transaction | 事务管理 | `server/http/handlers/transaction.rs` |
| http/handlers/session | 会话管理 | `server/http/handlers/session.rs` |
| http/handlers/schema | Schema管理 | `server/http/handlers/schema.rs` |
| http/handlers/auth | 认证 | `server/http/handlers/auth.rs` |
| http/handlers/health | 健康检查 | `server/http/handlers/health.rs` |
| auth | 认证器 | `server/auth/` |
| permission | 权限管理 | `server/permission/` |
| session | 会话管理 | `server/session/` |

## 缺失功能清单

### 高优先级 (P0)

1. **批量操作 API** - 大数据导入核心需求
2. **结构化结果返回** - 当前返回字符串，需改为JSON结构化数据
3. **预编译语句 API** - 提升性能，防止注入

### 中优先级 (P1)

4. **统计信息 API** - 监控和调优
5. **配置管理 API** - 运行时配置调整

### 低优先级 (P2)

6. **自定义函数 API** - 高级功能扩展
7. **结果集流式处理** - 大数据集处理

---

## 第一阶段：核心功能完善 (P0)

### 目标
实现批量操作、结构化结果返回、预编译语句三大核心功能。

### 1.1 批量操作 API

#### 新增文件
- `src/api/server/http/handlers/batch.rs`
- `src/api/server/batch/manager.rs`
- `src/api/server/batch/mod.rs`

#### HTTP API 设计

```rust
// 创建批量任务
POST /v1/batch
Request: {
    "space_id": 1,
    "batch_type": "vertex" | "edge" | "mixed",
    "batch_size": 1000
}
Response: {
    "batch_id": "uuid",
    "status": "created",
    "created_at": "2024-01-01T00:00:00Z"
}

// 添加数据到批量任务
POST /v1/batch/:batch_id/items
Request: {
    "items": [
        {"type": "vertex", "data": {...}},
        {"type": "edge", "data": {...}}
    ]
}
Response: {
    "accepted": 100,
    "buffered": 50
}

// 执行批量插入
POST /v1/batch/:batch_id/execute
Response: {
    "batch_id": "uuid",
    "status": "completed",
    "result": {
        "vertices_inserted": 1000,
        "edges_inserted": 500,
        "errors": []
    }
}

// 查询批量任务状态
GET /v1/batch/:batch_id/status
Response: {
    "batch_id": "uuid",
    "status": "running" | "completed" | "failed",
    "progress": {
        "total": 1000,
        "processed": 500,
        "failed": 0
    }
}

// 取消批量任务
DELETE /v1/batch/:batch_id
```

#### 实现参考
参考 `embedded/batch.rs` 中的 `BatchInserter` 结构体，创建 `BatchManager` 管理批量任务。

### 1.2 结构化结果返回

#### 修改文件
- `src/api/server/http/handlers/query.rs`
- `src/api/server/http/handlers/mod.rs`

#### 数据结构

```rust
// 新的 QueryResponse
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub success: bool,
    pub data: Option<QueryData>,
    pub error: Option<QueryError>,
    pub metadata: QueryMetadata,
}

#[derive(Debug, Serialize)]
pub struct QueryData {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub row_count: usize,
}

#[derive(Debug, Serialize)]
pub struct QueryMetadata {
    pub execution_time_ms: u64,
    pub rows_scanned: u64,
    pub rows_returned: usize,
    pub space_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct QueryError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}
```

#### 实现步骤
1. 修改 `query::execute` handler，使用 `QueryResult` 替代字符串
2. 实现 `Value` 到 `serde_json::Value` 的转换
3. 添加列名到值的映射

### 1.3 预编译语句 API

#### 新增文件
- `src/api/server/http/handlers/statement.rs`
- `src/api/server/statement/manager.rs`
- `src/api/server/statement/mod.rs`

#### HTTP API 设计

```rust
// 创建预编译语句
POST /v1/statements
Request: {
    "query": "MATCH (n:User {id: $id}) RETURN n",
    "space_id": 1
}
Response: {
    "statement_id": "uuid",
    "parameters": ["id"],
    "created_at": "2024-01-01T00:00:00Z"
}

// 执行预编译语句
POST /v1/statements/:statement_id/execute
Request: {
    "parameters": {
        "id": 123
    }
}
Response: QueryResponse

// 批量执行预编译语句
POST /v1/statements/:statement_id/batch
Request: {
    "batch_parameters": [
        {"id": 1},
        {"id": 2},
        {"id": 3}
    ]
}
Response: {
    "results": [QueryResponse, ...],
    "summary": {
        "total": 3,
        "success": 3,
        "failed": 0
    }
}

// 获取语句信息
GET /v1/statements/:statement_id
Response: {
    "statement_id": "uuid",
    "query": "MATCH (n:User {id: $id}) RETURN n",
    "parameters": ["id"],
    "execution_count": 10,
    "avg_execution_time_ms": 5.5,
    "created_at": "2024-01-01T00:00:00Z",
    "last_used_at": "2024-01-01T01:00:00Z"
}

// 释放预编译语句
DELETE /v1/statements/:statement_id
```

#### 实现参考
参考 `embedded/statement/statement.rs` 中的 `PreparedStatement` 结构体。

---

## 第二阶段：监控与配置 (P1)

### 目标
实现统计信息查询和配置管理功能。

### 2.1 统计信息 API

#### 新增文件
- `src/api/server/http/handlers/statistics.rs`

#### HTTP API 设计

```rust
// 获取会话统计
GET /v1/statistics/sessions/:session_id
Response: {
    "session_id": 123,
    "username": "admin",
    "statistics": {
        "total_queries": 100,
        "total_changes": 50,
        "last_insert_vertex_id": 1001,
        "last_insert_edge_id": 5001,
        "avg_execution_time_ms": 10.5
    },
    "created_at": "2024-01-01T00:00:00Z",
    "last_activity_at": "2024-01-01T01:00:00Z"
}

// 获取查询统计
GET /v1/statistics/queries?from=2024-01-01&to=2024-01-02
Response: {
    "total_queries": 1000,
    "slow_queries": [
        {
            "query": "MATCH (n) RETURN n",
            "execution_time_ms": 5000,
            "executed_at": "2024-01-01T00:00:00Z"
        }
    ],
    "query_types": {
        "MATCH": 500,
        "CREATE": 200,
        "UPDATE": 100,
        "DELETE": 50
    }
}

// 获取数据库统计
GET /v1/statistics/database
Response: {
    "spaces": {
        "count": 5,
        "total_vertices": 100000,
        "total_edges": 50000
    },
    "storage": {
        "total_size_bytes": 1073741824,
        "index_size_bytes": 104857600,
        "data_size_bytes": 968884224
    },
    "performance": {
        "queries_per_second": 100,
        "avg_latency_ms": 5.0,
        "cache_hit_rate": 0.95
    }
}

// 获取系统资源使用
GET /v1/statistics/system
Response: {
    "cpu_usage_percent": 25.5,
    "memory_usage": {
        "used_bytes": 1073741824,
        "total_bytes": 8589934592
    },
    "connections": {
        "active": 10,
        "total": 100,
        "max": 1000
    }
}
```

### 2.2 配置管理 API

#### 新增文件
- `src/api/server/http/handlers/config.rs`

#### HTTP API 设计

```rust
// 获取当前配置
GET /v1/config
Response: {
    "database": {
        "host": "0.0.0.0",
        "port": 8080,
        "max_connections": 1000,
        "default_timeout": 30
    },
    "storage": {
        "cache_size_mb": 128,
        "enable_wal": true,
        "sync_mode": "Normal"
    },
    "query": {
        "max_execution_time_ms": 30000,
        "enable_cache": true,
        "cache_size": 1000
    }
}

// 更新配置（热更新）
PUT /v1/config
Request: {
    "query": {
        "max_execution_time_ms": 60000
    }
}
Response: {
    "updated": ["query.max_execution_time_ms"],
    "requires_restart": []
}

// 获取配置项
GET /v1/config/:section/:key
// 更新配置项
PUT /v1/config/:section/:key
Request: {
    "value": 60000
}

// 重置配置为默认值
DELETE /v1/config/:section/:key
```

---

## 第三阶段：高级功能 (P2)

### 目标
实现自定义函数和流式结果处理。

### 3.1 自定义函数 API

#### 新增文件
- `src/api/server/http/handlers/function.rs`

#### HTTP API 设计

```rust
// 注册自定义函数
POST /v1/functions
Request: {
    "name": "custom_add",
    "type": "scalar",
    "parameters": ["x", "y"],
    "return_type": "int",
    "implementation": {
        "language": "wasm",
        "code": "..."
    }
}
Response: {
    "function_id": "uuid",
    "name": "custom_add",
    "status": "registered"
}

// 列出所有函数
GET /v1/functions
Response: {
    "functions": [
        {
            "name": "custom_add",
            "type": "scalar",
            "parameters": ["x", "y"],
            "registered_at": "2024-01-01T00:00:00Z"
        }
    ]
}

// 获取函数详情
GET /v1/functions/:name

// 注销函数
DELETE /v1/functions/:name
```

### 3.2 流式结果 API

#### HTTP API 设计

```rust
// 执行查询并流式返回结果
POST /v1/query/stream
Request: {
    "query": "MATCH (n) RETURN n",
    "session_id": 123,
    "batch_size": 100
}
Response: SSE (Server-Sent Events)

event: row
data: {"n": {"id": 1, "name": "Alice"}}

event: row
data: {"n": {"id": 2, "name": "Bob"}}

event: metadata
data: {"rows_returned": 2, "execution_time_ms": 100}

event: done
data: {}
```

---

## 路由更新计划

### 更新 `src/api/server/http/router.rs`

```rust
// 第一阶段新增路由
let phase1_routes = Router::new()
    // 批量操作
    .route("/batch", post(batch::create))
    .route("/batch/:id", get(batch::status).delete(batch::cancel))
    .route("/batch/:id/items", post(batch::add_items))
    .route("/batch/:id/execute", post(batch::execute))
    // 预编译语句
    .route("/statements", post(statement::create))
    .route("/statements/:id", get(statement::info).delete(statement::drop))
    .route("/statements/:id/execute", post(statement::execute))
    .route("/statements/:id/batch", post(statement::batch_execute));

// 第二阶段新增路由
let phase2_routes = Router::new()
    // 统计信息
    .route("/statistics/sessions/:id", get(statistics::session))
    .route("/statistics/queries", get(statistics::queries))
    .route("/statistics/database", get(statistics::database))
    .route("/statistics/system", get(statistics::system))
    // 配置管理
    .route("/config", get(config::get).put(config::update))
    .route("/config/:section/:key", get(config::get_key).put(config::update_key).delete(config::reset));

// 第三阶段新增路由
let phase3_routes = Router::new()
    // 自定义函数
    .route("/functions", get(function::list).post(function::register))
    .route("/functions/:name", get(function::info).delete(function::unregister))
    // 流式查询
    .route("/query/stream", post(query::execute_stream));
```

---

## 实施时间表

| 阶段 | 功能 | 预计工时 | 依赖 |
|------|------|---------|------|
| **第一阶段** | | **2周** | |
| 1.1 | 批量操作 API | 3天 | 无 |
| 1.2 | 结构化结果返回 | 2天 | 无 |
| 1.3 | 预编译语句 API | 4天 | 1.2 |
| 1.4 | 集成测试 | 1天 | 1.1-1.3 |
| **第二阶段** | | **1周** | |
| 2.1 | 统计信息 API | 2天 | 1.2 |
| 2.2 | 配置管理 API | 2天 | 无 |
| 2.3 | 集成测试 | 1天 | 2.1-2.2 |
| **第三阶段** | | **1周** | |
| 3.1 | 自定义函数 API | 3天 | 无 |
| 3.2 | 流式结果 API | 2天 | 1.2 |
| 3.3 | 集成测试 | 1天 | 3.1-3.2 |

**总计：4周**

---

## 风险与注意事项

### 技术风险

1. **内存管理**：批量操作和流式结果需要谨慎管理内存，避免OOM
2. **并发安全**：预编译语句管理器需要处理并发访问
3. **向后兼容**：结构化结果返回是破坏性变更，需要版本控制

### 缓解措施

1. 批量操作设置最大批次大小和内存限制
2. 使用 `Arc<Mutex<>>` 或 `RwLock` 保护共享状态
3. 通过 `Accept` header 或 API 版本号支持新旧格式

---

## 附录：参考代码

### 批量操作参考

```rust
// src/api/server/batch/manager.rs
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use uuid::Uuid;

pub struct BatchManager {
    tasks: Arc<Mutex<HashMap<String, BatchTask>>>,
}

pub struct BatchTask {
    pub id: String,
    pub space_id: u64,
    pub status: BatchStatus,
    pub items: Vec<BatchItem>,
    pub result: Option<BatchResult>,
}

pub enum BatchStatus {
    Created,
    Running,
    Completed,
    Failed,
}
```

### 预编译语句参考

```rust
// src/api/server/statement/manager.rs
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

pub struct StatementManager<S: StorageClient> {
    statements: Arc<RwLock<HashMap<String, PreparedStatementHolder<S>>>>,
}

pub struct PreparedStatementHolder<S: StorageClient> {
    pub statement: PreparedStatement<S>,
    pub created_at: Instant,
    pub last_used_at: AtomicInstant,
    pub execution_count: AtomicU64,
}
```

---

*文档版本: 1.0*
*创建日期: 2026-03-10*
*作者: GraphDB Team*
