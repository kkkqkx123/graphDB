//! FIND PATH 语句验证器 - 新体系版本
//! 对应 NebulaGraph FindPathValidator.h/.cpp 的功能
//! 验证 FIND PATH 语句的合法性

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::context::ast::AstContext;
use crate::query::parser::ast::stmt::FindPathStmt;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的路径查找信息
#[derive(Debug, Clone)]
pub struct ValidatedFindPath {
    pub space_id: u64,
    pub from: crate::query::parser::ast::stmt::FromClause,
    pub to: crate::core::Expression,
    pub over: Option<crate::query::parser::ast::stmt::OverClause>,
    pub where_clause: Option<crate::core::Expression>,
    pub shortest: bool,
    pub max_steps: Option<usize>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub yield_clause: Option<crate::query::parser::ast::stmt::YieldClause>,
    pub weight_expression: Option<String>,
    pub heuristic_expression: Option<String>,
    pub with_loop: bool,
    pub with_cycle: bool,
}

/// FIND PATH 验证器 - 新体系实现
#[derive(Debug)]
pub struct FindPathValidator {
    schema_manager: Option<Arc<dyn SchemaManager>>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
    validated_result: Option<ValidatedFindPath>,
}

impl FindPathValidator {
    pub fn new() -> Self {
        Self {
            schema_manager: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validated_result: None,
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    pub fn validated_result(&self) -> Option<&ValidatedFindPath> {
        self.validated_result.as_ref()
    }

    fn validate_find_path(&self, stmt: &FindPathStmt) -> Result<(), ValidationError> {
        // 验证 FROM 子句
        if stmt.from.vertices.is_empty() {
            return Err(ValidationError::new(
                "FIND PATH must specify source vertices in FROM clause".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        
        // 验证步数限制
        if let Some(max_steps) = stmt.max_steps {
            if max_steps > 100 {
                return Err(ValidationError::new(
                    "Maximum steps cannot exceed 100".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        
        Ok(())
    }

    fn validate_yield_clause(&self, yield_clause: &Option<crate::query::parser::ast::stmt::YieldClause>) -> Result<(), ValidationError> {
        if let Some(ref yc) = yield_clause {
            let mut seen_names: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            for item in &yc.items {
                let name = item.alias.clone()
                    .unwrap_or_else(|| format!("{:?}", item.expression));
                let count = seen_names.entry(name.clone()).or_insert(0);
                *count += 1;
                if *count > 1 {
                    return Err(ValidationError::new(
                        format!("Duplicate column name '{}' in YIELD clause", name),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        Ok(())
    }
}

impl Default for FindPathValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for FindPathValidator {
    fn validate(&mut self, ast: &mut AstContext) -> Result<ValidationResult, ValidationError> {
        // 1. 检查是否需要空间
        let query_context = ast.query_context();
        if !self.is_global_statement() && query_context.is_none() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 获取 FIND PATH 语句
        let stmt = ast.sentence()
            .ok_or_else(|| ValidationError::new(
                "No statement found in AST context".to_string(),
                ValidationErrorType::SemanticError,
            ))?;

        let find_path_stmt = match stmt {
            crate::query::parser::ast::Stmt::FindPath(find_path_stmt) => find_path_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected FIND PATH statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 3. 执行基础验证
        self.validate_find_path(find_path_stmt)?;

        // 4. 验证 YIELD 子句
        self.validate_yield_clause(&find_path_stmt.yield_clause)?;

        // 5. 获取 space_id
        let space_id = query_context
            .map(|qc| qc.space_id())
            .filter(|&id| id != 0)
            .or_else(|| ast.space().space_id.map(|id| id as u64))
            .unwrap_or(0);

        // 6. 创建验证结果
        let validated = ValidatedFindPath {
            space_id,
            from: find_path_stmt.from.clone(),
            to: find_path_stmt.to.clone(),
            over: find_path_stmt.over.clone(),
            where_clause: find_path_stmt.where_clause.clone(),
            shortest: find_path_stmt.shortest,
            max_steps: find_path_stmt.max_steps,
            limit: find_path_stmt.limit,
            offset: find_path_stmt.offset,
            yield_clause: find_path_stmt.yield_clause.clone(),
            weight_expression: find_path_stmt.weight_expression.clone(),
            heuristic_expression: find_path_stmt.heuristic_expression.clone(),
            with_loop: find_path_stmt.with_loop,
            with_cycle: find_path_stmt.with_cycle,
        };

        // 7. 设置输出列
        self.outputs.clear();
        if let Some(ref yc) = find_path_stmt.yield_clause {
            for item in &yc.items {
                let col_name = item.alias.clone()
                    .unwrap_or_else(|| format!("{:?}", item.expression));
                self.outputs.push(ColumnDef {
                    name: col_name,
                    type_: ValueType::Path,
                });
            }
        }

        self.validated_result = Some(validated);

        // 8. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::FindPath
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // FIND PATH 不是全局语句，需要预先选择空间
        false
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
    use crate::query::parser::ast::Span;

    #[test]
    fn test_find_path_validator_new() {
        let validator = FindPathValidator::new();
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
    }

    #[test]
    fn test_statement_type() {
        let validator = FindPathValidator::new();
        assert_eq!(validator.statement_type(), StatementType::FindPath);
    }

    #[test]
    fn test_is_global_statement() {
        let validator = FindPathValidator::new();
        assert!(!validator.is_global_statement());
    }
}
