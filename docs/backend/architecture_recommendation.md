# GraphDB 后端架构建议

**文档版本**: v1.0  
**创建日期**: 2026-03-29  
**分析目标**: 确定后端实现方式（独立 crate vs src/api/server 扩展）

---

## 1. 方案对比

### 方案 A: 在 src/api/server 中扩展

在现有的 `src/api/server` 模块中直接添加新功能。

#### 目录结构

```
src/api/
├── core/                       # 已有: 核心 API 类型
├── embedded/                   # 已有: 嵌入式 API
├── server/                     # 扩展: HTTP Server API
│   ├── auth/                   # 已有: 认证
│   ├── batch/                  # 已有: 批量操作
│   ├── client/                 # 已有: 客户端管理
│   ├── http/
│   │   ├── handlers/
│   │   │   ├── auth.rs         # 已有
│   │   │   ├── batch.rs        # 已有
│   │   │   ├── config.rs       # 已有
│   │   │   ├── health.rs       # 已有
│   │   │   ├── mod.rs          # 已有
│   │   │   ├── query.rs        # 已有
│   │   │   ├── query_types.rs  # 已有
│   │   │   ├── schema.rs       # 已有 (需扩展)
│   │   │   ├── session.rs      # 已有
│   │   │   ├── statistics.rs   # 已有
│   │   │   ├── transaction.rs  # 已有
│   │   │   ├── metadata.rs     # 新增: 查询历史/收藏
│   │   │   ├── graph_data.rs   # 新增: 图数据查询
│   │   │   └── data_browser.rs # 新增: 数据浏览
│   │   ├── middleware/         # 已有
│   │   ├── mod.rs
│   │   ├── router.rs           # 修改: 添加新路由
│   │   ├── server.rs
│   │   └── state.rs
│   ├── metadata/               # 新增: 元数据管理模块
│   │   ├── mod.rs
│   │   ├── history.rs          # 查询历史
│   │   ├── favorite.rs         # 查询收藏
│   │   ├── storage.rs          # 存储抽象
│   │   └── sqlite.rs           # SQLite 实现
│   ├── permission/             # 已有
│   ├── session/                # 已有
│   ├── graph_service.rs        # 已有
│   └── mod.rs
└── mod.rs
```

#### 优点

1. **架构统一**: 与现有代码保持一致，维护简单
2. **共享状态**: 可以直接访问 `AppState` 和 `GraphService`
3. **路由集中**: 所有 HTTP 端点在一个地方管理
4. **依赖简单**: 不需要处理 crate 间的依赖关系

#### 缺点

1. **代码膨胀**: server 模块会变得越来越大
2. **编译时间**: 修改任何一个部分都需要重新编译整个 server 模块
3. **职责混杂**: 元数据管理（前端专属）与核心服务混在一起

---

### 方案 B: 独立 crate (graphdb-web)

创建一个新的 crate `graphdb-web` 作为独立的后端服务。

#### 目录结构

```
graphDB/
├── Cargo.toml                  # workspace 配置
├── src/                        # 核心库
│   └── ...
├── graphdb-web/                # 新增: Web 后端 crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs             # 服务入口
│       ├── lib.rs
│       ├── config.rs           # Web 服务配置
│       ├── routes/             # 路由模块
│       │   ├── mod.rs
│       │   ├── auth.rs
│       │   ├── query.rs
│       │   ├── schema.rs
│       │   ├── metadata.rs     # 查询历史/收藏
│       │   └── graph.rs        # 图数据
│       ├── handlers/           # 请求处理器
│       │   ├── mod.rs
│       │   ├── query.rs
│       │   ├── schema.rs
│       │   ├── metadata.rs
│       │   └── graph.rs
│       ├── services/           # 业务逻辑层
│       │   ├── mod.rs
│       │   ├── query_service.rs
│       │   ├── schema_service.rs
│       │   ├── metadata_service.rs
│       │   └── graph_service.rs
│       ├── models/             # 数据模型
│       │   ├── mod.rs
│       │   ├── history.rs
│       │   ├── favorite.rs
│       │   └── schema.rs
│       ├── db/                 # 数据库访问
│       │   ├── mod.rs
│       │   ├── connection.rs
│       │   └── migrations/
│       ├── middleware/         # HTTP 中间件
│       │   ├── mod.rs
│       │   ├── auth.rs
│       │   └── logging.rs
│       └── error.rs            # 错误定义
└── docs/
```

