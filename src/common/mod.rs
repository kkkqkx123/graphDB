//! 通用基础设施模块
//!
//! 这个模块包含了所有通用的基础设施代码，包括：
//! - 基础工具和ID生成
//! - 内存管理
//! - 线程管理

pub mod id;
pub mod memory;
pub mod thread;

// 重新导出常用的类型和函数，方便其他模块使用
pub use id::*;
pub use memory::*;
pub use thread::*;
