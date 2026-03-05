# C API 架构调整说明

## 调整原因

根据项目架构分析，C API 应该整合到 `embedded` 模块中，原因如下：

1. **功能定位一致**：C API 和 embedded API 都是面向嵌入场景的接口
2. **依赖关系清晰**：C API 依赖于 embedded API 的功能
3. **特性管理简化**：通过 `embedded/c_api` 特性统一管理
4. **代码组织合理**：所有嵌入相关的代码集中在一个目录

## 架构变更

### 变更前

```
src/api/
├── core/
├── embedded/
│   ├── batch.rs
│   ├── config.rs
│   ├── database.rs
│   ├── result.rs
│   ├── session.rs
│   ├── statement.rs
│   └── transaction.rs
├── server/
└── c_api/          # 独立的 C API 模块
    ├── mod.rs
    ├── types.rs
    └── error.rs
```

### 变更后

```
src/api/
├── core/
├── embedded/
│   ├── batch.rs
│   ├── config.rs
│   ├── database.rs
│   ├── result.rs
│   ├── session.rs
│   ├── statement.rs
│   ├── transaction.rs
│   └── c_api/      # C API 作为 embedded 的子模块
│       ├── mod.rs
│       ├── types.rs
│       └── error.rs
└── server/
```

## 特性配置变更

### Cargo.toml

**变更前**：
```toml
[features]
default = ["redb", "embedded", "server"]
redb = ["dep:redb"]
embedded = []
server = ["dep:axum", "dep:tower", "dep:tower-http", "dep:http"]
c_api = []  # 独立的 c_api 特性
```

**变更后**：
```toml
[features]
default = ["redb", "embedded", "server"]
redb = ["dep:redb"]
embedded = ["embedded/c_api"]  # c_api 作为 embedded 的子特性
server = ["dep:axum", "dep:tower", "dep:tower-http", "dep:http"]
```

## 模块导入变更

### src/api/mod.rs

**变更前**：
```rust
//! - `embedded` - 嵌入式 API（单机使用）
//! - `c_api` - C 语言 API（跨语言绑定）

pub mod core;
pub mod embedded;

#[cfg(feature = "c_api")]
pub mod c_api;
```

**变更后**：
```rust
//! - `embedded` - 嵌入式 API（单机使用，包含 C API）

pub mod core;
pub mod embedded;
```

### src/api/embedded/mod.rs

**变更前**：
```rust
// 子模块
pub mod batch;
pub mod config;
pub mod database;
pub mod result;
pub mod session;
pub mod statement;
pub mod transaction;
```

**变更后**：
```rust
// 子模块
pub mod batch;
pub mod config;
pub mod database;
pub mod result;
pub mod session;
pub mod statement;
pub mod transaction;

#[cfg(feature = "c_api")]
pub mod c_api;  // C API 作为条件编译的子模块
```

## 构建脚本变更

### build.rs

**变更前**：
```rust
fn main() {
    #[cfg(feature = "c_api")]
    {
        println!("cargo:rerun-if-changed=src/api/c_api");
        // ...
    }
}
```

**变更后**：
```rust
fn main() {
    #[cfg(feature = "embedded/c_api")]
    {
        println!("cargo:rerun-if-changed=src/api/embedded/c_api");
        // ...
    }
}
```

## 使用方式变更

### Rust 项目中使用

**变更前**：
```toml
[dependencies]
graphdb = { version = "0.1.0", features = ["embedded", "c_api"] }
```

**变更后**：
```toml
[dependencies]
graphdb = { version = "0.1.0", features = ["embedded"] }  # 自动包含 c_api
```

### C 项目中使用

**变更前**：
```bash
cargo build --features c_api
```

**变更后**：
```bash
cargo build --features embedded  # embedded 特性自动包含 c_api
```

## 优势

1. **架构更清晰**：所有嵌入相关的代码集中在一个目录
2. **依赖关系明确**：C API 明确依赖于 embedded API
3. **特性管理简化**：不需要单独管理 c_api 特性
4. **代码复用**：C API 可以直接使用 embedded 的类型和函数
5. **文档组织**：所有嵌入相关的文档集中管理

## 兼容性

- ✅ 不影响现有 embedded API 的使用
- ✅ 不影响 server API 的使用
- ✅ C API 功能完全保留
- ✅ 头文件生成路径不变

## 迁移指南

### 对于现有用户

如果已经在使用 `c_api` 特性，需要：

1. 更新 Cargo.toml：
   ```toml
   # 旧版本
   features = ["embedded", "c_api"]
   
   # 新版本
   features = ["embedded"]
   ```

2. 更新构建命令：
   ```bash
   # 旧版本
   cargo build --features c_api
   
   # 新版本
   cargo build --features embedded
   ```

### 对于开发者

如果正在开发 C API 功能：

1. 更新导入路径：
   ```rust
   // 旧版本
   use graphdb::api::c_api::*;
   
   // 新版本
   use graphdb::api::embedded::c_api::*;
   ```

2. 更新特性检查：
   ```rust
   // 旧版本
   #[cfg(feature = "c_api")]
   
   // 新版本
   #[cfg(feature = "embedded/c_api")]
   ```

## 总结

通过将 C API 整合到 embedded 模块，我们实现了：

1. ✅ 更清晰的代码组织
2. ✅ 更简单的特性管理
3. ✅ 更合理的依赖关系
4. ✅ 更好的代码复用

这个调整使得 GraphDB 的架构更加合理，同时也为后续的开发和维护提供了更好的基础。
