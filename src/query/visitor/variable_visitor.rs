//! VariableVisitor - 用于收集表达式中变量的访问器
//!
//! 对应 NebulaGraph VariableVisitor.h/.cpp 的功能
//! 优化实现：只重写必要的方法，其他使用默认遍历逻辑

use crate::core::types::expression::visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::types::expression::Expression;
use crate::core::Value;
use std::collections::HashSet;

#[derive(Debug)]
pub struct VariableVisitor {
    variables: HashSet<String>,
    state: ExpressionVisitorState,
}

impl VariableVisitor {
    pub fn new() -> Self {
        Self {
            variables: HashSet::new(),
            state: ExpressionVisitorState::new(),
        }
    }

    pub fn collect_variables(&mut self, expression: &Expression) -> HashSet<String> {
        self.variables.clear();
        let _ = self.visit_expression(expression);
        self.variables.clone()
    }

    pub fn has_variables(&mut self, expression: &Expression) -> bool {
        self.variables.clear();
        let _ = self.visit_expression(expression);
        !self.variables.is_empty()
    }

    pub fn get_variables(&self) -> Vec<String> {
        self.variables.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.variables.clear();
    }
}

impl ExpressionVisitor for VariableVisitor {
    type Result = ();

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.variables.insert(name.to_string());
    }

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {}

    fn visit_property(&mut self, object: &Expression, _property: &str) -> Self::Result {
        self.visit_expression(object);
    }

    fn visit_binary(&mut self, left: &Expression, _op: &crate::core::types::operators::BinaryOperator, right: &Expression) -> Self::Result {
        self.visit_expression(left);
        self.visit_expression(right);
    }

    fn visit_unary(&mut self, _op: &crate::core::types::operators::UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand);
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) -> Self::Result {
        for arg in args {
            self.visit_expression(arg);
        }
    }

    fn visit_aggregate(&mut self, _func: &crate::core::types::operators::AggregateFunction, arg: &Expression, _distinct: bool) -> Self::Result {
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

    fn visit_case(&mut self, test_expr: Option<&Expression>, conditions: &[(Expression, Expression)], default: Option<&Expression>) -> Self::Result {
        if let Some(test) = test_expr {
            self.visit_expression(test);
        }
        for (cond, val) in conditions {
            self.visit_expression(cond);
            self.visit_expression(val);
        }
        if let Some(expr) = default {
            self.visit_expression(expr);
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: &crate::core::types::expression::DataType) -> Self::Result {
        self.visit_expression(expression);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit_expression(collection);
        self.visit_expression(index);
    }

    fn visit_range(&mut self, collection: &Expression, start: Option<&Expression>, end: Option<&Expression>) -> Self::Result {
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

    fn visit_list_comprehension(&mut self, _variable: &str, source: &Expression, filter: Option<&Expression>, map: Option<&Expression>) -> Self::Result {
        self.visit_expression(source);
        if let Some(expr) = filter {
            self.visit_expression(expr);
        }
        if let Some(expr) = map {
            self.visit_expression(expr);
        }
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_collect_variables() {
        let mut visitor = VariableVisitor::new();

        let expression = Expression::Variable("x".to_string());
        let variables = visitor.collect_variables(&expression);
        assert_eq!(variables.len(), 1);
        assert!(variables.contains("x"));

        let expression = Expression::Binary {
            left: Box::new(Expression::Variable("a".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Variable("b".to_string())),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(Value::Int(2))),
            }),
        };

        let variables = visitor.collect_variables(&expression);
        assert_eq!(variables.len(), 2);
        assert!(variables.contains("a"));
        assert!(variables.contains("b"));
    }

    #[test]
    fn test_has_variables() {
        let mut visitor = VariableVisitor::new();

        let expression = Expression::Variable("x".to_string());
        assert!(visitor.has_variables(&expression));

        let expression = Expression::Literal(Value::Int(42));
        assert!(!visitor.has_variables(&expression));

        let expression = Expression::Binary {
            left: Box::new(Expression::Variable("a".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(1))),
        };

        assert!(visitor.has_variables(&expression));
    }

    #[test]
    fn test_get_variables() {
        let mut visitor = VariableVisitor::new();

        let expression = Expression::Binary {
            left: Box::new(Expression::Variable("var1".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Variable("var2".to_string())),
        };

        visitor.collect_variables(&expression);
        let variables = visitor.get_variables();
        
        assert_eq!(variables.len(), 2);
        assert!(variables.contains(&"var1".to_string()));
        assert!(variables.contains(&"var2".to_string()));
    }

    #[test]
    fn test_clear() {
        let mut visitor = VariableVisitor::new();

        let expression = Expression::Variable("x".to_string());
        visitor.collect_variables(&expression);
        
        assert!(!visitor.get_variables().is_empty());
        
        visitor.clear();
        assert!(visitor.get_variables().is_empty());
    }
}
