// 工具模块 - 仅用于导出各个子模块，不包含具体实现

// 对象池模块
pub mod object_pool;
pub use object_pool::ObjectPool;

// 错误处理辅助函数模块
pub mod error_handling;
pub use error_handling::{expect_arc_mut, safe_lock, safe_read, safe_write};

// 重试机制模块
pub mod retry;
pub use retry::{RetryConfig, RetryStrategy, retry_with_backoff, retry_with_strategy};

// 宏从 crate 根目录导出（#[macro_export] 会导出到 crate 根）
pub use crate::{db_assert, db_return_if_err};
