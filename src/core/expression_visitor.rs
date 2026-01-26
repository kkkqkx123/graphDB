//! 表达式访问者模式
//!
//! 这个模块提供了表达式访问者接口。
//!
//! **重要**: 此模块保留旧的接口定义以保持向后兼容。
//! 新代码请使用 `core::types::expression::visitor` 模块中的统一接口。

use crate::core::types::expression::{DataType, Expression};
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;
use std::collections::HashMap;

/// 表达式访问者状态
#[derive(Debug, Clone)]
pub struct ExpressionVisitorState {
    pub continue_visiting: bool,
    pub depth: usize,
    pub max_depth_reached: usize,
    pub visit_count: usize,
    pub max_depth: Option<usize>,
    pub custom_data: HashMap<String, Value>,
}

impl ExpressionVisitorState {
    pub fn new() -> Self {
        Self {
            continue_visiting: true,
            depth: 0,
            max_depth_reached: 0,
            visit_count: 0,
            max_depth: None,
            custom_data: HashMap::new(),
        }
    }

    pub fn max_depth_reached(&self) -> usize {
        self.max_depth_reached
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn visit_count(&self) -> usize {
        self.visit_count
    }

    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = Some(max_depth);
    }

    pub fn clear_max_depth(&mut self) {
        self.max_depth = None;
    }

    pub fn exceeds_max_depth(&self) -> bool {
        if let Some(max) = self.max_depth {
            self.depth > max
        } else {
            false
        }
    }

    pub fn increment_depth(&mut self) {
        self.depth += 1;
        if self.depth > self.max_depth_reached {
            self.max_depth_reached = self.depth;
        }
    }

    pub fn decrement_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    pub fn increment_visit_count(&mut self) {
        self.visit_count += 1;
    }

    pub fn set_custom_data(&mut self, key: String, value: Value) {
        self.custom_data.insert(key, value);
    }

    pub fn get_custom_data(&self, key: &str) -> Option<&Value> {
        self.custom_data.get(key)
    }

    pub fn remove_custom_data(&mut self, key: &str) -> Option<Value> {
        self.custom_data.remove(key)
    }
}

impl Default for ExpressionVisitorState {
    fn default() -> Self {
        Self::new()
    }
}

/// 表达式访问者结果类型
pub type VisitorResult<T> = Result<T, VisitorError>;

/// 表达式访问者错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisitorError {
    MaxDepthExceeded,
    VisitationStopped,
    TypeMismatch(String),
    Custom(String),
}

impl std::fmt::Display for VisitorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisitorError::MaxDepthExceeded => write!(f, "超过最大深度限制"),
            VisitorError::VisitationStopped => write!(f, "访问被停止"),
            VisitorError::TypeMismatch(msg) => write!(f, "类型不匹配: {}", msg),
            VisitorError::Custom(msg) => write!(f, "自定义错误: {}", msg),
        }
    }
}

impl std::error::Error for VisitorError {}

pub mod prelude {
    pub use super::{
        ExprAcceptor, ExpressionAcceptor, ExpressionDepthFirstVisitor,
        ExpressionTransformer, ExpressionVisitor, ExpressionVisitorExt,
        ExpressionVisitorState, ExpressionVisitable, GenericExpressionVisitor, VisitorError,
        VisitorResult,
    };
}

/// 泛型表达式访问者trait
///
/// 使用泛型参数T来支持不同的表达式类型
/// 通过impl Trait约束实现零开销抽象
pub trait GenericExpressionVisitor<T: ?Sized> {
    /// 访问者结果类型
    type Result;

    /// 主入口点 - 访问表达式
    fn visit(&mut self, expression: &T) -> Self::Result;
}

/// 表达式可访问 trait
/// 定义表达式类型如何接受访问者
pub trait ExpressionVisitable {
    type Result;
    fn accept<V: GenericExpressionVisitor<Self> + ?Sized>(&self, visitor: &mut V) -> V::Result;
}

