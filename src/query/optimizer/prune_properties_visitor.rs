//! 属性修剪访问者
//!
//! 用于修剪未使用的属性，优化查询计划

use crate::core::expression_visitor::ExpressionVisitor;
use crate::core::types::expression::Expression;
use crate::query::optimizer::property_tracker::PropertyTracker;
use crate::query::context::validate::types::Variable;
use std::collections::HashSet;

/// 属性修剪访问者
///
/// 遍历表达式树，收集使用的属性
#[derive(Debug)]
pub struct PrunePropertiesVisitor {
    /// 属性跟踪器
    tracker: PropertyTracker,
    /// 访问状态
    state: PruneVisitorState,
}

/// 属性修剪访问者状态
#[derive(Debug, Clone)]
pub struct PruneVisitorState {
    /// 是否继续访问
    pub continue_visiting: bool,
    /// 访问深度
    pub depth: usize,
    /// 访问计数
    pub visit_count: usize,
    /// 最大深度限制
    pub max_depth: Option<usize>,
    /// 收集的变量
    pub collected_variables: HashSet<String>,
}

impl PruneVisitorState {
    /// 创建新的访问者状态
    pub fn new() -> Self {
        Self {
            continue_visiting: true,
            depth: 0,
            visit_count: 0,
            max_depth: None,
            collected_variables: HashSet::new(),
        }
    }

    /// 检查是否超过最大深度
    pub fn exceeds_max_depth(&self) -> bool {
        if let Some(max) = self.max_depth {
            self.depth > max
        } else {
            false
        }
    }

    /// 增加访问深度
    pub fn increment_depth(&mut self) {
        self.depth += 1;
    }

    /// 减少访问深度
    pub fn decrement_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    /// 增加访问计数
    pub fn increment_visit_count(&mut self) {
        self.visit_count += 1;
    }

    /// 设置最大深度限制
    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = Some(max_depth);
    }

    /// 停止访问
    pub fn stop(&mut self) {
        self.continue_visiting = false;
    }
}

impl Default for PruneVisitorState {
    fn default() -> Self {
        Self::new()
    }
}

impl PrunePropertiesVisitor {
    /// 创建新的属性修剪访问者
    pub fn new(tracker: PropertyTracker) -> Self {
        Self {
            tracker,
            state: PruneVisitorState::new(),
        }
    }

    /// 获取属性跟踪器
    pub fn tracker(&self) -> &PropertyTracker {
        &self.tracker
    }

    /// 获取可变属性跟踪器
    pub fn tracker_mut(&mut self) -> &mut PropertyTracker {
        &mut self.tracker
    }

    /// 获取访问状态
    pub fn state(&self) -> &PruneVisitorState {
        &self.state
    }

    /// 获取可变访问状态
    pub fn state_mut(&mut self) -> &mut PruneVisitorState {
        &mut self.state
    }

    /// 检查是否应该继续访问
    pub fn should_continue(&self) -> bool {
        self.state.continue_visiting
    }

    /// 停止访问
    pub fn stop(&mut self) {
        self.state.stop();
    }

    /// 收集表达式中的所有属性
    pub fn collect_properties(&mut self, expr: &Expression) {
        self.visit_expression(expr);
    }

    /// 获取收集的属性
    pub fn get_collected_properties(&self) -> &HashSet<String> {
        &self.state.collected_variables
    }

    /// 获取特定变量的使用属性
    pub fn get_used_properties(&self, var: &str) -> Option<&HashSet<String>> {
        self.tracker.get_used_properties(var)
    }
}

impl ExpressionVisitor for PrunePropertiesVisitor {
    type Result = ();

    fn visit_expression(&mut self, expr: &Expression) -> Self::Result {
        if !self.should_continue() {
            return;
        }

        self.state.increment_depth();

        if self.state.exceeds_max_depth() {
            self.state.decrement_depth();
            return;
        }

        self.state.increment_visit_count();

        match expr {
            Expression::Variable(name) => {
                self.state.collected_variables.insert(name.clone());
            }
            Expression::Property { object, property } => {
                // 跟踪属性
                if let Expression::Variable(var_name) = object.as_ref() {
                    self.tracker.track_property(var_name, property);
                }
            }
            Expression::TagProperty { tag, prop } => {
                // 跟踪顶点属性
                self.tracker.track_property(tag, prop);
            }
            Expression::EdgeProperty { edge, prop } => {
                // 跟踪边属性
                self.tracker.track_property(edge, prop);
            }
            Expression::InputProperty(prop) => {
                // 跟踪输入属性
                self.tracker.track_property("$-", prop);
            }
            Expression::VariableProperty { var, prop } => {
                // 跟踪变量属性
                self.tracker.track_property(var, prop);
            }
            Expression::SourceProperty { tag, prop } => {
                // 跟踪源顶点属性
                self.tracker.track_property(&format!("^{}", tag), prop);
            }
            Expression::DestinationProperty { tag, prop } => {
                // 跟踪目标顶点属性
                self.tracker.track_property(&format!("${}", tag), prop);
            }
            _ => {}
        }

        // 访问子表达式
        for child in expr.children() {
            self.visit_expression(child);
        }

        self.state.decrement_depth();
    }

