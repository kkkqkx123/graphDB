//! Expression analysis context
//!
//! This module defines the ExpressionAnalysisContext, which serves as a shared context across different stages.
//! Store all the complete information about the expressions.
//!
//! Note: This context is used for the compilation-time analysis phase (optimizers, type inference, etc.).
//! For runtime evaluation, please use the `expression::evaluator::ExpressionContext` trait.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::types::expr::{Expression, ExpressionId, ExpressionMeta};
use crate::core::types::operators::BinaryOperator;
use crate::core::types::operators::UnaryOperator;
use crate::core::types::DataType;
use crate::core::Value;
use crate::query::optimizer::analysis::ExpressionAnalysis;

/// Expression Optimization Status Indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OptimizationFlags {
    /// Has type inference already been performed?
    pub typed: bool,
    /// Has constant folding already been performed?
    pub constant_folded: bool,
    /// Has the elimination of common subexpressions already been performed?
    pub cse_eliminated: bool,
}

/// Expression analysis context
///
/// Expression information storage that is shared across different stages, supporting concurrent access.
/// Store the complete information of the expression, including:
/// Expression Registry: Stores complete information about all expressions.
/// Type information caching: Expression ID -> Derived type
/// Constant folding result: Expression ID -> The calculated constant value
/// Expression Analysis Results: Expression ID -> Analysis Results
/// Optimization Status: Expression ID
///
/// Note: This context is used for the compilation-time analysis phase (optimizers, type inference, etc.).
/// For runtime evaluation, please use the `expression::evaluator::ExpressionContext` trait.
///
/// # Optimization Note
/// Uses `RwLock<HashMap>` instead of `DashMap` because:
/// - This context is primarily used during compilation-time analysis (single-threaded or low contention)
/// - `RwLock<HashMap>` has better read performance for the typical access patterns
/// - Reduces dependency complexity and compile times
#[derive(Debug)]
pub struct ExpressionAnalysisContext {
    /// Expression Registry: Stores complete information about all expressions.
    expressions: Arc<RwLock<HashMap<ExpressionId, Arc<ExpressionMeta>>>>,

    /// Type information cache: Expression ID -> Derived type
    type_cache: Arc<RwLock<HashMap<ExpressionId, DataType>>>,

    /// Constant folding result: Expression ID -> The calculated constant value
    constant_cache: Arc<RwLock<HashMap<ExpressionId, Value>>>,

    /// Expression analysis results: Expression ID -> Analysis results
    analysis_cache: Arc<RwLock<HashMap<ExpressionId, ExpressionAnalysis>>>,

    /// Optimization flag: Expression ID -> Optimization status
    optimization_flags: Arc<RwLock<HashMap<ExpressionId, OptimizationFlags>>>,
}

