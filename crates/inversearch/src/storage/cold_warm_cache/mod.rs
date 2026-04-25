//! 冷热缓存存储模块
//!
//! 提供三层缓存架构 + WAL 的高性能持久化存储
//!
//! ## 架构
//!
//! ```text
//! ColdWarmCache
//! ├── Hot Cache (L1) - 内存，最热数据，LRU 淘汰
//! ├── Warm Cache (L2) - 内存映射文件，频繁访问
//! └── Cold Storage   - 磁盘文件，冷数据，压缩存储
//! ```
//!
//! ## 模块结构
//!
//! ```text
//! cold_warm_cache/
//! ├── mod.rs              # 主模块
//! ├── config.rs           # 配置结构
//! ├── manager.rs          # ColdWarmCacheManager 核心实现
//! └── background.rs       # 后台任务
//! ```

pub mod background;
pub mod config;
pub mod manager;

pub use config::{ColdWarmCacheConfig, WALConfig};
pub use manager::{CacheStats, ColdWarmCacheManager, IndexData, WALEntry, WALManager};
