use crate::core::{ExpressionError, Value};
use crate::core::expressions::ExpressionContext;
use crate::core::Expression;
use crate::core::types::operators::UnaryOperator;

/// 评估一元操作表达式
///
/// 现在直接使用Core层的操作符和求值器
pub fn evaluate_unary_op(
    op: &UnaryOperator,
    operand: &Expression,
    context: &dyn ExpressionContext,
) -> Result<Value, ExpressionError> {
    let evaluator = crate::core::evaluator::ExpressionEvaluator;
    let operand_val = evaluator.evaluate(operand, context)?;

    // 直接委托给Core求值器
    evaluator.eval_unary_operation(op, &operand_val)
}

// 保留一些向后兼容的工具函数

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

/// 创建一元操作表达式（便捷函数）
pub fn unary_expr(op: UnaryOperator, operand: Expression) -> Expression {
    Expression::Unary {
        op,
        operand: Box::new(operand),
    }
}

/// 创建算术操作表达式
pub fn plus(operand: Expression) -> Expression {
    unary_expr(UnaryOperator::Plus, operand)
}

pub fn minus(operand: Expression) -> Expression {
    unary_expr(UnaryOperator::Minus, operand)
}

pub fn increment(operand: Expression) -> Expression {
    unary_expr(UnaryOperator::Increment, operand)
}

pub fn decrement(operand: Expression) -> Expression {
    unary_expr(UnaryOperator::Decrement, operand)
}

/// 创建逻辑操作表达式
pub fn not(operand: Expression) -> Expression {
    unary_expr(UnaryOperator::Not, operand)
}

/// 创建存在性检查表达式
pub fn is_null(operand: Expression) -> Expression {
    unary_expr(UnaryOperator::IsNull, operand)
}

pub fn is_not_null(operand: Expression) -> Expression {
    unary_expr(UnaryOperator::IsNotNull, operand)
}

pub fn is_empty(operand: Expression) -> Expression {
    unary_expr(UnaryOperator::IsEmpty, operand)
}

pub fn is_not_empty(operand: Expression) -> Expression {
    unary_expr(UnaryOperator::IsNotEmpty, operand)
}

// Legacy类型已移除 - 现在直接使用Core层的UnaryOperator
// 所有操作符都在Core层定义，无需转换

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::Expression;
    use crate::core::expressions::default_context::DefaultExpressionContext;

    #[test]
    fn test_unified_unary_operator() {
        let context = DefaultExpressionContext::new();
        let operand = Expression::int(5);
        
        // 测试Core操作符（现在直接使用）
        let result = evaluate_unary_op(
            &UnaryOperator::Minus,
            &operand,
            &context,
        ).unwrap();
        
        assert_eq!(result, Value::Int(-5));
        
        // 测试逻辑操作符
        let result = evaluate_unary_op(
            &UnaryOperator::Not,
            &Expression::bool(true),
            &context,
        ).unwrap();
        
        assert_eq!(result, Value::Bool(false));
    }
}