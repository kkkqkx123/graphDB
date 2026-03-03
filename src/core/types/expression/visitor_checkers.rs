//! 表达式检查器
//!
//! 本模块提供各种表达式检查器的实现，用于检查表达式是否满足特定条件。
//!
//! # 可用的检查器
//!
//! - [`ConstantChecker`] - 检查表达式是否为常量表达式
//! - [`PropertyContainsChecker`] - 检查表达式是否包含指定的属性名

use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::types::expression::visitor::ExpressionVisitor;
use crate::core::Expression;

/// 常量检查器
///
/// 检查表达式是否为常量表达式（不包含变量或属性）。
///
/// # 示例
///
/// ```rust
/// use crate::core::types::expression::visitor::ConstantChecker;
/// use crate::core::Expression;
///
/// let expr = Expression::literal(42);
/// assert!(ConstantChecker::check(&expr));
///
/// let expr = Expression::variable("a");
/// assert!(!ConstantChecker::check(&expr));
/// ```
#[derive(Debug, Default)]
pub struct ConstantChecker {
    /// 是否为常量表达式
    pub is_constant: bool,
}

impl ConstantChecker {
    /// 创建新的常量检查器
    pub fn new() -> Self {
        Self { is_constant: true }
    }

    /// 检查表达式是否为常量表达式
    ///
    /// # 参数
    /// - `expr`: 要检查的表达式
    ///
    /// # 返回
    /// - `true`: 表达式是常量表达式
    /// - `false`: 表达式包含变量或属性
    pub fn check(expr: &Expression) -> bool {
        let mut checker = Self::new();
        checker.visit(expr);
        checker.is_constant
    }
}

impl ExpressionVisitor for ConstantChecker {
    fn visit_literal(&mut self, _value: &crate::core::Value) {}

    fn visit_variable(&mut self, _name: &str) {
        self.is_constant = false;
    }

    fn visit_property(&mut self, _object: &Expression, _property: &str) {
        self.is_constant = false;
    }

    fn visit_binary(
        &mut self,
        _op: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) {
        if self.is_constant {
            self.visit(left);
        }
        if self.is_constant {
            self.visit(right);
        }
    }

    fn visit_unary(
        &mut self,
        _op: UnaryOperator,
        operand: &Expression,
    ) {
        if self.is_constant {
            self.visit(operand);
        }
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) {
        if self.is_constant {
            for arg in args {
                self.visit(arg);
                if !self.is_constant {
                    break;
                }
            }
        }
    }

    fn visit_aggregate(
        &mut self,
        _func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) {
        if self.is_constant {
            self.visit(arg);
        }
    }

    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) {
        if self.is_constant {
            if let Some(test) = test_expr {
                self.visit(test);
                if !self.is_constant {
                    return;
                }
            }
            for (when, then) in conditions {
                self.visit(when);
                if !self.is_constant {
                    return;
                }
                self.visit(then);
                if !self.is_constant {
                    return;
                }
            }
            if let Some(default_expr) = default {
                self.visit(default_expr);
            }
        }
    }

    fn visit_list(&mut self, items: &[Expression]) {
        if self.is_constant {
            for item in items {
                self.visit(item);
                if !self.is_constant {
                    break;
                }
            }
        }
    }

    fn visit_map(&mut self, entries: &[(String, Expression)]) {
        if self.is_constant {
            for (_, value) in entries {
                self.visit(value);
                if !self.is_constant {
                    break;
                }
            }
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: &crate::core::types::DataType) {
        if self.is_constant {
            self.visit(expression);
        }
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        if self.is_constant {
            self.visit(collection);
            if self.is_constant {
                self.visit(index);
            }
        }
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) {
        if self.is_constant {
            self.visit(collection);
            if self.is_constant {
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                    if !self.is_constant {
                        return;
                    }
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr);
                }
            }
        }
    }

    fn visit_path(&mut self, items: &[Expression]) {
        if self.is_constant {
            for item in items {
                self.visit(item);
                if !self.is_constant {
                    break;
                }
            }
        }
    }

    fn visit_label(&mut self, _label: &str) {}

    fn visit_list_comprehension(
        &mut self,
        _variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) {
        if self.is_constant {
            self.visit(source);
            if self.is_constant {
                if let Some(filter_expr) = filter {
                    self.visit(filter_expr);
                    if !self.is_constant {
                        return;
                    }
                }
                if let Some(map_expr) = map {
                    self.visit(map_expr);
                }
            }
        }
    }

    fn visit_label_tag_property(&mut self, tag: &Expression, _property: &str) {
        if self.is_constant {
            self.visit(tag);
        }
    }

    fn visit_tag_property(&mut self, _tag_name: &str, _property: &str) {
        self.is_constant = false;
    }

    fn visit_edge_property(&mut self, _edge_name: &str, _property: &str) {
        self.is_constant = false;
    }

    fn visit_predicate(&mut self, _func: &str, args: &[Expression]) {
        if self.is_constant {
            for arg in args {
                self.visit(arg);
                if !self.is_constant {
                    break;
                }
            }
        }
    }

    fn visit_reduce(
        &mut self,
        _accumulator: &str,
        initial: &Expression,
        _variable: &str,
        source: &Expression,
        mapping: &Expression,
    ) {
        if self.is_constant {
            self.visit(initial);
            if self.is_constant {
                self.visit(source);
                if self.is_constant {
                    self.visit(mapping);
                }
            }
        }
    }

    fn visit_path_build(&mut self, items: &[Expression]) {
        if self.is_constant {
            for item in items {
                self.visit(item);
                if !self.is_constant {
                    break;
                }
            }
        }
    }

    fn visit_parameter(&mut self, _name: &str) {}
}

