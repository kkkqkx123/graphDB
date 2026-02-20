//! LOOKUP 语句验证器
//! 对应 NebulaGraph LookupValidator.h/.cpp 的功能
//! 验证 LOOKUP 语句的合法性

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::Stmt;
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

#[derive(Debug)]
pub struct LookupValidator {
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expression_props: ExpressionProps,
    user_defined_vars: Vec<String>,
    validated_result: Option<ValidatedLookup>,
    schema_manager: Option<Arc<dyn SchemaManager>>,
    lookup_target: LookupTarget,
    filter_expression: Option<Expression>,
    yield_columns: Vec<LookupYieldColumn>,
    is_yield_all: bool,
}

#[derive(Debug, Clone)]
pub struct LookupTarget {
    pub label: String,
    pub index_type: LookupIndexType,
    pub properties: HashMap<String, LookupProperty>,
}

#[derive(Debug, Clone)]
pub struct LookupProperty {
    pub name: String,
    pub type_: ValueType,
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
            lookup_target: LookupTarget {
                label: String::new(),
                index_type: LookupIndexType::None,
                properties: HashMap::new(),
            },
            filter_expression: None,
            yield_columns: Vec::new(),
            is_yield_all: false,
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    pub fn set_lookup_target(mut self, target: LookupTarget) -> Self {
        self.lookup_target = target;
        self
    }

    pub fn set_filter(mut self, filter: Expression) -> Self {
        self.filter_expression = Some(filter);
        self
    }

    pub fn set_yield_all(mut self) -> Self {
        self.is_yield_all = true;
        self
    }

    pub fn add_yield_column(mut self, col: LookupYieldColumn) -> Self {
        self.yield_columns.push(col);
        self
    }

