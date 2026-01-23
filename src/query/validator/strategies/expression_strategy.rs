use crate::core::types::expression::Expression;
use crate::core::DataType;
use crate::query::validator::{
    ValidationStrategy, ValidationError, ValidationErrorType,
    ValidationStrategyType, WhereClauseContext, MatchClauseContext, ReturnClauseContext,
    WithClauseContext, UnwindClauseContext, YieldClauseContext, BoundaryClauseContext,
    YieldColumn
};
use crate::query::validator::validation_interface::ValidationContext;

/// 表达式验证策略
pub struct ExpressionValidationStrategy;

impl ExpressionValidationStrategy {
    pub fn new() -> Self {
        Self
    }

    /// 验证过滤条件
    pub fn validate_filter(
        &self,
        filter: &Expression,
        context: &WhereClauseContext,
    ) -> Result<(), ValidationError> {
        // 过滤条件必须是布尔类型或可转换为布尔类型
        let type_validator = crate::query::validator::strategies::type_inference::TypeValidator;
        let filter_type = type_validator.deduce_expression_type_full(filter, context);
        
        if !type_validator.are_types_compatible(&filter_type, &DataType::Bool) {
            return Err(ValidationError::new(
                format!("过滤条件必须是布尔类型，当前类型为 {:?}", filter_type),
                ValidationErrorType::TypeError,
            ));
        }

        // 验证表达式中的变量引用
        let var_validator = crate::query::validator::strategies::variable_validator::VariableValidator::new();
        var_validator.validate_expression_variables(filter, context)?;
        
        // 验证表达式操作
        let expr_validator = crate::query::validator::strategies::expression_operations::ExpressionOperationsValidator::new();
        expr_validator.validate_expression_operations(filter)?;

        Ok(())
    }

    /// 验证Match路径
    pub fn validate_path(
        &self,
        path: &Expression,
        context: &MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证路径表达式的类型
        let type_validator = crate::query::validator::strategies::type_inference::TypeValidator;
        let path_type = type_validator.deduce_expression_type_full(path, context);
        
        // 路径表达式应该是路径类型或可以转换为路径类型
        if !matches!(path_type, DataType::Path) && !matches!(path_type, DataType::Empty) {
            return Err(ValidationError::new(
                format!("路径表达式类型不匹配，期望路径类型，实际为 {:?}", path_type),
                ValidationErrorType::TypeError,
            ));
        }

        // 验证路径中的变量引用
        let var_validator = crate::query::validator::strategies::variable_validator::VariableValidator::new();
        var_validator.validate_expression_variables(path, context)?;

        Ok(())
    }

