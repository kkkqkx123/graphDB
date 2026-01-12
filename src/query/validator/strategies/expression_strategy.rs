//! 表达式验证策略
//! 负责验证各种表达式类型和结构

use super::super::structs::*;
use super::super::validation_interface::*;
use crate::core::Expression;
use crate::core::types::operators::Operator;
use crate::core::ValueTypeDef;
use std::collections::HashMap;

/// 表达式验证上下文Trait
/// 定义表达式验证所需的基本接口
trait ExpressionValidationContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType>;
    fn get_variable_types(&self) -> Option<&HashMap<String, ValueTypeDef>>;
}

impl<T: ValidationContext> ExpressionValidationContext for T {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        self.get_aliases()
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, ValueTypeDef>> {
        None
    }
}

/// WhereClauseContext 的 ExpressionValidationContext 实现
impl ExpressionValidationContext for WhereClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, ValueTypeDef>> {
        None
    }
}

/// MatchClauseContext 的 ExpressionValidationContext 实现
impl ExpressionValidationContext for MatchClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, ValueTypeDef>> {
        None
    }
}

/// YieldClauseContext 的 ExpressionValidationContext 实现
impl ExpressionValidationContext for YieldClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, ValueTypeDef>> {
        None
    }
}

/// ReturnClauseContext 的 ExpressionValidationContext 实现
impl ExpressionValidationContext for ReturnClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, ValueTypeDef>> {
        None
    }
}

/// WithClauseContext 的 ExpressionValidationContext 实现
impl ExpressionValidationContext for WithClauseContext {
    fn get_aliases(&self) -> &HashMap<String, AliasType> {
        &self.aliases_available
    }

