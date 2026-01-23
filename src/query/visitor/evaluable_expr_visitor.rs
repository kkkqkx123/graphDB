//! EvaluableExprVisitor - 用于判断表达式是否可求值的访问器
//! 对应 NebulaGraph EvaluableExprVisitor.h/.cpp 的功能

use crate::core::expression_visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::Value;
use crate::core::{AggregateFunction, BinaryOperator, DataType, Expression, UnaryOperator};
use crate::expression::Expr;

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

    pub fn is_evaluable(&mut self, expr: &Expr) -> bool {
        self.evaluable = true;
        self.error = None;
        self.visit_expression(expr);
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

    fn visit_property(&mut self, _object: &Expr, _property: &str) -> Self::Result {
        // 属性访问不可求值
        self.evaluable = false;
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expr) -> Self::Result {
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
        arg: &Expr,
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
        conditions: &[(Expr, Expr)],
        default: &Option<Box<Expr>>,
    ) -> Self::Result {
        for (when_expr, then_expr) in conditions {
            self.visit_expression(when_expr);
            self.visit_expression(then_expr);
        }
        if let Some(default_expr) = default {
            self.visit_expression(default_expr.as_ref());
        }
    }

    fn visit_type_cast(&mut self, expr: &Expr, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_subscript(
        &mut self,
        collection: &Expr,
        index: &Expr,
    ) -> Self::Result {
        self.visit_expression(collection);
        self.visit_expression(index);
    }

    fn visit_range(
        &mut self,
        collection: &Expr,
        start: &Option<Box<Expr>>,
        end: &Option<Box<Expr>>,
    ) -> Self::Result {
        self.visit_expression(collection);
        if let Some(start_expr) = start {
            self.visit_expression(start_expr);
        }
        if let Some(end_expr) = end {
            self.visit_expression(end_expr);
        }
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        _op: &crate::core::types::operators::BinaryOperator,
        right: &Expr,
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
