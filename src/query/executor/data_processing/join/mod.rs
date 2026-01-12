//! JOIN 执行器模块
//!
//! 包含所有 JOIN 操作相关的执行器，包括：
//! - InnerJoin（内连接）
//! - LeftJoin（左外连接）
//! - CrossJoin/CartesianProduct（笛卡尔积）
//!
//! 基于nebula-graph的join实现，使用哈希连接算法优化性能

use crate::core::Expression;

pub mod base_join;
pub mod cross_join;
pub mod full_outer_join;
pub mod hash_table;
pub mod inner_join;
pub mod join_key_evaluator;
pub mod left_join;
pub mod right_join;

// 重新导出主要类型
pub use base_join::{
    BaseJoinExecutor, CartesianProductOperation, InnerJoinOperation, JoinOperation,
    LeftJoinOperation,
};
pub use cross_join::CrossJoinExecutor;
pub use full_outer_join::FullOuterJoinExecutor;
pub use hash_table::{
    HashTableBuilder, HashTableProbe, HashTableStats, JoinKey, MultiKeyHashTable,
    SingleKeyHashTable,
};
pub use inner_join::{HashInnerJoinExecutor, InnerJoinExecutor};
pub use join_key_evaluator::JoinKeyEvaluator;
pub use left_join::{HashLeftJoinExecutor, LeftJoinExecutor};
pub use right_join::RightJoinExecutor;

/// Join操作的类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    /// 内连接
    Inner,
    /// 左外连接
    Left,
    /// 右外连接
    Right,
    /// 全外连接
    Full,
    /// 笛卡尔积
    Cross,
}

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

    /// 创建笛卡尔积配置
    pub fn cross_join(_input_vars: Vec<String>, output_columns: Vec<String>) -> Self {
        Self {
            join_type: JoinType::Cross,
            left_var: String::new(),
            right_var: String::new(),
            left_keys: Vec::new(),
            right_keys: Vec::new(),
            output_columns,
            enable_parallel: false,
            memory_limit: None,
        }
    }

    /// 设置并行处理
    pub fn with_parallel(mut self, enable: bool) -> Self {
        self.enable_parallel = enable;
        self
    }

    /// 设置内存限制
    pub fn with_memory_limit(mut self, limit: usize) -> Self {
        self.memory_limit = Some(limit);
        self
    }
}

/// Join执行器工厂
pub struct JoinExecutorFactory;

impl JoinExecutorFactory {
    /// 根据配置创建相应的join执行器
    pub fn create_executor<S: crate::storage::StorageEngine + Send + 'static>(
        id: i64,
        storage: std::sync::Arc<std::sync::Mutex<S>>,
        config: JoinConfig,
    ) -> Result<Box<dyn crate::query::executor::traits::Executor<S>>, crate::query::QueryError>
    {
        match config.join_type {
            JoinType::Inner => {
                let hash_keys: Vec<Expression> = config
                    .left_keys
                    .into_iter()
                    .map(Expression::Variable)
                    .collect();
                let probe_keys: Vec<Expression> = config
                    .right_keys
                    .into_iter()
                    .map(Expression::Variable)
                    .collect();

                if config.enable_parallel {
                    Ok(Box::new(HashInnerJoinExecutor::new(
                        id,
                        storage,
                        config.left_var,
                        config.right_var,
                        hash_keys,
                        probe_keys,
                        config.output_columns,
                    )))
                } else {
                    Ok(Box::new(InnerJoinExecutor::new(
                        id,
                        storage,
                        config.left_var,
                        config.right_var,
                        hash_keys,
                        probe_keys,
                        config.output_columns,
                    )))
                }
            }
            JoinType::Left => {
                let hash_keys: Vec<Expression> = config
                    .left_keys
                    .into_iter()
                    .map(Expression::Variable)
                    .collect();
                let probe_keys: Vec<Expression> = config
                    .right_keys
                    .into_iter()
                    .map(Expression::Variable)
                    .collect();

                if config.enable_parallel {
                    Ok(Box::new(HashLeftJoinExecutor::new(
                        id,
                        storage,
                        config.left_var,
                        config.right_var,
                        hash_keys,
                        probe_keys,
                        config.output_columns,
                    )))
                } else {
                    Ok(Box::new(LeftJoinExecutor::new(
                        id,
                        storage,
                        config.left_var,
                        config.right_var,
                        hash_keys,
                        probe_keys,
                        config.output_columns,
                    )))
                }
            }
            JoinType::Right => Ok(Box::new(RightJoinExecutor::new(
                id,
                storage,
                config.left_var,
                config.right_var,
                config.left_keys,
                config.right_keys,
                config.output_columns,
            ))),
            JoinType::Full => Ok(Box::new(FullOuterJoinExecutor::new(
                id,
                storage,
                config.left_var,
                config.right_var,
                config.left_keys,
                config.right_keys,
                config.output_columns,
            ))),
            JoinType::Cross => Ok(Box::new(CrossJoinExecutor::new(
                id,
                storage,
                vec![config.left_var, config.right_var],
                config.output_columns,
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Direction, Value};
    use crate::storage::test_mock::MockStorage;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_join_config_creation() {
        let config = JoinConfig::inner_join(
            "left".to_string(),
            "right".to_string(),
            vec!["0".to_string()],
            vec!["0".to_string()],
            vec!["id".to_string(), "name".to_string()],
        )
        .with_parallel(true);

        assert_eq!(config.join_type, JoinType::Inner);
        assert_eq!(config.left_var, "left");
        assert_eq!(config.right_var, "right");
        assert_eq!(config.enable_parallel, true);
    }

    #[test]
    fn test_join_executor_factory() {
        let storage = Arc::new(Mutex::new(MockStorage));

        let config = JoinConfig::inner_join(
            "left".to_string(),
            "right".to_string(),
            vec!["0".to_string()],
            vec!["0".to_string()],
            vec!["id".to_string(), "name".to_string()],
        );

        let executor = JoinExecutorFactory::create_executor(1, storage, config);
        assert!(executor.is_ok());
    }

    #[test]
    fn test_cross_join_config() {
        let config = JoinConfig::cross_join(
            vec!["table1".to_string(), "table2".to_string()],
            vec!["a".to_string(), "b".to_string()],
        );

        assert_eq!(config.join_type, JoinType::Cross);
        assert!(config.left_var.is_empty());
        assert!(config.right_var.is_empty());
    }
}
