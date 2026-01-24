//! RewriteVisitor - 用于重写表达式的访问器
//!
//! 主要功能：
//! - 通用的表达式重写器
//! - 使用 Matcher 函数判断是否需要重写
//! - 使用 Rewriter 函数执行重写
//! - 支持选择性访问表达式类型

use crate::core::types::expression::Expression;
use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;

/// 匹配器类型：判断表达式是否需要重写
pub type Matcher = fn(&Expression) -> bool;

/// 重写器类型：执行表达式重写
pub type Rewriter = fn(&Expression) -> Expression;

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
        expression: &Expression,
        matcher: Matcher,
        rewriter: Rewriter,
    ) -> Expression {
        let mut visitor = Self::with_matcher_rewriter(matcher, rewriter);
        visitor.rewrite(expression)
    }

    /// 重写表达式
    pub fn rewrite(&mut self, expression: &Expression) -> Expression {
        if let Some(matcher) = self.matcher {
            if matcher(expression) {
                if let Some(rewriter) = self.rewriter {
                    return rewriter(expression);
                }
            }
        }
        self.visit_expression(expression);
        expression.clone()
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

    fn visit_property(&mut self, object: &Expression, _property: &str) -> Self::Result {
        self.visit_expression(object);
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        _op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        self.visit_expression(left);
        self.visit_expression(right);
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
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
        for (cond, expression) in conditions {
            self.visit_expression(cond);
            self.visit_expression(expression);
        }
        if let Some(default_expression) = default {
            self.visit_expression(default_expression);
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expression);
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

    fn visit_label(&mut self, _name: &str) -> Self::Result {
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}
