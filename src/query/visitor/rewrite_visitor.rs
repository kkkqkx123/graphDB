//! RewriteVisitor - 用于重写表达式的访问器
//!
//! 主要功能：
//! - 通用的表达式重写器
//! - 使用 Matcher 函数判断是否需要重写
//! - 使用 Rewriter 函数执行重写
//! - 支持选择性访问表达式类型

use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, Expression, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use crate::expression::Expr;
use crate::query::parser::ast::expr::*;

/// 匹配器类型：判断表达式是否需要重写
pub type Matcher = fn(&Expr) -> bool;

/// 重写器类型：执行表达式重写
pub type Rewriter = fn(&Expr) -> Expression;

/// 表达式重写访问器
///
/// 用于基于匹配器和重写器转换表达式
#[derive(Debug)]
pub struct RewriteVisitor {
    /// 匹配器
    matcher: Option<Matcher>,
    /// 重写器
    rewriter: Option<Rewriter>,
    /// 是否成功
    ok: bool,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl RewriteVisitor {
    /// 创建新的表达式重写访问器
    pub fn new() -> Self {
        Self {
            matcher: None,
            rewriter: None,
            ok: true,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 创建带有匹配器和重写器的访问器
    pub fn with_matcher_rewriter(matcher: Matcher, rewriter: Rewriter) -> Self {
        Self {
            matcher: Some(matcher),
            rewriter: Some(rewriter),
            ok: true,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 静态方法：转换表达式
    pub fn transform(
        expr: &Expr,
        matcher: Matcher,
        rewriter: Rewriter,
    ) -> Expression {
        let mut visitor = Self::with_matcher_rewriter(matcher, rewriter);
        visitor.rewrite(expr)
    }

    /// 重写表达式
    pub fn rewrite(&mut self, expr: &Expr) -> Expression {
        if let Some(matcher) = self.matcher {
            if matcher(expr) {
                if let Some(rewriter) = self.rewriter {
                    return rewriter(expr);
                }
            }
        }
        self.visit_expression(expr);
        expr.clone()
    }

    /// 获取匹配器
    pub fn matcher(&self) -> Option<Matcher> {
        self.matcher
    }

    /// 获取重写器
    pub fn rewriter(&self) -> Option<Rewriter> {
        self.rewriter
    }

    /// 检查是否成功
    pub fn ok(&self) -> bool {
        self.ok
    }
}

impl Default for RewriteVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionVisitor for RewriteVisitor {
    type Result = ();

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
    }

    fn visit_property(&mut self, object: &Expr, _property: &str) -> Self::Result {
        self.visit_expression(object);
    }

    fn visit_binary(
        &mut self,
        left: &Expr,
        _op: &BinaryOperator,
        right: &Expr,
    ) -> Self::Result {
        self.visit_expression(left);
        self.visit_expression(right);
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expr) -> Self::Result {
        self.visit_expression(operand);
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
        conditions: &[(Expr, Expr)],
        default: &Option<Box<Expr>>,
    ) -> Self::Result {
        for (cond, expr) in conditions {
            self.visit_expression(cond);
            self.visit_expression(expr);
        }
        if let Some(default_expr) = default {
            self.visit_expression(default_expr);
        }
    }

    fn visit_type_cast(&mut self, expr: &Expr, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_subscript(&mut self, collection: &Expr, index: &Expr) -> Self::Result {
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

    fn visit_label(&mut self, _name: &str) -> Self::Result {
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}