#### Workspace 配置

```toml
# graphDB/Cargo.toml
[workspace]
members = [
    ".",           # 核心库
    "graphdb-web", # Web 后端
]

[workspace.dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
# ... 其他共享依赖
```

#### 优点

1. **职责分离**: 核心库与 Web 服务完全分离
2. **独立部署**: Web 服务可以独立部署和扩展
3. **编译隔离**: 修改 Web 代码不需要重新编译核心库
4. **技术灵活**: Web 服务可以使用不同的技术栈
5. **清晰边界**: 强制通过 API 与核心库交互

#### 缺点

1. **复杂性增加**: 需要管理 crate 间依赖
2. **共享状态**: 需要通过 Arc<Mutex<>> 等方式共享 GraphService
3. **重复代码**: 部分类型定义可能需要复制
4. **构建复杂度**: 需要配置 workspace 和跨 crate 依赖

---

### 方案 C: 混合方案 (推荐)

在 `src/api` 下创建 `web` 子模块，既保持代码组织清晰，又避免独立的 crate 复杂性。

#### 目录结构

```
src/api/
├── core/                       # 已有: 核心 API 类型
├── embedded/                   # 已有: 嵌入式 API
├── server/                     # 已有: HTTP Server (保持精简)
│   ├── auth/
│   ├── http/
│   ├── session/
│   └── ...
├── web/                        # 新增: Web 管理界面后端
│   ├── mod.rs
│   ├── config.rs               # Web 模块配置
│   ├── server.rs               # Web 服务器启动
│   ├── router.rs               # Web 专属路由
│   ├── state.rs                # Web 状态管理
│   ├── handlers/               # Web 专属处理器
│   │   ├── mod.rs
│   │   ├── metadata.rs         # 查询历史/收藏
│   │   ├── schema_ext.rs       # Schema 扩展
│   │   ├── graph_data.rs       # 图数据查询
│   │   └── data_browser.rs     # 数据浏览
│   ├── services/               # Web 业务逻辑
│   │   ├── mod.rs
│   │   ├── metadata_service.rs
│   │   ├── schema_ext_service.rs
│   │   └── graph_data_service.rs
│   ├── models/                 # Web 数据模型
│   │   ├── mod.rs
│   │   ├── history.rs
│   │   ├── favorite.rs
│   │   └── schema_snapshot.rs
│   └── storage/                # 元数据存储
│       ├── mod.rs
│       ├── sqlite.rs
│       └── migrations/
└── mod.rs
```

#### 启动方式

```rust
// src/main.rs 或 src/bin/web.rs
use graphdb::api::web;

#[tokio::main]
async fn main() {
    // 启动 Web 管理服务
    web::start_server(config).await;
}
```

或者集成到现有 server：

```rust
// src/api/server/http/router.rs
use crate::api::web::handlers;

pub fn create_router<S: StorageClient>(state: AppState<S>) -> Router {
    let api_routes = Router::new()
        // 核心 API 路由
        .route("/query", post(query::execute))
        // ... 其他核心路由
        ;
    
    let web_routes = web::router::create_web_router(state.clone());
    
    Router::new()
        .nest("/v1", api_routes)
        .nest("/web", web_routes)  // Web 管理界面 API
}
```

#### 优点

1. **代码组织清晰**: Web 专属功能独立组织
2. **渐进式演进**: 可以先在内部实现，未来再拆分为独立 crate
3. **共享核心**: 可以直接使用 `GraphService` 等核心组件
4. **编译效率**: 不需要处理跨 crate 依赖
5. **灵活部署**: 可以选择只启动核心 API 或同时启动 Web 服务

#### 缺点

1. **概念区分**: 需要明确区分 `server` 和 `web` 的职责
2. **命名注意**: 避免与现有 `server` 模块混淆

---

## 2. 详细对比

