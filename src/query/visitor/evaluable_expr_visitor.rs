//! EvaluableExprVisitor - 用于判断表达式是否可求值的访问器
//! 对应 NebulaGraph EvaluableExprVisitor.h/.cpp 的功能

use crate::core::types::expression::Expression;
use crate::core::expression_visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::Value;
use crate::core::{AggregateFunction, BinaryOperator, DataType, UnaryOperator};

#[derive(Debug)]
pub struct EvaluableExprVisitor {
    /// 表达式是否可求值
    evaluable: bool,
    /// 错误信息
    error: Option<String>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl EvaluableExprVisitor {
    pub fn new() -> Self {
        Self {
            evaluable: true,
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    pub fn is_evaluable(&mut self, expression: &Expression) -> bool {
        self.evaluable = true;
        self.error = None;
        self.visit_expression(expression);
        self.evaluable
    }

    pub fn get_error(&self) -> Option<&String> {
        self.error.as_ref()
    }
}

impl ExpressionVisitor for EvaluableExprVisitor {
    type Result = ();

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
        // 字面量总是可求值的
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        // 变量引用不可求值
        self.evaluable = false;
    }

    fn visit_property(&mut self, _object: &Expression, _property: &str) -> Self::Result {
        // 属性访问不可求值
        self.evaluable = false;
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand)
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) -> Self::Result {
        for arg in args {
            self.visit_expression(arg);
        }
    }

    fn visit_aggregate(
        &mut self,
        _func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        self.visit_expression(arg)
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_key, value) in pairs {
            self.visit_expression(value);
        }
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        for (when_expression, then_expression) in conditions {
            self.visit_expression(when_expression);
            self.visit_expression(then_expression);
        }
        if let Some(default_expression) = default {
            self.visit_expression(default_expression.as_ref());
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expression)
    }

    fn visit_subscript(
        &mut self,
        collection: &Expression,
        index: &Expression,
    ) -> Self::Result {
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
        if let Some(start_expression) = start {
            self.visit_expression(start_expression);
        }
        if let Some(end_expression) = end {
            self.visit_expression(end_expression);
        }
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item);
        }
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

    fn visit_label(&mut self, _name: &str) -> Self::Result {}

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}
