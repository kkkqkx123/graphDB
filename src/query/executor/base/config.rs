//! 执行器配置结构体
//!
//! 本模块定义各种执行器的配置结构体，用于减少构造函数的参数数量

use std::sync::Arc;

use crate::core::Expression;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// 通用执行器配置
///
/// 封装所有执行器共有的基础配置
pub struct ExecutorConfig<S: StorageClient> {
    pub id: i64,
    pub storage: Arc<Mutex<S>>,
    pub expr_context: Arc<ExpressionAnalysisContext>,
}

impl<S: StorageClient> ExecutorConfig<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            id,
            storage,
            expr_context,
        }
    }
}

/// 索引扫描执行器配置
pub struct IndexScanConfig {
    pub space_id: u64,
    pub tag_id: i32,
    pub index_id: i32,
    pub scan_type: String,
    pub scan_limits: Vec<crate::query::planning::plan::core::nodes::access::IndexLimit>,
    pub filter: Option<Expression>,
    pub return_columns: Vec<String>,
    pub limit: Option<usize>,
    pub is_edge: bool,
}

/// 路径执行器配置
pub struct PathConfig {
    pub start_vertex: crate::core::Value,
    pub end_vertex: Option<crate::core::Value>,
    pub max_hops: usize,
    pub edge_types: Option<Vec<String>>,
    pub direction: crate::core::types::EdgeDirection,
}

/// BFS 最短路径算法配置
pub struct BfsShortestConfig {
    pub steps: usize,
    pub direction: crate::core::types::EdgeDirection,
    pub edge_types: Option<Vec<String>>,
}

/// 多起点最短路径配置
pub struct MultiShortestPathConfig {
    pub start_vids: Vec<crate::core::Value>,
    pub direction: crate::core::types::EdgeDirection,
    pub edge_types: Option<Vec<String>>,
    pub max_steps: usize,
}

/// 所有路径配置
pub struct AllPathsConfig {
    pub left_start_ids: Vec<crate::core::Value>,
    pub right_start_ids: Vec<crate::core::Value>,
    pub max_hops: usize,
    pub edge_types: Option<Vec<String>>,
    pub direction: crate::core::types::EdgeDirection,
}

/// 最短路径配置
pub struct ShortestPathConfig {
    pub start_vertex_ids: Vec<crate::core::Value>,
    pub direction: crate::core::types::EdgeDirection,
    pub edge_types: Option<Vec<String>>,
}

/// 连接执行器配置
pub struct JoinConfig {
    pub left_var: String,
    pub right_var: String,
    pub hash_keys: Vec<Expression>,
    pub probe_keys: Vec<Expression>,
    pub col_names: Vec<String>,
}

/// 带描述的连接执行器配置
pub struct JoinConfigWithDesc {
    pub left_var: String,
    pub right_var: String,
    pub hash_keys: Vec<Expression>,
    pub probe_keys: Vec<Expression>,
    pub col_names: Vec<String>,
    pub description: String,
}

/// 循环执行器配置
pub struct LoopConfig {
    pub loop_var: String,
    pub loop_condition: Expression,
}

/// 附加顶点执行器配置
pub struct AppendVerticesConfig {
    pub input_var: String,
    pub src_expression: Expression,
    pub v_filter: Option<Expression>,
    pub col_names: Vec<String>,
    pub dedup: bool,
    pub need_fetch_prop: bool,
}

/// 模式应用执行器配置
pub struct PatternApplyConfig {
    pub left_input_var: String,
    pub right_input_var: String,
    pub key_cols: Vec<Expression>,
    pub col_names: Vec<String>,
    pub is_anti_predicate: bool,
}

/// 汇总应用执行器配置
pub struct RollupApplyConfig {
    pub left_input_var: String,
    pub right_input_var: String,
    pub compare_cols: Vec<Expression>,
    pub collect_col: Expression,
    pub col_names: Vec<String>,
}