| 维度 | 方案 A: 扩展 | 方案 B: 独立 crate | 方案 C: 混合 (推荐) |
|------|-------------|-------------------|-------------------|
| **代码组织** | ⭐⭐ 逐渐混乱 | ⭐⭐⭐⭐⭐ 清晰 | ⭐⭐⭐⭐ 清晰 |
| **编译效率** | ⭐⭐ 重新编译多 | ⭐⭐⭐⭐ 增量编译 | ⭐⭐⭐ 中等 |
| **部署灵活** | ⭐⭐ 单一部署 | ⭐⭐⭐⭐⭐ 独立部署 | ⭐⭐⭐⭐ 可选部署 |
| **开发复杂度** | ⭐⭐⭐⭐ 简单 | ⭐⭐ 复杂 | ⭐⭐⭐⭐ 简单 |
| **维护成本** | ⭐⭐ 逐渐增高 | ⭐⭐⭐⭐ 稳定 | ⭐⭐⭐⭐ 稳定 |
| **扩展性** | ⭐⭐ 受限 | ⭐⭐⭐⭐⭐ 高 | ⭐⭐⭐⭐ 高 |
| **与核心耦合** | ⭐⭐⭐ 紧耦合 | ⭐⭐⭐⭐⭐ 松耦合 | ⭐⭐⭐⭐ 中等耦合 |
| **适合团队规模** | 小团队 | 大团队 | 中小团队 |

---

## 3. 推荐方案: 方案 C (混合方案)

### 3.1 推荐理由

1. **符合当前项目阶段**: GraphDB 目前处于开发期，不需要过度设计
2. **平衡各方面因素**: 在代码组织和开发复杂度之间取得平衡
3. **未来可演进**: 当需要时，可以很容易地将 `web` 模块拆分为独立 crate
4. **与现有架构兼容**: 不需要大幅重构现有代码

### 3.2 实施步骤

```
Step 1: 创建目录结构
├── src/api/web/
│   ├── mod.rs
│   ├── config.rs
│   ├── router.rs
│   ├── state.rs
│   ├── handlers/
│   ├── services/
│   ├── models/
│   └── storage/

Step 2: 实现元数据存储模块
├── src/api/web/storage/
│   ├── mod.rs          # 存储抽象接口
│   └── sqlite.rs       # SQLite 实现

Step 3: 实现元数据服务
├── src/api/web/services/
│   ├── mod.rs
│   └── metadata_service.rs  # 历史/收藏业务逻辑

Step 4: 实现元数据 API 处理器
├── src/api/web/handlers/
│   ├── mod.rs
│   └── metadata.rs     # 历史/收藏 HTTP 处理器

Step 5: 扩展 Schema API
├── src/api/web/handlers/
│   └── schema_ext.rs   # Schema 列表/详情/索引

Step 6: 集成到主路由
// src/api/server/http/router.rs
.nest("/web", web::router::create_router())

Step 7: 实现数据浏览和图数据 API
├── src/api/web/handlers/
│   ├── data_browser.rs
│   └── graph_data.rs
```

### 3.3 代码示例

#### Web 模块入口

```rust
// src/api/web/mod.rs

pub mod config;
pub mod handlers;
pub mod models;
pub mod router;
pub mod services;
pub mod state;
pub mod storage;

use crate::api::server::state::AppState;
use crate::storage::StorageClient;

/// 启动 Web 管理服务
pub async fn start_server<S: StorageClient>(
    config: WebConfig,
    app_state: AppState<S>,
) -> Result<(), Box<dyn std::error::Error>> {
    let web_state = WebState::new(app_state, config.storage_path);
    let router = router::create_router(web_state);
    
    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    axum::serve(listener, router).await?;
    
    Ok(())
}
```

#### Web 状态管理

```rust
// src/api/web/state.rs

use crate::api::server::state::AppState;
use crate::storage::StorageClient;
use std::sync::Arc;

pub struct WebState<S: StorageClient> {
    /// 共享的核心应用状态
    pub app_state: AppState<S>,
    /// 元数据存储
    pub metadata_storage: Arc<dyn MetadataStorage>,
}

impl<S: StorageClient> WebState<S> {
    pub fn new(app_state: AppState<S>, storage_path: String) -> Self {
        let metadata_storage = Arc::new(SqliteStorage::new(&storage_path));
        
        Self {
            app_state,
            metadata_storage,
        }
    }
}
```

#### 元数据存储抽象