    /// 验证 LOOKUP 目标
    fn validate_lookup_target(&self) -> Result<(), ValidationError> {
        if self.lookup_target.label.is_empty() {
            return Err(ValidationError::new(
                "LOOKUP must specify a label".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        match &self.lookup_target.index_type {
            LookupIndexType::None => {
                return Err(ValidationError::new(
                    format!("No index found for label '{}'", self.lookup_target.label),
                    ValidationErrorType::SemanticError,
                ));
            }
            LookupIndexType::Single(prop_name) => {
                if self.lookup_target.properties.get(prop_name).is_none() {
                    return Err(ValidationError::new(
                        format!("Index on property '{}' does not exist", prop_name),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            LookupIndexType::Composite(prop_names) => {
                for prop_name in prop_names {
                    if self.lookup_target.properties.get(prop_name).is_none() {
                        return Err(ValidationError::new(
                            format!("Index on property '{}' does not exist", prop_name),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// 验证过滤条件
    fn validate_filter(&self) -> Result<(), ValidationError> {
        if let Some(ref filter) = self.filter_expression {
            // 验证过滤器类型
            self.validate_filter_type(filter)?;

            // 检查是否包含聚合表达式
            if self.has_aggregate_expression(filter) {
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
        // 简化处理：假设过滤器返回布尔类型
        // 实际应该进行类型推导
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
    fn validate_yields(&self) -> Result<(), ValidationError> {
        if self.is_yield_all {
            return Ok(());
        }

        if self.yield_columns.is_empty() {
            return Err(ValidationError::new(
                "LOOKUP must have YIELD clause or YIELD *".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let mut seen_names: HashMap<String, usize> = HashMap::new();
        for col in &self.yield_columns {
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
    fn generate_output_columns(&mut self) {
        self.outputs.clear();
        if self.is_yield_all {
            self.outputs.push(ColumnDef {
                name: "*".to_string(),
                type_: ValueType::List,
            });
        } else {
            for col in &self.yield_columns {
                self.outputs.push(ColumnDef {
                    name: col.alias.clone().unwrap_or_else(|| col.name.clone()),
                    type_: ValueType::String,
                });
            }
        }
    }
}

impl Default for LookupValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for LookupValidator {
    fn validate(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<ValidationResult, ValidationError> {
        // 1. 检查是否需要空间
        if !self.is_global_statement(ast) && query_context.is_none() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 获取 LOOKUP 语句（如果存在）
        if let Some(ref stmt) = ast.sentence() {
            if let Stmt::Lookup(lookup_stmt) = stmt {
                // 可以从 lookup_stmt 中提取信息
                // 这里简化处理，使用预设的值
                if self.lookup_target.label.is_empty() && !lookup_stmt.label.is_empty() {
                    self.lookup_target.label = lookup_stmt.label.clone();
                }
            }
        }

        // 3. 验证 LOOKUP 目标
        self.validate_lookup_target()?;

        // 4. 验证过滤条件
        self.validate_filter()?;

        // 5. 验证 YIELD 子句
        self.validate_yields()?;

        // 6. 获取 space_id
        let space_id = query_context
            .map(|qc| qc.space_id())
            .filter(|&id| id != 0)
            .or_else(|| ast.space().space_id.map(|id| id as u64))
            .unwrap_or(0);

        // 7. 创建验证结果
        let validated = ValidatedLookup {
            space_id,
            label: self.lookup_target.label.clone(),
            index_type: self.lookup_target.index_type.clone(),
            filter_expression: self.filter_expression.clone(),
            yield_columns: self.yield_columns.clone(),
            is_yield_all: self.is_yield_all,
        };

        self.validated_result = Some(validated);

        // 8. 生成输出列
        self.generate_output_columns();

        // 9. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Lookup
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

    fn create_lookup_target(label: &str, index_type: LookupIndexType) -> LookupTarget {
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), LookupProperty {
            name: "name".to_string(),
            type_: ValueType::String,
        });
        properties.insert("age".to_string(), LookupProperty {
            name: "age".to_string(),
            type_: ValueType::Int,
        });

        LookupTarget {
            label: label.to_string(),
            index_type,
            properties,
        }
    }

    #[test]
    fn test_lookup_validator_basic() {
        let mut validator = LookupValidator::new()
            .set_lookup_target(create_lookup_target("person", LookupIndexType::Single("name".to_string())))
            .set_yield_all();

        let mut ast = AstContext::default();
        let result = validator.validate(None, &mut ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lookup_validator_empty_label() {
        let mut validator = LookupValidator::new()
            .set_lookup_target(create_lookup_target("", LookupIndexType::None))
            .set_yield_all();

        let mut ast = AstContext::default();
        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("must specify a label"));
    }

    #[test]
    fn test_lookup_validator_no_index() {
        let mut validator = LookupValidator::new()
            .set_lookup_target(create_lookup_target("person", LookupIndexType::None))
            .set_yield_all();

        let mut ast = AstContext::default();
        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("No index found"));
    }

    #[test]
    fn test_lookup_validator_invalid_index() {
        let mut validator = LookupValidator::new()
            .set_lookup_target(create_lookup_target("person", LookupIndexType::Single("invalid".to_string())))
            .set_yield_all();

        let mut ast = AstContext::default();
        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("does not exist"));
    }

    #[test]
    fn test_lookup_validator_no_yield() {
        let mut validator = LookupValidator::new()
            .set_lookup_target(create_lookup_target("person", LookupIndexType::Single("name".to_string())));

        let mut ast = AstContext::default();
        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("must have YIELD clause"));
    }

    #[test]
    fn test_lookup_validator_with_yield_columns() {
        let mut validator = LookupValidator::new()
            .set_lookup_target(create_lookup_target("person", LookupIndexType::Single("name".to_string())))
            .add_yield_column(LookupYieldColumn {
                name: "name".to_string(),
                alias: None,
                expression: None,
            })
            .add_yield_column(LookupYieldColumn {
                name: "age".to_string(),
                alias: Some("user_age".to_string()),
                expression: None,
            });

        let mut ast = AstContext::default();
        let result = validator.validate(None, &mut ast);
        assert!(result.is_ok());

        let outputs = validator.outputs();
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].name, "name");
        assert_eq!(outputs[1].name, "user_age");
    }

    #[test]
    fn test_lookup_validator_duplicate_columns() {
        let mut validator = LookupValidator::new()
            .set_lookup_target(create_lookup_target("person", LookupIndexType::Single("name".to_string())))
            .add_yield_column(LookupYieldColumn {
                name: "name".to_string(),
                alias: None,
                expression: None,
            })
            .add_yield_column(LookupYieldColumn {
                name: "name".to_string(),
                alias: None,
                expression: None,
            });

        let mut ast = AstContext::default();
        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Duplicate column name"));
    }

    #[test]
    fn test_lookup_validator_with_filter() {
        let filter = Expression::Binary {
            left: Box::new(Expression::Variable("name".to_string())),
            op: crate::core::BinaryOperator::Equal,
            right: Box::new(Expression::literal("Alice")),
        };

        let mut validator = LookupValidator::new()
            .set_lookup_target(create_lookup_target("person", LookupIndexType::Single("name".to_string())))
            .set_filter(filter)
            .set_yield_all();

        let mut ast = AstContext::default();
        let result = validator.validate(None, &mut ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lookup_validator_with_aggregate_filter() {
        let filter = Expression::Aggregate {
            func: crate::core::AggregateFunction::Count,
            arg: Box::new(Expression::Variable("name".to_string())),
            distinct: false,
        };

        let mut validator = LookupValidator::new()
            .set_lookup_target(create_lookup_target("person", LookupIndexType::Single("name".to_string())))
            .set_filter(filter)
            .set_yield_all();

        let mut ast = AstContext::default();
        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("cannot contain aggregate expressions"));
    }

    #[test]
    fn test_lookup_validator_trait_interface() {
        let validator = LookupValidator::new();

        assert_eq!(validator.statement_type(), StatementType::Lookup);
        assert!(validator.inputs().is_empty());
        assert!(validator.user_defined_vars().is_empty());
    }

    #[test]
    fn test_lookup_validator_composite_index() {
        let mut properties = HashMap::new();
        properties.insert("first_name".to_string(), LookupProperty {
            name: "first_name".to_string(),
            type_: ValueType::String,
        });
        properties.insert("last_name".to_string(), LookupProperty {
            name: "last_name".to_string(),
            type_: ValueType::String,
        });

        let target = LookupTarget {
            label: "person".to_string(),
            index_type: LookupIndexType::Composite(vec!["first_name".to_string(), "last_name".to_string()]),
            properties,
        };

        let mut validator = LookupValidator::new()
            .set_lookup_target(target)
            .set_yield_all();

        let mut ast = AstContext::default();
        let result = validator.validate(None, &mut ast);
        assert!(result.is_ok());
    }
}
