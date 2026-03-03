//! 表达式上下文
//!
//! 本模块定义 ExpressionContext，作为跨阶段的共享上下文，
//! 存储所有表达式的完整信息。

use dashmap::DashMap;
use std::sync::Arc;

use super::{Expression, ExpressionId, ExpressionMeta};
use crate::core::types::DataType;
use crate::core::types::operators::BinaryOperator;
use crate::core::types::operators::UnaryOperator;
use crate::core::Value;

/// 表达式优化状态标记
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptimizationFlags {
    /// 是否已经过类型推导
    pub typed: bool,
    /// 是否已经过常量折叠
    pub constant_folded: bool,
    /// 是否已经过公共子表达式消除
    pub cse_eliminated: bool,
}

impl Default for OptimizationFlags {
    fn default() -> Self {
        Self {
            typed: false,
            constant_folded: false,
            cse_eliminated: false,
        }
    }
}

/// 表达式上下文
///
/// 跨阶段共享的表达式信息存储，支持并发访问。
/// 存储表达式的完整信息，包括：
/// - 表达式注册表：存储所有表达式的完整信息
/// - 类型信息缓存：表达式ID -> 推导出的类型
/// - 常量折叠结果：表达式ID -> 计算出的常量值
/// - 优化标记：表达式ID -> 优化状态
#[derive(Debug, Clone)]
pub struct ExpressionContext {
    /// 表达式注册表：存储所有表达式的完整信息
    expressions: Arc<DashMap<ExpressionId, Arc<ExpressionMeta>>>,

    /// 类型信息缓存：表达式ID -> 推导出的类型
    type_cache: Arc<DashMap<ExpressionId, DataType>>,

    /// 常量折叠结果：表达式ID -> 计算出的常量值
    constant_cache: Arc<DashMap<ExpressionId, Value>>,

    /// 优化标记：表达式ID -> 优化状态
    optimization_flags: Arc<DashMap<ExpressionId, OptimizationFlags>>,
}

impl ExpressionContext {
    /// 创建新的表达式上下文
    pub fn new() -> Self {
        Self {
            expressions: Arc::new(DashMap::new()),
            type_cache: Arc::new(DashMap::new()),
            constant_cache: Arc::new(DashMap::new()),
            optimization_flags: Arc::new(DashMap::new()),
        }
    }

    /// 注册表达式到上下文中
    ///
    /// 如果表达式已有ID，使用该ID；否则生成新的ID
    pub fn register_expression(&self, expr: ExpressionMeta) -> ExpressionId {
        let id = expr
            .id()
            .cloned()
            .unwrap_or_else(|| ExpressionId::new(self.expressions.len() as u64));

        self.expressions.insert(id.clone(), Arc::new(expr));
        id
    }

    /// 获取表达式
    pub fn get_expression(&self, id: &ExpressionId) -> Option<Arc<ExpressionMeta>> {
        self.expressions.get(id).map(|r| r.clone())
    }

    /// 设置表达式类型
    pub fn set_type(&self, id: &ExpressionId, data_type: DataType) {
        self.type_cache.insert(id.clone(), data_type);
        let mut flags = self
            .optimization_flags
            .get(id)
            .map(|r| *r.value())
            .unwrap_or_default();
        flags.typed = true;
        self.optimization_flags.insert(id.clone(), flags);
    }

    /// 获取表达式类型
    pub fn get_type(&self, id: &ExpressionId) -> Option<DataType> {
        self.type_cache.get(id).map(|r| r.clone())
    }

    /// 设置常量值
    pub fn set_constant(&self, id: &ExpressionId, value: Value) {
        self.constant_cache.insert(id.clone(), value);
        self.optimization_flags.insert(
            id.clone(),
            OptimizationFlags {
                typed: true,
                constant_folded: true,
                cse_eliminated: false,
            },
        );
    }

    /// 获取常量值
    pub fn get_constant(&self, id: &ExpressionId) -> Option<Value> {
        self.constant_cache.get(id).map(|r| r.clone())
    }

    /// 设置优化标记
    pub fn set_optimization_flag(&self, id: &ExpressionId, flags: OptimizationFlags) {
        self.optimization_flags.insert(id.clone(), flags);
    }

    /// 获取优化标记
    pub fn get_optimization_flags(&self, id: &ExpressionId) -> Option<OptimizationFlags> {
        self.optimization_flags.get(id).map(|r| *r.value())
    }

    /// 检查表达式是否为常量
    pub fn is_constant(&self, id: &ExpressionId) -> bool {
        self.constant_cache.contains_key(id)
    }

    /// 检查表达式是否已经过类型推导
    pub fn is_typed(&self, id: &ExpressionId) -> bool {
        self.optimization_flags
            .get(id)
            .map(|r| r.value().typed)
            .unwrap_or(false)
    }

    /// 检查表达式是否已经过常量折叠
    pub fn is_constant_folded(&self, id: &ExpressionId) -> bool {
        self.optimization_flags
            .get(id)
            .map(|r| r.value().constant_folded)
            .unwrap_or(false)
    }

