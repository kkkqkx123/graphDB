//! 索引优化规则
//!
//! 这些规则负责优化索引扫描操作，提高查询性能

pub mod optimize_edge_index_scan_by_filter;
pub mod edge_index_full_scan;
pub mod tag_index_full_scan;
pub mod index_scan;
pub mod union_all_edge_index_scan;
pub mod union_all_tag_index_scan;
pub mod index_covering_scan;

pub use optimize_edge_index_scan_by_filter::OptimizeEdgeIndexScanByFilterRule;
pub use edge_index_full_scan::EdgeIndexFullScanRule;
pub use tag_index_full_scan::TagIndexFullScanRule;
pub use index_scan::IndexScanRule;
pub use union_all_edge_index_scan::UnionAllEdgeIndexScanRule;
pub use union_all_tag_index_scan::UnionAllTagIndexScanRule;
pub use index_covering_scan::IndexCoveringScanRule;
