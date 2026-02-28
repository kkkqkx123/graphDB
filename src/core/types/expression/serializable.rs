//! 可序列化表达式
//!
//! 本模块定义 SerializableExpression，用于存储和传输。

use std::sync::Arc;
use serde::{Serialize, Deserialize};

use super::{Expression, ExpressionMeta, ExpressionId};
use super::context::ExpressionContext;
use super::contextual::ContextualExpression;
use crate::core::types::DataType;
use crate::core::Value;

/// 可序列化的表达式引用（用于存储/传输）
///
/// 包含表达式的完整信息，可以序列化和反序列化。
/// 用于在需要序列化的场景（如网络传输、持久化）中使用。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableExpression {
    pub id: ExpressionId,
    pub expression: Expression,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constant_value: Option<Value>,
}

impl SerializableExpression {
    /// 从 ContextualExpression 转换为可序列化形式
    pub fn from_contextual(ctx_expr: &ContextualExpression) -> Self {
        let expr_meta = ctx_expr.expression().expect("Expression not found in context");
        Self {
            id: ctx_expr.id().clone(),
            expression: expr_meta.inner().clone(),
            data_type: ctx_expr.data_type(),
            constant_value: ctx_expr.constant_value(),
        }
    }
    
    /// 转换为 ContextualExpression
    pub fn to_contextual(self, ctx: Arc<ExpressionContext>) -> ContextualExpression {
        let expr_meta = ExpressionMeta::new(self.expression).with_id(self.id.clone());
        ctx.register_expression(expr_meta);
        
        if let Some(data_type) = self.data_type {
            ctx.set_type(&self.id, data_type);
        }
        
        if let Some(constant_value) = self.constant_value {
            ctx.set_constant(&self.id, constant_value);
        }
        
        ContextualExpression::new(self.id, ctx)
    }
    
    /// 获取表达式ID
    pub fn id(&self) -> &ExpressionId {
        &self.id
    }
    
    /// 获取表达式
    pub fn expression(&self) -> &Expression {
        &self.expression
    }
    
    /// 获取数据类型
    pub fn data_type(&self) -> Option<&DataType> {
        self.data_type.as_ref()
    }
    
    /// 获取常量值
    pub fn constant_value(&self) -> Option<&Value> {
        self.constant_value.as_ref()
    }
    
    /// 是否为常量
    pub fn is_constant(&self) -> bool {
        self.constant_value.is_some()
    }
    
    /// 检查表达式是否为字面量
    pub fn is_literal(&self) -> bool {
        self.expression.is_literal()
    }
    
    /// 检查表达式是否为变量
    pub fn is_variable(&self) -> bool {
        self.expression.is_variable()
    }
    
    /// 检查表达式是否为聚合表达式
    pub fn is_aggregate(&self) -> bool {
        self.expression.is_aggregate()
    }
    
    /// 获取变量名
    pub fn as_variable(&self) -> Option<String> {
        self.expression.as_variable().map(|s| s.to_string())
    }
    
    /// 获取字面量值
    pub fn as_literal(&self) -> Option<Value> {
        self.expression.as_literal().cloned()
    }
    
    /// 获取变量列表
    pub fn get_variables(&self) -> Vec<String> {
        self.expression.get_variables()
    }
    
    /// 转换为字符串表示
    pub fn to_expression_string(&self) -> String {
        self.expression.to_expression_string()
    }
    
    /// 检查是否包含聚合函数
    pub fn contains_aggregate(&self) -> bool {
        self.expression.contains_aggregate()
    }
}

impl PartialEq for SerializableExpression {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.expression == other.expression
    }
}

impl Eq for SerializableExpression {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_serializable_expression_creation() {
        let expr = Expression::literal(42);
        let ser_expr = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr,
            data_type: Some(DataType::Int),
            constant_value: Some(Value::Int(42)),
        };
        
