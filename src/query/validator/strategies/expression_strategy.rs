use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::types::expression::Expression;
use crate::core::DataType;
use crate::core::YieldColumn;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::validator::structs::{
    WhereClauseContext, MatchClauseContext,
    ReturnClauseContext, WithClauseContext, UnwindClauseContext, YieldClauseContext,
};

use super::helpers::TypeValidator;
use super::helpers::VariableChecker;
use super::expression_operations::ExpressionOperationsValidator;

/// 表达式验证策略
pub struct ExpressionValidationStrategy;

impl ExpressionValidationStrategy {
    pub fn new() -> Self {
        Self
    }

    /// 验证过滤条件
    pub fn validate_filter(
        &self,
        filter: &ContextualExpression,
        context: &WhereClauseContext,
    ) -> Result<(), ValidationError> {
        // 从 ContextualExpression 获取 Expression
        let expr_meta = match filter.expression() {
            Some(e) => e,
            None => return Ok(()),
        };
        let expr = expr_meta.inner().as_ref();

        // 过滤条件必须是布尔类型或可转换为布尔类型
        let type_validator = TypeValidator;
        let filter_type = type_validator.deduce_expression_type_full(&expr, context);

        if !type_validator.are_types_compatible(&filter_type, &DataType::Bool) {
            return Err(ValidationError::new(
                format!("过滤条件必须是布尔类型，当前类型为 {:?}", filter_type),
                ValidationErrorType::TypeError,
            ));
        }

        // 验证表达式中的变量引用
        let var_validator = VariableChecker::new();
        var_validator.validate_expression_variables(filter, &context.aliases_available)?;

        // 验证表达式操作
        let expr_validator = ExpressionOperationsValidator::new();
        expr_validator.validate_expression_operations(&expr)?;

        Ok(())
    }

    /// 验证Match路径
    pub fn validate_path(
        &self,
        path: &ContextualExpression,
        context: &MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 从 ContextualExpression 获取 Expression
        let expr_meta = match path.expression() {
            Some(e) => e,
            None => return Ok(()),
        };
        let expr = expr_meta.inner().as_ref();

        // 验证路径表达式的类型
        let type_validator = TypeValidator;
        let path_type = type_validator.deduce_expression_type_full(&expr, context);

        // 路径表达式应该是路径类型或可以转换为路径类型
        if !matches!(path_type, DataType::Path) && !matches!(path_type, DataType::Empty) {
            return Err(ValidationError::new(
                format!("路径表达式类型不匹配，期望路径类型，实际为 {:?}", path_type),
                ValidationErrorType::TypeError,
            ));
        }

        // 验证路径中的变量引用
        let var_validator = VariableChecker::new();
        var_validator.validate_expression_variables(path, &context.aliases_available)?;

        Ok(())
    }