```rust
// src/api/web/storage/mod.rs

use async_trait::async_trait;
use crate::api::web::models::{HistoryItem, FavoriteItem};

#[async_trait]
pub trait MetadataStorage: Send + Sync {
    /// 添加查询历史
    async fn add_history(&self, item: HistoryItem) -> Result<String, StorageError>;
    
    /// 获取历史列表
    async fn get_history(
        &self,
        session_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<HistoryItem>, StorageError>;
    
    /// 添加收藏
    async fn add_favorite(&self, item: FavoriteItem) -> Result<String, StorageError>;
    
    /// 获取收藏列表
    async fn get_favorites(
        &self,
        session_id: &str,
    ) -> Result<Vec<FavoriteItem>, StorageError>;
    
    /// 删除收藏
    async fn delete_favorite(&self, id: &str) -> Result<(), StorageError>;
}
```

---

## 4. 与前端 PRD 的对应关系

### 4.1 功能映射

| 前端 PRD 阶段 | 后端实现位置 | 优先级 |
|--------------|-------------|--------|
| 阶段 1: 连接管理 | `src/api/server/` (已有) | P0 |
| 阶段 2: 查询历史 | `src/api/web/handlers/metadata.rs` | P0 |
| 阶段 2: 查询收藏 | `src/api/web/handlers/metadata.rs` | P0 |
| 阶段 2: 批量查询 | `src/api/web/handlers/query.rs` | P0 |
| 阶段 3: Space 管理 | `src/api/web/handlers/schema_ext.rs` | P0 |
| 阶段 4: Tag/Edge 管理 | `src/api/web/handlers/schema_ext.rs` | P0 |
| 阶段 5: 索引管理 | `src/api/web/handlers/schema_ext.rs` | P0 |
| 阶段 6: 图可视化 | `src/api/web/handlers/graph_data.rs` | P1 |
| 阶段 7: 数据浏览 | `src/api/web/handlers/data_browser.rs` | P1 |

### 4.2 API 路径规划

```
# 核心 API (已有)
/v1/health
/v1/auth/*
/v1/sessions/*
/v1/query
/v1/transactions/*
/v1/batch/*
/v1/schema/spaces/* (基础 CRUD)

# Web 管理 API (新增)
/web/v1/queries/history      # 查询历史
/web/v1/queries/favorites    # 查询收藏
/web/v1/schema/spaces/*/tags         # Tag 列表/详情
/web/v1/schema/spaces/*/edge-types   # Edge 列表/详情
/web/v1/schema/spaces/*/indexes      # 索引管理
/web/v1/data/*               # 数据浏览
/web/v1/graph/*              # 图数据查询
```

---

## 5. 总结

### 5.1 最终建议

**采用方案 C (混合方案)**，理由如下：

1. **适合当前阶段**: GraphDB 仍在开发期，不需要过度架构
2. **渐进式演进**: 可以在不破坏现有代码的情况下添加新功能
3. **未来可拆分**: 当项目成熟时，可以很容易地将 `web` 模块拆分为独立 crate
4. **团队友好**: 对开发团队的技术要求适中

### 5.2 实施优先级

```
Phase 1 (立即开始)
├── 1. 创建 src/api/web/ 目录结构
├── 2. 实现元数据存储模块 (SQLite)
├── 3. 实现查询历史/收藏 API
└── 4. 扩展 Schema API (列表/详情)

Phase 2 (Schema 管理)
├── 5. 实现 Tag/Edge 修改删除
├── 6. 实现索引管理 API
└── 7. 实现数据浏览 API

Phase 3 (增强功能)
├── 8. 实现图数据查询 API
└── 9. 优化和性能调优
```

### 5.3 关键决策点

| 决策 | 建议 | 理由 |
|------|------|------|
| 元数据存储 | SQLite | 轻量、可靠、与 Nebula Studio 一致 |
| API 前缀 | `/web/v1/` | 与核心 API 区分 |
| 认证方式 | 复用现有 JWT | 保持一致性 |
| 错误处理 | 统一错误格式 | 便于前端处理 |

---

## 6. 条件编译考虑

### 6.1 当前项目的条件编译结构

项目已在 `Cargo.toml` 中定义了以下 features：

