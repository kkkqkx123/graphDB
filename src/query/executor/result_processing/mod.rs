//! 结果处理执行器模块
//!
//! 包含所有与结果处理相关的执行器，这些执行器对查询结果进行最终的处理和优化
//!
//! 模块组织：
//! - `projection` - 列投影（SELECT 列）
//! - `topn` - 排序优化（TOP N）

// 列投影
pub mod projection;

// TOP N 优化
pub mod topn;
pub use topn::TopNExecutor;