/// 属性包含检查器
///
/// 检查表达式是否包含指定的属性名。
///
/// # 示例
///
/// ```rust
/// use crate::core::types::expression::visitor::PropertyContainsChecker;
/// use crate::core::Expression;
///
/// let expr = Expression::property("a", "name");
/// assert!(PropertyContainsChecker::check(&expr, &["name".to_string()]));
///
/// assert!(!PropertyContainsChecker::check(&expr, &["age".to_string()]));
/// ```
#[derive(Debug)]
pub struct PropertyContainsChecker {
    /// 要检查的属性名列表
    pub property_names: Vec<String>,
    /// 是否包含指定的属性
    pub contains: bool,
}

impl PropertyContainsChecker {
    /// 创建新的属性包含检查器
    ///
    /// # 参数
    /// - `property_names`: 要检查的属性名列表
    pub fn new(property_names: Vec<String>) -> Self {
        Self {
            property_names,
            contains: false,
        }
    }

    /// 检查表达式是否包含指定的属性名
    ///
    /// # 参数
    /// - `expr`: 要检查的表达式
    /// - `property_names`: 要检查的属性名列表
    ///
    /// # 返回
    /// - `true`: 表达式包含指定的属性
    /// - `false`: 表达式不包含指定的属性
    pub fn check(expr: &Expression, property_names: &[String]) -> bool {
        let mut checker = Self::new(property_names.to_vec());
        checker.visit(expr);
        checker.contains
    }
}

impl ExpressionVisitor for PropertyContainsChecker {
    fn visit_literal(&mut self, _value: &crate::core::Value) {}

    fn visit_variable(&mut self, _name: &str) {}

    fn visit_property(&mut self, _object: &Expression, property: &str) {
        if self.property_names.contains(&property.to_string()) {
            self.contains = true;
        }
    }

    fn visit_binary(
        &mut self,
        _op: BinaryOperator,
        left: &Expression,
        right: &Expression,
    ) {
        if !self.contains {
            self.visit(left);
        }
        if !self.contains {
            self.visit(right);
        }
    }

    fn visit_unary(
        &mut self,
        _op: UnaryOperator,
        operand: &Expression,
    ) {
        if !self.contains {
            self.visit(operand);
        }
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) {
        if !self.contains {
            for arg in args {
                self.visit(arg);
                if self.contains {
                    break;
                }
            }
        }
    }

    fn visit_aggregate(
        &mut self,
        _func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) {
        if !self.contains {
            self.visit(arg);
        }
    }

    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) {
        if !self.contains {
            if let Some(test) = test_expr {
                self.visit(test);
                if self.contains {
                    return;
                }
            }
            for (when, then) in conditions {
                self.visit(when);
                if self.contains {
                    return;
                }
                self.visit(then);
                if self.contains {
                    return;
                }
            }
            if let Some(default_expr) = default {
                self.visit(default_expr);
            }
        }
    }

    fn visit_list(&mut self, items: &[Expression]) {
        if !self.contains {
            for item in items {
                self.visit(item);
                if self.contains {
                    break;
                }
            }
        }
    }

    fn visit_map(&mut self, entries: &[(String, Expression)]) {
        if !self.contains {
            for (_, value) in entries {
                self.visit(value);
                if self.contains {
                    break;
                }
            }
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: &crate::core::types::DataType) {
        if !self.contains {
            self.visit(expression);
        }
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        if !self.contains {
            self.visit(collection);
            if !self.contains {
                self.visit(index);
            }
        }
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) {
        if !self.contains {
            self.visit(collection);
            if !self.contains {
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                    if self.contains {
                        return;
                    }
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr);
                }
            }
        }
    }

    fn visit_path(&mut self, items: &[Expression]) {
        if !self.contains {
            for item in items {
                self.visit(item);
                if self.contains {
                    break;
                }
            }
        }
    }

    fn visit_label(&mut self, _label: &str) {}

    fn visit_list_comprehension(
        &mut self,
        _variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) {
        if !self.contains {
            self.visit(source);
            if !self.contains {
                if let Some(filter_expr) = filter {
                    self.visit(filter_expr);
                    if self.contains {
                        return;
                    }
                }
                if let Some(map_expr) = map {
                    self.visit(map_expr);
                }
            }
        }
    }

    fn visit_label_tag_property(&mut self, tag: &Expression, _property: &str) {
        if !self.contains {
            self.visit(tag);
        }
    }

    fn visit_tag_property(&mut self, _tag_name: &str, property: &str) {
        if self.property_names.contains(&property.to_string()) {
            self.contains = true;
        }
    }

    fn visit_edge_property(&mut self, _edge_name: &str, property: &str) {
        if self.property_names.contains(&property.to_string()) {
            self.contains = true;
        }
    }

    fn visit_predicate(&mut self, _func: &str, args: &[Expression]) {
        if !self.contains {
            for arg in args {
                self.visit(arg);
                if self.contains {
                    break;
                }
            }
        }
    }

    fn visit_reduce(
        &mut self,
        _accumulator: &str,
        initial: &Expression,
        _variable: &str,
        source: &Expression,
        mapping: &Expression,
    ) {
        if !self.contains {
            self.visit(initial);
            if !self.contains {
                self.visit(source);
                if !self.contains {
                    self.visit(mapping);
                }
            }
        }
    }

    fn visit_path_build(&mut self, items: &[Expression]) {
        if !self.contains {
            for item in items {
                self.visit(item);
                if self.contains {
                    break;
                }
            }
        }
    }

    fn visit_parameter(&mut self, _name: &str) {}
}
