//! 通用基础设施模块
//!
//! 这个模块包含了所有通用的基础设施代码，包括：
//! - 基础工具和ID生成
//! - 内存管理
//! - 线程和进程管理
//! - 文件系统操作
//! - 日志系统
//! - 字符集处理

pub mod charset;
pub mod fs;
pub mod id;
pub mod log;
pub mod memory;
pub mod process;
pub mod thread;

// 重新导出常用的类型和函数，方便其他模块使用
pub use id::*;
pub use charset::*;
pub use fs::*;
pub use log::*;
pub use memory::*;
pub use process::*;
pub use thread::*;
