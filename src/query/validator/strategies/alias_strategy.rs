//! 别名验证策略
//! 负责验证表达式中的别名引用和可用性

use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::types::expression::ExpressionMeta;
use crate::core::types::expression::ExpressionContext;
use crate::core::types::expression::ExpressionId;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::validator::structs::AliasType;
use std::collections::HashMap;
use std::sync::Arc;

/// 别名验证策略
pub struct AliasValidationStrategy;

impl AliasValidationStrategy {
    pub fn new() -> Self {
        Self
    }

    /// 验证表达式列表中的别名
    pub fn validate_aliases(
        &self,
        exprs: &[ContextualExpression],
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        for expression in exprs {
            self.validate_expression_aliases(expression, aliases)?;
        }
        Ok(())
    }

    /// 验证单个表达式中的别名
    pub fn validate_expression_aliases(
        &self,
        expression: &ContextualExpression,
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        // 从 ContextualExpression 获取 Expression
        let expr_meta = match expression.expression() {
            Some(e) => e,
            None => return Ok(()),
        };
        let expr = expr_meta.inner();

        // 首先检查表达式本身是否引用了一个别名
        if let Some(alias_name) = self.extract_alias_name_internal(&expr) {
            if !aliases.contains_key(&alias_name) {
                return Err(ValidationError::new(
                    format!("未定义的变量别名: {}", alias_name),
                    ValidationErrorType::AliasError,
                ));
            }
        }

        // 递归验证子表达式
        self.validate_subexpressions_aliases_internal(&expr, aliases)?;

        Ok(())
    }

    /// 从表达式中提取别名名称
    pub fn extract_alias_name(&self, expression: &ContextualExpression) -> Option<String> {
        let expr_meta = match expression.expression() {
            Some(e) => e,
            None => return None,
        };
        self.extract_alias_name_internal(expr_meta.inner())
    }

    /// 内部方法：从表达式中提取别名名称
    fn extract_alias_name_internal(&self, expression: &crate::core::types::expression::Expression) -> Option<String> {
        match expression {
            crate::core::types::expression::Expression::Variable(name) => Some(name.clone()),
            crate::core::types::expression::Expression::Property { property, .. } => Some(property.clone()),
            crate::core::types::expression::Expression::Label(name) => Some(name.clone()),
            crate::core::types::expression::Expression::TagProperty { tag_name, property } => Some(format!("{}.{}", tag_name, property)),
            crate::core::types::expression::Expression::EdgeProperty { edge_name, property } => Some(format!("{}.{}", edge_name, property)),
            // 根据实际的表达式类型，可能需要处理其他别名引用
            _ => None,
        }
    }

    /// 递归验证子表达式中的别名
    fn validate_subexpressions_aliases(
        &self,
        expression: &ContextualExpression,
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        let expr_meta = match expression.expression() {
            Some(e) => e,
            None => return Ok(()),
        };
        self.validate_subexpressions_aliases_internal(expr_meta.inner(), aliases)
    }