    /// 检查表达式是否已经过公共子表达式消除
    pub fn is_cse_eliminated(&self, id: &ExpressionId) -> bool {
        self.optimization_flags
            .get(id)
            .map(|r| r.value().cse_eliminated)
            .unwrap_or(false)
    }

    /// 获取注册的表达式数量
    pub fn expression_count(&self) -> usize {
        self.expressions.len()
    }

    /// 清空所有缓存（保留表达式注册表）
    pub fn clear_caches(&self) {
        self.type_cache.clear();
        self.constant_cache.clear();
        self.optimization_flags.clear();
    }

    /// 清空所有数据
    pub fn clear_all(&self) {
        self.expressions.clear();
        self.clear_caches();
    }

    // ==================== 表达式重写 API ====================
    // 以下方法用于表达式重写和组合，避免在 Rewrite 层直接操作 Expression

    /// 克隆表达式并注册到上下文
    ///
    /// 从现有的 ContextualExpression 中提取 Expression，创建副本并注册到上下文
    /// 返回新的 ContextualExpression
    pub fn clone_expression(
        &self,
        ctx_expr: &crate::core::types::expression::contextual::ContextualExpression,
    ) -> Option<crate::core::types::expression::contextual::ContextualExpression> {
        let expr_meta = ctx_expr.expression()?;
        let inner_expr = expr_meta.inner().clone();
        let meta = ExpressionMeta::new(inner_expr);
        let id = self.register_expression(meta);
        Some(crate::core::types::expression::contextual::ContextualExpression::new(
            id,
            ctx_expr.context().clone(),
        ))
    }

    /// 组合两个表达式为二元表达式
    ///
    /// # 参数
    /// - `op`: 二元操作符
    /// - `left`: 左操作数的 ContextualExpression
    /// - `right`: 右操作数的 ContextualExpression
    ///
    /// # 返回
    /// 组合后的 ContextualExpression
    pub fn combine_expressions(
        &self,
        op: BinaryOperator,
        left: &crate::core::types::expression::contextual::ContextualExpression,
        right: &crate::core::types::expression::contextual::ContextualExpression,
    ) -> Option<crate::core::types::expression::contextual::ContextualExpression> {
        let left_meta = left.expression()?;
        let right_meta = right.expression()?;

        let combined_expr = Expression::Binary {
            left: Box::new(left_meta.inner().clone()),
            op,
            right: Box::new(right_meta.inner().clone()),
        };

        let meta = ExpressionMeta::new(combined_expr);
        let id = self.register_expression(meta);
        Some(crate::core::types::expression::contextual::ContextualExpression::new(
            id,
            left.context().clone(),
        ))
    }

    /// 创建一元表达式
    ///
    /// # 参数
    /// - `op`: 一元操作符
    /// - `operand`: 操作数的 ContextualExpression
    ///
    /// # 返回
    /// 新的 ContextualExpression
    pub fn create_unary_expression(
        &self,
        op: UnaryOperator,
        operand: &crate::core::types::expression::contextual::ContextualExpression,
    ) -> Option<crate::core::types::expression::contextual::ContextualExpression> {
        let operand_meta = operand.expression()?;

        let unary_expr = Expression::Unary {
            op,
            operand: Box::new(operand_meta.inner().clone()),
        };

        let meta = ExpressionMeta::new(unary_expr);
        let id = self.register_expression(meta);
        Some(crate::core::types::expression::contextual::ContextualExpression::new(
            id,
            operand.context().clone(),
        ))
    }

    /// 创建属性访问表达式
    ///
    /// # 参数
    /// - `object`: 对象的 ContextualExpression
    /// - `property`: 属性名
    ///
    /// # 返回
    /// 新的 ContextualExpression
    pub fn create_property_expression(
        &self,
        object: &crate::core::types::expression::contextual::ContextualExpression,
        property: &str,
    ) -> Option<crate::core::types::expression::contextual::ContextualExpression> {
        let object_meta = object.expression()?;

        let property_expr = Expression::Property {
            object: Box::new(object_meta.inner().clone()),
            property: property.to_string(),
        };

        let meta = ExpressionMeta::new(property_expr);
        let id = self.register_expression(meta);
        Some(crate::core::types::expression::contextual::ContextualExpression::new(
            id,
            object.context().clone(),
        ))
    }

    /// 创建函数调用表达式
    ///
    /// # 参数
    /// - `name`: 函数名
    /// - `args`: 参数的 ContextualExpression 列表
    /// - `ctx_expr`: 用于获取上下文的 ContextualExpression
    ///
    /// # 返回
    /// 新的 ContextualExpression
    pub fn create_function_expression(
        &self,
        name: &str,
        args: &[crate::core::types::expression::contextual::ContextualExpression],
        ctx_expr: &crate::core::types::expression::contextual::ContextualExpression,
    ) -> Option<crate::core::types::expression::contextual::ContextualExpression> {
        let arg_exprs: Vec<Expression> = args
            .iter()
            .filter_map(|arg| arg.expression().map(|meta| meta.inner().clone()))
            .collect();

        if arg_exprs.len() != args.len() {
            return None;
        }

        let function_expr = Expression::Function {
            name: name.to_string(),
            args: arg_exprs,
        };

        let meta = ExpressionMeta::new(function_expr);
        let id = self.register_expression(meta);
        Some(crate::core::types::expression::contextual::ContextualExpression::new(
            id,
            ctx_expr.context().clone(),
        ))
    }

