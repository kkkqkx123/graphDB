//! 投影下推优化规则
//!
//! 这些规则负责将投影操作推向数据源，减少数据传输量

pub mod push_project_down_edge_index_scan;
pub mod push_project_down_get_edges;
pub mod push_project_down_get_neighbors;
pub mod push_project_down_get_vertices;
pub mod push_project_down_scan_edges;
pub mod push_project_down_scan_vertices;

pub use push_project_down_edge_index_scan::PushProjectDownEdgeIndexScanRule;
pub use push_project_down_get_edges::PushProjectDownGetEdgesRule;
pub use push_project_down_get_neighbors::PushProjectDownGetNeighborsRule;
pub use push_project_down_get_vertices::PushProjectDownGetVerticesRule;
pub use push_project_down_scan_edges::PushProjectDownScanEdgesRule;
pub use push_project_down_scan_vertices::PushProjectDownScanVerticesRule;