    fn visit_literal(&mut self, _value: &crate::core::Value) -> Self::Result {
        // 字面量不需要处理
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.state.collected_variables.insert(name.to_string());
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        if let Expression::Variable(var_name) = object {
            self.tracker.track_property(var_name, property);
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
        for (condition, expr) in conditions {
            self.visit_expression(condition);
            self.visit_expression(expr);
        }
        if let Some(default_expr) = default {
            self.visit_expression(default_expr);
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
        // 标签不需要处理
    }

    fn visit_tag_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        self.tracker.track_property(tag, prop);
    }

    fn visit_edge_property(&mut self, edge: &str, prop: &str) -> Self::Result {
        self.tracker.track_property(edge, prop);
    }

    fn visit_input_property(&mut self, prop: &str) -> Self::Result {
        self.tracker.track_property("$-", prop);
    }

    fn visit_variable_property(&mut self, var: &str, prop: &str) -> Self::Result {
        self.tracker.track_property(var, prop);
    }

    fn visit_source_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        self.tracker.track_property(&format!("^{}", tag), prop);
    }

    fn visit_destination_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        self.tracker.track_property(&format!("${}", tag), prop);
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

    fn visit_list_comprehension(
        &mut self,
        generator: &Expression,
        condition: &Option<Box<Expression>>,
    ) -> Self::Result {
        self.visit_expression(generator);
        if let Some(cond) = condition {
            self.visit_expression(cond);
        }
    }

    fn visit_predicate(&mut self, list: &Expression, condition: &Expression) -> Self::Result {
        self.visit_expression(list);
        self.visit_expression(condition);
    }

    fn visit_reduce(
        &mut self,
        list: &Expression,
        _var: &str,
        initial: &Expression,
        expr: &Expression,
    ) -> Self::Result {
        self.visit_expression(list);
        self.visit_expression(initial);
        self.visit_expression(expr);
    }

    fn visit_es_query(&mut self, _query: &str) -> Self::Result {
        // ES 查询不需要处理
    }

    fn visit_uuid(&mut self) -> Self::Result {
        // UUID 不需要处理
    }

    fn visit_match_path_pattern(
        &mut self,
        _path_alias: &str,
        patterns: &[Expression],
    ) -> Self::Result {
        for pattern in patterns {
            self.visit_expression(pattern);
        }
    }

    fn visit_constant(&mut self, _e: &crate::query::parser::ast::expr::ConstantExpr) -> Self::Result {
        // 常量不需要处理
    }

    fn visit_variable_expr(&mut self, e: &crate::query::parser::ast::expr::VariableExpr) -> Self::Result {
        self.state.collected_variables.insert(e.name.clone());
    }

    fn visit_binary_expr(&mut self, e: &crate::query::parser::ast::expr::BinaryExpr) -> Self::Result {
        self.visit_expression(&e.left);
        self.visit_expression(&e.right);
    }

    fn visit_unary_expr(&mut self, e: &crate::query::parser::ast::expr::UnaryExpr) -> Self::Result {
        self.visit_expression(&e.operand);
    }

