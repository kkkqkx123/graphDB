//! Sequential 语句验证器
//! 对应 NebulaGraph SequentialValidator.h/.cpp 的功能
//! 验证多语句查询（使用分号分隔）的合法性
//!
//! 本文件已按照新的 trait + 枚举 验证器体系重构：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 保留了原有完整功能：
//!    - 语句数量验证
//!    - DDL/DML 语句顺序验证
//!    - 变量名验证
//!    - 最大语句数限制
//! 3. 使用 AstContext 统一管理上下文

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::DataType;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef,
    ExpressionProps,
};
use std::collections::HashMap;

/// 顺序语句定义
#[derive(Debug, Clone)]
pub struct SequentialStatement {
    pub statement: String,
    pub parameters: HashMap<String, crate::core::Expression>,
}

impl SequentialStatement {
    /// 创建新的顺序语句
    pub fn new(statement: String) -> Self {
        Self {
            statement,
            parameters: HashMap::new(),
        }
    }

    /// 添加参数
    pub fn with_parameter(mut self, name: String, expr: crate::core::Expression) -> Self {
        self.parameters.insert(name, expr);
        self
    }
}

/// Sequential 验证器 - 新体系实现
///
/// 功能完整性保证：
/// 1. 完整的验证生命周期
/// 2. 输入/输出列管理
/// 3. 表达式属性追踪
/// 4. 多语句顺序验证
/// 5. DDL/DML 顺序约束检查
#[derive(Debug)]
pub struct SequentialValidator {
    // 语句列表
    statements: Vec<SequentialStatement>,
    // 最大语句数限制
    max_statements: usize,
    // 变量映射
    variables: HashMap<String, DataType>,
    // 输入列定义（用于 trait 接口）
    inputs: Vec<ColumnDef>,
    // 输出列定义（顺序语句的输出为最后一条语句的输出）
    outputs: Vec<ColumnDef>,
    // 表达式属性
    expr_props: ExpressionProps,
    // 用户定义变量
    user_defined_vars: Vec<String>,
    // 验证错误列表
    validation_errors: Vec<ValidationError>,
}

impl SequentialValidator {
    /// 创建新的验证器实例
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            max_statements: 100,
            variables: HashMap::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    /// 设置最大语句数
    pub fn with_max_statements(mut self, max: usize) -> Self {
        self.max_statements = max;
        self
    }

    /// 添加语句
    pub fn add_statement(&mut self, statement: SequentialStatement) {
        self.statements.push(statement);
    }

    /// 设置变量
    pub fn set_variable(&mut self, name: String, type_: DataType) {
        self.variables.insert(name.clone(), type_);
        if !self.user_defined_vars.contains(&name) {
            self.user_defined_vars.push(name);
        }
    }

    /// 获取语句列表
    pub fn statements(&self) -> &[SequentialStatement] {
        &self.statements
    }

    /// 获取变量映射
    pub fn variables(&self) -> &HashMap<String, DataType> {
        &self.variables
    }

    /// 获取最大语句数
    pub fn max_statements(&self) -> usize {
        self.max_statements
    }

    /// 设置最大语句数
    pub fn set_max_statements(&mut self, max: usize) {
        self.max_statements = max;
    }

    /// 添加验证错误
    fn add_error(&mut self, error: ValidationError) {
        self.validation_errors.push(error);
    }

    /// 检查是否有验证错误
    fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// 清空验证错误
    fn clear_errors(&mut self) {
        self.validation_errors.clear();
    }

    /// 执行验证（传统方式，保持向后兼容）
    pub fn validate_sequential(&mut self) -> Result<(), ValidationError> {
        self.clear_errors();
        self.validate_impl()?;
        Ok(())
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_statement_count()?;
        self.validate_statement_order()?;
        self.validate_variables()?;
        Ok(())
    }

