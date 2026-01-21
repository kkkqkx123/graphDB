//! RewriteVisitor - 用于重写表达式的访问器
//!
//! 主要功能：
//! - 通用的表达式重写器
//! - 使用 Matcher 函数判断是否需要重写
//! - 使用 Rewriter 函数执行重写
//! - 支持选择性访问表达式类型

use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, Expression, ExpressionType, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use crate::query::parser::ast::expr::*;
use std::collections::HashSet;

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
    /// 需要访问的表达式类型
    need_visited_types: HashSet<ExpressionType>,
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
            need_visited_types: HashSet::new(),
            ok: true,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 创建带有匹配器和重写器的访问器
    pub fn with_matcher_rewriter(matcher: Matcher, rewriter: Rewriter) -> Self {
        Self {
            matcher: Some(matcher),
            rewriter: Some(rewriter),
            need_visited_types: HashSet::new(),
            ok: true,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 创建带有匹配器、重写器和需要访问类型的访问器
    pub fn with_types(
        matcher: Matcher,
        rewriter: Rewriter,
        need_visited_types: HashSet<ExpressionType>,
    ) -> Self {
        Self {
            matcher: Some(matcher),
            rewriter: Some(rewriter),
            need_visited_types,
            ok: true,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 静态方法：转换表达式
    pub fn transform(
        expr: &Expression,
        matcher: Matcher,
        rewriter: Rewriter,
    ) -> Expression {
        let mut visitor = Self::with_matcher_rewriter(matcher, rewriter);
        visitor.rewrite(expr)
    }

    /// 静态方法：转换表达式（带类型限制）
    pub fn transform_with_types(
        expr: &Expression,
        matcher: Matcher,
        rewriter: Rewriter,
        need_visited_types: HashSet<ExpressionType>,
    ) -> Expression {
        let mut visitor = Self::with_types(matcher, rewriter, need_visited_types);
        visitor.rewrite(expr)
    }

    /// 重写表达式
    pub fn rewrite(&mut self, expr: &Expression) -> Expression {
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

    /// 检查是否需要访问某种类型
    fn care(&self, kind: ExpressionType) -> bool {
        self.need_visited_types.is_empty() || self.need_visited_types.contains(&kind)
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
        if self.care(ExpressionType::Literal) {}
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        if self.care(ExpressionType::Variable) {}
    }

    fn visit_property(&mut self, object: &Expression, _property: &str) -> Self::Result {
        if self.care(ExpressionType::Property) {
            self.visit_expression(object);
        }
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        _op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        if self.care(ExpressionType::Binary) {
            self.visit_expression(left);
            self.visit_expression(right);
        }
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        if self.care(ExpressionType::Unary) {
            self.visit_expression(operand);
        }
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) -> Self::Result {
        if self.care(ExpressionType::Function) {
            for arg in args {
                self.visit_expression(arg);
            }
        }
    }

    fn visit_aggregate(
        &mut self,
        _func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        if self.care(ExpressionType::Aggregate) {
            self.visit_expression(arg);
        }
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        if self.care(ExpressionType::List) {
            for item in items {
                self.visit_expression(item);
            }
        }
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        if self.care(ExpressionType::Map) {
            for (_, value) in pairs {
                self.visit_expression(value);
            }
        }
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.care(ExpressionType::Case) {
            for (cond, expr) in conditions {
                self.visit_expression(cond);
                self.visit_expression(expr);
            }
            if let Some(default_expr) = default {
                self.visit_expression(default_expr);
            }
        }
    }

    fn visit_type_cast(&mut self, expr: &Expression, _target_type: &DataType) -> Self::Result {
        if self.care(ExpressionType::TypeCast) {
            self.visit_expression(expr);
        }
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        if self.care(ExpressionType::Subscript) {
            self.visit_expression(collection);
            self.visit_expression(index);
        }
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        if self.care(ExpressionType::Range) {
            self.visit_expression(collection);
            if let Some(start_expr) = start {
                self.visit_expression(start_expr);
            }
            if let Some(end_expr) = end {
                self.visit_expression(end_expr);
            }
        }
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        if self.care(ExpressionType::Path) {
            for item in items {
                self.visit_expression(item);
            }
        }
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        if self.care(ExpressionType::Label) {}
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_constant_expr(&mut self, _expr: &ConstantExpr) -> Self::Result {
        if self.care(ExpressionType::Literal) {}
    }

    fn visit_variable_expr(&mut self, _expr: &VariableExpr) -> Self::Result {
        if self.care(ExpressionType::Variable) {}
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Self::Result {
        if self.care(ExpressionType::Binary) {
            self.visit_expr(&expr.left);
            self.visit_expr(&expr.right);
        }
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Self::Result {
        if self.care(ExpressionType::Unary) {
            self.visit_expr(&expr.operand);
        }
    }

    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpr) -> Self::Result {
        if self.care(ExpressionType::Function) {
            for arg in &expr.args {
                self.visit_expr(arg);
            }
        }
    }

    fn visit_property_access_expr(&mut self, expr: &PropertyAccessExpr) -> Self::Result {
        if self.care(ExpressionType::Property) {
            self.visit_expr(&expr.object);
        }
    }

    fn visit_list_expr(&mut self, expr: &ListExpr) -> Self::Result {
        if self.care(ExpressionType::List) {
            for item in &expr.elements {
                self.visit_expr(item);
            }
        }
    }

    fn visit_map_expr(&mut self, expr: &MapExpr) -> Self::Result {
        if self.care(ExpressionType::Map) {
            for (_, value) in &expr.pairs {
                self.visit_expr(value);
            }
        }
    }

    fn visit_case_expr(&mut self, expr: &CaseExpr) -> Self::Result {
        if self.care(ExpressionType::Case) {
            for (condition, value) in &expr.when_then_pairs {
                self.visit_expr(condition);
                self.visit_expr(value);
            }
            if let Some(default_expr) = &expr.default {
                self.visit_expr(default_expr);
            }
        }
    }

    fn visit_subscript_expr(&mut self, expr: &SubscriptExpr) -> Self::Result {
        self.visit_expr(expr.collection.as_ref());
        self.visit_expr(expr.index.as_ref());
    }

    fn visit_type_cast_expr(&mut self, expr: &TypeCastExpr) -> Self::Result {
        self.visit_expr(expr.expr.as_ref());
    }

    fn visit_range_expr(&mut self, expr: &RangeExpr) -> Self::Result {
        if self.care(ExpressionType::Range) {
            self.visit_expr(&expr.collection);
            if let Some(start_expr) = &expr.start {
                self.visit_expr(start_expr);
            }
            if let Some(end_expr) = &expr.end {
                self.visit_expr(end_expr);
            }
        }
    }

    fn visit_path_expr(&mut self, expr: &PathExpr) -> Self::Result {
        if self.care(ExpressionType::Path) {
            for item in &expr.elements {
                self.visit_expr(item);
            }
        }
    }

    fn visit_label_expr(&mut self, _expr: &LabelExpr) -> Self::Result {
        if self.care(ExpressionType::Label) {}
    }
}
