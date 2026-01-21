//! EvaluableExprVisitor - 用于判断表达式是否可求值的访问器
//! 对应 NebulaGraph EvaluableExprVisitor.h/.cpp 的功能

use crate::core::expression_visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::Value;
use crate::core::{AggregateFunction, BinaryOperator, DataType, Expression, UnaryOperator};

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

    pub fn is_evaluable(&mut self, expr: &Expression) -> bool {
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
        for (when_expr, then_expr) in conditions {
            self.visit_expression(when_expr);
            self.visit_expression(then_expr);
        }
        if let Some(default_expr) = default {
            self.visit_expression(default_expr.as_ref());
        }
    }

    fn visit_type_cast(&mut self, expr: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expr)
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

    fn visit_constant_expr(&mut self, _expr: &crate::query::parser::ast::expr::ConstantExpr) -> Self::Result {
        // 常量表达式总是可求值的
    }

    fn visit_variable_expr(&mut self, _expr: &crate::query::parser::ast::expr::VariableExpr) -> Self::Result {
        // 变量表达式不可求值
        self.evaluable = false;
    }

    fn visit_binary_expr(&mut self, expr: &crate::query::parser::ast::expr::BinaryExpr) -> Self::Result {
        self.visit_expr(&expr.left);
        self.visit_expr(&expr.right);
    }

    fn visit_unary_expr(&mut self, expr: &crate::query::parser::ast::expr::UnaryExpr) -> Self::Result {
        self.visit_expr(&expr.operand);
    }

    fn visit_function_call_expr(&mut self, expr: &crate::query::parser::ast::expr::FunctionCallExpr) -> Self::Result {
        for arg in &expr.args {
            self.visit_expr(arg);
        }
    }

    fn visit_property_access_expr(&mut self, expr: &crate::query::parser::ast::expr::PropertyAccessExpr) -> Self::Result {
        self.visit_expr(&expr.object);
    }

    fn visit_list_expr(&mut self, expr: &crate::query::parser::ast::expr::ListExpr) -> Self::Result {
        for item in &expr.elements {
            self.visit_expr(item);
        }
    }

    fn visit_map_expr(&mut self, expr: &crate::query::parser::ast::expr::MapExpr) -> Self::Result {
        for (_, value) in &expr.pairs {
            self.visit_expr(value);
        }
    }

    fn visit_case_expr(&mut self, expr: &crate::query::parser::ast::expr::CaseExpr) -> Self::Result {
        for (cond, val) in &expr.when_then_pairs {
            self.visit_expr(cond);
            self.visit_expr(val);
        }
        if let Some(d) = &expr.default {
            self.visit_expr(d);
        }
    }

    fn visit_subscript_expr(&mut self, expr: &crate::query::parser::ast::expr::SubscriptExpr) -> Self::Result {
        self.visit_expr(&expr.collection);
        self.visit_expr(&expr.index);
    }

    fn visit_type_cast_expr(&mut self, expr: &crate::query::parser::ast::expr::TypeCastExpr) -> Self::Result {
        self.visit_expr(&expr.expr);
    }

    fn visit_range_expr(&mut self, expr: &crate::query::parser::ast::expr::RangeExpr) -> Self::Result {
        self.visit_expr(&expr.collection);
        if let Some(s) = &expr.start {
            self.visit_expr(s);
        }
        if let Some(e) = &expr.end {
            self.visit_expr(e);
        }
    }

    fn visit_path_expr(&mut self, expr: &crate::query::parser::ast::expr::PathExpr) -> Self::Result {
        for item in &expr.elements {
            self.visit_expr(item);
        }
    }

    fn visit_label_expr(&mut self, _expr: &crate::query::parser::ast::expr::LabelExpr) -> Self::Result {
        // 标签总是可求值的
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
