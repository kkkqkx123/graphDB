/// 操作符转换模块
/// 提供各种操作符之间的转换功能

/// 将 expression::BinaryOperator 转换为 binary::BinaryOperator
pub fn convert_binary_operator(
    op: &crate::expression::expression::BinaryOperator,
) -> super::binary::BinaryOperator {
    use super::binary::BinaryOperator as BinOp;
    use crate::expression::expression::BinaryOperator as ExprBinOp;

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

/// 将 expression::UnaryOperator 转换为 unary::UnaryOperator
pub fn convert_unary_operator(
    op: &crate::expression::expression::UnaryOperator,
) -> super::unary::UnaryOperator {
    use super::unary::UnaryOperator as UnaryOp;
    use crate::expression::expression::UnaryOperator as ExprUnaryOp;

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

/// 转换Cypher二元操作符
pub fn convert_cypher_binary_operator(
    cypher_op: &crate::query::parser::cypher::ast::expressions::BinaryOperator,
) -> crate::expression::expression::BinaryOperator {
    use crate::expression::expression::BinaryOperator as GraphOp;
    use crate::query::parser::cypher::ast::expressions::BinaryOperator as CypherOp;

    match cypher_op {
        CypherOp::Add => GraphOp::Add,
        CypherOp::Subtract => GraphOp::Subtract,
        CypherOp::Multiply => GraphOp::Multiply,
        CypherOp::Divide => GraphOp::Divide,
        CypherOp::Modulo => GraphOp::Modulo,
        CypherOp::Exponent => GraphOp::Multiply, // 临时映射
        CypherOp::And => GraphOp::And,
        CypherOp::Or => GraphOp::Or,
        CypherOp::Xor => GraphOp::And, // 临时映射
        CypherOp::Equal => GraphOp::Equal,
        CypherOp::NotEqual => GraphOp::NotEqual,
        CypherOp::LessThan => GraphOp::LessThan,
        CypherOp::LessThanOrEqual => GraphOp::LessThanOrEqual,
        CypherOp::GreaterThan => GraphOp::GreaterThan,
        CypherOp::GreaterThanOrEqual => GraphOp::GreaterThanOrEqual,
        CypherOp::In => GraphOp::In,
        CypherOp::StartsWith => GraphOp::Like,
        CypherOp::EndsWith => GraphOp::Like,
        CypherOp::Contains => GraphOp::Like,
        CypherOp::RegexMatch => GraphOp::Like,
    }
}

/// 转换Cypher一元操作符
pub fn convert_cypher_unary_operator(
    cypher_op: &crate::query::parser::cypher::ast::expressions::UnaryOperator,
) -> crate::expression::expression::UnaryOperator {
    use crate::expression::expression::UnaryOperator as GraphOp;
    use crate::query::parser::cypher::ast::expressions::UnaryOperator as CypherOp;

    match cypher_op {
        CypherOp::Not => GraphOp::Not,
        CypherOp::Positive => GraphOp::Plus,
        CypherOp::Negative => GraphOp::Minus,
    }
}

/// 将统一表达式二元操作符转换为Cypher二元操作符
pub fn convert_unified_to_cypher_binary_operator(
    op: &crate::expression::expression::BinaryOperator,
) -> Result<crate::query::parser::cypher::ast::expressions::BinaryOperator, String> {
    use crate::expression::expression::BinaryOperator as GraphOp;
    use crate::query::parser::cypher::ast::expressions::BinaryOperator as CypherOp;

    match op {
        GraphOp::Add => Ok(CypherOp::Add),
        GraphOp::Subtract => Ok(CypherOp::Subtract),
        GraphOp::Multiply => Ok(CypherOp::Multiply),
        GraphOp::Divide => Ok(CypherOp::Divide),
        GraphOp::Modulo => Ok(CypherOp::Modulo),
        GraphOp::And => Ok(CypherOp::And),
        GraphOp::Or => Ok(CypherOp::Or),
        GraphOp::Equal => Ok(CypherOp::Equal),
        GraphOp::NotEqual => Ok(CypherOp::NotEqual),
        GraphOp::LessThan => Ok(CypherOp::LessThan),
        GraphOp::LessThanOrEqual => Ok(CypherOp::LessThanOrEqual),
        GraphOp::GreaterThan => Ok(CypherOp::GreaterThan),
        GraphOp::GreaterThanOrEqual => Ok(CypherOp::GreaterThanOrEqual),
        GraphOp::In => Ok(CypherOp::In),
        GraphOp::StringConcat => Ok(CypherOp::Add), // 临时映射
        GraphOp::Like => Ok(CypherOp::RegexMatch),  // 临时映射
        GraphOp::Union => Ok(CypherOp::Add),        // 临时映射
        GraphOp::Intersect => Ok(CypherOp::And),    // 临时映射
        GraphOp::Except => Ok(CypherOp::Subtract),  // 临时映射
    }
}

/// 将统一表达式一元操作符转换为Cypher一元操作符
pub fn convert_unified_to_cypher_unary_operator(
    op: &crate::expression::expression::UnaryOperator,
) -> Result<crate::query::parser::cypher::ast::expressions::UnaryOperator, String> {
    use crate::expression::expression::UnaryOperator as GraphOp;
    use crate::query::parser::cypher::ast::expressions::UnaryOperator as CypherOp;

    match op {
        GraphOp::Plus => Ok(CypherOp::Positive),
        GraphOp::Minus => Ok(CypherOp::Negative),
        GraphOp::Not => Ok(CypherOp::Not),
        GraphOp::IsNull => Ok(CypherOp::Positive), // 临时映射
        GraphOp::IsNotNull => Ok(CypherOp::Positive), // 临时映射
        GraphOp::IsEmpty => Ok(CypherOp::Positive), // 临时映射
        GraphOp::IsNotEmpty => Ok(CypherOp::Positive), // 临时映射
        GraphOp::Increment => Ok(CypherOp::Positive), // 临时映射
        GraphOp::Decrement => Ok(CypherOp::Positive), // 临时映射
    }
}
