//! 优化规则模块
//!
//! 所有优化规则按功能分类组织，每个规则独立一个文件

// 宏定义
pub mod macros;

// 谓词下推规则
pub mod predicate_pushdown;

// 消除优化规则
pub mod elimination;

// 操作合并规则
pub mod merge;

// LIMIT下推规则
pub mod limit_pushdown;

// 索引优化规则
pub mod index;

// 扫描优化规则
pub mod scan;

// 投影下推规则
pub mod projection_pushdown;

// 连接优化规则
pub mod join;

// 聚合相关规则
pub mod aggregate;

// 转换规则
pub mod transformation;

// 统一导出所有规则，保持向后兼容
pub use predicate_pushdown::*;
pub use elimination::*;
pub use merge::*;
pub use limit_pushdown::*;
pub use index::*;
pub use scan::*;
pub use projection_pushdown::*;
pub use join::*;
pub use aggregate::*;
pub use transformation::*;
