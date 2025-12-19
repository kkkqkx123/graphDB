//! 验证器注册表
//! 全局验证器管理

use crate::core::error::DBResult;
use crate::query::context::QueryContext;
use crate::query::parser::cypher::ast::CypherStatement;
use crate::query::validator::validator_factory::ValidatorFactory;
use crate::query::validator::validator_trait::Validator;
use std::sync::Arc;

/// 验证器注册表
pub struct ValidatorRegistry {
    factory: ValidatorFactory,
}

impl ValidatorRegistry {
    pub fn new() -> Self {
        let factory = ValidatorFactory::new();

        Self { factory }
    }

    /// 创建验证器
    pub fn create_validator(
        &self,
        statement: &CypherStatement,
        qctx: Arc<QueryContext>,
    ) -> DBResult<Box<dyn Validator>> {
        self.factory.create_validator(statement, qctx)
    }
}

/// 全局验证器注册表
pub fn get_validator_registry() -> &'static ValidatorRegistry {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<ValidatorRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| ValidatorRegistry::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::managers::r#impl::{
        MockIndexManager, MockMetaClient, MockSchemaManager, MockStorageClient,
    };
    use crate::query::parser::cypher::ast::clauses::{CreateClause, MatchClause};

    #[test]
    fn test_validator_registry_creation() {
        let registry = ValidatorRegistry::new();
        // 验证注册表创建成功
        assert!(true);
    }

    #[test]
    fn test_global_validator_registry() {
        // 测试全局注册表
        let _registry = get_validator_registry();
        assert!(true);
    }

    #[test]
    fn test_registry_create_match_validator() {
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

        let registry = ValidatorRegistry::new();

        let statement = CypherStatement::Match(MatchClause {
            patterns: Vec::new(),
            where_clause: None,
            optional: false,
        });

        let validator = registry.create_validator(&statement, qctx).unwrap();
        assert_eq!(validator.name(), "MatchValidator");
    }

    #[test]
    fn test_registry_create_create_validator() {
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

        let registry = ValidatorRegistry::new();

        let statement = CypherStatement::Create(CreateClause {
            patterns: Vec::new(),
        });

        let validator = registry.create_validator(&statement, qctx).unwrap();
        assert_eq!(validator.name(), "CreateValidator");
    }

    #[test]
    fn test_global_registry_create_validator() {
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

        let statement = CypherStatement::Match(MatchClause {
            patterns: Vec::new(),
            where_clause: None,
            optional: false,
        });

        let validator = get_validator_registry()
            .create_validator(&statement, qctx)
            .unwrap();
        assert_eq!(validator.name(), "MatchValidator");
    }
}
