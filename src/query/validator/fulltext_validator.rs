//! Full-Text Search Validator
//!
//! This module implements the validator for full-text search statements,
//! ensuring semantic correctness before plan generation.

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::parser::ast::{
    AlterFulltextIndex, CreateFulltextIndex, DescribeFulltextIndex, DropFulltextIndex,
    FulltextMatchCondition, FulltextQueryExpr, LookupFulltext, MatchFulltext, SearchStatement,
    ShowFulltextIndex,
};
use crate::query::validator::{ValidationContext, ValidationInfo, Validator};
use crate::query::QueryContext;

/// Full-text search validator
pub struct FulltextValidator<'a> {
    context: &'a ValidationContext,
    query_context: Arc<QueryContext>,
}

impl<'a> FulltextValidator<'a> {
    /// Create a new full-text validator
    pub fn new(context: &'a ValidationContext, query_context: Arc<QueryContext>) -> Self {
        Self {
            context,
            query_context,
        }
    }

    /// Validate full-text search statements
    pub fn validate(&self, stmt: &crate::query::parser::ast::Stmt) -> Result<ValidationInfo, ValidationError> {
        match stmt {
            crate::query::parser::ast::Stmt::CreateFulltextIndex(create) => {
                self.validate_create_index(create)
            }
            crate::query::parser::ast::Stmt::DropFulltextIndex(drop) => {
                self.validate_drop_index(drop)
            }
            crate::query::parser::ast::Stmt::AlterFulltextIndex(alter) => {
                self.validate_alter_index(alter)
            }
            crate::query::parser::ast::Stmt::ShowFulltextIndex(show) => {
                self.validate_show_index(show)
            }
            crate::query::parser::ast::Stmt::DescribeFulltextIndex(describe) => {
                self.validate_describe_index(describe)
            }
            crate::query::parser::ast::Stmt::Search(search) => {
                self.validate_search(search)
            }
            crate::query::parser::ast::Stmt::LookupFulltext(lookup) => {
                self.validate_lookup(lookup)
            }
            crate::query::parser::ast::Stmt::MatchFulltext(match_stmt) => {
                self.validate_match(match_stmt)
            }
            _ => Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Not a full-text search statement".to_string(),
            )),
        }
    }

    /// Validate CREATE FULLTEXT INDEX statement
    fn validate_create_index(&self, create: &CreateFulltextIndex) -> Result<ValidationInfo, ValidationError> {
        // Check if schema exists
        let space_id = self.context.space_id();
        
        // Validate index name
        if create.index_name.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Index name cannot be empty".to_string(),
            ));
        }

        // Validate fields
        if create.fields.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Full-text index must have at least one field".to_string(),
            ));
        }

        // Validate engine-specific options
        match create.engine_type {
            crate::core::types::FulltextEngineType::Bm25 => {
                if let Some(ref config) = create.options.bm25_config {
                    // Validate BM25 parameters
                    if let Some(k1) = config.k1 {
                        if k1 < 0.0 {
                            return Err(ValidationError::new(
                                ValidationErrorType::SemanticError,
                                "BM25 k1 parameter must be non-negative".to_string(),
                            ));
                        }
                    }
                    if let Some(b) = config.b {
                        if b < 0.0 || b > 1.0 {
                            return Err(ValidationError::new(
                                ValidationErrorType::SemanticError,
                                "BM25 b parameter must be between 0 and 1".to_string(),
                            ));
                        }
                    }
                }
            }
            crate::core::types::FulltextEngineType::Inversearch => {
                if let Some(ref config) = create.options.inversearch_config {
                    // Validate Inversearch parameters
                    if let Some(resolution) = config.resolution {
                        if resolution == 0 {
                            return Err(ValidationError::new(
                                ValidationErrorType::SemanticError,
                                "Inversearch resolution must be positive".to_string(),
                            ));
                        }
                    }
                }
            }
        }

        Ok(ValidationInfo::new(create.index_name.clone(), space_id))
    }

    /// Validate DROP FULLTEXT INDEX statement
    fn validate_drop_index(&self, drop: &DropFulltextIndex) -> Result<ValidationInfo, ValidationError> {
        if drop.index_name.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Index name cannot be empty".to_string(),
            ));
        }

        Ok(ValidationInfo::new(drop.index_name.clone(), self.context.space_id()))
    }

    /// Validate ALTER FULLTEXT INDEX statement
    fn validate_alter_index(&self, alter: &AlterFulltextIndex) -> Result<ValidationInfo, ValidationError> {
        if alter.index_name.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Index name cannot be empty".to_string(),
            ));
        }

        if alter.actions.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "ALTER INDEX must have at least one action".to_string(),
            ));
        }

        // Validate each action
        for action in &alter.actions {
            match action {
                crate::query::parser::ast::AlterIndexAction::AddField(field) => {
                    if field.field_name.is_empty() {
                        return Err(ValidationError::new(
                            ValidationErrorType::SemanticError,
                            "Field name cannot be empty".to_string(),
                        ));
                    }
                }
                crate::query::parser::ast::AlterIndexAction::DropField(field_name) => {
                    if field_name.is_empty() {
                        return Err(ValidationError::new(
                            ValidationErrorType::SemanticError,
                            "Field name cannot be empty".to_string(),
                        ));
                    }
                }
                _ => {} // Other actions don't need special validation
            }
        }

        Ok(ValidationInfo::new(alter.index_name.clone(), self.context.space_id()))
    }

    /// Validate SHOW FULLTEXT INDEX statement
    fn validate_show_index(&self, _show: &ShowFulltextIndex) -> Result<ValidationInfo, ValidationError> {
        Ok(ValidationInfo::new("show_indexes".to_string(), self.context.space_id()))
    }

    /// Validate DESCRIBE FULLTEXT INDEX statement
    fn validate_describe_index(&self, describe: &DescribeFulltextIndex) -> Result<ValidationInfo, ValidationError> {
        if describe.index_name.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Index name cannot be empty".to_string(),
            ));
        }

        Ok(ValidationInfo::new(describe.index_name.clone(), self.context.space_id()))
    }

    /// Validate SEARCH statement
    fn validate_search(&self, search: &SearchStatement) -> Result<ValidationInfo, ValidationError> {
        if search.index_name.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Index name cannot be empty".to_string(),
            ));
        }

        // Validate query expression
        self.validate_query_expr(&search.query)?;

        // Validate limit and offset
        if let Some(limit) = search.limit {
            if limit == 0 {
                return Err(ValidationError::new(
                    ValidationErrorType::SemanticError,
                    "LIMIT must be positive".to_string(),
                ));
            }
        }

        if let Some(offset) = search.offset {
            if offset == 0 {
                return Err(ValidationError::new(
                    ValidationErrorType::SemanticError,
                    "OFFSET must be non-negative".to_string(),
                ));
            }
        }

        Ok(ValidationInfo::new(search.index_name.clone(), self.context.space_id()))
    }

    /// Validate full-text query expression
    fn validate_query_expr(&self, expr: &FulltextQueryExpr) -> Result<(), ValidationError> {
        match expr {
            FulltextQueryExpr::Simple(text) => {
                if text.is_empty() {
                    return Err(ValidationError::new(
                        ValidationErrorType::SemanticError,
                        "Query text cannot be empty".to_string(),
                    ));
                }
            }
            FulltextQueryExpr::Field(field, query) => {
                if field.is_empty() || query.is_empty() {
                    return Err(ValidationError::new(
                        ValidationErrorType::SemanticError,
                        "Field name and query text cannot be empty".to_string(),
                    ));
                }
            }
            FulltextQueryExpr::MultiField(fields) => {
                if fields.is_empty() {
                    return Err(ValidationError::new(
                        ValidationErrorType::SemanticError,
                        "Multi-field query must have at least one field".to_string(),
                    ));
                }
                for (field, query) in fields {
                    if field.is_empty() || query.is_empty() {
                        return Err(ValidationError::new(
                            ValidationErrorType::SemanticError,
                            "Field name and query text cannot be empty".to_string(),
                        ));
                    }
                }
            }
            FulltextQueryExpr::Boolean { must, should, must_not } => {
                if must.is_empty() && should.is_empty() {
                    return Err(ValidationError::new(
                        ValidationErrorType::SemanticError,
                        "Boolean query must have at least one must or should clause".to_string(),
                    ));
                }
                // Recursively validate sub-queries
                for q in must.iter().chain(should.iter()).chain(must_not.iter()) {
                    self.validate_query_expr(q)?;
                }
            }
            FulltextQueryExpr::Phrase(text) => {
                if text.is_empty() {
                    return Err(ValidationError::new(
                        ValidationErrorType::SemanticError,
                        "Phrase query text cannot be empty".to_string(),
                    ));
                }
            }
            FulltextQueryExpr::Prefix(prefix) => {
                if prefix.is_empty() {
                    return Err(ValidationError::new(
                        ValidationErrorType::SemanticError,
                        "Prefix cannot be empty".to_string(),
                    ));
                }
            }
            FulltextQueryExpr::Fuzzy(text, distance) => {
                if text.is_empty() {
                    return Err(ValidationError::new(
                        ValidationErrorType::SemanticError,
                        "Fuzzy query text cannot be empty".to_string(),
                    ));
                }
                if let Some(d) = distance {
                    if *d > 20 {
                        return Err(ValidationError::new(
                            ValidationErrorType::SemanticError,
                            "Fuzzy distance cannot exceed 20".to_string(),
                        ));
                    }
                }
            }
            FulltextQueryExpr::Range { field, lower, upper, .. } => {
                if field.is_empty() {
                    return Err(ValidationError::new(
                        ValidationErrorType::SemanticError,
                        "Range field name cannot be empty".to_string(),
                    ));
                }
                if lower.is_none() && upper.is_none() {
                    return Err(ValidationError::new(
                        ValidationErrorType::SemanticError,
                        "Range query must have at least one bound".to_string(),
                    ));
                }
            }
            FulltextQueryExpr::Wildcard(pattern) => {
                if pattern.is_empty() {
                    return Err(ValidationError::new(
                        ValidationErrorType::SemanticError,
                        "Wildcard pattern cannot be empty".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Validate LOOKUP FULLTEXT statement
    fn validate_lookup(&self, lookup: &LookupFulltext) -> Result<ValidationInfo, ValidationError> {
        if lookup.schema_name.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Schema name cannot be empty".to_string(),
            ));
        }

        if lookup.index_name.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Index name cannot be empty".to_string(),
            ));
        }

        if lookup.query.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Query text cannot be empty".to_string(),
            ));
        }

        Ok(ValidationInfo::new(lookup.index_name.clone(), self.context.space_id()))
    }

    /// Validate MATCH with full-text statement
    fn validate_match(&self, match_stmt: &MatchFulltext) -> Result<ValidationInfo, ValidationError> {
        if match_stmt.pattern.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Match pattern cannot be empty".to_string(),
            ));
        }

        if match_stmt.fulltext_condition.field.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Full-text match field cannot be empty".to_string(),
            ));
        }

        if match_stmt.fulltext_condition.query.is_empty() {
            return Err(ValidationError::new(
                ValidationErrorType::SemanticError,
                "Full-text match query cannot be empty".to_string(),
            ));
        }

        Ok(ValidationInfo::new("match_fulltext".to_string(), self.context.space_id()))
    }
}