    /// 创建 AND 表达式
    ///
    /// 便捷方法，用于组合两个条件表达式
    pub fn and(
        &self,
        left: &crate::core::types::expression::contextual::ContextualExpression,
        right: &crate::core::types::expression::contextual::ContextualExpression,
    ) -> Option<crate::core::types::expression::contextual::ContextualExpression> {
        self.combine_expressions(BinaryOperator::And, left, right)
    }

    /// 创建 OR 表达式
    ///
    /// 便捷方法，用于组合两个条件表达式
    pub fn or(
        &self,
        left: &crate::core::types::expression::contextual::ContextualExpression,
        right: &crate::core::types::expression::contextual::ContextualExpression,
    ) -> Option<crate::core::types::expression::contextual::ContextualExpression> {
        self.combine_expressions(BinaryOperator::Or, left, right)
    }

    /// 创建 NOT 表达式
    ///
    /// 便捷方法，用于创建否定表达式
    pub fn not(
        &self,
        operand: &crate::core::types::expression::contextual::ContextualExpression,
    ) -> Option<crate::core::types::expression::contextual::ContextualExpression> {
        self.create_unary_expression(UnaryOperator::Not, operand)
    }
}

impl Default for ExpressionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_expression_context_creation() {
        let ctx = ExpressionContext::new();
        assert_eq!(ctx.expression_count(), 0);
    }

    #[test]
    fn test_register_expression() {
        let ctx = ExpressionContext::new();
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);

        let id = ctx.register_expression(meta);
        assert_eq!(ctx.expression_count(), 1);
        assert_eq!(id.0, 0);
    }

    #[test]
    fn test_register_expression_with_id() {
        let ctx = ExpressionContext::new();
        let expr = Expression::literal("test");
        let meta = ExpressionMeta::new(expr).with_id(ExpressionId::new(100));

        let id = ctx.register_expression(meta);
        assert_eq!(id.0, 100);
    }

    #[test]
    fn test_get_expression() {
        let ctx = ExpressionContext::new();
        let expr = Expression::variable("x");
        let meta = ExpressionMeta::new(expr);

        let id = ctx.register_expression(meta);
        let retrieved = ctx.get_expression(&id);
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().is_variable());
    }

    #[test]
    fn test_set_and_get_type() {
        let ctx = ExpressionContext::new();
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);

        ctx.set_type(&id, DataType::Int);
        let data_type = ctx.get_type(&id);
        assert_eq!(data_type, Some(DataType::Int));
        assert!(ctx.is_typed(&id));
    }

    #[test]
    fn test_set_and_get_constant() {
        let ctx = ExpressionContext::new();
        let expr = Expression::binary(
            Expression::literal(1),
            BinaryOperator::Add,
            Expression::literal(2),
        );
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);

        ctx.set_constant(&id, Value::Int(3));
        let constant = ctx.get_constant(&id);
        assert_eq!(constant, Some(Value::Int(3)));
        assert!(ctx.is_constant(&id));
        assert!(ctx.is_constant_folded(&id));
    }

    #[test]
    fn test_optimization_flags() {
        let ctx = ExpressionContext::new();
        let expr = Expression::variable("x");
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);

        let flags = OptimizationFlags {
            typed: true,
            constant_folded: false,
            cse_eliminated: true,
        };
        ctx.set_optimization_flag(&id, flags);

        let retrieved = ctx.get_optimization_flags(&id);
        assert_eq!(retrieved, Some(flags));
        assert!(ctx.is_typed(&id));
        assert!(!ctx.is_constant_folded(&id));
        assert!(ctx.is_cse_eliminated(&id));
    }

    #[test]
    fn test_clear_caches() {
        let ctx = ExpressionContext::new();
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);

        ctx.set_type(&id, DataType::Int);
        ctx.set_constant(&id, Value::Int(42));

        ctx.clear_caches();

        assert!(ctx.get_type(&id).is_none());
        assert!(ctx.get_constant(&id).is_none());
        assert_eq!(ctx.expression_count(), 1);
    }

    #[test]
    fn test_clear_all() {
        let ctx = ExpressionContext::new();
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);

        ctx.set_type(&id, DataType::Int);

        ctx.clear_all();

        assert_eq!(ctx.expression_count(), 0);
        assert!(ctx.get_expression(&id).is_none());
    }

    #[test]
    fn test_default() {
        let ctx = ExpressionContext::default();
        assert_eq!(ctx.expression_count(), 0);
    }
}
