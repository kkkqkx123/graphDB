//! 聚合相关规则
//!
//! 这些规则负责优化聚合操作

pub mod push_filter_down_aggregate;

pub use push_filter_down_aggregate::PushFilterDownAggregateRule;
