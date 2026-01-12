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
    next_id: i64,
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
    pub fn next_id(&self) -> i64 {
        self.next_id
    }

    /// 重置ID计数器
    pub fn reset_id_counter(&mut self) {
        self.next_id = 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::config::test_config::test_config;
    use crate::query::executor::cypher::CypherExecutorFactory;
    use crate::query::executor::traits::Executor;
    use crate::query::parser::cypher::ast::clauses::*;
    use crate::query::parser::cypher::ast::expressions::Expression;
    use crate::query::parser::cypher::ast::statements::CypherStatement;
    use crate::storage::NativeStorage;
    use std::sync::{Arc, Mutex};

    /// 创建测试用的存储引擎
    fn create_test_storage() -> Arc<Mutex<NativeStorage>> {
        create_test_storage_with_name("factory_test_db")
    }

    /// 创建带有指定名称的测试存储引擎
    fn create_test_storage_with_name(name: &str) -> Arc<Mutex<NativeStorage>> {
        let config = test_config();
        // 使用时间戳和线程ID创建唯一的数据库名称，避免锁定冲突
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();
        let thread_id = std::thread::current().id();
        let db_name = format!("{}_{}_{:?}", name, timestamp, thread_id);
        let storage = NativeStorage::new(config.test_db_path(&db_name))
            .expect("Failed to create test storage");
        Arc::new(Mutex::new(storage))
    }

    /// 创建测试用的MATCH语句
    fn create_test_match_statement() -> CypherStatement {
        use crate::query::parser::cypher::ast::patterns::{NodePattern, Pattern, PatternPart};

        CypherStatement::Match(MatchClause {
            patterns: vec![Pattern {
                parts: vec![PatternPart {
                    node: NodePattern {
                        variable: Some("n".to_string()),
                        labels: vec!["Person".to_string()],
                        properties: None,
                    },
                    relationships: vec![],
                }],
            }],
            where_clause: None,
            optional: false,
        })
    }

    /// 创建测试用的CREATE语句
    fn create_test_create_statement() -> CypherStatement {
        use crate::query::parser::cypher::ast::patterns::{NodePattern, Pattern, PatternPart};

        CypherStatement::Create(CreateClause {
            patterns: vec![Pattern {
                parts: vec![PatternPart {
                    node: NodePattern {
                        variable: Some("n".to_string()),
                        labels: vec!["Person".to_string()],
                        properties: None,
                    },
                    relationships: vec![],
                }],
            }],
        })
    }

    /// 创建测试用的DELETE语句
    fn create_test_delete_statement() -> CypherStatement {
        CypherStatement::Delete(DeleteClause {
            expressions: vec![Expression::Variable("n".to_string())],
            detach: false,
        })
    }

    /// 创建测试用的RETURN语句
    fn create_test_return_statement() -> CypherStatement {
        CypherStatement::Return(ReturnClause {
            return_items: vec![],
            distinct: false,
            order_by: None,
            skip: None,
            limit: None,
        })
    }

    /// 创建测试用的SET语句
    fn create_test_set_statement() -> CypherStatement {
        CypherStatement::Set(SetClause { items: vec![] })
    }

    /// 创建测试用的WHERE语句
    fn create_test_where_statement() -> CypherStatement {
        CypherStatement::Where(WhereClause {
            expression: Expression::Variable("n".to_string()),
        })
    }

    #[tokio::test]
    async fn test_factory_creation() {
        let storage = create_test_storage_with_name("test_factory_creation");
        let factory = CypherExecutorFactory::new(storage);

        // 验证工厂初始状态
        assert_eq!(factory.next_id(), 1);
    }

    #[tokio::test]
    async fn test_create_executor() {
        let storage = create_test_storage_with_name("test_create_executor");
        let mut factory = CypherExecutorFactory::new(storage);

        // 创建第一个执行器
        let executor1 = factory
            .create_executor()
            .expect("Failed to create executor");
        assert_eq!(executor1.id(), 1);
        assert_eq!(executor1.name(), "CypherExecutor-1");
        assert_eq!(factory.next_id(), 2);

        // 创建第二个执行器
        let executor2 = factory
            .create_executor()
            .expect("Failed to create executor");
        assert_eq!(executor2.id(), 2);
        assert_eq!(executor2.name(), "CypherExecutor-2");
        assert_eq!(factory.next_id(), 3);
    }

    #[tokio::test]
    async fn test_create_executor_for_match_statement() {
        let storage = create_test_storage_with_name("test_create_executor_for_match_statement");
        let mut factory = CypherExecutorFactory::new(storage);
        let statement = create_test_match_statement();

        let executor = factory
            .create_executor_for_statement(&statement)
            .expect("Failed to create executor");

        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "MatchExecutor-1");
        assert_eq!(factory.next_id(), 2);
    }

    #[tokio::test]
    async fn test_create_executor_for_create_statement() {
        let storage = create_test_storage_with_name("test_create_executor_for_create_statement");
        let mut factory = CypherExecutorFactory::new(storage);
        let statement = create_test_create_statement();

        let executor = factory
            .create_executor_for_statement(&statement)
            .expect("Failed to create executor");

        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "CreateExecutor-1");
        assert_eq!(factory.next_id(), 2);
    }

    #[tokio::test]
    async fn test_create_executor_for_delete_statement() {
        let storage = create_test_storage_with_name("test_create_executor_for_delete_statement");
        let mut factory = CypherExecutorFactory::new(storage);
        let statement = create_test_delete_statement();

        let executor = factory
            .create_executor_for_statement(&statement)
            .expect("Failed to create executor");

        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "DeleteExecutor-1");
        assert_eq!(factory.next_id(), 2);
    }

    #[tokio::test]
    async fn test_create_executor_for_return_statement() {
        let storage = create_test_storage_with_name("test_create_executor_for_return_statement");
        let mut factory = CypherExecutorFactory::new(storage);
        let statement = create_test_return_statement();

        let executor = factory
            .create_executor_for_statement(&statement)
            .expect("Failed to create executor");

        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "ReturnExecutor-1");
        assert_eq!(factory.next_id(), 2);
    }

    #[tokio::test]
    async fn test_create_executor_for_set_statement() {
        let storage = create_test_storage_with_name("test_create_executor_for_set_statement");
        let mut factory = CypherExecutorFactory::new(storage);
        let statement = create_test_set_statement();

        let executor = factory
            .create_executor_for_statement(&statement)
            .expect("Failed to create executor");

        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "SetExecutor-1");
        assert_eq!(factory.next_id(), 2);
    }

    #[tokio::test]
    async fn test_create_executor_for_where_statement() {
        let storage = create_test_storage_with_name("test_create_executor_for_where_statement");
        let mut factory = CypherExecutorFactory::new(storage);
        let statement = create_test_where_statement();

        let executor = factory
            .create_executor_for_statement(&statement)
            .expect("Failed to create executor");

        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "WhereExecutor-1");
        assert_eq!(factory.next_id(), 2);
    }

    #[tokio::test]
    async fn test_create_executor_for_unsupported_statement() {
        let storage =
            create_test_storage_with_name("test_create_executor_for_unsupported_statement");
        let mut factory = CypherExecutorFactory::new(storage);

        // 创建一个不支持的语句类型（例如MERGE）
        use crate::query::parser::cypher::ast::patterns::{NodePattern, Pattern, PatternPart};

        let statement = CypherStatement::Merge(MergeClause {
            pattern: Pattern {
                parts: vec![PatternPart {
                    node: NodePattern {
                        variable: Some("n".to_string()),
                        labels: vec!["Person".to_string()],
                        properties: None,
                    },
                    relationships: vec![],
                }],
            },
            actions: vec![],
        });

        let executor = factory
            .create_executor_for_statement(&statement)
            .expect("Failed to create executor");

        assert_eq!(executor.id(), 1);
        assert_eq!(executor.name(), "CypherExecutor-1");
        assert!(executor.description().contains("MERGE"));
        assert_eq!(factory.next_id(), 2);
    }

    #[tokio::test]
    async fn test_create_executor_chain() {
        let storage = create_test_storage_with_name("test_create_executor_chain");
        let mut factory = CypherExecutorFactory::new(storage);

        let statements = vec![
            create_test_match_statement(),
            create_test_where_statement(),
            create_test_return_statement(),
        ];

        let executors = factory
            .create_executor_chain(&statements)
            .expect("Failed to create executor chain");

        assert_eq!(executors.len(), 3);
        assert_eq!(executors[0].id(), 1);
        assert_eq!(executors[0].name(), "MatchExecutor-1");
        assert_eq!(executors[1].id(), 2);
        assert_eq!(executors[1].name(), "WhereExecutor-2");
        assert_eq!(executors[2].id(), 3);
        assert_eq!(executors[2].name(), "ReturnExecutor-3");
        assert_eq!(factory.next_id(), 4);
    }

    #[tokio::test]
    async fn test_create_executor_chain_empty() {
        let storage = create_test_storage_with_name("test_create_executor_chain_empty");
        let mut factory = CypherExecutorFactory::new(storage);

        let statements: Vec<CypherStatement> = vec![];
        let executors = factory
            .create_executor_chain(&statements)
            .expect("Failed to create executor chain");

        assert_eq!(executors.len(), 0);
        assert_eq!(factory.next_id(), 1); // ID不应该增加
    }

    #[tokio::test]
    async fn test_reset_id_counter() {
        let storage = create_test_storage_with_name("test_reset_id_counter");
        let mut factory = CypherExecutorFactory::new(storage);

        // 创建几个执行器
        let _executor1 = factory
            .create_executor()
            .expect("Failed to create executor");
        let _executor2 = factory
            .create_executor()
            .expect("Failed to create executor");
        assert_eq!(factory.next_id(), 3);

        // 重置ID计数器
        factory.reset_id_counter();
        assert_eq!(factory.next_id(), 1);

        // 创建新执行器应该从ID 1开始
        let executor3 = factory
            .create_executor()
            .expect("Failed to create executor");
        assert_eq!(executor3.id(), 1);
        assert_eq!(factory.next_id(), 2);
    }

    #[tokio::test]
    async fn test_executor_descriptions() {
        let storage = create_test_storage_with_name("test_executor_descriptions");
        let mut factory = CypherExecutorFactory::new(storage);

        // 测试MATCH执行器描述
        let match_executor = factory
            .create_executor_for_statement(&create_test_match_statement())
            .expect("Failed to create executor");
        assert!(match_executor.description().contains("MATCH语句执行器"));
        assert!(match_executor.description().contains("图模式匹配"));

        // 测试CREATE执行器描述
        let create_executor = factory
            .create_executor_for_statement(&create_test_create_statement())
            .expect("Failed to create executor");
        assert!(create_executor.description().contains("CREATE语句执行器"));
        assert!(create_executor.description().contains("创建节点和关系"));

        // 测试DELETE执行器描述
        let delete_executor = factory
            .create_executor_for_statement(&create_test_delete_statement())
            .expect("Failed to create executor");
        assert!(delete_executor.description().contains("DELETE语句执行器"));
        assert!(delete_executor.description().contains("删除节点和关系"));

        // 测试RETURN执行器描述
        let return_executor = factory
            .create_executor_for_statement(&create_test_return_statement())
            .expect("Failed to create executor");
        assert!(return_executor.description().contains("RETURN语句执行器"));
        assert!(return_executor.description().contains("返回查询结果"));

        // 测试SET执行器描述
        let set_executor = factory
            .create_executor_for_statement(&create_test_set_statement())
            .expect("Failed to create executor");
        assert!(set_executor.description().contains("SET语句执行器"));
        assert!(set_executor.description().contains("设置属性值"));

        // 测试WHERE执行器描述
        let where_executor = factory
            .create_executor_for_statement(&create_test_where_statement())
            .expect("Failed to create executor");
        assert!(where_executor.description().contains("WHERE语句执行器"));
        assert!(where_executor.description().contains("条件过滤"));
    }

    #[tokio::test]
    async fn test_executor_lifecycle() {
        let storage = create_test_storage_with_name("test_executor_lifecycle");
        let mut factory = CypherExecutorFactory::new(storage);

        let mut executor = factory
            .create_executor()
            .expect("Failed to create executor");

        // 测试执行器生命周期
        assert!(!executor.is_open());
        assert!(executor.open().is_ok());
        assert!(executor.is_open());
        assert!(executor.close().is_ok());
        assert!(!executor.is_open());
    }

    #[tokio::test]
    async fn test_mixed_executor_creation() {
        let storage = create_test_storage_with_name("test_mixed_executor_creation");
        let mut factory = CypherExecutorFactory::new(storage);

        // 先创建一个通用执行器
        let _general_executor = factory
            .create_executor()
            .expect("Failed to create executor");
        assert_eq!(factory.next_id(), 2);

        // 然后创建一个专用执行器
        let match_executor = factory
            .create_executor_for_statement(&create_test_match_statement())
            .expect("Failed to create executor");
        assert_eq!(match_executor.id(), 2);
        assert_eq!(factory.next_id(), 3);

        // 再创建一个通用执行器
        let _general_executor2 = factory
            .create_executor()
            .expect("Failed to create executor");
        assert_eq!(factory.next_id(), 4);
    }
}
