//! 表达式访问者模式实现
//!
//! 这个模块提供了表达式访问者模式的基础设施，专注于表达式树的遍历和转换

use crate::core::types::expression::{DataType, Expression, ExpressionType};
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;

/// 表达式访问者 trait - 用于访问Expression类型的各个变体
///
/// 这个trait定义了访问表达式树的标准接口，访问者可以针对不同类型的表达式
/// 实现特定的处理逻辑。
pub trait ExpressionVisitor {
    /// 访问者结果类型
    type Result;

    /// 访问表达式 - 入口方法
    fn visit(&mut self, expr: &Expression) -> Self::Result {
        match expr {
            Expression::Literal(value) => self.visit_literal(value),
            Expression::Variable(name) => self.visit_variable(name),
            Expression::Property { object, property } => self.visit_property(object, property),
            Expression::Binary { left, op, right } => self.visit_binary(left, op, right),
            Expression::Unary { op, operand } => self.visit_unary(op, operand),
            Expression::Function { name, args } => self.visit_function(name, args),
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => self.visit_aggregate(func, arg, *distinct),
            Expression::List(items) => self.visit_list(items),
            Expression::Map(pairs) => self.visit_map(pairs),
            Expression::Case {
                conditions,
                default,
            } => self.visit_case(conditions, default),
            Expression::TypeCast { expr, target_type } => self.visit_type_cast(expr, target_type),
            Expression::Subscript { collection, index } => self.visit_subscript(collection, index),
            Expression::Range {
                collection,
                start,
                end,
            } => self.visit_range(collection, start, end),
            Expression::Path(items) => self.visit_path(items),
            Expression::Label(name) => self.visit_label(name),
            Expression::TagProperty { tag, prop } => self.visit_tag_property(tag, prop),
            Expression::EdgeProperty { edge, prop } => self.visit_edge_property(edge, prop),
            Expression::InputProperty(prop) => self.visit_input_property(prop),
            Expression::VariableProperty { var, prop } => self.visit_variable_property(var, prop),
            Expression::SourceProperty { tag, prop } => self.visit_source_property(tag, prop),
            Expression::DestinationProperty { tag, prop } => {
                self.visit_destination_property(tag, prop)
            }
            Expression::UnaryPlus(expr) => self.visit_unary_plus(expr),
            Expression::UnaryNegate(expr) => self.visit_unary_negate(expr),
            Expression::UnaryNot(expr) => self.visit_unary_not(expr),
            Expression::UnaryIncr(expr) => self.visit_unary_incr(expr),
            Expression::UnaryDecr(expr) => self.visit_unary_decr(expr),
            Expression::IsNull(expr) => self.visit_is_null(expr),
            Expression::IsNotNull(expr) => self.visit_is_not_null(expr),
            Expression::IsEmpty(expr) => self.visit_is_empty(expr),
            Expression::IsNotEmpty(expr) => self.visit_is_not_empty(expr),
            Expression::ListComprehension {
                generator,
                condition,
            } => self.visit_list_comprehension(generator, condition),
            Expression::Predicate { list, condition } => self.visit_predicate(list, condition),
            Expression::Reduce {
                list,
                var,
                initial,
                expr,
            } => self.visit_reduce(list, var, initial, expr),
            Expression::ESQuery(query) => self.visit_es_query(query),
            Expression::UUID => self.visit_uuid(),
            Expression::MatchPathPattern {
                path_alias,
                patterns,
            } => self.visit_match_path_pattern(path_alias, patterns),
        }
    }

    fn visit_literal(&mut self, value: &Value) -> Self::Result;

    fn visit_variable(&mut self, name: &str) -> Self::Result;

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result;

    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result;

    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Self::Result;

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result;

    fn visit_aggregate(
        &mut self,
        func: &AggregateFunction,
        arg: &Expression,
        distinct: bool,
    ) -> Self::Result;

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result;

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result;

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result;

    fn visit_type_cast(&mut self, expr: &Expression, target_type: &DataType) -> Self::Result;

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result;

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result;

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result;

    fn visit_label(&mut self, name: &str) -> Self::Result;

    fn visit_tag_property(&mut self, tag: &str, prop: &str) -> Self::Result;

    fn visit_edge_property(&mut self, edge: &str, prop: &str) -> Self::Result;

    fn visit_input_property(&mut self, prop: &str) -> Self::Result;

    fn visit_variable_property(&mut self, var: &str, prop: &str) -> Self::Result;

    fn visit_source_property(&mut self, tag: &str, prop: &str) -> Self::Result;

    fn visit_destination_property(&mut self, tag: &str, prop: &str) -> Self::Result;

    fn visit_unary_plus(&mut self, expr: &Expression) -> Self::Result;

    fn visit_unary_negate(&mut self, expr: &Expression) -> Self::Result;

    fn visit_unary_not(&mut self, expr: &Expression) -> Self::Result;

    fn visit_unary_incr(&mut self, expr: &Expression) -> Self::Result;

    fn visit_unary_decr(&mut self, expr: &Expression) -> Self::Result;

    fn visit_is_null(&mut self, expr: &Expression) -> Self::Result;

    fn visit_is_not_null(&mut self, expr: &Expression) -> Self::Result;

    fn visit_is_empty(&mut self, expr: &Expression) -> Self::Result;

    fn visit_is_not_empty(&mut self, expr: &Expression) -> Self::Result;

    fn visit_type_casting(&mut self, expr: &Expression, target_type: &str) -> Self::Result;

    fn visit_list_comprehension(
        &mut self,
        generator: &Expression,
        condition: &Option<Box<Expression>>,
    ) -> Self::Result;

