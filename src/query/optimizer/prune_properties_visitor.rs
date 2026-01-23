//! 属性修剪访问者
//!
//! 用于修剪未使用的属性，优化查询计划

use crate::core::expression_visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::types::expression::Expr;
use crate::core::Expression;
use crate::query::optimizer::property_tracker::PropertyTracker;
use std::collections::HashSet;

/// 属性修剪访问者
///
/// 遍历表达式树，收集使用的属性
#[derive(Debug)]
pub struct PrunePropertiesVisitor {
    /// 属性跟踪器
    tracker: PropertyTracker,
    /// 访问状态
    state: ExpressionVisitorState,
}

impl PrunePropertiesVisitor {
    /// 创建新的属性修剪访问者
    pub fn new(tracker: PropertyTracker) -> Self {
        Self {
            tracker,
            state: ExpressionVisitorState::new(),
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

    /// 收集表达式中的所有属性
    pub fn collect_properties(&mut self, expr: &Expr) {
        self.visit_expression(expr);
    }

    /// 获取收集的属性
    pub fn get_collected_properties(&self) -> HashSet<String> {
        self.state.get_custom_data("collected_variables")
            .and_then(|v| {
                if let crate::core::Value::List(vars) = v {
                    Some(vars.iter().filter_map(|v| {
                        if let crate::core::Value::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    }).collect::<HashSet<_>>())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    /// 获取特定变量的使用属性
    pub fn get_used_properties(&self, var: &str) -> Option<&HashSet<String>> {
        self.tracker.get_used_properties(var)
    }

    /// 获取最大达到的深度
    pub fn get_max_depth_reached(&self) -> usize {
        self.state.get_max_depth_reached()
    }
}

impl ExpressionVisitor for PrunePropertiesVisitor {
    type Result = ();

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_expression(&mut self, expr: &Expr) -> Self::Result {
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
            Expr::Variable(name) => {
                let mut vars = self.state.get_custom_data("collected_variables")
                    .and_then(|v| {
                        if let crate::core::Value::List(vars) = v {
                            Some(vars.iter().filter_map(|v| {
                                if let crate::core::Value::String(s) = v {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            }).collect::<Vec<String>>())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                if !vars.contains(name) {
                    vars.push(name.clone());
                }
                let vars_value: Vec<crate::core::Value> = vars.into_iter().map(|s| crate::core::Value::String(s)).collect();
                self.state.set_custom_data("collected_variables".to_string(), crate::core::Value::List(vars_value));
            }
            Expr::Property { object, property } => {
                if let Expr::Variable(var_name) = object.as_ref() {
                    self.tracker.track_property(var_name, property);
                }
            }
            Expr::Property { object, property } => {
                if let Expr::Variable(var_name) = object.as_ref() {
                    self.tracker.track_property(var_name, property);
                } else {
                    self.visit_expression(object.as_ref());
                }
            }
            _ => {}
        }

        for child in Expr::children(expr) {
            self.visit_expression(child);
        }

        self.state.decrement_depth();
    }

    fn visit_literal(&mut self, _value: &crate::core::Value) -> Self::Result {
        // 字面量不需要处理
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        let mut vars = self.state.get_custom_data("collected_variables")
            .and_then(|v| {
                if let crate::core::Value::List(vars) = v {
                    Some(vars.iter().filter_map(|v| {
                        if let crate::core::Value::String(s) = v {
                            Some(s.clone())
                        } else {
                            None
                        }
                    }).collect::<Vec<String>>())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        if !vars.contains(&name.to_string()) {
            vars.push(name.to_string());
        }
        let vars_value: Vec<crate::core::Value> = vars.into_iter().map(|s| crate::core::Value::String(s)).collect();
        self.state.set_custom_data("collected_variables".to_string(), crate::core::Value::List(vars_value));
    }

    fn visit_property(&mut self, object: &Expr, property: &str) -> Self::Result {
        if let Expr::Variable(var_name) = object {
            self.tracker.track_property(var_name, property);
        }
        self.visit_expression(object);
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

    fn visit_unary(
        &mut self,
        _op: &crate::core::types::operators::UnaryOperator,
        operand: &Expr,
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
        expr: &Expr,
        _target_type: &crate::core::types::expression::DataType,
    ) -> Self::Result {
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
        // 标签不需要处理
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

        let expr = Expr::Binary {
            left: Box::new(Expr::Variable("v".to_string())),
            op: crate::core::types::operators::BinaryOperator::Equal,
            right: Box::new(Expr::Literal(Value::Int(1))),
        };

        visitor.collect_properties(&expr);

        assert!(visitor.get_collected_properties().contains("v"));
        assert_eq!(visitor.state().visit_count, 3);
    }

    #[test]
    fn test_property_tracking() {
        let tracker = PropertyTracker::new();
        let mut visitor = PrunePropertiesVisitor::new(tracker);

        let expr = Expr::Property {
            object: Box::new(Expr::Variable("v".to_string())),
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

        let expr = Expr::Binary {
            left: Box::new(Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal(Value::Int(1))),
                    op: crate::core::types::operators::BinaryOperator::Add,
                    right: Box::new(Expr::Literal(Value::Int(2))),
                }),
                op: crate::core::types::operators::BinaryOperator::Add,
                right: Box::new(Expr::Literal(Value::Int(3))),
            }),
            op: crate::core::types::operators::BinaryOperator::Add,
            right: Box::new(Expr::Literal(Value::Int(4))),
        };

        visitor.collect_properties(&expr);
        let max_depth_reached = visitor.get_max_depth_reached();

        assert!(max_depth_reached >= 2);
    }
}
