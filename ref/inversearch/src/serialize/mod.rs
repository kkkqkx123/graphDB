//! 序列化模块
//!
//! 提供索引和文档的导入导出功能，支持多种格式（JSON、Binary、MessagePack、CBOR）
//! 和压缩算法（Zstd、Lz4）。
//!
//! # 模块结构
//!
//! - `types`: 核心类型定义（配置、数据结构）
//! - `format`: 格式处理（JSON/Binary/MessagePack/CBOR）
//! - `compression`: 压缩/解压缩工具
//! - `index`: Index 的序列化实现
//! - `document`: Document 的序列化实现
//! - `async`: 异步序列化包装器
//! - `chunked`: 分块序列化处理

// 核心类型定义 - 数据的唯一来源
pub mod types;

// 格式处理
pub mod format;

// 压缩工具
pub mod compression;

// Index 序列化实现
pub mod index;

// Document 序列化实现
pub mod document;

// 异步序列化
pub mod r#async;

// 分块序列化
pub mod chunked;

// 重新导出常用类型
pub use types::*;
pub use format::*;
pub use compression::*;
pub use r#async::{AsyncSerializer, AsyncDocumentSerializer};
pub use chunked::{ChunkedSerializer, ChunkDataProvider};
