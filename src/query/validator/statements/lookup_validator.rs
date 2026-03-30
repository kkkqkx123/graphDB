//! LOOKUP Statement Validator
//! 对应 NebulaGraph LookupValidator.h/.cpp 的功能
//! Verify the validity of the LOOKUP statement.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::Expression;
use crate::query::parser::ast::stmt::Ast;
use crate::query::parser::ast::{Stmt, YieldItem};
use crate::query::validator::structs::validation_info::{
    IndexHint, OptimizationHint, ValidationInfo,
};
use crate::query::validator::structs::AliasType;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};
use crate::query::QueryContext;
use crate::storage::metadata::redb_schema_manager::RedbSchemaManager;
use crate::storage::metadata::schema_manager::SchemaManager;

/// Verified LOOKUP information
#[derive(Debug, Clone)]
pub struct ValidatedLookup {
    pub space_id: u64,
    pub label: String,
    pub is_edge: bool,
    pub index_type: LookupIndexType,
    pub filter_expression: Option<ContextualExpression>,
    pub yield_columns: Vec<LookupYieldColumn>,
    pub is_yield_all: bool,
}

#[derive(Debug, Clone)]
pub struct LookupYieldColumn {
    pub name: String,
    pub alias: Option<String>,
    pub expression: Option<ContextualExpression>,
}

