//! 表达式元数据包装器
//!
//! 本模块定义 ExpressionMeta 类型，它是核心 Expression 的包装器，
//! 包含位置信息（Span）和表达式 ID 等元数据。

use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::Expression;
use crate::core::types::{Position, Span};

/// 表达式 ID，用于缓存和追踪
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExpressionId(pub u64);

impl ExpressionId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 表达式元数据包装器
///
/// 核心表达式的包装器，提供：
/// - 位置信息（用于错误报告）
/// - 表达式 ID（用于缓存）
/// - 表达式复用（Arc）
///
/// # 示例
///
/// ```rust
/// use crate::core::types::{Expression, ExpressionMeta, Span, Position};
///
/// let expr = Expression::literal(42);
/// let meta = ExpressionMeta::with_span(expr, Span::new(Position::new(1, 1), Position::new(1, 2)));
/// assert!(meta.span().is_some());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "ExpressionMetaSerde", into = "ExpressionMetaSerde")]
pub struct ExpressionMeta {
    inner: Arc<Expression>,
    span: Option<Span>,
    id: Option<ExpressionId>,
}

/// 序列化辅助结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ExpressionMetaSerde {
    inner: Expression,
    #[serde(skip_serializing_if = "Option::is_none")]
    span_line_start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span_col_start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span_line_end: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    span_col_end: Option<usize>,
}

impl ExpressionMeta {
    /// 创建新的表达式元数据包装器（不包含位置信息）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::core::types::{Expression, ExpressionMeta};
    ///
    /// let expr = Expression::variable("x");
    /// let meta = ExpressionMeta::new(expr);
    /// assert!(meta.span().is_none());
    /// ```
    pub fn new(inner: Expression) -> Self {
        Self {
            inner: Arc::new(inner),
            span: None,
            id: None,
        }
    }

    /// 创建并设置位置信息
    ///
    /// # 示例
    ///
    /// ```rust
    /// use crate::core::types::{Expression, ExpressionMeta, Span, Position};
    ///
    /// let expr = Expression::literal("test");
    /// let span = Span::new(Position::new(5, 10), Position::new(5, 14));
    /// let meta = ExpressionMeta::with_span(expr, span);
    /// ```
    pub fn with_span(inner: Expression, span: Span) -> Self {
        Self {
            inner: Arc::new(inner),
            span: Some(span),
            id: None,
        }
    }

    /// 创建并设置表达式 ID
    pub fn with_id(mut self, id: ExpressionId) -> Self {
        self.id = Some(id);
        self
    }

    /// 获取位置信息
    pub fn span(&self) -> Option<&Span> {
        self.span.as_ref()
    }

    /// 获取表达式 ID
    pub fn id(&self) -> Option<&ExpressionId> {
        self.id.as_ref()
    }

    /// 获取内部引用
    pub fn inner(&self) -> &Expression {
        &self.inner
    }

    /// 克隆内部表达式（不克隆元数据）
    ///
    /// 注意：此方法会克隆整个表达式树，如果表达式较大请谨慎使用
    pub fn into_inner(self) -> Expression {
        self.inner.as_ref().clone()
    }

    /// 获取可变内部引用（必要时克隆）
    ///
    /// 如果 Arc 是唯一的引用，直接返回可变引用；
    /// 否则克隆内部表达式
    pub fn make_mut(&mut self) -> &mut Expression {
        if Arc::get_mut(&mut self.inner).is_none() {
            let cloned = self.inner.as_ref().clone();
            self.inner = Arc::new(cloned);
        }
        Arc::get_mut(&mut self.inner).expect("Arc should be unique after cloning")
    }

    /// 检查是否为字面量
    pub fn is_literal(&self) -> bool {
        self.inner.as_ref().is_literal()
    }

    /// 获取字面量值
    pub fn as_literal(&self) -> Option<&super::Value> {
        self.inner.as_ref().as_literal()
    }

    /// 检查是否为变量
    pub fn is_variable(&self) -> bool {
        self.inner.as_ref().is_variable()
    }

    /// 获取变量名
    pub fn as_variable(&self) -> Option<&str> {
        self.inner.as_ref().as_variable()
    }

    /// 检查是否为聚合表达式
    pub fn is_aggregate(&self) -> bool {
        self.inner.as_ref().is_aggregate()
    }

    /// 获取变量列表
    pub fn get_variables(&self) -> Vec<String> {
        self.inner.as_ref().get_variables()
    }

    /// 转换为字符串表示
    pub fn to_expression_string(&self) -> String {
        self.inner.as_ref().to_expression_string()
    }

    /// 获取所有子表达式
    pub fn children(&self) -> Vec<&Expression> {
        self.inner.as_ref().children()
    }

    /// 检查是否包含聚合函数
    pub fn contains_aggregate(&self) -> bool {
        self.inner.as_ref().contains_aggregate()
    }
}

/// 从 ExpressionMeta 提取核心表达式
impl From<ExpressionMeta> for Expression {
    fn from(meta: ExpressionMeta) -> Self {
        meta.into_inner()
    }
}

/// 从核心表达式创建
impl From<Expression> for ExpressionMeta {
    fn from(expr: Expression) -> Self {
        ExpressionMeta::new(expr)
    }
}