/// Helper function to validate full-text statements
pub fn validate_fulltext_statement(
    stmt: &crate::query::parser::ast::Stmt,
    context: &ValidationContext,
    query_context: Arc<QueryContext>,
) -> Result<ValidationInfo, ValidationError> {
    let validator = FulltextValidator::new(context, query_context);
    validator.validate(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::{FulltextQueryExpr, IndexFieldDef, SearchStatement};

    fn create_test_context() -> (ValidationContext, Arc<QueryContext>) {
        let validation_context = ValidationContext::new(1, 1);
        let query_context = Arc::new(QueryContext::new());
        (validation_context, query_context)
    }

    #[test]
    fn test_validate_create_index() {
        let (context, qctx) = create_test_context();
        let validator = FulltextValidator::new(&context, qctx);

        let create = CreateFulltextIndex::new(
            "idx_test".to_string(),
            "schema".to_string(),
            vec![IndexFieldDef::new("field".to_string())],
            crate::core::types::FulltextEngineType::Bm25,
        );

        let stmt = crate::query::parser::ast::Stmt::CreateFulltextIndex(create);
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_search_statement() {
        let (context, qctx) = create_test_context();
        let validator = FulltextValidator::new(&context, qctx);

        let search = SearchStatement::new(
            "idx_test".to_string(),
            FulltextQueryExpr::Simple("database".to_string()),
        );

        let stmt = crate::query::parser::ast::Stmt::Search(search);
        let result = validator.validate(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_query() {
        let (context, qctx) = create_test_context();
        let validator = FulltextValidator::new(&context, qctx);

        let search = SearchStatement::new(
            "idx_test".to_string(),
            FulltextQueryExpr::Simple("".to_string()),
        );

        let stmt = crate::query::parser::ast::Stmt::Search(search);
        let result = validator.validate(&stmt);
        assert!(result.is_err());
    }
}
