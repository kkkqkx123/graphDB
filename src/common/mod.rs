//! 通用基础设施模块
//!
//! 这个模块包含了所有通用的基础设施代码，包括：
//! - 基础工具和ID生成
//! - 时间处理
//! - 内存管理
//! - 线程和进程管理
//! - 网络工具
//! - 文件系统操作
//! - 日志系统
//! - 字符集处理

pub mod base;
pub mod charset;
pub mod fs;
pub mod log;
pub mod memory;
pub mod network;
pub mod process;
pub mod thread;
pub mod time;

// 重新导出常用的类型和函数，方便其他模块使用
pub use base::id::*;
pub use charset::*;
pub use fs::*;
pub use log::*;
pub use memory::*;
pub use network::*;
pub use process::*;
pub use thread::*;
pub use time::*;
