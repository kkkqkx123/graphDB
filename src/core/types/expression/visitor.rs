//! 统一的表达式访问者接口
//!
//! 本模块提供统一的表达式访问者接口，用于遍历和转换 `Expression` 类型。
//! 主要组件：
//! - `ExpressionVisitor`: 核心访问者 trait
//! - `ExpressionVisitorState`: 访问者状态管理
//! - `ExpressionDepthFirstVisitor`: 深度优先遍历 trait
//! - `ExpressionTransformer`: 表达式转换 trait
//! - `ExpressionVisitorExt`: 访问者扩展 trait

use crate::core::types::expression::{DataType, Expression};
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 统一的表达式访问者 trait
///
/// 基于 `core::types::Expression` 的统一访问者接口，
/// 提供完整的表达式遍历、转换和状态管理功能。
pub trait ExpressionVisitor: Send + Sync {
    /// 访问者结果类型
    type Result;

    /// 主入口点 - 访问表达式
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
            Expression::Case { conditions, default } => self.visit_case(conditions, default.as_deref()),
            Expression::TypeCast { expression, target_type } => {
                self.visit_type_cast(expression, target_type)
            }
            Expression::Subscript { collection, index } => {
                self.visit_subscript(collection, index)
            }
            Expression::Range { collection, start, end } => {
                self.visit_range(collection, start.as_deref(), end.as_deref())
            }
            Expression::Path(items) => self.visit_path(items),
            Expression::Label(name) => self.visit_label(name),
            Expression::ListComprehension { variable, source, filter, map } => {
                self.visit_list_comprehension(variable, source.as_ref(), filter.as_deref(), map.as_deref())
            }
        }
    }

    /// 访问字面量
    fn visit_literal(&mut self, value: &Value) -> Self::Result;

    /// 访问变量
    fn visit_variable(&mut self, name: &str) -> Self::Result;

    /// 访问属性访问
    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result;

    /// 访问二元运算
    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result;

    /// 访问一元运算
    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Self::Result;

    /// 访问函数调用
    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result;

    /// 访问聚合函数
    fn visit_aggregate(
        &mut self,
        func: &AggregateFunction,
        arg: &Expression,
        distinct: bool,
    ) -> Self::Result;

    /// 访问列表
    fn visit_list(&mut self, items: &[Expression]) -> Self::Result;

    /// 访问映射
    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result;

    /// 访问 CASE 表达式
    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result;

    /// 访问类型转换
    fn visit_type_cast(&mut self, expression: &Expression, target_type: &DataType) -> Self::Result;

    /// 访问下标访问
    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result;

    /// 访问范围表达式
    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) -> Self::Result;

    /// 访问路径表达式
    fn visit_path(&mut self, items: &[Expression]) -> Self::Result;

    /// 访问标签表达式
    fn visit_label(&mut self, name: &str) -> Self::Result;

    /// 访问列表推导表达式
    fn visit_list_comprehension(
        &mut self,
        variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) -> Self::Result;

    /// 获取访问者状态（默认实现）
    fn state(&self) -> &ExpressionVisitorState {
        panic!("state() must be implemented or use ExpressionVisitorState struct")
    }

    /// 获取可变访问者状态（默认实现）
    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        panic!("state_mut() must be implemented or use ExpressionVisitorState struct")
    }

    /// 检查是否应该继续访问
    fn should_continue(&self) -> bool {
        self.state().continue_visiting
    }

    /// 停止访问
    fn stop(&mut self) {
        self.state_mut().continue_visiting = false
    }
}

/// 表达式访问者状态
///
/// 用于跟踪访问过程中的状态信息，如访问深度、访问计数等。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionVisitorState {
    /// 是否继续访问
    pub continue_visiting: bool,
    /// 当前访问深度
    pub depth: usize,
    /// 最大达到的深度
    pub max_depth_reached: usize,
    /// 访问计数
    pub visit_count: usize,
    /// 最大深度限制
    pub max_depth: Option<usize>,
    /// 自定义状态数据
    pub custom_data: HashMap<String, Value>,
}

impl ExpressionVisitorState {
    /// 创建新的访问者状态
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

    /// 获取当前访问深度
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// 获取最大达到的深度
    pub fn max_depth_reached(&self) -> usize {
        self.max_depth_reached
    }

    /// 获取访问计数
    pub fn visit_count(&self) -> usize {
        self.visit_count
    }

    /// 增加访问深度
    pub fn increment_depth(&mut self) {
        self.depth += 1;
        self.max_depth_reached = self.max_depth_reached.max(self.depth);
    }

