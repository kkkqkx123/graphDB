//! 安全验证器
//!
//! 负责验证执行器的安全性配置

use crate::core::error::QueryError;
use crate::storage::StorageClient;
use std::sync::Arc;

/// 执行器安全配置
#[derive(Debug, Clone)]
pub struct ExecutorSafetyConfig {
    /// 最大递归深度
    pub max_recursion_depth: usize,
    /// 最大循环迭代次数
    pub max_loop_iterations: usize,
    /// 是否启用递归检测
    pub enable_recursion_detection: bool,
    /// 最大执行器数量
    pub max_executor_count: usize,
}

impl Default for ExecutorSafetyConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: 100,
            max_loop_iterations: 10000,
            enable_recursion_detection: true,
            max_executor_count: 1000,
        }
    }
}

/// 安全验证器
pub struct SafetyValidator<S: StorageClient + 'static> {
    config: ExecutorSafetyConfig,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + 'static> SafetyValidator<S> {
    /// 创建新的安全验证器
    pub fn new(config: ExecutorSafetyConfig) -> Self {
        Self {
            config,
            _phantom: std::marker::PhantomData,
        }
    }

    /// 获取配置
    pub fn config(&self) -> &ExecutorSafetyConfig {
        &self.config
    }

    /// 验证扩展配置
    pub fn validate_expand_config(&self, step_limit: usize) -> Result<(), QueryError> {
        if step_limit > self.config.max_recursion_depth {
            return Err(QueryError::ExecutionError(format!(
                "扩展步数限制 {} 超过最大递归深度 {}",
                step_limit, self.config.max_recursion_depth
            )));
        }
        Ok(())
    }

    /// 验证最短路径配置
    pub fn validate_shortest_path_config(&self, max_step: usize) -> Result<(), QueryError> {
        if max_step > self.config.max_recursion_depth {
            return Err(QueryError::ExecutionError(format!(
                "最短路径最大步数 {} 超过最大递归深度 {}",
                max_step, self.config.max_recursion_depth
            )));
        }
        Ok(())
    }
}
