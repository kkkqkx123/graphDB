//! 缓存实现模块
//!
//! 提供各种缓存策略的具体实现

pub mod adaptive;
pub mod fifo;
pub mod lfu;
pub mod lru;
pub mod stats_wrapper;
pub mod ttl;
pub mod unbounded;

// 重新导出主要的缓存实现
pub use adaptive::{AdaptiveCache, ConcurrentAdaptiveCache};
pub use fifo::{ConcurrentFifoCache, FifoCache};
pub use lfu::{ConcurrentLfuCache, LfuCache};
pub use lru::{ConcurrentLruCache, LruCache};
pub use stats_wrapper::StatsCacheWrapper;
pub use ttl::{ConcurrentTtlCache, TtlCache, TtlEntry};
pub use unbounded::{ConcurrentUnboundedCache, UnboundedCache};
