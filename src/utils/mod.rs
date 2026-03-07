// 工具模块 - 仅用于导出各个子模块，不包含具体实现

// ID生成模块
pub mod id_gen;
pub use id_gen::{generate_id, is_valid_id, IdGenerator, INVALID_ID};

// 对象池模块
pub mod object_pool;
pub use object_pool::ObjectPool;

// 重试机制模块
pub mod retry;
pub use retry::{retry_with_backoff, retry_with_strategy, RetryConfig, RetryStrategy};

// 日志模块
pub mod logging;
pub use logging::{
    init as init_logging, is_initialized as is_logging_initialized, shutdown as shutdown_logging,
};

// 错误转换模块
pub mod error_convert;
pub use error_convert::{ErrorConvert, ErrorContext, ResultErrorConvert};