impl ExpressionAnalysisContext {
    /// Create a new context for expression analysis.
    pub fn new() -> Self {
        Self {
            expressions: Arc::new(RwLock::new(HashMap::new())),
            type_cache: Arc::new(RwLock::new(HashMap::new())),
            constant_cache: Arc::new(RwLock::new(HashMap::new())),
            analysis_cache: Arc::new(RwLock::new(HashMap::new())),
            optimization_flags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register the expression in its context.
    ///
    /// If the expression already has an ID, use that ID; otherwise, generate a new ID.
    pub fn register_expression(&self, expr: ExpressionMeta) -> ExpressionId {
        let id = expr
            .id()
            .cloned()
            .unwrap_or_else(|| ExpressionId::new(self.expressions.read().len() as u64));

        self.expressions.write().insert(id.clone(), Arc::new(expr));
        id
    }

    /// Obtain the expression
    pub fn get_expression(&self, id: &ExpressionId) -> Option<Arc<ExpressionMeta>> {
        self.expressions.read().get(id).cloned()
    }

    /// Set the expression type
    pub fn set_type(&self, id: &ExpressionId, data_type: DataType) {
        self.type_cache.write().insert(id.clone(), data_type);
        let mut flags = self
            .optimization_flags
            .read()
            .get(id)
            .copied()
            .unwrap_or_default();
        flags.typed = true;
        self.optimization_flags.write().insert(id.clone(), flags);
    }

    /// Determine the type of the expression.
    pub fn get_type(&self, id: &ExpressionId) -> Option<DataType> {
        self.type_cache.read().get(id).cloned()
    }

    /// Setting constant values
    pub fn set_constant(&self, id: &ExpressionId, value: Value) {
        self.constant_cache.write().insert(id.clone(), value);
        self.optimization_flags.write().insert(
            id.clone(),
            OptimizationFlags {
                typed: true,
                constant_folded: true,
                cse_eliminated: false,
            },
        );
    }

    /// Obtain the constant value
    pub fn get_constant(&self, id: &ExpressionId) -> Option<Value> {
        self.constant_cache.read().get(id).cloned()
    }

    /// Set optimization flags
    pub fn set_optimization_flag(&self, id: &ExpressionId, flags: OptimizationFlags) {
        self.optimization_flags.write().insert(id.clone(), flags);
    }

    /// Obtain the optimization markers.
    pub fn get_optimization_flags(&self, id: &ExpressionId) -> Option<OptimizationFlags> {
        self.optimization_flags.read().get(id).copied()
    }

    /// Check whether the expression is a constant.
    pub fn is_constant(&self, id: &ExpressionId) -> bool {
        self.constant_cache.read().contains_key(id)
    }

    /// Check whether the expression has already undergone type inference.
    pub fn is_typed(&self, id: &ExpressionId) -> bool {
        self.optimization_flags
            .read()
            .get(id)
            .map(|f| f.typed)
            .unwrap_or(false)
    }

    /// Check whether the expression has undergone constant folding.
    pub fn is_constant_folded(&self, id: &ExpressionId) -> bool {
        self.optimization_flags
            .read()
            .get(id)
            .map(|f| f.constant_folded)
            .unwrap_or(false)
    }

    /// Check whether the expression has already undergone the elimination of common subexpressions.
    pub fn is_cse_eliminated(&self, id: &ExpressionId) -> bool {
        self.optimization_flags
            .read()
            .get(id)
            .map(|f| f.cse_eliminated)
            .unwrap_or(false)
    }

    /// Obtain the number of registered expressions.
    pub fn expression_count(&self) -> usize {
        self.expressions.read().len()
    }

    /// Clear all caches (expressions and the registry will be retained).
    pub fn clear_caches(&self) {
        self.type_cache.write().clear();
        self.constant_cache.write().clear();
        self.analysis_cache.write().clear();
        self.optimization_flags.write().clear();
    }

    /// Clear all data.
    pub fn clear_all(&self) {
        self.expressions.write().clear();
        self.clear_caches();
    }

    /// Set the analysis results of the expression.
    ///
    /// # Parameters
    /// `id`: Expression ID
    /// “analysis”: Results of the analysis
    pub fn set_analysis(&self, id: &ExpressionId, analysis: ExpressionAnalysis) {
        self.analysis_cache.write().insert(id.clone(), analysis);
    }

    /// Obtain the results of the expression analysis.
    ///
    /// # Parameters
    /// - `id`: expression ID
    ///
    /// # Return
    /// Analysis results (if any)
    pub fn get_analysis(&self, id: &ExpressionId) -> Option<ExpressionAnalysis> {
        self.analysis_cache.read().get(id).cloned()
    }

    /// Check whether the expression has already been analyzed.
    ///
    /// # Parameters
    /// - `id`: 表达式ID
    ///
    /// # Back
    /// “true” if the analysis has already been performed.
    pub fn is_analyzed(&self, id: &ExpressionId) -> bool {
        self.analysis_cache.read().contains_key(id)
    }

    // ==================== Expression Rewriting API ====================
    // The following methods are used for the rewriting and combination of expressions, in order to avoid direct operations on expressions at the Rewrite layer.

    /// Clone the expression and register it in the context.
    ///
    /// Extract the Expression from the existing ContextualExpression, create a copy of it, and register it in the context.
    /// Return the new ContextualExpression.
    pub fn clone_expression(
        &self,
        ctx_expr: &ContextualExpression,
    ) -> Option<ContextualExpression> {
        let expr_meta = ctx_expr.expression()?;
        let inner_expr = expr_meta.inner().clone();
        let meta = ExpressionMeta::new(inner_expr);
        let id = self.register_expression(meta);
        Some(ContextualExpression::new(id, ctx_expr.context().clone()))
    }

    /// Combine the two expressions into a binary expression.
    ///
    /// # Parameters
    /// “op”: Binary operator
    /// “left”: The ContextualExpression of the left operand.
    /// “right”: The ContextualExpression of the right operand
    ///
    /// # Back
    /// The combined ContextualExpression
    pub fn combine_expressions(
        &self,
        op: BinaryOperator,
        left: &ContextualExpression,
        right: &ContextualExpression,
    ) -> Option<ContextualExpression> {
        let left_meta = left.expression()?;
        let right_meta = right.expression()?;

        let combined_expr = Expression::Binary {
            left: Box::new(left_meta.inner().clone()),
            op,
            right: Box::new(right_meta.inner().clone()),
        };

        let meta = ExpressionMeta::new(combined_expr);
        let id = self.register_expression(meta);
        Some(ContextualExpression::new(id, left.context().clone()))
    }

    /// Create a monomial expression.
    ///
    /// # Parameters
    /// “op” stands for “unary operator”.
    /// `operand`: The ContextualExpression of the operand.
    ///
    /// # Back
    /// New ContextualExpression
    pub fn create_unary_expression(
        &self,
        op: UnaryOperator,
        operand: &ContextualExpression,
    ) -> Option<ContextualExpression> {
        let operand_meta = operand.expression()?;

        let unary_expr = Expression::Unary {
            op,
            operand: Box::new(operand_meta.inner().clone()),
        };

        let meta = ExpressionMeta::new(unary_expr);
        let id = self.register_expression(meta);
        Some(ContextualExpression::new(id, operand.context().clone()))
    }

    /// Create attribute access expressions
    ///
    /// # Parameters
    /// “object”: The ContextualExpression of the object.
    /// “property”: The name of the property.
    ///
    /// # Back
    /// 新的 ContextualExpression
    pub fn create_property_expression(
        &self,
        object: &ContextualExpression,
        property: &str,
    ) -> Option<ContextualExpression> {
        let object_meta = object.expression()?;

        let property_expr = Expression::Property {
            object: Box::new(object_meta.inner().clone()),
            property: property.to_string(),
        };

        let meta = ExpressionMeta::new(property_expr);
        let id = self.register_expression(meta);
        Some(ContextualExpression::new(id, object.context().clone()))
    }

    /// Create a function call expression.
    ///
    /// # Parameters
    /// `name`: The name of the function
    /// `args`: A list of `ContextualExpression` objects representing the parameters.
    /// `ctx_expr`: A `ContextualExpression` used to retrieve the context.
    ///
    /// # Back
    /// 新的 ContextualExpression
    pub fn create_function_expression(
        &self,
        name: &str,
        args: &[ContextualExpression],
        ctx_expr: &ContextualExpression,
    ) -> Option<ContextualExpression> {
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
        Some(ContextualExpression::new(id, ctx_expr.context().clone()))
    }

    /// Creating an AND expression
    ///
    /// A convenient method for combining two conditional expressions
    pub fn and(
        &self,
        left: &ContextualExpression,
        right: &ContextualExpression,
    ) -> Option<ContextualExpression> {
        self.combine_expressions(BinaryOperator::And, left, right)
    }

    /// Creating an OR expression
    ///
    /// Convenience method for combining two conditional expressions
    pub fn or(
        &self,
        left: &ContextualExpression,
        right: &ContextualExpression,
    ) -> Option<ContextualExpression> {
        self.combine_expressions(BinaryOperator::Or, left, right)
    }

    /// Creating a NOT expression
    ///
    /// A convenient method for creating negative expressions
    pub fn not(&self, operand: &ContextualExpression) -> Option<ContextualExpression> {
        self.create_unary_expression(UnaryOperator::Not, operand)
    }
}

impl Clone for ExpressionAnalysisContext {
    fn clone(&self) -> Self {
        Self {
            expressions: Arc::new(RwLock::new(self.expressions.read().clone())),
            type_cache: Arc::new(RwLock::new(self.type_cache.read().clone())),
            constant_cache: Arc::new(RwLock::new(self.constant_cache.read().clone())),
            analysis_cache: Arc::new(RwLock::new(self.analysis_cache.read().clone())),
            optimization_flags: Arc::new(RwLock::new(self.optimization_flags.read().clone())),
        }
    }
}

impl Default for ExpressionAnalysisContext {
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
        let ctx = ExpressionAnalysisContext::new();
        assert_eq!(ctx.expression_count(), 0);
    }

    #[test]
    fn test_register_expression() {
        let ctx = ExpressionAnalysisContext::new();
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);

        let id = ctx.register_expression(meta);
        assert_eq!(ctx.expression_count(), 1);
        assert_eq!(id.0, 0);
    }

