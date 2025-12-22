//! 表达式访问者模式实现
//! 
//! 这个模块提供了统一的表达式访问者基础设施，支持零成本抽象

use crate::core::visitor::{VisitorCore, VisitorContext, VisitorResult};
use crate::expression::{Expression, LiteralValue, BinaryOperator, UnaryOperator, AggregateFunction, DataType};

/// 表达式访问者 trait - 用于访问Expression类型的各个变体
pub trait ExpressionVisitor: VisitorCore<Expression> {
    fn visit_literal(&mut self, value: &LiteralValue) -> Self::Result;
    fn visit_variable(&mut self, name: &str) -> Self::Result;
    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result;
    fn visit_binary(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression) -> Self::Result;
    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Self::Result;
    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result;
    fn visit_aggregate(&mut self, func: &AggregateFunction, arg: &Expression, distinct: bool) -> Self::Result;
    fn visit_list(&mut self, items: &[Expression]) -> Self::Result;
    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result;
    fn visit_case(&mut self, conditions: &[(Expression, Expression)], default: &Option<Expression>) -> Self::Result;
    fn visit_type_cast(&mut self, expr: &Expression, target_type: &DataType) -> Self::Result;
    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result;
    fn visit_range(&mut self, collection: &Expression, start: &Option<Expression>, end: &Option<Expression>) -> Self::Result;
    fn visit_path(&mut self, items: &[Expression]) -> Self::Result;
    fn visit_label(&mut self, name: &str) -> Self::Result;
    fn visit_tag_property(&mut self, tag: &str, prop: &str) -> Self::Result;
    fn visit_edge_property(&mut self, edge: &str, prop: &str) -> Self::Result;
    fn visit_input_property(&mut self, prop: &str) -> Self::Result;
    fn visit_variable_property(&mut self, var: &str, prop: &str) -> Self::Result;
    fn visit_source_property(&mut self, tag: &str, prop: &str) -> Self::Result;
    fn visit_destination_property(&mut self, tag: &str, prop: &str) -> Self::Result;
}

/// 表达式访问者接受器 trait - 为Expression类型提供接受访问者的能力
pub trait ExpressionAcceptor {
    /// 接受访问者进行访问
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result;
}

impl ExpressionAcceptor for Expression {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result {
        use Expression::*;
        
        match self {
            Literal(value) => visitor.visit_literal(value),
            Variable(name) => visitor.visit_variable(name),
            Property { object, property } => visitor.visit_property(object, property),
            Binary { left, op, right } => visitor.visit_binary(left, op, right),
            Unary { op, operand } => visitor.visit_unary(op, operand),
            Function { name, args } => visitor.visit_function(name, args),
            Aggregate { func, arg, distinct } => visitor.visit_aggregate(func, arg, *distinct),
            List(items) => visitor.visit_list(items),
            Map(pairs) => visitor.visit_map(pairs),
            Case { conditions, default } => {
                let default_cloned = default.as_ref().map(|b| (**b).clone());
                visitor.visit_case(conditions, &default_cloned)
            }
            TypeCast { expr, target_type } => visitor.visit_type_cast(expr, target_type),
            Subscript { collection, index } => visitor.visit_subscript(collection, index),
            Range { collection, start, end } => {
                let start_cloned = start.as_ref().map(|b| (**b).clone());
                let end_cloned = end.as_ref().map(|b| (**b).clone());
                visitor.visit_range(collection, &start_cloned, &end_cloned)
            }
            Path(items) => visitor.visit_path(items),
            Label(name) => visitor.visit_label(name),
            TagProperty { tag, prop } => visitor.visit_tag_property(tag, prop),
            EdgeProperty { edge, prop } => visitor.visit_edge_property(edge, prop),
            InputProperty(prop) => visitor.visit_input_property(prop),
            VariableProperty { var, prop } => visitor.visit_variable_property(var, prop),
            SourceProperty { tag, prop } => visitor.visit_source_property(tag, prop),
            DestinationProperty { tag, prop } => visitor.visit_destination_property(tag, prop),
            
            // 处理新增的表达式类型
            UnaryPlus(expr) => visitor.visit_unary(&UnaryOperator::Plus, expr),
            UnaryNegate(expr) => visitor.visit_unary(&UnaryOperator::Minus, expr),
            UnaryNot(expr) => visitor.visit_unary(&UnaryOperator::Not, expr),
            UnaryIncr(expr) => visitor.visit_unary(&UnaryOperator::Increment, expr),
            UnaryDecr(expr) => visitor.visit_unary(&UnaryOperator::Decrement, expr),
            IsNull(expr) => visitor.visit_unary(&UnaryOperator::IsNull, expr),
            IsNotNull(expr) => visitor.visit_unary(&UnaryOperator::IsNotNull, expr),
            IsEmpty(expr) => visitor.visit_unary(&UnaryOperator::IsEmpty, expr),
            IsNotEmpty(expr) => visitor.visit_unary(&UnaryOperator::IsNotEmpty, expr),
            
            TypeCasting { expr, .. } => visitor.visit_type_cast(expr, &DataType::String),
            ListComprehension { generator, condition } => {
                // 简化为函数调用
                let cond_expr = condition
                    .as_ref()
                    .map(|c| (**c).clone())
                    .unwrap_or(Expression::bool(true));
                visitor.visit_function(
                    "list_comprehension",
                    &[(**generator).clone(), cond_expr],
                )
            }
            Predicate { list, condition } => {
                visitor.visit_function("predicate", &[(**list).clone(), (**condition).clone()])
            }
            Reduce { list, initial, expr, .. } => {
                visitor.visit_function("reduce", &[(**list).clone(), (**initial).clone(), (**expr).clone()])
            }
            PathBuild(items) => visitor.visit_path(items),
            ESQuery(query) => visitor.visit_function("es_query", &[Expression::string(query)]),
            UUID => visitor.visit_function("uuid", &[]),
            SubscriptRange { collection, start, end } => {
                let start_cloned = start.as_ref().map(|b| (**b).clone());
                let end_cloned = end.as_ref().map(|b| (**b).clone());
                visitor.visit_range(collection, &start_cloned, &end_cloned)
            }
            MatchPathPattern { patterns, .. } => visitor.visit_list(patterns),
        }
    }
}

