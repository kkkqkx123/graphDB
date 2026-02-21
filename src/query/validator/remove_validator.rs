//! Remove 语句验证器
//! 用于验证 REMOVE 语句（Cypher 风格的属性/标签删除）
//! 参考 nebula-graph MutateValidator.cpp 中的删除操作验证

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::context::ast::AstContext;
use crate::query::parser::ast::stmt::RemoveStmt;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};

/// Remove 语句验证器
#[derive(Debug)]
pub struct RemoveValidator {
    items: Vec<crate::core::types::expression::Expression>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl RemoveValidator {
    /// 创建新的 Remove 验证器
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    /// 验证移除项
    fn validate_remove_item(
        &self,
        item: &crate::core::types::expression::Expression,
    ) -> Result<(), ValidationError> {
        use crate::core::types::expression::Expression;

        match item {
            // 移除属性: REMOVE n.property
            Expression::Property { object, property } => {
                self.validate_property_access(object, property)
            }
            // 变量本身: REMOVE n (移除节点)
            Expression::Variable(var) => {
                self.validate_variable_remove(var)
            }
            _ => Err(ValidationError::new(
                format!("Invalid REMOVE expression: {:?}", item),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证属性访问移除
    fn validate_property_access(
        &self,
        object: &crate::core::types::expression::Expression,
        property: &str,
    ) -> Result<(), ValidationError> {
        use crate::core::types::expression::Expression;

        // 对象必须是变量
        if let Expression::Variable(var_name) = object {
            // 检查变量是否存在
            if !self.user_defined_vars.iter().any(|v| v == var_name) {
                return Err(ValidationError::new(
                    format!("Variable '{}' not defined", var_name),
                    ValidationErrorType::SemanticError,
                ));
            }
        } else {
            return Err(ValidationError::new(
                "REMOVE property target must be a variable".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 属性名不能为空
        if property.is_empty() {
            return Err(ValidationError::new(
                "Property name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    /// 验证变量移除（删除节点/边）
    fn validate_variable_remove(&self, var: &str) -> Result<(), ValidationError> {
        // 检查变量是否存在
        if !self.user_defined_vars.iter().any(|v| v == var) {
            return Err(ValidationError::new(
                format!("Variable '{}' not defined", var),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    fn validate_impl(&mut self, stmt: &RemoveStmt) -> Result<(), ValidationError> {
        // 验证至少有一个移除项
        if stmt.items.is_empty() {
            return Err(ValidationError::new(
                "REMOVE clause must have at least one item".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证每个移除项
        for item in &stmt.items {
            self.validate_remove_item(item)?;
        }

        // 保存信息
        self.items = stmt.items.clone();

        // 设置输出列
        self.setup_outputs();

        Ok(())
    }

    fn setup_outputs(&mut self) {
        // REMOVE 语句返回被移除的项数
        self.outputs = vec![
            ColumnDef {
                name: "removed_count".to_string(),
                type_: ValueType::Int,
            },
        ];
    }

    /// 设置输入列（从父查询传递的列）
    pub fn set_inputs(&mut self, inputs: Vec<ColumnDef>) {
        // 更新可用的用户定义变量
        self.user_defined_vars = inputs.iter().map(|c| c.name.clone()).collect();
        self.inputs = inputs;
    }
}

impl Default for RemoveValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for RemoveValidator {
    fn validate(&mut self, ast: &mut AstContext) -> Result<ValidationResult, ValidationError> {
        let stmt = ast.sentence.as_ref()
            .and_then(|s| s.as_remove())
            .ok_or_else(|| ValidationError::new(
                "Expected REMOVE statement".to_string(),
                ValidationErrorType::SemanticError,
            ))?;

        self.validate_impl(stmt)?;

        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Remove
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // REMOVE 不是全局语句
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
    use crate::core::types::expression::Expression;

    #[test]
    fn test_remove_validator_new() {
        let validator = RemoveValidator::new();
        assert_eq!(validator.statement_type(), StatementType::Remove);
        assert!(!validator.is_global_statement());
    }

    #[test]
    fn test_validate_property_access() {
        let mut validator = RemoveValidator::new();
        validator.user_defined_vars.push("n".to_string());

        // 有效的属性访问
        let obj = Expression::Variable("n".to_string());
        assert!(validator.validate_property_access(&obj, "name").is_ok());

        // 无效的属性名
        assert!(validator.validate_property_access(&obj, "").is_err());

        // 未定义的变量
        let obj2 = Expression::Variable("m".to_string());
        assert!(validator.validate_property_access(&obj2, "name").is_err());
    }

    #[test]
    fn test_validate_variable_remove() {
        let mut validator = RemoveValidator::new();
        validator.user_defined_vars.push("n".to_string());

        // 有效的变量
        assert!(validator.validate_variable_remove("n").is_ok());

        // 未定义的变量
        assert!(validator.validate_variable_remove("m").is_err());
    }
}
