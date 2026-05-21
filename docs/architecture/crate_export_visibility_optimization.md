# Crate Export Visibility Optimization

## 背景

分析 `crates/bm25` (bm25-service) 与 `crates/inversearch` (inversearch-service) 两个 crate 的导出结构，缩减非必要的 `pub` 导出，将仅在 crate 内部使用的模块和项改为 `pub(crate)`。

当前唯一的真实外部使用者是根 crate (`graphdb`)，仅使用以下入口：
- **bm25-service**: `Bm25Index` + `IndexManagerConfig`
- **inversearch-service**: `EmbeddedIndex` + `EmbeddedConfig`

两个 crate 的 `lib.rs` 存在大量宽泛的 re-export（`pub use module::*` 或逐项 re-export），将大量内部实现细节暴露为公共 API，增加了维护负担和误用风险。

---

## 1. crates/bm25 (bm25-service)

### 1.1 外部真实依赖

| 路径 | 用途 |
|------|------|
| `bm25_service::api::embedded::Bm25Index` | graphdb 的 `Bm25SearchEngine` 包装 |
| `bm25_service::config::IndexManagerConfig` | graphdb 的 config 层引用 |

### 1.2 当前 lib.rs 导出问题

```rust
// lib.rs (当前)
pub mod api;                    // 对外开放所有 api 子模块
pub mod config;                 // 对外开放 config 子模块
pub mod error;                  // 对外开放 error（但 error 内部类型均已在 lib 层 re-export）
pub mod storage;                // 对外开放整个 storage 实现
pub mod tokenizer;              // 对外开放整个 tokenizer

pub use api::core;              // 将 core 模块整体提升至 crate 根路径
pub use api::core::{...};       // 再次逐项 re-export core 内所有项
pub use api::embedded::{...};   // 正确，仅导出外部需要的项
pub use error::{...};           // 正确
pub use config::{...};          // 正确
pub use storage::{...};         // 过度导出：storage 内部实现不应暴露
```

### 1.3 修改方案

#### 1.3.1 模块级可见性调整

| 模块 | 当前 | 改为 | 原因 |
|------|------|------|------|
| `api::core` | `pub` (子模块全部 `pub`) | `pub(crate)` (子模块项改为 `pub(crate)`) | 仅被 `api/embedded.rs` 和 `config/mod.rs` 内部使用 |
| `storage` | `pub` | `pub(crate)` | 所有 storage 实现仅内部使用，外部只需通过 `Bm25Index` 操作 |
| `tokenizer` | `pub` | `pub(crate)` | 仅内部使用 |
| `config::loader` | `pub` | `pub(crate)` | 仅内部使用 |
| `config::validator` | `pub` | `pub(crate)` | 仅内部使用 |
| `config::builder` | `pub(crate)` 已有 | — | 维持 |
| `error` | `pub` | `pub(crate)` | 所有 error 类型已在 `lib.rs` re-export，无需暴露模块路径 |
| `api::embedded` | `pub` | 维持 `pub` | 外部通过此路径使用 `Bm25Index` |

#### 1.3.2 子模块项可见性调整 (`api::core/*`)

`api/core/` 下的各项函数和类型仅在 `api/embedded.rs` 内部调用，无需对外暴露：

| 文件 | 当前导出 | 改为 | 原因 |
|------|----------|------|------|
| `api/core/mod.rs` | `pub mod batch;` 等 + 大量 `pub use` | 模块改为 `pub(crate)`, `pub use` 改为 `pub(crate) use` | 所有 re-export 仅内部使用 |
| `api/core/batch.rs` | `pub fn batch_*` | `pub(crate)` | 仅被 `api/embedded.rs` 调用 |
| `api/core/delete.rs` | `pub fn *` | `pub(crate)` | 同上 |
| `api/core/document.rs` | `pub fn *` | `pub(crate)` | 同上 |
| `api/core/index.rs` | `pub struct IndexManager` 等 | `pub(crate)` | 除 `IndexManagerConfig` 外均不需要对外暴露。`IndexManagerConfig` 需要保留 `pub` 因为通过 `config` 模块 re-export 给外部 |
| `api/core/persistence.rs` | `pub *` | `pub(crate)` | 仅内部使用 |
| `api/core/schema.rs` | `pub struct IndexSchema` | `pub(crate)` | 仅内部使用 |
| `api/core/search.rs` | `pub fn search` 等 | `pub(crate)` | 仅内部使用 |
| `api/core/stats.rs` | `pub *` | `pub(crate)` | 仅内部使用 |

#### 1.3.3 lib.rs re-export 精简

移除以下不再需要的 lib 层 re-export：

```rust
// 移除整行
pub use api::core;    // 不需要将 core 模块暴露为公共路径

// 保留以下（外部确实需要）：
pub use api::embedded::{Bm25Index, SearchResult, SearchResultWithHighlights};
pub use config::IndexManagerConfigBuilder;
pub use config::{Bm25Config, FieldWeights, SearchConfig};
```

---

## 2. crates/inversearch (inversearch-service)

### 2.1 外部真实依赖

| 路径 | 用途 |
|------|------|
| `inversearch_service::api::embedded::EmbeddedIndex` | graphdb 的 `InversearchEngine` 包装 |
| `inversearch_service::config::EmbeddedConfig` | graphdb 的 config 层引用 |

### 2.2 当前 lib.rs 导出问题

`lib.rs` 对几乎所有内部模块都做了 `pub use` 将内部项提升到 crate 根路径（共约 200+ 个 re-export），而外部仅使用了其中 2 个入口。此外 `api::core` 模块整体是 `lib.rs` 的镜像，属于冗余导出层。

### 2.3 修改方案

#### 2.3.1 模块级可见性调整

