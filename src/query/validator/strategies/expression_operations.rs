//! 表达式操作验证器
//! 负责验证表达式的操作合法性和结构完整性

use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::types::DataType;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::query::validator::strategies::type_deduce::TypeDeduceValidator;
use std::collections::HashSet;

/// 表达式操作验证器
pub struct ExpressionOperationsValidator;

impl ExpressionOperationsValidator {
    pub fn new() -> Self {
        Self
    }

    /// 验证表达式操作的合法性
    pub fn validate_expression_operations(&self, expression: &ContextualExpression) -> Result<(), ValidationError> {
        if let Some(expr) = expression.expression() {
            // 使用 BFS 方式检查表达式深度（防止 OOM）
            self.check_expression_depth_bfs(expression, 100)?;
            self.validate_expression_operations_recursive(&expr, 0)
        } else {
            Err(ValidationError::new(
                "表达式无效".to_string(),
                ValidationErrorType::ExpressionError,
            ))
        }
    }

    /// 递归验证表达式操作
    fn validate_expression_operations_recursive(&self, expression: &crate::core::types::expression::Expression, depth: usize) -> Result<(), ValidationError> {
        // 检查表达式深度，防止栈溢出
        if depth > 100 {
            return Err(ValidationError::new(
                "表达式嵌套层级过深".to_string(),
                ValidationErrorType::ExpressionDepthError,
            ));
        }

        match expression {
            crate::core::types::expression::Expression::Binary { op, left, right } => {
                // 验证二元操作符
                self.validate_binary_operation(op, left, right, depth)?;
            }
            crate::core::types::expression::Expression::Unary { op, operand } => {
                // 验证一元操作符
                self.validate_unary_operation(op, operand, depth)?;
            }
            crate::core::types::expression::Expression::Function { name, args } => {
                // 验证函数调用
                self.validate_function_call(name, args, depth)?;
            }
            crate::core::types::expression::Expression::Aggregate { func, arg, distinct } => {
                // 验证聚合函数
                self.validate_aggregate_operation(func, arg, *distinct, depth)?;
            }
            crate::core::types::expression::Expression::Property { object: prop_expression, property: name } => {
                // 验证属性访问
                self.validate_property_access(prop_expression, name, depth)?;
            }
            crate::core::types::expression::Expression::Subscript { collection: index_expression, index } => {
                // 验证索引访问
                self.validate_index_access(index_expression, index, depth)?;
            }
            crate::core::types::expression::Expression::List(items) => {
                // 验证列表表达式
                self.validate_list_expression(items, depth)?;
            }
            crate::core::types::expression::Expression::Map(pairs) => {
                // 验证映射表达式
                self.validate_map_expression(pairs, depth)?;
            }
            crate::core::types::expression::Expression::Case {
                test_expr,
                conditions: when_clauses,
                default: else_clause,
            } => {
                // 验证条件表达式
                self.validate_case_expression(&test_expr, when_clauses, else_clause, depth)?;
            }
            _ => {
                // 其他表达式类型无需特殊验证
            }
        }

        Ok(())
    }

