# Inversearch API 架构重构设计方案

## 1. 背景与动机

### 1.1 当前问题

Inversearch 目前采用平铺式目录结构，所有模块直接位于 `src/` 目录下：

```
src/
├── charset/      # 字符集处理
├── document/     # 文档管理
├── index/        # 索引核心
├── search/       # 搜索功能
├── storage/      # 存储层
├── service.rs    # gRPC服务实现
├── proto.rs      # Protocol Buffers
├── lib.rs        # 库入口
└── main.rs       # 服务入口
```

**存在的问题：**

1. **职责边界模糊**：核心功能、嵌入式API、服务端实现混杂在一起
2. **库使用体验差**：嵌入式用户需要理解 `Index`, `StorageInterface`, `Resolver` 等复杂内部概念
3. **编译依赖冗余**：`Cargo.toml` 中 `default = ["service", "store"]` 强制库用户编译 gRPC 依赖
4. **与 BM25 不一致**：同为搜索服务，架构风格差异大，增加维护成本

### 1.2 参考：BM25 的优秀实践

BM25 采用清晰的三层架构：

```
src/
├── api/
│   ├── core/      # 核心功能（索引、搜索、文档管理）
│   ├── embedded/  # 嵌入式库API（简洁的高级接口）
│   └── server/    # gRPC服务实现
├── lib.rs         # 条件导出
└── main.rs        # 服务入口
```

**优势：**
- 嵌入式用户只需使用 `Bm25Index::create()`, `add_document()`, `search()` 等简单API
- 服务端代码与核心功能解耦
- 通过 feature flag 灵活控制编译内容

---

## 2. 设计目标

### 2.1 核心目标

1. **分层清晰**：明确区分核心功能、嵌入式API、服务端实现
2. **使用友好**：为嵌入式用户提供简洁的高级API，隐藏内部复杂性
3. **编译灵活**：库模式不依赖 gRPC 相关 crate
4. **架构一致**：与 BM25 保持统一的架构风格

### 2.2 非目标

- 不修改核心搜索算法和索引逻辑
- 不改变 gRPC 接口定义
- 不引入新的外部依赖

---

## 3. 新架构设计

### 3.1 目录结构

```
inversearch/src/
├── api/
│   ├── core/              # 核心功能层
│   │   ├── mod.rs         # 导出核心模块
│   │   ├── charset/       # 字符集处理（从 src/charset 移动）
│   │   ├── common/        # 通用工具（从 src/common 移动）
│   │   ├── compress/      # 压缩工具（从 src/compress 移动）
│   │   ├── document/      # 文档管理（从 src/document 移动）
│   │   ├── encoder/       # 编码器（从 src/encoder 移动）
│   │   ├── highlight/     # 高亮功能（从 src/highlight 移动）
│   │   ├── index/         # 索引核心（从 src/index 移动）
│   │   ├── intersect/     # 交集计算（从 src/intersect 移动）
│   │   ├── keystore/      # 键值存储（从 src/keystore 移动）
│   │   ├── resolver/      # 结果解析（从 src/resolver 移动）
│   │   ├── search/        # 搜索功能（从 src/search 移动）
│   │   ├── serialize/     # 序列化（从 src/serialize 移动）
│   │   ├── storage/       # 存储层（从 src/storage 移动）
│   │   ├── tokenizer/     # 分词器（从 src/tokenizer 移动）
│   │   ├── type/          # 类型定义（从 src/type 移动）
│   │   ├── async_.rs      # 异步工具（从 src/async_.rs 移动）
│   │   ├── config.rs      # 配置（从 src/config 移动）
│   │   ├── error.rs       # 错误定义（从 src/error 移动）
│   │   └── metrics.rs     # 指标（从 src/metrics 移动）
│   │
│   ├── embedded/          # 嵌入式库API层
│   │   ├── mod.rs         # 导出嵌入式API
│   │   └── index.rs       # EmbeddedIndex 实现
│   │
│   └── server/            # 服务端实现层
│       ├── mod.rs         # 导出服务端组件
│       ├── config.rs      # 服务配置（从 src/config 分离）
│       ├── grpc.rs        # gRPC服务（从 src/service.rs 移动）
│       ├── metrics.rs     # 服务指标
│       └── proto.rs       # Protocol Buffers（从 src/proto.rs 移动）
│
├── lib.rs                 # 库入口，条件导出
└── main.rs                # 服务入口（保持不变）
```

