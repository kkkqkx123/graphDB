//! 别名验证策略
//! 负责验证表达式中的别名引用和可用性

use super::super::structs::*;
use super::super::validation_interface::*;
use crate::core::Expr;
use std::collections::HashMap;

/// 别名验证策略
pub struct AliasValidationStrategy;

impl AliasValidationStrategy {
    pub fn new() -> Self {
        Self
    }

    /// 验证表达式列表中的别名
    pub fn validate_aliases(
        &self,
        exprs: &[Expr],
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        for expr in exprs {
            self.validate_expression_aliases(expr, aliases)?;
        }
        Ok(())
    }

    /// 验证单个表达式中的别名
    pub fn validate_expression_aliases(
        &self,
        expr: &Expr,
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        // 首先检查表达式本身是否引用了一个别名
        if let Some(alias_name) = self.extract_alias_name(expr) {
            if !aliases.contains_key(&alias_name) {
                return Err(ValidationError::new(
                    format!("未定义的变量别名: {}", alias_name),
                    ValidationErrorType::AliasError,
                ));
            }
        }

        // 递归验证子表达式
        self.validate_subexpressions_aliases(expr, aliases)?;

        Ok(())
    }

    /// 从表达式中提取别名名称
    pub fn extract_alias_name(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Variable(name) => Some(name.clone()),
            Expr::Property { property, .. } => Some(property.clone()),
            Expr::Label(name) => Some(name.clone()),
            // 根据实际的表达式类型，可能需要处理其他别名引用
            _ => None,
        }
    }

    /// 递归验证子表达式中的别名
    fn validate_subexpressions_aliases(
        &self,
        expr: &Expr,
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        match expr {
            Expr::Unary { operand, .. } => self.validate_expression_aliases(operand, aliases),
            Expr::Binary { left, right, .. } => {
                self.validate_expression_aliases(left, aliases)?;
                self.validate_expression_aliases(right, aliases)
            }
            Expr::Function { args, .. } => {
                for arg in args {
                    self.validate_expression_aliases(arg, aliases)?;
                }
                Ok(())
            }
            Expr::List(items) => {
                for item in items {
                    self.validate_expression_aliases(item, aliases)?;
                }
                Ok(())
            }
            Expr::Map(items) => {
                for (_, value) in items {
                    self.validate_expression_aliases(value, aliases)?;
                }
                Ok(())
            }
            Expr::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    self.validate_expression_aliases(condition, aliases)?;
                    self.validate_expression_aliases(value, aliases)?;
                }
                if let Some(default_expr) = default {
                    self.validate_expression_aliases(default_expr, aliases)?;
                }
                Ok(())
            }
            Expr::Subscript { collection, index } => {
                self.validate_expression_aliases(collection, aliases)?;
                self.validate_expression_aliases(index, aliases)
            }
            Expr::Literal(_)
            | Expr::Property { .. }
            | Expr::Unary { .. }
            | Expr::Function { .. }
            | Expr::Variable(_)
            | Expr::Label(_) => Ok(()),
            Expr::TypeCast { expr, .. } => {
                // 类型转换表达式需要验证其子表达式
                self.validate_expression_aliases(expr, aliases)
            }
            Expr::Aggregate { arg, .. } => {
                // 聚合函数表达式需要验证其参数表达式
                self.validate_expression_aliases(arg, aliases)
            }
            Expr::Range {
                collection,
                start,
                end,
            } => {
                // 范围访问表达式需要验证集合和范围表达式
                self.validate_expression_aliases(collection, aliases)?;
                if let Some(start_expr) = start {
                    self.validate_expression_aliases(start_expr, aliases)?;
                }
                if let Some(end_expr) = end {
                    self.validate_expression_aliases(end_expr, aliases)?;
                }
                Ok(())
            }
            Expr::Path(items) => {
                // 路径表达式需要验证其所有项
                for item in items {
                    self.validate_expression_aliases(item, aliases)?;
                }
                Ok(())
            }
        }
    }

    /// 检查别名类型是否匹配使用方式
    pub fn check_alias(
        &self,
        ref_expr: &Expression,
        aliases_available: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        // 提取表达式中的别名名称
        if let Some(alias_name) = self.extract_alias_name(ref_expr) {
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

impl ValidationStrategy for AliasValidationStrategy {
    fn validate(&self, context: &dyn ValidationContext) -> Result<(), ValidationError> {
        // 遍历所有查询部分，验证别名使用
        for query_part in context.get_query_parts() {
            // 验证Match子句中的别名
            for match_ctx in &query_part.matchs {
                if let Some(where_clause) = &match_ctx.where_clause {
                    self.validate_aliases(&[], &where_clause.aliases_available)?;
                }
            }

            // 验证边界子句中的别名
            if let Some(boundary) = &query_part.boundary {
                match boundary {
                    BoundaryClauseContext::With(with_ctx) => {
                        self.validate_aliases(&[], &with_ctx.aliases_available)?;
                    }
                    BoundaryClauseContext::Unwind(unwind_ctx) => {
                        self.validate_aliases(&[], &unwind_ctx.aliases_available)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn strategy_type(&self) -> ValidationStrategyType {
        ValidationStrategyType::Alias
    }

    fn strategy_name(&self) -> &'static str {
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
        assert_eq!(strategy.strategy_type(), ValidationStrategyType::Alias);
        assert_eq!(strategy.strategy_name(), "AliasValidationStrategy");
    }

    #[test]
    fn test_extract_alias_name() {
        let strategy = AliasValidationStrategy::new();

        // 测试从变量表达式中提取别名
        let var_expr = Expression::Variable("test_var".to_string());
        assert_eq!(
            strategy.extract_alias_name(&var_expr),
            Some("test_var".to_string())
        );

        // 测试从常量表达式中提取别名（应该返回None）
        let const_expr = Expression::Literal(crate::core::Value::Int(42));
        assert_eq!(strategy.extract_alias_name(&const_expr), None);
    }
}
