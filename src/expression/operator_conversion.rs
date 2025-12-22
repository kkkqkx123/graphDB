//! 操作符转换模块
//! 
//! 现在直接使用Core操作符，无需转换

use crate::core::types::operators::{
    BinaryOperator, UnaryOperator, AggregateFunction
};

/// 转换Cypher二元操作符为Core二元操作符
pub fn convert_cypher_binary_operator(
    cypher_op: &crate::query::parser::cypher::ast::expressions::BinaryOperator,
) -> BinaryOperator {
    use crate::query::parser::cypher::ast::expressions::BinaryOperator as CypherOp;

    match cypher_op {
        CypherOp::Add => BinaryOperator::Add,
        CypherOp::Subtract => BinaryOperator::Subtract,
        CypherOp::Multiply => BinaryOperator::Multiply,
        CypherOp::Divide => BinaryOperator::Divide,
        CypherOp::Modulo => BinaryOperator::Modulo,
        CypherOp::Exponent => BinaryOperator::Multiply, // 临时映射
        CypherOp::And => BinaryOperator::And,
        CypherOp::Or => BinaryOperator::Or,
        CypherOp::Xor => BinaryOperator::Xor,
        CypherOp::Equal => BinaryOperator::Equal,
        CypherOp::NotEqual => BinaryOperator::NotEqual,
        CypherOp::LessThan => BinaryOperator::LessThan,
        CypherOp::LessThanOrEqual => BinaryOperator::LessThanOrEqual,
        CypherOp::GreaterThan => BinaryOperator::GreaterThan,
        CypherOp::GreaterThanOrEqual => BinaryOperator::GreaterThanOrEqual,
        CypherOp::In => BinaryOperator::In,
        CypherOp::StartsWith => BinaryOperator::StartsWith,
        CypherOp::EndsWith => BinaryOperator::EndsWith,
        CypherOp::Contains => BinaryOperator::Contains,
        CypherOp::RegexMatch => BinaryOperator::Like,
    }
}

/// 转换Cypher一元操作符为Core一元操作符
pub fn convert_cypher_unary_operator(
    cypher_op: &crate::query::parser::cypher::ast::expressions::UnaryOperator,
) -> UnaryOperator {
    use crate::query::parser::cypher::ast::expressions::UnaryOperator as CypherOp;

    match cypher_op {
        CypherOp::Not => UnaryOperator::Not,
        CypherOp::Positive => UnaryOperator::Plus,
        CypherOp::Negative => UnaryOperator::Minus,
    }
}

/// 将Core二元操作符转换为Cypher二元操作符
pub fn convert_core_to_cypher_binary_operator(
    op: &BinaryOperator,
) -> Result<crate::query::parser::cypher::ast::expressions::BinaryOperator, String> {
    use crate::query::parser::cypher::ast::expressions::BinaryOperator as CypherOp;

    match op {
        BinaryOperator::Add => Ok(CypherOp::Add),
        BinaryOperator::Subtract => Ok(CypherOp::Subtract),
        BinaryOperator::Multiply => Ok(CypherOp::Multiply),
        BinaryOperator::Divide => Ok(CypherOp::Divide),
        BinaryOperator::Modulo => Ok(CypherOp::Modulo),
        BinaryOperator::And => Ok(CypherOp::And),
        BinaryOperator::Or => Ok(CypherOp::Or),
        BinaryOperator::Xor => Ok(CypherOp::Xor),
        BinaryOperator::Equal => Ok(CypherOp::Equal),
        BinaryOperator::NotEqual => Ok(CypherOp::NotEqual),
        BinaryOperator::LessThan => Ok(CypherOp::LessThan),
        BinaryOperator::LessThanOrEqual => Ok(CypherOp::LessThanOrEqual),
        BinaryOperator::GreaterThan => Ok(CypherOp::GreaterThan),
        BinaryOperator::GreaterThanOrEqual => Ok(CypherOp::GreaterThanOrEqual),
        BinaryOperator::In => Ok(CypherOp::In),
        BinaryOperator::StringConcat => Ok(CypherOp::Add), // 临时映射
        BinaryOperator::Like => Ok(CypherOp::RegexMatch),  // 临时映射
        BinaryOperator::Union => Ok(CypherOp::Add),        // 临时映射
        BinaryOperator::Intersect => Ok(CypherOp::And),    // 临时映射
        BinaryOperator::Except => Ok(CypherOp::Subtract),  // 临时映射
        BinaryOperator::NotIn => Err("NOT IN not directly supported in Cypher".to_string()),
        BinaryOperator::Contains => Ok(CypherOp::Contains),
        BinaryOperator::StartsWith => Ok(CypherOp::StartsWith),
        BinaryOperator::EndsWith => Ok(CypherOp::EndsWith),
        BinaryOperator::Subscript => Err("Subscript not directly supported in Cypher".to_string()),
        BinaryOperator::Attribute => Err("Attribute access not directly supported in Cypher".to_string()),
    }
}

/// 将Core一元操作符转换为Cypher一元操作符
pub fn convert_core_to_cypher_unary_operator(
    op: &UnaryOperator,
) -> Result<crate::query::parser::cypher::ast::expressions::UnaryOperator, String> {
    use crate::query::parser::cypher::ast::expressions::UnaryOperator as CypherOp;

    match op {
        UnaryOperator::Plus => Ok(CypherOp::Positive),
        UnaryOperator::Minus => Ok(CypherOp::Negative),
        UnaryOperator::Not => Ok(CypherOp::Not),
        UnaryOperator::IsNull => Ok(CypherOp::Positive), // 临时映射
        UnaryOperator::IsNotNull => Ok(CypherOp::Positive), // 临时映射
        UnaryOperator::IsEmpty => Ok(CypherOp::Positive), // 临时映射
        UnaryOperator::IsNotEmpty => Ok(CypherOp::Positive), // 临时映射
        UnaryOperator::Increment => Ok(CypherOp::Positive), // 临时映射
        UnaryOperator::Decrement => Ok(CypherOp::Positive), // 临时映射
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_cypher_binary_operator_conversion() {
        use crate::query::parser::cypher::ast::expressions::BinaryOperator as CypherOp;
        
        // 测试Cypher到Core的转换
        let cypher_op = CypherOp::Add;
        let core_op = convert_cypher_binary_operator(&cypher_op);
        assert_eq!(core_op, BinaryOperator::Add);
        
        // 测试Core到Cypher的转换
        let back_to_cypher = convert_core_to_cypher_binary_operator(&core_op);
        assert!(back_to_cypher.is_ok());
        assert_eq!(back_to_cypher.unwrap(), CypherOp::Add);
        
        // 测试扩展操作符
        let xor_op = BinaryOperator::Xor;
        let xor_to_cypher = convert_core_to_cypher_binary_operator(&xor_op);
        assert!(xor_to_cypher.is_ok());
        assert_eq!(xor_to_cypher.unwrap(), CypherOp::Xor);
    }
}