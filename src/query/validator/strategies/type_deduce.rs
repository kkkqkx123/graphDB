//! 类型推导验证器
//!
//! 本模块实现表达式类型推导功能，类似于 nebula-graph 的 DeduceTypeVisitor。
//! 用于在验证阶段推导表达式的返回类型，并检查类型兼容性。

use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::types::expression::Expression;
use crate::core::types::DataType;

/// 推导表达式的数据类型
///
/// 这是 Expression::deduce_type() 的简单包装，用于类型推导验证。
///
/// # 示例
///
/// ```rust
/// use crate::query::validator::strategies::type_deduce::deduce_expression_type;
/// use crate::core::types::expression::Expression;
/// use crate::core::types::DataType;
///
/// let expr = Expression::add(Expression::int(1), Expression::int(2));
/// let data_type = deduce_expression_type(&expr);
/// assert_eq!(data_type, DataType::Int);
/// ```
pub fn deduce_expression_type(expression: &Expression) -> DataType {
    expression.deduce_type()
}

/// 类型推导验证器
///
/// 用于推导表达式的返回类型，并检查类型兼容性。
/// 现在直接委托给 Expression::deduce_type() 方法。
#[derive(Debug, Default)]
pub struct TypeDeduceValidator;

impl TypeDeduceValidator {
    /// 创建新的类型推导验证器
    pub fn new() -> Self {
        Self
    }

    /// 推导表达式的数据类型
    ///
    /// # 参数
    ///
    /// * `expression` - 要推导类型的表达式
    ///
    /// # 返回
    ///
    /// 返回推导出的数据类型。如果无法确定，返回 DataType::Empty。
    pub fn deduce_type(&self, expression: &ContextualExpression) -> DataType {
        if let Some(expr) = expression.expression() {
            expr.deduce_type()
        } else {
            DataType::Empty
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::Expression;
    use crate::core::types::DataType;

    #[test]
    fn test_deduce_literal_type() {
        let expr = Expression::int(42);
        let validator = TypeDeduceValidator::new();
        assert_eq!(validator.deduce_type(&expr), DataType::Int);
    }

    #[test]
    fn test_deduce_binary_type() {
        let expr = Expression::add(Expression::int(1), Expression::int(2));
        let validator = TypeDeduceValidator::new();
        assert_eq!(validator.deduce_type(&expr), DataType::Int);
    }

    #[test]
    fn test_deduce_variable_type() {
        let expr = Expression::variable("x");
        let validator = TypeDeduceValidator::new();
        assert_eq!(validator.deduce_type(&expr), DataType::Empty);
    }
}