    /// 减少访问深度
    pub fn decrement_depth(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    /// 增加访问计数
    pub fn increment_visit_count(&mut self) {
        self.visit_count += 1;
    }

    /// 设置最大深度限制
    pub fn set_max_depth(&mut self, max_depth: usize) {
        self.max_depth = Some(max_depth);
    }

    /// 清除最大深度限制
    pub fn clear_max_depth(&mut self) {
        self.max_depth = None;
    }

    /// 检查是否超过最大深度
    pub fn exceeds_max_depth(&self) -> bool {
        self.max_depth.map_or(false, |max| self.depth > max)
    }

    /// 设置自定义数据
    pub fn set_custom_data(&mut self, key: impl Into<String>, value: Value) {
        self.custom_data.insert(key.into(), value);
    }

    /// 获取自定义数据
    pub fn get_custom_data(&self, key: &str) -> Option<&Value> {
        self.custom_data.get(key)
    }

    /// 移除自定义数据
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisitorError {
    /// 超过最大深度限制
    MaxDepthExceeded,
    /// 访问被停止
    VisitationStopped,
    /// 类型不匹配
    TypeMismatch(String),
    /// 自定义错误
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

/// 深度优先遍历器 trait
///
/// 提供深度优先遍历表达式树的默认实现。
/// 实现者需要自行处理 `Self::Result` 的类型兼容性问题。
pub trait ExpressionDepthFirstVisitor: ExpressionVisitor {
    /// 访问子表达式
    ///
    /// 此方法由实现者根据具体的 `Self::Result` 类型自行实现。
    /// 如果 `Self::Result` 是 `Result` 类型，可以参考以下实现：
    ///
    /// ```rust
    /// fn visit_children(&mut self, expression: &Expression) -> Self::Result {
    ///     for child in expression.children() {
    ///         self.visit_expression(child)?;
    ///     }
    ///     Ok(())
    /// }
    /// ```
    fn visit_children(&mut self, expression: &Expression) -> Self::Result;

    /// 获取访问者状态的可变引用
    fn state_mut(&mut self) -> &mut ExpressionVisitorState;
}

/// 表达式转换器 trait
///
/// 允许访问者修改和转换表达式树。
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
            Expression::Literal(_) | Expression::Variable(_) | Expression::Label(_) | Expression::ListComprehension { .. } => {
                expression.clone()
            }
        }
    }
}

/// 访问者辅助 trait
///
/// 提供额外的实用方法。
pub trait ExpressionVisitorExt: ExpressionVisitor {
    /// 获取表达式树的最大深度
    fn max_depth(&mut self, expression: &Expression) -> usize {
        if expression.children().is_empty() {
            return 1;
        }

        expression
            .children()
            .iter()
            .map(|child| self.max_depth(child))
            .max()
            .map(|d| d + 1)
            .unwrap_or(1)
    }

    /// 获取表达式树中的所有变量名
    fn collect_variables(&mut self, expression: &Expression) -> Vec<String> {
        let mut variables = Vec::new();
        self.collect_variables_recursive(expression, &mut variables);
        variables
    }

    /// 递归收集变量名
    fn collect_variables_recursive(&mut self, expression: &Expression, variables: &mut Vec<String>) {
        if let Expression::Variable(name) = expression {
            variables.push(name.clone());
        }
        for child in expression.children() {
            self.collect_variables_recursive(child, variables);
        }
    }

    /// 检查表达式是否有效（无循环引用等）
    fn is_valid_expression(&mut self, expression: &Expression) -> bool {
        let mut visited = std::collections::HashSet::new();
        self.check_no_cycles(expression, &mut visited)
    }

    /// 检查循环引用
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

/// 为所有实现了 ExpressionVisitor 的类型自动实现 ExpressionVisitorExt
impl<T: ExpressionVisitor> ExpressionVisitorExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    /// 测试用的简单访问者
    struct CountingVisitor {
        count: usize,
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
            default: Option<&Expression>,
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
            start: Option<&Expression>,
            end: Option<&Expression>,
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
            self.count
        }
    }

    #[test]
    fn test_counting_visitor() {
        let expr = Expression::binary(
            Expression::variable("a"),
            BinaryOperator::Add,
            Expression::variable("b"),
        );

        let mut visitor = CountingVisitor { count: 0 };
        let result = visitor.visit_expression(&expr);

        assert_eq!(result, 2);
    }

    #[test]
    fn test_visitor_state() {
        let mut state = ExpressionVisitorState::new();
        assert_eq!(state.depth(), 0);
        assert_eq!(state.visit_count(), 0);

        state.increment_depth();
        state.increment_visit_count();
        state.increment_depth();

        assert_eq!(state.depth(), 2);
        assert_eq!(state.visit_count(), 1);
        assert_eq!(state.max_depth_reached(), 2);
    }

    #[test]
    fn test_visitor_state_max_depth() {
        let mut state = ExpressionVisitorState::new();
        state.set_max_depth(2);

        state.increment_depth();
        assert!(!state.exceeds_max_depth());

        state.increment_depth();
        assert!(!state.exceeds_max_depth());

        state.increment_depth();
        assert!(state.exceeds_max_depth());
    }

    #[test]
    fn test_visitor_error_display() {
        assert_eq!(VisitorError::MaxDepthExceeded.to_string(), "超过最大深度限制");
        assert_eq!(VisitorError::VisitationStopped.to_string(), "访问被停止");
        assert_eq!(
            VisitorError::TypeMismatch("test".to_string()).to_string(),
            "类型不匹配: test"
        );
        assert_eq!(
            VisitorError::Custom("error".to_string()).to_string(),
            "自定义错误: error"
        );
    }
}

/// 表达式接受者 trait
///
/// 提供统一的表达式访问入口，简化访问者模式的使用。
pub trait ExpressionAcceptor {
    /// 接受访问者访问
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result;
}

impl ExpressionAcceptor for Expression {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_expression(self)
    }
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