### 3.2 Feature 配置

**Cargo.toml 修改：**

```toml
[features]
default = ["embedded"]                    # 默认嵌入式模式
embedded = []                             # 纯库模式，无额外依赖
service = ["tonic", "prost", "tokio/full"] # 服务模式，启用gRPC
store = ["store-cold-warm-cache"]
store-cold-warm-cache = []
store-file = []
store-redis = ["redis"]
store-wal = []

[dependencies]
# 核心依赖（始终需要）
tokio = { version = "1.48", features = ["rt-multi-thread", "macros"] }
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
# ... 其他核心依赖

# 服务依赖（可选）
tonic = { version = "0.12", optional = true }
prost = { version = "0.13", optional = true }
redis = { version = "0.29", optional = true }
```

### 3.3 API 设计

#### 3.3.1 核心层 (api/core/)

核心层保持现有功能不变，仅调整模块路径：

```rust
// api/core/mod.rs
pub mod charset;
pub mod common;
pub mod compress;
pub mod document;
pub mod encoder;
pub mod error;
pub mod highlight;
pub mod index;
pub mod intersect;
pub mod keystore;
pub mod metrics;
pub mod resolver;
pub mod search;
pub mod serialize;
pub mod storage;
pub mod tokenizer;
pub mod r#type;
pub mod async_;
pub mod config;

// 重新导出核心类型
pub use document::{Document, Field, Batch, ...};
pub use index::Index;
pub use search::{search, SearchOptions, SearchResult, ...};
// ... 其他导出
```

#### 3.3.2 嵌入式层 (api/embedded/)

提供简洁的高级API，隐藏内部复杂性：

```rust
// api/embedded/mod.rs
pub mod index;

pub use index::{EmbeddedIndex, EmbeddedSearchResult, EmbeddedIndexConfig};
```

```rust
// api/embedded/index.rs
use crate::api::core::{Index, Document, SearchOptions, ...};
use crate::api::core::storage::common::trait::StorageInterface;
use std::sync::Arc;

/// 搜索结果
#[derive(Debug, Clone)]
pub struct EmbeddedSearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub highlights: Option<Vec<String>>,
}

/// 嵌入式索引配置
#[derive(Debug, Clone)]
pub struct EmbeddedIndexConfig {
    pub index_path: String,
    pub enable_highlighting: bool,
    pub default_search_limit: usize,
    // ... 其他用户友好的配置项
}

impl Default for EmbeddedIndexConfig {
    fn default() -> Self {
        Self {
            index_path: "./index".to_string(),
            enable_highlighting: true,
            default_search_limit: 10,
        }
    }
}

/// 嵌入式索引 - 为库用户提供简洁的API
pub struct EmbeddedIndex {
    index: Index,
    config: EmbeddedIndexConfig,
}

impl EmbeddedIndex {
    /// 创建新索引
    pub fn create(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let config = EmbeddedIndexConfig::default();
        Self::create_with_config(path, config)
    }

    /// 使用配置创建索引
    pub fn create_with_config(
        path: impl AsRef<std::path::Path>,
        config: EmbeddedIndexConfig
    ) -> Result<Self, Error> {
        // 简化创建逻辑，隐藏内部复杂性
        let index = Index::create(path, ...)?;
        Ok(Self { index, config })
    }

    /// 打开已有索引
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        // ...
    }

    /// 添加文档（简化版）
    pub fn add(&self, id: &str, content: &str) -> Result<(), Error> {
        // 隐藏 Document, Field 等复杂类型的创建
        let doc = Document::new(id, content);
        self.index.add_document(doc)
    }

    /// 添加带字段的文档
    pub fn add_with_fields(
        &self,
        id: &str,
        fields: Vec<(String, String)>
    ) -> Result<(), Error> {
        // ...
    }

    /// 搜索（简化版）
    pub fn search(&self, query: &str) -> Result<Vec<EmbeddedSearchResult>, Error> {
        self.search_with_limit(query, self.config.default_search_limit)
    }

    /// 带限制的搜索
    pub fn search_with_limit(
        &self,
        query: &str,
        limit: usize
    ) -> Result<Vec<EmbeddedSearchResult>, Error> {
        // 隐藏 SearchOptions, Resolver 等复杂类型
        let options = SearchOptions::default();
        let results = self.index.search(query, &options, limit)?;
        
        // 转换为简化的结果格式
        results.into_iter()
            .map(|r| self.to_embedded_result(r))
            .collect()
    }

    /// 删除文档
    pub fn remove(&self, id: &str) -> Result<(), Error> {
        self.index.remove_document(id)
    }

    /// 批量操作
    pub fn batch(&self) -> EmbeddedBatch {
        EmbeddedBatch::new(&self.index)
    }

    /// 获取统计信息
    pub fn stats(&self) -> EmbeddedIndexStats {
        // 返回简化的统计信息
    }

    // 内部转换方法
    fn to_embedded_result(&self, result: CoreSearchResult) -> EmbeddedSearchResult {
        // ...
    }
}

/// 批量操作构建器
pub struct EmbeddedBatch<'a> {
    index: &'a Index,
    operations: Vec<BatchOperation>,
}

impl<'a> EmbeddedBatch<'a> {
    pub fn add(&mut self, id: &str, content: &str) -> &mut Self {
        // ...
        self
    }

    pub fn remove(&mut self, id: &str) -> &mut Self {
        // ...
        self
    }

    pub fn execute(self) -> Result<BatchResult, Error> {
        // ...
    }
}
```

