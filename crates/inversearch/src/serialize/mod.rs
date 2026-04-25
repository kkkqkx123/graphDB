//! Serialization Module
//!
//! Provide indexing and document import/export functionality, supporting multiple formats (JSON, Binary, MessagePack, CBOR)
//! and compression algorithms (Zstd, Lz4).
//!
//! # Module Structure
//!
//! - `types`: 核心类型定义（配置、数据结构）
//! - `format`: 格式处理（JSON/Binary/MessagePack/CBOR）
//! - `compression`: 压缩/解压缩工具
//! - `index`: Index 的序列化实现
//! - `document`: Document 的序列化实现
//! - `async`: 异步序列化包装器
//! - `chunked`: 分块序列化处理

// Core Type Definition - Unique Source of Data
pub mod types;

// format processing
pub mod format;

// Compression tools
pub mod compression;

// Index Serialization Implementation
pub mod index;

// Document Serialization Implementation
pub mod document;

// asynchronous serialization
pub mod r#async;

// chunking serialization
pub mod chunked;

// Re-export common types
pub use chunked::{ChunkDataProvider, ChunkedSerializer};
pub use compression::*;
pub use format::*;
pub use r#async::{AsyncDocumentSerializer, AsyncSerializer};
pub use types::*;
