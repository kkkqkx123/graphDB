//! 数据转换执行器模块
//!
//! 包含所有数据转换相关的执行器，包括：
//! - Assign（变量赋值）
//! - AppendVertices（追加顶点）
//! - Unwind（列表展开）
//! - PatternApply（模式匹配）
//! - RollUpApply（聚合操作）
//!
//! 对应 NebulaGraph 实现：
//! nebula-3.8.0/src/graph/executor/query/

// 变量赋值执行器
pub mod assign;
pub use assign::AssignExecutor;

// 列表展开执行器
pub mod unwind;
pub use unwind::UnwindExecutor;

// 追加顶点执行器
pub mod append_vertices;
pub use append_vertices::AppendVerticesExecutor;

// 模式匹配执行器
pub mod pattern_apply;
pub use pattern_apply::{PatternApplyExecutor, PatternType};

// 聚合操作执行器
pub mod rollup_apply;
pub use rollup_apply::RollUpApplyExecutor;
