use crate::core::{Value};
use crate::core::{Expression, ExpressionError};
use crate::core::expressions::ExpressionContext;
use crate::core::types::operators::BinaryOperator;

/// 评估二元操作表达式
///
/// 现在直接使用Core层的操作符和求值器
pub fn evaluate_binary_op<C: ExpressionContext>(
    left: &Expression,
    op: &BinaryOperator,
    right: &Expression,
    context: &mut C,
) -> Result<Value, ExpressionError> {
    let evaluator = crate::core::evaluator::ExpressionEvaluator;
    let left_val = evaluator.evaluate(left, context)?;
    let right_val = evaluator.evaluate(right, context)?;

    // 直接委托给Core求值器
    evaluator.eval_binary_operation(&left_val, op, &right_val)
}

// 保留一些向后兼容的工具函数，但委托给Core层

/// 将值转换为布尔值
pub fn value_to_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Float(f) => *f != 0.0 && !f.is_nan(),
        Value::String(s) => !s.is_empty(),
        Value::Null(_) => false,
        Value::Empty => false,
        _ => true, // Default to true for other types
    }
}

/// 创建二元操作表达式（便捷函数）
pub fn binary_expr(left: Expression, op: BinaryOperator, right: Expression) -> Expression {
    Expression::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    }
}

/// 创建算术操作表达式
pub fn add(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::Add, right)
}

pub fn subtract(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::Subtract, right)
}

pub fn multiply(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::Multiply, right)
}

pub fn divide(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::Divide, right)
}

/// 创建比较操作表达式
pub fn equal(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::Equal, right)
}

pub fn not_equal(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::NotEqual, right)
}

pub fn less_than(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::LessThan, right)
}

pub fn greater_than(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::GreaterThan, right)
}

/// 创建逻辑操作表达式
pub fn and(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::And, right)
}

pub fn or(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::Or, right)
}

pub fn xor(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::Xor, right)
}

/// 创建字符串操作表达式
pub fn string_concat(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::StringConcat, right)
}

pub fn contains(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::Contains, right)
}

pub fn starts_with(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::StartsWith, right)
}

pub fn ends_with(left: Expression, right: Expression) -> Expression {
    binary_expr(left, BinaryOperator::EndsWith, right)
}

// Legacy类型已移除 - 现在直接使用Core层的BinaryOperator
// 所有操作符都在Core层定义，无需转换

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::Expression;
    use crate::core::expressions::default_context::DefaultExpressionContext;

    #[test]
    fn test_unified_binary_operator() {
        let context = DefaultExpressionContext::new();
        let left = Expression::int(5);
        let right = Expression::int(3);
        
        // 测试Core操作符（现在直接使用）
        let result = evaluate_binary_op(
            &left,
            &BinaryOperator::Add,
            &right,
            &context,
        ).unwrap();
        
        assert_eq!(result, Value::Int(8));
        
        // 测试扩展操作符（现在也在Core层）
        let result = evaluate_binary_op(
            &Expression::bool(true),
            &BinaryOperator::Xor,
            &Expression::bool(false),
            &context,
        ).unwrap();
        
        assert_eq!(result, Value::Bool(true)); // true XOR false = true
    }
}