        assert_eq!(ser_expr.id().0, 1);
        assert!(ser_expr.is_literal());
        assert_eq!(ser_expr.as_literal(), Some(Value::Int(42)));
        assert_eq!(ser_expr.data_type(), Some(&DataType::Int));
        assert_eq!(ser_expr.constant_value(), Some(&Value::Int(42)));
        assert!(ser_expr.is_constant());
    }
    
    #[test]
    fn test_serializable_expression_from_contextual() {
        let ctx = Arc::new(ExpressionContext::new());
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        
        ctx.set_type(&id, DataType::Int);
        ctx.set_constant(&id, Value::Int(42));
        
        let ctx_expr = ContextualExpression::new(id, ctx);
        let ser_expr = SerializableExpression::from_contextual(&ctx_expr);
        
        assert_eq!(ser_expr.id().0, 0);
        assert_eq!(ser_expr.data_type(), Some(&DataType::Int));
        assert_eq!(ser_expr.constant_value(), Some(&Value::Int(42)));
    }
    
    #[test]
    fn test_serializable_expression_to_contextual() {
        let ctx = Arc::new(ExpressionContext::new());
        let expr = Expression::literal(42);
        let ser_expr = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr,
            data_type: Some(DataType::Int),
            constant_value: Some(Value::Int(42)),
        };
        
        let ctx_expr = ser_expr.to_contextual(ctx);
        
        assert_eq!(ctx_expr.id().0, 1);
        assert_eq!(ctx_expr.data_type(), Some(DataType::Int));
        assert_eq!(ctx_expr.constant_value(), Some(Value::Int(42)));
        assert!(ctx_expr.is_constant());
    }
    
    #[test]
    fn test_serializable_expression_is_variable() {
        let expr = Expression::variable("x");
        let ser_expr = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr,
            data_type: None,
            constant_value: None,
        };
        
        assert!(ser_expr.is_variable());
        assert_eq!(ser_expr.as_variable(), Some("x".to_string()));
    }
    
    #[test]
    fn test_serializable_expression_get_variables() {
        let expr = Expression::binary(
            Expression::variable("a"),
            BinaryOperator::Add,
            Expression::variable("b"),
        );
        let ser_expr = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr,
            data_type: None,
            constant_value: None,
        };
        
        let vars = ser_expr.get_variables();
        assert_eq!(vars, vec!["a", "b"]);
    }
    
    #[test]
    fn test_serializable_expression_to_string() {
        let expr = Expression::variable("x");
        let ser_expr = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr,
            data_type: None,
            constant_value: None,
        };
        
        let s = ser_expr.to_expression_string();
        assert!(s.contains("x"));
    }
    
    #[test]
    fn test_serializable_expression_serde() {
        let expr = Expression::literal(42);
        let ser_expr = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr,
            data_type: Some(DataType::Int),
            constant_value: Some(Value::Int(42)),
        };
        
        let json = serde_json::to_string(&ser_expr).expect("Serialization should succeed");
        let decoded: SerializableExpression = serde_json::from_str(&json).expect("Deserialization should succeed");
        
        assert_eq!(ser_expr, decoded);
    }
    
    #[test]
    fn test_serializable_expression_partial_eq() {
        let expr1 = Expression::literal(42);
        let ser_expr1 = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr1,
            data_type: Some(DataType::Int),
            constant_value: Some(Value::Int(42)),
        };
        
        let expr2 = Expression::literal(42);
        let ser_expr2 = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr2,
            data_type: Some(DataType::Int),
            constant_value: Some(Value::Int(42)),
        };
        
        assert_eq!(ser_expr1, ser_expr2);
    }
    
    #[test]
    fn test_serializable_expression_partial_eq_different_id() {
        let expr1 = Expression::literal(42);
        let ser_expr1 = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr1,
            data_type: Some(DataType::Int),
            constant_value: Some(Value::Int(42)),
        };
        
        let expr2 = Expression::literal(42);
        let ser_expr2 = SerializableExpression {
            id: ExpressionId::new(2),
            expression: expr2,
            data_type: Some(DataType::Int),
            constant_value: Some(Value::Int(42)),
        };
        
        assert_ne!(ser_expr1, ser_expr2);
    }
    
    #[test]
    fn test_serializable_expression_is_aggregate() {
        use crate::core::types::operators::AggregateFunction;
        
        let expr = Expression::aggregate(
            AggregateFunction::Count(None),
            Expression::variable("x"),
            false,
        );
        let ser_expr = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr,
            data_type: None,
            constant_value: None,
        };
        
        assert!(ser_expr.is_aggregate());
    }
    
    #[test]
    fn test_serializable_expression_contains_aggregate() {
        use crate::core::types::operators::AggregateFunction;
        
        let expr = Expression::aggregate(
            AggregateFunction::Count(None),
            Expression::variable("x"),
            false,
        );
        let ser_expr = SerializableExpression {
            id: ExpressionId::new(1),
            expression: expr,
            data_type: None,
            constant_value: None,
        };
        
        assert!(ser_expr.contains_aggregate());
    }
}
