# GraphDB HTTP 服务异步化改造方案

## 1. 现状分析

### 1.1 当前架构
- **核心服务层**: `GraphService`、`QueryApi`、`TransactionApi` 等均为同步实现
- **HTTP 层**: 仅包含基础结构定义，未实现实际的 HTTP 请求处理
- **存储层**: 使用 `parking_lot::Mutex` 进行同步访问
- **已有异步**: `SessionManager` 的后台清理任务使用 `tokio::spawn`

### 1.2 同步 vs 异步适用性分析

| 层级 | 当前实现 | 建议 | 理由 |
|------|----------|------|------|
| 核心服务层 | 同步 | **保持同步** | CPU 密集型，异步无益 |
| 存储访问层 | 同步 | **保持同步** | 内存/磁盘操作，使用 spawn_blocking 包装 |
| HTTP 处理层 | 未实现 | **异步实现** | 网络 IO，需要高并发支持 |
| 后台任务 | 异步 | **保持异步** | 定时任务，已在运行 |

## 2. 技术选型

### 2.1 Web 框架选择: Axum

**选择理由**:
- 与 Tokio 生态深度集成，项目已使用 Tokio
- 基于 Tower 和 Hyper，性能优异
- 类型安全的路由系统，无宏 API
- 与现有代码风格一致（模块化、显式依赖）

**对比其他框架**:

| 框架 | 优点 | 缺点 | 适用性 |
|------|------|------|--------|
| Axum | Tokio 原生，类型安全，模块化 | 学习曲线稍陡 | **推荐** |
| Actix-web | 性能极高，生态成熟 | 运行时较重 | 可选 |
| Warp | 函数式风格，组合性强 | 编译错误难读 | 可选 |
| Rocket | 开发体验好 | 需要 nightly，较重 | 不推荐 |

### 2.2 依赖库版本

```toml
[dependencies]
# Web 框架
axum = { version = "0.8", features = ["macros"] }
tower = { version = "0.5", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "trace", "timeout", "compression"] }

# 序列化（项目已有，确认版本）
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 异步运行时（项目已有）
tokio = { version = "1.48", features = ["full"] }

# 可选：API 文档
utoipa = { version = "5.0", features = ["axum_extras"] }
utoipa-swagger-ui = { version = "8.0", features = ["axum"] }
```

## 3. 架构设计

### 3.1 目标架构

```
┌─────────────────────────────────────────────────────────────┐
│                    HTTP 服务层 (异步)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Router    │  │  Handlers   │  │    Middleware       │  │
│  │   (axum)    │  │  (async)    │  │  (CORS/Trace/Auth)  │  │
│  └──────┬──────┘  └──────┬──────┘  └─────────────────────┘  │
└─────────┼────────────────┼──────────────────────────────────┘
          │                │
          │    ┌───────────┘
          │    │
          ▼    ▼
┌─────────────────────────────────────────────────────────────┐
│              同步操作包装层 (spawn_blocking)                  │
│         将同步核心服务包装为异步接口供 HTTP 层使用            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    核心服务层 (保持同步)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ GraphService│  │   QueryApi  │  │  TransactionApi     │  │
│  │   (同步)    │  │   (同步)    │  │     (同步)          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 目录结构

```
src/api/server/http/
├── mod.rs              # 模块导出，启动入口
├── server.rs           # HttpServer 结构体（现有，保持同步）
├── router.rs           # 路由定义（新增，异步）
├── handlers/           # 请求处理器（新增，异步）
│   ├── mod.rs
│   ├── query.rs        # 查询相关处理
│   ├── auth.rs         # 认证相关处理
│   ├── session.rs      # 会话相关处理
│   └── health.rs       # 健康检查
├── middleware/         # 中间件（新增，异步）
│   ├── mod.rs
│   ├── auth.rs         # 认证中间件
│   ├── logging.rs      # 日志中间件
│   └── error.rs        # 错误处理
├── state.rs            # 应用状态共享（新增）
└── error.rs            # HTTP 错误类型（新增）
```

## 4. 分阶段实施方案

### 阶段 1: 基础框架搭建（1-2 天）

**目标**: 引入 Axum 依赖，实现基础 HTTP 服务器

#### 4.1.1 添加依赖

```toml
# Cargo.toml
[dependencies]
axum = { version = "0.8", features = ["macros"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "timeout"] }
```

#### 4.1.2 创建基础文件

**src/api/server/http/state.rs**:
```rust
use crate::api::server::HttpServer;
use crate::storage::StorageClient;
use std::sync::Arc;