    /// 验证Return子句
    pub fn validate_return(
        &self,
        return_expression: &ContextualExpression,
        return_items: &[YieldColumn],
        context: &ReturnClauseContext,
    ) -> Result<(), ValidationError> {
        // 从 ContextualExpression 获取 Expression
        let expr_meta = match return_expression.expression() {
            Some(e) => e,
            None => return Ok(()),
        };
        let expr = expr_meta.inner().as_ref();

        // 验证Return表达式的类型
        let type_validator = TypeValidator;
        let _return_type = type_validator.deduce_expression_type_full(&expr, context);

        // 检查Return项中的聚合函数使用
        for item in return_items {
            let item_expr_meta = match item.expression.expression() {
                Some(e) => e,
                None => continue,
            };
            let item_expr = item_expr_meta.inner().as_ref();
            if type_validator.has_aggregate_expression_internal(&item_expr) {
                // 验证聚合函数的使用是否符合上下文
                if !context.yield_clause.has_agg && context.yield_clause.group_keys.is_empty() {
                    return Err(ValidationError::new(
                        "在GROUP BY子句中使用聚合函数时，必须指定GROUP BY键".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        // 验证表达式中的变量引用
        let var_validator = VariableChecker::new();
        var_validator.validate_expression_variables(return_expression, &context.aliases_available)?;

        Ok(())
    }

    /// 验证With子句
    pub fn validate_with(
        &self,
        with_expression: &ContextualExpression,
        with_items: &[YieldColumn],
        context: &WithClauseContext,
    ) -> Result<(), ValidationError> {
        // With子句的验证逻辑与Return子句类似
        let return_context = ReturnClauseContext {
            yield_clause: context.yield_clause.clone(),
            aliases_available: context.aliases_available.clone(),
            aliases_generated: context.aliases_generated.clone(),
            pagination: context.pagination.clone(),
            order_by: context.order_by.clone(),
            distinct: context.distinct,
            query_parts: context.query_parts.clone(),
            errors: context.errors.clone(),
        };
        self.validate_return(with_expression, with_items, &return_context)
    }

    /// 验证Unwind子句
    pub fn validate_unwind(
        &self,
        unwind_expression: &ContextualExpression,
        context: &UnwindClauseContext,
    ) -> Result<(), ValidationError> {
        // 从 ContextualExpression 获取 Expression
        if let Some(expr) = unwind_expression.get_expression() {
            // Unwind表达式必须是列表类型或可迭代类型
            let type_validator = TypeValidator;
            let unwind_type = type_validator.deduce_expression_type_full(&expr, context);

            if unwind_type != DataType::List && unwind_type != DataType::Empty {
                return Err(ValidationError::new(
                    format!("Unwind表达式必须是列表类型，当前类型为 {:?}", unwind_type),
                    ValidationErrorType::TypeError,
                ));
            }

            // 验证表达式中的变量引用
            let var_validator = VariableChecker::new();
            var_validator.validate_expression_variables(unwind_expression, &context.aliases_available)?;
        }

        Ok(())
    }

    /// 验证Yield子句
    pub fn validate_yield(&self, context: &YieldClauseContext) -> Result<(), ValidationError> {
        // 验证每个Yield列
        let type_validator = TypeValidator;
        let var_validator = VariableChecker::new();

        for column in &context.yield_columns {
            // 从 ContextualExpression 获取 Expression
            let expr_meta = match column.expression.expression() {
                Some(e) => e,
                None => continue,
            };
            let expr = expr_meta.inner().as_ref();

            // 验证表达式的类型
            let _column_type = type_validator.deduce_expression_type_full(&expr, context);

            // 验证聚合函数的使用
            if type_validator.has_aggregate_expression_internal(&expr) {
                if !context.has_agg && context.group_keys.is_empty() {
                    return Err(ValidationError::new(
                        "在GROUP BY子句中使用聚合函数时，必须指定GROUP BY键".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }

            // 验证表达式中的变量引用
            var_validator.validate_expression_variables(&column.expression, &context.aliases_available)?;
        }

        // 验证分组键
        for group_key in &context.group_keys {
            let expr_meta = match group_key.expression() {
                Some(e) => e,
                None => continue,
            };
            let expr = expr_meta.inner().as_ref();
            type_validator.validate_group_key_type_internal(&expr, context)?;
        }

        Ok(())
    }

    /// 验证单个路径模式
    pub fn validate_single_path_pattern(
        &self,
        pattern: &ContextualExpression,
        context: &mut MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 从 ContextualExpression 获取 Expression
        let expr_meta = match pattern.expression() {
            Some(e) => e,
            None => return Ok(()),
        };
        let expr = expr_meta.inner().as_ref();

        // 验证路径模式的类型
        let type_validator = TypeValidator;
        let pattern_type = type_validator.deduce_expression_type_full(&expr, context);

        if !matches!(pattern_type, DataType::Path) && !matches!(pattern_type, DataType::Empty) {
            return Err(ValidationError::new(
                format!("路径模式必须是路径类型，当前类型为 {:?}", pattern_type),
                ValidationErrorType::TypeError,
            ));
        }

        // 验证路径模式中的变量引用
        let var_validator = VariableChecker::new();
        var_validator.validate_expression_variables(pattern, &context.aliases_available)?;

        Ok(())
    }
}

impl ExpressionValidationStrategy {
    /// 获取策略名称
    pub fn strategy_name(&self) -> &'static str {
        "ExpressionValidationStrategy"
    }
}