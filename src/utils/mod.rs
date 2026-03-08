// 工具模块 - 仅用于导出各个子模块，不包含具体实现

// ID生成模块
pub mod id_gen;
pub use id_gen::{generate_id, IdGenerator};

// 日志模块
pub mod logging;
pub use logging::{
    init as init_logging, is_initialized as is_logging_initialized, shutdown as shutdown_logging,
};
