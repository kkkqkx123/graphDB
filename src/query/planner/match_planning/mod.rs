//! MATCH查询规划器模块
//! 处理Cypher MATCH语句的查询规划

// 子模块
pub mod core;
pub mod paths;
pub mod seeks;
pub mod clauses;
pub mod utils;
pub mod match_planner;

// 重新导出核心模块的主要类型
pub use match_planner::MatchPlanner;
pub use core::cypher_clause_planner::CypherClausePlanner;
pub use core::match_clause_planner::MatchClausePlanner;

// 重新导出新的核心接口
pub use core::cypher_clause_planner::{
    ClauseType, PlanningContext,
    FlowDirection, VariableRequirement, VariableProvider, VariableType, DataFlowValidator
};

// 重新导出路径模块的主要类型
pub use paths::match_path_planner::MatchPathPlanner;
pub use paths::shortest_path_planner::ShortestPathPlanner;

// 重新导出查找策略模块的主要类型
pub use seeks::seek_strategy::{SeekStrategy, SeekStrategyType, SeekStrategySelector};
pub use seeks::scan_seek::ScanSeek;
pub use seeks::index_seek::{IndexSeek, IndexSeekType, IndexScanMetadata};
pub use seeks::vertex_seek::{VertexSeek, VertexSeekType};

// 重新导出子句规划器模块的主要类型
pub use clauses::clause_planner::{ClausePlanner, BaseClausePlanner, ClausePlannerFactory};
pub use clauses::projection_planner::ProjectionPlanner;
pub use clauses::where_clause_planner::WhereClausePlanner;
pub use clauses::return_clause_planner::ReturnClausePlanner;
pub use clauses::with_clause_planner::WithClausePlanner;
pub use clauses::order_by_planner::OrderByClausePlanner;
pub use clauses::pagination_planner::PaginationPlanner;
pub use clauses::unwind_planner::UnwindClausePlanner;
pub use clauses::yield_planner::YieldClausePlanner;

// 重新导出辅助工具模块的主要类型
pub use utils::finder::{Finder, FinderResult};
pub use utils::node_factory::{create_start_node, create_nested_start_node, create_empty_node};
pub use utils::connection_strategy::{
    ConnectionType, ConnectionStrategy, UnifiedConnector,
    InnerJoinStrategy, LeftJoinStrategy, CartesianStrategy, SequentialStrategy,
    PatternApplyStrategy, RollUpApplyStrategy,
};

// 为了向后兼容，保留一些旧的导出（已废弃）
#[deprecated(note = "使用 Finder 替代")]
pub use utils::finder::Finder as ArgumentFinder;

#[deprecated(note = "使用 Finder 替代")]
pub use utils::finder::Finder as StartVidFinder;

#[deprecated(note = "使用 VertexSeek 替代")]
pub use seeks::vertex_seek::VertexSeek as VertexIdSeek;

#[deprecated(note = "使用 VertexSeek 替代")]
pub use seeks::vertex_seek::VertexSeek as VariableVertexIdSeek;

#[deprecated(note = "使用 IndexSeek 替代")]
pub use seeks::index_seek::IndexSeek as LabelIndexSeek;

#[deprecated(note = "使用 IndexSeek 替代")]
pub use seeks::index_seek::IndexSeek as PropIndexSeek;

#[deprecated(note = "使用 IndexSeek 替代")]
pub use seeks::index_seek::IndexSeek as VariablePropIndexSeek;