/// 为Expression实现可访问 trait
impl ExpressionVisitable for Expression {
    type Result = Result<Value, crate::core::error::ExpressionError>;

    fn accept<V: GenericExpressionVisitor<Self> + ?Sized>(&self, visitor: &mut V) -> V::Result {
        visitor.visit(self)
    }
}

/// 表达式访问者trait
///
/// 此 trait 是向后兼容版本，用于遍历 Expression 类型。
/// 新代码请考虑使用 `core::types::expression::visitor::ExpressionVisitor`。
pub trait ExpressionVisitor: std::fmt::Debug + Send + Sync {
    /// 访问者结果类型
    type Result;

    /// 主入口点 - 访问Expression类型
    fn visit_expression(&mut self, expression: &Expression) -> Self::Result {
        match expression {
            Expression::Literal(value) => self.visit_literal(value),
            Expression::Variable(name) => self.visit_variable(name),
            Expression::Property { object, property } => self.visit_property(object, property),
            Expression::Binary { left, op, right } => self.visit_binary(left, op, right),
            Expression::Unary { op, operand } => self.visit_unary(op, operand),
            Expression::Function { name, args } => self.visit_function(name, args),
            Expression::Aggregate { func, arg, distinct } => {
                self.visit_aggregate(func, arg, *distinct)
            }
            Expression::List(items) => self.visit_list(items),
            Expression::Map(pairs) => self.visit_map(pairs),
            Expression::Case { conditions, default } => self.visit_case(conditions, default),
            Expression::TypeCast { expression, target_type } => {
                self.visit_type_cast(expression, target_type)
            }
            Expression::Subscript { collection, index } => {
                self.visit_subscript(collection, index)
            }
            Expression::Range { collection, start, end } => {
                self.visit_range(collection, start, end)
            }
            Expression::Path(items) => self.visit_path(items),
            Expression::Label(name) => self.visit_label(name),
        }
    }

    /// 访问统一Expression类型的便捷方法（可选覆盖）
    fn visit(&mut self, expression: &Expression) -> Self::Result {
        self.visit_expression(expression)
    }

    /// Expression类型访问方法
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
    fn visit_type_cast(&mut self, expression: &Expression, target_type: &DataType) -> Self::Result;
    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result;
    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result;
    fn visit_path(&mut self, items: &[Expression]) -> Self::Result;
    fn visit_label(&mut self, name: &str) -> Self::Result;

    /// 预访问钩子 - 在访问开始前调用
    fn pre_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }

    /// 后访问钩子 - 在访问结束后调用
    fn post_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }

    /// 获取访问者状态
    fn state(&self) -> &ExpressionVisitorState;

    /// 获取可变访问者状态
    fn state_mut(&mut self) -> &mut ExpressionVisitorState;

    /// 检查是否应该继续访问
    fn should_continue(&self) -> bool {
        self.state().continue_visiting
    }

    /// 停止访问
    fn stop(&mut self) {
        self.state_mut().continue_visiting = false
    }
}

/// 表达式访问者状态类型
pub type ExpressionVisitorStateAlias = ExpressionVisitorState;

/// 表达式访问者结果类型
pub type VisitorResultAlias<T> = VisitorResult<T>;

/// 表达式访问者错误类型
pub type VisitorErrorAlias = VisitorError;

/// 表达式深度优先遍历器trait
///
/// 提供深度优先遍历表达式树的默认实现
pub trait ExpressionDepthFirstVisitor: ExpressionVisitor {
    /// 访问子表达式
    fn visit_children(&mut self, expression: &Expression) -> Self::Result {
        for child in expression.children() {
            self.visit_expression(child);
        }
        self.default_result()
    }

    /// 默认结果
    fn default_result(&self) -> Self::Result;

    /// 带深度控制的访问
    fn visit_with_depth(&mut self, expression: &Expression) -> VisitorResult<Self::Result> {
        {
            let state = self.state_mut();
            state.increment_depth();

            if state.exceeds_max_depth() {
                state.decrement_depth();
                return Err(VisitorError::MaxDepthExceeded);
            }
        }

        let result = self.visit_expression(expression);

        let state = self.state_mut();
        state.decrement_depth();
        Ok(result)
    }
}