```toml
[features]
default = ["redb", "embedded", "server", "c-api"]
redb = ["dep:redb"]
embedded = []
server = ["dep:axum", "dep:tower", "dep:tower-http", "dep:http"]
c-api = ["embedded"]
```

当前代码中已使用条件编译：
- `src/api/mod.rs`: `#[cfg(feature = "server")]` 控制 server 模块
- `src/api/mod.rs`: `#[cfg(feature = "embedded")]` 控制 embedded 模块
- `src/lib.rs`: `#[cfg(feature = "c-api")]` 控制 c_api 模块
- `src/main.rs`: `#[cfg(feature = "server")]` 控制服务端程序入口

### 6.2 Web 管理模块的条件编译策略

**推荐方案**: 将 Web 管理功能作为 `server` feature 的一部分，不单独创建 feature。

#### 理由

1. **功能依赖关系**
   - Web 管理功能完全依赖 server 功能（HTTP 服务、认证、会话管理）
   - 没有 server feature，Web 管理功能无法独立运行
   - 单独创建 `web` feature 会增加复杂性，没有实际收益

2. **编译效率**
   - Web 管理模块的依赖（SQLite、sqlx）已经包含在 server feature 的依赖树中
   - 单独 feature 不会显著减少编译时间

3. **使用场景**
   - 启用 server feature 的用户通常需要完整的 Web 管理功能
   - 不需要 Web 管理的场景（纯嵌入式使用）通常也不启用 server

#### 代码组织

```rust
// src/api/mod.rs
#[cfg(feature = "server")]
pub mod server;

// Web 管理模块作为 server 的子模块
// src/api/server/mod.rs
pub mod web;  // 不需要额外的条件编译
```

```rust
// src/api/server/web/mod.rs
// 这个模块只有在 server feature 启用时才会被编译
// 因为它位于 server 模块下

pub mod handlers;
pub mod services;
pub mod models;
pub mod storage;
```

### 6.3 可选的细化方案

如果未来需要更细粒度的控制，可以考虑：

```toml
[features]
default = ["redb", "embedded", "server", "c-api"]
server = ["dep:axum", "dep:tower", "dep:tower-http", "dep:http"]
web-console = ["server", "dep:sqlx"]  # 可选：单独控制 Web 控制台
```

**但当前不建议**，原因：
- 增加用户理解成本
- 维护两个相似但略有不同的 server 变体
- 测试矩阵复杂化

### 6.4 依赖管理

Web 管理模块需要的新依赖：

```toml
# Cargo.toml
[dependencies]
# 已有 server 依赖
axum = { version = "0.8", optional = true }
tower = { version = "0.5", optional = true }
tower-http = { version = "0.6", optional = true }

# 新增 Web 管理依赖
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"], optional = true }
```

注意：将 `sqlx` 添加到 `server` feature 中：

```toml
server = [
    "dep:axum", 
    "dep:tower", 
    "dep:tower-http", 
    "dep:http",
    "dep:sqlx"  # 新增
]
```

### 6.5 条件编译最佳实践

```rust
// src/api/server/http/router.rs
// 核心路由（始终编译当 server feature 启用）
pub fn create_core_router() -> Router {
    Router::new()
        .route("/health", get(health::check))
        .route("/query", post(query::execute))
        // ... 其他核心路由
}

// Web 管理路由（作为 server 的一部分）
pub fn create_web_router() -> Router {
    Router::new()
        .route("/web/v1/queries/history", get(metadata::get_history))
        .route("/web/v1/queries/favorites", get(metadata::get_favorites))
        // ... 其他 Web 管理路由
}

// 组合路由
pub fn create_router<S: StorageClient>(state: AppState<S>) -> Router {
    Router::new()
        .nest("/v1", create_core_router())
        .nest("/web", create_web_router())  // Web 管理路由
        .with_state(state)
}
```

### 6.6 总结

| 方案 | 条件编译策略 | 适用场景 |
|------|-------------|----------|
| **推荐** | Web 作为 server 子模块 | 当前阶段，简化设计 |
| 可选 | 单独 `web-console` feature | 未来需要细粒度控制时 |

**最终建议**：
- 将 Web 管理功能实现为 `src/api/server/web/` 子模块
- 不创建额外的 feature flag
- 依赖 `server` feature 的条件编译
- 保持与现有架构的一致性
