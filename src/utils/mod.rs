// 工具模块 - 仅用于导出各个子模块，不包含具体实现

// ID生成模块
pub mod id_gen;
pub use id_gen::{generate_id, is_valid_id, IdGenerator, INVALID_ID};

// 对象池模块
pub mod object_pool;
pub use object_pool::ObjectPool;

// 日志模块
pub mod logging;
pub use logging::{
    init as init_logging, is_initialized as is_logging_initialized, shutdown as shutdown_logging,
};