/// 表达式转换器trait
///
/// 允许访问者修改和转换表达式树
pub trait ExpressionTransformer: ExpressionVisitor<Result = Expression> {
    /// 转换表达式
    fn transform(&mut self, expression: &Expression) -> Expression {
        self.visit_expression(expression)
    }

    /// 转换子表达式
    fn transform_children(&mut self, expression: &Expression) -> Expression {
        match expression {
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
            Expression::Aggregate { func, arg, distinct } => {
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
            Expression::TypeCast { expression, target_type } => {
                let new_expression = self.transform(expression);
                Expression::TypeCast {
                    expression: Box::new(new_expression),
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
            _ => expression.clone(),
        }
    }
}

/// 表达式接受器trait
///
/// 为Expression类型提供接受访问者的能力
pub trait ExpressionAcceptor {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result;
}

impl ExpressionAcceptor for Expression {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_expression(self)
    }
}

/// 表达式接受器trait（AST Expression类型）
pub trait ExprAcceptor {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result;
}

impl ExprAcceptor for Expression {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_expression(self)
    }
}

/// 表达式访问者辅助trait
///
/// 提供额外的实用方法
pub trait ExpressionVisitorExt: ExpressionVisitor {
    /// 获取表达式树的最大深度
    fn max_depth(&mut self, expression: &Expression) -> usize {
        if expression.children().is_empty() {
            return 1;
        }

        let max_child_depth = expression
            .children()
            .iter()
            .map(|child| self.max_depth(child))
            .max()
            .unwrap_or(0);

        max_child_depth + 1
    }

    /// 获取表达式树中的所有变量名
    fn collect_variables(&mut self, expression: &Expression) -> Vec<String> {
        let mut variables = Vec::new();

        if let Expression::Variable(name) = expression {
            variables.push(name.clone());
        }

        for child in expression.children() {
            variables.extend(self.collect_variables(child));
        }

        variables
    }

    /// 检查表达式是否有效（无循环引用等）
    fn is_valid_expression(&mut self, expression: &Expression) -> bool {
        let mut visited = std::collections::HashSet::new();
        self.check_no_cycles(expression, &mut visited)
    }

    fn check_no_cycles(
        &mut self,
        expression: &Expression,
        visited: &mut std::collections::HashSet<usize>,
    ) -> bool {
        let expr_id = expression as *const _ as usize;

        if visited.contains(&expr_id) {
            return false;
        }

        visited.insert(expr_id);

        for child in expression.children() {
            if !self.check_no_cycles(child, visited) {
                return false;
            }
        }

        visited.remove(&expr_id);
        true
    }
}

/// 为所有实现了ExpressionVisitor的类型自动实现ExpressionVisitorExt
impl<T: ExpressionVisitor> ExpressionVisitorExt for T {}

/// Adapter: 将 ExpressionVisitor 转换为新版 ExpressionVisitor
///
/// 用于在旧版接口和新版接口之间进行兼容
#[derive(Debug, Clone)]
pub struct LegacyToNewAdapter<T: ExpressionVisitor> {
    visitor: T,
}

impl<T: ExpressionVisitor> LegacyToNewAdapter<T> {
    /// 创建新的适配器
    pub fn new(visitor: T) -> Self {
        Self { visitor }
    }

    /// 获取内部 visitor
    pub fn inner(&self) -> &T {
        &self.visitor
    }

    /// 获取内部 visitor 的可变引用
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.visitor
    }
}

