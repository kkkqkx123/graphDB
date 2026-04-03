# 条件编译与 Feature Flags 设计文档

## 概述

本文档描述了 GraphDB 项目的条件编译系统设计，包括 feature flags 的配置、使用场景以及最佳实践。

## 设计目标

1. **模块化**：允许用户根据需求选择功能组件
2. **依赖优化**：仅编译实际需要的依赖项，减少编译时间和二进制大小
3. **清晰的 API 边界**：通过 feature flags 明确区分不同的使用场景
4. **向后兼容**：保持默认配置的稳定性

## Feature Flags 配置

### 完整配置列表

```toml
[features]
default = ["server", "redb"]

# Storage backend
redb = ["dep:redb"]

# Embedded API for standalone/embedded usage (Rust API only)
embedded = []

# C API bindings (requires embedded, enables cdylib generation and cbindgen)
c-api = ["embedded", "dep:cbindgen"]

# Server API (HTTP/Web interface)
server = [
    "dep:axum",
    "dep:tower",
    "dep:tower-http",
    "dep:http",
    "dep:sqlx",
    "dep:async-trait",
]
```

### Feature 依赖关系图

```
default (server + redb)
├── server
│   ├── axum
│   ├── tower
│   ├── tower-http
│   ├── http
│   ├── sqlx
│   └── async-trait
└── redb

embedded (standalone)
└── (no additional dependencies)

c-api
├── embedded (required)
└── cbindgen (build dependency)
```

## 使用场景

### 场景 1：默认服务器模式（推荐用于生产环境）

**用途**：完整的 HTTP/Web 服务器，包含所有功能

**编译命令**：
```bash
cargo build --release
# 或显式指定
cargo build --release --features server,redb
```

**包含组件**：
- ✅ HTTP API 服务器（Axum）
- ✅ Web 管理界面后端
- ✅ 用户认证与权限管理
- ✅ 批处理接口
- ✅ 全文搜索（BM25 + Inverted Index）
- ✅ Redb 存储引擎
- ❌ C API（不包含）

**适用场景**：
- 部署为独立服务
- 通过 HTTP API 访问数据库
- 需要 Web 管理界面

---

### 场景 2：仅嵌入式 Rust 库

**用途**：作为 Rust 库直接嵌入应用程序，无需网络功能

**编译命令**：
```bash
cargo build --release --no-default-features --features embedded,redb
```

**包含组件**：
- ✅ Embedded Rust API（类似 SQLite 的使用方式）
- ✅ Redb 存储引擎
- ❌ HTTP 服务器
- ❌ C API
- ❌ 网络相关依赖

**代码示例**：
```rust
use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 打开数据库
    let db = GraphDatabase::open("my_database")?;
    
    // 创建会话
    let mut session = db.session()?;
    
    // 切换到空间
    session.use_space("test_space")?;
    
    // 执行查询
    let result = session.execute("MATCH (n) RETURN n")?;
    
    Ok(())
}
```

**适用场景**：
- 桌面应用程序
- 嵌入式设备
- 单机应用
- 测试和开发环境

---

### 场景 3：C API 库（FFI）

**用途**：提供 C 语言接口，用于与其他语言绑定

**编译命令**：
```bash
cargo build --release --features c-api
```

**包含组件**：
- ✅ Embedded Rust API
- ✅ C API 绑定（`src/api/embedded/c_api/`）
- ✅ cbindgen（构建时生成 C 头文件）
- ✅ 生成 `libgraphdb.so` / `graphdb.dll` 动态库
- ✅ 生成 `include/graphdb.h` 头文件
- ❌ HTTP 服务器（除非同时启用 server feature）

**代码示例（C 语言）**：
```c
#include <graphdb.h>

int main() {
    graphdb_t* db = graphdb_open("my_database");
    if (!db) return 1;
    
    graphdb_session_t* session = graphdb_create_session(db);
    graphdb_use_space(session, "test_space");
    
    graphdb_result_t* result = graphdb_execute(session, "MATCH (n) RETURN n");
    // 处理结果...
    
    graphdb_result_free(result);
    graphdb_session_free(session);
    graphdb_close(db);
    
    return 0;
}
```

**适用场景**：
- Python/Node.js/Ruby 等语言绑定
- 与 C/C++ 项目集成
- 跨语言调用

---

### 场景 4：混合模式（Embedded + Server）

**用途**：同时提供嵌入式 API 和 HTTP 服务

**编译命令**：
```bash
cargo build --release --features embedded,server,redb
```

**包含组件**：
- ✅ 所有 Embedded API 功能
- ✅ 所有 Server API 功能
- ✅ Redb 存储引擎
- ❌ C API（除非启用 c-api）

**适用场景**：
- 需要本地嵌入 + 远程访问双重模式
- 开发调试工具

---

## 条件编译在代码中的使用

### 模块级别条件编译

```rust
// src/api/mod.rs
pub mod core;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "embedded")]
pub mod embedded;

#[cfg(feature = "server")]
pub use server::{session, HttpServer};

#[cfg(feature = "embedded")]
pub use embedded::GraphDatabase;
```

```rust
// src/lib.rs
pub mod api;
pub mod core;
pub mod storage;
// ... 其他模块

#[cfg(feature = "c-api")]
pub mod c_api;
```

### 子模块条件编译

```rust
// src/api/embedded/mod.rs
pub mod database;
pub mod session;
pub mod transaction;

// C API 模块（仅在启用 c-api feature 时编译）
#[cfg(feature = "c-api")]
pub mod c_api;
```