    /// 验证Return子句
    pub fn validate_return(
        &self,
        return_expression: &Expression,
        return_items: &[YieldColumn],
        context: &ReturnClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证Return表达式的类型
        let type_validator = crate::query::validator::strategies::type_inference::TypeValidator;
        let _return_type = type_validator.deduce_expression_type_full(return_expression, context);
        
        // 检查Return项中的聚合函数使用
        for item in return_items {
            if type_validator.has_aggregate_expression(&item.expression) {
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
        let var_validator = crate::query::validator::strategies::variable_validator::VariableValidator::new();
        var_validator.validate_expression_variables(return_expression, context)?;

        Ok(())
    }

    /// 验证With子句
    pub fn validate_with(
        &self,
        with_expression: &Expression,
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
        unwind_expression: &Expression,
        context: &UnwindClauseContext,
    ) -> Result<(), ValidationError> {
        // Unwind表达式必须是列表类型或可迭代类型
        let type_validator = crate::query::validator::strategies::type_inference::TypeValidator;
        let unwind_type = type_validator.deduce_expression_type_full(unwind_expression, context);
        
        if unwind_type != DataType::List && unwind_type != DataType::Empty {
            return Err(ValidationError::new(
                format!("Unwind表达式必须是列表类型，当前类型为 {:?}", unwind_type),
                ValidationErrorType::TypeError,
            ));
        }

        // 验证表达式中的变量引用
        let var_validator = crate::query::validator::strategies::variable_validator::VariableValidator::new();
        var_validator.validate_expression_variables(unwind_expression, context)?;

        Ok(())
    }

    /// 验证Yield子句
    pub fn validate_yield(&self, context: &YieldClauseContext) -> Result<(), ValidationError> {
        // 验证每个Yield列
        let type_validator = crate::query::validator::strategies::type_inference::TypeValidator;
        let var_validator = crate::query::validator::strategies::variable_validator::VariableValidator::new();
        
        for column in &context.yield_columns {
            // 验证表达式的类型
            let _column_type = type_validator.deduce_expression_type_full(&column.expression, context);
            
            // 验证聚合函数的使用
            if type_validator.has_aggregate_expression(&column.expression) {
                if !context.has_agg && context.group_keys.is_empty() {
                    return Err(ValidationError::new(
                        "在GROUP BY子句中使用聚合函数时，必须指定GROUP BY键".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }

            // 验证表达式中的变量引用
            var_validator.validate_expression_variables(&column.expression, context)?;
        }

        // 验证分组键
        for group_key in &context.group_keys {
            type_validator.validate_group_key_type(group_key, context)?;
        }

        Ok(())
    }

    /// 验证单个路径模式
    pub fn validate_single_path_pattern(
        &self,
        pattern: &Expression,
        context: &mut MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证路径模式的类型
        let type_validator = crate::query::validator::strategies::type_inference::TypeValidator;
        let pattern_type = type_validator.deduce_expression_type_full(pattern, context);
        
        if !matches!(pattern_type, DataType::Path) && !matches!(pattern_type, DataType::Empty) {
            return Err(ValidationError::new(
                format!("路径模式必须是路径类型，当前类型为 {:?}", pattern_type),
                ValidationErrorType::TypeError,
            ));
        }

        // 验证路径模式中的变量引用
        let var_validator = crate::query::validator::strategies::variable_validator::VariableValidator::new();
        var_validator.validate_expression_variables(pattern, context)?;

        Ok(())
    }

    /// 验证表达式循环引用（辅助函数）
    fn validate_expression_cycles(&self, expression: &Expression) -> Result<(), ValidationError> {
        // 检查表达式是否包含循环引用
        // 例如：a = b + 1, b = a + 2
        
        // 使用访问器收集所有变量引用
        use crate::query::visitor::VariableVisitor;
        let mut visitor = VariableVisitor::new();
        let variables = visitor.collect_variables(expression);
        
        // 检查是否有明显的循环引用模式
        // 例如：表达式包含同一个变量的多次引用，且这些引用之间存在依赖关系
        
        // 简化实现：检查变量数量是否合理
        if variables.len() > 100 {
            return Err(ValidationError::new(
                format!(
                    "表达式包含过多的变量引用（{}个），可能存在复杂的依赖关系",
                    variables.len()
                ),
                ValidationErrorType::SemanticError,
            ));
        }
        
        // 检查是否有重复的变量引用
        let mut var_count = std::collections::HashMap::new();
        for var in &variables {
            *var_count.entry(var.clone()).or_insert(0) += 1;
        }
        
        // 如果某个变量被引用超过50次，可能表示循环引用或过度复杂的表达式
        for (var, count) in &var_count {
            if *count > 50 {
                return Err(ValidationError::new(
                    format!(
                        "变量 {} 被引用 {} 次，可能存在循环引用或过度复杂的表达式",
                        var, count
                    ),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        
        // 检查表达式深度是否合理（避免过深的嵌套）
        let depth = self.calculate_expression_depth(expression);
        if depth > 50 {
            return Err(ValidationError::new(
                format!(
                    "表达式嵌套深度过大（{}层），可能存在循环引用或过度复杂的表达式",
                    depth
                ),
                ValidationErrorType::SemanticError,
            ));
        }
        
        Ok(())
    }

    /// 计算表达式的嵌套深度
    fn calculate_expression_depth(&self, expression: &Expression) -> usize {
        match expression {
            Expression::Literal(_) => 1,
            Expression::Variable(_) => 1,
            Expression::Label(_) => 1,

            Expression::Property { object, .. } => {
                1 + self.calculate_expression_depth(object)
            }

            Expression::Binary { left, right, .. } => {
                1 + self.calculate_expression_depth(left).max(self.calculate_expression_depth(right))
            }

            Expression::Unary { operand, .. } => {
                1 + self.calculate_expression_depth(operand)
            }

            Expression::Function { args, .. } => {
                let max_arg_depth = args.iter()
                    .map(|arg| self.calculate_expression_depth(arg))
                    .max()
                    .unwrap_or(0);
                1 + max_arg_depth
            }

            Expression::Aggregate { arg, .. } => {
                1 + self.calculate_expression_depth(arg)
            }

            Expression::List(items) => {
                let max_item_depth = items.iter()
                    .map(|item| self.calculate_expression_depth(item))
                    .max()
                    .unwrap_or(0);
                1 + max_item_depth
            }

            Expression::Map(entries) => {
                let max_value_depth = entries.iter()
                    .map(|(_, value)| self.calculate_expression_depth(value))
                    .max()
                    .unwrap_or(0);
                1 + max_value_depth
            }

            Expression::Case { conditions, default } => {
                let mut max_depth = 0;
                for (when_expression, then_expression) in conditions {
                    max_depth = max_depth.max(self.calculate_expression_depth(when_expression));
                    max_depth = max_depth.max(self.calculate_expression_depth(then_expression));
                }
                if let Some(default_expression) = default {
                    max_depth = max_depth.max(self.calculate_expression_depth(default_expression));
                }
                1 + max_depth
            }

            Expression::TypeCast { expression, .. } => {
                1 + self.calculate_expression_depth(expression)
            }

            Expression::Subscript { collection, index } => {
                1 + self.calculate_expression_depth(collection)
                    .max(self.calculate_expression_depth(index))
            }

            Expression::Range { collection, start, end } => {
                let mut max_depth = self.calculate_expression_depth(collection);
                if let Some(start_expression) = start {
                    max_depth = max_depth.max(self.calculate_expression_depth(start_expression));
                }
                if let Some(end_expression) = end {
                    max_depth = max_depth.max(self.calculate_expression_depth(end_expression));
                }
                1 + max_depth
            }

            Expression::Path(items) => {
                let max_item_depth = items.iter()
                    .map(|item| self.calculate_expression_depth(item))
                    .max()
                    .unwrap_or(0);
                1 + max_item_depth
            }
        }
    }
}

impl ValidationStrategy for ExpressionValidationStrategy {
    fn validate(&self, context: &dyn ValidationContext) -> Result<(), ValidationError> {
        // 遍历所有查询部分，验证表达式
        for query_part in context.get_query_parts() {
            // 验证Match子句中的表达式
            for match_ctx in &query_part.matchs {
                if let Some(where_clause) = &match_ctx.where_clause {
                    if let Some(filter) = &where_clause.filter {
                        self.validate_filter(filter, where_clause)?;
                    }
                }
            }

            // 验证边界子句中的表达式
            if let Some(boundary) = &query_part.boundary {
                match boundary {
                    BoundaryClauseContext::With(with_ctx) => {
                        if let Some(where_clause) = &with_ctx.where_clause {
                            if let Some(filter) = &where_clause.filter {
                                self.validate_filter(filter, where_clause)?;
                            }
                        }
                    }
                    BoundaryClauseContext::Unwind(unwind_ctx) => {
                        self.validate_unwind(&unwind_ctx.unwind_expression, unwind_ctx)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn strategy_type(&self) -> ValidationStrategyType {
        ValidationStrategyType::Expression
    }

    fn strategy_name(&self) -> &'static str {
        "ExpressionValidationStrategy"
    }
}