impl<T: ExpressionVisitor> crate::core::types::expression::visitor::ExpressionVisitor
    for LegacyToNewAdapter<T>
{
    type Result = T::Result;

    fn visit_expression(&mut self, expression: &Expression) -> Self::Result {
        self.visitor.visit_expression(expression)
    }

    fn visit_literal(&mut self, value: &Value) -> Self::Result {
        self.visitor.visit_literal(value)
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.visitor.visit_variable(name)
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        self.visitor.visit_property(object, property)
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        self.visitor.visit_binary(left, op, right)
    }

    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visitor.visit_unary(op, operand)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        self.visitor.visit_function(name, args)
    }

    fn visit_aggregate(
        &mut self,
        func: &AggregateFunction,
        arg: &Expression,
        distinct: bool,
    ) -> Self::Result {
        self.visitor.visit_aggregate(func, arg, distinct)
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        self.visitor.visit_list(items)
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        self.visitor.visit_map(pairs)
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result {
        let default_ref = default.map(|e| Box::new(e.clone()));
        self.visitor.visit_case(conditions, &default_ref)
    }

    fn visit_type_cast(&mut self, expression: &Expression, target_type: &DataType) -> Self::Result {
        self.visitor.visit_type_cast(expression, target_type)
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visitor.visit_subscript(collection, index)
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) -> Self::Result {
        let start_ref = start.map(|e| Box::new(e.clone()));
        let end_ref = end.map(|e| Box::new(e.clone()));
        self.visitor.visit_range(collection, &start_ref, &end_ref)
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        self.visitor.visit_path(items)
    }

    fn visit_label(&mut self, name: &str) -> Self::Result {
        self.visitor.visit_label(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    /// 测试用的简单访问者
    struct CountingVisitor {
        count: usize,
        state: ExpressionVisitorState,
    }

    impl CountingVisitor {
        fn new() -> Self {
            Self {
                count: 0,
                state: ExpressionVisitorState::new(),
            }
        }
    }

    impl ExpressionVisitor for CountingVisitor {
        type Result = usize;

        fn visit_literal(&mut self, _value: &Value) -> Self::Result {
            self.count += 1;
            self.count
        }

        fn visit_variable(&mut self, _name: &str) -> Self::Result {
            self.count += 1;
            self.count
        }

        fn visit_property(&mut self, object: &Expression, _property: &str) -> Self::Result {
            self.visit_expression(object)
        }

        fn visit_binary(
            &mut self,
            left: &Expression,
            _op: &BinaryOperator,
            right: &Expression,
        ) -> Self::Result {
            self.visit_expression(left)?;
            self.visit_expression(right)?;
            Ok(self.count)
        }

        fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
            self.visit_expression(operand)
        }

        fn visit_function(&mut self, _name: &str, args: &[Expression]) -> Self::Result {
            for arg in args {
                self.visit_expression(arg)?;
            }
            Ok(self.count)
        }

        fn visit_aggregate(
            &mut self,
            _func: &AggregateFunction,
            arg: &Expression,
            _distinct: bool,
        ) -> Self::Result {
            self.visit_expression(arg)
        }

        fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
            for item in items {
                self.visit_expression(item)?;
            }
            Ok(self.count)
        }

        fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
            for (_, value) in pairs {
                self.visit_expression(value)?;
            }
            Ok(self.count)
        }

        fn visit_case(
            &mut self,
            conditions: &[(Expression, Expression)],
            default: &Option<Box<Expression>>,
        ) -> Self::Result {
            for (when, then) in conditions {
                self.visit_expression(when)?;
                self.visit_expression(then)?;
            }
            if let Some(default) = default {
                self.visit_expression(default)?;
            }
            Ok(self.count)
        }

        fn visit_type_cast(&mut self, expression: &Expression, _target_type: &DataType) -> Self::Result {
            self.visit_expression(expression)
        }

        fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
            self.visit_expression(collection)?;
            self.visit_expression(index)
        }

        fn visit_range(
            &mut self,
            collection: &Expression,
            start: &Option<Box<Expression>>,
            end: &Option<Box<Expression>>,
        ) -> Self::Result {
            self.visit_expression(collection)?;
            if let Some(start) = start {
                self.visit_expression(start)?;
            }
            if let Some(end) = end {
                self.visit_expression(end)?;
            }
            Ok(self.count)
        }

        fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
            for item in items {
                self.visit_expression(item)?;
            }
            Ok(self.count)
        }

        fn visit_label(&mut self, _name: &str) -> Self::Result {
            self.count += 1;
            Ok(self.count)
        }
    }

    #[test]
    fn test_new_visitor_interface() {
        let expr = Expression::binary(
            Expression::variable("a"),
            BinaryOperator::Add,
            Expression::variable("b"),
        );

        let mut visitor = CountingVisitor::new();
        let result = visitor.visit_expression(&expr);

        assert!(result.is_ok());
        assert_eq!(visitor.count, 2);
    }

    #[test]
    fn test_legacy_adapter() {
        struct LegacyVisitor {
            count: usize,
        }

        impl LegacyVisitor {
            fn new() -> Self {
                Self { count: 0 }
            }
        }

        impl ExpressionVisitor for LegacyVisitor {
            type Result = usize;

            fn visit(&mut self, expression: &Expression) -> Self::Result {
                self.visit_expression(expression)
            }

            fn visit_literal(&mut self, _value: &Value) -> Self::Result {
                self.count += 1;
                self.count
            }

            fn visit_variable(&mut self, _name: &str) -> Self::Result {
                self.count += 1;
                self.count
            }

            fn visit_property(&mut self, object: &Expression, _property: &str) -> Self::Result {
                self.visit_expression(object)
            }

            fn visit_binary(
                &mut self,
                left: &Expression,
                _op: &BinaryOperator,
                right: &Expression,
            ) -> Self::Result {
                self.visit_expression(left)?;
                self.visit_expression(right)?;
                Ok(self.count)
            }

            fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
                self.visit_expression(operand)
            }

            fn visit_function(&mut self, _name: &str, args: &[Expression]) -> Self::Result {
                for arg in args {
                    self.visit_expression(arg)?;
                }
                Ok(self.count)
            }

            fn visit_aggregate(
                &mut self,
                _func: &AggregateFunction,
                arg: &Expression,
                _distinct: bool,
            ) -> Self::Result {
                self.visit_expression(arg)
            }

            fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
                for item in items {
                    self.visit_expression(item)?;
                }
                Ok(self.count)
            }

            fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
                for (_, value) in pairs {
                    self.visit_expression(value)?;
                }
                Ok(self.count)
            }

            fn visit_case(
                &mut self,
                conditions: &[(Expression, Expression)],
                default: &Option<Box<Expression>>,
            ) -> Self::Result {
                for (when, then) in conditions {
                    self.visit_expression(when)?;
                    self.visit_expression(then)?;
                }
                if let Some(default) = default {
                    self.visit_expression(default)?;
                }
                Ok(self.count)
            }

            fn visit_type_cast(&mut self, expression: &Expression, _target_type: &DataType) -> Self::Result {
                self.visit_expression(expression)
            }

            fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
                self.visit_expression(collection)?;
                self.visit_expression(index)
            }

            fn visit_range(
                &mut self,
                collection: &Expression,
                start: &Option<Box<Expression>>,
                end: &Option<Box<Expression>>,
            ) -> Self::Result {
                self.visit_expression(collection)?;
                if let Some(start) = start {
                    self.visit_expression(start)?;
                }
                if let Some(end) = end {
                    self.visit_expression(end)?;
                }
                Ok(self.count)
            }

            fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
                for item in items {
                    self.visit_expression(item)?;
                }
                Ok(self.count)
            }

            fn visit_label(&mut self, _name: &str) -> Self::Result {
                self.count += 1;
                Ok(self.count)
            }

            fn state(&self) -> &ExpressionVisitorState {
                panic!("Not implemented for test")
            }

            fn state_mut(&mut self) -> &mut ExpressionVisitorState {
                panic!("Not implemented for test")
            }
        }

        // Test that the module exports work correctly
        let _ = prelude::ExpressionVisitor;
        let _ = ExpressionDepthFirstVisitor;
        let _ = ExpressionTransformer;
        let _ = ExpressionVisitorExt;
        let _ = ExpressionVisitorState::new();
    }
}