### 函数级别条件编译

```rust
// src/query/executor/expression/functions/mod.rs
#[cfg(feature = "c-api")]
pub fn c_api_function_example() {
    // C API 特定函数
}

#[cfg(not(feature = "c-api"))]
pub fn c_api_function_example() {
    // 空实现或返回错误
}
```

### 测试文件条件编译

```rust
// tests/integration_embedded_api.rs
#![cfg(feature = "embedded")]

#[test]
fn test_embedded_database() {
    // Embedded API 测试
}
```

```rust
// tests/integration_c_api.rs
#![cfg(feature = "c-api")]

#[test]
fn test_c_api_binding() {
    // C API 测试
}
```

---

## 构建产物对比

| Feature 组合 | 库类型 | 二进制 | 头文件 | 主要依赖 |
|-------------|--------|--------|--------|----------|
| `default` | rlib | graphdb-server | 无 | axum, tower, sqlx |
| `embedded` | rlib | 无 | 无 | 基础依赖 |
| `c-api` | cdylib + rlib | 无 | graphdb.h | cbindgen |
| `server` | rlib | graphdb-server | 无 | axum, tower, sqlx |
| `embedded,server` | rlib | graphdb-server | 无 | axum, tower, sqlx |
| `c-api,server` | cdylib + rlib | graphdb-server | graphdb.h | 全部 |

---

## 依赖管理

### Optional Dependencies

以下依赖项被标记为 `optional = true`，仅在对应的 feature 启用时编译：

| 依赖 | Feature | 用途 |
|------|---------|------|
| `redb` | `redb` | 存储引擎 |
| `axum` | `server` | HTTP 框架 |
| `tower` | `server` | 服务抽象层 |
| `tower-http` | `server` | HTTP 中间件 |
| `http` | `server` | HTTP 类型定义 |
| `sqlx` | `server` | SQLite 客户端（Web 元数据存储） |
| `async-trait` | `server` | 异步 trait 支持 |
| `cbindgen` | `c-api` | C 头文件生成（build-dependency） |

### 平台特定依赖

```toml
[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["..."] }
```

这些依赖根据目标操作系统自动选择，无需手动配置。

---

## Build Script 行为

`build.rs` 根据 feature flags 执行不同的构建逻辑：

```rust
// build.rs
fn main() {
    // 仅在启用 c-api feature 时生成 C 头文件
    if env::var("CARGO_FEATURE_C_API").is_ok() {
        generate_c_header();
    }
    
    // 设置重新编译触发条件
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/api/embedded/c_api/");
}
```

**环境变量检测**：
- `CARGO_FEATURE_C_API`：当启用 `c-api` feature 时设置
- `CARGO_FEATURE_EMBEDDED`：当启用 `embedded` feature 时设置
- `CARGO_FEATURE_SERVER`：当启用 `server` feature 时设置

---

## 最佳实践

### 1. 最小化依赖原则

在 `Cargo.toml` 中仅声明实际需要的 features：

```toml
# ❌ 不推荐：引入不必要的依赖
[dependencies]
graphdb = "0.1.0"  # 默认启用 server

# ✅ 推荐：明确指定需要的功能
[dependencies]
graphdb = { version = "0.1.0", default-features = false, features = ["embedded", "redb"] }
```

### 2. 条件编译代码组织

- 将 feature 特定的代码放在独立的模块中
- 使用 `#[cfg(feature = "...")]` 标记模块而非单个函数
- 为不同 feature 提供一致的公共 API（使用空实现或返回错误）

### 3. 测试策略

为不同的 feature 组合编写集成测试：

```rust
// tests/integration_embedded_api.rs
#![cfg(feature = "embedded")]

// tests/integration_server_api.rs
#![cfg(feature = "server")]

// tests/integration_c_api.rs
#![cfg(feature = "c-api")]
```

### 4. 文档示例

在代码示例中明确标注所需的 features：

```rust
//! ```rust,ignore
//! // 需要启用 embedded feature
//! use graphdb::api::embedded::GraphDatabase;
//! ```
```

---

## 常见问题

### Q1: 为什么 `embedded` feature 是空的？

`embedded` feature 本身不引入额外依赖，它作为标记用于：
- 控制 `src/api/embedded/` 模块的编译
- 作为 `c-api` feature 的基础依赖
- 提供清晰的 API 边界

### Q2: 可以同时启用 `server` 和 `c-api` 吗？

可以。这会同时编译 HTTP 服务器和 C API 绑定，生成动态库和可执行文件。

### Q3: 如何只编译库而不生成二进制文件？

```bash
cargo build --lib --no-default-features --features embedded,redb
```

### Q4: `crate-type = ["cdylib", "rlib"]` 会影响性能吗？

不会。这仅影响编译输出的格式，不影响运行时性能。但在仅需要 rlib 的场景下，会额外编译 cdylib 版本。

---

## 版本历史

| 版本 | 日期 | 变更说明 |
|------|------|----------|
| 0.1.0 | 2026-04-03 | 初始版本，移除未使用的 `system_monitor` 和 `executor_internal` features |

---

## 参考文档

- [Cargo Features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Conditional Compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
- [crate-type 字段](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-crate-type-field)

---

## 维护者备注

- 添加新 feature 时，确保在文档中更新依赖关系图
- 删除 feature 前，检查所有代码引用和测试文件
- 保持 feature 名称的语义清晰，避免歧义
- 定期审查 optional dependencies，移除未使用的依赖
