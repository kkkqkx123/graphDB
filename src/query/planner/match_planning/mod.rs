//! MATCH查询规划器模块
//! 处理Cypher MATCH语句的查询规划

// 子模块
pub mod clauses;
pub mod core;
pub mod match_planner;
pub mod paths;
pub mod seeks;
pub mod utils;

// 重新导出核心模块的主要类型
pub use core::cypher_clause_planner::CypherClausePlanner;
pub use core::match_clause_planner::MatchClausePlanner;
pub use match_planner::MatchPlanner;

// 重新导出新的核心接口
pub use core::cypher_clause_planner::{
    ClauseType, DataFlowManager, DataFlowNode, FlowDirection, PlanningContext, QueryInfo,
    VariableInfo,
};

// 重新导出路径模块的主要类型
pub use paths::match_path_planner::MatchPathPlanner;
pub use paths::shortest_path_planner::ShortestPathPlanner;

// 重新导出查找策略模块的主要类型
pub use seeks::index_seek::{IndexScanMetadata, IndexSeek, IndexSeekType};
pub use seeks::scan_seek::ScanSeek;
pub use seeks::seek_strategy::{SeekStrategy, SeekStrategySelector, SeekStrategyType};
pub use seeks::vertex_seek::{VertexSeek, VertexSeekType};

// 重新导出子句规划器模块的主要类型
pub use clauses::clause_planner::{BaseClausePlanner, ClausePlanner, ClausePlannerFactory};
pub use clauses::order_by_planner::OrderByClausePlanner;
pub use clauses::pagination_planner::PaginationPlanner;
pub use clauses::projection_planner::ProjectionPlanner;
pub use clauses::return_clause_planner::ReturnClausePlanner;
pub use clauses::unwind_planner::UnwindClausePlanner;
pub use clauses::where_clause_planner::WhereClausePlanner;
pub use clauses::with_clause_planner::WithClausePlanner;
pub use clauses::yield_planner::YieldClausePlanner;

// 重新导出辅助工具模块的主要类型
pub use utils::connection_strategy::{
    CartesianStrategy, ConnectionStrategy, ConnectionType, InnerJoinStrategy, LeftJoinStrategy,
    PatternApplyStrategy, RollUpApplyStrategy, SequentialStrategy, UnifiedConnector,
};
pub use utils::finder::{Finder, FinderResult};
