//! Match语句验证器（重构版）
//! 对应原match_validator_main.rs的功能，使用新的策略模式架构

use super::base_validator::Validator;
use super::structs::{
    AliasType, MatchStepRange, PaginationContext, Path, QueryPart, ReturnClauseContext,
    UnwindClauseContext, WhereClauseContext, WithClauseContext, YieldClauseContext, YieldColumn,
};
// 使用context版本的ValidationContext
use super::validation_factory::ValidationFactory;
use super::validation_interface::{ValidationError, ValidationErrorType, ValidationStrategy};
use super::ValidationContext;
use crate::core::Expression;
use crate::query::parser::ast::stmt::MatchStmt;
use crate::query::parser::ast::Pattern;
use std::collections::HashMap;

/// Match语句验证器
pub struct MatchValidator {
    base: Validator,
    validation_strategies: Vec<Box<dyn ValidationStrategy>>,
}

impl MatchValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::with_context(context),
            validation_strategies: ValidationFactory::create_all_strategies(),
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        // 执行所有验证策略
        for strategy in &self.validation_strategies {
            // 现在ValidationContext已经实现了ValidationContext trait
            if let Err(error) = strategy.validate(self.base.context()) {
                self.base.context_mut().add_validation_error(error);
            }
        }

        if self.base.context().has_validation_errors() {
            return Err(ValidationError::new(
                "验证失败".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    /// 使用统一错误类型的验证方法
    pub fn validate_unified(&mut self) -> Result<(), crate::core::error::DBError> {
        // 执行所有验证策略
        for strategy in &self.validation_strategies {
            // 现在ValidationContext已经实现了ValidationContext trait
            if let Err(error) = strategy.validate(self.base.context()) {
                self.base.context_mut().add_validation_error(error);
            }
        }

        if self.base.context().has_validation_errors() {
            // 将ValidationError转换为DBError
            let errors = self.base.context().get_validation_errors();
            if let Some(first_error) = errors.first() {
                return Err(first_error.clone().to_db_error());
            }
        }

        Ok(())
    }

    /// 获取验证上下文的可变引用
    pub fn context_mut(&mut self) -> &mut ValidationContext {
        self.base.context_mut()
    }

    /// 获取验证上下文的引用
    pub fn context(&self) -> &ValidationContext {
        self.base.context()
    }

    /// 验证别名（委托给AliasValidationStrategy）
    pub fn validate_aliases(
        &mut self,
        exprs: &[Expression],
        aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        use super::strategies::AliasValidationStrategy;
        let strategy = AliasValidationStrategy::new();
        strategy.validate_aliases(exprs, aliases)
    }

    /// 检查表达式是否包含聚合函数（委托给AggregateValidationStrategy）
    pub fn has_aggregate_expression(&self, expression: &Expression) -> bool {
        use super::strategies::AggregateValidationStrategy;
        let strategy = AggregateValidationStrategy::new();
        strategy.has_aggregate_expression(expression)
    }

    /// 验证分页（委托给PaginationValidationStrategy）
    pub fn validate_pagination(
        &mut self,
        skip_expression: Option<&Expression>,
        limit_expression: Option<&Expression>,
        context: &PaginationContext,
    ) -> Result<(), ValidationError> {
        use super::strategies::PaginationValidationStrategy;
        let strategy = PaginationValidationStrategy::new();
        strategy.validate_pagination(skip_expression, limit_expression, context)
    }

    /// 验证步数范围（委托给PaginationValidationStrategy）
    pub fn validate_step_range(&self, range: &MatchStepRange) -> Result<(), ValidationError> {
        use super::strategies::PaginationValidationStrategy;
        let strategy = PaginationValidationStrategy::new();
        strategy.validate_step_range(range)
    }

    /// 验证过滤条件（委托给ExpressionValidationStrategy）
    pub fn validate_filter(
        &mut self,
        filter: &Expression,
        context: &WhereClauseContext,
    ) -> Result<(), ValidationError> {
        use super::strategies::ExpressionValidationStrategy;
        let strategy = ExpressionValidationStrategy::new();
        strategy.validate_filter(filter, context)
    }

    /// 验证Return子句（委托给ExpressionValidationStrategy）
    pub fn validate_return(
        &mut self,
        return_expression: &Expression,
        return_items: &[YieldColumn],
        context: &ReturnClauseContext,
    ) -> Result<(), ValidationError> {
        use super::strategies::ExpressionValidationStrategy;
        let strategy = ExpressionValidationStrategy::new();
        strategy.validate_return(return_expression, return_items, context)
    }

    /// 验证With子句（委托给ExpressionValidationStrategy）
    pub fn validate_with(
        &mut self,
        with_expression: &Expression,
        with_items: &[YieldColumn],
        context: &WithClauseContext,
    ) -> Result<(), ValidationError> {
        use super::strategies::ExpressionValidationStrategy;
        let strategy = ExpressionValidationStrategy::new();
        strategy.validate_with(with_expression, with_items, context)
    }

    /// 验证Unwind子句（委托给ExpressionValidationStrategy）
    pub fn validate_unwind(
        &mut self,
        unwind_expression: &Expression,
        context: &UnwindClauseContext,
    ) -> Result<(), ValidationError> {
        use super::strategies::ExpressionValidationStrategy;
        let strategy = ExpressionValidationStrategy::new();
        strategy.validate_unwind(unwind_expression, context)
    }

    /// 验证Yield子句（委托给ClauseValidationStrategy）
    pub fn validate_yield(&mut self, context: &YieldClauseContext) -> Result<(), ValidationError> {
        use super::strategies::ClauseValidationStrategy;
        let strategy = ClauseValidationStrategy::new();
        strategy.validate_yield_clause(context)
    }

    /// 构建所有命名别名的列（委托给ClauseValidationStrategy）
    pub fn build_columns_for_all_named_aliases(
        &mut self,
        query_parts: &[QueryPart],
        columns: &mut Vec<YieldColumn>,
    ) -> Result<(), ValidationError> {
        use super::strategies::ClauseValidationStrategy;
        let strategy = ClauseValidationStrategy::new();
        strategy.build_columns_for_all_named_aliases(query_parts, columns)
    }

    /// 结合别名（委托给AliasValidationStrategy）
    pub fn combine_aliases(
        &mut self,
        cur_aliases: &mut HashMap<String, AliasType>,
        last_aliases: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        use super::strategies::AliasValidationStrategy;
        let strategy = AliasValidationStrategy::new();
        strategy.combine_aliases(cur_aliases, last_aliases)
    }

    /// 构建输出（委托给ClauseValidationStrategy）
    pub fn build_outputs(&mut self, paths: &mut Vec<Path>) -> Result<(), ValidationError> {
        use super::strategies::ClauseValidationStrategy;
        let strategy = ClauseValidationStrategy::new();
        strategy.build_outputs(paths)
    }

    /// 检查别名（委托给AliasValidationStrategy）
    pub fn check_alias(
        &mut self,
        ref_expression: &Expression,
        aliases_available: &HashMap<String, AliasType>,
    ) -> Result<(), ValidationError> {
        use super::strategies::AliasValidationStrategy;
        let strategy = AliasValidationStrategy::new();
        strategy.check_alias(ref_expression, aliases_available)
    }

    /// 验证完整的 MATCH 语句
    pub fn validate_match_statement(&mut self, match_stmt: &MatchStmt) -> Result<(), ValidationError> {
        // 1. 验证模式不为空
        if match_stmt.patterns.is_empty() {
            return Err(ValidationError::new(
                "MATCH 语句必须包含至少一个模式".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 验证每个模式
        for (idx, pattern) in match_stmt.patterns.iter().enumerate() {
            if let Err(e) = self.validate_pattern(pattern, idx) {
                self.base.context_mut().add_validation_error(e);
            }
        }

        // 3. 验证 RETURN 子句存在性
        if match_stmt.return_clause.is_none() {
            return Err(ValidationError::new(
                "MATCH 语句必须包含 RETURN 子句".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 4. 验证 WHERE 子句（如果存在）
        if let Some(ref where_clause) = match_stmt.where_clause {
            if let Err(e) = self.validate_where_clause(where_clause) {
                self.base.context_mut().add_validation_error(e);
            }
        }

        // 5. 验证 RETURN 子句
        if let Some(ref return_clause) = match_stmt.return_clause {
            if let Err(e) = self.validate_return_clause(return_clause) {
                self.base.context_mut().add_validation_error(e);
            }
        }

        // 6. 验证 ORDER BY 子句（如果存在）
        if let Some(ref order_by) = match_stmt.order_by {
            if let Err(e) = self.validate_order_by(order_by) {
                self.base.context_mut().add_validation_error(e);
            }
        }

        // 7. 验证分页参数
        if let (Some(skip), Some(limit)) = (match_stmt.skip, match_stmt.limit) {
            if skip >= limit {
                return Err(ValidationError::new(
                    format!("SKIP 值 ({}) 必须小于 LIMIT 值 ({})", skip, limit),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        if self.base.context().has_validation_errors() {
            return Err(ValidationError::new(
                "MATCH 语句验证失败".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        Ok(())
    }

    /// 验证单个模式
    fn validate_pattern(&mut self, pattern: &Pattern, idx: usize) -> Result<(), ValidationError> {
        // 验证模式中的节点和边
        match pattern {
            Pattern::Node(node_pattern) => {
                // 验证节点模式
                if node_pattern.variable.is_none() && node_pattern.labels.is_empty() {
                    return Err(ValidationError::new(
                        format!("第 {} 个模式: 匿名节点必须指定标签", idx + 1),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            Pattern::Edge(edge_pattern) => {
                // 验证边模式
                if edge_pattern.edge_types.is_empty() && edge_pattern.variable.is_none() {
                    // 警告：匿名边类型
                }
            }
            Pattern::Path(path_pattern) => {
                // 验证路径模式
                if path_pattern.elements.is_empty() {
                    return Err(ValidationError::new(
                        format!("第 {} 个模式: 路径不能为空", idx + 1),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            Pattern::Variable(_) => {
                // 变量模式无需额外验证
            }
        }
        Ok(())
    }

    /// 验证 WHERE 子句
    fn validate_where_clause(&mut self, where_expr: &Expression) -> Result<(), ValidationError> {
        // 验证 WHERE 表达式是否有效
        // TODO: 使用 ExpressionValidationStrategy 进行更详细的验证
        
        // 检查表达式是否是布尔类型或可以转换为布尔类型
        match where_expr {
            Expression::Binary { op, .. } => {
                // 检查比较操作符
                use crate::core::BinaryOperator;
                match op {
                    BinaryOperator::Equal | BinaryOperator::NotEqual | BinaryOperator::LessThan |
                    BinaryOperator::LessThanOrEqual | BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual |
                    BinaryOperator::And | BinaryOperator::Or => Ok(()),
                    _ => Err(ValidationError::new(
                        "WHERE 子句包含无效的操作符".to_string(),
                        ValidationErrorType::TypeError,
                    )),
                }
            }
            Expression::Unary { op, .. } => {
                use crate::core::UnaryOperator;
                match op {
                    UnaryOperator::Not => Ok(()),
                    _ => Err(ValidationError::new(
                        "WHERE 子句包含无效的一元操作符".to_string(),
                        ValidationErrorType::TypeError,
                    )),
                }
            }
            _ => Ok(()), // 其他表达式类型暂时通过
        }
    }

    /// 验证 RETURN 子句
    fn validate_return_clause(
        &mut self,
        return_clause: &crate::query::parser::ast::stmt::ReturnClause,
    ) -> Result<(), ValidationError> {
        if return_clause.items.is_empty() && !matches!(return_clause.items.first(), Some(crate::query::parser::ast::stmt::ReturnItem::All)) {
            return Err(ValidationError::new(
                "RETURN 子句必须包含至少一个返回项".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 验证每个返回项
        for (idx, item) in return_clause.items.iter().enumerate() {
            match item {
                crate::query::parser::ast::stmt::ReturnItem::All => {
                    // RETURN * 是有效的
                }
                crate::query::parser::ast::stmt::ReturnItem::Expression { expression, alias } => {
                    // 验证表达式
                    if let Err(e) = self.validate_return_expression(expression, idx) {
                        return Err(e);
                    }
                    
                    // 验证别名（如果存在）
                    if let Some(ref alias_name) = alias {
                        if alias_name.is_empty() {
                            return Err(ValidationError::new(
                                format!("第 {} 个返回项的别名不能为空", idx + 1),
                                ValidationErrorType::SemanticError,
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 验证返回表达式
    fn validate_return_expression(
        &mut self,
        expr: &Expression,
        idx: usize,
    ) -> Result<(), ValidationError> {
        match expr {
            Expression::Variable(var_name) => {
                // 检查变量是否在上下文中定义
                if self.base.context().get_variable(var_name).is_none() {
                    return Err(ValidationError::new(
                        format!("第 {} 个返回项引用了未定义的变量 '{}'", idx + 1, var_name),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            Expression::Property { object, property: _property } => {
                // 验证属性访问
                if let Expression::Variable(var_name) = object.as_ref() {
                    if self.base.context().get_variable(var_name).is_none() {
                        return Err(ValidationError::new(
                            format!("第 {} 个返回项引用了未定义的变量 '{}'", idx + 1, var_name),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
                // TODO: 验证属性名是否存在于对应节点的 Schema 中
            }
            _ => {}
        }
        Ok(())
    }

    /// 验证 ORDER BY 子句
    fn validate_order_by(
        &mut self,
        order_by: &crate::query::parser::ast::stmt::OrderByClause,
    ) -> Result<(), ValidationError> {
        if order_by.items.is_empty() {
            return Err(ValidationError::new(
                "ORDER BY 子句必须包含至少一个排序项".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for (idx, item) in order_by.items.iter().enumerate() {
            // 验证排序表达式
            match &item.expression {
                Expression::Variable(var_name) => {
                    if self.base.context().get_variable(var_name).is_none() {
                        return Err(ValidationError::new(
                            format!("第 {} 个排序项引用了未定义的变量 '{}'", idx + 1, var_name),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_match_validator_creation() {
        let context = ValidationContext::new();
        let validator = MatchValidator::new(context);

        assert_eq!(validator.validation_strategies.len(), 5); // 应该有5个策略
    }

    #[test]
    fn test_basic_validation() {
        let context = ValidationContext::new();
        let mut validator = MatchValidator::new(context);

        // 简单验证应该成功
        assert!(validator.validate().is_ok());
    }

    #[test]
    fn test_validate_pagination() {
        let context = ValidationContext::new();
        let mut validator = MatchValidator::new(context);

        // 测试有效的分页表达式
        let skip_expression = Expression::Literal(crate::core::Value::Int(1));
        let limit_expression = Expression::Literal(crate::core::Value::Int(10));
        let pagination_ctx = PaginationContext { skip: 0, limit: 10 };

        assert!(validator
            .validate_pagination(Some(&skip_expression), Some(&limit_expression), &pagination_ctx)
            .is_ok());
    }

    #[test]
    fn test_validate_aliases() {
        let context = ValidationContext::new();
        let mut validator = MatchValidator::new(context);

        // 创建一个别名映射
        let mut aliases = HashMap::new();
        aliases.insert("n".to_string(), AliasType::Node);
        aliases.insert("e".to_string(), AliasType::Edge);

        // 测试有效的别名引用
        let expression = Expression::Variable("n".to_string());
        assert!(validator.validate_aliases(&[expression], &aliases).is_ok());

        // 测试无效的别名引用
        let invalid_expression = Expression::Variable("invalid".to_string());
        assert!(validator
            .validate_aliases(&[invalid_expression], &aliases)
            .is_err());
    }

    #[test]
    fn test_has_aggregate_expression() {
        let context = ValidationContext::new();
        let validator = MatchValidator::new(context);

        // 测试没有聚合函数的表达式
        let non_agg_expression = Expression::Literal(crate::core::Value::Int(1));
        assert_eq!(validator.has_aggregate_expression(&non_agg_expression), false);
    }

    #[test]
    fn test_combine_aliases() {
        let context = ValidationContext::new();
        let mut validator = MatchValidator::new(context);

        let mut cur_aliases = HashMap::new();
        cur_aliases.insert("a".to_string(), AliasType::Node);

        let mut last_aliases = HashMap::new();
        last_aliases.insert("b".to_string(), AliasType::Edge);
        last_aliases.insert("c".to_string(), AliasType::Path);

        // 组合别名
        assert!(validator
            .combine_aliases(&mut cur_aliases, &last_aliases)
            .is_ok());
        assert_eq!(cur_aliases.len(), 3);
        assert!(cur_aliases.contains_key("a"));
        assert!(cur_aliases.contains_key("b"));
        assert!(cur_aliases.contains_key("c"));
    }

    #[test]
    fn test_validate_step_range() {
        let context = ValidationContext::new();
        let validator = MatchValidator::new(context);

        // 测试有效的范围（min <= max）
        let valid_range = MatchStepRange::new(1, 3);
        assert!(validator.validate_step_range(&valid_range).is_ok());

        // 测试无效的范围（min > max）
        let invalid_range = MatchStepRange::new(3, 1);
        assert!(validator.validate_step_range(&invalid_range).is_err());
    }
}
