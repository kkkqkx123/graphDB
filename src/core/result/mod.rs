//! 执行结果模块 - 表示查询执行的结果
//! 对应原C++中的Result.h/cpp

pub mod memory_manager;
pub mod result_builder;
pub mod result_core;
pub mod result_iterator;

pub use memory_manager::*;
pub use result_builder::*;
pub use result_core::*;
pub use result_iterator::*;
