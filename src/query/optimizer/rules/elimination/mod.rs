//! 消除优化规则
//!
//! 这些规则负责消除冗余的操作，如永真式过滤、无操作投影、不必要的去重等

pub mod eliminate_filter;
pub mod dedup_elimination;
pub mod remove_noop_project;
pub mod eliminate_append_vertices;
pub mod remove_append_vertices_below_join;
pub mod eliminate_row_collect;
pub mod eliminate_empty_set_operation;

// 导出所有规则
pub use eliminate_filter::EliminateFilterRule;
pub use dedup_elimination::DedupEliminationRule;
pub use remove_noop_project::RemoveNoopProjectRule;
pub use eliminate_append_vertices::EliminateAppendVerticesRule;
pub use remove_append_vertices_below_join::RemoveAppendVerticesBelowJoinRule;
pub use eliminate_row_collect::EliminateRowCollectRule;
pub use eliminate_empty_set_operation::EliminateEmptySetOperationRule;
