//! 表达式操作验证器
//! 负责验证表达式的操作合法性和结构完整性

use crate::core::Expression;
use crate::query::validator::{ValidationError, ValidationErrorType};
use std::collections::HashSet;

/// 表达式操作验证器
pub struct ExpressionOperationsValidator;

impl ExpressionOperationsValidator {
    pub fn new() -> Self {
        Self
    }

    /// 验证表达式操作的合法性
    pub fn validate_expression_operations(&self, expr: &Expression) -> Result<(), ValidationError> {
        self.validate_expression_operations_recursive(expr, 0)
    }

    /// 递归验证表达式操作
    fn validate_expression_operations_recursive(&self, expr: &Expression, depth: usize) -> Result<(), ValidationError> {
        // 检查表达式深度，防止栈溢出
        if depth > 100 {
            return Err(ValidationError::new(
                "表达式嵌套层级过深".to_string(),
                ValidationErrorType::ExpressionDepthError,
            ));
        }

        match expr {
            Expression::Binary { op, left, right } => {
                // 验证二元操作符
                self.validate_binary_operation(op, left, right, depth)?;
            }
            Expression::Unary { op, operand } => {
                // 验证一元操作符
                self.validate_unary_operation(op, operand, depth)?;
            }
            Expression::Function { name, args } => {
                // 验证函数调用
                self.validate_function_call(name, args, depth)?;
            }
            Expression::Aggregate { func, arg, distinct } => {
                // 验证聚合函数
                self.validate_aggregate_operation(func, arg, *distinct, depth)?;
            }
            Expression::Property { object: prop_expr, property: name } => {
                // 验证属性访问
                self.validate_property_access(prop_expr, name, depth)?;
            }
            Expression::Subscript { collection: index_expr, index } => {
                // 验证索引访问
                self.validate_index_access(index_expr, index, depth)?;
            }
            Expression::List(items) => {
                // 验证列表表达式
                self.validate_list_expression(items, depth)?;
            }
            Expression::Map(pairs) => {
                // 验证映射表达式
                self.validate_map_expression(pairs, depth)?;
            }
            Expression::Case {
                conditions: when_clauses,
                default: else_clause,
            } => {
                // 验证条件表达式
                self.validate_case_expression(&None, when_clauses, else_clause, depth)?;
            }
            Expression::Reduce {
                list,
                var,
                initial,
                expr,
            } => {
                // 验证归约表达式
                self.validate_reduce_expression(var, initial, list, expr, depth)?;
            }
            Expression::Predicate {
                list: pred_list,
                condition,
            } => {
                // 验证谓词表达式
                self.validate_expression_operations_recursive(pred_list, depth + 1)?;
                self.validate_expression_operations_recursive(condition, depth + 1)?;
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                // 验证列表推导表达式
                self.validate_expression_operations_recursive(generator, depth + 1)?;
                if let Some(cond) = condition {
                    self.validate_expression_operations_recursive(cond, depth + 1)?;
                }
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
        left: &Expression,
        right: &Expression,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 递归验证左右操作数
        self.validate_expression_operations_recursive(left, depth + 1)?;
        self.validate_expression_operations_recursive(right, depth + 1)?;

        // 验证操作符的合法性
        match op {
            crate::core::BinaryOperator::Divide => {
                // 除法需要特殊检查：除数不能为常量0
                if let Expression::Literal(crate::core::Value::Int(0)) = right {
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
        operand: &Expression,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 递归验证操作数
        self.validate_expression_operations_recursive(operand, depth + 1)
    }

    /// 验证函数调用
    fn validate_function_call(
        &self,
        name: &str,
        args: &[Expression],
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
        arg: &Expression,
        distinct: bool,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 递归验证聚合参数
        self.validate_expression_operations_recursive(arg, depth + 1)?;

        // 验证聚合函数参数
        match func {
            crate::core::AggregateFunction::Count(_) => {
                // COUNT 函数可以接受任意表达式
            }
            crate::core::AggregateFunction::Sum(_) | crate::core::AggregateFunction::Avg(_) => {
                // SUM 和 AVG 需要数值类型参数
                // 这里简化处理，实际应该验证类型
            }
            crate::core::AggregateFunction::Max(_) | crate::core::AggregateFunction::Min(_) => {
                // MAX 和 MIN 可以接受任意类型参数
            }
            crate::core::AggregateFunction::Collect(_) => {
                // COLLECT 可以接受任意类型参数
            }
            crate::core::AggregateFunction::Distinct(_) => {
                // DISTINCT 可以接受任意类型参数
            }
            crate::core::AggregateFunction::Percentile(_, _) => {
                // PERCENTILE 需要数值类型参数
                // 这里简化处理，实际应该验证类型
            }
        }

        // 验证 DISTINCT 标记
        if distinct {
            // 这里可以添加 DISTINCT 特定的验证逻辑
        }

        Ok(())
    }

    /// 验证属性访问
    fn validate_property_access(
        &self,
        expr: &Expression,
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
        self.validate_expression_operations_recursive(expr, depth + 1)
    }

    /// 验证索引访问
    fn validate_index_access(
        &self,
        expr: &Expression,
        index: &Expression,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 递归验证表达式和索引
        self.validate_expression_operations_recursive(expr, depth + 1)?;
        self.validate_expression_operations_recursive(index, depth + 1)?;

        // 验证索引类型（简化处理）
        match index {
            Expression::Literal(crate::core::Value::Int(_)) => {
                // 整数索引是合法的
            }
            Expression::Literal(crate::core::Value::String(_)) => {
                // 字符串索引也是合法的（用于映射）
            }
            _ => {
                // 其他类型的索引需要进一步验证
            }
        }

        Ok(())
    }

    /// 验证列表表达式
    fn validate_list_expression(&self, items: &[Expression], depth: usize) -> Result<(), ValidationError> {
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
    fn validate_map_expression(&self, pairs: &[(String, Expression)], depth: usize) -> Result<(), ValidationError> {
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
        operand: &Option<Box<Expression>>,
        when_clauses: &[(Expression, Expression)],
        else_clause: &Option<Box<Expression>>,
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
        for (i, (when_expr, then_expr)) in when_clauses.iter().enumerate() {
            self.validate_expression_operations_recursive(when_expr, depth + 1)
                .map_err(|e| ValidationError::new(
                    format!("CASE 表达式第 {} 个 WHEN 子句验证失败: {}", i + 1, e.message),
                    e.error_type,
                ))?;
            self.validate_expression_operations_recursive(then_expr, depth + 1)
                .map_err(|e| ValidationError::new(
                    format!("CASE 表达式第 {} 个 THEN 子句验证失败: {}", i + 1, e.message),
                    e.error_type,
                ))?;
        }

        // 验证 ELSE 子句（如果存在）
        if let Some(else_expr) = else_clause {
            self.validate_expression_operations_recursive(else_expr, depth + 1)?;
        }

        Ok(())
    }

    /// 验证模式理解表达式
    fn validate_pattern_comprehension(
        &self,
        _pattern: &Expression,
        predicate: &Option<&Expression>,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 验证谓词（如果存在）
        if let Some(pred) = predicate {
            self.validate_expression_operations_recursive(pred, depth + 1)?;
        }

        Ok(())
    }

    /// 验证归约表达式
    fn validate_reduce_expression(
        &self,
        _accumulator: &str,
        initial: &Expression,
        list: &Expression,
        _expression: &Expression,
        depth: usize,
    ) -> Result<(), ValidationError> {
        // 递归验证初始值和列表
        self.validate_expression_operations_recursive(initial, depth + 1)?;
        self.validate_expression_operations_recursive(list, depth + 1)?;

        Ok(())
    }

    /// 验证表达式循环依赖
    pub fn validate_expression_cycles(&self, expr: &Expression) -> Result<(), ValidationError> {
        let mut visited = HashSet::new();
        self.check_expression_cycles(expr, &mut visited, 0)
    }

    /// 检查表达式循环依赖
    fn check_expression_cycles(
        &self,
        expr: &Expression,
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

        // 这里简化处理，实际应该实现更复杂的循环检测
        // 例如检测变量之间的循环引用等

        match expr {
            Expression::Variable(name) => {
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
    pub fn calculate_expression_depth(&self, expr: &Expression) -> usize {
        match expr {
            Expression::Literal(_) | Expression::Variable(_) => 1,
            Expression::Binary { left, right, .. } => {
                let left_depth = self.calculate_expression_depth(left);
                let right_depth = self.calculate_expression_depth(right);
                1 + left_depth.max(right_depth)
            }
            Expression::Unary { operand, .. } => 1 + self.calculate_expression_depth(operand),
            Expression::Function { args, .. } => {
                let max_arg_depth = args.iter()
                    .map(|arg| self.calculate_expression_depth(arg))
                    .max()
                    .unwrap_or(0);
                1 + max_arg_depth
            }
            Expression::Aggregate { arg, .. } => 1 + self.calculate_expression_depth(arg),
            Expression::Property { object: prop_expr, .. } => 1 + self.calculate_expression_depth(prop_expr),
            Expression::Subscript { collection: index_expr, index } => {
                let expr_depth = self.calculate_expression_depth(index_expr);
                let index_depth = self.calculate_expression_depth(index);
                1 + expr_depth.max(index_depth)
            }
            Expression::List(items) => {
                let max_item_depth = items.iter()
                    .map(|item| self.calculate_expression_depth(item))
                    .max()
                    .unwrap_or(0);
                1 + max_item_depth
            }
            Expression::Map(pairs) => {
                let max_value_depth = pairs.iter()
                    .map(|(_, value)| self.calculate_expression_depth(value))
                    .max()
                    .unwrap_or(0);
                1 + max_value_depth
            }
            Expression::Case {
                conditions,
                default,
            } => {
                let mut depths = Vec::new();
                
                for (when_expr, then_expr) in conditions {
                    depths.push(self.calculate_expression_depth(when_expr));
                    depths.push(self.calculate_expression_depth(then_expr));
                }
                
                if let Some(else_expr) = default {
                    depths.push(self.calculate_expression_depth(else_expr));
                }
                
                let max_depth = depths.into_iter().max().unwrap_or(0);
                1 + max_depth
            }
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Expression, Value};

    #[test]
    fn test_expression_operations_validator_creation() {
        let validator = ExpressionOperationsValidator::new();
        assert!(true);
    }

    #[test]
    fn test_validate_expression_operations() {
        let validator = ExpressionOperationsValidator::new();
        
        // 简单的字面量表达式
        let literal_expr = Expression::Literal(Value::Int(42));
        assert!(validator.validate_expression_operations(&literal_expr).is_ok());
        
        // 简单的二元表达式
        let binary_expr = Expression::Binary {
            op: crate::core::BinaryOperator::Add,
            left: Box::new(Expression::Literal(Value::Int(1))),
            right: Box::new(Expression::Literal(Value::Int(2))),
        };
        assert!(validator.validate_expression_operations(&binary_expr).is_ok());
        
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
        let literal_expr = Expression::Literal(Value::Int(42));
        assert_eq!(validator.calculate_expression_depth(&literal_expr), 1);
        
        // 嵌套表达式
        let nested_expr = Expression::Binary {
            op: crate::core::BinaryOperator::Add,
            left: Box::new(Expression::Literal(Value::Int(1))),
            right: Box::new(Expression::Binary {
                op: crate::core::BinaryOperator::Multiply,
                left: Box::new(Expression::Literal(Value::Int(2))),
                right: Box::new(Expression::Literal(Value::Int(3))),
            }),
        };
        assert_eq!(validator.calculate_expression_depth(&nested_expr), 3);
    }
}