    fn visit_function_call_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::FunctionCallExpr,
    ) -> Self::Result {
        for arg in &e.args {
            self.visit_expression(arg);
        }
    }

    fn visit_property_access_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::PropertyAccessExpr,
    ) -> Self::Result {
        self.visit_expression(&e.object);
    }

    fn visit_list_expr(&mut self, e: &crate::query::parser::ast::expr::ListExpr) -> Self::Result {
        for item in &e.items {
            self.visit_expression(item);
        }
    }

    fn visit_map_expr(&mut self, e: &crate::query::parser::ast::expr::MapExpr) -> Self::Result {
        for (_, value) in &e.pairs {
            self.visit_expression(value);
        }
    }

    fn visit_case_expr(&mut self, e: &crate::query::parser::ast::expr::CaseExpr) -> Self::Result {
        for (condition, expr) in &e.cases {
            self.visit_expression(condition);
            self.visit_expression(expr);
        }
        if let Some(default_expr) = &e.default {
            self.visit_expression(default_expr);
        }
    }

    fn visit_subscript_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::SubscriptExpr,
    ) -> Self::Result {
        self.visit_expression(&e.collection);
        self.visit_expression(&e.index);
    }

    fn visit_predicate_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::PredicateExpr,
    ) -> Self::Result {
        self.visit_expression(&e.list);
        self.visit_expression(&e.condition);
    }

    fn visit_tag_property_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::TagPropertyExpr,
    ) -> Self::Result {
        self.tracker.track_property(&e.tag, &e.prop);
    }

    fn visit_edge_property_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::EdgePropertyExpr,
    ) -> Self::Result {
        self.tracker.track_property(&e.edge, &e.prop);
    }

    fn visit_input_property_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::InputPropertyExpr,
    ) -> Self::Result {
        self.tracker.track_property("$-", &e.prop);
    }

    fn visit_variable_property_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::VariablePropertyExpr,
    ) -> Self::Result {
        self.tracker.track_property(&e.var, &e.prop);
    }

    fn visit_source_property_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::SourcePropertyExpr,
    ) -> Self::Result {
        self.tracker.track_property(&format!("^{}", e.tag), &e.prop);
    }

    fn visit_destination_property_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::DestinationPropertyExpr,
    ) -> Self::Result {
        self.tracker.track_property(&format!("${}", e.tag), &e.prop);
    }

    fn visit_type_cast_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::TypeCastExpr,
    ) -> Self::Result {
        self.visit_expression(&e.expr);
    }

    fn visit_range_expr(&mut self, e: &crate::query::parser::ast::expr::RangeExpr) -> Self::Result {
        self.visit_expression(&e.collection);
        if let Some(start) = &e.start {
            self.visit_expression(start);
        }
        if let Some(end) = &e.end {
            self.visit_expression(end);
        }
    }

    fn visit_path_expr(&mut self, e: &crate::query::parser::ast::expr::PathExpr) -> Self::Result {
        for item in &e.items {
            self.visit_expression(item);
        }
    }

    fn visit_label_expr(&mut self, e: &crate::query::parser::ast::expr::LabelExpr) -> Self::Result {
        // 标签不需要处理
    }

    fn visit_reduce_expr(&mut self, e: &crate::query::parser::ast::expr::ReduceExpr) -> Self::Result {
        self.visit_expression(&e.list);
        self.visit_expression(&e.initial);
        self.visit_expression(&e.expr);
    }

    fn visit_list_comprehension_expr(
        &mut self,
        e: &crate::query::parser::ast::expr::ListComprehensionExpr,
    ) -> Self::Result {
        self.visit_expression(&e.generator);
        if let Some(condition) = &e.condition {
            self.visit_expression(condition);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Value;

    #[test]
    fn test_prune_properties_visitor() {
        let tracker = PropertyTracker::new();
        let mut visitor = PrunePropertiesVisitor::new(tracker);

        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("v".to_string())),
            op: crate::core::types::operators::BinaryOperator::Equal,
            right: Box::new(Expression::Literal(Value::Int(1))),
        };

        visitor.collect_properties(&expr);

        assert!(visitor.get_collected_properties().contains("v"));
        assert_eq!(visitor.state().visit_count, 1);
    }

    #[test]
    fn test_property_tracking() {
        let tracker = PropertyTracker::new();
        let mut visitor = PrunePropertiesVisitor::new(tracker);

        let expr = Expression::Property {
            object: Box::new(Expression::Variable("v".to_string())),
            property: "name".to_string(),
        };

        visitor.collect_properties(&expr);

        assert!(visitor.tracker().is_property_used("v", "name"));
    }

    #[test]
    fn test_max_depth() {
        let tracker = PropertyTracker::new();
        let mut visitor = PrunePropertiesVisitor::new(tracker);
        visitor.state_mut().set_max_depth(2);

        let expr = Expression::Binary {
            left: Box::new(Expression::Binary {
                left: Box::new(Expression::Binary {
                    left: Box::new(Expression::Literal(Value::Int(1))),
                    op: crate::core::types::operators::BinaryOperator::Add,
                    right: Box::new(Expression::Literal(Value::Int(2))),
                }),
                op: crate::core::types::operators::BinaryOperator::Add,
                right: Box::new(Expression::Literal(Value::Int(3))),
            }),
            op: crate::core::types::operators::BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(4))),
        };

        visitor.collect_properties(&expr);

        // 由于深度限制，应该只访问部分表达式
        assert!(visitor.state().depth > 0);
    }
}