/// 默认表达式访问者实现
#[derive(Debug)]
pub struct DefaultExpressionVisitor {
    context: VisitorContext,
    state: crate::core::visitor::visitor_state_enum::VisitorStateEnum,
}

impl DefaultExpressionVisitor {
    /// 创建新的默认表达式访问者
    pub fn new() -> Self {
        Self {
            context: VisitorContext::new(crate::core::visitor::VisitorConfig::new()),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::new(),
        }
    }

    /// 创建带配置的默认表达式访问者
    pub fn with_config(config: crate::core::visitor::VisitorConfig) -> Self {
        Self {
            context: VisitorContext::new(config),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::new(),
        }
    }

    /// 创建带初始深度的默认表达式访问者
    pub fn with_depth(depth: usize) -> Self {
        Self {
            context: VisitorContext::new(crate::core::visitor::VisitorConfig::new()),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::with_depth(depth),
        }
    }

    /// 创建带配置和初始深度的默认表达式访问者
    pub fn with_config_and_depth(config: crate::core::visitor::VisitorConfig, depth: usize) -> Self {
        Self {
            context: VisitorContext::new(config),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::with_depth(depth),
        }
    }
}

impl Default for DefaultExpressionVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl VisitorCore<Expression> for DefaultExpressionVisitor {
    type Result = ();

    fn visit(&mut self, target: &Expression) -> Self::Result {
        target.accept(self);
    }

    fn context(&self) -> &VisitorContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut VisitorContext {
        &mut self.context
    }

    fn state(&self) -> &crate::core::visitor::visitor_state_enum::VisitorStateEnum {
        &self.state
    }

    fn state_mut(&mut self) -> &mut crate::core::visitor::visitor_state_enum::VisitorStateEnum {
        &mut self.state
    }
}

impl ExpressionVisitor for DefaultExpressionVisitor {
    fn visit_literal(&mut self, _value: &LiteralValue) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_variable(&mut self, _name: &str) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_property(&mut self, _object: &Expression, _property: &str) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_binary(&mut self, _left: &Expression, _op: &BinaryOperator, _right: &Expression) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, _operand: &Expression) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_function(&mut self, _name: &str, _args: &[Expression]) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_aggregate(&mut self, _func: &AggregateFunction, _arg: &Expression, _distinct: bool) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_list(&mut self, _items: &[Expression]) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_map(&mut self, _pairs: &[(String, Expression)]) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_case(&mut self, _conditions: &[(Expression, Expression)], _default: &Option<Expression>) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_type_cast(&mut self, _expr: &Expression, _target_type: &DataType) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_subscript(&mut self, _collection: &Expression, _index: &Expression) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_range(&mut self, _collection: &Expression, _start: &Option<Expression>, _end: &Option<Expression>) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_path(&mut self, _items: &[Expression]) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_tag_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_edge_property(&mut self, _edge: &str, _prop: &str) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_input_property(&mut self, _prop: &str) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_variable_property(&mut self, _var: &str, _prop: &str) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_source_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        // 默认实现什么也不做
    }

    fn visit_destination_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        // 默认实现什么也不做
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::{BinaryOperator, LiteralValue};

    #[test]
    fn test_default_expression_visitor() {
        let mut visitor = DefaultExpressionVisitor::new();
        let expr = Expression::binary(
            Expression::literal(LiteralValue::Int(1)),
            BinaryOperator::Add,
            Expression::literal(LiteralValue::Int(2)),
        );

        // 测试访问表达式
        visitor.visit(&expr);
        assert!(visitor.should_continue());
    }

    #[test]
    fn test_expression_acceptor() {
        let mut visitor = DefaultExpressionVisitor::new();
        let expr = Expression::variable("test");

        // 测试接受器模式
        expr.accept(&mut visitor);
        assert!(visitor.should_continue());
    }

    #[test]
    fn test_visitor_with_config() {
        let config = crate::core::visitor::VisitorConfig::new().with_max_depth(5);
        let visitor = DefaultExpressionVisitor::with_config(config);

        assert_eq!(visitor.context().config().max_depth, 5);
    }
}