/// 应用状态，在处理器间共享
#[derive(Clone)]
pub struct AppState<S: StorageClient + Clone + 'static> {
    pub server: Arc<HttpServer<S>>,
}

impl<S: StorageClient + Clone + 'static> AppState<S> {
    pub fn new(server: Arc<HttpServer<S>>) -> Self {
        Self { server }
    }
}
```

**src/api/server/http/error.rs**:
```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// HTTP 错误类型
#[derive(Debug)]
pub enum HttpError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    InternalError(String),
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            HttpError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            HttpError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            HttpError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            HttpError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
```

**src/api/server/http/router.rs**:
```rust
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use std::time::Duration;

use super::{
    state::AppState,
    handlers::{health, query, auth, session},
};

pub fn create_router<S: StorageClient + Clone + Send + Sync + 'static>(
    state: AppState<S>,
) -> Router {
    Router::new()
        // 健康检查
        .route("/health", get(health::check))
        // 认证
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout))
        // 会话
        .route("/sessions", post(session::create))
        .route("/sessions/:id", get(session::get).delete(session::delete))
        // 查询
        .route("/query", post(query::execute))
        .route("/query/validate", post(query::validate))
        // 中间件
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .with_state(state)
}
```

#### 4.1.3 健康检查处理器

**src/api/server/http/handlers/health.rs**:
```rust
use axum::{
    http::StatusCode,
    response::Json,
};
use serde_json::json;

/// 健康检查端点
pub async fn check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "graphdb",
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}
```

#### 4.1.4 修改启动入口

**src/api/mod.rs**:
```rust
// 添加新的启动函数
pub async fn start_http_server<S: StorageClient + Clone + Send + Sync + 'static>(
    server: Arc<HttpServer<S>>,
    config: &Config,
) -> Result<()> {
    use axum::serve;
    use tokio::net::TcpListener;
    
    let state = crate::api::server::http::state::AppState::new(server);
    let app = crate::api::server::http::router::create_router(state);
    
    let addr = format!("{}:{}", config.host(), config.port());
    let listener = TcpListener::bind(&addr).await?;
    
    info!("HTTP server listening on {}", addr);
    
    serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    Ok(())
}

// 异步关闭信号
async fn shutdown_signal() {
    use tokio::signal;
    
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    
    info!("shutdown signal received, starting graceful shutdown");
}
```

### 阶段 2: 核心处理器实现（3-5 天）

**目标**: 实现查询、认证、会话等核心功能的异步处理器

#### 4.2.1 查询处理器（同步包装示例）

**src/api/server/http/handlers/query.rs**:
```rust
use axum::{
    extract::{State, Json},
    response::Json as JsonResponse,
};
use std::sync::Arc;
use tokio::task;

use crate::api::server::http::{
    state::AppState,
    error::HttpError,
};
use crate::storage::StorageClient;

/// 查询请求
#[derive(Debug, serde::Deserialize)]
pub struct QueryRequest {
    pub query: String,
    pub session_id: i64,
    #[serde(default)]
    pub parameters: std::collections::HashMap<String, String>,
}

/// 查询响应
#[derive(Debug, serde::Serialize)]
pub struct QueryResponse {
    pub result: String,
    pub execution_time_ms: u64,
}