    fn visit_predicate(&mut self, list: &Expression, condition: &Expression) -> Self::Result;

    fn visit_reduce(
        &mut self,
        list: &Expression,
        var: &str,
        initial: &Expression,
        expr: &Expression,
    ) -> Self::Result;

    fn visit_path_build(&mut self, items: &[Expression]) -> Self::Result;

    fn visit_es_query(&mut self, query: &str) -> Self::Result;

    fn visit_uuid(&mut self) -> Self::Result;

    fn visit_subscript_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result;

    fn visit_match_path_pattern(
        &mut self,
        path_alias: &str,
        patterns: &[Expression],
    ) -> Self::Result;
}

/// 表达式访问者接受器 trait - 为Expression类型提供接受访问者的能力
///
/// 这个trait允许Expression类型接受访问者进行访问，实现了访问者模式的双向分发机制。
pub trait ExpressionAcceptor {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result;
}

impl ExpressionAcceptor for Expression {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit(self)
    }
}

/// 表达式类型过滤器 - 用于过滤特定类型的表达式
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionTypeFilter {
    target_types: std::collections::HashSet<ExpressionType>,
}

impl ExpressionTypeFilter {
    pub fn new() -> Self {
        Self {
            target_types: std::collections::HashSet::new(),
        }
    }

    pub fn with_types(mut self, types: &[ExpressionType]) -> Self {
        self.target_types.extend(types.iter().cloned());
        self
    }

    pub fn add_type(&mut self, expr_type: ExpressionType) {
        self.target_types.insert(expr_type);
    }

    pub fn remove_type(&mut self, expr_type: &ExpressionType) {
        self.target_types.remove(expr_type);
    }

    pub fn contains(&self, expr_type: &ExpressionType) -> bool {
        self.target_types.contains(expr_type)
    }

    pub fn is_empty(&self) -> bool {
        self.target_types.is_empty()
    }

    pub fn clear(&mut self) {
        self.target_types.clear();
    }
}

impl Default for ExpressionTypeFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// 表达式深度优先遍历器
///
/// 提供深度优先遍历表达式树的默认实现，访问者可以继承这个trait来获得
/// 标准的遍历行为。
pub trait ExpressionDepthFirstVisitor: ExpressionVisitor {
    fn visit_children(&mut self, expr: &Expression) -> Self::Result {
        for child in expr.children() {
            self.visit(child);
        }
        self.default_result()
    }

    fn default_result(&self) -> Self::Result;
}

/// 表达式转换器 - 用于转换表达式树
///
/// 这个trait允许访问者修改和转换表达式树，适用于重写和优化场景。
pub trait ExpressionTransformer: ExpressionVisitor<Result = Expression> {
    fn transform(&mut self, expr: &Expression) -> Expression {
        self.visit(expr)
    }

    fn transform_children(&mut self, expr: &Expression) -> Expression {
        let _children = expr.children();
        match expr {
            Expression::Binary { left, op, right } => {
                let new_left = self.transform(left);
                let new_right = self.transform(right);
                Expression::Binary {
                    left: Box::new(new_left),
                    op: op.clone(),
                    right: Box::new(new_right),
                }
            }
            Expression::Unary { op, operand } => {
                let new_operand = self.transform(operand);
                Expression::Unary {
                    op: op.clone(),
                    operand: Box::new(new_operand),
                }
            }
            Expression::Function { name, args } => {
                let new_args: Vec<Expression> = args.iter().map(|a| self.transform(a)).collect();
                Expression::Function {
                    name: name.clone(),
                    args: new_args,
                }
            }
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let new_arg = self.transform(arg);
                Expression::Aggregate {
                    func: func.clone(),
                    arg: Box::new(new_arg),
                    distinct: *distinct,
                }
            }
            Expression::List(items) => {
                let new_items: Vec<Expression> = items.iter().map(|i| self.transform(i)).collect();
                Expression::List(new_items)
            }
            Expression::Map(pairs) => {
                let new_pairs: Vec<(String, Expression)> = pairs
                    .iter()
                    .map(|(k, v)| (k.clone(), self.transform(v)))
                    .collect();
                Expression::Map(new_pairs)
            }
            Expression::Case {
                conditions,
                default,
            } => {
                let new_conditions: Vec<(Expression, Expression)> = conditions
                    .iter()
                    .map(|(c, v)| (self.transform(c), self.transform(v)))
                    .collect();
                let new_default = default.as_ref().map(|d| Box::new(self.transform(d)));
                Expression::Case {
                    conditions: new_conditions,
                    default: new_default,
                }
            }
            Expression::TypeCast { expr, target_type } => {
                let new_expr = self.transform(expr);
                Expression::TypeCast {
                    expr: Box::new(new_expr),
                    target_type: target_type.clone(),
                }
            }
            Expression::Subscript { collection, index } => {
                let new_collection = self.transform(collection);
                let new_index = self.transform(index);
                Expression::Subscript {
                    collection: Box::new(new_collection),
                    index: Box::new(new_index),
                }
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                let new_collection = self.transform(collection);
                let new_start = start.as_ref().map(|s| Box::new(self.transform(s)));
                let new_end = end.as_ref().map(|e| Box::new(self.transform(e)));
                Expression::Range {
                    collection: Box::new(new_collection),
                    start: new_start,
                    end: new_end,
                }
            }
            Expression::Path(items) => {
                let new_items: Vec<Expression> = items.iter().map(|i| self.transform(i)).collect();
                Expression::Path(new_items)
            }
            Expression::Property { object, property } => {
                let new_object = self.transform(object);
                Expression::Property {
                    object: Box::new(new_object),
                    property: property.clone(),
                }
            }
            _ => expr.clone(),
        }
    }
}
