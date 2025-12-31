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

    fn visit_tag_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        // 标签属性不可求值
        self.evaluable = false;
    }

    fn visit_edge_property(&mut self, _edge: &str, _prop: &str) -> Self::Result {
        // 边属性不可求值
        self.evaluable = false;
    }

    fn visit_input_property(&mut self, _prop: &str) -> Self::Result {
        // 输入属性不可求值
        self.evaluable = false;
    }

    fn visit_variable_property(&mut self, _var: &str, _prop: &str) -> Self::Result {
        // 变量属性不可求值
        self.evaluable = false;
    }

    fn visit_source_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        // 源属性不可求值
        self.evaluable = false;
    }

    fn visit_destination_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        // 目标属性不可求值
        self.evaluable = false;
    }

    fn visit_es_query(&mut self, _query: &str) -> Self::Result {
        // ES查询不可求值
        self.evaluable = false;
    }

    // 其他方法使用默认实现，自动处理遍历
    // ExpressionVisitor 的默认实现会自动调用子表达式的 visit_expression
    // 因此我们不需要手动实现遍历逻辑

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_type_casting(&mut self, expr: &Expression, _target_type: &str) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_path_build(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_subscript_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        self.visit_expression(collection);
        if let Some(s) = start {
            self.visit_expression(s);
        }
        if let Some(e) = end {
            self.visit_expression(e);
        }
    }

    fn visit_match_path_pattern(
        &mut self,
        _path_alias: &str,
        _patterns: &[Expression],
    ) -> Self::Result {
        // 匹配路径模式不可求值
        self.evaluable = false;
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

    fn visit_predicate_expr(&mut self, expr: &crate::query::parser::ast::expr::PredicateExpr) -> Self::Result {
        self.visit_expr(&expr.list);
        self.visit_expr(&expr.condition);
    }

    fn visit_tag_property_expr(&mut self, _expr: &crate::query::parser::ast::expr::TagPropertyExpr) -> Self::Result {
        // 标签属性不可求值
        self.evaluable = false;
    }

    fn visit_edge_property_expr(&mut self, _expr: &crate::query::parser::ast::expr::EdgePropertyExpr) -> Self::Result {
        // 边属性不可求值
        self.evaluable = false;
    }

    fn visit_input_property_expr(&mut self, _expr: &crate::query::parser::ast::expr::InputPropertyExpr) -> Self::Result {
        // 输入属性不可求值
        self.evaluable = false;
    }

    fn visit_variable_property_expr(&mut self, _expr: &crate::query::parser::ast::expr::VariablePropertyExpr) -> Self::Result {
        // 变量属性不可求值
        self.evaluable = false;
    }

    fn visit_source_property_expr(&mut self, _expr: &crate::query::parser::ast::expr::SourcePropertyExpr) -> Self::Result {
        // 源属性不可求值
        self.evaluable = false;
    }

    fn visit_destination_property_expr(&mut self, _expr: &crate::query::parser::ast::expr::DestinationPropertyExpr) -> Self::Result {
        // 目标属性不可求值
        self.evaluable = false;
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

    fn visit_reduce_expr(&mut self, expr: &crate::query::parser::ast::expr::ReduceExpr) -> Self::Result {
        self.visit_expr(expr.list.as_ref());
        self.visit_expr(expr.initial.as_ref());
        self.visit_expr(expr.expr.as_ref());
    }

    fn visit_list_comprehension_expr(&mut self, expr: &crate::query::parser::ast::expr::ListComprehensionExpr) -> Self::Result {
        self.visit_expr(expr.generator.as_ref());
        if let Some(c) = &expr.condition {
            self.visit_expr(c.as_ref());
        }
    }

    fn visit_binary(&mut self, left: &Expression, _op: &BinaryOperator, right: &Expression) -> Self::Result {
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

    fn visit_aggregate(&mut self, _func: &AggregateFunction, arg: &Expression, _distinct: bool) -> Self::Result {
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

    fn visit_case(&mut self, conditions: &[(Expression, Expression)], default: &Option<Box<Expression>>) -> Self::Result {
        for (cond, val) in conditions {
            self.visit_expression(cond);
            self.visit_expression(val);
        }
        if let Some(d) = default {
            self.visit_expression(d);
        }
    }

    fn visit_type_cast(&mut self, expr: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit_expression(collection);
        self.visit_expression(index);
    }

    fn visit_range(&mut self, collection: &Expression, start: &Option<Box<Expression>>, end: &Option<Box<Expression>>) -> Self::Result {
        self.visit_expression(collection);
        if let Some(s) = start {
            self.visit_expression(s);
        }
        if let Some(e) = end {
            self.visit_expression(e);
        }
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item);
        }
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        // 标签总是可求值的
    }

    fn visit_unary_plus(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_unary_negate(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_unary_not(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_unary_incr(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_unary_decr(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_is_null(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_is_not_null(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_is_empty(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_is_not_empty(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr);
    }

    fn visit_list_comprehension(&mut self, generator: &Expression, condition: &Option<Box<Expression>>) -> Self::Result {
        self.visit_expression(generator);
        if let Some(c) = condition {
            self.visit_expression(c);
        }
    }

    fn visit_predicate(&mut self, list: &Expression, condition: &Expression) -> Self::Result {
        self.visit_expression(list);
        self.visit_expression(condition);
    }

    fn visit_reduce(&mut self, list: &Expression, _var: &str, initial: &Expression, expr: &Expression) -> Self::Result {
        self.visit_expression(list);
        self.visit_expression(initial);
        self.visit_expression(expr);
    }

    fn visit_uuid(&mut self) -> Self::Result {
        // UUID总是可求值的
    }
}