    /// 内部方法：递归验证子表达式中的别名
    fn validate_subexpressions_aliases_internal(
        &self,
        expression: &crate::core::types::expression::Expression,
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        match expression {
            crate::core::types::expression::Expression::Unary { operand, .. } => {
                self.validate_expression_aliases_internal(operand, aliases)
            }
            crate::core::types::expression::Expression::Binary { left, right, .. } => {
                self.validate_expression_aliases_internal(left, aliases)?;
                self.validate_expression_aliases_internal(right, aliases)
            }
            crate::core::types::expression::Expression::Function { args, .. } => {
                for arg in args {
                    self.validate_expression_aliases_internal(arg, aliases)?;
                }
                Ok(())
            }
            crate::core::types::expression::Expression::List(items) => {
                for item in items {
                    self.validate_expression_aliases_internal(item, aliases)?;
                }
                Ok(())
            }
            crate::core::types::expression::Expression::Map(items) => {
                for (_, value) in items {
                    self.validate_expression_aliases_internal(value, aliases)?;
                }
                Ok(())
            }
            crate::core::types::expression::Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                if let Some(test_expression) = test_expr {
                    self.validate_expression_aliases_internal(test_expression, aliases)?;
                }
                for (condition, value) in conditions {
                    self.validate_expression_aliases_internal(condition, aliases)?;
                    self.validate_expression_aliases_internal(value, aliases)?;
                }
                if let Some(default_expression) = default {
                    self.validate_expression_aliases_internal(default_expression, aliases)?;
                }
                Ok(())
            }
            crate::core::types::expression::Expression::Subscript { collection, index } => {
                self.validate_expression_aliases_internal(collection, aliases)?;
                self.validate_expression_aliases_internal(index, aliases)
            }
            crate::core::types::expression::Expression::Literal(_)
            | crate::core::types::expression::Expression::Property { .. }
            | crate::core::types::expression::Expression::Variable(_)
            | crate::core::types::expression::Expression::Label(_)
            | crate::core::types::expression::Expression::ListComprehension { .. }
            | crate::core::types::expression::Expression::TagProperty { .. }
            | crate::core::types::expression::Expression::EdgeProperty { .. }
            | crate::core::types::expression::Expression::LabelTagProperty { .. }
            | crate::core::types::expression::Expression::Predicate { .. }
            | crate::core::types::expression::Expression::Reduce { .. }
            | crate::core::types::expression::Expression::PathBuild(_)
            | crate::core::types::expression::Expression::Parameter(_) => Ok(()),
            crate::core::types::expression::Expression::TypeCast { expression, .. } => {
                // 类型转换表达式需要验证其子表达式
                self.validate_expression_aliases_internal(expression, aliases)
            }
            crate::core::types::expression::Expression::Aggregate { arg, .. } => {
                // 聚合函数表达式需要验证其参数表达式
                self.validate_expression_aliases_internal(arg, aliases)
            }
            crate::core::types::expression::Expression::Range {
                collection,
                start,
                end,
            } => {
                // 范围访问表达式需要验证集合和范围表达式
                self.validate_expression_aliases_internal(collection, aliases)?;
                if let Some(start_expression) = start {
                    self.validate_expression_aliases_internal(start_expression, aliases)?;
                }
                if let Some(end_expression) = end {
                    self.validate_expression_aliases_internal(end_expression, aliases)?;
                }
                Ok(())
            }
            crate::core::types::expression::Expression::Path(items) => {
                // 路径表达式需要验证其所有项
                for item in items {
                    self.validate_expression_aliases_internal(item, aliases)?;
                }
                Ok(())
            }
        }
    }

    /// 内部方法：验证单个表达式中的别名
    fn validate_expression_aliases_internal(
        &self,
        expression: &crate::core::types::expression::Expression,
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        // 首先检查表达式本身是否引用了一个别名
        if let Some(alias_name) = self.extract_alias_name_internal(expression) {
            if !aliases.contains_key(&alias_name) {
                return Err(ValidationError::new(
                    format!("未定义的变量别名: {}", alias_name),
                    ValidationErrorType::AliasError,
                ));
            }
        }

        // 递归验证子表达式
        self.validate_subexpressions_aliases_internal(expression, aliases)
    }

    /// 检查别名类型是否匹配使用方式
    pub fn check_alias(
        &self,
        ref_expression: &ContextualExpression,
        aliases_available: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        // 提取表达式中的别名名称
        if let Some(alias_name) = self.extract_alias_name(ref_expression) {
            if !aliases_available.contains_key(&alias_name) {
                return Err(ValidationError::new(
                    format!("未定义的别名: {}", alias_name),
                    ValidationErrorType::AliasError,
                ));
            }
        }

        Ok(())
    }

    /// 结合别名
    pub fn combine_aliases(
        &self,
        cur_aliases: &mut HashMap<String, AliasType>,
        last_aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        for (name, alias_type) in last_aliases {
            if !cur_aliases.contains_key(name) {
                if cur_aliases
                    .insert(name.clone(), alias_type.clone())
                    .is_some()
                {
                    return Err(ValidationError::new(
                        format!("`{}': 重复定义的别名", name),
                        ValidationErrorType::AliasError,
                    ));
                }
            }
        }
        Ok(())
    }
}

impl AliasValidationStrategy {
    /// 获取策略名称
    pub fn strategy_name(&self) -> &'static str {
        "AliasValidationStrategy"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_alias_validation_strategy_creation() {
        let strategy = AliasValidationStrategy::new();
        assert_eq!(strategy.strategy_name(), "AliasValidationStrategy");
    }

    #[test]
    fn test_extract_alias_name() {
        let strategy = AliasValidationStrategy::new();

        // 测试从变量表达式中提取别名
        let var_expression = Expression::Variable("test_var".to_string());
        let meta = ExpressionMeta::new(var_expression);
        let expr_ctx = ExpressionContext::new();
        let id = expr_ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, Arc::new(expr_ctx));
        assert_eq!(
            strategy.extract_alias_name(&ctx_expr),
            Some("test_var".to_string())
        );

        // 测试从常量表达式中提取别名（应该返回None）
        let const_expression = Expression::Literal(crate::core::Value::Int(42));
        let meta = ExpressionMeta::new(const_expression);
        let expr_ctx = ExpressionContext::new();
        let id = expr_ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, Arc::new(expr_ctx));
        assert_eq!(strategy.extract_alias_name(&ctx_expr), None);
    }
}
