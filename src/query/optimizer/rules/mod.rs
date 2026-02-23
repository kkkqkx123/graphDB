//! 优化规则模块
//!
//! 该模块包含基于代价的优化规则，需要统计信息和代价计算来选择最优方案。
//! 启发式优化规则已迁移到 planner/rewrite 模块。
//!
//! # 规则分类
//!
//! ## 索引优化规则 (index)
//! 基于代价选择最优索引扫描方案：
//! - `IndexScanRule` - 索引扫描选择
//! - `EdgeIndexFullScanRule` - 边索引全扫描
//! - `TagIndexFullScanRule` - 标签索引全扫描
//! - `IndexCoveringScanRule` - 覆盖索引扫描
//! - `OptimizeEdgeIndexScanByFilterRule` - 基于过滤优化边索引扫描
//! - `UnionAllEdgeIndexScanRule` - 联合边索引扫描
//! - `UnionAllTagIndexScanRule` - 联合标签索引扫描
//!
//! ## 扫描优化规则 (scan)
//! 基于代价选择最优扫描方式：
//! - `IndexFullScanRule` - 索引全扫描 vs 全表扫描选择
//! - `ScanWithFilterOptimizationRule` - 带过滤条件的扫描优化
//!
//! ## 连接优化规则 (join)
//! 基于代价选择最优连接算法和顺序：
//! - `JoinOptimizationRule` - 连接算法选择（哈希连接、嵌套循环连接等）
//! - 连接顺序优化
//!
//! ## 转换规则 (transformation)
//! 基于代价的计划转换：
//! - `TopNRule` - TopN 优化
//! - `OptimizeSetOperationInputOrderRule` - 集合操作输入顺序优化

// 宏定义
pub mod macros;

// 索引优化规则（基于代价）
pub mod index;

// 扫描优化规则（基于代价）
pub mod scan;

// 连接优化规则（基于代价）
pub mod join;

// 转换规则（基于代价）
pub mod transformation;

// 统一导出所有基于代价的规则
pub use index::*;
pub use scan::*;
pub use join::*;
pub use transformation::*;

// 注意：启发式规则已迁移到 planner/rewrite 模块
// 如需使用启发式规则，请直接从 planner::rewrite 导入
// pub use crate::query::planner::rewrite::*;
