//! LOOKUP 语句验证器
//! 对应 NebulaGraph LookupValidator.h/.cpp 的功能
//! 验证 LOOKUP 语句的合法性

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::parser::ast::{Stmt, YieldItem};
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};
use std::collections::HashMap;
use std::sync::Arc;
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的 LOOKUP 信息
#[derive(Debug, Clone)]
pub struct ValidatedLookup {
    pub space_id: u64,
    pub label: String,
    pub is_edge: bool,
    pub index_type: LookupIndexType,
    pub filter_expression: Option<Expression>,
    pub yield_columns: Vec<LookupYieldColumn>,
    pub is_yield_all: bool,
}

#[derive(Debug, Clone)]
pub struct LookupYieldColumn {
    pub name: String,
    pub alias: Option<String>,
    pub expression: Option<Expression>,
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

/// LOOKUP 验证器
/// 完全从 AST 解析 LOOKUP 语句，不依赖外部预设值
#[derive(Debug)]
pub struct LookupValidator {
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expression_props: ExpressionProps,
    user_defined_vars: Vec<String>,
    validated_result: Option<ValidatedLookup>,
    schema_manager: Option<Arc<dyn SchemaManager>>,
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

    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    /// 从 AST 解析 LOOKUP 语句
    fn parse_from_ast(
        &self,
        ast: &AstContext,
    ) -> Result<ParsedLookupInfo, ValidationError> {
        let stmt = ast.sentence().ok_or_else(|| {
            ValidationError::new(
                "AST 中未找到语句".to_string(),
                ValidationErrorType::SemanticError,
            )
        })?;

        let lookup_stmt = match stmt {
            Stmt::Lookup(lookup_stmt) => lookup_stmt,
            _ => {
                return Err(ValidationError::new(
                    "期望 LOOKUP 语句".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 解析目标（Tag 或 Edge）
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

        // 解析 WHERE 子句
        let filter_expression = lookup_stmt.where_clause.clone();

        // 解析 YIELD 子句
        let mut yield_columns = Vec::new();
        let mut is_yield_all = false;

        if let Some(ref yield_clause) = lookup_stmt.yield_clause {
            for item in &yield_clause.items {
                yield_columns.push(self.parse_yield_item(item)?);
            }
            // 检查是否是 YIELD *
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

    /// 解析单个 YIELD 项
    fn parse_yield_item(
        &self,
        item: &YieldItem,
    ) -> Result<LookupYieldColumn, ValidationError> {
        let name = self.extract_column_name(&item.expression)?;
        Ok(LookupYieldColumn {
            name,
            alias: item.alias.clone(),
            expression: Some(item.expression.clone()),
        })
    }

    /// 从表达式中提取列名
    fn extract_column_name(&self, expr: &Expression) -> Result<String, ValidationError> {
        match expr {
            Expression::Variable(name) => Ok(name.clone()),
            Expression::Label(name) => Ok(name.clone()),
            Expression::Property { property, .. } => Ok(property.clone()),
            Expression::Literal(value) => Ok(format!("{:?}", value)),
            _ => Ok(format!("{:?}", expr)),
        }
    }

    /// 验证 LOOKUP 目标
    /// 对应 NebulaGraph 的 validateFrom() 方法
    fn validate_lookup_target(
        &self,
        space_name: &str,
        label: &str,
        is_edge: bool,
    ) -> Result<LookupIndexType, ValidationError> {
        // 检查 schema_manager 是否可用
        let schema_manager = self.schema_manager.as_ref().ok_or_else(|| {
            ValidationError::new(
                "Schema manager not available".to_string(),
                ValidationErrorType::SemanticError,
            )
        })?;

        if is_edge {
            // 验证 Edge Type 是否存在
            match schema_manager.get_edge_type(space_name, label) {
                Ok(Some(_edge_info)) => {
                    // Edge Type 存在，返回 Single 索引类型
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
            // 验证 Tag 是否存在
            match schema_manager.get_tag(space_name, label) {
                Ok(Some(_tag_info)) => {
                    // Tag 存在，返回 Single 索引类型
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

    /// 验证过滤条件
    fn validate_filter(&self, filter: &Option<Expression>) -> Result<(), ValidationError> {
        if let Some(ref filter_expr) = filter {
            // 验证过滤器类型
            self.validate_filter_type(filter_expr)?;

            // 检查是否包含聚合表达式
            if self.has_aggregate_expression(filter_expr) {
                return Err(ValidationError::new(
                    "LOOKUP filter cannot contain aggregate expressions".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// 验证过滤器类型
    fn validate_filter_type(&self, filter: &Expression) -> Result<(), ValidationError> {
        match filter {
            Expression::Binary { op, .. } => {
                use crate::core::BinaryOperator;
                match op {
                    BinaryOperator::Equal | BinaryOperator::NotEqual |
                    BinaryOperator::LessThan | BinaryOperator::LessThanOrEqual |
                    BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual |
                    BinaryOperator::And | BinaryOperator::Or => Ok(()),
                    _ => Err(ValidationError::new(
                        "Filter expression must return bool type".to_string(),
                        ValidationErrorType::TypeError,
                    )),
                }
            }
            _ => Ok(()),
        }
    }

    /// 检查是否包含聚合表达式
    fn has_aggregate_expression(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Aggregate { .. } => true,
            Expression::Binary { left, right, .. } => {
                self.has_aggregate_expression(left) || self.has_aggregate_expression(right)
            }
            Expression::Unary { operand, .. } => {
                self.has_aggregate_expression(operand)
            }
            Expression::Function { args, .. } => {
                args.iter().any(|arg| self.has_aggregate_expression(arg))
            }
            _ => false,
        }
    }

    /// 验证 YIELD 子句
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

    /// 生成输出列
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

/// 从 AST 解析的 LOOKUP 信息
#[derive(Debug)]
struct ParsedLookupInfo {
    label: String,
    is_edge: bool,
    filter_expression: Option<Expression>,
    yield_columns: Vec<LookupYieldColumn>,
    is_yield_all: bool,
}

impl Default for LookupValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for LookupValidator {
    fn validate(&mut self, ast: &mut AstContext) -> Result<ValidationResult, ValidationError> {
        // 1. 检查是否需要空间
        let query_context = ast.query_context();
        if !self.is_global_statement() && query_context.is_none() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 从 AST 解析 LOOKUP 语句
        let parsed_info = self.parse_from_ast(ast)?;

        // 3. 获取当前空间名称
        let space_name = query_context
            .and_then(|qc| {
                if qc.vctx().space_chosen() {
                    Some(qc.vctx().which_space().space_name.clone())
                } else {
                    None
                }
            })
            .or_else(|| Some(ast.space().space_name.clone()))
            .unwrap_or_default();

        if space_name.is_empty() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 4. 验证 LOOKUP 目标
        let index_type = self.validate_lookup_target(
            &space_name,
            &parsed_info.label,
            parsed_info.is_edge,
        )?;

        // 4. 验证过滤条件
        self.validate_filter(&parsed_info.filter_expression)?;

        // 5. 验证 YIELD 子句
        self.validate_yields(&parsed_info.yield_columns, parsed_info.is_yield_all)?;

        // 6. 获取 space_id
        let space_id = query_context
            .map(|qc| qc.space_id())
            .filter(|&id| id != 0)
            .or_else(|| ast.space().space_id.map(|id| id as u64))
            .unwrap_or(0);

        // 7. 创建验证结果
        let validated = ValidatedLookup {
            space_id,
            label: parsed_info.label,
            is_edge: parsed_info.is_edge,
            index_type,
            filter_expression: parsed_info.filter_expression,
            yield_columns: parsed_info.yield_columns.clone(),
            is_yield_all: parsed_info.is_yield_all,
        };

        self.validated_result = Some(validated);

        // 8. 生成输出列
        self.outputs = self.generate_output_columns(
            &parsed_info.yield_columns,
            parsed_info.is_yield_all,
        );

        // 9. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Lookup
    }

    fn is_global_statement(&self) -> bool {
        // LOOKUP 不是全局语句，需要预先选择空间
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
    use crate::query::parser::ast::stmt::{LookupStmt, LookupTarget, YieldClause};
    use crate::query::parser::ast::Span;

    fn create_test_ast_with_lookup(lookup_stmt: LookupStmt) -> AstContext {
        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Lookup(lookup_stmt));
        ast
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
        let mut ast = create_test_ast_with_lookup(lookup_stmt);

        let result = validator.validate(None, &mut ast);
        // 当前会失败，因为没有 YIELD 列且不是 YIELD *
        assert!(result.is_err());
    }

    #[test]
    fn test_lookup_validator_empty_label() {
        let mut validator = LookupValidator::new();
        let lookup_stmt = create_simple_lookup_stmt("", false);
        let mut ast = create_test_ast_with_lookup(lookup_stmt);

        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("必须指定"));
    }

    #[test]
    fn test_lookup_validator_not_lookup_stmt() {
        let mut validator = LookupValidator::new();
        let mut ast = AstContext::default();
        // 不设置 LOOKUP 语句

        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
    }
}