#### 3.3.3 服务端层 (api/server/)

服务端代码从现有 `service.rs` 和 `proto.rs` 迁移：

```rust
// api/server/mod.rs
#![cfg(feature = "service")]

pub mod config;
pub mod grpc;
pub mod metrics;
pub mod proto;

pub use config::{ServerConfig, ServiceConfig};
pub use grpc::{run_server, InversearchService};
```

```rust
// api/server/grpc.rs
#![cfg(feature = "service")]

use tonic::{transport::Server, Request, Response, Status};
use crate::api::server::proto::...;
use crate::api::core::{Index, ...};

pub struct InversearchService {
    // ...
}

pub async fn run_server(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // ...
}
```

### 3.4 库入口 (lib.rs)

```rust
// lib.rs

// ========== 核心模块（始终可用）==========
pub mod api;

// 重新导出核心API
pub use api::core;

// 核心类型导出
pub use api::core::{
    Document, Field, FieldType,
    Index, IndexOptions,
    SearchOptions, SearchResult,
    // ... 其他核心类型
};

// ========== 嵌入式API（embedded feature）==========
#[cfg(feature = "embedded")]
pub use api::embedded;

#[cfg(feature = "embedded")]
pub use api::embedded::{
    EmbeddedIndex,
    EmbeddedSearchResult,
    EmbeddedIndexConfig,
    EmbeddedBatch,
};

// ========== 服务端API（service feature）==========
#[cfg(feature = "service")]
pub use api::server;

#[cfg(feature = "service")]
pub use api::server::{
    ServerConfig,
    ServiceConfig,
    run_server,
    InversearchService,
};

// 条件编译的存储后端
#[cfg(feature = "store-file")]
pub use api::core::storage::FileStorage;

#[cfg(feature = "store-redis")]
pub use api::core::storage::RedisStorage;

#[cfg(feature = "store-wal")]
pub use api::core::storage::{WALStorage, WALManager};
```

### 3.5 服务入口 (main.rs)

保持现有逻辑，仅调整导入路径：

