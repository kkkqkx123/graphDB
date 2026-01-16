//! 变量验证器
//! 负责验证变量的作用域、命名格式和使用

use crate::core::Expression;
use crate::query::validator::structs::*;
use crate::query::validator::{ValidationError, ValidationErrorType};
use crate::query::validator::validation_interface::ValidationContext;
use std::collections::HashMap;

/// 变量验证器
pub struct VariableValidator;

impl VariableValidator {
    pub fn new() -> Self {
        Self
    }

    /// 验证变量作用域
    pub fn validate_variable_scope<C: ValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
        available_aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        // 提取表达式中使用的变量
        let variables = self.extract_variables(expr);
        
        // 验证每个变量的作用域
        for var in &variables {
            self.validate_variable_usage(var, context, available_aliases)?;
        }
        
        Ok(())
    }

    /// 验证变量命名格式
    pub fn validate_variable_name_format(&self, var: &str) -> Result<(), ValidationError> {
        if var.is_empty() {
            return Err(ValidationError::new(
                "变量名不能为空".to_string(),
                ValidationErrorType::SyntaxError,
            ));
        }

        // 检查变量名格式
        let first_char = var.chars().next().ok_or_else(|| {
            ValidationError::new(
                "变量名不能为空".to_string(),
                ValidationErrorType::SyntaxError,
            )
        })?;
        if !first_char.is_alphabetic() && first_char != '_' {
            return Err(ValidationError::new(
                format!("变量名必须以字母或下划线开头: {:?}", var),
                ValidationErrorType::SyntaxError,
            ));
        }

        // 检查变量名是否只包含字母、数字和下划线
        if !var.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ValidationError::new(
                format!("变量名只能包含字母、数字和下划线: {:?}", var),
                ValidationErrorType::SyntaxError,
            ));
        }

        // 检查变量名长度
        if var.len() > 255 {
            return Err(ValidationError::new(
                format!("变量名太长: {:?}", var),
                ValidationErrorType::SyntaxError,
            ));
        }

        Ok(())
    }

    /// 验证变量使用
    fn validate_variable_usage<C: ValidationContext>(
        &self,
        var: &str,
        _context: &C,
        available_aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        // 首先验证变量名格式
        self.validate_variable_name_format(var)?;

        // 检查变量是否在可用别名中
        if !available_aliases.contains_key(var) {
            return Err(ValidationError::new(
                format!("变量 {:?} 未定义", var),
                ValidationErrorType::VariableNotFound,
            ));
        }

        Ok(())
    }

    /// 验证简单变量作用域
    pub fn validate_variable_scope_simple(
        &self,
        variables: &[String],
    ) -> Result<(), ValidationError> {
        for var in variables {
            self.validate_variable_name_format(var)?;
        }
        Ok(())
    }

    /// 提取表达式中的变量
    fn extract_variables(&self, expr: &Expression) -> Vec<String> {
        let mut variables = Vec::new();
        self.collect_variables(expr, &mut variables);
        variables
    }

    /// 递归收集变量
    fn collect_variables(&self, expr: &Expression, variables: &mut Vec<String>) {
        match expr {
            Expression::Variable(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            }
            Expression::Binary { left, right, .. } => {
                self.collect_variables(left, variables);
                self.collect_variables(right, variables);
            }
            Expression::Unary { operand, .. } => {
                self.collect_variables(operand, variables);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.collect_variables(arg, variables);
                }
            }
            Expression::Aggregate { arg, .. } => {
                self.collect_variables(arg, variables);
            }
            Expression::Property { object: inner_expr, .. } => {
                self.collect_variables(inner_expr, variables);
            }
            Expression::Subscript { collection: inner_expr, index } => {
                self.collect_variables(inner_expr, variables);
                self.collect_variables(index, variables);
            }
            Expression::List(items) => {
                for item in items {
                    self.collect_variables(item, variables);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.collect_variables(value, variables);
                }
            }
            Expression::Case {
                conditions,
                default,
            } => {
                for (when_expr, then_expr) in conditions {
                    self.collect_variables(when_expr, variables);
                    self.collect_variables(then_expr, variables);
                }
                if let Some(else_expr) = default {
                    self.collect_variables(else_expr, variables);
                }
            }
            _ => {}
        }
    }

    /// 检查是否包含指定变量
    pub fn contains_variable(&self, expr: &Expression, var: &str) -> bool {
        match expr {
            Expression::Variable(name) => name == var,
            Expression::Binary { left, right, .. } => {
                self.contains_variable(left, var) || self.contains_variable(right, var)
            }
            Expression::Unary { operand, .. } => self.contains_variable(operand, var),
            Expression::Function { args, .. } => {
                args.iter().any(|arg| self.contains_variable(arg, var))
            }
            Expression::Aggregate { arg, .. } => self.contains_variable(arg, var),
            Expression::Property { object: inner_expr, .. } => self.contains_variable(inner_expr, var),
            Expression::Subscript { collection: inner_expr, index } => {
                self.contains_variable(inner_expr, var) || self.contains_variable(index, var)
            }
            Expression::List(items) => items.iter().any(|item| self.contains_variable(item, var)),
            Expression::Map(pairs) => {
                pairs.iter().any(|(_, value)| self.contains_variable(value, var))
            }
            Expression::Case {
                conditions,
                default,
            } => {
                let mut has_var = false;
                for (when_expr, then_expr) in conditions {
                    has_var = has_var || self.contains_variable(when_expr, var);
                    has_var = has_var || self.contains_variable(then_expr, var);
                }
                if let Some(else_expr) = default {
                    has_var = has_var || self.contains_variable(else_expr, var);
                }
                has_var
            }
            _ => false,
        }
    }

    /// 验证表达式中的变量
    pub fn validate_expression_variables<C: ValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
    ) -> Result<(), ValidationError> {
        self.validate_variable_scope(expr, context, context.get_aliases())
    }

    /// 检查是否为算术表达式
    pub fn is_arithmetic_expression(&self, expr: &Expression, var: &str) -> bool {
        match expr {
            Expression::Binary { op, left, right } => {
                match op {
                    crate::core::BinaryOperator::Add
                    | crate::core::BinaryOperator::Subtract
                    | crate::core::BinaryOperator::Multiply
                    | crate::core::BinaryOperator::Divide
                    | crate::core::BinaryOperator::Modulo => {
                        self.contains_variable(left, var) || self.contains_variable(right, var)
                    }
                    _ => false,
                }
            }
            Expression::Unary { op, operand } => {
                match op {
                    crate::core::UnaryOperator::Minus | crate::core::UnaryOperator::Plus => {
                        self.contains_variable(operand, var)
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::core::Value;
    use std::collections::HashMap;

    #[test]
    fn test_variable_validator_creation() {
        let validator = VariableValidator::new();
        assert!(true);
    }

    #[test]
    fn test_validate_variable_name_format() {
        let validator = VariableValidator::new();
        
        // 有效变量名
        assert!(validator.validate_variable_name_format("var").is_ok());
        assert!(validator.validate_variable_name_format("var1").is_ok());
        assert!(validator.validate_variable_name_format("var_name").is_ok());
        assert!(validator.validate_variable_name_format("_var").is_ok());
        
        // 无效变量名
        assert!(validator.validate_variable_name_format("").is_err());
        assert!(validator.validate_variable_name_format("1var").is_err());
        assert!(validator.validate_variable_name_format("var-name").is_err());
        assert!(validator.validate_variable_name_format("var name").is_err());
    }

    #[test]
    fn test_contains_variable() {
        let validator = VariableValidator::new();
        
        let var_expr = Expression::Variable("test_var".to_string());
        assert!(validator.contains_variable(&var_expr, "test_var"));
        assert!(!validator.contains_variable(&var_expr, "other_var"));
        
        let literal_expr = Expression::Literal(Value::Int(42));
        assert!(!validator.contains_variable(&literal_expr, "test_var"));
    }

    #[test]
    fn test_is_arithmetic_expression() {
        let validator = VariableValidator::new();
        
        let add_expr = Expression::Binary {
            op: crate::core::BinaryOperator::Add,
            left: Box::new(Expression::Variable("var".to_string())),
            right: Box::new(Expression::Literal(Value::Int(1))),
        };
        assert!(validator.is_arithmetic_expression(&add_expr, "var"));
        
        let eq_expr = Expression::Binary {
            op: crate::core::BinaryOperator::Equal,
            left: Box::new(Expression::Variable("var".to_string())),
            right: Box::new(Expression::Literal(Value::Int(1))),
        };
        assert!(!validator.is_arithmetic_expression(&eq_expr, "var"));
    }

    #[test]
    fn test_extract_variables() {
        let validator = VariableValidator::new();
        
        let complex_expr = Expression::Binary {
            op: crate::core::BinaryOperator::Add,
            left: Box::new(Expression::Variable("var1".to_string())),
            right: Box::new(Expression::Variable("var2".to_string())),
        };
        
        let variables = validator.extract_variables(&complex_expr);
        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"var1".to_string()));
        assert!(variables.contains(&"var2".to_string()));
    }
}