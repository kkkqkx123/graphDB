/// 操作符转换模块
/// 提供各种操作符之间的转换功能

use crate::core::types::operators::BinaryOperator as CoreBinaryOperator;
use crate::core::types::operators::UnaryOperator as CoreUnaryOperator;
use crate::core::types::operators::AggregateFunction as CoreAggregateFunction;
use crate::expression::operators_ext::{ExtendedBinaryOperator, ExtendedUnaryOperator, ExtendedAggregateFunction};
use crate::expression::operators_ext::conversion as ext_conversion;

/// 将Core二元操作符转换为扩展二元操作符
pub fn convert_core_binary_operator(
    op: &CoreBinaryOperator,
) -> ExtendedBinaryOperator {
    ExtendedBinaryOperator::Core(op.clone())
}

/// 将扩展二元操作符转换为Core二元操作符（如果可能）
pub fn convert_extended_binary_operator(
    op: &ExtendedBinaryOperator,
) -> Option<CoreBinaryOperator> {
    ext_conversion::to_core_binary_operator(op)
}

/// 将Core一元操作符转换为扩展一元操作符
pub fn convert_core_unary_operator(
    op: &CoreUnaryOperator,
) -> ExtendedUnaryOperator {
    ExtendedUnaryOperator::Core(op.clone())
}

/// 将扩展一元操作符转换为Core一元操作符（如果可能）
pub fn convert_extended_unary_operator(
    op: &ExtendedUnaryOperator,
) -> Option<CoreUnaryOperator> {
    match op {
        ExtendedUnaryOperator::Core(core_op) => Some(core_op.clone()),
    }
}

/// 将Core聚合函数转换为扩展聚合函数
pub fn convert_core_aggregate_function(
    op: &CoreAggregateFunction,
) -> ExtendedAggregateFunction {
    ExtendedAggregateFunction::Core(op.clone())
}

/// 将扩展聚合函数转换为Core聚合函数（如果可能）
pub fn convert_extended_aggregate_function(
    op: &ExtendedAggregateFunction,
) -> Option<CoreAggregateFunction> {
    match op {
        ExtendedAggregateFunction::Core(core_func) => Some(core_func.clone()),
    }
}

/// 转换Cypher二元操作符为扩展二元操作符
pub fn convert_cypher_binary_operator(
    cypher_op: &crate::query::parser::cypher::ast::expressions::BinaryOperator,
) -> ExtendedBinaryOperator {
    use crate::core::types::operators::BinaryOperator as CoreBinOp;
    use crate::query::parser::cypher::ast::expressions::BinaryOperator as CypherOp;

    let core_op = match cypher_op {
        CypherOp::Add => CoreBinOp::Add,
        CypherOp::Subtract => CoreBinOp::Subtract,
        CypherOp::Multiply => CoreBinOp::Multiply,
        CypherOp::Divide => CoreBinOp::Divide,
        CypherOp::Modulo => CoreBinOp::Modulo,
        CypherOp::Exponent => CoreBinOp::Multiply, // 临时映射
        CypherOp::And => CoreBinOp::And,
        CypherOp::Or => CoreBinOp::Or,
        CypherOp::Xor => {
            // Xor是扩展操作符
            return ExtendedBinaryOperator::Xor;
        }
        CypherOp::Equal => CoreBinOp::Equal,
        CypherOp::NotEqual => CoreBinOp::NotEqual,
        CypherOp::LessThan => CoreBinOp::LessThan,
        CypherOp::LessThanOrEqual => CoreBinOp::LessThanOrEqual,
        CypherOp::GreaterThan => CoreBinOp::GreaterThan,
        CypherOp::GreaterThanOrEqual => CoreBinOp::GreaterThanOrEqual,
        CypherOp::In => CoreBinOp::In,
        CypherOp::StartsWith => CoreBinOp::Like,
        CypherOp::EndsWith => CoreBinOp::Like,
        CypherOp::Contains => {
            // Contains是扩展操作符
            return ExtendedBinaryOperator::Contains;
        }
        CypherOp::RegexMatch => CoreBinOp::Like,
    };
    
    ExtendedBinaryOperator::Core(core_op)
}

