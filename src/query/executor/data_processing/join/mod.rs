//! JOIN 执行器模块
//!
//! 包含所有 JOIN 操作相关的执行器，包括：
//! - InnerJoin（内连接）
//! - LeftJoin（左外连接）
//! - FullOuterJoin（全外连接）
//! - CrossJoin/CartesianProduct（笛卡尔积）
//!
//! 基于nebula-graph的join实现，使用哈希连接算法优化性能
//!
//! 注意：RightJoin 已被移除，因为可以用 LeftJoin 交换表顺序实现相同功能

pub mod base_join;
pub mod cross_join;
pub mod full_outer_join;
pub mod hash_table;
pub mod inner_join;
pub mod join_key_evaluator;
pub mod left_join;

// 重新导出主要类型
pub use base_join::BaseJoinExecutor;
pub use cross_join::CrossJoinExecutor;
pub use full_outer_join::FullOuterJoinExecutor;
pub use hash_table::{
    HashTableBuilder, HashTableProbe, HashTableStats, JoinKey, MultiKeyHashTable,
    SingleKeyHashTable,
};
pub use inner_join::{HashInnerJoinExecutor, InnerJoinExecutor};
pub use join_key_evaluator::JoinKeyEvaluator;
pub use left_join::{HashLeftJoinExecutor, LeftJoinExecutor};

// 从 core 模块导入 JoinType
pub use crate::core::types::JoinType;

/// Join操作的配置
#[derive(Debug, Clone)]
pub struct JoinConfig {
    /// Join类型
    pub join_type: JoinType,
    /// 左输入变量名
    pub left_var: String,
    /// 右输入变量名
    pub right_var: String,
    /// 连接键表达式列表（左表）
    pub left_keys: Vec<String>,
    /// 连接键表达式列表（右表）
    pub right_keys: Vec<String>,
    /// 输出列名
    pub output_columns: Vec<String>,
    /// 是否启用并行处理
    pub enable_parallel: bool,
    /// 内存限制（字节）
    pub memory_limit: Option<usize>,
}

impl JoinConfig {
    /// 创建内连接配置
    pub fn inner_join(
        left_var: String,
        right_var: String,
        left_keys: Vec<String>,
        right_keys: Vec<String>,
        output_columns: Vec<String>,
    ) -> Self {
        Self {
            join_type: JoinType::Inner,
            left_var,
            right_var,
            left_keys,
            right_keys,
            output_columns,
            enable_parallel: false,
            memory_limit: None,
        }
    }

    /// 创建左外连接配置
    pub fn left_join(
        left_var: String,
        right_var: String,
        left_keys: Vec<String>,
        right_keys: Vec<String>,
        output_columns: Vec<String>,
    ) -> Self {
        Self {
            join_type: JoinType::Left,
            left_var,
            right_var,
            left_keys,
            right_keys,
            output_columns,
            enable_parallel: false,
            memory_limit: None,
        }
    }
}
