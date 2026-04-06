# BM25 模块架构重构设计

## 目录

1. [概述](#概述)
2. [架构目标](#架构目标)
3. [目录结构](#目录结构)
4. [模块设计](#模块设计)
5. [条件编译策略](#条件编译策略)
6. [API 设计](#api 设计)
7. [迁移计划](#迁移计划)

---

## 概述

当前 BM25 模块采用扁平化的模块结构，所有功能都直接放在 `src/` 目录下，通过 `#[cfg(feature)]` 进行条件编译。这种结构存在以下问题：

1. **模块职责不清晰**：库模式和服务模式的代码混在一起
2. **条件编译分散**：`#[cfg]` 标记散落在各个文件中，难以维护
3. **API 层次混乱**：底层 Tantivy API 和高层抽象 API 没有明确分离
4. **扩展性差**：新增使用模式需要修改多个文件

本设计提出将 `src/` 重构为 `src/api/` 目录，包含 `core`、`embedded`、`server` 三个子模块，实现清晰的职责分离。

---

## 架构目标

### 1. 职责分离
- **core**: 核心索引功能，与使用模式无关
- **embedded**: 嵌入式库模式的高级 API
- **server**: 服务模式（gRPC、缓存等）

### 2. 条件编译集中化
- 将 `#[cfg]` 标记集中在模块级别，而非文件内部
- 每个子模块要么完全编译，要么完全不编译

### 3. API 层次化
- **底层 API**: 直接暴露 Tantivy 功能（IndexManager）
- **中层 API**: 简化的库模式 API（Bm25Index）
- **高层 API**: 服务模式 API（gRPC 接口）

### 4. 易于扩展
- 新增使用模式只需添加新的子模块
- 不影响现有模块

---

## 目录结构

### 重构前
```
src/
├── config/          # 配置模块
├── index/           # 索引模块
│   ├── manager.rs   # 索引管理器
│   ├── schema.rs    # 索引模式
│   ├── search.rs    # 搜索功能
│   ├── document.rs  # 文档操作
│   ├── delete.rs    # 删除操作
│   ├── batch.rs     # 批量操作
│   ├── stats.rs     # 统计信息
│   ├── simple.rs    # 高级 API（命名不当）
│   └── mod.rs
├── service/         # 服务模块（条件编译）
├── error.rs         # 错误定义
├── lib.rs           # 库入口
└── main.rs          # 服务入口
```

### 重构后
```
src/
├── api/                      # API 模块（新增）
│   ├── core/                 # 核心 API（总是编译）
│   │   ├── index.rs          # 索引管理核心功能
│   │   ├── search.rs         # 搜索核心功能
│   │   ├── document.rs       # 文档操作核心功能
│   │   └── mod.rs            # 核心 API 导出
│   │
│   ├── embedded/             # 嵌入式 API（embedded 特性）
│   │   ├── index.rs          # Bm25Index 高级 API
│   │   ├── builder.rs        # 索引构建器
│   │   └── mod.rs            # 嵌入式 API 导出
│   │
│   ├── server/               # 服务器 API（service 特性）
│   │   ├── grpc.rs           # gRPC 服务实现
│   │   ├── handlers.rs       # 请求处理器
│   │   ├── cache.rs          # Redis 缓存
│   │   └── mod.rs            # 服务器 API 导出
│   │
│   └── mod.rs                # API 模块总入口
│
├── config/                   # 配置模块（保留）
├── error.rs                  # 错误定义（保留）
├── lib.rs                    # 库入口（更新）
└── main.rs                   # 服务入口（更新）
```

---

## 模块设计

### 1. `api::core` - 核心 API 模块

**职责**: 提供与使用模式无关的核心索引功能

**编译条件**: 总是编译（无 `#[cfg]`）

**导出内容**:
```rust
// api/core/mod.rs
pub use index::{IndexManager, IndexManagerConfig, IndexSchema};
pub use search::SearchOptions;
pub use document::Document;
```

**关键类型**:
- `IndexManager`: 索引管理器，直接封装 Tantivy
- `IndexManagerConfig`: 索引配置
- `IndexSchema`: 索引模式定义
- `SearchOptions`: 搜索选项
- `Document`: 文档表示

**示例**:
```rust
use bm25_service::api::core::{IndexManager, IndexSchema};

let manager = IndexManager::create("/path/to/index")?;
let schema = IndexSchema::new();
// 直接使用 Tantivy API
```

---

### 2. `api::embedded` - 嵌入式 API 模块

**职责**: 提供简化的高级 API，适合嵌入式场景

**编译条件**: `#[cfg(feature = "embedded")]`

**导出内容**:
```rust
// api/embedded/mod.rs
pub use index::{Bm25Index, SearchResult};
pub use builder::IndexBuilder;
```

**关键类型**:
- `Bm25Index`: 高级索引 API，封装 IndexManager
- `SearchResult`: 搜索结果结构
- `IndexBuilder`: 索引构建器

**示例**:
```rust
use bm25_service::api::embedded::Bm25Index;

let index = Bm25Index::create("/path/to/index")?;
index.add_document("1", "Title", "Content")?;
let results = index.search("query", 10)?;
```

---

### 3. `api::server` - 服务器 API 模块

**职责**: 提供 gRPC 服务实现

**编译条件**: `#[cfg(feature = "service")]`

**导出内容**:
```rust
// api/server/mod.rs
pub use grpc::{run_server, BM25Service};
pub use cache::CacheManager;
```

**关键类型**:
- `BM25Service`: gRPC 服务实现
- `CacheManager`: Redis 缓存管理
- `RequestHandler`: 请求处理器

**示例**:
```rust
use bm25_service::api::server::{run_server, Config};

let config = Config::default();
run_server(config).await?;
```

---

## 条件编译策略

### Cargo.toml 配置

```toml
[features]
default = ["embedded"]
embedded = []
service = [
    "embedded",  # 服务模式包含嵌入式模式
    "tonic",
    "prost",
    "tokio/full",
    "redis",
    "tracing",
    "tracing-subscriber",
    "metrics",
    "prost-build",
    "tonic-build",
]

[dependencies]
# 核心依赖（总是编译）
tantivy = "0.24"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"

# 可选依赖
tonic = { version = "0.12", optional = true }
prost = { version = "0.13", optional = true }
redis = { version = "1.0", features = ["tokio-comp"], optional = true }
# ...

[build-dependencies]
prost-build = { version = "0.13", optional = true }
tonic-build = { version = "0.12", optional = true }
```

### 模块级条件编译

```rust
// src/api/mod.rs
pub mod core;  // 总是编译

#[cfg(feature = "embedded")]
pub mod embedded;

#[cfg(feature = "service")]
pub mod server;

// src/lib.rs
pub use api::core;

#[cfg(feature = "embedded")]
pub use api::embedded;

#[cfg(feature = "service")]
pub use api::server;
```

---

## API 设计

### 1. 核心 API（api::core）

```rust
pub mod core {
    /// 索引管理器 - 直接封装 Tantivy
    pub struct IndexManager {
        index: Index,
        schema: Schema,
        config: IndexManagerConfig,
    }
    
    impl IndexManager {
        pub fn create<P: AsRef<Path>>(path: P) -> Result<Self>;
        pub fn open<P: AsRef<Path>>(path: P) -> Result<Self>;
        pub fn writer(&self) -> Result<IndexWriter>;
        pub fn reader(&self) -> Result<IndexReader>;
    }
    
    /// 搜索选项
    pub struct SearchOptions {
        pub limit: usize,
        pub offset: usize,
        pub filters: Vec<Filter>,
    }
}
```

**特点**:
- 直接暴露 Tantivy 的 `IndexWriter` 和 `IndexReader`
- 用户需要了解 Tantivy 的基本概念
- 适合需要精细控制的场景

---

### 2. 嵌入式 API（api::embedded）

```rust
pub mod embedded {
    /// 高级索引 API - 简化使用
    pub struct Bm25Index {
        manager: core::IndexManager,
        schema: core::IndexSchema,
    }
    
    impl Bm25Index {
        pub fn create<P: AsRef<Path>>(path: P) -> Result<Self>;
        pub fn add_document(&self, id: &str, title: &str, content: &str) -> Result<()>;
        pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
        pub fn delete(&self, id: &str) -> Result<()>;
    }
    
    /// 搜索结果
    pub struct SearchResult {
        pub document_id: String,
        pub title: Option<String>,
        pub content: Option<String>,
        pub score: f32,
    }
}
```

**特点**:
- 隐藏 Tantivy 的复杂性
- 提供简单直观的 API
- 适合快速集成到现有应用

---

### 3. 服务器 API（api::server）

```rust
pub mod server {
    /// gRPC 服务配置
    pub struct Config {
        pub address: SocketAddr,
        pub redis_url: String,
        pub index_path: String,
    }
    
    /// gRPC 服务实现
    pub struct BM25Service {
        index: embedded::Bm25Index,
        cache: CacheManager,
    }
    
    pub async fn run_server(config: Config) -> Result<()>;
}
```

**特点**:
- 基于 gRPC 的远程服务
- 集成 Redis 缓存
- 支持并发请求处理

---

## 迁移计划

### 阶段 1: 创建 api 目录结构（已完成设计）

1. 创建 `src/api/` 目录
2. 创建 `core`、`embedded`、`server` 子目录
3. 创建各模块的 `mod.rs`

### 阶段 2: 迁移核心功能

1. 将 `index/manager.rs` 迁移到 `api/core/index.rs`
2. 将 `index/search.rs` 迁移到 `api/core/search.rs`
3. 将 `index/document.rs` 迁移到 `api/core/document.rs`
4. 更新导入路径

### 阶段 3: 迁移嵌入式 API

1. 将 `index/api.rs` 重命名为 `api/embedded/index.rs`
2. 创建 `api/embedded/builder.rs`
3. 更新导入路径

### 阶段 4: 迁移服务器 API

1. 将 `service/` 目录内容迁移到 `api/server/`
2. 重构为模块化结构
3. 更新导入路径

### 阶段 5: 更新导出和文档

1. 更新 `src/lib.rs` 的导出
2. 更新 README.md
3. 更新示例代码

---

## 优势

### 1. 清晰的职责分离
- `core`: 核心功能，与模式无关
- `embedded`: 库模式，简化 API
- `server`: 服务模式，gRPC 接口

### 2. 条件编译集中化
- 模块级别的 `#[cfg]`，而非文件内部
- 编译条件一目了然

### 3. 易于理解和维护
- 目录结构反映功能层次
- 新开发者能快速理解架构

### 4. 易于扩展
- 新增使用模式只需添加新子模块
- 不影响现有代码

### 5. 更好的 API 设计
- 分层 API 满足不同需求
- 核心 API 保持灵活性
- 嵌入式 API 提供便利性

---

## 风险评估

### 低风险
- 内部重构，不改变公共 API 行为
- 渐进式迁移，可回滚

### 中风险
- 导入路径变化需要全面更新
- 需要全面测试

### 缓解措施
- 保持向后兼容的导出
- 充分的单元测试
- 分阶段迁移

---

## 时间估算

| 阶段 | 工作量 | 说明 |
|------|--------|------|
| 阶段 1 | 0.5 天 | 创建目录结构 |
| 阶段 2 | 1-2 天 | 迁移核心功能 |
| 阶段 3 | 0.5 天 | 迁移嵌入式 API |
| 阶段 4 | 1 天 | 迁移服务器 API |
| 阶段 5 | 0.5 天 | 更新导出和文档 |
| **总计** | **3.5-4.5 天** | 包含测试时间 |

---

## 总结

本设计提出将 BM25 模块重构为 `api/` 目录包含 `core`、`embedded`、`server` 三个子模块的架构，实现：

1. ✅ **职责清晰**: 每个子模块有明确的职责边界
2. ✅ **条件编译集中**: 模块级 `#[cfg]` 标记
3. ✅ **API 层次化**: 满足不同使用场景
4. ✅ **易于扩展**: 新增模式不影响现有代码

这种架构既保持了灵活性，又提供了便利性，适合作为长期维护的基础。
