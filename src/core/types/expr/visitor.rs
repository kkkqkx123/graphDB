//! Expression Visitor trait
//!
//! This module defines ExpressionVisitor traits for traversing and analyzing expression trees.
//! Visitor patterns avoid duplicate pattern matching code and improve code maintainability and extensibility.
//!
//! # Examples of use
//!
//! ```rust
//! use crate::core::types::expr::visitor::{ExpressionVisitor, PropertyCollector};
//!
//! let expr = Expression::property("a", "name");
//! let mut collector = PropertyCollector::new();
//! collector.visit(&expr);
//! assert_eq!(collector.properties, vec!["name".to_string()]);
//! ```

use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::types::DataType;
use crate::core::Expression;
use crate::core::Value;

/// Expression Visitor trait
///
/// Used to traverse and analyze the expression tree to avoid repetitive pattern matching code.
/// Implementing this trait creates a custom expression parser.
///
/// # Examples
///
/// ```rust
/// use crate::core::types::expr::visitor::ExpressionVisitor;
///
/// struct MyVisitor {
///     count: usize,
/// }
///
/// impl ExpressionVisitor for MyVisitor {
///     fn visit_literal(&mut self, _value: &Value) {
///         self.count += 1;
///     }
///
///     fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
///         self.visit(left);
///         self.visit(right);
///     }
///
// ... Other methods
/// }
/// ```
pub trait ExpressionVisitor {
    /// access expression
    ///
    /// The default implementation distributes to specific access methods based on expression type.
    /// Subtypes can override this method to implement custom traversal logic.
    fn visit(&mut self, expr: &Expression) {
        match expr {
            Expression::Literal(value) => self.visit_literal(value),
            Expression::Variable(name) => self.visit_variable(name),
            Expression::Property { object, property } => {
                self.visit_property(object, property);
            }
            Expression::Binary { left, op, right } => {
                self.visit_binary(*op, left, right);
            }
            Expression::Unary { op, operand } => {
                self.visit_unary(*op, operand);
            }
            Expression::Function { name, args } => {
                self.visit_function(name, args);
            }
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                self.visit_aggregate(func, arg, *distinct);
            }
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => {
                self.visit_case(test_expr.as_deref(), conditions, default.as_deref());
            }
            Expression::List(items) => {
                self.visit_list(items);
            }
            Expression::Map(entries) => {
                self.visit_map(entries);
            }
            Expression::TypeCast {
                expression,
                target_type,
            } => {
                self.visit_type_cast(expression, target_type);
            }
            Expression::Subscript { collection, index } => {
                self.visit_subscript(collection, index);
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                self.visit_range(collection, start.as_deref(), end.as_deref());
            }
            Expression::Path(items) => {
                self.visit_path(items);
            }
            Expression::Label(label) => {
                self.visit_label(label);
            }
            Expression::ListComprehension {
                variable,
                source,
                filter,
                map,
            } => {
                self.visit_list_comprehension(variable, source, filter.as_deref(), map.as_deref());
            }
            Expression::LabelTagProperty { tag, property } => {
                self.visit_label_tag_property(tag, property);
            }
            Expression::TagProperty { tag_name, property } => {
                self.visit_tag_property(tag_name, property);
            }
            Expression::EdgeProperty {
                edge_name,
                property,
            } => {
                self.visit_edge_property(edge_name, property);
            }
            Expression::Predicate { func, args } => {
                self.visit_predicate(func, args);
            }
            Expression::Reduce {
                accumulator,
                initial,
                variable,
                source,
                mapping,
            } => {
                self.visit_reduce(accumulator, initial, variable, source, mapping);
            }
            Expression::PathBuild(items) => {
                self.visit_path_build(items);
            }
            Expression::Parameter(name) => {
                self.visit_parameter(name);
            }
        }
    }

    /// Accessing Literal Expressions
    fn visit_literal(&mut self, value: &Value);

    /// Accessing variable expressions
    fn visit_variable(&mut self, name: &str);

    /// Accessing Property Expressions
    fn visit_property(&mut self, object: &Expression, property: &str);

    /// Accessing binary arithmetic expressions
    fn visit_binary(&mut self, op: BinaryOperator, left: &Expression, right: &Expression);

    /// Accessing unary arithmetic expressions
    fn visit_unary(&mut self, op: UnaryOperator, operand: &Expression);

    /// Accessing function call expressions
    fn visit_function(&mut self, name: &str, args: &[Expression]);

    /// Accessing Aggregate Function Expressions
    fn visit_aggregate(&mut self, func: &AggregateFunction, arg: &Expression, distinct: bool);

    /// Accessing Conditional Expressions
    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    );

    /// Accessing List Expressions
    fn visit_list(&mut self, items: &[Expression]);

    /// Accessing mapping expressions
    fn visit_map(&mut self, entries: &[(String, Expression)]);

    /// Access to type conversion expressions
    fn visit_type_cast(&mut self, expression: &Expression, target_type: &DataType);

    /// Access subscript access expression
    fn visit_subscript(&mut self, collection: &Expression, index: &Expression);

    /// Access Range Expressions
    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    );

    /// Access path expression
    fn visit_path(&mut self, items: &[Expression]);

    /// Accessing tag expressions
    fn visit_label(&mut self, label: &str);

    /// Access List Derivation Expressions
    fn visit_list_comprehension(
        &mut self,
        variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    );

    /// Dynamic access expressions for accessing tag attributes
    fn visit_label_tag_property(&mut self, tag: &Expression, property: &str);

    /// Access Tag Attribute Access Expressions
    fn visit_tag_property(&mut self, tag_name: &str, property: &str);

    /// Accessing edge attribute access expressions
    fn visit_edge_property(&mut self, edge_name: &str, property: &str);

    /// Access predicate expressions
    fn visit_predicate(&mut self, func: &str, args: &[Expression]);

    /// Accessing Reduce Expressions
    fn visit_reduce(
        &mut self,
        accumulator: &str,
        initial: &Expression,
        variable: &str,
        source: &Expression,
        mapping: &Expression,
    );

    /// Access Path Construction Expressions
    fn visit_path_build(&mut self, items: &[Expression]);

    /// Accessing query parameter expressions
    fn visit_parameter(&mut self, name: &str);
}