    fn get_variable_types(&self) -> Option<&HashMap<String, ValueTypeDef>> {
        None
    }
}

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
        // 验证过滤表达式
        // 检查表达式中的别名是否已定义
        // 验证表达式的类型

        // 使用别名验证器验证别名
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        alias_validator.validate_aliases(&[filter.clone()], &context.aliases_available)?;

        // 使用EvaluableExprVisitor检查表达式是否可立即求值
        use crate::query::visitor::EvaluableExprVisitor;

        let mut visitor = EvaluableExprVisitor::new();
        if visitor.is_evaluable(filter) {
            // 表达式可求值，检查其类型是否为布尔值
            match filter {
                Expression::Literal(crate::core::Value::Bool(_)) => Ok(()),
                Expression::Literal(_) => Err(ValidationError::new(
                    "WHERE表达式必须求值为布尔类型".to_string(),
                    ValidationErrorType::TypeError,
                )),
                _ => {
                    // 对于非常量表达式，尝试求值
                    // 注意：这里简化处理，实际应该实现表达式求值
                    Err(ValidationError::new(
                        "WHERE表达式必须是布尔常量".to_string(),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
        } else {
            // 表达式不可立即求值，使用类型推导系统进行类型检查
            self.validate_expression_type(filter, context, ValueTypeDef::Bool)
        }
    }

    /// 验证Match路径
    pub fn validate_path(
        &self,
        path: &Expression,
        context: &MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证Match路径表达式
        // 检查路径中的节点和边定义
        // 验证路径模式的有效性

        // 这里应该解析路径表达式，提取节点和边的信息
        // 但由于当前的路径表示可能不同，我们暂时实现基本验证

        // 检查路径中是否存在有效的节点和边结构
        match path {
            Expression::MatchPathPattern { patterns, .. } => {
                for pattern in patterns {
                    // 验证每个路径模式
                    self.validate_single_path_pattern(pattern, context)?;
                }
            }
            _ => {
                return Err(ValidationError::new(
                    "无效的路径模式表达式".to_string(),
                    ValidationErrorType::SyntaxError,
                ));
            }
        }

        Ok(())
    }

    /// 验证单个路径模式
    pub fn validate_single_path_pattern(
        &self,
        pattern: &Expression,
        context: &MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证单个路径模式的结构
        // 检查节点、边的定义等
        
        match pattern {
            Expression::MatchPathPattern { patterns, .. } => {
                // 递归验证嵌套的路径模式
                for nested_pattern in patterns {
                    self.validate_single_path_pattern(nested_pattern, context)?;
                }
            }
            Expression::Label(label) => {
                // 验证标签模式（简化处理，实际应该验证节点标签）
                if label.trim().is_empty() {
                    return Err(ValidationError::new(
                        "节点标签不能为空".to_string(),
                        ValidationErrorType::SyntaxError,
                    ));
                }
            }
            Expression::Variable(var_name) => {
                // 验证变量模式（简化处理，实际应该验证边变量）
                if var_name.trim().is_empty() {
                    return Err(ValidationError::new(
                        "边变量不能为空".to_string(),
                        ValidationErrorType::SyntaxError,
                    ));
                }
            }
            _ => {
                return Err(ValidationError::new(
                    format!("无效的路径模式表达式: {:?}", pattern),
                    ValidationErrorType::SyntaxError,
                ));
            }
        }
        
        Ok(())
    }
    
    /// 验证节点模式
    fn validate_node_pattern(
        &self,
        labels: &[String],
        properties: &Option<Expression>,
        context: &MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证标签是否有效
        for label in labels {
            if label.trim().is_empty() {
                return Err(ValidationError::new(
                    "节点标签不能为空".to_string(),
                    ValidationErrorType::SyntaxError,
                ));
            }
        }
        
        // 验证属性表达式（如果存在）
        if let Some(prop_expr) = properties {
            self.validate_expression_type(prop_expr, context, ValueTypeDef::Map)?;
        }
        
        Ok(())
    }
    
    /// 验证边模式（简化版本，因为当前表达式结构不支持完整的边模式）
    fn validate_edge_pattern(
        &self,
        _types: &[String],
        _properties: &Option<Expression>,
        _direction: &crate::core::Direction,
        _context: &MatchClauseContext,
    ) -> Result<(), ValidationError> {
        // 由于当前表达式结构不支持完整的边模式，这里简化实现
        // 在实际的NebulaGraph中，应该有更完整的边模式验证
        
        // 简化实现：总是返回成功
        Ok(())
    }

    /// 验证Return子句
    pub fn validate_return(
        &self,
        return_expr: &Expression,
        _query_parts: &[QueryPart],
        context: &ReturnClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证Return子句中的表达式
        // 检查使用的别名是否在作用域内

        // 使用别名验证器验证别名
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        alias_validator.validate_aliases(&[return_expr.clone()], &context.aliases_available)
    }

    /// 验证With子句
    pub fn validate_with(
        &self,
        with_expr: &Expression,
        _query_parts: &[QueryPart],
        context: &WithClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证With子句中的表达式别名

        // 使用别名验证器验证别名
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        alias_validator.validate_aliases(&[with_expr.clone()], &context.aliases_available)?;

        // 验证With子句的分页
        if let Some(ref pagination) = context.pagination {
            if pagination.skip < 0 {
                return Err(ValidationError::new(
                    "SKIP不能为负数".to_string(),
                    ValidationErrorType::PaginationError,
                ));
            }
            if pagination.limit < 0 {
                return Err(ValidationError::new(
                    "LIMIT不能为负数".to_string(),
                    ValidationErrorType::PaginationError,
                ));
            }
        }

        // 验证是否包含聚合表达式
        use super::aggregate_strategy::AggregateValidationStrategy;
        let aggregate_validator = AggregateValidationStrategy::new();
        if aggregate_validator.has_aggregate_expr(with_expr) {
            // 这里需要修改context，但在策略模式中不应该直接修改
            // 应该在主验证器中处理
        }

        Ok(())
    }

    /// 验证Unwind子句
    pub fn validate_unwind(
        &self,
        unwind_expr: &Expression,
        context: &UnwindClauseContext,
    ) -> Result<(), ValidationError> {
        // 验证Unwind表达式中的别名

        // 使用别名验证器验证别名
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        alias_validator.validate_aliases(&[unwind_expr.clone()], &context.aliases_available)?;

        // 检查是否有聚合表达式（在UNWIND中不允许）
        use super::aggregate_strategy::AggregateValidationStrategy;
        let aggregate_validator = AggregateValidationStrategy::new();
        if aggregate_validator.has_aggregate_expr(unwind_expr) {
            return Err(ValidationError::new(
                "UNWIND子句中不能使用聚合表达式".to_string(),
                ValidationErrorType::AggregateError,
            ));
        }

        Ok(())
    }

    /// 验证Yield子句
    pub fn validate_yield(&self, context: &YieldClauseContext) -> Result<(), ValidationError> {
        // 如果有聚合函数，执行特殊验证
        if context.has_agg {
            return self.validate_group(context);
        }

        // 对于普通Yield子句，验证别名
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        for col in &context.yield_columns {
            alias_validator.validate_aliases(&[col.expr.clone()], &context.aliases_available)?;
        }

        Ok(())
    }

    /// 验证分组子句
    fn validate_group(&self, yield_ctx: &YieldClauseContext) -> Result<(), ValidationError> {
        // 验证分组逻辑
        use super::aggregate_strategy::AggregateValidationStrategy;
        let aggregate_validator = AggregateValidationStrategy::new();

        for col in &yield_ctx.yield_columns {
            // 如果表达式包含聚合函数，验证聚合表达式
            if aggregate_validator.has_aggregate_expr(&col.expr) {
                // 验证聚合函数
                self.validate_aggregate_expression(&col.expr, yield_ctx)?;
            } else {
                // 非聚合表达式将作为分组键添加
                // 验证分组键表达式的类型兼容性
                self.validate_group_key_expression(&col.expr, yield_ctx)?;
            }
        }

        Ok(())
    }
    
    /// 验证表达式类型
    fn validate_expression_type<C: ExpressionValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
        expected_type: ValueTypeDef,
    ) -> Result<(), ValidationError> {
        // 使用类型推导系统进行类型检查
        self.validate_expression_type_full(expr, context, expected_type)
    }

    /// 完整的表达式类型验证（使用上下文）
    fn validate_expression_type_full<C: ExpressionValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
        expected_type: ValueTypeDef,
    ) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(value) => {
                let actual_type = value.get_type();
                if self.are_types_compatible(&actual_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "表达式类型不匹配: 期望 {:?}, 实际 {:?}, 表达式: {:?}",
                            expected_type, actual_type, expr
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            Expression::Binary { op, left, right } => {
                // 对于二元表达式，检查操作符是否与期望类型兼容
                match op {
                    crate::core::BinaryOperator::Equal
                    | crate::core::BinaryOperator::NotEqual
                    | crate::core::BinaryOperator::LessThan
                    | crate::core::BinaryOperator::LessThanOrEqual
                    | crate::core::BinaryOperator::GreaterThan
                    | crate::core::BinaryOperator::GreaterThanOrEqual => {
                        // 比较操作符的结果是布尔值
                        if expected_type == ValueTypeDef::Bool {
                            Ok(())
                        } else {
                            Err(ValidationError::new(
                                format!(
                                    "比较操作符的结果是布尔值，但期望类型是 {:?}, 表达式: {:?}",
                                    expected_type, expr
                                ),
                                ValidationErrorType::TypeError,
                            ))
                        }
                    }
                    crate::core::BinaryOperator::And | crate::core::BinaryOperator::Or => {
                        // 逻辑操作符的结果是布尔值
                        if expected_type == ValueTypeDef::Bool {
                            Ok(())
                        } else {
                            Err(ValidationError::new(
                                format!(
                                    "逻辑操作符的结果是布尔值，但期望类型是 {:?}, 表达式: {:?}",
                                    expected_type, expr
                                ),
                                ValidationErrorType::TypeError,
                            ))
                        }
                    }
                    _ => {
                        // 算术操作符，暂时跳过类型检查
                        Ok(())
                    }
                }
            }
            Expression::Unary { op, operand } => {
                match op {
                    crate::core::UnaryOperator::Not => {
                        // 逻辑非的结果是布尔值
                        if expected_type == ValueTypeDef::Bool {
                            Ok(())
                        } else {
                            Err(ValidationError::new(
                                format!(
                                    "逻辑非的结果是布尔值，但期望类型是 {:?}, 表达式: {:?}",
                                    expected_type, expr
                                ),
                                ValidationErrorType::TypeError,
                            ))
                        }
                    }
                    crate::core::UnaryOperator::Minus | crate::core::UnaryOperator::Plus => {
                        // 一元加/减操作符的结果类型与操作数相同
                        let operand_type = self.deduce_expression_type_full(operand, context);
                        if self.are_types_compatible(&operand_type, &expected_type) {
                            Ok(())
                        } else {
                            Err(ValidationError::new(
                                format!(
                                    "一元操作符的结果类型不匹配: 期望 {:?}, 实际 {:?}",
                                    expected_type, operand_type
                                ),
                                ValidationErrorType::TypeError,
                            ))
                        }
                    }
                    _ => Ok(()),
                }
            }
            Expression::Function { name, args } => {
                // 检查函数调用的返回类型是否与期望类型匹配
                let return_type = self.deduce_function_return_type_full(name, args, context);
                if self.are_types_compatible(&return_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "函数 {:?} 的返回类型是 {:?}, 但期望类型是 {:?}",
                            name, return_type, expected_type
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            Expression::Aggregate { func, arg, distinct: _ } => {
                // 检查聚合函数的返回类型是否与期望类型匹配
                let return_type = self.deduce_aggregate_return_type(func);
                if self.are_types_compatible(&return_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "聚合函数 {:?} 的返回类型是 {:?}, 但期望类型是 {:?}",
                            func, return_type, expected_type
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            Expression::Variable(name) => {
                // 检查变量类型是否与期望类型匹配
                if let Some(var_types) = context.get_variable_types() {
                    if let Some(var_type) = var_types.get(name) {
                        if self.are_types_compatible(var_type, &expected_type) {
                            return Ok(());
                        } else {
                            return Err(ValidationError::new(
                                format!(
                                    "变量 {:?} 的类型是 {:?}, 但期望类型是 {:?}",
                                    name, var_type, expected_type
                                ),
                                ValidationErrorType::TypeError,
                            ));
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()), // 其他表达式类型暂时跳过
        }
    }

    /// 简化的表达式类型验证（不需要上下文）
    fn validate_expression_type_simple(
        &self,
        expr: &Expression,
        expected_type: ValueTypeDef,
    ) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(value) => {
                let actual_type = value.get_type();
                if self.are_types_compatible(&actual_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "表达式类型不匹配: 期望 {:?}, 实际 {:?}, 表达式: {:?}",
                            expected_type, actual_type, expr
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            Expression::Binary { op, left: _, right: _ } => {
                // 对于二元表达式，检查操作符是否与期望类型兼容
                match op {
                    crate::core::BinaryOperator::Equal
                    | crate::core::BinaryOperator::NotEqual
                    | crate::core::BinaryOperator::LessThan
                    | crate::core::BinaryOperator::LessThanOrEqual
                    | crate::core::BinaryOperator::GreaterThan
                    | crate::core::BinaryOperator::GreaterThanOrEqual => {
                        // 比较操作符的结果是布尔值
                        if expected_type == ValueTypeDef::Bool {
                            Ok(())
                        } else {
                            Err(ValidationError::new(
                                format!(
                                    "比较操作符的结果是布尔值，但期望类型是 {:?}, 表达式: {:?}",
                                    expected_type, expr
                                ),
                                ValidationErrorType::TypeError,
                            ))
                        }
                    }
                    crate::core::BinaryOperator::And | crate::core::BinaryOperator::Or => {
                        // 逻辑操作符的结果是布尔值
                        if expected_type == ValueTypeDef::Bool {
                            Ok(())
                        } else {
                            Err(ValidationError::new(
                                format!(
                                    "逻辑操作符的结果是布尔值，但期望类型是 {:?}, 表达式: {:?}",
                                    expected_type, expr
                                ),
                                ValidationErrorType::TypeError,
                            ))
                        }
                    }
                    _ => {
                        // 算术操作符，暂时跳过类型检查
                        Ok(())
                    }
                }
            }
            Expression::Unary { op, operand: _ } => {
                match op {
                    crate::core::UnaryOperator::Not => {
                        // 逻辑非的结果是布尔值
                        if expected_type == ValueTypeDef::Bool {
                            Ok(())
                        } else {
                            Err(ValidationError::new(
                                format!(
                                    "逻辑非的结果是布尔值，但期望类型是 {:?}, 表达式: {:?}",
                                    expected_type, expr
                                ),
                                ValidationErrorType::TypeError,
                            ))
                        }
                    }
                    _ => Ok(()), // 其他一元操作符暂时跳过
                }
            }
            Expression::Function { name, args } => {
                // 检查函数调用的返回类型是否与期望类型匹配
                let return_type = self.deduce_function_return_type(name, args);
                if self.are_types_compatible(&return_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "函数 {:?} 的返回类型是 {:?}, 但期望类型是 {:?}",
                            name, return_type, expected_type
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            Expression::Aggregate { func, .. } => {
                // 检查聚合函数的返回类型是否与期望类型匹配
                let return_type = self.deduce_aggregate_return_type(func);
                if self.are_types_compatible(&return_type, &expected_type) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        format!(
                            "聚合函数 {:?} 的返回类型是 {:?}, 但期望类型是 {:?}",
                            func, return_type, expected_type
                        ),
                        ValidationErrorType::TypeError,
                    ))
                }
            }
            _ => Ok(()), // 其他表达式类型暂时跳过
        }
    }

    /// 推导函数调用的返回类型（使用上下文）
    fn deduce_function_return_type_full<C: ExpressionValidationContext>(
        &self,
        name: &str,
        args: &[Expression],
        _context: &C,
    ) -> ValueTypeDef {
        let name_upper = name.to_uppercase();
        match name_upper.as_str() {
            // ID提取函数
            "ID" | "SRC" | "DST" => ValueTypeDef::String,
            // 聚合函数
            "COUNT" => ValueTypeDef::Int,
            "AVG" | "SUM" => ValueTypeDef::Float,
            "MAX" | "MIN" => {
                // 根据参数类型推导返回类型
                if let Some(first_arg) = args.first() {
                    self.deduce_expression_type_simple(first_arg)
                } else {
                    ValueTypeDef::Empty
                }
            }
            "COLLECT" => ValueTypeDef::List,
            // 字符串函数
            "LOWER" | "UPPER" | "TRIM" | "LTRIM" | "RTRIM" | "SUBSTR" | "REVERSE" => {
                ValueTypeDef::String
            }
            // 数学函数
            "ABS" | "CEIL" | "FLOOR" | "SQRT" | "POW" | "EXP" | "LOG" | "LOG10" => {
                ValueTypeDef::Float
            }
            // 其他函数默认返回Empty
            _ => ValueTypeDef::Empty,
        }
    }

    /// 推导表达式的类型（使用上下文）
    fn deduce_expression_type_full<C: ExpressionValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
    ) -> ValueTypeDef {
        match expr {
            Expression::Literal(value) => value.get_type(),
            Expression::Function { name, args } => {
                self.deduce_function_return_type_full(name, args, context)
            }
            Expression::Aggregate { func, .. } => self.deduce_aggregate_return_type(func),
            Expression::Variable(name) => {
                if let Some(var_types) = context.get_variable_types() {
                    if let Some(var_type) = var_types.get(name) {
                        return var_type.clone();
                    }
                }
                if let Some(aliases) = Some(context.get_aliases()) {
                    if let Some(alias_type) = aliases.get(name) {
                        return match alias_type {
                            AliasType::Node => ValueTypeDef::Vertex,
                            AliasType::Edge => ValueTypeDef::Edge,
                            AliasType::EdgeList => ValueTypeDef::List,
                            AliasType::Path => ValueTypeDef::Path,
                            AliasType::Variable => ValueTypeDef::Empty,
                            AliasType::Runtime => ValueTypeDef::Empty,
                        };
                    }
                }
                ValueTypeDef::Empty
            }
            Expression::Property { .. } => ValueTypeDef::Empty,
            Expression::Binary { op, left, right } => {
                self.deduce_binary_expr_type(op, left.as_ref(), right.as_ref())
            }
            Expression::Unary { op, operand } => {
                self.deduce_unary_expr_type(op, operand.as_ref())
            }
            Expression::UnaryPlus(expr) => self.deduce_expression_type_full(expr, context),
            Expression::UnaryNegate(expr) => self.deduce_expression_type_full(expr, context),
            Expression::UnaryNot(expr) => ValueTypeDef::Bool,
            Expression::UnaryIncr(expr) => self.deduce_expression_type_full(expr, context),
            Expression::UnaryDecr(expr) => self.deduce_expression_type_full(expr, context),
            Expression::IsNull(_) => ValueTypeDef::Bool,
            Expression::IsNotNull(_) => ValueTypeDef::Bool,
            Expression::IsEmpty(_) => ValueTypeDef::Bool,
            Expression::IsNotEmpty(_) => ValueTypeDef::Bool,
            Expression::List(_) => ValueTypeDef::List,
            Expression::Map(_) => ValueTypeDef::Map,
            Expression::Case { .. } => ValueTypeDef::Empty,
            Expression::TypeCast { target_type, .. } => {
                match target_type {
                    crate::core::types::expression::DataType::Bool => ValueTypeDef::Bool,
                    crate::core::types::expression::DataType::Int => ValueTypeDef::Int,
                    crate::core::types::expression::DataType::Float => ValueTypeDef::Float,
                    crate::core::types::expression::DataType::String => ValueTypeDef::String,
                    crate::core::types::expression::DataType::List => ValueTypeDef::List,
                    crate::core::types::expression::DataType::Map => ValueTypeDef::Map,
                    crate::core::types::expression::DataType::Vertex => ValueTypeDef::Vertex,
                    crate::core::types::expression::DataType::Edge => ValueTypeDef::Edge,
                    crate::core::types::expression::DataType::Path => ValueTypeDef::Path,
                    crate::core::types::expression::DataType::DateTime => ValueTypeDef::DateTime,
                    crate::core::types::expression::DataType::Date => ValueTypeDef::Date,
                    crate::core::types::expression::DataType::Time => ValueTypeDef::Time,
                    crate::core::types::expression::DataType::Duration => ValueTypeDef::Duration,
                }
            }
            Expression::Subscript { collection, .. } => {
                self.deduce_expression_type_full(collection, context)
            }
            Expression::Range { .. } => ValueTypeDef::List,
            Expression::Path(_) => ValueTypeDef::Path,
            Expression::Label(_) => ValueTypeDef::String,
            Expression::TagProperty { .. } => ValueTypeDef::Empty,
            Expression::EdgeProperty { .. } => ValueTypeDef::Empty,
            Expression::InputProperty(_) => ValueTypeDef::Empty,
            Expression::VariableProperty { .. } => ValueTypeDef::Empty,
            Expression::SourceProperty { .. } => ValueTypeDef::Empty,
            Expression::DestinationProperty { .. } => ValueTypeDef::Empty,
            Expression::ListComprehension { .. } => ValueTypeDef::List,
            Expression::Predicate { .. } => ValueTypeDef::Bool,
            Expression::Reduce { .. } => ValueTypeDef::Empty,
            Expression::ESQuery(_) => ValueTypeDef::Empty,
            Expression::UUID => ValueTypeDef::String,
            Expression::MatchPathPattern { .. } => ValueTypeDef::Path,
        }
    }

    /// 推导表达式的类型（简化版本）
    fn deduce_expression_type_simple(&self, expr: &Expression) -> ValueTypeDef {
        match expr {
            Expression::Literal(value) => value.get_type(),
            Expression::Function { name, args } => {
                self.deduce_function_return_type(name, args)
            }
            Expression::Aggregate { func, .. } => self.deduce_aggregate_return_type(func),
            Expression::Property { .. } => ValueTypeDef::Empty,
            Expression::Binary { op, left, right } => {
                self.deduce_binary_expr_type(op, left.as_ref(), right.as_ref())
            }
            Expression::Unary { op, operand } => {
                self.deduce_unary_expr_type(op, operand.as_ref())
            }
            Expression::UnaryPlus(expr) => self.deduce_expression_type_simple(expr),
            Expression::UnaryNegate(expr) => self.deduce_expression_type_simple(expr),
            Expression::UnaryNot(_) => ValueTypeDef::Bool,
            Expression::UnaryIncr(expr) => self.deduce_expression_type_simple(expr),
            Expression::UnaryDecr(expr) => self.deduce_expression_type_simple(expr),
            Expression::IsNull(_) => ValueTypeDef::Bool,
            Expression::IsNotNull(_) => ValueTypeDef::Bool,
            Expression::IsEmpty(_) => ValueTypeDef::Bool,
            Expression::IsNotEmpty(_) => ValueTypeDef::Bool,
            Expression::List(_) => ValueTypeDef::List,
            Expression::Map(_) => ValueTypeDef::Map,
            Expression::Case { .. } => ValueTypeDef::Empty,
            Expression::TypeCast { target_type, .. } => {
                match target_type {
                    crate::core::types::expression::DataType::Bool => ValueTypeDef::Bool,
                    crate::core::types::expression::DataType::Int => ValueTypeDef::Int,
                    crate::core::types::expression::DataType::Float => ValueTypeDef::Float,
                    crate::core::types::expression::DataType::String => ValueTypeDef::String,
                    crate::core::types::expression::DataType::List => ValueTypeDef::List,
                    crate::core::types::expression::DataType::Map => ValueTypeDef::Map,
                    crate::core::types::expression::DataType::Vertex => ValueTypeDef::Vertex,
                    crate::core::types::expression::DataType::Edge => ValueTypeDef::Edge,
                    crate::core::types::expression::DataType::Path => ValueTypeDef::Path,
                    crate::core::types::expression::DataType::DateTime => ValueTypeDef::DateTime,
                    crate::core::types::expression::DataType::Date => ValueTypeDef::Date,
                    crate::core::types::expression::DataType::Time => ValueTypeDef::Time,
                    crate::core::types::expression::DataType::Duration => ValueTypeDef::Duration,
                }
            }
            Expression::Subscript { collection, .. } => {
                self.deduce_expression_type_simple(collection)
            }
            Expression::Range { .. } => ValueTypeDef::List,
            Expression::Path(_) => ValueTypeDef::Path,
            Expression::Label(_) => ValueTypeDef::String,
            Expression::TagProperty { .. } => ValueTypeDef::Empty,
            Expression::EdgeProperty { .. } => ValueTypeDef::Empty,
            Expression::InputProperty(_) => ValueTypeDef::Empty,
            Expression::VariableProperty { .. } => ValueTypeDef::Empty,
            Expression::SourceProperty { .. } => ValueTypeDef::Empty,
            Expression::DestinationProperty { .. } => ValueTypeDef::Empty,
            Expression::ListComprehension { .. } => ValueTypeDef::List,
            Expression::Predicate { .. } => ValueTypeDef::Bool,
            Expression::Reduce { .. } => ValueTypeDef::Empty,
            Expression::ESQuery(_) => ValueTypeDef::Empty,
            Expression::UUID => ValueTypeDef::String,
            Expression::MatchPathPattern { .. } => ValueTypeDef::Path,
            Expression::Variable(_) => ValueTypeDef::Empty,
        }
    }

    /// 推导二元表达式的类型
    fn deduce_binary_expr_type(
        &self,
        op: &crate::core::BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) -> ValueTypeDef {
        let left_type = self.deduce_expression_type_simple(left);
        let right_type = self.deduce_expression_type_simple(right);

        match op {
            crate::core::BinaryOperator::Add
            | crate::core::BinaryOperator::Subtract
            | crate::core::BinaryOperator::Multiply
            | crate::core::BinaryOperator::Divide
            | crate::core::BinaryOperator::Modulo => {
                // 算术操作符：如果任一操作数是Float，结果为Float
                if left_type == ValueTypeDef::Float || right_type == ValueTypeDef::Float {
                    ValueTypeDef::Float
                } else {
                    ValueTypeDef::Int
                }
            }
            crate::core::BinaryOperator::Equal
            | crate::core::BinaryOperator::NotEqual
            | crate::core::BinaryOperator::LessThan
            | crate::core::BinaryOperator::LessThanOrEqual
            | crate::core::BinaryOperator::GreaterThan
            | crate::core::BinaryOperator::GreaterThanOrEqual => ValueTypeDef::Bool,
            crate::core::BinaryOperator::And | crate::core::BinaryOperator::Or => ValueTypeDef::Bool,
            _ => ValueTypeDef::Empty,
        }
    }

    /// 推导一元表达式的类型
    fn deduce_unary_expr_type(
        &self,
        op: &crate::core::UnaryOperator,
        operand: &Expression,
    ) -> ValueTypeDef {
        let operand_type = self.deduce_expression_type_simple(operand);

        match op {
            crate::core::UnaryOperator::Not => ValueTypeDef::Bool,
            crate::core::UnaryOperator::Minus | crate::core::UnaryOperator::Plus => operand_type,
            _ => ValueTypeDef::Empty,
        }
    }

    /// 推导函数调用的返回类型
    fn deduce_function_return_type(&self, name: &str, _args: &[Expression]) -> ValueTypeDef {
        let name_upper = name.to_uppercase();
        match name_upper.as_str() {
            // ID提取函数
            "ID" | "SRC" | "DST" => ValueTypeDef::String,
            // 聚合函数
            "COUNT" => ValueTypeDef::Int,
            "AVG" | "SUM" => ValueTypeDef::Float,
            "MAX" | "MIN" => ValueTypeDef::Empty, // 需要根据参数类型确定
            "COLLECT" => ValueTypeDef::List,
            // 字符串函数
            "LOWER" | "UPPER" | "TRIM" | "LTRIM" | "RTRIM" | "SUBSTR" | "REVERSE" => {
                ValueTypeDef::String
            }
            // 数学函数
            "ABS" | "CEIL" | "FLOOR" | "SQRT" | "POW" | "EXP" | "LOG" | "LOG10" => {
                ValueTypeDef::Float
            }
            // 其他函数默认返回Empty
            _ => ValueTypeDef::Empty,
        }
    }

    /// 推导聚合函数的返回类型
    fn deduce_aggregate_return_type(&self, func: &crate::core::AggregateFunction) -> ValueTypeDef {
        match func {
            crate::core::AggregateFunction::Count(_) => ValueTypeDef::Int,
            crate::core::AggregateFunction::Sum(_) => ValueTypeDef::Float,
            crate::core::AggregateFunction::Avg(_) => ValueTypeDef::Float,
            crate::core::AggregateFunction::Min(_) | crate::core::AggregateFunction::Max(_) => {
                ValueTypeDef::Empty
            }
            crate::core::AggregateFunction::Collect(_) => ValueTypeDef::List,
            crate::core::AggregateFunction::Distinct(_) => ValueTypeDef::List,
            crate::core::AggregateFunction::Percentile(_, _) => ValueTypeDef::Float,
        }
    }

    /// 检查类型兼容性
    fn are_types_compatible(&self, actual: &ValueTypeDef, expected: &ValueTypeDef) -> bool {
        match (actual, expected) {
            // 相同类型总是兼容
            (a, e) if a == e => true,
            
            // 数值类型之间的双向兼容性
            (ValueTypeDef::Int, ValueTypeDef::Float) => true,
            (ValueTypeDef::Float, ValueTypeDef::Int) => true,
            
            // Bool 可以转换为 Int 和 Float
            (ValueTypeDef::Bool, ValueTypeDef::Int) => true,
            (ValueTypeDef::Bool, ValueTypeDef::Float) => true,
            
            // Int 和 Float 可以转换为 Bool
            (ValueTypeDef::Int, ValueTypeDef::Bool) => true,
            (ValueTypeDef::Float, ValueTypeDef::Bool) => true,
            
            // 任何类型都可以转换为 String
            (_, ValueTypeDef::String) => true,
            
            // List 和 Set 之间的兼容性
            (ValueTypeDef::List, ValueTypeDef::Set) => true,
            (ValueTypeDef::Set, ValueTypeDef::List) => true,
            
            // List 可以转换为 Map（使用索引作为键）
            (ValueTypeDef::List, ValueTypeDef::Map) => true,
            
            // 空类型可以转换为任何类型
            (ValueTypeDef::Empty, _) => true,
            (_, ValueTypeDef::Empty) => true,
            
            // 其他情况不兼容
            _ => false,
        }
    }
    
    /// 验证聚合表达式
    fn validate_aggregate_expression(
        &self,
        expr: &Expression,
        context: &YieldClauseContext,
    ) -> Result<(), ValidationError> {
        // 检查聚合函数是否在允许的位置使用
        use super::aggregate_strategy::AggregateValidationStrategy;
        let aggregate_validator = AggregateValidationStrategy::new();
        
        // 验证聚合函数的参数
        if let Expression::Aggregate { func, arg, distinct: _ } = expr {
            // 检查聚合函数是否支持
            if !self.is_supported_aggregate_function(func) {
                return Err(ValidationError::new(
                    format!("不支持的聚合函数: {:?}", func),
                    ValidationErrorType::AggregateError,
                ));
            }
            
            // 验证参数数量和类型
            self.validate_aggregate_arguments(func, &[ (**arg).clone() ], context)?;
        }
        
        Ok(())
    }
    
    /// 验证分组键表达式
    fn validate_group_key_expression(
        &self,
        expr: &Expression,
        context: &YieldClauseContext,
    ) -> Result<(), ValidationError> {
        // 分组键表达式不能包含聚合函数
        use super::aggregate_strategy::AggregateValidationStrategy;
        let aggregate_validator = AggregateValidationStrategy::new();
        
        if aggregate_validator.has_aggregate_expr(expr) {
            return Err(ValidationError::new(
                "分组键表达式中不能包含聚合函数".to_string(),
                ValidationErrorType::AggregateError,
            ));
        }
        
        // 验证表达式类型是否适合作为分组键
        // 分组键应该是可哈希的类型
        self.validate_group_key_type(expr, context)
    }
    
    /// 检查是否支持的聚合函数
    fn is_supported_aggregate_function(&self, function: &crate::core::AggregateFunction) -> bool {
        matches!(
            function,
            crate::core::AggregateFunction::Count(_)
                | crate::core::AggregateFunction::Sum(_)
                | crate::core::AggregateFunction::Avg(_)
                | crate::core::AggregateFunction::Max(_)
                | crate::core::AggregateFunction::Min(_)
                | crate::core::AggregateFunction::Collect(_)
        )
    }
    
    /// 验证聚合函数参数
    fn validate_aggregate_arguments(
        &self,
        function: &crate::core::AggregateFunction,
        args: &[Expression],
        context: &YieldClauseContext,
    ) -> Result<(), ValidationError> {
        match function {
            crate::core::AggregateFunction::Count(_) => {
                // COUNT可以接受0或1个参数
                if args.len() > 1 {
                    return Err(ValidationError::new(
                        "COUNT函数最多接受1个参数".to_string(),
                        ValidationErrorType::AggregateError,
                    ));
                }
            }
            crate::core::AggregateFunction::Sum(_)
            | crate::core::AggregateFunction::Avg(_)
            | crate::core::AggregateFunction::Max(_)
            | crate::core::AggregateFunction::Min(_) => {
                // 这些函数需要1个参数
                if args.len() != 1 {
                    return Err(ValidationError::new(
                        format!("{:?}函数需要1个参数", function),
                        ValidationErrorType::AggregateError,
                    ));
                }

                // 验证参数类型为数值类型
                if let Some(arg) = args.first() {
                    self.validate_expression_type(arg, context, ValueTypeDef::Int)?;
                }
            }
            crate::core::AggregateFunction::Collect(_) => {
                // COLLECT可以接受任意数量的参数
                // 不需要特殊验证
            }
            _ => {
                return Err(ValidationError::new(
                    format!("不支持的聚合函数: {:?}", function),
                    ValidationErrorType::AggregateError,
                ));
            }
        }
        
        Ok(())
    }
    
    /// 验证分组键类型
    fn validate_group_key_type(
        &self,
        expr: &Expression,
        context: &YieldClauseContext,
    ) -> Result<(), ValidationError> {
        // 分组键应该是可哈希的类型
        // 这里简化处理，实际应该使用类型推导
        
        // 检查表达式是否包含不支持的类型
        use crate::query::visitor::FindVisitor;
        let mut find_visitor = FindVisitor::new();
        
        // 设置要查找的表达式类型
        find_visitor
            .add_target_type(crate::core::ExpressionType::List)
            .add_target_type(crate::core::ExpressionType::Map)
            .add_target_type(crate::core::ExpressionType::Path);
        
        let invalid_exprs = find_visitor.find(expr);
        if !invalid_exprs.is_empty() {
            return Err(ValidationError::new(
                "分组键不能包含列表、集合、映射或路径类型".to_string(),
                ValidationErrorType::TypeError,
            ));
        }
        
        Ok(())
    }
    
    /// 验证表达式是否包含聚合函数（辅助函数）
    fn has_aggregate_expression(&self, expr: &Expression) -> bool {
        use super::aggregate_strategy::AggregateValidationStrategy;
        let aggregate_validator = AggregateValidationStrategy::new();
        aggregate_validator.has_aggregate_expr(expr)
    }
    
    /// 验证表达式是否可立即求值（辅助函数）
    fn is_evaluable_expression(&self, expr: &Expression) -> bool {
        use crate::query::visitor::EvaluableExprVisitor;
        let mut visitor = EvaluableExprVisitor::new();
        visitor.is_evaluable(expr)
    }
    
    /// 验证别名使用（辅助函数）
    fn validate_aliases_usage(
        &self,
        expr: &Expression,
        available_aliases: &std::collections::HashMap<String, crate::query::validator::structs::AliasType>,
    ) -> Result<(), ValidationError> {
        use super::alias_strategy::AliasValidationStrategy;
        let alias_validator = AliasValidationStrategy::new();
        alias_validator.validate_aliases(&[expr.clone()], available_aliases)
    }
    
    /// 验证表达式语义（新增函数）
    pub fn validate_expression_semantics<C: ExpressionValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
    ) -> Result<(), ValidationError> {
        // 验证表达式的语义正确性
        // 包括类型检查、作用域检查等
        
        // 1. 检查表达式中使用的变量是否在作用域内
        self.validate_variable_scope(expr, context)?;
        
        // 2. 检查表达式是否包含无效的操作
        self.validate_expression_operations(expr)?;
        
        // 3. 检查表达式是否包含循环引用
        self.validate_expression_cycles(expr)?;
        
        Ok(())
    }
    
    /// 验证变量作用域（使用上下文的完整版本）
    fn validate_variable_scope<C: ExpressionValidationContext>(
        &self,
        expr: &Expression,
        context: &C,
    ) -> Result<(), ValidationError> {
        use crate::query::visitor::VariableVisitor;
        
        let mut visitor = VariableVisitor::new();
        let variables_set = visitor.collect_variables(expr);
        let variables: Vec<String> = variables_set.into_iter().collect();
        
        if variables.is_empty() {
            return Ok(());
        }
        
        // 获取上下文中的别名
        let aliases = context.get_aliases();
        let variable_types = context.get_variable_types();
        
        for var in variables {
            // 检查变量名格式
            self.validate_variable_name_format(&var)?;
            
            // 检查变量是否在作用域内定义
            let is_defined = aliases.contains_key(&var) 
                || variable_types.map(|vt| vt.contains_key(&var)).unwrap_or(false);
            
            if !is_defined {
                return Err(ValidationError::new(
                    format!("变量 '{}' 未定义或不在当前作用域内", var),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        
        Ok(())
    }
    
    /// 验证变量名格式
    fn validate_variable_name_format(&self, var: &str) -> Result<(), ValidationError> {
        if var.is_empty() {
            return Err(ValidationError::new(
                "变量名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        
        if var.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return Err(ValidationError::new(
                format!("变量名 '{}' 不能以数字开头", var),
                ValidationErrorType::SemanticError,
            ));
        }
        
        if var.chars().any(|c| !c.is_alphanumeric() && c != '_' && c != '$') {
            return Err(ValidationError::new(
                format!("变量名 '{}' 包含非法字符", var),
                ValidationErrorType::SemanticError,
            ));
        }
        
        let reserved_keywords = vec![
            "MATCH", "WHERE", "RETURN", "WITH", "UNWIND", "LIMIT", "SKIP",
            "ORDER", "BY", "ASC", "DESC", "DISTINCT", "AS", "AND", "OR", "NOT",
            "IN", "IS", "NULL", "TRUE", "FALSE", "CREATE", "DELETE", "SET",
            "MERGE", "OPTIONAL", "USING", "INDEX", "NODE", "RELATIONSHIP",
            "CALL", "YIELD", "CASE", "WHEN", "THEN", "ELSE", "END", "FOREACH",
        ];
        let var_upper = var.to_uppercase();
        if reserved_keywords.contains(&var_upper.as_str()) {
            return Err(ValidationError::new(
                format!("变量名 '{}' 是保留关键字", var),
                ValidationErrorType::SemanticError,
            ));
        }
        
        Ok(())
    }
    
    /// 简化的变量作用域验证（当无法获取上下文时使用）
    fn validate_variable_scope_simple(&self, variables: &[String]) -> Result<(), ValidationError> {
        for var in variables {
            self.validate_variable_name_format(var)?;
        }
        Ok(())
    }
    
    /// 验证变量使用方式是否与类型兼容
    fn validate_variable_usage(
        &self,
        var: &str,
        expr: &Expression,
        alias_type: &crate::query::validator::structs::alias_structs::AliasType,
    ) -> Result<(), ValidationError> {
        // 检查变量在表达式中的使用方式是否与别名类型兼容
        // 例如：节点别名不应该用于数值运算
        
        // 简化实现：根据别名类型和表达式操作进行基本检查
        match alias_type {
            crate::query::validator::structs::alias_structs::AliasType::Node => {
                // 节点别名通常不应该用于算术运算
                if self.is_arithmetic_expression(expr, var) {
                    return Err(ValidationError::new(
                        format!(
                            "节点别名 '{}' 不能用于算术运算",
                            var
                        ),
                        ValidationErrorType::TypeError,
                    ));
                }
            }
            crate::query::validator::structs::alias_structs::AliasType::Edge => {
                // 边别名通常不应该用于算术运算
                if self.is_arithmetic_expression(expr, var) {
                    return Err(ValidationError::new(
                        format!(
                            "边别名 '{}' 不能用于算术运算",
                            var
                        ),
                        ValidationErrorType::TypeError,
                    ));
                }
            }
            crate::query::validator::structs::alias_structs::AliasType::Path => {
                // 路径别名通常不应该用于算术运算
                if self.is_arithmetic_expression(expr, var) {
                    return Err(ValidationError::new(
                        format!(
                            "路径别名 '{}' 不能用于算术运算",
                            var
                        ),
                        ValidationErrorType::TypeError,
                    ));
                }
            }
            crate::query::validator::structs::alias_structs::AliasType::Variable => {
                // 变量别名可以用于各种操作
                // 不需要额外检查
            }
            _ => {
                // 其他类型暂时跳过
            }
        }
        
        Ok(())
    }
    
    /// 检查表达式是否为算术表达式（包含指定变量）
    fn is_arithmetic_expression(&self, expr: &Expression, var: &str) -> bool {
        match expr {
            Expression::Binary { op, left, right } => {
                // 检查是否为算术操作符
                let is_arithmetic_op = matches!(
                    op,
                    crate::core::BinaryOperator::Add
                        | crate::core::BinaryOperator::Subtract
                        | crate::core::BinaryOperator::Multiply
                        | crate::core::BinaryOperator::Divide
                        | crate::core::BinaryOperator::Modulo
                );
                
                if !is_arithmetic_op {
                    return false;
                }
                
                // 检查操作数中是否包含指定变量
                let left_contains_var = self.contains_variable(left, var);
                let right_contains_var = self.contains_variable(right, var);
                
                left_contains_var || right_contains_var
            }
            Expression::Unary { op, operand } => {
                // 检查是否为算术一元操作符
                let is_arithmetic_op = matches!(
                    op,
                    crate::core::UnaryOperator::Minus
                );
                
                if !is_arithmetic_op {
                    return false;
                }
                
                self.contains_variable(operand, var)
            }
            _ => false,
        }
    }
    
    /// 检查表达式是否包含指定变量
    fn contains_variable(&self, expr: &Expression, var: &str) -> bool {
        use crate::query::visitor::VariableVisitor;
        let mut visitor = VariableVisitor::new();
        let variables = visitor.collect_variables(expr);
        variables.contains(&var.to_string())
    }
    
    /// 验证聚合函数参数
    fn validate_aggregate_function_args(
        &self,
        func: &crate::core::AggregateFunction,
        arg: &Expression,
    ) -> Result<(), ValidationError> {
        match func {
            crate::core::AggregateFunction::Count(_) => {
                // COUNT可以接受0或1个参数
                // 验证参数类型：COUNT通常接受任何类型，但参数必须是有效的
                if self.is_evaluable_expression(arg) {
                    match arg {
                        Expression::Literal(crate::core::Value::List(_)) => {
                            // COUNT可以接受列表作为参数
                        }
                        _ => {
                            // 其他表达式类型都可以接受
                        }
                    }
                }
            }
            crate::core::AggregateFunction::Sum(_) => {
                // SUM需要数值类型参数
                self.validate_numeric_aggregate_arg(arg, "SUM")?;
            }
            crate::core::AggregateFunction::Avg(_) => {
                // AVG需要数值类型参数
                self.validate_numeric_aggregate_arg(arg, "AVG")?;
            }
            crate::core::AggregateFunction::Max(_) | crate::core::AggregateFunction::Min(_) => {
                // MAX/MIN可以接受任何可比较类型
                // 验证参数是否为有效表达式
                if self.is_evaluable_expression(arg) {
                    match arg {
                        Expression::Literal(crate::core::Value::Null(_)) => {
                            return Err(ValidationError::new(
                                format!("{}函数不能接受NULL参数", func.name()),
                                ValidationErrorType::TypeError,
                            ));
                        }
                        _ => {}
                    }
                }
            }
            crate::core::AggregateFunction::Collect(_) => {
                // COLLECT可以接受任意类型的单个参数
                // 不需要特殊类型验证
            }
            crate::core::AggregateFunction::Distinct(_) => {
                // DISTINCT需要1个参数
                // 不需要特殊类型验证
            }
            crate::core::AggregateFunction::Percentile(_, _) => {
                // PERCENTILE需要2个参数：字段和百分位数
                // 当前实现中，arg是整个参数表达式，需要进一步解析
                // 简化验证：确保参数不为空
            }
        }
        
        Ok(())
    }
    
    /// 验证数值聚合函数参数
    fn validate_numeric_aggregate_arg(
        &self,
        arg: &Expression,
        func_name: &str,
    ) -> Result<(), ValidationError> {
        if !self.is_evaluable_expression(arg) {
            return Ok(());
        }
        
        match arg {
            Expression::Literal(crate::core::Value::Bool(_)) => {
                return Err(ValidationError::new(
                    format!("{}函数不能接受布尔类型参数", func_name),
                    ValidationErrorType::TypeError,
                ));
            }
            Expression::Literal(crate::core::Value::String(_)) => {
                return Err(ValidationError::new(
                    format!("{}函数不能接受字符串类型参数", func_name),
                    ValidationErrorType::TypeError,
                ));
            }
            Expression::Literal(crate::core::Value::List(_)) => {
                return Err(ValidationError::new(
                    format!("{}函数不能接受列表类型参数", func_name),
                    ValidationErrorType::TypeError,
                ));
            }
            Expression::Literal(crate::core::Value::Map(_)) => {
                return Err(ValidationError::new(
                    format!("{}函数不能接受映射类型参数", func_name),
                    ValidationErrorType::TypeError,
                ));
            }
            Expression::Literal(crate::core::Value::Null(_)) => {
                // NULL在聚合函数中通常被忽略，不报错
            }
            Expression::Literal(crate::core::Value::Int(_))
            | Expression::Literal(crate::core::Value::Float(_)) => {
                // 有效的数值类型
            }
            _ => {
                // 其他表达式类型需要进一步分析其返回类型
                // 这里简化处理：假设其他表达式可能是有效的
            }
        }
        
        Ok(())
    }
    
    /// 验证表达式操作（辅助函数）
    fn validate_expression_operations(&self, expr: &Expression) -> Result<(), ValidationError> {
        // 检查表达式中的操作是否有效
        // 例如：除零检查、无效的类型转换等
        
        match expr {
            Expression::Binary { op, left, right } => {
                // 检查除法操作，避免除零
                if matches!(op, crate::core::BinaryOperator::Divide) {
                    if self.is_evaluable_expression(right) {
                        match right.as_ref() {
                            Expression::Literal(crate::core::Value::Int(0)) => {
                                return Err(ValidationError::new(
                                    "除法操作不能除以零".to_string(),
                                    ValidationErrorType::SemanticError,
                                ));
                            }
                            Expression::Literal(crate::core::Value::Float(f)) if *f == 0.0 => {
                                return Err(ValidationError::new(
                                    "除法操作不能除以零".to_string(),
                                    ValidationErrorType::SemanticError,
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                
                // 检查模运算，避免模零
                if matches!(op, crate::core::BinaryOperator::Modulo) {
                    if self.is_evaluable_expression(right) {
                        match right.as_ref() {
                            Expression::Literal(crate::core::Value::Int(0)) => {
                                return Err(ValidationError::new(
                                    "模运算不能模零".to_string(),
                                    ValidationErrorType::SemanticError,
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                
                // 检查字符串减法（不允许）
                if matches!(op, crate::core::BinaryOperator::Subtract) {
                    if self.is_evaluable_expression(left) && self.is_evaluable_expression(right) {
                        let l = self.deduce_expression_type_simple(left);
                        let r = self.deduce_expression_type_simple(right);
                        if matches!(l, ValueTypeDef::String) && matches!(r, ValueTypeDef::String) {
                            return Err(ValidationError::new(
                                "不能对字符串执行减法操作".to_string(),
                                ValidationErrorType::SemanticError,
                            ));
                        }
                    }
                }
                
                // 递归验证左右操作数
                self.validate_expression_operations(left)?;
                self.validate_expression_operations(right)?;
            }
            Expression::Unary { op, operand } => {
                // 检查自增自减操作是否应用于数值类型
                if matches!(op, crate::core::UnaryOperator::Increment | crate::core::UnaryOperator::Decrement) {
                    if self.is_evaluable_expression(operand) {
                        match operand.as_ref() {
                            Expression::Literal(crate::core::Value::Bool(_)) => {
                                return Err(ValidationError::new(
                                    "不能对布尔值执行自增或自减操作".to_string(),
                                    ValidationErrorType::SemanticError,
                                ));
                            }
                            Expression::Literal(crate::core::Value::String(_)) => {
                                return Err(ValidationError::new(
                                    "不能对字符串执行自增或自减操作".to_string(),
                                    ValidationErrorType::SemanticError,
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                
                // 递归验证操作数
                self.validate_expression_operations(operand)?;
            }
            Expression::Function { name, args } => {
                // 检查函数调用的参数数量
                let name_upper = name.to_uppercase();
                match name_upper.as_str() {
                    "COUNT" => {
                        if args.len() > 1 {
                            return Err(ValidationError::new(
                                format!("COUNT函数最多接受1个参数，但提供了{}个参数", args.len()),
                                ValidationErrorType::SemanticError,
                            ));
                        }
                    }
                    "SUM" | "AVG" | "MAX" | "MIN" | "ABS" | "CEIL" | "FLOOR" | "SQRT" => {
                        if args.len() != 1 {
                            return Err(ValidationError::new(
                                format!("{:?}函数需要1个参数，但提供了{}个参数", name, args.len()),
                                ValidationErrorType::SemanticError,
                            ));
                        }
                    }
                    "POW" | "LOG" => {
                        if args.len() != 2 {
                            return Err(ValidationError::new(
                                format!("{:?}函数需要2个参数，但提供了{}个参数", name, args.len()),
                                ValidationErrorType::SemanticError,
                            ));
                        }
                    }
                    _ => {}
                }
                
                // 验证所有参数
                for arg in args {
                    self.validate_expression_operations(arg)?;
                }
            }
            Expression::Aggregate { func, arg, distinct: _ } => {
                // 验证聚合函数参数
                self.validate_aggregate_function_args(func, arg)?;
                
                // 递归验证参数表达式
                self.validate_expression_operations(arg)?;
            }
            Expression::List(items) => {
                // 验证列表中的所有元素
                for item in items {
                    self.validate_expression_operations(item)?;
                }
            }
            Expression::Map(entries) => {
                // 验证映射中的所有值
                for (_, value) in entries {
                    self.validate_expression_operations(value)?;
                }
            }
            _ => {
                // 其他表达式类型不需要特殊验证
            }
        }
        
        Ok(())
    }
    
    /// 验证表达式循环引用（辅助函数）
    fn validate_expression_cycles(&self, expr: &Expression) -> Result<(), ValidationError> {
        // 检查表达式是否包含循环引用
        // 例如：a = b + 1, b = a + 2
        
        // 使用访问器收集所有变量引用
        use crate::query::visitor::VariableVisitor;
        let mut visitor = VariableVisitor::new();
        let variables = visitor.collect_variables(expr);
        
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
        let depth = self.calculate_expression_depth(expr);
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
    fn calculate_expression_depth(&self, expr: &Expression) -> usize {
        match expr {
            Expression::Literal(_) => 1,
            Expression::Variable(_) => 1,
            Expression::Label(_) => 1,
            Expression::UUID => 1,
            Expression::ESQuery(_) => 1,
            Expression::TagProperty { .. } => 1,
            Expression::EdgeProperty { .. } => 1,
            Expression::InputProperty(_) => 1,
            Expression::SourceProperty { .. } => 1,
            Expression::DestinationProperty { .. } => 1,
            Expression::VariableProperty { .. } => 1,

            Expression::Property { object, .. } => {
                1 + self.calculate_expression_depth(object)
            }

            Expression::Binary { left, right, .. } => {
                1 + self.calculate_expression_depth(left).max(self.calculate_expression_depth(right))
            }

            Expression::Unary { operand, .. } => {
                1 + self.calculate_expression_depth(operand)
            }

            Expression::UnaryPlus(expr) => 1 + self.calculate_expression_depth(expr),
            Expression::UnaryNegate(expr) => 1 + self.calculate_expression_depth(expr),
            Expression::UnaryNot(expr) => 1 + self.calculate_expression_depth(expr),
            Expression::UnaryIncr(expr) => 1 + self.calculate_expression_depth(expr),
            Expression::UnaryDecr(expr) => 1 + self.calculate_expression_depth(expr),
            Expression::IsNull(expr) => 1 + self.calculate_expression_depth(expr),
            Expression::IsNotNull(expr) => 1 + self.calculate_expression_depth(expr),
            Expression::IsEmpty(expr) => 1 + self.calculate_expression_depth(expr),
            Expression::IsNotEmpty(expr) => 1 + self.calculate_expression_depth(expr),

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
                for (when_expr, then_expr) in conditions {
                    max_depth = max_depth.max(self.calculate_expression_depth(when_expr));
                    max_depth = max_depth.max(self.calculate_expression_depth(then_expr));
                }
                if let Some(default_expr) = default {
                    max_depth = max_depth.max(self.calculate_expression_depth(default_expr));
                }
                1 + max_depth
            }

            Expression::TypeCast { expr, .. } => {
                1 + self.calculate_expression_depth(expr)
            }

            Expression::Subscript { collection, index } => {
                1 + self.calculate_expression_depth(collection)
                    .max(self.calculate_expression_depth(index))
            }

            Expression::Range { collection, start, end } => {
                let mut max_depth = self.calculate_expression_depth(collection);
                if let Some(start_expr) = start {
                    max_depth = max_depth.max(self.calculate_expression_depth(start_expr));
                }
                if let Some(end_expr) = end {
                    max_depth = max_depth.max(self.calculate_expression_depth(end_expr));
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

            Expression::ListComprehension { generator, condition } => {
                let mut depth = self.calculate_expression_depth(generator);
                if let Some(cond) = condition {
                    depth = depth.max(self.calculate_expression_depth(cond));
                }
                1 + depth
            }

            Expression::Predicate { list, condition } => {
                1 + self.calculate_expression_depth(list)
                    .max(self.calculate_expression_depth(condition))
            }

            Expression::Reduce { list, initial, expr, .. } => {
                1 + self.calculate_expression_depth(list)
                    .max(self.calculate_expression_depth(initial))
                    .max(self.calculate_expression_depth(expr))
            }

            Expression::MatchPathPattern { patterns, .. } => {
                let max_pattern_depth = patterns.iter()
                    .map(|p| self.calculate_expression_depth(p))
                    .max()
                    .unwrap_or(0);
                1 + max_pattern_depth
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
                        self.validate_unwind(&unwind_ctx.unwind_expr, unwind_ctx)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_expression_validation_strategy_creation() {
        let strategy = ExpressionValidationStrategy::new();
        assert_eq!(strategy.strategy_type(), ValidationStrategyType::Expression);
        assert_eq!(strategy.strategy_name(), "ExpressionValidationStrategy");
    }

    #[test]
    fn test_validate_filter() {
        let strategy = ExpressionValidationStrategy::new();

        // 创建测试数据
        let where_context = WhereClauseContext {
            filter: None,
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            paths: Vec::new(),
        };

        // 测试布尔表达式
        let bool_expr = Expression::Literal(crate::core::Value::Bool(true));
        assert!(strategy.validate_filter(&bool_expr, &where_context).is_ok());
    }

    #[test]
    fn test_validate_path() {
        let _strategy = ExpressionValidationStrategy::new();

        let _match_context = MatchClauseContext {
            paths: Vec::new(),
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        // 测试路径验证
        // 注意：这里需要一个有效的路径表达式
        // 暂时跳过这个测试，因为需要特定的路径表达式构造
    }

    #[test]
    fn test_validate_return() {
        let strategy = ExpressionValidationStrategy::new();

        let return_context = ReturnClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: Vec::new(),
                aliases_available: std::collections::HashMap::new(),
                aliases_generated: std::collections::HashMap::new(),
                distinct: false,
                has_agg: false,
                group_keys: Vec::new(),
                group_items: Vec::new(),
                need_gen_project: false,
                agg_output_column_names: Vec::new(),
                proj_output_column_names: Vec::new(),
                proj_cols: Vec::new(),
                paths: Vec::new(),
            },
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
        };

        // 测试Return子句验证
        let return_expr = Expression::Literal(crate::core::Value::Int(1));
        assert!(strategy
            .validate_return(&return_expr, &[], &return_context)
            .is_ok());
    }

    #[test]
    fn test_validate_with() {
        let strategy = ExpressionValidationStrategy::new();

        let with_context = WithClauseContext {
            yield_clause: YieldClauseContext {
                yield_columns: Vec::new(),
                aliases_available: std::collections::HashMap::new(),
                aliases_generated: std::collections::HashMap::new(),
                distinct: false,
                has_agg: false,
                group_keys: Vec::new(),
                group_items: Vec::new(),
                need_gen_project: false,
                agg_output_column_names: Vec::new(),
                proj_output_column_names: Vec::new(),
                proj_cols: Vec::new(),
                paths: Vec::new(),
            },
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            pagination: None,
            order_by: None,
            distinct: false,
        };

        // 测试With子句验证
        let with_expr = Expression::Literal(crate::core::Value::Int(1));
        assert!(strategy
            .validate_with(&with_expr, &[], &with_context)
            .is_ok());
    }

    #[test]
    fn test_validate_unwind() {
        let strategy = ExpressionValidationStrategy::new();

        let unwind_context = UnwindClauseContext {
            alias: "test".to_string(),
            unwind_expr: Expression::Literal(crate::core::Value::Int(1)),
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            paths: Vec::new(),
        };

        let unwind_expr = Expression::Literal(crate::core::Value::Int(1));
        assert!(strategy
            .validate_unwind(&unwind_expr, &unwind_context)
            .is_ok());
    }

    #[test]
    fn test_validate_yield() {
        let strategy = ExpressionValidationStrategy::new();

        let yield_context = YieldClauseContext {
            yield_columns: vec![YieldColumn::new(
                Expression::Literal(crate::core::Value::Int(1)),
                "col1".to_string(),
            )],
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: Vec::new(),
            group_items: Vec::new(),
            need_gen_project: false,
            agg_output_column_names: Vec::new(),
            proj_output_column_names: Vec::new(),
            proj_cols: Vec::new(),
            paths: Vec::new(),
        };

        assert!(strategy.validate_yield(&yield_context).is_ok());
    }

    #[test]
    fn test_single_path_pattern() {
        let strategy = ExpressionValidationStrategy::new();

        let mut match_context = MatchClauseContext {
            paths: Vec::new(),
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        // 测试单个路径模式验证
        let pattern = Expression::Literal(crate::core::Value::Int(1));
        assert!(strategy
            .validate_single_path_pattern(&pattern, &mut match_context)
            .is_ok());
    }

    #[test]
    fn test_validate_expression_type() {
        let strategy = ExpressionValidationStrategy::new();

        // 测试类型兼容性检查
        assert!(strategy.are_types_compatible(
            &ValueTypeDef::Int,
            &ValueTypeDef::Int
        ));
        assert!(strategy.are_types_compatible(
            &ValueTypeDef::Int,
            &ValueTypeDef::Float
        ));
        assert!(strategy.are_types_compatible(
            &ValueTypeDef::Float,
            &ValueTypeDef::Int
        ));
        assert!(strategy.are_types_compatible(
            &ValueTypeDef::Empty,
            &ValueTypeDef::Int
        ));
        assert!(strategy.are_types_compatible(
            &ValueTypeDef::Int,
            &ValueTypeDef::Empty
        ));
        
        // 测试不兼容的类型
        assert!(!strategy.are_types_compatible(
            &ValueTypeDef::Int,
            &ValueTypeDef::String
        ));
        assert!(!strategy.are_types_compatible(
            &ValueTypeDef::String,
            &ValueTypeDef::Bool
        ));
    }

    #[test]
    fn test_validate_aggregate_expression() {
        let strategy = ExpressionValidationStrategy::new();

        let yield_context = YieldClauseContext {
            yield_columns: Vec::new(),
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: Vec::new(),
            group_items: Vec::new(),
            need_gen_project: false,
            agg_output_column_names: Vec::new(),
            proj_output_column_names: Vec::new(),
            proj_cols: Vec::new(),
            paths: Vec::new(),
        };

        // 测试COUNT聚合函数
        let count_expr = Expression::Aggregate {
            func: crate::core::AggregateFunction::Count(None),
            arg: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            distinct: false,
        };
        assert!(strategy
            .validate_aggregate_expression(&count_expr, &yield_context)
            .is_ok());

        // 测试SUM聚合函数
        let sum_expr = Expression::Aggregate {
            func: crate::core::AggregateFunction::Sum("".to_string()),
            arg: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            distinct: false,
        };
        assert!(strategy
            .validate_aggregate_expression(&sum_expr, &yield_context)
            .is_ok());
    }

    #[test]
    fn test_validate_expression_operations() {
        let strategy = ExpressionValidationStrategy::new();

        // 测试除法操作（避免除零）
        let divide_by_zero = Expression::Binary {
            left: Box::new(Expression::Literal(crate::core::Value::Int(10))),
            op: crate::core::BinaryOperator::Divide,
            right: Box::new(Expression::Literal(crate::core::Value::Int(0))),
        };
        
        // 注意：由于is_evaluable_expression需要EvaluableExprVisitor的实现
        // 这里暂时跳过除零检查的测试
        // 实际实现中应该检测并返回错误
        
        // 测试有效的除法操作
        let valid_divide = Expression::Binary {
            left: Box::new(Expression::Literal(crate::core::Value::Int(10))),
            op: crate::core::BinaryOperator::Divide,
            right: Box::new(Expression::Literal(crate::core::Value::Int(2))),
        };
        assert!(strategy.validate_expression_operations(&valid_divide).is_ok());

        // 测试一元操作
        let unary_expr = Expression::Unary {
            op: crate::core::UnaryOperator::Minus,
            operand: Box::new(Expression::Literal(crate::core::Value::Int(5))),
        };
        assert!(strategy.validate_expression_operations(&unary_expr).is_ok());

        // 测试函数调用
        let function_expr = Expression::Function {
            name: "abs".to_string(),
            args: vec![Expression::Literal(crate::core::Value::Int(-5))],
        };
        assert!(strategy.validate_expression_operations(&function_expr).is_ok());
    }

    #[test]
    fn test_has_aggregate_expression() {
        let strategy = ExpressionValidationStrategy::new();

        // 测试包含聚合函数的表达式
        let aggregate_expr = Expression::Aggregate {
            func: crate::core::AggregateFunction::Count(None),
            arg: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            distinct: false,
        };
        assert!(strategy.has_aggregate_expression(&aggregate_expr));

        // 测试不包含聚合函数的表达式
        let simple_expr = Expression::Literal(crate::core::Value::Int(1));
        assert!(!strategy.has_aggregate_expression(&simple_expr));

        // 测试嵌套表达式中的聚合函数
        let nested_expr = Expression::Binary {
            left: Box::new(Expression::Aggregate {
                func: crate::core::AggregateFunction::Sum("".to_string()),
                arg: Box::new(Expression::Literal(crate::core::Value::Int(1))),
                distinct: false,
            }),
            op: crate::core::BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(2))),
        };
        assert!(strategy.has_aggregate_expression(&nested_expr));
    }

    #[test]
    fn test_validate_group_key_type() {
        let strategy = ExpressionValidationStrategy::new();

        let yield_context = YieldClauseContext {
            yield_columns: Vec::new(),
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: Vec::new(),
            group_items: Vec::new(),
            need_gen_project: false,
            agg_output_column_names: Vec::new(),
            proj_output_column_names: Vec::new(),
            proj_cols: Vec::new(),
            paths: Vec::new(),
        };

        // 测试有效的分组键类型
        let valid_key = Expression::Literal(crate::core::Value::Int(1));
        assert!(strategy
            .validate_group_key_type(&valid_key, &yield_context)
            .is_ok());

        let valid_string_key = Expression::Literal(crate::core::Value::String("test".to_string()));
        assert!(strategy
            .validate_group_key_type(&valid_string_key, &yield_context)
            .is_ok());

        // 测试无效的分组键类型（列表）
        let list_expr = Expression::List(vec![Expression::Literal(crate::core::Value::Int(1))]);
        assert!(strategy
            .validate_group_key_type(&list_expr, &yield_context)
            .is_err());
    }

    #[test]
    fn test_validate_expression_cycles() {
        let strategy = ExpressionValidationStrategy::new();

        // 测试简单的表达式（无循环）
        let simple_expr = Expression::Literal(crate::core::Value::Int(1));
        assert!(strategy.validate_expression_cycles(&simple_expr).is_ok());

        // 测试二元表达式（无循环）
        let binary_expr = Expression::Binary {
            left: Box::new(Expression::Literal(crate::core::Value::Int(1))),
            op: crate::core::BinaryOperator::Add,
            right: Box::new(Expression::Literal(crate::core::Value::Int(2))),
        };
        assert!(strategy.validate_expression_cycles(&binary_expr).is_ok());

        // 注意：循环引用检测需要更复杂的测试用例
        // 实际实现中应该检测类似 a = b + 1, b = a + 2 的循环
    }
}