/// 转换Cypher一元操作符为扩展一元操作符
pub fn convert_cypher_unary_operator(
    cypher_op: &crate::query::parser::cypher::ast::expressions::UnaryOperator,
) -> ExtendedUnaryOperator {
    use crate::core::types::operators::UnaryOperator as CoreUnaryOp;
    use crate::query::parser::cypher::ast::expressions::UnaryOperator as CypherOp;

    let core_op = match cypher_op {
        CypherOp::Not => CoreUnaryOp::Not,
        CypherOp::Positive => CoreUnaryOp::Plus,
        CypherOp::Negative => CoreUnaryOp::Minus,
    };
    
    ExtendedUnaryOperator::Core(core_op)
}

/// 将扩展二元操作符转换为Cypher二元操作符
pub fn convert_extended_to_cypher_binary_operator(
    op: &ExtendedBinaryOperator,
) -> Result<crate::query::parser::cypher::ast::expressions::BinaryOperator, String> {
    use crate::core::types::operators::BinaryOperator as CoreBinOp;
    use crate::query::parser::cypher::ast::expressions::BinaryOperator as CypherOp;

    match op {
        ExtendedBinaryOperator::Core(core_op) => {
            match core_op {
                CoreBinOp::Add => Ok(CypherOp::Add),
                CoreBinOp::Subtract => Ok(CypherOp::Subtract),
                CoreBinOp::Multiply => Ok(CypherOp::Multiply),
                CoreBinOp::Divide => Ok(CypherOp::Divide),
                CoreBinOp::Modulo => Ok(CypherOp::Modulo),
                CoreBinOp::And => Ok(CypherOp::And),
                CoreBinOp::Or => Ok(CypherOp::Or),
                CoreBinOp::Equal => Ok(CypherOp::Equal),
                CoreBinOp::NotEqual => Ok(CypherOp::NotEqual),
                CoreBinOp::LessThan => Ok(CypherOp::LessThan),
                CoreBinOp::LessThanOrEqual => Ok(CypherOp::LessThanOrEqual),
                CoreBinOp::GreaterThan => Ok(CypherOp::GreaterThan),
                CoreBinOp::GreaterThanOrEqual => Ok(CypherOp::GreaterThanOrEqual),
                CoreBinOp::In => Ok(CypherOp::In),
                CoreBinOp::StringConcat => Ok(CypherOp::Add), // 临时映射
                CoreBinOp::Like => Ok(CypherOp::RegexMatch),  // 临时映射
                CoreBinOp::Union => Ok(CypherOp::Add),        // 临时映射
                CoreBinOp::Intersect => Ok(CypherOp::And),    // 临时映射
                CoreBinOp::Except => Ok(CypherOp::Subtract),  // 临时映射
            }
        }
        ExtendedBinaryOperator::Xor => Ok(CypherOp::Xor),
        ExtendedBinaryOperator::NotIn => Err("NOT IN not directly supported in Cypher".to_string()),
        ExtendedBinaryOperator::Subscript => Err("Subscript not directly supported in Cypher".to_string()),
        ExtendedBinaryOperator::Attribute => Err("Attribute access not directly supported in Cypher".to_string()),
        ExtendedBinaryOperator::Contains => Ok(CypherOp::Contains),
        ExtendedBinaryOperator::StartsWith => Ok(CypherOp::StartsWith),
        ExtendedBinaryOperator::EndsWith => Ok(CypherOp::EndsWith),
    }
}