| 模块 | 当前 | 改为 | 原因 |
|------|------|------|------|
| `api::core` | `pub` | `pub(crate)` | 该模块仅为 lib.rs 的镜像，外部无直接使用。模块内所有 re-export 改为 `pub(crate)` |
| `charset` | `pub` | `pub(crate)` | 仅内部使用 |
| `common` | `pub` | `pub(crate)` | 仅内部使用 |
| `compress` | `pub` | `pub(crate)` | 仅内部使用 |
| `document` | `pub` | `pub(crate)` | 仅内部使用 |
| `encoder` | `pub` | `pub(crate)` | 仅内部使用 |
| `highlight` | `pub` | `pub(crate)` | 仅内部使用 |
| `index` | `pub` | `pub(crate)` | 仅内部使用 |
| `intersect` | `pub` | `pub(crate)` | 仅内部使用 |
| `keystore` | `pub` | `pub(crate)` | 仅内部使用 |
| `resolver` | `pub` | `pub(crate)` | 仅内部使用 |
| `search` | `pub` | `pub(crate)` | 仅内部使用 |
| `serialize` | `pub` | `pub(crate)` | 仅内部使用 |
| `storage` | `pub` | `pub(crate)` | 仅内部使用 |
| `tokenizer` | `pub` | `pub(crate)` | 仅内部使用 |
| `r#type` | `pub` | `pub(crate)` | 仅内部使用 |
| `error` | `pub` | `pub(crate)` | 类型已在 lib.rs 层 re-export |
| `async_` | `pub` | `pub(crate)` | 仅内部使用 |
| `api::embedded` | `pub` | 维持 `pub` | 外部通过此路径使用 `EmbeddedIndex` |
| `config` | `pub` | 维持 `pub` | 外部通过此路径使用 `EmbeddedConfig` |

#### 2.3.2 api::core 模块

`api/core/mod.rs` 是 `lib.rs` 的完全镜像，是冗余导出层，可直接降为 `pub(crate)`：

- 模块声明: `pub mod core` → `pub(crate) mod core` (在 `api/mod.rs` 中修改)
- 内部所有 `pub use crate::xxx::...` → `pub(crate) use crate::xxx::...`

#### 2.3.3 lib.rs re-export 精简

当前 `lib.rs` 有 108 行，包含 200+ 个 re-export 项。精简后仅保留外部需要的：

```rust
// 保留的模块声明
pub mod api;
pub mod config;

// 保留的 re-export（外部真实依赖）
pub use api::embedded::{
    EmbeddedBatch, EmbeddedBatchOperation, EmbeddedBatchResult,
    EmbeddedIndex, EmbeddedIndexBuilder, EmbeddedIndexStats,
    EmbeddedSearchResult,
};
pub use config::EmbeddedConfig;
```

其余所有 lib 层的 `pub use` 均移除。

#### 2.3.4 内部模块的跨模块引用调整

部分内部模块之间通过 `pub(crate)` 相互引用（如 `api/embedded.rs` → `Index`、`search`、`config`），这些引用在模块降级后不受影响，因为 `pub(crate)` 允许同 crate 内任意位置访问。

**例外：integration tests**

`crates/inversearch/tests/` 下有大量集成测试，它们以外部视角访问 crate 的 `pub` 项。若将内部模块全部降级，集成测试将无法编译。以下为两种解决方案：

**方案 A：将集成测试转为单元测试**

将 `tests/*.rs` 迁移为 `src/*_test.rs` 或 `src/**/tests.rs` 内的 `#[cfg(test)] mod tests`。这样它们作为 crate 内部代码可以访问 `pub(crate)` 项。

**方案 B：保留模块 `pub`，仅收缩 lib 层 re-export**

保持内部模块为 `pub`，但不在 `lib.rs` 中 re-export（外部需通过完整路径访问）。这种方式容易遗漏，且仍然暴露了模块路径给外部。

**推荐：方案 A**。集成测试转为单元测试后，整体可见性更可控，且测试代码与实现代码放在一起更易维护。

---

## 3. 实施步骤

### Phase 1: bm25-service

1. `storage/` 模块改为 `pub(crate)`，内部项改为 `pub(crate)`
2. `tokenizer/` 模块改为 `pub(crate)`
3. `api/core/` → 模块及内部所有项改为 `pub(crate)`
4. `config/loader.rs` `config/validator.rs` 改为 `pub(crate)`
5. `error/` 模块改为 `pub(crate)`
6. 精简 `lib.rs` re-export，移除 `pub use api::core;`
7. 运行 `cargo clippy --all-targets` 验证

### Phase 2: inversearch-service

1. 将 `tests/` 下集成测试迁移为 crate 内单元测试（`src/**/tests.rs` 或 `src/*_test.rs`）
2. 将 `charset`, `common`, `compress`, `document`, `encoder`, `highlight`, `index`, `intersect`, `keystore`, `resolver`, `search`, `serialize`, `storage`, `tokenizer`, `r#type`, `async_`, `error` 模块改为 `pub(crate)`
3. `api/core/` 模块及内部 re-export 改为 `pub(crate)`
4. 精简 `lib.rs` re-export
5. 运行 `cargo clippy --all-targets` 验证

### Phase 3: 验证

- `cargo clippy --all-targets --all-features`（无 warning）
- `cargo test --lib`（所有单元测试通过）
- `cargo test --test '*'`（如有残留集成测试，通过）
- 确认 graphdb 主 crate 可正常编译

---

## 4. 预期收益

| 指标 | bm25 | inversearch |
|------|------|-------------|
| lib.rs 行数 | 30 → ~15 | 108 → ~15 |
| 公共模块数 | 5 → 2 | 22 → 2 |
| 公共 API 项数 | ~50 → ~10 | ~200 → ~10 |
| 减少误用风险 | 中 | 高 |