/// 为 ExpressionMeta 实现 PartialEq（比较内部表达式）
impl PartialEq for ExpressionMeta {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

/// 序列化实现
impl From<ExpressionMetaSerde> for ExpressionMeta {
    fn from(s: ExpressionMetaSerde) -> Self {
        let span = s.span_line_start.and_then(|start_line| {
            let start_col = s.span_col_start?;
            let end_line = s.span_line_end?;
            let end_col = s.span_col_end?;
            Some(Span::new(
                Position::new(start_line, start_col),
                Position::new(end_line, end_col),
            ))
        });
        Self {
            inner: Arc::new(s.inner),
            span,
            id: None,
        }
    }
}

impl From<ExpressionMeta> for ExpressionMetaSerde {
    fn from(m: ExpressionMeta) -> Self {
        Self {
            inner: m.inner.as_ref().clone(),
            span_line_start: m.span.as_ref().map(|s| s.start.line),
            span_col_start: m.span.as_ref().map(|s| s.start.column),
            span_line_end: m.span.as_ref().map(|s| s.end.line),
            span_col_end: m.span.as_ref().map(|s| s.end.column),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_meta_creation() {
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        assert!(meta.span().is_none());
        assert!(meta.id().is_none());
    }

    #[test]
    fn test_expression_meta_with_span() {
        let expr = Expression::variable("x");
        let span = Span::new(Position::new(1, 0), Position::new(1, 1));
        let meta = ExpressionMeta::with_span(expr, span);
        assert!(meta.span().is_some());
        assert_eq!(meta.span().unwrap().start.line, 1);
    }

    #[test]
    fn test_expression_meta_with_id() {
        let expr = Expression::literal(true);
        let meta = ExpressionMeta::new(expr).with_id(ExpressionId::new(42));
        assert!(meta.id().is_some());
        assert_eq!(meta.id().unwrap().0, 42);
    }

    #[test]
    fn test_expression_meta_into_inner() {
        let expr = Expression::literal("test");
        let meta = ExpressionMeta::new(expr);
        let inner: Expression = meta.into();
        assert!(matches!(inner, Expression::Literal(_)));
    }

    #[test]
    fn test_expression_meta_make_mut_shared() {
        let expr = Expression::variable("a");
        let mut meta1 = ExpressionMeta::new(expr);
        let _meta2 = meta1.clone();

        let inner_arc = &meta1.inner;
        assert!(Arc::strong_count(inner_arc) > 1);

        let _ = meta1.make_mut();
        let inner_arc = &meta1.inner;
        assert_eq!(Arc::strong_count(inner_arc), 1);
    }

    #[test]
    fn test_expression_meta_is_literal() {
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        assert!(meta.is_literal());

        let expr = Expression::variable("x");
        let meta = ExpressionMeta::new(expr);
        assert!(!meta.is_literal());
    }

    #[test]
    fn test_expression_meta_as_literal() {
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        assert!(meta.as_literal().is_some());
    }

    #[test]
    fn test_expression_meta_is_variable() {
        let expr = Expression::variable("x");
        let meta = ExpressionMeta::new(expr);
        assert!(meta.is_variable());

        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        assert!(!meta.is_variable());
    }

    #[test]
    fn test_expression_meta_as_variable() {
        let expr = Expression::variable("count");
        let meta = ExpressionMeta::new(expr);
        assert_eq!(meta.as_variable(), Some("count"));
    }

    #[test]
    fn test_expression_meta_partial_eq() {
        let expr1 = Expression::literal(42);
        let expr2 = Expression::literal(42);
        let meta1 = ExpressionMeta::new(expr1);
        let meta2 = ExpressionMeta::new(expr2);
        assert_eq!(meta1, meta2);

        let expr3 = Expression::literal(100);
        let meta3 = ExpressionMeta::new(expr3);
        assert_ne!(meta1, meta3);
    }

    #[test]
    fn test_expression_meta_serde() {
        let expr = Expression::literal("test");
        let meta = ExpressionMeta::new(expr);
        let json = serde_json::to_string(&meta).expect("Serialization should succeed");
        let decoded: ExpressionMeta = serde_json::from_str(&json).expect("Deserialization should succeed");
        assert_eq!(meta, decoded);
    }

    #[test]
    fn test_expression_meta_serde_with_span() {
        let expr = Expression::literal(42);
        let span = Span::new(Position::new(1, 5), Position::new(1, 10));
        let meta = ExpressionMeta::with_span(expr, span);
        let json = serde_json::to_string(&meta).expect("Serialization should succeed");
        let decoded: ExpressionMeta = serde_json::from_str(&json).expect("Deserialization should succeed");
        assert!(decoded.span().is_some());
        assert_eq!(decoded.span().unwrap().start.line, 1);
    }

    #[test]
    fn test_expression_meta_get_variables() {
        let expr = Expression::binary(
            Expression::variable("a"),
            super::super::BinaryOperator::Add,
            Expression::variable("b"),
        );
        let meta = ExpressionMeta::new(expr);
        let vars = meta.get_variables();
        assert_eq!(vars, vec!["a", "b"]);
    }

    #[test]
    fn test_expression_meta_to_string() {
        let expr = Expression::binary(
            Expression::variable("x"),
            super::super::BinaryOperator::Add,
            Expression::literal(1),
        );
        let meta = ExpressionMeta::new(expr);
        let s = meta.to_expression_string();
        assert!(s.contains("+"));
    }
}
