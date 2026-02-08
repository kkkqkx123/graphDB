//! 核心类型模块
//! 提供优化器所需的核心数据类型，包括代价模型、优化阶段和配置

pub mod cost;
pub mod config;

pub use cost::{Cost, Statistics, TableStats, ColumnStats, PlanNodeProperties};
pub use config::{OptimizationConfig, OptimizationStats};
pub use crate::query::core::OptimizationPhase;
