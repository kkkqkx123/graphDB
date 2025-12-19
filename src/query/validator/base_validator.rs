//! 基础验证器实现
//! 对应 NebulaGraph Validator.h/.cpp 的功能

use crate::core::error::{DBError, DBResult, ValidationError};
use crate::query::context::ast_context::ColumnDefinition;
use crate::query::context::{AstContext, QueryContext};
use crate::query::parser::cypher::ast::CypherStatement;
use crate::query::planner::plan::execution_plan::ExecutionPlan;
use crate::query::validator::validator_trait::{Validator, ValidatorExt};
use std::sync::Arc;

/// 基础验证器实现
pub struct BaseValidator {
    statement: CypherStatement,
    qctx: Arc<QueryContext>,
    ast_ctx: AstContext,
    input_var_name: Option<String>,
    output_columns: Vec<ColumnDefinition>,
    input_columns: Vec<ColumnDefinition>,
}

impl BaseValidator {
    pub fn new(statement: CypherStatement, qctx: Arc<QueryContext>) -> Self {
        let query_type = statement.statement_type().to_string();
        let query_text = format!("{:?}", statement); // 简化实现，实际应该有原始查询文本

        let ast_ctx = AstContext::new(query_type, query_text);

        Self {
            statement,
            qctx,
            ast_ctx,
            input_var_name: None,
            output_columns: Vec::new(),
            input_columns: Vec::new(),
        }
    }

    /// 检查空间是否已选择
    pub fn check_space_chosen(&self) -> DBResult<()> {
        if self.qctx.space_id.is_none() {
            return Err(DBError::Validation(ValidationError::ContextError(
                "No space selected".to_string(),
            )));
        }
        Ok(())
    }

    /// 检查权限
    pub fn check_permission(&self) -> DBResult<()> {
        // 实现权限检查逻辑
        // 这里可以调用权限管理器
        Ok(())
    }

    /// 推断表达式类型
    pub fn deduce_expression_type(
        &self,
        expr: &crate::graph::expression::Expression,
    ) -> DBResult<crate::core::ValueTypeDef> {
        // 实现表达式类型推断
        Ok(crate::core::ValueTypeDef::Unknown)
    }

    /// 收集表达式属性
    pub fn collect_expression_properties(
        &self,
        expr: &crate::graph::expression::Expression,
    ) -> DBResult<ExpressionProperties> {
        // 实现表达式属性收集
        Ok(ExpressionProperties::new())
    }

    /// 验证列名不重复
    pub fn validate_no_duplicate_columns(&self) -> DBResult<()> {
        let mut column_names = std::collections::HashSet::new();

        for col in &self.output_columns {
            if column_names.contains(&col.name) {
                return Err(DBError::Validation(ValidationError::SemanticError(
                    format!("Duplicate column name: {}", col.name),
                )));
            }
            column_names.insert(col.name.clone());
        }

        Ok(())
    }

    /// 设置输出列
    pub fn set_output_columns(&mut self, columns: Vec<ColumnDefinition>) {
        self.output_columns = columns;
    }

    /// 设置输入列
    pub fn set_input_columns(&mut self, columns: Vec<ColumnDefinition>) {
        self.input_columns = columns;
    }

    /// 获取语句引用
    pub fn statement(&self) -> &CypherStatement {
        &self.statement
    }

    /// 获取查询上下文引用
    pub fn query_context(&self) -> &QueryContext {
        &self.qctx
    }
}

impl Validator for BaseValidator {
    fn validate(&mut self) -> DBResult<()> {
        // 基础验证流程
        self.check_space_chosen()?;
        self.check_permission()?;
        self.validate_no_duplicate_columns()?;

        // 调用具体验证逻辑
        self.validate_impl()
    }

    fn to_plan(&mut self) -> DBResult<ExecutionPlan> {
        // 调用具体规划逻辑
        self.to_plan_impl()
    }

    fn ast_context(&self) -> &AstContext {
        &self.ast_ctx
    }

    fn name(&self) -> &'static str {
        "BaseValidator"
    }

    fn input_var_name(&self) -> Option<&str> {
        self.input_var_name.as_deref()
    }

    fn set_input_var_name(&mut self, name: String) {
        self.input_var_name = Some(name);
    }

    fn output_columns(&self) -> &[ColumnDefinition] {
        &self.output_columns
    }

    fn input_columns(&self) -> &[ColumnDefinition] {
        &self.input_columns
    }
}

/// 表达式属性
#[derive(Debug, Clone)]
pub struct ExpressionProperties {
    // 表达式属性定义
}

impl ExpressionProperties {
    pub fn new() -> Self {
        Self {}
    }
}

/// BaseValidator的扩展trait，供具体验证器实现
impl ValidatorExt for BaseValidator {
    fn validate_impl(&mut self) -> DBResult<()> {
        // 基类默认实现，子类应该重写此方法
        Ok(())
    }

    fn to_plan_impl(&mut self) -> DBResult<ExecutionPlan> {
        // 基类默认实现，子类应该重写此方法
        Err(DBError::Validation(ValidationError::PlanError(
            "BaseValidator does not implement to_plan_impl".to_string(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::managers::r#impl::{
        MockIndexManager, MockMetaClient, MockSchemaManager, MockStorageClient,
    };
    use crate::query::parser::cypher::ast::clauses::MatchClause;

    #[test]
    fn test_base_validator_creation() {
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

        let validator = BaseValidator::new(statement, qctx);
        assert_eq!(validator.name(), "BaseValidator");
        assert!(validator.input_var_name().is_none());
        assert_eq!(validator.output_columns().len(), 0);
        assert_eq!(validator.input_columns().len(), 0);
    }

    #[test]
    fn test_check_space_chosen() {
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

        let validator = BaseValidator::new(statement, qctx);

        // 没有选择空间应该返回错误
        assert!(validator.check_space_chosen().is_err());

        // 选择空间后应该成功
        // validator.qctx.set_space(1); // 需要实现set_space方法
        // assert!(validator.check_space_chosen().is_ok());
    }

    #[test]
    fn test_validate_no_duplicate_columns() {
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

        let mut validator = BaseValidator::new(statement, qctx);

        // 没有重复列名应该成功
        let col1 = ColumnDefinition::new("name".to_string(), "string".to_string());
        let col2 = ColumnDefinition::new("age".to_string(), "integer".to_string());
        validator.set_output_columns(vec![col1, col2]);
        assert!(validator.validate_no_duplicate_columns().is_ok());

        // 有重复列名应该失败
        let col1 = ColumnDefinition::new("name".to_string(), "string".to_string());
        let col2 = ColumnDefinition::new("name".to_string(), "integer".to_string());
        validator.set_output_columns(vec![col1, col2]);
        assert!(validator.validate_no_duplicate_columns().is_err());
    }
}
