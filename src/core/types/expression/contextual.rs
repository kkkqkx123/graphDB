//! 上下文表达式
//!
//! 本模块定义 ContextualExpression，作为轻量级的表达式引用，
//! 持有 ExpressionId 和 Context 引用。

use std::sync::Arc;

use super::{Expression, ExpressionMeta, ExpressionId};
use super::context::ExpressionContext;
use crate::core::types::DataType;
use crate::core::Value;

/// 增强的表达式元数据，包含查询上下文引用
///
/// 轻量级的表达式引用，持有 ExpressionId 和 Context 引用。
/// 通过 ExpressionContext 可以访问表达式的完整信息、类型、常量值等。
#[derive(Debug, Clone)]
pub struct ContextualExpression {
    /// 表达式ID
    id: ExpressionId,
    /// 查询上下文引用
    context: Arc<ExpressionContext>,
}

impl ContextualExpression {
    /// 创建上下文表达式
    pub fn new(id: ExpressionId, context: Arc<ExpressionContext>) -> Self {
        Self { id, context }
    }
    
    /// 获取表达式ID
    pub fn id(&self) -> &ExpressionId {
        &self.id
    }
    
    /// 获取表达式元数据
    pub fn expression(&self) -> Option<Arc<ExpressionMeta>> {
        self.context.get_expression(&self.id)
    }
    
    /// 获取底层 Expression 的克隆
    /// 
    /// 此方法用于需要直接操作 Expression 的场景，
    /// 如模板提取、参数化等。大多数场景应使用 expression() 方法
    /// 
    /// # 使用限制
    /// 此方法只能在 Executor 层使用，其他层禁止调用
    /// 违反此限制将破坏表达式系统的设计原则
    pub fn get_expression(&self) -> Option<Expression> {
        self.expression().map(|meta| meta.inner.as_ref().clone())
    }
    
    /// 消费 self 并获取底层 Expression
    /// 
    /// 此方法用于需要获取 Expression 所有权而非引用的场景
    /// 
    /// # 使用限制
    /// 此方法只能在 Executor 层使用，其他层禁止调用
    /// 违反此限制将破坏表达式系统的设计原则
    pub fn into_expression(self) -> Expression {
        self.get_expression()
            .expect("Expression should exist in context")
    }
    
    /// 获取表达式类型
    pub fn data_type(&self) -> Option<DataType> {
        self.context.get_type(&self.id)
    }
    
    /// 获取常量值
    pub fn constant_value(&self) -> Option<Value> {
        self.context.get_constant(&self.id)
    }
    
    /// 是否为常量
    pub fn is_constant(&self) -> bool {
        self.context.is_constant(&self.id)
    }
    
    /// 是否已经过类型推导
    pub fn is_typed(&self) -> bool {
        self.context.is_typed(&self.id)
    }
    
    /// 是否已经过常量折叠
    pub fn is_constant_folded(&self) -> bool {
        self.context.is_constant_folded(&self.id)
    }
    
    /// 是否已经过公共子表达式消除
    pub fn is_cse_eliminated(&self) -> bool {
        self.context.is_cse_eliminated(&self.id)
    }
    
    /// 获取表达式上下文
    pub fn context(&self) -> &Arc<ExpressionContext> {
        &self.context
    }
    
    /// 检查表达式是否为字面量
    pub fn is_literal(&self) -> bool {
        self.expression()
            .map(|e| e.is_literal())
            .unwrap_or(false)
    }
    
    /// 检查表达式是否为变量
    pub fn is_variable(&self) -> bool {
        self.expression()
            .map(|e| e.is_variable())
            .unwrap_or(false)
    }
    
    /// 检查表达式是否为聚合表达式
    pub fn is_aggregate(&self) -> bool {
        self.expression()
            .map(|e| e.is_aggregate())
            .unwrap_or(false)
    }
    
    /// 获取变量名
    pub fn as_variable(&self) -> Option<String> {
        self.expression()
            .and_then(|e| e.as_variable().map(|s| s.to_string()))
    }
    
    /// 获取字面量值
    pub fn as_literal(&self) -> Option<Value> {
        self.expression()
            .and_then(|e| e.as_literal().cloned())
    }
    
    /// 获取变量列表
    pub fn get_variables(&self) -> Vec<String> {
        self.expression()
            .map(|e| e.get_variables())
            .unwrap_or_default()
    }
    
    /// 转换为字符串表示
    pub fn to_expression_string(&self) -> String {
        self.expression()
            .map(|e| e.to_expression_string())
            .unwrap_or_else(|| format!("<unknown expression {}>", self.id.0))
    }
    