    /// 验证二元操作
    fn validate_binary_operation(
        &self,
        op: &crate::core::BinaryOperator,
        left: &crate::core::types::expression::Expression,
        right: &crate::core::types::expression::Expression,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 递归验证左右操作数
        self.validate_expression_operations_recursive(left, depth + 1)?;
        self.validate_expression_operations_recursive(right, depth + 1)?;

        // 验证操作符的合法性
        match op {
            crate::core::BinaryOperator::Divide => {
                // 除法需要特殊检查：除数不能为常量0
                if let crate::core::types::expression::Expression::Literal(crate::core::Value::Int(0)) = right {
                    return Err(ValidationError::new(
                        "除数不能为0".to_string(),
                        ValidationErrorType::DivisionByZero,
                    ));
                }
                if let Expression::Literal(crate::core::Value::Float(0.0)) = right {
                    return Err(ValidationError::new(
                        "除数不能为0.0".to_string(),
                        ValidationErrorType::DivisionByZero,
                    ));
                }
            }
            crate::core::BinaryOperator::Modulo => {
                // 模运算需要特殊检查：模数不能为常量0
                if let Expression::Literal(crate::core::Value::Int(0)) = right {
                    return Err(ValidationError::new(
                        "模数不能为0".to_string(),
                        ValidationErrorType::DivisionByZero,
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// 验证一元操作
    fn validate_unary_operation(
        &self,
        _op: &crate::core::UnaryOperator,
        operand: &crate::core::types::expression::Expression,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 递归验证操作数
        self.validate_expression_operations_recursive(operand, depth + 1)
    }

    /// 验证函数调用
    fn validate_function_call(
        &self,
        name: &str,
        args: &[crate::core::types::expression::Expression],
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 验证函数名格式
        if name.is_empty() {
            return Err(ValidationError::new(
                "函数名不能为空".to_string(),
                ValidationErrorType::SyntaxError,
            ));
        }

        // 验证参数数量限制
        if args.len() > 100 {
            return Err(ValidationError::new(
                format!("函数 {:?} 的参数数量过多: {}", name, args.len()),
                ValidationErrorType::TooManyArguments,
            ));
        }

        // 递归验证每个参数
        for (i, arg) in args.iter().enumerate() {
            self.validate_expression_operations_recursive(arg, depth + 1)
                .map_err(|e| ValidationError::new(
                    format!("函数 {:?} 的第 {} 个参数验证失败: {}", name, i + 1, e.message),
                    e.error_type,
                ))?;
        }

        Ok(())
    }

    /// 验证聚合操作
    fn validate_aggregate_operation(
        &self,
        func: &crate::core::AggregateFunction,
        arg: &crate::core::types::expression::Expression,
        distinct: bool,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 递归验证聚合参数
        self.validate_expression_operations_recursive(arg, depth + 1)?;

        // 使用类型推导验证器验证聚合函数参数类型
        let type_validator = TypeDeduceValidator::new();
        let _ = type_validator.deduce_type(arg);

        // 验证 DISTINCT 标记
        if distinct {
            match func {
                crate::core::AggregateFunction::Count(_) | 
                crate::core::AggregateFunction::Sum(_) | 
                crate::core::AggregateFunction::Avg(_) => {
                    // 这些函数支持 DISTINCT
                }
                _ => {
                    return Err(ValidationError::new(
                        format!("聚合函数 {} 不支持 DISTINCT 关键字", func.name()),
                        ValidationErrorType::SyntaxError,
                    ));
                }
            }
        }

        Ok(())
    }

    /// 验证属性访问
    fn validate_property_access(
        &self,
        expression: &crate::core::types::expression::Expression,
        name: &str,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 验证属性名格式
        if name.is_empty() {
            return Err(ValidationError::new(
                "属性名不能为空".to_string(),
                ValidationErrorType::SyntaxError,
            ));
        }

        // 递归验证表达式
        self.validate_expression_operations_recursive(expression, depth + 1)
    }

    /// 验证索引访问
    fn validate_index_access(
        &self,
        expression: &crate::core::types::expression::Expression,
        index: &crate::core::types::expression::Expression,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 递归验证表达式和索引
        self.validate_expression_operations_recursive(expression, depth + 1)?;
        self.validate_expression_operations_recursive(index, depth + 1)?;

        // 使用类型推导验证器验证索引类型
        let type_validator = TypeDeduceValidator::new();
        let expr_type = type_validator.deduce_type(expression);
        let index_type = type_validator.deduce_type(index);

        match expr_type {
            DataType::List => {
                // 列表需要整数索引
                if index_type != DataType::Int && index_type != DataType::Empty {
                    return Err(ValidationError::new(
                        format!("列表下标需要整数类型，但得到: {:?}", index_type),
                        ValidationErrorType::TypeError,
                    ));
                }
            }
            DataType::Map => {
                // 映射需要字符串键
                if index_type != DataType::String && index_type != DataType::Empty {
                    return Err(ValidationError::new(
                        format!("映射键需要字符串类型，但得到: {:?}", index_type),
                        ValidationErrorType::TypeError,
                    ));
                }
            }
            DataType::Empty => {
                // 类型未知时跳过验证
            }
            _ => {
                return Err(ValidationError::new(
                    format!("下标操作不支持类型: {:?}", expr_type),
                    ValidationErrorType::TypeError,
                ));
            }
        }

        Ok(())
    }

    /// 验证列表表达式
    fn validate_list_expression(&self, items: &[crate::core::types::expression::Expression], depth: usize) -> Result<(), ValidationError> {
        // 验证列表大小限制
        if items.len() > 10000 {
            return Err(ValidationError::new(
                "列表表达式元素过多".to_string(),
                ValidationErrorType::TooManyElements,
            ));
        }

        // 递归验证每个元素
        for (i, item) in items.iter().enumerate() {
            self.validate_expression_operations_recursive(item, depth + 1)
                .map_err(|e| ValidationError::new(
                    format!("列表表达式第 {} 个元素验证失败: {}", i + 1, e.message),
                    e.error_type,
                ))?;
        }

        Ok(())
    }

    /// 验证映射表达式
    fn validate_map_expression(&self, pairs: &[(String, crate::core::types::expression::Expression)], depth: usize) -> Result<(), ValidationError> {
        // 验证映射大小限制
        if pairs.len() > 10000 {
            return Err(ValidationError::new(
                "映射表达式键值对过多".to_string(),
                ValidationErrorType::TooManyElements,
            ));
        }

        // 检查键的唯一性
        let mut keys = HashSet::new();
        for (key, _) in pairs {
            if !keys.insert(key) {
                return Err(ValidationError::new(
                    format!("映射表达式中存在重复的键: {:?}", key),
                    ValidationErrorType::DuplicateKey,
                ));
            }
        }

        // 递归验证每个值
        for (key, value) in pairs {
            self.validate_expression_operations_recursive(value, depth + 1)
                .map_err(|e| ValidationError::new(
                    format!("映射表达式键 {:?} 的值验证失败: {}", key, e.message),
                    e.error_type,
                ))?;
        }

        Ok(())
    }

    /// 验证条件表达式
    fn validate_case_expression(
        &self,
        operand: &Option<Box<crate::core::types::expression::Expression>>,
        when_clauses: &[(crate::core::types::expression::Expression, crate::core::types::expression::Expression)],
        else_clause: &Option<Box<crate::core::types::expression::Expression>>,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 验证 WHEN 子句数量
        if when_clauses.is_empty() {
            return Err(ValidationError::new(
                "CASE 表达式必须至少有一个 WHEN 子句".to_string(),
                ValidationErrorType::SyntaxError,
            ));
        }

        // 验证操作数（如果存在）
        if let Some(op) = operand {
            self.validate_expression_operations_recursive(op, depth + 1)?;
        }

        // 递归验证每个 WHEN 子句
        for (i, (when_expression, then_expression)) in when_clauses.iter().enumerate() {
            self.validate_expression_operations_recursive(when_expression, depth + 1)
                .map_err(|e| ValidationError::new(
                    format!("CASE 表达式第 {} 个 WHEN 子句验证失败: {}", i + 1, e.message),
                    e.error_type,
                ))?;
            self.validate_expression_operations_recursive(then_expression, depth + 1)
                .map_err(|e| ValidationError::new(
                    format!("CASE 表达式第 {} 个 THEN 子句验证失败: {}", i + 1, e.message),
                    e.error_type,
                ))?;
        }

        // 验证 ELSE 子句（如果存在）
        if let Some(else_expression) = else_clause {
            self.validate_expression_operations_recursive(else_expression, depth + 1)?;
        }

        Ok(())
    }

    /// 验证表达式循环依赖
    pub fn validate_expression_cycles(&self, expression: &ContextualExpression) -> Result<(), ValidationError> {
        if let Some(expr) = expression.expression() {
            let mut visited = HashSet::new();
            self.check_expression_cycles(&expr, &mut visited, 0)
        } else {
            Err(ValidationError::new(
                "表达式无效".to_string(),
                ValidationErrorType::ExpressionError,
            ))
        }
    }

    /// 检查表达式循环依赖
    fn check_expression_cycles(
        &self,
        expression: &crate::core::types::expression::Expression,
        visited: &mut HashSet<String>,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 防止无限递归
        if depth > 100 {
            return Err(ValidationError::new(
                "表达式循环依赖检测深度超限".to_string(),
                ValidationErrorType::ExpressionDepthError,
            ));
        }

        match expression {
            crate::core::types::expression::Expression::Variable(name) => {
                if visited.contains(name) {
                    return Err(ValidationError::new(
                        format!("检测到变量循环依赖: {:?}", name),
                        ValidationErrorType::CyclicReference,
                    ));
                }
                visited.insert(name.clone());
            }
            Expression::Binary { left, right, .. } => {
                self.check_expression_cycles(left, visited, depth + 1)?;
                self.check_expression_cycles(right, visited, depth + 1)?;
            }
            Expression::Unary { operand, .. } => {
                self.check_expression_cycles(operand, visited, depth + 1)?;
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.check_expression_cycles(arg, visited, depth + 1)?;
                }
            }
            Expression::Aggregate { arg, .. } => {
                self.check_expression_cycles(arg, visited, depth + 1)?;
            }
            _ => {}
        }

        Ok(())
    }

    /// 计算表达式深度
    pub fn calculate_expression_depth(&self, expression: &ContextualExpression) -> usize {
        if let Some(expr) = expression.expression() {
            self.calculate_expression_depth_internal(&expr)
        } else {
            0
        }
    }

    /// 内部方法：计算表达式深度
    fn calculate_expression_depth_internal(&self, expression: &crate::core::types::expression::Expression) -> usize {
        match expression {
            crate::core::types::expression::Expression::Literal(_) | crate::core::types::expression::Expression::Variable(_) => 1,
            crate::core::types::expression::Expression::Binary { left, right, .. } => {
                let left_depth = self.calculate_expression_depth_internal(left);
                let right_depth = self.calculate_expression_depth_internal(right);
                1 + left_depth.max(right_depth)
            }
            crate::core::types::expression::Expression::Unary { operand, .. } => 1 + self.calculate_expression_depth_internal(operand),
            crate::core::types::expression::Expression::Function { args, .. } => {
                let max_arg_depth = args.iter()
                    .map(|arg| self.calculate_expression_depth_internal(arg))
                    .max()
                    .unwrap_or(0);
                1 + max_arg_depth
            }
            crate::core::types::expression::Expression::Aggregate { arg, .. } => 1 + self.calculate_expression_depth_internal(arg),
            crate::core::types::expression::Expression::Property { object: prop_expression, .. } => 1 + self.calculate_expression_depth_internal(prop_expression),
            crate::core::types::expression::Expression::Subscript { collection: index_expression, index } => {
                let expr_depth = self.calculate_expression_depth_internal(index_expression);
                let index_depth = self.calculate_expression_depth_internal(index);
                1 + expr_depth.max(index_depth)
            }
            crate::core::types::expression::Expression::List(items) => {
                let max_item_depth = items.iter()
                    .map(|item| self.calculate_expression_depth_internal(item))
                    .max()
                    .unwrap_or(0);
                1 + max_item_depth
            }
            crate::core::types::expression::Expression::Map(pairs) => {
                let max_value_depth = pairs.iter()
                    .map(|(_, value)| self.calculate_expression_depth_internal(value))
                    .max()
                    .unwrap_or(0);
                1 + max_value_depth
            }
            crate::core::types::expression::Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                let mut depths = Vec::new();
                
                if let Some(test_expression) = test_expr {
                    depths.push(self.calculate_expression_depth_internal(test_expression));
                }
                
                for (when_expression, then_expression) in conditions {
                    depths.push(self.calculate_expression_depth_internal(when_expression));
                    depths.push(self.calculate_expression_depth_internal(then_expression));
                }
                
                if let Some(else_expression) = default {
                    depths.push(self.calculate_expression_depth_internal(else_expression));
                }
                
                let max_depth = depths.into_iter().max().unwrap_or(0);
                1 + max_depth
            }
            _ => 1,
        }
    }

    /// 使用 BFS 方式检查表达式深度
    /// 
    /// 类似于 nebula-graph 的 ExpressionUtils::checkExprDepth
    /// 使用广度优先遍历检查表达式深度，防止 OOM
    pub fn check_expression_depth_bfs(&self, expression: &ContextualExpression, max_depth: usize) -> Result<(), ValidationError> {
        if let Some(expr) = expression.expression() {
            self.check_expression_depth_bfs_internal(&expr, max_depth)
        } else {
            Err(ValidationError::new(
                "表达式无效".to_string(),
                ValidationErrorType::ExpressionError,
            ))
        }
    }

    /// 内部方法：使用 BFS 方式检查表达式深度
    fn check_expression_depth_bfs_internal(&self, expression: &crate::core::types::expression::Expression, max_depth: usize) -> Result<(), ValidationError> {
        use std::collections::VecDeque;
        
        let mut queue = VecDeque::new();
        queue.push_back((expression, 0usize));
        
        while let Some((expr, depth)) = queue.pop_front() {
            if depth > max_depth {
                return Err(ValidationError::new(
                    format!("表达式嵌套层级过深，最大允许深度为: {}", max_depth),
                    ValidationErrorType::ExpressionDepthError,
                ));
            }
            
            for child in expr.children() {
                queue.push_back((child, depth + 1));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Expression, Value};

    #[test]
    fn test_expression_operations_validator_creation() {
        let _validator = ExpressionOperationsValidator::new();
        assert!(true);
    }

    #[test]
    fn test_validate_expression_operations() {
        let validator = ExpressionOperationsValidator::new();
        
        // 简单的字面量表达式
        let literal_expression = Expression::Literal(Value::Int(42));
        assert!(validator.validate_expression_operations(&literal_expression).is_ok());
        
        // 简单的二元表达式
        let binary_expression = Expression::Binary {
            op: crate::core::BinaryOperator::Add,
            left: Box::new(Expression::Literal(Value::Int(1))),
            right: Box::new(Expression::Literal(Value::Int(2))),
        };
        assert!(validator.validate_expression_operations(&binary_expression).is_ok());
        
        // 除零检测
        let divide_by_zero = Expression::Binary {
            op: crate::core::BinaryOperator::Divide,
            left: Box::new(Expression::Literal(Value::Int(10))),
            right: Box::new(Expression::Literal(Value::Int(0))),
        };
        assert!(validator.validate_expression_operations(&divide_by_zero).is_err());
    }

    #[test]
    fn test_validate_function_call() {
        let validator = ExpressionOperationsValidator::new();
        
        // 有效的函数调用
        let valid_function = Expression::Function {
            name: "length".to_string(),
            args: vec![Expression::Literal(Value::String("test".to_string()))],
        };
        assert!(validator.validate_expression_operations(&valid_function).is_ok());
        
        // 空函数名
        let empty_function_name = Expression::Function {
            name: "".to_string(),
            args: vec![Expression::Literal(Value::Int(1))],
        };
        assert!(validator.validate_expression_operations(&empty_function_name).is_err());
    }

    #[test]
    fn test_calculate_expression_depth() {
        let validator = ExpressionOperationsValidator::new();
        
        // 简单表达式
        let literal_expression = Expression::Literal(Value::Int(42));
        assert_eq!(validator.calculate_expression_depth(&literal_expression), 1);
        
        // 嵌套表达式
        let nested_expression = Expression::Binary {
            op: crate::core::BinaryOperator::Add,
            left: Box::new(Expression::Literal(Value::Int(1))),
            right: Box::new(Expression::Binary {
                op: crate::core::BinaryOperator::Multiply,
                left: Box::new(Expression::Literal(Value::Int(2))),
                right: Box::new(Expression::Literal(Value::Int(3))),
            }),
        };
        assert_eq!(validator.calculate_expression_depth(&nested_expression), 3);
    }
}