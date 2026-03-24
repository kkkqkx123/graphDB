//! 表达式访问者 trait
//!
//! 本模块定义 ExpressionVisitor trait，用于遍历和分析表达式树。
//! 访问者模式可以避免重复的模式匹配代码，提高代码的可维护性和可扩展性。
//!
//! # 使用示例
//!
//! ```rust
//! use crate::core::types::expression::visitor::{ExpressionVisitor, PropertyCollector};
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

/// 表达式访问者 trait
///
/// 用于遍历和分析表达式树，避免重复的模式匹配代码。
/// 实现此 trait 可以创建自定义的表达式分析器。
///
/// # 示例
///
/// ```rust
/// use crate::core::types::expression::visitor::ExpressionVisitor;
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
///     // ... 其他方法
/// }
/// ```
pub trait ExpressionVisitor {
    /// 访问表达式
    ///
    /// 默认实现根据表达式类型分发到具体的访问方法。
    /// 子类型可以重写此方法以实现自定义的遍历逻辑。
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

    /// 访问字面量表达式
    fn visit_literal(&mut self, value: &Value);

    /// 访问变量表达式
    fn visit_variable(&mut self, name: &str);

    /// 访问属性表达式
    fn visit_property(&mut self, object: &Expression, property: &str);

    /// 访问二元运算表达式
    fn visit_binary(&mut self, op: BinaryOperator, left: &Expression, right: &Expression);

    /// 访问一元运算表达式
    fn visit_unary(&mut self, op: UnaryOperator, operand: &Expression);

    /// 访问函数调用表达式
    fn visit_function(&mut self, name: &str, args: &[Expression]);

    /// 访问聚合函数表达式
    fn visit_aggregate(&mut self, func: &AggregateFunction, arg: &Expression, distinct: bool);

    /// 访问条件表达式
    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    );

    /// 访问列表表达式
    fn visit_list(&mut self, items: &[Expression]);

    /// 访问映射表达式
    fn visit_map(&mut self, entries: &[(String, Expression)]);

    /// 访问类型转换表达式
    fn visit_type_cast(&mut self, expression: &Expression, target_type: &DataType);

    /// 访问下标访问表达式
    fn visit_subscript(&mut self, collection: &Expression, index: &Expression);

    /// 访问范围表达式
    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    );

    /// 访问路径表达式
    fn visit_path(&mut self, items: &[Expression]);

    /// 访问标签表达式
    fn visit_label(&mut self, label: &str);

    /// 访问列表推导表达式
    fn visit_list_comprehension(
        &mut self,
        variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    );

    /// 访问标签属性动态访问表达式
    fn visit_label_tag_property(&mut self, tag: &Expression, property: &str);

    /// 访问标签属性访问表达式
    fn visit_tag_property(&mut self, tag_name: &str, property: &str);

    /// 访问边属性访问表达式
    fn visit_edge_property(&mut self, edge_name: &str, property: &str);

    /// 访问谓词表达式
    fn visit_predicate(&mut self, func: &str, args: &[Expression]);

    /// 访问 Reduce 表达式
    fn visit_reduce(
        &mut self,
        accumulator: &str,
        initial: &Expression,
        variable: &str,
        source: &Expression,
        mapping: &Expression,
    );

    /// 访问路径构建表达式
    fn visit_path_build(&mut self, items: &[Expression]);

    /// 访问查询参数表达式
    fn visit_parameter(&mut self, name: &str);
}
