//! Update Configs 语句验证器
//! 用于验证 UPDATE CONFIGS 语句
//! 参考 nebula-graph AdminValidator.cpp 中的 SetConfigValidator 实现

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::context::QueryContext;
use crate::query::parser::ast::stmt::UpdateConfigsStmt;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};

/// Update Configs 语句验证器
#[derive(Debug)]
pub struct UpdateConfigsValidator {
    module: Option<String>,
    config_name: String,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl UpdateConfigsValidator {
    /// 创建新的 UpdateConfigs 验证器
    pub fn new() -> Self {
        Self {
            module: None,
            config_name: String::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    /// 验证配置模块名
    fn validate_module(&self, module: &Option<String>) -> Result<(), ValidationError> {
        if let Some(ref m) = module {
            let valid_modules = ["GRAPH", "META", "STORAGE", "ALL"];
            let upper = m.to_uppercase();
            if !valid_modules.contains(&upper.as_str()) {
                return Err(ValidationError::new(
                    format!("Invalid config module: {}. Valid modules are: GRAPH, META, STORAGE, ALL", m),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// 验证配置名
    fn validate_config_name(&self, name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::new(
                "Config name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 检查配置名格式（只允许字母、数字、下划线）
        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ValidationError::new(
                format!("Invalid config name format: {}", name),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    /// 验证配置值
    fn validate_config_value(
        &self,
        value: &crate::core::types::expression::Expression,
    ) -> Result<(), ValidationError> {
        use crate::core::types::expression::Expression;

        // 配置值必须是常量表达式
        match value {
            Expression::Literal(_) => Ok(()),
            _ => Err(ValidationError::new(
                "Config value must be a constant expression".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    fn validate_impl(&mut self, stmt: &UpdateConfigsStmt) -> Result<(), ValidationError> {
        // 验证模块名
        self.validate_module(&stmt.module)?;

        // 验证配置名
        self.validate_config_name(&stmt.config_name)?;

        // 验证配置值
        self.validate_config_value(&stmt.config_value)?;

        // 保存信息
        self.module = stmt.module.clone();
        self.config_name = stmt.config_name.clone();

        // 设置输出列
        self.setup_outputs();

        Ok(())
    }

    fn setup_outputs(&mut self) {
        // UPDATE CONFIGS 输出更新结果
        self.outputs = vec![
            ColumnDef {
                name: "module".to_string(),
                type_: ValueType::String,
            },
            ColumnDef {
                name: "name".to_string(),
                type_: ValueType::String,
            },
            ColumnDef {
                name: "value".to_string(),
                type_: ValueType::String,
            },
        ];
    }
}

impl Default for UpdateConfigsValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for UpdateConfigsValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let update_configs_stmt = match stmt {
            crate::query::parser::ast::Stmt::UpdateConfigs(update_configs_stmt) => update_configs_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected UPDATE CONFIGS statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        self.validate_impl(update_configs_stmt)?;

        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::UpdateConfigs
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // UPDATE CONFIGS 是全局语句
        true
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
    use crate::core::Value;

    #[test]
    fn test_update_configs_validator_new() {
        let validator = UpdateConfigsValidator::new();
        assert_eq!(validator.statement_type(), StatementType::UpdateConfigs);
        assert!(validator.is_global_statement());
    }

    #[test]
    fn test_validate_module() {
        let validator = UpdateConfigsValidator::new();
        
        // 有效模块
        assert!(validator.validate_module(&Some("GRAPH".to_string())).is_ok());
        assert!(validator.validate_module(&Some("META".to_string())).is_ok());
        assert!(validator.validate_module(&Some("STORAGE".to_string())).is_ok());
        assert!(validator.validate_module(&Some("ALL".to_string())).is_ok());
        assert!(validator.validate_module(&None).is_ok());
        
        // 无效模块
        assert!(validator.validate_module(&Some("INVALID".to_string())).is_err());
    }

    #[test]
    fn test_validate_config_name() {
        let validator = UpdateConfigsValidator::new();
        
        // 有效配置名
        assert!(validator.validate_config_name("max_connections").is_ok());
        assert!(validator.validate_config_name("timeout_ms").is_ok());
        
        // 无效配置名
        assert!(validator.validate_config_name("").is_err());
        assert!(validator.validate_config_name("invalid-name").is_err());
        assert!(validator.validate_config_name("invalid.name").is_err());
    }

    #[test]
    fn test_validate_config_value() {
        let validator = UpdateConfigsValidator::new();

        // 有效配置值
        assert!(validator.validate_config_value(&Expression::Literal(Value::Int(100))).is_ok());
        assert!(validator.validate_config_value(&Expression::Literal(Value::Bool(true))).is_ok());

        // 无效配置值（非常量）
        assert!(validator.validate_config_value(&Expression::Variable("var".to_string())).is_err());
    }
}
