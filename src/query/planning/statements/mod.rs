//! Statement-level planner
//!
//! A planner implementation that includes all statements for graph databases
//! All statement types supported by Cypher and NGQL are available.
//!
//! ## Architecture Description
//!
//! Adopts a three-layer architecture design:
//! `Planner trait`: The basic interface for planners.
//! `StatementPlanner` trait: A statement-level planner that processes entire statements.
//! `ClausePlanner` trait: A clause-level planner that processes individual clauses.

pub mod clauses;
pub mod paths;
pub mod seeks;

pub mod match_statement_planner;
pub mod statement_planner;

pub mod create_planner;
pub mod delete_planner;
pub mod fetch_edges_planner;
pub mod fetch_vertices_planner;
pub mod go_planner;
pub mod group_by_planner;
pub mod insert_planner;
pub mod lookup_planner;
pub mod maintain_planner;
pub mod merge_planner;
pub mod path_planner;
pub mod remove_planner;
pub mod return_planner;
pub mod set_operation_planner;
pub mod subgraph_planner;
pub mod update_planner;
pub mod use_planner;
pub mod user_management_planner;
pub mod with_planner;
pub mod yield_planner;

// Re-export the Statement Planner module
pub use match_statement_planner::MatchStatementPlanner;
pub use statement_planner::{ClausePlanner, StatementPlanner};