/// 执行查询 - 使用 spawn_blocking 包装同步操作
pub async fn execute<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<QueryRequest>,
) -> Result<JsonResponse<QueryResponse>, HttpError> {
    let server = state.server.clone();
    
    // 在阻塞线程池中执行同步查询
    let result = task::spawn_blocking(move || {
        let session_manager = server.get_session_manager();
        let session = session_manager
            .find_session(request.session_id)
            .ok_or_else(|| HttpError::Unauthorized("无效会话".to_string()))?;
        
        // 获取 GraphService 并执行查询
        // 注意：这里需要根据实际架构调整
        // 当前 HttpServer 没有直接暴露 execute 方法
        // 可能需要通过 GraphService 来执行
        
        Ok::<_, HttpError>(("查询结果".to_string(), 100u64))
    })
    .await
    .map_err(|e| HttpError::InternalError(format!("任务执行失败: {}", e)))?;
    
    let (result_str, exec_time) = result?;
    
    Ok(JsonResponse(QueryResponse {
        result: result_str,
        execution_time_ms: exec_time,
    }))
}

/// 验证查询语法
pub async fn validate<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<QueryRequest>,
) -> Result<JsonResponse<serde_json::Value>, HttpError> {
    // 语法验证通常很快，可以直接执行
    // 如果验证涉及复杂解析，也应使用 spawn_blocking
    
    Ok(JsonResponse(serde_json::json!({
        "valid": true,
        "message": "语法正确",
    })))
}
```

#### 4.2.2 认证处理器

**src/api/server/http/handlers/auth.rs**:
```rust
use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::task;

use crate::api::server::http::{state::AppState, error::HttpError};
use crate::storage::StorageClient;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub session_id: i64,
    pub username: String,
    pub expires_at: u64,
}

/// 登录 - 使用 spawn_blocking 因为 bcrypt 是计算密集型
pub async fn login<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, HttpError> {
    let server = state.server.clone();
    
    let session = task::spawn_blocking(move || {
        // 这里需要访问 GraphService 的 authenticate 方法
        // 当前架构可能需要调整
        
        // 临时返回模拟数据
        Ok::<_, HttpError>(LoginResponse {
            session_id: 12345,
            username: request.username,
            expires_at: 0,
        })
    })
    .await
    .map_err(|e| HttpError::InternalError(e.to_string()))?;
    
    Ok(Json(session?))
}

/// 登出 - 同步操作，直接执行
pub async fn logout<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
) -> Result<StatusCode, HttpError> {
    // 从请求头获取 session_id 并移除会话
    Ok(StatusCode::NO_CONTENT)
}
```

### 阶段 3: 架构适配（2-3 天）

**目标**: 调整现有架构，使 HttpServer 能够访问 GraphService

#### 4.3.1 问题分析

当前 `HttpServer` 和 `GraphService` 是独立的结构体：
- `HttpServer` 包含 `QueryApi`、`TransactionApi` 等
- `GraphService` 包含 `session_manager`、`pipeline_manager` 等
- `GraphService::execute()` 是主要的查询执行入口

#### 4.3.2 解决方案

**方案 A: HttpServer 包含 GraphService**（推荐）

```rust
// src/api/server/http/server.rs
pub struct HttpServer<S: StorageClient + Clone + 'static> {
    graph_service: Arc<GraphService<S>>,  // 新增
    query_api: QueryApi<S>,
    txn_api: TransactionApi,
    // ... 其他字段
}

impl<S: StorageClient + Clone + 'static> HttpServer<S> {
    pub fn new(
        graph_service: Arc<GraphService<S>>,  // 修改为接收 GraphService
        config: &Config,
    ) -> Self {
        // ...
    }
    
    /// 获取 GraphService
    pub fn get_graph_service(&self) -> &GraphService<S> {
        &self.graph_service
    }
}
```

**方案 B: 统一使用 GraphService**

如果 `QueryApi` 等功能可以通过 `GraphService` 访问，可以简化架构：

```rust
pub struct HttpServer<S: StorageClient + Clone + 'static> {
    graph_service: Arc<GraphService<S>>,
    // 移除独立的 API 实例
}
```

### 阶段 4: 中间件与优化（2-3 天）

**目标**: 添加认证、日志、错误处理等中间件

#### 4.4.1 认证中间件

**src/api/server/http/middleware/auth.rs**:
```rust
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
    http::StatusCode,
};

