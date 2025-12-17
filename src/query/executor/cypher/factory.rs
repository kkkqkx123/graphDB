//! Cypher执行器工厂
//!
//! 负责创建和管理不同类型的Cypher执行器，
//! 基于nebula-graph的工厂模式设计

use crate::query::executor::cypher::base::CypherExecutor;
use crate::query::executor::cypher::{CypherExecutorError, CypherExecutorTrait};
use crate::query::parser::cypher::ast::statements::CypherStatement;
use crate::storage::StorageEngine;
use std::sync::{Arc, Mutex};

/// Cypher执行器工厂
///
/// 负责根据查询类型创建合适的执行器实例
#[derive(Debug)]
pub struct CypherExecutorFactory<S: StorageEngine> {
    /// 存储引擎引用
    storage: Arc<Mutex<S>>,
    /// 执行器ID计数器
    next_id: usize,
}

impl<S: StorageEngine + Send + 'static> CypherExecutorFactory<S> {
    /// 创建新的执行器工厂
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            next_id: 1,
        }
    }

    /// 创建通用Cypher执行器
    pub fn create_executor(&mut self) -> Result<CypherExecutor<S>, CypherExecutorError> {
        let id = self.next_id;
        self.next_id += 1;

        let executor = CypherExecutor::with_description(
            id,
            format!("CypherExecutor-{}", id),
            "通用Cypher查询执行器".to_string(),
            self.storage.clone(),
        );

        Ok(executor)
    }

    /// 根据语句类型创建专用执行器
    pub fn create_executor_for_statement(
        &mut self,
        statement: &CypherStatement,
    ) -> Result<Box<dyn CypherExecutorTrait<S>>, CypherExecutorError> {
        let id = self.next_id;
        self.next_id += 1;

        match statement {
            CypherStatement::Match(_) => {
                let executor = CypherExecutor::with_description(
                    id,
                    format!("MatchExecutor-{}", id),
                    "MATCH语句执行器 - 用于图模式匹配".to_string(),
                    self.storage.clone(),
                );
                Ok(Box::new(executor))
            }
            CypherStatement::Create(_) => {
                let executor = CypherExecutor::with_description(
                    id,
                    format!("CreateExecutor-{}", id),
                    "CREATE语句执行器 - 用于创建节点和关系".to_string(),
                    self.storage.clone(),
                );
                Ok(Box::new(executor))
            }
            CypherStatement::Delete(_) => {
                let executor = CypherExecutor::with_description(
                    id,
                    format!("DeleteExecutor-{}", id),
                    "DELETE语句执行器 - 用于删除节点和关系".to_string(),
                    self.storage.clone(),
                );
                Ok(Box::new(executor))
            }
            CypherStatement::Return(_) => {
                let executor = CypherExecutor::with_description(
                    id,
                    format!("ReturnExecutor-{}", id),
                    "RETURN语句执行器 - 用于返回查询结果".to_string(),
                    self.storage.clone(),
                );
                Ok(Box::new(executor))
            }
            CypherStatement::Set(_) => {
                let executor = CypherExecutor::with_description(
                    id,
                    format!("SetExecutor-{}", id),
                    "SET语句执行器 - 用于设置属性值".to_string(),
                    self.storage.clone(),
                );
                Ok(Box::new(executor))
            }
            CypherStatement::Where(_) => {
                let executor = CypherExecutor::with_description(
                    id,
                    format!("WhereExecutor-{}", id),
                    "WHERE语句执行器 - 用于条件过滤".to_string(),
                    self.storage.clone(),
                );
                Ok(Box::new(executor))
            }
            _ => {
                // 对于其他语句类型，使用通用执行器
                let executor = CypherExecutor::with_description(
                    id,
                    format!("CypherExecutor-{}", id),
                    format!("处理{}语句的执行器", statement.statement_type()),
                    self.storage.clone(),
                );
                Ok(Box::new(executor))
            }
        }
    }

    /// 创建执行器链
    pub fn create_executor_chain(
        &mut self,
        statements: &[CypherStatement],
    ) -> Result<Vec<Box<dyn CypherExecutorTrait<S>>>, CypherExecutorError> {
        let mut executors = Vec::new();

        for statement in statements {
            let executor = self.create_executor_for_statement(statement)?;
            executors.push(executor);
        }

        Ok(executors)
    }

    /// 获取下一个执行器ID
    pub fn next_id(&self) -> usize {
        self.next_id
    }

    /// 重置ID计数器
    pub fn reset_id_counter(&mut self) {
        self.next_id = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::cypher::ast::clauses::{MatchClause, ReturnClause};
    use crate::query::parser::cypher::ast::statements::CypherStatement;

    // 测试代码已注释，等待 MemoryStorageEngine 实现
    // #[tokio::test]
    // async fn test_factory_creation() {
    //     let storage = Arc::new(Mutex::new(MemoryStorageEngine::new()));
    //     let factory = CypherExecutorFactory::new(storage);
    //     assert_eq!(factory.next_id(), 1);
    // }

    // #[tokio::test]
    // async fn test_reset_id_counter() {
    //     let storage = Arc::new(Mutex::new(MemoryStorageEngine::new()));
    //     let mut factory = CypherExecutorFactory::new(storage);
    //     factory.create_executor().unwrap();
    //     factory.create_executor().unwrap();
    //     assert_eq!(factory.next_id(), 3);
    //     factory.reset_id_counter();
    //     assert_eq!(factory.next_id(), 1);
    // }
}
