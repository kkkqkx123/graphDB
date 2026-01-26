//! Expression utilities
//!
//! This module provides utilities for working with expressions.

use crate::core::types::expression::{Expression, ExpressionMeta};

/// 从字符串解析表达式元数据
///
/// 此函数直接使用 Parser 解析表达式，返回 Core Expression 元数据。
/// 不再需要 AST 到 Core 的转换层。
///
/// # 参数
///
/// * `condition` - 包含表达式的字符串
///
/// # 返回值
///
/// 成功时返回 ExpressionMeta
/// 失败时返回错误信息字符串
///
/// # 示例
///
/// ```rust
/// let result = parse_expression_meta_from_string("n.age > 25");
/// assert!(result.is_ok());
/// let meta = result.unwrap();
/// assert!(meta.span().is_some());
/// ```
pub fn parse_expression_meta_from_string(condition: &str) -> Result<ExpressionMeta, String> {
    let mut parser = crate::query::parser::Parser::new(condition);
    let core_expression = parser
        .parse_expression()
        .map_err(|e| format!("语法分析错误: {:?}", e))?;
    Ok(ExpressionMeta::new(core_expression))
}
