//! Cypher查询执行器模块
//!
//! 本模块包含所有Cypher查询语言的执行器实现，
//! 基于nebula-graph的执行器架构设计。

use crate::core::error::DBError;
use crate::query::executor::traits::ExecutionResult;
use crate::query::executor::Executor;
use crate::query::parser::cypher::ast::statements::CypherStatement;
use crate::storage::StorageEngine;
use async_trait::async_trait;

pub mod base;
pub mod clauses;
pub mod context;
pub mod expression_evaluator;
pub mod factory;

// 重新导出主要类型
pub use base::CypherExecutor;
pub use context::CypherExecutionContext;
pub use expression_evaluator::CypherExpressionEvaluator;
pub use factory::CypherExecutorFactory;

/// Cypher执行器特征
#[async_trait]
pub trait CypherExecutorTrait<S: StorageEngine>: Executor<S> {
    /// 执行Cypher语句
    async fn execute_cypher(
        &mut self,
        statement: CypherStatement,
    ) -> Result<ExecutionResult, DBError>;

    /// 批量执行Cypher语句
    async fn execute_cypher_batch(
        &mut self,
        statements: Vec<CypherStatement>,
    ) -> Result<Vec<ExecutionResult>, DBError> {
        let mut results = Vec::new();
        for statement in statements {
            let result = self.execute_cypher(statement).await?;
            results.push(result);
        }
        Ok(results)
    }
}

/// Cypher执行器错误类型
#[derive(Debug, thiserror::Error)]
pub enum CypherExecutorError {
    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("执行错误: {0}")]
    ExecutionError(String),

    #[error("不支持的Cypher语句: {0}")]
    UnsupportedStatement(String),

    #[error("上下文错误: {0}")]
    ContextError(String),

    #[error("存储错误: {0}")]
    StorageError(#[from] DBError),
}