#[derive(Debug, Clone)]
pub enum LookupIndexType {
    None,
    Single(String),
    Composite(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct LookupProperty {
    pub name: String,
    pub type_: ValueType,
}

/// LOOKUP Validator
/// Parse the LOOKUP statement entirely from the AST (Abstract Syntax Tree), without relying on any external preset values.
#[derive(Debug)]
pub struct LookupValidator {
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expression_props: ExpressionProps,
    user_defined_vars: Vec<String>,
    validated_result: Option<ValidatedLookup>,
    schema_manager: Option<Arc<RedbSchemaManager>>,
}

impl LookupValidator {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            expression_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validated_result: None,
            schema_manager: None,
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: Arc<RedbSchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    pub fn set_schema_manager(&mut self, schema_manager: Arc<RedbSchemaManager>) {
        self.schema_manager = Some(schema_manager);
    }

    /// Parsing a LOOKUP statement from AST (Abstract Syntax Tree)
    fn parse_from_ast(&self, ast: &Arc<Ast>) -> Result<ParsedLookupInfo, ValidationError> {
        let lookup_stmt = match &ast.stmt {
            Stmt::Lookup(lookup_stmt) => lookup_stmt,
            _ => {
                return Err(ValidationError::new(
                    "期望 LOOKUP 语句".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // Analysis target (Tag or Edge)
        let (label, is_edge) = match &lookup_stmt.target {
            crate::query::parser::ast::stmt::LookupTarget::Tag(name) => (name.clone(), false),
            crate::query::parser::ast::stmt::LookupTarget::Edge(name) => (name.clone(), true),
        };

        if label.is_empty() {
            return Err(ValidationError::new(
                "LOOKUP 必须指定 Tag 或 Edge 名称".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // Analyzing the WHERE clause
        let filter_expression = lookup_stmt.where_clause.clone();

        // Analyzing the YIELD clause
        let mut yield_columns = Vec::new();
        let mut is_yield_all = false;

        if let Some(ref yield_clause) = lookup_stmt.yield_clause {
            for item in &yield_clause.items {
                yield_columns.push(self.parse_yield_item(item)?);
            }
            // Check whether it is YIELD *
            if yield_columns.len() == 1 && yield_columns[0].name == "*" {
                is_yield_all = true;
            }
        }

        Ok(ParsedLookupInfo {
            label,
            is_edge,
            filter_expression,
            yield_columns,
            is_yield_all,
        })
    }

    /// Analyzing a single YIELD entry
    fn parse_yield_item(&self, item: &YieldItem) -> Result<LookupYieldColumn, ValidationError> {
        let name = self.extract_column_name(&item.expression)?;
        Ok(LookupYieldColumn {
            name,
            alias: item.alias.clone(),
            expression: Some(item.expression.clone()),
        })
    }

    /// Extract column names from the expression.
    fn extract_column_name(&self, expr: &ContextualExpression) -> Result<String, ValidationError> {
        if let Some(inner_expr) = expr.expression() {
            let expr_inner = inner_expr.inner();
            match expr_inner {
                Expression::Variable(name) => Ok(name.clone()),
                Expression::Label(name) => Ok(name.clone()),
                Expression::Property { property, .. } => Ok(property.clone()),
                _ => Err(ValidationError::new(
                    "无法从表达式中提取列名".to_string(),
                    ValidationErrorType::SemanticError,
                )),
            }
        } else {
            Err(ValidationError::new(
                "表达式无效".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    /// Verify the LOOKUP target
    /// 对应 NebulaGraph 的 validateFrom() 方法
    fn validate_lookup_target(
        &self,
        space_name: &str,
        label: &str,
        is_edge: bool,
    ) -> Result<LookupIndexType, ValidationError> {
        // Check whether schema_manager is available.
        let schema_manager = self.schema_manager.as_ref().ok_or_else(|| {
            ValidationError::new(
                "Schema manager not available".to_string(),
                ValidationErrorType::SemanticError,
            )
        })?;

        if is_edge {
            // Verify whether the Edge Type exists.
            match schema_manager.as_ref().get_edge_type(space_name, label) {
                Ok(Some(_edge_info)) => {
                    // If the “Edge Type” is present, the “Single” index type should be returned.
                    Ok(LookupIndexType::Single(label.to_string()))
                }
                Ok(None) => Err(ValidationError::new(
                    format!("Edge type '{}' not found in space '{}'", label, space_name),
                    ValidationErrorType::SemanticError,
                )),
                Err(e) => Err(ValidationError::new(
                    format!("Failed to get edge type '{}': {}", label, e),
                    ValidationErrorType::SemanticError,
                )),
            }
        } else {
            // Verify whether the Tag exists.
            match schema_manager.as_ref().get_tag(space_name, label) {
                Ok(Some(_tag_info)) => {
                    // If the “Tag” field exists, return the “Single” index type.
                    Ok(LookupIndexType::Single(label.to_string()))
                }
                Ok(None) => Err(ValidationError::new(
                    format!("Tag '{}' not found in space '{}'", label, space_name),
                    ValidationErrorType::SemanticError,
                )),
                Err(e) => Err(ValidationError::new(
                    format!("Failed to get tag '{}': {}", label, e),
                    ValidationErrorType::SemanticError,
                )),
            }
        }
    }

    /// Verify the filtering criteria.
    fn validate_filter(
        &self,
        filter: &Option<ContextualExpression>,
    ) -> Result<(), ValidationError> {
        if let Some(ref filter_expr) = filter {
            let expr_meta = match filter_expr.expression() {
                Some(m) => m,
                None => {
                    return Err(ValidationError::new(
                        "过滤表达式无效".to_string(),
                        ValidationErrorType::SemanticError,
                    ))
                }
            };
            let expr = expr_meta.inner();

            self.validate_filter_type(expr)?;

            if self.has_aggregate_expression(expr) {
                return Err(ValidationError::new(
                    "LOOKUP filter cannot contain aggregate expressions".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// Verify the filter type.
    fn validate_filter_type(&self, filter: &Expression) -> Result<(), ValidationError> {
        match filter {
            Expression::Binary { op, .. } => {
                use crate::core::BinaryOperator;
                match op {
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::LessThan
                    | BinaryOperator::LessThanOrEqual
                    | BinaryOperator::GreaterThan
                    | BinaryOperator::GreaterThanOrEqual
                    | BinaryOperator::And
                    | BinaryOperator::Or => Ok(()),
                    _ => Err(ValidationError::new(
                        "Filter expression must return bool type".to_string(),
                        ValidationErrorType::TypeError,
                    )),
                }
            }
            _ => Ok(()),
        }
    }

    /// Check whether it contains aggregate expressions.
    fn has_aggregate_expression(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Aggregate { .. } => true,
            Expression::Binary { left, right, .. } => {
                self.has_aggregate_expression(left) || self.has_aggregate_expression(right)
            }
            Expression::Unary { operand, .. } => self.has_aggregate_expression(operand),
            Expression::Function { args, .. } => {
                args.iter().any(|arg| self.has_aggregate_expression(arg))
            }
            _ => false,
        }
    }

    /// Verify the YIELD clause
    fn validate_yields(
        &self,
        yield_columns: &[LookupYieldColumn],
        is_yield_all: bool,
    ) -> Result<(), ValidationError> {
        if is_yield_all {
            return Ok(());
        }

        if yield_columns.is_empty() {
            return Err(ValidationError::new(
                "LOOKUP must have YIELD clause or YIELD *".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let mut seen_names: HashMap<String, usize> = HashMap::new();
        for col in yield_columns {
            let count = seen_names.entry(col.name.clone()).or_insert(0);
            *count += 1;
            if *count > 1 {
                return Err(ValidationError::new(
                    format!("Duplicate column name '{}' in YIELD clause", col.name),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// Generate a column of outputs.
    fn generate_output_columns(
        &self,
        yield_columns: &[LookupYieldColumn],
        is_yield_all: bool,
    ) -> Vec<ColumnDef> {
        let mut outputs = Vec::new();
        if is_yield_all {
            outputs.push(ColumnDef {
                name: "*".to_string(),
                type_: ValueType::List,
            });
        } else {
            for col in yield_columns {
                outputs.push(ColumnDef {
                    name: col.alias.clone().unwrap_or_else(|| col.name.clone()),
                    type_: ValueType::String,
                });
            }
        }
        outputs
    }
}

/// LOOKUP information parsed from AST
#[derive(Debug)]
struct ParsedLookupInfo {
    label: String,
    is_edge: bool,
    filter_expression: Option<ContextualExpression>,
    yield_columns: Vec<LookupYieldColumn>,
    is_yield_all: bool,
}

impl Default for LookupValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementing the StatementValidator trait
///
/// # Refactoring changes
/// The `validate` method accepts `Arc<Ast>` and `Arc<QueryContext>` as arguments.
impl StatementValidator for LookupValidator {
    fn validate(
        &mut self,
        ast: Arc<Ast>,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        // 1. Check whether additional space is needed.
        if !self.is_global_statement() && qctx.space_id().is_none() {
            return Err(ValidationError::new(
                "No image space selected, please execute first USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. Parsing the LOOKUP statement from Ast
        let parsed_info = self.parse_from_ast(&ast)?;

        // 3. Obtain the current name of the space.
        let space_name = qctx.space_name().unwrap_or_default();

        if space_name.is_empty() {
            return Err(ValidationError::new(
                "No image space selected, please execute first USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 4. Verify the LOOKUP target
        let index_type =
            self.validate_lookup_target(&space_name, &parsed_info.label, parsed_info.is_edge)?;

        // 4. Verify the filtering criteria
        self.validate_filter(&parsed_info.filter_expression)?;

        // 5. Verify the YIELD clause
        self.validate_yields(&parsed_info.yield_columns, parsed_info.is_yield_all)?;

        // 6. Obtain the space_id
        let space_id = qctx.space_id().unwrap_or(0);

        // 7. Generate the output column.
        self.outputs =
            self.generate_output_columns(&parsed_info.yield_columns, parsed_info.is_yield_all);

        // 8. Constructing detailed ValidationInfo
        let mut info = ValidationInfo::new();

        // 8.1 添加别名映射
        let alias_type = if parsed_info.is_edge {
            AliasType::Edge
        } else {
            AliasType::Node
        };
        info.add_alias(parsed_info.label.clone(), alias_type);

        // 8.2 添加语义信息
        if parsed_info.is_edge {
            info.semantic_info
                .referenced_edges
                .push(parsed_info.label.clone());
        } else {
            info.semantic_info
                .referenced_tags
                .push(parsed_info.label.clone());
        }

        // 8.3 添加优化提示
        if let Some(ref filter) = parsed_info.filter_expression {
            info.add_optimization_hint(OptimizationHint::UseIndexScan {
                table: parsed_info.label.clone(),
                column: "id".to_string(),
                condition: filter.clone(),
            });
        }

        // 8.4 添加索引提示
        match &index_type {
            LookupIndexType::Single(column) => {
                info.add_index_hint(IndexHint {
                    index_name: format!("{}_{}_index", parsed_info.label, column),
                    table_name: parsed_info.label.clone(),
                    columns: vec![column.clone()],
                    applicable_conditions: parsed_info
                        .filter_expression
                        .clone()
                        .map_or_else(std::vec::Vec::new, |f| vec![f]),
                    estimated_selectivity: 0.1,
                });
            }
            LookupIndexType::Composite(columns) => {
                info.add_index_hint(IndexHint {
                    index_name: format!("{}_composite_index", parsed_info.label),
                    table_name: parsed_info.label.clone(),
                    columns: columns.clone(),
                    applicable_conditions: parsed_info
                        .filter_expression
                        .clone()
                        .map_or_else(std::vec::Vec::new, |f| vec![f]),
                    estimated_selectivity: 0.05,
                });
            }
            LookupIndexType::None => {}
        }

        // 8.5 添加验证通过的子句
        info.validated_clauses
            .push(crate::query::validator::structs::ClauseKind::Match);

        // 9. Create the validation results (place this in the final step to avoid unnecessary clones).
        let validated = ValidatedLookup {
            space_id,
            label: parsed_info.label,
            is_edge: parsed_info.is_edge,
            index_type,
            filter_expression: parsed_info.filter_expression,
            yield_columns: parsed_info.yield_columns,
            is_yield_all: parsed_info.is_yield_all,
        };

        self.validated_result = Some(validated);

        // 10. Return the verification results containing detailed information.
        Ok(ValidationResult::success_with_info(info))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Lookup
    }

    fn is_global_statement(&self) -> bool {
        // LOOKUP is not a global statement; the relevant scope must be selected in advance.
        false
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expression_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::stmt::{Ast, LookupStmt, LookupTarget, YieldClause};
    use crate::query::parser::ast::Span;
    use crate::query::query_request_context::QueryRequestContext;
    use crate::query::validator::context::expression_context::ExpressionAnalysisContext;
    use std::sync::Arc;

    /// Create a QueryContext for testing purposes, which should contain a valid space_id.
    fn create_test_query_context() -> Arc<QueryContext> {
        let rctx = Arc::new(QueryRequestContext::new("TEST".to_string()));
        let mut qctx = QueryContext::new(rctx);
        let space_info = crate::core::types::SpaceInfo::new("test_space".to_string());
        qctx.set_space_info(space_info);
        Arc::new(qctx)
    }

    fn create_test_ast(stmt: Stmt) -> Arc<Ast> {
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        Arc::new(Ast::new(stmt, ctx))
    }

    fn create_simple_lookup_stmt(label: &str, is_edge: bool) -> LookupStmt {
        let target = if is_edge {
            LookupTarget::Edge(label.to_string())
        } else {
            LookupTarget::Tag(label.to_string())
        };

        LookupStmt {
            span: Span::default(),
            target,
            where_clause: None,
            yield_clause: Some(YieldClause {
                span: Span::default(),
                items: vec![],
                where_clause: None,
                order_by: None,
                limit: None,
                skip: None,
                sample: None,
            }),
        }
    }

    #[test]
    fn test_lookup_validator_basic() {
        let mut validator = LookupValidator::new();
        let lookup_stmt = create_simple_lookup_stmt("person", false);
        let qctx = create_test_query_context();

        let result = validator.validate(create_test_ast(Stmt::Lookup(lookup_stmt)), qctx);
        // The current attempt will fail because there is no YIELD column, and the calculation cannot be performed using the formula YIELD * …
        assert!(result.is_err());
    }

    #[test]
    fn test_lookup_validator_empty_label() {
        let mut validator = LookupValidator::new();
        let lookup_stmt = create_simple_lookup_stmt("", false);
        let qctx = create_test_query_context();

        let result = validator.validate(create_test_ast(Stmt::Lookup(lookup_stmt)), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("必须指定"));
    }

    #[test]
    fn test_lookup_validator_not_lookup_stmt() {
        let mut validator = LookupValidator::new();
        let qctx = create_test_query_context();
        // Do not set the LOOKUP statement.

        let result = validator.validate(
            create_test_ast(Stmt::Use(crate::query::parser::ast::stmt::UseStmt {
                span: Span::default(),
                space: "test".to_string(),
            })),
            qctx,
        );
        assert!(result.is_err());
    }
}
