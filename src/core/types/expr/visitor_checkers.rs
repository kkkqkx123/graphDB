//! 表达式检查器
//!
//! 本模块提供各种表达式检查器的实现，用于检查表达式是否满足特定条件。
//!
//! # 可用的检查器
//!
//! - [`ConstantChecker`] - 检查表达式是否为常量表达式
//! - [`PropertyContainsChecker`] - 检查表达式是否包含指定的属性名
//! - [`WildcardReplacer`] - 替换表达式中的通配符变量
//! - [`AggregateFunctionChecker`] - 检查表达式是否包含聚合函数
//! - [`VariableContainsChecker`] - 检查表达式是否包含指定变量
//! - [`PathBuildContainsChecker`] - 检查表达式是否包含PathBuild

use crate::core::types::expression::visitor::ExpressionVisitor;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
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

    fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
        if self.is_constant {
            self.visit(left);
        }
        if self.is_constant {
            self.visit(right);
        }
    }

    fn visit_unary(&mut self, _op: UnaryOperator, operand: &Expression) {
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

    fn visit_aggregate(&mut self, _func: &AggregateFunction, arg: &Expression, _distinct: bool) {
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

    fn visit_type_cast(
        &mut self,
        expression: &Expression,
        _target_type: &crate::core::types::DataType,
    ) {
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

    fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
        if !self.contains {
            self.visit(left);
        }
        if !self.contains {
            self.visit(right);
        }
    }

    fn visit_unary(&mut self, _op: UnaryOperator, operand: &Expression) {
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

    fn visit_aggregate(&mut self, _func: &AggregateFunction, arg: &Expression, _distinct: bool) {
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

    fn visit_type_cast(
        &mut self,
        expression: &Expression,
        _target_type: &crate::core::types::DataType,
    ) {
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

/// 通配符替换器
///
/// 将表达式中的通配符变量（`*` 或 `_`）替换为具体的别名。
///
/// # 示例
///
/// ```rust
/// use crate::core::types::expression::visitor::WildcardReplacer;
/// use crate::core::Expression;
///
/// let expr = Expression::property("*", "name");
/// let mut replacer = WildcardReplacer::new("v");
/// let replaced = replacer.replace(&expr);
/// ```
#[derive(Debug)]
pub struct WildcardReplacer {
    /// 替换目标别名
    pub alias: String,
}

impl WildcardReplacer {
    /// 创建新的通配符替换器
    ///
    /// # 参数
    /// - `alias`: 用于替换通配符的别名
    pub fn new(alias: &str) -> Self {
        Self {
            alias: alias.to_string(),
        }
    }

    /// 替换表达式中的通配符
    ///
    /// # 参数
    /// - `expr`: 要替换的表达式
    ///
    /// # 返回
    /// 替换后的表达式
    pub fn replace(&self, expr: &Expression) -> Expression {
        self.replace_internal(expr)
    }

    fn replace_internal(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Literal(value) => Expression::Literal(value.clone()),
            Expression::Variable(name) => {
                if name == "*" || name == "_" {
                    Expression::Variable(self.alias.clone())
                } else {
                    Expression::Variable(name.clone())
                }
            }
            Expression::Property { object, property } => Expression::Property {
                object: Box::new(self.replace_internal(object)),
                property: property.clone(),
            },
            Expression::Binary { left, op, right } => Expression::Binary {
                left: Box::new(self.replace_internal(left)),
                op: *op,
                right: Box::new(self.replace_internal(right)),
            },
            Expression::Unary { op, operand } => Expression::Unary {
                op: *op,
                operand: Box::new(self.replace_internal(operand)),
            },
            Expression::Function { name, args } => Expression::Function {
                name: name.clone(),
                args: args.iter().map(|arg| self.replace_internal(arg)).collect(),
            },
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => Expression::Aggregate {
                func: func.clone(),
                arg: Box::new(self.replace_internal(arg)),
                distinct: *distinct,
            },
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => Expression::Case {
                test_expr: test_expr
                    .as_ref()
                    .map(|e| Box::new(self.replace_internal(e))),
                conditions: conditions
                    .iter()
                    .map(|(w, t)| (self.replace_internal(w), self.replace_internal(t)))
                    .collect(),
                default: default.as_ref().map(|e| Box::new(self.replace_internal(e))),
            },
            Expression::List(items) => Expression::List(
                items
                    .iter()
                    .map(|item| self.replace_internal(item))
                    .collect(),
            ),
            Expression::Map(entries) => Expression::Map(
                entries
                    .iter()
                    .map(|(k, v)| (k.clone(), self.replace_internal(v)))
                    .collect(),
            ),
            Expression::TypeCast {
                expression,
                target_type,
            } => Expression::TypeCast {
                expression: Box::new(self.replace_internal(expression)),
                target_type: target_type.clone(),
            },
            Expression::Subscript { collection, index } => Expression::Subscript {
                collection: Box::new(self.replace_internal(collection)),
                index: Box::new(self.replace_internal(index)),
            },
            Expression::Range {
                collection,
                start,
                end,
            } => Expression::Range {
                collection: Box::new(self.replace_internal(collection)),
                start: start.as_ref().map(|e| Box::new(self.replace_internal(e))),
                end: end.as_ref().map(|e| Box::new(self.replace_internal(e))),
            },
            Expression::Path(items) => Expression::Path(
                items
                    .iter()
                    .map(|item| self.replace_internal(item))
                    .collect(),
            ),
            Expression::Label(label) => Expression::Label(label.clone()),
            Expression::ListComprehension {
                variable,
                source,
                filter,
                map,
            } => Expression::ListComprehension {
                variable: variable.clone(),
                source: Box::new(self.replace_internal(source)),
                filter: filter.as_ref().map(|e| Box::new(self.replace_internal(e))),
                map: map.as_ref().map(|e| Box::new(self.replace_internal(e))),
            },
            Expression::LabelTagProperty { tag, property } => Expression::LabelTagProperty {
                tag: Box::new(self.replace_internal(tag)),
                property: property.clone(),
            },
            Expression::TagProperty { tag_name, property } => Expression::TagProperty {
                tag_name: tag_name.clone(),
                property: property.clone(),
            },
            Expression::EdgeProperty {
                edge_name,
                property,
            } => Expression::EdgeProperty {
                edge_name: edge_name.clone(),
                property: property.clone(),
            },
            Expression::Predicate { func, args } => Expression::Predicate {
                func: func.clone(),
                args: args.iter().map(|arg| self.replace_internal(arg)).collect(),
            },
            Expression::Reduce {
                accumulator,
                initial,
                variable,
                source,
                mapping,
            } => Expression::Reduce {
                accumulator: accumulator.clone(),
                initial: Box::new(self.replace_internal(initial)),
                variable: variable.clone(),
                source: Box::new(self.replace_internal(source)),
                mapping: Box::new(self.replace_internal(mapping)),
            },
            Expression::PathBuild(items) => Expression::PathBuild(
                items
                    .iter()
                    .map(|item| self.replace_internal(item))
                    .collect(),
            ),
            Expression::Parameter(name) => Expression::Parameter(name.clone()),
        }
    }
}

/// 聚合函数检查器
///
/// 检查表达式是否包含聚合函数。
///
/// # 示例
///
/// ```rust
/// use crate::core::types::expression::visitor::AggregateFunctionChecker;
/// use crate::core::Expression;
///
/// let expr = Expression::aggregate("count", Expression::variable("v"), false);
/// assert!(AggregateFunctionChecker::check(&expr));
///
/// let expr = Expression::variable("a");
/// assert!(!AggregateFunctionChecker::check(&expr));
/// ```
#[derive(Debug, Default)]
pub struct AggregateFunctionChecker {
    /// 是否包含聚合函数
    pub contains_aggregate: bool,
}

impl AggregateFunctionChecker {
    /// 创建新的聚合函数检查器
    pub fn new() -> Self {
        Self {
            contains_aggregate: false,
        }
    }

    /// 检查表达式是否包含聚合函数
    ///
    /// # 参数
    /// - `expr`: 要检查的表达式
    ///
    /// # 返回
    /// - `true`: 表达式包含聚合函数
    /// - `false`: 表达式不包含聚合函数
    pub fn check(expr: &Expression) -> bool {
        let mut checker = Self::new();
        checker.visit(expr);
        checker.contains_aggregate
    }
}

impl ExpressionVisitor for AggregateFunctionChecker {
    fn visit_literal(&mut self, _value: &crate::core::Value) {}

    fn visit_variable(&mut self, _name: &str) {}

    fn visit_property(&mut self, object: &Expression, _property: &str) {
        if !self.contains_aggregate {
            self.visit(object);
        }
    }

    fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
        if !self.contains_aggregate {
            self.visit(left);
        }
        if !self.contains_aggregate {
            self.visit(right);
        }
    }

    fn visit_unary(&mut self, _op: UnaryOperator, operand: &Expression) {
        if !self.contains_aggregate {
            self.visit(operand);
        }
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) {
        if !self.contains_aggregate {
            for arg in args {
                self.visit(arg);
                if self.contains_aggregate {
                    break;
                }
            }
        }
    }

    fn visit_aggregate(&mut self, _func: &AggregateFunction, _arg: &Expression, _distinct: bool) {
        self.contains_aggregate = true;
    }

    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) {
        if !self.contains_aggregate {
            if let Some(test) = test_expr {
                self.visit(test);
                if self.contains_aggregate {
                    return;
                }
            }
            for (when, then) in conditions {
                self.visit(when);
                if self.contains_aggregate {
                    return;
                }
                self.visit(then);
                if self.contains_aggregate {
                    return;
                }
            }
            if let Some(default_expr) = default {
                self.visit(default_expr);
            }
        }
    }

    fn visit_list(&mut self, items: &[Expression]) {
        if !self.contains_aggregate {
            for item in items {
                self.visit(item);
                if self.contains_aggregate {
                    break;
                }
            }
        }
    }

    fn visit_map(&mut self, entries: &[(String, Expression)]) {
        if !self.contains_aggregate {
            for (_, value) in entries {
                self.visit(value);
                if self.contains_aggregate {
                    break;
                }
            }
        }
    }

    fn visit_type_cast(
        &mut self,
        expression: &Expression,
        _target_type: &crate::core::types::DataType,
    ) {
        if !self.contains_aggregate {
            self.visit(expression);
        }
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        if !self.contains_aggregate {
            self.visit(collection);
            if !self.contains_aggregate {
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
        if !self.contains_aggregate {
            self.visit(collection);
            if !self.contains_aggregate {
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                    if self.contains_aggregate {
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
        if !self.contains_aggregate {
            for item in items {
                self.visit(item);
                if self.contains_aggregate {
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
        if !self.contains_aggregate {
            self.visit(source);
            if !self.contains_aggregate {
                if let Some(filter_expr) = filter {
                    self.visit(filter_expr);
                    if self.contains_aggregate {
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
        if !self.contains_aggregate {
            self.visit(tag);
        }
    }

    fn visit_tag_property(&mut self, _tag_name: &str, _property: &str) {}

    fn visit_edge_property(&mut self, _edge_name: &str, _property: &str) {}

    fn visit_predicate(&mut self, _func: &str, args: &[Expression]) {
        if !self.contains_aggregate {
            for arg in args {
                self.visit(arg);
                if self.contains_aggregate {
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
        if !self.contains_aggregate {
            self.visit(initial);
            if !self.contains_aggregate {
                self.visit(source);
                if !self.contains_aggregate {
                    self.visit(mapping);
                }
            }
        }
    }

    fn visit_path_build(&mut self, items: &[Expression]) {
        if !self.contains_aggregate {
            for item in items {
                self.visit(item);
                if self.contains_aggregate {
                    break;
                }
            }
        }
    }

    fn visit_parameter(&mut self, _name: &str) {}
}

/// 变量包含检查器
///
/// 检查表达式是否包含指定的变量名。
///
/// # 示例
///
/// ```rust
/// use crate::core::types::expression::visitor::VariableContainsChecker;
/// use crate::core::Expression;
///
/// let expr = Expression::property("a", "name");
/// assert!(VariableContainsChecker::check(&expr, "a"));
///
/// assert!(!VariableContainsChecker::check(&expr, "b"));
/// ```
#[derive(Debug)]
pub struct VariableContainsChecker {
    /// 要检查的变量名
    pub variable_name: String,
    /// 是否包含指定的变量
    pub contains: bool,
}

impl VariableContainsChecker {
    /// 创建新的变量包含检查器
    ///
    /// # 参数
    /// - `variable_name`: 要检查的变量名
    pub fn new(variable_name: &str) -> Self {
        Self {
            variable_name: variable_name.to_string(),
            contains: false,
        }
    }

    /// 检查表达式是否包含指定的变量名
    ///
    /// # 参数
    /// - `expr`: 要检查的表达式
    /// - `variable_name`: 要检查的变量名
    ///
    /// # 返回
    /// - `true`: 表达式包含指定的变量
    /// - `false`: 表达式不包含指定的变量
    pub fn check(expr: &Expression, variable_name: &str) -> bool {
        let mut checker = Self::new(variable_name);
        checker.visit(expr);
        checker.contains
    }
}

impl ExpressionVisitor for VariableContainsChecker {
    fn visit_literal(&mut self, _value: &crate::core::Value) {}

    fn visit_variable(&mut self, name: &str) {
        if name == self.variable_name {
            self.contains = true;
        }
    }

    fn visit_property(&mut self, object: &Expression, _property: &str) {
        if !self.contains {
            self.visit(object);
        }
    }

    fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
        if !self.contains {
            self.visit(left);
        }
        if !self.contains {
            self.visit(right);
        }
    }

    fn visit_unary(&mut self, _op: UnaryOperator, operand: &Expression) {
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

    fn visit_aggregate(&mut self, _func: &AggregateFunction, arg: &Expression, _distinct: bool) {
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

    fn visit_type_cast(
        &mut self,
        expression: &Expression,
        _target_type: &crate::core::types::DataType,
    ) {
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

    fn visit_tag_property(&mut self, _tag_name: &str, _property: &str) {}

    fn visit_edge_property(&mut self, _edge_name: &str, _property: &str) {}

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

/// PathBuild包含检查器
///
/// 检查表达式是否包含PathBuild表达式。
///
/// # 示例
///
/// ```rust
/// use crate::core::types::expression::visitor::PathBuildContainsChecker;
/// use crate::core::Expression;
///
/// let expr = Expression::path_build(vec![Expression::variable("a")]);
/// assert!(PathBuildContainsChecker::check(&expr));
///
/// let expr = Expression::variable("a");
/// assert!(!PathBuildContainsChecker::check(&expr));
/// ```
#[derive(Debug, Default)]
pub struct PathBuildContainsChecker {
    /// 是否包含PathBuild
    pub contains_path_build: bool,
}

impl PathBuildContainsChecker {
    /// 创建新的PathBuild包含检查器
    pub fn new() -> Self {
        Self {
            contains_path_build: false,
        }
    }

    /// 检查表达式是否包含PathBuild
    ///
    /// # 参数
    /// - `expr`: 要检查的表达式
    ///
    /// # 返回
    /// - `true`: 表达式包含PathBuild
    /// - `false`: 表达式不包含PathBuild
    pub fn check(expr: &Expression) -> bool {
        let mut checker = Self::new();
        checker.visit(expr);
        checker.contains_path_build
    }
}

impl ExpressionVisitor for PathBuildContainsChecker {
    fn visit_literal(&mut self, _value: &crate::core::Value) {}

    fn visit_variable(&mut self, _name: &str) {}

    fn visit_property(&mut self, object: &Expression, _property: &str) {
        if !self.contains_path_build {
            self.visit(object);
        }
    }

    fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
        if !self.contains_path_build {
            self.visit(left);
        }
        if !self.contains_path_build {
            self.visit(right);
        }
    }

    fn visit_unary(&mut self, _op: UnaryOperator, operand: &Expression) {
        if !self.contains_path_build {
            self.visit(operand);
        }
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) {
        if !self.contains_path_build {
            for arg in args {
                self.visit(arg);
                if self.contains_path_build {
                    break;
                }
            }
        }
    }

    fn visit_aggregate(&mut self, _func: &AggregateFunction, arg: &Expression, _distinct: bool) {
        if !self.contains_path_build {
            self.visit(arg);
        }
    }

    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) {
        if !self.contains_path_build {
            if let Some(test) = test_expr {
                self.visit(test);
                if self.contains_path_build {
                    return;
                }
            }
            for (when, then) in conditions {
                self.visit(when);
                if self.contains_path_build {
                    return;
                }
                self.visit(then);
                if self.contains_path_build {
                    return;
                }
            }
            if let Some(default_expr) = default {
                self.visit(default_expr);
            }
        }
    }

    fn visit_list(&mut self, items: &[Expression]) {
        if !self.contains_path_build {
            for item in items {
                self.visit(item);
                if self.contains_path_build {
                    break;
                }
            }
        }
    }

    fn visit_map(&mut self, entries: &[(String, Expression)]) {
        if !self.contains_path_build {
            for (_, value) in entries {
                self.visit(value);
                if self.contains_path_build {
                    break;
                }
            }
        }
    }

    fn visit_type_cast(
        &mut self,
        expression: &Expression,
        _target_type: &crate::core::types::DataType,
    ) {
        if !self.contains_path_build {
            self.visit(expression);
        }
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        if !self.contains_path_build {
            self.visit(collection);
            if !self.contains_path_build {
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
        if !self.contains_path_build {
            self.visit(collection);
            if !self.contains_path_build {
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                    if self.contains_path_build {
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
        if !self.contains_path_build {
            for item in items {
                self.visit(item);
                if self.contains_path_build {
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
        if !self.contains_path_build {
            self.visit(source);
            if !self.contains_path_build {
                if let Some(filter_expr) = filter {
                    self.visit(filter_expr);
                    if self.contains_path_build {
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
        if !self.contains_path_build {
            self.visit(tag);
        }
    }

    fn visit_tag_property(&mut self, _tag_name: &str, _property: &str) {}

    fn visit_edge_property(&mut self, _edge_name: &str, _property: &str) {}

    fn visit_predicate(&mut self, _func: &str, args: &[Expression]) {
        if !self.contains_path_build {
            for arg in args {
                self.visit(arg);
                if self.contains_path_build {
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
        if !self.contains_path_build {
            self.visit(initial);
            if !self.contains_path_build {
                self.visit(source);
                if !self.contains_path_build {
                    self.visit(mapping);
                }
            }
        }
    }

    fn visit_path_build(&mut self, _items: &[Expression]) {
        self.contains_path_build = true;
    }

    fn visit_parameter(&mut self, _name: &str) {}
}
