//! VariableVisitor - 用于收集表达式中变量的访问器
//! 对应 NebulaGraph VariableVisitor.h/.cpp 的功能

use crate::core::expression_visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::Value;
use crate::expression::Expression;
use std::collections::HashSet;

#[derive(Debug)]
pub struct VariableVisitor {
    /// 收集到的变量名集合
    variables: HashSet<String>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl VariableVisitor {
    pub fn new() -> Self {
        Self {
            variables: HashSet::new(),
            state: ExpressionVisitorState::new(),
        }
    }

    /// 收集表达式中使用的所有变量
    pub fn collect_variables(&mut self, expr: &Expression) -> HashSet<String> {
        self.variables.clear();
        let _ = self.visit_expression(expr);
        self.variables.clone()
    }

    /// 检查表达式中是否包含变量
    pub fn has_variables(&mut self, expr: &Expression) -> bool {
        self.variables.clear();
        let _ = self.visit_expression(expr);
        !self.variables.is_empty()
    }

    /// 获取收集到的变量列表
    pub fn get_variables(&self) -> Vec<String> {
        self.variables.iter().cloned().collect()
    }

    /// 清空收集到的变量
    pub fn clear(&mut self) {
        self.variables.clear();
    }
}

impl ExpressionVisitor for VariableVisitor {
    type Result = ();

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.variables.insert(name.to_string());
    }

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {}

    fn visit_property(&mut self, object: &Expression, _property: &str) -> Self::Result {
        self.visit_expression(object);
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        _op: &crate::core::types::operators::BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        self.visit_expression(left);
        self.visit_expression(right);
    }

    fn visit_unary(
        &mut self,
        _op: &crate::core::types::operators::UnaryOperator,
        operand: &Expression,
    ) -> Self::Result {
        self.visit_expression(operand);
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) -> Self::Result {
        for arg in args {
            self.visit_expression(arg);
        }
    }

    fn visit_aggregate(
        &mut self,
        _func: &crate::core::types::operators::AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        self.visit_expression(arg);
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_, value) in pairs {
            self.visit_expression(value);
        }
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        for (condition, value) in conditions {
            self.visit_expression(condition);
            self.visit_expression(value);
        }
        if let Some(expr) = default {
            self.visit_expression(expr);
        }
    }

    fn visit_type_cast(
        &mut self,
        expr: &Expression,
        _target_type: &crate::core::types::expression::DataType,
    ) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit_expression(collection);
        self.visit_expression(index);
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        self.visit_expression(collection);
        if let Some(expr) = start {
            self.visit_expression(expr);
        }
        if let Some(expr) = end {
            self.visit_expression(expr);
        }
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {}

    fn visit_constant_expr(&mut self, _e: &crate::query::parser::ast::expr::ConstantExpr) -> Self::Result {
        // 常量表达式不包含变量，无需处理
    }

    fn visit_variable_expr(&mut self, e: &crate::query::parser::ast::expr::VariableExpr) -> Self::Result {
        self.visit_variable(&e.name);
    }

    fn visit_binary_expr(&mut self, e: &crate::query::parser::ast::expr::BinaryExpr) -> Self::Result {
        self.visit_expr(e.left.as_ref());
        self.visit_expr(e.right.as_ref());
    }

    fn visit_unary_expr(&mut self, e: &crate::query::parser::ast::expr::UnaryExpr) -> Self::Result {
        self.visit_expr(e.operand.as_ref());
    }

    fn visit_function_call_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::FunctionCallExpr,
    ) -> Self::Result {
        for arg in &e.args {
            self.visit_expr(arg);
        }
    }

    fn visit_property_access_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::PropertyAccessExpr,
    ) -> Self::Result {
        self.visit_expr(e.object.as_ref());
    }

    fn visit_list_expr(&mut self, e: &crate::query::parser::ast::expr::ListExpr) -> Self::Result {
        for item in &e.elements {
            self.visit_expr(item);
        }
    }

    fn visit_map_expr(&mut self, e: &crate::query::parser::ast::expr::MapExpr) -> Self::Result {
        for (_, value) in &e.pairs {
            self.visit_expr(value);
        }
    }

    fn visit_case_expr(&mut self, e: &crate::query::parser::ast::expr::CaseExpr) -> Self::Result {
        for (condition, value) in &e.when_then_pairs {
            self.visit_expr(condition);
            self.visit_expr(value);
        }
        if let Some(expr) = &e.default {
            self.visit_expr(expr.as_ref());
        }
    }

    fn visit_subscript_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::SubscriptExpr,
    ) -> Self::Result {
        self.visit_expr(e.collection.as_ref());
        self.visit_expr(e.index.as_ref());
    }

    fn visit_type_cast_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::TypeCastExpr,
    ) -> Self::Result {
        self.visit_expr(e.expr.as_ref());
    }

    fn visit_range_expr(&mut self, e: &crate::query::parser::ast::expr::RangeExpr) -> Self::Result {
        self.visit_expr(e.collection.as_ref());
        if let Some(expr) = &e.start {
            self.visit_expr(expr.as_ref());
        }
        if let Some(expr) = &e.end {
            self.visit_expr(expr.as_ref());
        }
    }

    fn visit_path_expr(&mut self, e: &crate::query::parser::ast::expr::PathExpr) -> Self::Result {
        for item in &e.elements {
            self.visit_expr(item);
        }
    }

    fn visit_label_expr(&mut self, _e: &crate::query::parser::ast::expr::LabelExpr) -> Self::Result {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_collect_variables() {
        let mut visitor = VariableVisitor::new();

        let expr = Expression::Variable("x".to_string());
        let variables = visitor.collect_variables(&expr);
        assert_eq!(variables.len(), 1);
        assert!(variables.contains("x"));

        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("a".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Variable("b".to_string())),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(Value::Int(2))),
            }),
        };

        let variables = visitor.collect_variables(&expr);
        assert_eq!(variables.len(), 2);
        assert!(variables.contains("a"));
        assert!(variables.contains("b"));
    }

    #[test]
    fn test_has_variables() {
        let mut visitor = VariableVisitor::new();

        let expr = Expression::Variable("x".to_string());
        assert!(visitor.has_variables(&expr));

        let expr = Expression::Literal(Value::Int(42));
        assert!(!visitor.has_variables(&expr));

        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("a".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(1))),
        };

        assert!(visitor.has_variables(&expr));
    }

    #[test]
    fn test_get_variables() {
        let mut visitor = VariableVisitor::new();

        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("var1".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Variable("var2".to_string())),
        };

        visitor.collect_variables(&expr);
        let variables = visitor.get_variables();
        
        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"var1".to_string()));
        assert!(variables.contains(&"var2".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut visitor = VariableVisitor::new();

        let expr = Expression::Variable("x".to_string());
        visitor.collect_variables(&expr);
        
        assert!(!visitor.get_variables().is_empty());
        
        visitor.clear();
        assert!(visitor.get_variables().is_empty());
    }
}
