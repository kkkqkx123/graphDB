//! 验证器工厂
//! 负责创建和管理验证器实例

use crate::core::error::{DBError, DBResult};
use crate::query::context::QueryContext;
use crate::query::parser::cypher::ast::CypherStatement;
use crate::query::validator::validator_trait::{Validator, ValidatorCreator};
use crate::query::validator::{MatchValidator, CreateValidator};
use std::collections::HashMap;
use std::sync::Arc;

/// 验证器工厂
pub struct ValidatorFactory {
    creators: HashMap<String, Box<dyn ValidatorCreator>>,
}

impl ValidatorFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            creators: HashMap::new(),
        };
        
        // 注册Cypher验证器
        factory.register_cypher_validators();
        
        factory
    }
    
    /// 创建验证器
    pub fn create_validator(
        &self,
        statement: &CypherStatement,
        qctx: Arc<QueryContext>,
    ) -> DBResult<Box<dyn Validator>> {
        let statement_type = statement.statement_type();
        
        match self.creators.get(statement_type) {
            Some(creator) => creator.create(statement, qctx),
            None => Err(DBError::Validation(
                crate::core::error::ValidationError::UnsupportedStatement(
                    format!("Unsupported statement type: {}", statement_type)
                )
            )),
        }
    }
    
    /// 注册验证器创建器
    pub fn register_creator(&mut self, statement_type: String, creator: Box<dyn ValidatorCreator>) {
        self.creators.insert(statement_type, creator);
    }
    
    /// 注册Cypher验证器
    fn register_cypher_validators(&mut self) {
        self.register_creator("MATCH".to_string(), Box::new(CypherValidatorFactory));
        self.register_creator("CREATE".to_string(), Box::new(CypherValidatorFactory));
        self.register_creator("RETURN".to_string(), Box::new(CypherValidatorFactory));
        self.register_creator("WITH".to_string(), Box::new(CypherValidatorFactory));
        self.register_creator("SET".to_string(), Box::new(CypherValidatorFactory));
        self.register_creator("DELETE".to_string(), Box::new(CypherValidatorFactory));
        self.register_creator("MERGE".to_string(), Box::new(CypherValidatorFactory));
        self.register_creator("UNWIND".to_string(), Box::new(CypherValidatorFactory));
    }
}

/// Cypher验证器工厂
pub struct CypherValidatorFactory;

impl CypherValidatorFactory {
    pub fn create_validator(statement: &CypherStatement, qctx: Arc<QueryContext>) -> DBResult<Box<dyn Validator>> {
        match statement {
            CypherStatement::Match(_) => Ok(Box::new(MatchValidator::new(statement.clone(), qctx)?)),
            CypherStatement::Create(_) => Ok(Box::new(CreateValidator::new(statement.clone(), qctx)?)),
            CypherStatement::Return(_) => {
                // Ok(Box::new(ReturnValidator::new(statement.clone(), qctx)?))
                Err(DBError::Validation(
                    crate::core::error::ValidationError::UnsupportedStatement(
                        "ReturnValidator not implemented yet".to_string()
                    )
                ))
            }
            CypherStatement::With(_) => {
                // Ok(Box::new(WithValidator::new(statement.clone(), qctx)?))
                Err(DBError::Validation(
                    crate::core::error::ValidationError::UnsupportedStatement(
                        "WithValidator not implemented yet".to_string()
                    )
                ))
            }
            CypherStatement::Set(_) => {
                // Ok(Box::new(SetValidator::new(statement.clone(), qctx)?))
                Err(DBError::Validation(
                    crate::core::error::ValidationError::UnsupportedStatement(
                        "SetValidator not implemented yet".to_string()
                    )
                ))
            }
            CypherStatement::Delete(_) => {
                // Ok(Box::new(DeleteValidator::new(statement.clone(), qctx)?))
                Err(DBError::Validation(
                    crate::core::error::ValidationError::UnsupportedStatement(
                        "DeleteValidator not implemented yet".to_string()
                    )
                ))
            }
            CypherStatement::Merge(_) => {
                // Ok(Box::new(MergeValidator::new(statement.clone(), qctx)?))
                Err(DBError::Validation(
                    crate::core::error::ValidationError::UnsupportedStatement(
                        "MergeValidator not implemented yet".to_string()
                    )
                ))
            }
            CypherStatement::Unwind(_) => {
                // Ok(Box::new(UnwindValidator::new(statement.clone(), qctx)?))
                Err(DBError::Validation(
                    crate::core::error::ValidationError::UnsupportedStatement(
                        "UnwindValidator not implemented yet".to_string()
                    )
                ))
            }
            _ => Err(DBError::Validation(
                crate::core::error::ValidationError::UnsupportedStatement(
                    format!("Unsupported Cypher statement: {:?}", statement)
                )
            )),
        }
    }
}

impl ValidatorCreator for CypherValidatorFactory {
    fn create(&self, statement: &CypherStatement, qctx: Arc<QueryContext>) -> DBResult<Box<dyn Validator>> {
        Self::create_validator(statement, qctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::cypher::ast::clauses::{MatchClause, CreateClause};
    use crate::query::context::managers::r#impl::{
        MockIndexManager, MockMetaClient, MockSchemaManager, MockStorageClient,
    };

    #[test]
    fn test_validator_factory_creation() {
        let factory = ValidatorFactory::new();
        assert!(!factory.creators.is_empty());
    }

    #[test]
    fn test_create_match_validator() {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let qctx = Arc::new(QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        ));

        let factory = ValidatorFactory::new();
        
        let statement = CypherStatement::Match(MatchClause {
            patterns: Vec::new(),
            where_clause: None,
            optional: false,
        });

        let validator = factory.create_validator(&statement, qctx).unwrap();
        assert_eq!(validator.name(), "MatchValidator");
    }

    #[test]
    fn test_create_create_validator() {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let qctx = Arc::new(QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        ));

        let factory = ValidatorFactory::new();
        
        let statement = CypherStatement::Create(CreateClause {
            patterns: Vec::new(),
        });

        let validator = factory.create_validator(&statement, qctx).unwrap();
        assert_eq!(validator.name(), "CreateValidator");
    }

    #[test]
    fn test_create_unsupported_validator() {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let qctx = Arc::new(QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        ));

        let factory = ValidatorFactory::new();
        
        // 测试未实现的验证器
        let statement = CypherStatement::Return(crate::query::parser::cypher::ast::clauses::ReturnClause {
            return_items: Vec::new(),
            distinct: false,
            order_by: None,
            skip: None,
            limit: None,
        });

        let result = factory.create_validator(&statement, qctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_cypher_validator_factory() {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let qctx = Arc::new(QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        ));

        let factory = CypherValidatorFactory;
        
        let statement = CypherStatement::Match(MatchClause {
            patterns: Vec::new(),
            where_clause: None,
            optional: false,
        });

        let validator = factory.create(&statement, qctx).unwrap();
        assert_eq!(validator.name(), "MatchValidator");
    }
}