/// 将扩展一元操作符转换为Cypher一元操作符
pub fn convert_extended_to_cypher_unary_operator(
    op: &ExtendedUnaryOperator,
) -> Result<crate::query::parser::cypher::ast::expressions::UnaryOperator, String> {
    use crate::core::types::operators::UnaryOperator as CoreUnaryOp;
    use crate::query::parser::cypher::ast::expressions::UnaryOperator as CypherOp;

    match op {
        ExtendedUnaryOperator::Core(core_op) => {
            match core_op {
                CoreUnaryOp::Plus => Ok(CypherOp::Positive),
                CoreUnaryOp::Minus => Ok(CypherOp::Negative),
                CoreUnaryOp::Not => Ok(CypherOp::Not),
                CoreUnaryOp::IsNull => Ok(CypherOp::Positive), // 临时映射
                CoreUnaryOp::IsNotNull => Ok(CypherOp::Positive), // 临时映射
                CoreUnaryOp::IsEmpty => Ok(CypherOp::Positive), // 临时映射
                CoreUnaryOp::IsNotEmpty => Ok(CypherOp::Positive), // 临时映射
                CoreUnaryOp::Increment => Ok(CypherOp::Positive), // 临时映射
                CoreUnaryOp::Decrement => Ok(CypherOp::Positive), // 临时映射
            }
        }
    }
}

/// 为了向后兼容，保留旧的转换函数
#[deprecated(note = "使用新的扩展操作符转换函数")]
pub fn convert_binary_operator(
    op: &crate::core::types::expression::BinaryOperator,
) -> super::binary::LegacyBinaryOperator {
    use super::binary::LegacyBinaryOperator as BinOp;
    use crate::core::types::expression::BinaryOperator as ExprBinOp;

    match op {
        ExprBinOp::Add => BinOp::Add,
        ExprBinOp::Subtract => BinOp::Sub,
        ExprBinOp::Multiply => BinOp::Mul,
        ExprBinOp::Divide => BinOp::Div,
        ExprBinOp::Modulo => BinOp::Mod,
        ExprBinOp::Equal => BinOp::Eq,
        ExprBinOp::NotEqual => BinOp::Ne,
        ExprBinOp::LessThan => BinOp::Lt,
        ExprBinOp::LessThanOrEqual => BinOp::Le,
        ExprBinOp::GreaterThan => BinOp::Gt,
        ExprBinOp::GreaterThanOrEqual => BinOp::Ge,
        ExprBinOp::And => BinOp::And,
        ExprBinOp::Or => BinOp::Or,
        ExprBinOp::StringConcat => BinOp::Attribute,
        ExprBinOp::Like => BinOp::StartsWith,
        ExprBinOp::In => BinOp::In,
        ExprBinOp::Union => BinOp::Add,
        ExprBinOp::Intersect => BinOp::And,
        ExprBinOp::Except => BinOp::Sub,
    }
}

#[deprecated(note = "使用新的扩展操作符转换函数")]
pub fn convert_unary_operator(
    op: &crate::core::types::expression::UnaryOperator,
) -> super::unary::LegacyUnaryOperator {
    use super::unary::LegacyUnaryOperator as UnaryOp;
    use crate::core::types::expression::UnaryOperator as ExprUnaryOp;

    match op {
        ExprUnaryOp::Plus => UnaryOp::Plus,
        ExprUnaryOp::Minus => UnaryOp::Minus,
        ExprUnaryOp::Not => UnaryOp::Not,
        ExprUnaryOp::IsNull => UnaryOp::IsNull,
        ExprUnaryOp::IsNotNull => UnaryOp::IsNotNull,
        ExprUnaryOp::IsEmpty => UnaryOp::IsEmpty,
        ExprUnaryOp::IsNotEmpty => UnaryOp::IsNotEmpty,
        ExprUnaryOp::Increment => UnaryOp::Increment,
        ExprUnaryOp::Decrement => UnaryOp::Decrement,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator as CoreBinOp;

    #[test]
    fn test_operator_conversion() {
        // 测试Core到扩展的转换
        let core_op = CoreBinOp::Add;
        let ext_op = convert_core_binary_operator(&core_op);
        assert!(matches!(ext_op, ExtendedBinaryOperator::Core(CoreBinOp::Add)));
        
        // 测试扩展到Core的转换
        let back_to_core = convert_extended_binary_operator(&ext_op);
        assert!(back_to_core.is_some());
        assert_eq!(back_to_core.unwrap(), CoreBinOp::Add);
        
        // 测试扩展操作符
        let xor_op = ExtendedBinaryOperator::Xor;
        let xor_to_core = convert_extended_binary_operator(&xor_op);
        assert!(xor_to_core.is_none());
    }
}