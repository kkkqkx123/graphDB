//! 结果处理执行器模块
//!
//! 包含所有与结果处理相关的执行器，这些执行器对查询结果进行最终的处理和优化
//!
//! 模块组织：
//! - `projection` - 列投影（SELECT 列）
//! - `aggregation` - 聚合函数（COUNT、SUM、AVG、MAX、MIN）
//! - `sorting` - 排序操作（ORDER BY）
//! - `limiting` - 结果限制（LIMIT、OFFSET）
//! - `dedup` - 去重操作（DISTINCT）
//! - `sampling` - 采样操作（SAMPLE）
//! - `topn` - 排序优化（TOP N）

// 列投影
pub mod projection;

// 聚合函数
pub mod aggregation;

// 排序
pub mod sorting;

// 结果限制
pub mod limiting;
pub use limiting::{LimitExecutor, OffsetExecutor};

// 去重
pub mod dedup;
pub use dedup::DistinctExecutor;

// 采样
pub mod sampling;
pub use sampling::SampleExecutor;

// TOP N 优化
pub mod topn;
pub use topn::TopNExecutor;
