//! 代价计算模块
//!
//! 提供查询优化器所需的代价计算功能
//!
//! ## 模块结构
//!
//! - `calculator` - 代价计算器，计算各种操作的代价
//! - `selectivity` - 选择性估计器，估算查询条件的选择性
//! - `config` - 代价模型配置
//! - `assigner` - 代价赋值器，为执行计划节点赋值代价

pub mod calculator;
pub mod selectivity;
pub mod config;
pub mod assigner;

pub use calculator::CostCalculator;
pub use selectivity::SelectivityEstimator;
pub use config::CostModelConfig;
pub use assigner::{CostAssigner, CostError};