    /// 检查是否包含聚合函数
    pub fn contains_aggregate(&self) -> bool {
        self.expression()
            .map(|e| e.contains_aggregate())
            .unwrap_or(false)
    }
}

impl PartialEq for ContextualExpression {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Arc::ptr_eq(&self.context, &other.context)
    }
}

impl Eq for ContextualExpression {}

impl std::hash::Hash for ContextualExpression {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        let ptr = Arc::as_ptr(&self.context) as usize;
        ptr.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_contextual_expression_creation() {
        let ctx = Arc::new(ExpressionContext::new());
        let id = ExpressionId::new(1);
        let ctx_expr = ContextualExpression::new(id, ctx);
        
        assert_eq!(ctx_expr.id().0, 1);
    }
    
    #[test]
    fn test_contextual_expression_with_registered() {
        let ctx = Arc::new(ExpressionContext::new());
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        
        let ctx_expr = ContextualExpression::new(id.clone(), ctx);
        
        assert!(ctx_expr.expression().is_some());
        assert!(ctx_expr.is_literal());
        assert_eq!(ctx_expr.as_literal(), Some(Value::Int(42)));
    }
    
    #[test]
    fn test_contextual_expression_with_type() {
        let ctx = Arc::new(ExpressionContext::new());
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        
        ctx.set_type(&id, DataType::Int);
        
        let ctx_expr = ContextualExpression::new(id.clone(), ctx);
        
        assert_eq!(ctx_expr.data_type(), Some(DataType::Int));
        assert!(ctx_expr.is_typed());
    }
    
    #[test]
    fn test_contextual_expression_with_constant() {
        let ctx = Arc::new(ExpressionContext::new());
        let expr = Expression::binary(
            Expression::literal(1),
            BinaryOperator::Add,
            Expression::literal(2),
        );
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        
        ctx.set_constant(&id, Value::Int(3));
        
        let ctx_expr = ContextualExpression::new(id.clone(), ctx);
        
        assert_eq!(ctx_expr.constant_value(), Some(Value::Int(3)));
        assert!(ctx_expr.is_constant());
        assert!(ctx_expr.is_constant_folded());
    }
    
    #[test]
    fn test_contextual_expression_is_variable() {
        let ctx = Arc::new(ExpressionContext::new());
        let expr = Expression::variable("x");
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        
        let ctx_expr = ContextualExpression::new(id.clone(), ctx);
        
        assert!(ctx_expr.is_variable());
        assert_eq!(ctx_expr.as_variable(), Some("x".to_string()));
    }
    
    #[test]
    fn test_contextual_expression_get_variables() {
        let ctx = Arc::new(ExpressionContext::new());
        let expr = Expression::binary(
            Expression::variable("a"),
            BinaryOperator::Add,
            Expression::variable("b"),
        );
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        
        let ctx_expr = ContextualExpression::new(id.clone(), ctx);
        
        let vars = ctx_expr.get_variables();
        assert_eq!(vars, vec!["a", "b"]);
    }
    
    #[test]
    fn test_contextual_expression_to_string() {
        let ctx = Arc::new(ExpressionContext::new());
        let expr = Expression::variable("x");
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        
        let ctx_expr = ContextualExpression::new(id.clone(), ctx);
        
        let s = ctx_expr.to_expression_string();
        assert!(s.contains("x"));
    }
    
    #[test]
    fn test_contextual_expression_partial_eq() {
        let ctx = Arc::new(ExpressionContext::new());
        let id = ExpressionId::new(1);
        
        let ctx_expr1 = ContextualExpression::new(id.clone(), ctx.clone());
        let ctx_expr2 = ContextualExpression::new(id, ctx);
        
        assert_eq!(ctx_expr1, ctx_expr2);
    }
    
    #[test]
    fn test_contextual_expression_partial_eq_different_context() {
        let ctx1 = Arc::new(ExpressionContext::new());
        let ctx2 = Arc::new(ExpressionContext::new());
        let id = ExpressionId::new(1);
        
        let ctx_expr1 = ContextualExpression::new(id.clone(), ctx1);
        let ctx_expr2 = ContextualExpression::new(id, ctx2);
        
        assert_ne!(ctx_expr1, ctx_expr2);
    }
    
    #[test]
    fn test_contextual_expression_context() {
        let ctx = Arc::new(ExpressionContext::new());
        let id = ExpressionId::new(1);
        let ctx_expr = ContextualExpression::new(id, ctx.clone());
        
        assert!(Arc::ptr_eq(ctx_expr.context(), &ctx));
    }
}