```rust
// main.rs

#[cfg(feature = "service")]
use inversearch_service::api::server::{run_server, ServiceConfig};

#[cfg(feature = "service")]
fn main() {
    // ... 现有代码
}

#[cfg(not(feature = "service"))]
fn main() {
    eprintln!("Inversearch is compiled in library mode.");
    eprintln!("To build as a service: cargo build --features service");
    std::process::exit(1);
}
```

---

## 4. 迁移计划

### 4.1 阶段一：目录结构调整

1. 创建 `api/core/`, `api/embedded/`, `api/server/` 目录
2. 将现有模块移动到 `api/core/`
3. 更新所有内部导入路径

### 4.2 阶段二：嵌入式API实现

1. 设计 `EmbeddedIndex` API 接口
2. 实现 `api/embedded/index.rs`
3. 编写嵌入式API单元测试

### 4.3 阶段三：服务端迁移

1. 将 `service.rs` 移动到 `api/server/grpc.rs`
2. 将 `proto.rs` 移动到 `api/server/proto.rs`
3. 分离服务端配置到 `api/server/config.rs`

### 4.4 阶段四：Feature 配置调整

1. 修改 `Cargo.toml` 默认 feature
2. 添加条件编译属性
3. 验证不同 feature 组合下的编译

### 4.5 阶段五：测试与文档

1. 运行完整测试套件
2. 更新文档和示例
3. 编写迁移指南

---

## 5. 使用示例

### 5.1 嵌入式使用（新方式）

```rust
// Cargo.toml
[dependencies]
inversearch-service = { path = "../inversearch", default-features = false, features = ["embedded", "store-file"] }

// main.rs
use inversearch_service::{EmbeddedIndex, EmbeddedIndexConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建索引
    let index = EmbeddedIndex::create("./my_index")?;
    
    // 添加文档
    index.add("doc1", "Hello world")?;
    index.add("doc2", "Rust programming language")?;
    
    // 搜索
    let results = index.search("rust")?;
    for result in results {
        println!("{}: {} (score: {})", result.id, result.content, result.score);
    }
    
    // 批量操作
    index.batch()
        .add("doc3", "Batch document 1")
        .add("doc4", "Batch document 2")
        .remove("doc1")
        .execute()?;
    
    Ok(())
}
```

### 5.2 服务端使用（不变）

```bash
# 编译并运行服务
cargo run --features service
```

### 5.3 高级使用（直接访问核心API）

```rust
use inversearch_service::core::{Index, Document, SearchOptions, Resolver};

// 直接使用核心API进行高级操作
let index = Index::create(path, index_options)?;
let resolver = Resolver::new(resolver_options)?;
// ...
```

---

## 6. 收益分析

### 6.1 对嵌入式用户

| 方面 | 改造前 | 改造后 |
|------|--------|--------|
| 导入复杂度 | 需理解10+模块 | 只需 `EmbeddedIndex` |
| 代码行数（简单用例） | ~50行 | ~10行 |
| 编译依赖 | 强制包含 tonic, prost | 仅核心依赖 |
| 学习曲线 | 陡峭 | 平缓 |

### 6.2 对服务端用户

- 无负面影响，gRPC 接口保持不变
- 代码组织更清晰，便于维护

### 6.3 对开发者

- 与 BM25 架构一致，降低认知负担
- 分层清晰，便于单元测试
- 条件编译减少不必要依赖

---

## 7. 风险评估

### 7.1 潜在风险

1. **导入路径变更**：现有代码需要更新导入路径
2. **编译问题**：条件编译可能引入新的编译错误
3. **测试覆盖**：需要确保所有 feature 组合都被测试

### 7.2 缓解措施

1. 提供详细的迁移指南
2. 保持向后兼容的 re-export（短期内）
3. 添加 CI 测试矩阵覆盖所有 feature 组合

---

## 8. 结论

本设计方案通过引入 `api/core/`, `api/embedded/`, `api/server/` 三层架构，解决了 Inversearch 当前存在的职责边界模糊、库使用体验差、编译依赖冗余等问题。新架构与 BM25 保持一致，既提升了嵌入式用户的开发体验，又保持了服务端的完整功能。

**建议优先级：高** - 此改造将显著提升项目的可维护性和用户体验。
