# Rust 项目结构最佳实践

## 概述

本文档记录了在 GraphDB 项目中遇到的模块导入冲突问题及其解决方案，以及相关的 Rust 项目结构最佳实践。

## 问题描述

在 GraphDB 项目中，我们遇到了以下编译错误：

```
error[E0433]: failed to resolve: unresolved import
 --> src\core\query_context.rs:6:12
  |
6 | use crate::graph::utils::IdGenerator;
  |^^^^^^^^^^
  |
  | unresolved import
  | help: a similar path exists: `graphdb::graph`

error[E0433]: failed to resolve: unresolved import
 --> src\query\validator\match_validator.rs:5:12
  |
5 | use crate::graph::expression::expr_type::{Expression, ExpressionKind};
  |^^^^^^^^^^
  |
  | unresolved import
  | help: a similar path exists: `graphdb::graph`
```

## 根本原因

问题的根本原因是项目结构设计不当：

1. **模块定义冲突**：
   - `lib.rs` 定义了库的模块结构
   - `main.rs` 重新定义了相同的模块，导致在编译二进制目标时产生冲突

2. **编译目标差异**：
   - 当编译库目标时，使用 `lib.rs` 中的模块定义
   - 当编译二进制目标时，使用 `main.rs` 中的模块定义
   - 这导致了不一致的模块可见性

## 解决方案

我们采用了标准的 Rust 项目结构解决方案：

### 1. 修改 main.rs

```rust
// 修改前
mod config;
mod core;
mod storage;
mod query;
mod api;
mod utils;

// 修改后
use graphdb::api;
```

### 2. 使用库模块

```rust
// 修改前
api::start_service(config).await?;
api::execute_query(&query).await?;

// 修改后（保持不变，但现在使用的是库中的模块）
api::start_service(config).await?;
api::execute_query(&query).await?;
```

## Rust 项目结构最佳实践

### 1. 分离库和二进制代码

- **`lib.rs`**：包含所有的库代码和模块定义
- **`main.rs`**：只作为应用程序的入口点，使用库提供的功能

### 2. 避免模块重复定义

不要在 `main.rs` 中重新定义 `lib.rs` 中已有的模块。如果需要在二进制目标中使用库功能，应该：

```rust
use your_crate_name::module_name;
```

### 3. 清晰的依赖关系

- 二进制目标依赖于库目标
- 库目标应该是自包含的，不依赖于二进制目标
- 这种结构使得库可以被其他项目重用

### 4. 模块组织原则

```
src/
├── lib.rs          # 库入口，定义所有模块
├── main.rs         # 二进制入口，使用库功能
├── bin/            # 其他二进制文件（可选）
│   └── other_tool.rs
├── modules/        # 功能模块
│   ├── mod.rs
│   ├── submodule1.rs
│   └── submodule2.rs
└── tests/          # 集成测试
    └── integration_tests.rs
```

### 5. 导入路径规范

- 在库内部使用 `crate::` 前缀引用模块
- 在外部使用库时，使用 `crate_name::` 前缀
- 避免使用相对路径导入，除非在同一个文件内

## 实际应用示例

### 正确的项目结构

```rust
// lib.rs
pub mod config;
pub mod core;
pub mod storage;
pub mod query;
pub mod api;
pub mod utils;

// 重新导出常用类型
pub use crate::core::{Value, Vertex, Edge};
pub use crate::storage::StorageEngine;
```

```rust
// main.rs
use clap::Parser;
use anyhow::Result;

// 导入库模块
use graphdb::api;

#[tokio::main]
async fn main() -> Result<()> {
    // 使用库提供的功能
    api::start_service("config.toml".to_string()).await?;
    Ok(())
}
```

```rust
// api/mod.rs
use crate::config::Config;
use crate::query::QueryExecutor;
use crate::storage::NativeStorage;

pub async fn start_service(config_path: String) -> Result<()> {
    // 实现服务启动逻辑
}
```

## 常见陷阱

### 1. 重复模块定义

```rust
// 错误：在 main.rs 中重复定义模块
mod config;  // 这会与 lib.rs 中的定义冲突

// 正确：导入库模块
use graphdb::config;
```

### 2. 循环依赖

避免库和二进制目标之间的循环依赖。库应该是独立的，二进制目标依赖于库。

### 3. 混合导入路径

在同一个文件中混用 `crate::` 和 `super::` 可能会导致混淆。建议在库内部统一使用 `crate::`。

## 总结

通过遵循这些最佳实践，我们可以：

1. **避免编译错误**：消除模块导入冲突
2. **提高代码可维护性**：清晰的项目结构使代码更易理解
3. **增强代码重用性**：库可以被其他项目轻松使用
4. **简化测试**：库和二进制目标可以独立测试

这种结构特别适合像 GraphDB 这样的项目，其中核心功能应该作为库提供，而命令行工具只是该库的一个使用者。