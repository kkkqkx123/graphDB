// 工具模块 - 仅用于导出各个子模块，不包含具体实现

// ID生成模块
pub mod id_gen;
pub use id_gen::{EPIdGenerator, IdGenerator, generate_id, is_valid_id, INVALID_ID};

// 对象池模块
pub mod object_pool;
pub use object_pool::ObjectPool;

// 错误处理辅助函数模块
pub mod error_handling;
pub use error_handling::{safe_lock, safe_read, safe_write};

// 重试机制模块
pub mod retry;
pub use retry::{RetryConfig, RetryStrategy, retry_with_backoff, retry_with_strategy};