use crate::api::server::http::state::AppState;
use crate::storage::StorageClient;

/// 从请求头提取并验证 session
pub async fn auth_middleware<S: StorageClient + Clone + Send + Sync + 'static>(
    State(state): State<AppState<S>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 从 header 提取 session_id
    let session_id = request
        .headers()
        .get("X-Session-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // 验证会话
    let valid = state
        .server
        .get_session_manager()
        .find_session(session_id)
        .is_some();
    
    if !valid {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // 将 session_id 添加到请求扩展
    request.extensions_mut().insert(session_id);
    
    Ok(next.run(request).await)
}
```

#### 4.4.2 错误处理中间件

**src/api/server/http/middleware/error.rs**:
```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use log::error;

pub async fn error_handling_middleware(
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().to_string();
    
    let response = next.run(request).await;
    
    let status = response.status();
    if status.is_server_error() {
        error!("{} {} returned {}", method, path, status);
    }
    
    response
}
```

### 阶段 5: 测试与文档（2-3 天）

**目标**: 编写测试和 API 文档

#### 4.5.1 集成测试

**tests/http_api_test.rs**:
```rust
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_health_check() {
    let app = create_test_app();
    
    let response = app
        .oneshot(Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_query_endpoint() {
    let app = create_test_app();
    
    let response = app
        .oneshot(Request::builder()
            .uri("/query")
            .method("POST")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"query": "MATCH (n) RETURN n", "session_id": 1}"#))
            .unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}
```

#### 4.5.2 API 文档（可选）

使用 utoipa 生成 OpenAPI 文档：

```rust
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct QueryRequest {
    /// 查询语句
    #[schema(example = "MATCH (n) RETURN n LIMIT 10")]
    pub query: String,
    /// 会话 ID
    pub session_id: i64,
}

#[utoipa::path(
    post,
    path = "/query",
    request_body = QueryRequest,
    responses(
        (status = 200, description = "查询成功", body = QueryResponse),
        (status = 401, description = "未授权"),
        (status = 500, description = "服务器内部错误"),
    )
)]
pub async fn execute(...) { ... }
```

## 5. 风险评估与回滚方案

### 5.1 风险点

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 性能下降 | 高 | 使用 spawn_blocking 避免阻塞运行时 |
| 内存增加 | 中 | 控制并发数，使用连接池 |
| 兼容性问题 | 中 | 保持核心 API 不变，仅添加 HTTP 层 |
| 学习成本 | 低 | 团队已有 Tokio 经验 |

### 5.2 回滚方案

如果异步化出现问题，可以：
1. 保留同步核心服务层
2. 移除 Axum 依赖
3. 恢复原有的同步服务启动方式

## 6. 时间估算

| 阶段 | 预计时间 | 依赖 |
|------|----------|------|
| 阶段 1: 基础框架 | 1-2 天 | 无 |
| 阶段 2: 核心处理器 | 3-5 天 | 阶段 1 |
| 阶段 3: 架构适配 | 2-3 天 | 阶段 2 |
| 阶段 4: 中间件 | 2-3 天 | 阶段 3 |
| 阶段 5: 测试文档 | 2-3 天 | 阶段 4 |
| **总计** | **10-16 天** | - |

## 7. 总结

### 关键决策
1. **保持核心层同步**: CPU 密集型操作不需要异步
2. **HTTP 层异步化**: 使用 Axum + spawn_blocking 模式
3. **渐进式改造**: 分阶段实施，降低风险

### 预期收益
- 支持高并发 HTTP 连接
- 更好的资源利用率
- 现代 Rust 异步生态
- 易于扩展的模块化架构

### 下一步行动
1. 评审本方案
2. 创建功能分支
3. 开始阶段 1 实施
