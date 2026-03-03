//! 核心查询模块
//!
//! 提供查询系统的基础类型定义和通用功能。

mod execution_state;
mod node_type;

pub use execution_state::{
    ExecutorState, LoopExecutionState, OptimizationPhase, OptimizationState, QueryExecutionState,
    RowStatus,
};
pub use node_type::{NodeCategory, NodeType, NodeTypeMapping};