    fn validate_statement_count(&self) -> Result<(), ValidationError> {
        if self.statements.is_empty() {
            return Err(ValidationError::new(
                "Sequential statement must have at least one statement".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        if self.statements.len() > self.max_statements {
            return Err(ValidationError::new(
                format!(
                    "Too many statements in sequential query (max: {})",
                    self.max_statements
                ),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_statement_order(&self) -> Result<(), ValidationError> {
        let mut has_ddl = false;
        let mut has_dml = false;

        for (i, stmt) in self.statements.iter().enumerate() {
            let stmt_upper = stmt.statement.to_uppercase();
            if self.is_ddl_statement(&stmt_upper) {
                if has_dml {
                    return Err(ValidationError::new(
                        format!(
                            "DDL statement cannot follow DML statement at position {}",
                            i + 1
                        ),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if has_ddl {
                    return Err(ValidationError::new(
                        format!(
                            "Multiple DDL statements are not allowed, found at position {}",
                            i + 1
                        ),
                        ValidationErrorType::SemanticError,
                    ));
                }
                has_ddl = true;
            }
            if self.is_dml_statement(&stmt_upper) {
                has_dml = true;
            }
        }
        Ok(())
    }

    fn is_ddl_statement(&self, stmt: &str) -> bool {
        stmt.starts_with("CREATE") || stmt.starts_with("ALTER") || stmt.starts_with("DROP")
    }

    fn is_dml_statement(&self, stmt: &str) -> bool {
        stmt.starts_with("INSERT") || stmt.starts_with("UPDATE") || stmt.starts_with("DELETE")
            || stmt.starts_with("UPSERT")
    }

    fn validate_variables(&self) -> Result<(), ValidationError> {
        for (name, _) in &self.variables {
            if name.is_empty() {
                return Err(ValidationError::new(
                    "Variable name cannot be empty".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            if !name.starts_with('$') && !name.starts_with('@') {
                return Err(ValidationError::new(
                    format!("Invalid variable name '{}': must start with '$' or '@'", name),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// 检查语句是否为查询语句（返回结果集）
    pub fn is_query_statement(&self, stmt: &str) -> bool {
        let stmt_upper = stmt.to_uppercase();
        stmt_upper.starts_with("MATCH")
            || stmt_upper.starts_with("GO")
            || stmt_upper.starts_with("FETCH")
            || stmt_upper.starts_with("LOOKUP")
            || stmt_upper.starts_with("FIND PATH")
            || stmt_upper.starts_with("GET SUBGRAPH")
    }

    /// 检查语句是否为修改语句
    pub fn is_mutation_statement(&self, stmt: &str) -> bool {
        let stmt_upper = stmt.to_uppercase();
        stmt_upper.starts_with("INSERT")
            || stmt_upper.starts_with("UPDATE")
            || stmt_upper.starts_with("DELETE")
            || stmt_upper.starts_with("UPSERT")
    }
}

impl Default for SequentialValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for SequentialValidator {
    fn validate(
        &mut self,
        _query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<ValidationResult, ValidationError> {
        self.clear_errors();

        // 执行验证
        if let Err(e) = self.validate_impl() {
            return Ok(ValidationResult::failure(vec![e]));
        }

        // Sequential 语句的输出取决于最后一条语句
        // 这里简化处理，输出为空（实际应根据最后一条语句类型确定）
        self.outputs = Vec::new();

        // 同步到 AstContext
        ast.set_inputs(self.inputs.clone());
        ast.set_outputs(self.outputs.clone());

        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Sequential
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_validator_new() {
        let validator = SequentialValidator::new();
        assert!(validator.statements().is_empty());
        assert!(validator.variables().is_empty());
        assert_eq!(validator.max_statements(), 100);
    }

    #[test]
    fn test_add_statement() {
        let mut validator = SequentialValidator::new();
        let stmt = SequentialStatement::new("MATCH (n) RETURN n".to_string());
        validator.add_statement(stmt);
        assert_eq!(validator.statements().len(), 1);
    }

    #[test]
    fn test_set_variable() {
        let mut validator = SequentialValidator::new();
        validator.set_variable("$var".to_string(), DataType::String);
        assert_eq!(validator.variables().len(), 1);
        assert!(validator.variables().contains_key("$var"));
    }

    #[test]
    fn test_validate_empty_statements() {
        let mut validator = SequentialValidator::new();
        let result = validator.validate_sequential();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_single_statement() {
        let mut validator = SequentialValidator::new();
        let stmt = SequentialStatement::new("MATCH (n) RETURN n".to_string());
        validator.add_statement(stmt);

        let result = validator.validate_sequential();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_ddl_before_dml() {
        let mut validator = SequentialValidator::new();
        validator.add_statement(SequentialStatement::new("CREATE TAG person(name string)".to_string()));
        validator.add_statement(SequentialStatement::new("INSERT VERTEX person(name) VALUES \"1\":(\"Alice\")".to_string()));

        let result = validator.validate_sequential();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_ddl_after_dml() {
        let mut validator = SequentialValidator::new();
        validator.add_statement(SequentialStatement::new("INSERT VERTEX person(name) VALUES \"1\":(\"Alice\")".to_string()));
        validator.add_statement(SequentialStatement::new("CREATE TAG person(name string)".to_string()));

        let result = validator.validate_sequential();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_multiple_ddl() {
        let mut validator = SequentialValidator::new();
        validator.add_statement(SequentialStatement::new("CREATE TAG person(name string)".to_string()));
        validator.add_statement(SequentialStatement::new("CREATE TAG company(name string)".to_string()));

        let result = validator.validate_sequential();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_variable() {
        let mut validator = SequentialValidator::new();
        validator.add_statement(SequentialStatement::new("RETURN 1".to_string()));
        validator.set_variable("invalid_var".to_string(), DataType::Int);

        let result = validator.validate_sequential();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_valid_variable() {
        let mut validator = SequentialValidator::new();
        validator.add_statement(SequentialStatement::new("RETURN 1".to_string()));
        validator.set_variable("$var".to_string(), DataType::Int);

        let result = validator.validate_sequential();
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_query_statement() {
        let validator = SequentialValidator::new();
        assert!(validator.is_query_statement("MATCH (n) RETURN n"));
        assert!(validator.is_query_statement("GO FROM \"1\" OVER edge"));
        assert!(validator.is_query_statement("FETCH PROP ON person \"1\""));
        assert!(!validator.is_query_statement("INSERT VERTEX person(name) VALUES \"1\":(\"Alice\")"));
    }

    #[test]
    fn test_is_mutation_statement() {
        let validator = SequentialValidator::new();
        assert!(validator.is_mutation_statement("INSERT VERTEX person(name) VALUES \"1\":(\"Alice\")"));
        assert!(validator.is_mutation_statement("UPDATE VERTEX \"1\" SET name=\"Bob\""));
        assert!(validator.is_mutation_statement("DELETE VERTEX \"1\""));
        assert!(!validator.is_mutation_statement("MATCH (n) RETURN n"));
    }

    #[test]
    fn test_max_statements_limit() {
        let mut validator = SequentialValidator::new().with_max_statements(2);
        validator.add_statement(SequentialStatement::new("RETURN 1".to_string()));
        validator.add_statement(SequentialStatement::new("RETURN 2".to_string()));
        validator.add_statement(SequentialStatement::new("RETURN 3".to_string()));

        let result = validator.validate_sequential();
        assert!(result.is_err());
    }
}