    #[test]
    fn test_register_expression_with_id() {
        let ctx = ExpressionAnalysisContext::new();
        let expr = Expression::literal("test");
        let meta = ExpressionMeta::new(expr).with_id(ExpressionId::new(100));

        let id = ctx.register_expression(meta);
        assert_eq!(id.0, 100);
    }

    #[test]
    fn test_get_expression() {
        let ctx = ExpressionAnalysisContext::new();
        let expr = Expression::variable("x");
        let meta = ExpressionMeta::new(expr);

        let id = ctx.register_expression(meta);
        let retrieved = ctx.get_expression(&id);
        assert!(retrieved.is_some());
        assert!(retrieved.expect("The expression should exist").is_variable());
    }

    #[test]
    fn test_set_and_get_type() {
        let ctx = ExpressionAnalysisContext::new();
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
        let ctx = ExpressionAnalysisContext::new();
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
        let ctx = ExpressionAnalysisContext::new();
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
        let ctx = ExpressionAnalysisContext::new();
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
        let ctx = ExpressionAnalysisContext::new();
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
        let ctx = ExpressionAnalysisContext::default();
        assert_eq!(ctx.expression_count(), 0);
    }

    #[test]
    fn test_set_and_get_analysis() {
        let ctx = ExpressionAnalysisContext::new();
        let expr = Expression::variable("x");
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);

        let analysis = ExpressionAnalysis {
            is_deterministic: true,
            complexity_score: 10,
            referenced_properties: vec!["name".to_string()],
            referenced_variables: vec!["x".to_string()],
            called_functions: vec![],
            contains_aggregate: false,
            contains_subquery: false,
            node_count: 1,
        };

        ctx.set_analysis(&id, analysis.clone());
        let retrieved = ctx.get_analysis(&id);
        assert!(retrieved.is_some());
        assert!(ctx.is_analyzed(&id));
    }
}
