//! LIMIT下推优化规则
//!
//! 这些规则负责将LIMIT操作下推到计划树的底层，以减少数据处理量

pub mod push_limit_down_get_vertices;
pub mod push_limit_down_get_edges;
pub mod push_limit_down_scan_vertices;
pub mod push_limit_down_scan_edges;
pub mod push_limit_down_index_scan;

// 导出所有规则
pub use push_limit_down_get_vertices::PushLimitDownGetVerticesRule;
pub use push_limit_down_get_edges::PushLimitDownGetEdgesRule;
pub use push_limit_down_scan_vertices::PushLimitDownScanVerticesRule;
pub use push_limit_down_scan_edges::PushLimitDownScanEdgesRule;
pub use push_limit_down_index_scan::PushLimitDownIndexScanRule;
