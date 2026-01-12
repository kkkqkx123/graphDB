//! 表达式访问者模式
//!
//! 这个模块提供了统一的表达式访问者接口，支持泛型和特化两种模式
//! 主要组件：
//! - ExpressionVisitor: 特化的访问者trait，同时支持Expression和Expr类型
//! - GenericExpressionVisitor<T>: 泛型访问者接口，支持任意表达式类型

use crate::core::types::expression::{DataType, Expression, ExpressionType};
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::Value;
use crate::query::parser::ast::expr::*;
use std::collections::HashMap;

pub mod prelude {
    pub use super::{ExpressionVisitor, GenericExpressionVisitor};
}

/// 泛型表达式访问者trait
///
/// 使用泛型参数T来支持不同的表达式类型
/// 通过impl Trait约束实现零开销抽象
pub trait GenericExpressionVisitor<T: ?Sized> {
    /// 访问者结果类型
    type Result;

    /// 主入口点 - 访问表达式
    fn visit(&mut self, expr: &T) -> Self::Result;
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
/// 这个trait合并了原有的ExpressionVisitor和ExprVisitor，提供统一的表达式访问接口
/// 支持两种表达式类型：Expression（core/types/expression）和Expr（query/parser/ast）
pub trait ExpressionVisitor: std::fmt::Debug + Send + Sync {
    /// 访问者结果类型
    type Result;

    /// 访问Expression类型 - 主入口点
    fn visit_expression(&mut self, expr: &Expression) -> Self::Result {
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

    /// 访问Expr类型（AST表达式）- 主入口点
    fn visit_expr(&mut self, expr: &Expr) -> Self::Result {
        match expr {
            Expr::Constant(e) => self.visit_constant_expr(e),
            Expr::Variable(e) => self.visit_variable_expr(e),
            Expr::Binary(e) => self.visit_binary_expr(e),
            Expr::Unary(e) => self.visit_unary_expr(e),
            Expr::FunctionCall(e) => self.visit_function_call_expr(e),
            Expr::PropertyAccess(e) => self.visit_property_access_expr(e),
            Expr::List(e) => self.visit_list_expr(e),
            Expr::Map(e) => self.visit_map_expr(e),
            Expr::Case(e) => self.visit_case_expr(e),
            Expr::Subscript(e) => self.visit_subscript_expr(e),
            Expr::Predicate(e) => self.visit_predicate_expr(e),
            Expr::TagProperty(e) => self.visit_tag_property_expr(e),
            Expr::EdgeProperty(e) => self.visit_edge_property_expr(e),
            Expr::InputProperty(e) => self.visit_input_property_expr(e),
            Expr::VariableProperty(e) => self.visit_variable_property_expr(e),
            Expr::SourceProperty(e) => self.visit_source_property_expr(e),
            Expr::DestinationProperty(e) => self.visit_destination_property_expr(e),
            Expr::TypeCast(e) => self.visit_type_cast_expr(e),
            Expr::Range(e) => self.visit_range_expr(e),
            Expr::Path(e) => self.visit_path_expr(e),
            Expr::Label(e) => self.visit_label_expr(e),
            Expr::Reduce(e) => self.visit_reduce_expr(e),
            Expr::ListComprehension(e) => self.visit_list_comprehension_expr(e),
        }
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

    /// Expr类型（AST表达式）访问方法
    fn visit_constant_expr(&mut self, expr: &ConstantExpr) -> Self::Result;

    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> Self::Result;

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Self::Result;

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Self::Result;

    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpr) -> Self::Result;

    fn visit_property_access_expr(&mut self, expr: &PropertyAccessExpr) -> Self::Result;

    fn visit_list_expr(&mut self, expr: &ListExpr) -> Self::Result;

    fn visit_map_expr(&mut self, expr: &MapExpr) -> Self::Result;

    fn visit_case_expr(&mut self, expr: &CaseExpr) -> Self::Result;

    fn visit_subscript_expr(&mut self, expr: &SubscriptExpr) -> Self::Result;

    fn visit_predicate_expr(&mut self, expr: &PredicateExpr) -> Self::Result;

    fn visit_tag_property_expr(&mut self, expr: &TagPropertyExpr) -> Self::Result;

    fn visit_edge_property_expr(&mut self, expr: &EdgePropertyExpr) -> Self::Result;

    fn visit_input_property_expr(&mut self, expr: &InputPropertyExpr) -> Self::Result;

    fn visit_variable_property_expr(&mut self, expr: &VariablePropertyExpr) -> Self::Result;

    fn visit_source_property_expr(&mut self, expr: &SourcePropertyExpr) -> Self::Result;

    fn visit_destination_property_expr(&mut self, expr: &DestinationPropertyExpr) -> Self::Result;

    fn visit_type_cast_expr(&mut self, expr: &TypeCastExpr) -> Self::Result;

    fn visit_range_expr(&mut self, expr: &RangeExpr) -> Self::Result;

    fn visit_path_expr(&mut self, expr: &PathExpr) -> Self::Result;

    fn visit_label_expr(&mut self, expr: &LabelExpr) -> Self::Result;

    fn visit_reduce_expr(&mut self, expr: &ReduceExpr) -> Self::Result;

    fn visit_list_comprehension_expr(&mut self, expr: &ListComprehensionExpr) -> Self::Result;

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
        self.state_mut().continue_visiting = false;
    }
}

/// 表达式访问者状态
#[derive(Debug, Clone)]
pub struct ExpressionVisitorState {
    /// 是否继续访问
    pub continue_visiting: bool,
    /// 访问深度
    pub depth: usize,
    /// 访问计数
    pub visit_count: usize,
    /// 最大深度限制
    pub max_depth: Option<usize>,
    /// 表达式类型过滤器
    pub type_filter: Option<ExpressionTypeFilter>,
    /// 自定义状态数据
    pub custom_data: HashMap<String, Value>,
}

impl ExpressionVisitorState {
    /// 创建新的访问者状态
    pub fn new() -> Self {
        Self {
            continue_visiting: true,
            depth: 0,
            visit_count: 0,
            max_depth: None,
            type_filter: None,
            custom_data: HashMap::new(),
        }
    }

    /// 获取访问深度
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// 获取访问计数
    pub fn visit_count(&self) -> usize {
        self.visit_count
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

    /// 设置表达式类型过滤器
    pub fn set_type_filter(&mut self, filter: ExpressionTypeFilter) {
        self.type_filter = Some(filter);
    }

    /// 清除表达式类型过滤器
    pub fn clear_type_filter(&mut self) {
        self.type_filter = None;
    }

    /// 检查表达式类型是否被过滤
    pub fn is_type_filtered(&self, expr_type: &ExpressionType) -> bool {
        if let Some(ref filter) = self.type_filter {
            !filter.contains(expr_type)
        } else {
            false
        }
    }

    /// 设置自定义数据
    pub fn set_custom_data(&mut self, key: String, value: Value) {
        self.custom_data.insert(key, value);
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

/// 表达式类型过滤器
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

/// 表达式访问者结果类型
pub type VisitorResult<T> = Result<T, VisitorError>;

/// 表达式访问者错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisitorError {
    /// 超过最大深度限制
    MaxDepthExceeded,
    /// 访问被停止
    VisitationStopped,
    /// 无效的表达式类型
    InvalidExpressionType(String),
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
            VisitorError::InvalidExpressionType(msg) => write!(f, "无效的表达式类型: {}", msg),
            VisitorError::TypeMismatch(msg) => write!(f, "类型不匹配: {}", msg),
            VisitorError::Custom(msg) => write!(f, "自定义错误: {}", msg),
        }
    }
}

impl std::error::Error for VisitorError {}

/// 表达式深度优先遍历器trait
///
/// 提供深度优先遍历表达式树的默认实现
pub trait ExpressionDepthFirstVisitor: ExpressionVisitor {
    /// 访问子表达式
    fn visit_children(&mut self, expr: &Expression) -> Self::Result {
        for child in expr.children() {
            self.visit_expression(child);
        }
        self.default_result()
    }

    /// 默认结果
    fn default_result(&self) -> Self::Result;

    /// 带深度控制的访问
    fn visit_with_depth(&mut self, expr: &Expression) -> VisitorResult<Self::Result> {
        {
            let state = self.state_mut();
            state.increment_depth();

            if state.exceeds_max_depth() {
                state.decrement_depth();
                return Err(VisitorError::MaxDepthExceeded);
            }
        }

        let result = Ok(self.visit_expression(expr));

        let state = self.state_mut();
        state.decrement_depth();
        result
    }
}

/// 表达式转换器trait
///
/// 允许访问者修改和转换表达式树
pub trait ExpressionTransformer: ExpressionVisitor<Result = Expression> {
    /// 转换表达式
    fn transform(&mut self, expr: &Expression) -> Expression {
        self.visit_expression(expr)
    }

    /// 转换子表达式
    fn transform_children(&mut self, expr: &Expression) -> Expression {
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

/// 表达式接受器trait（AST Expr类型）
pub trait ExprAcceptor {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result;
}

impl ExprAcceptor for Expr {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_expr(self)
    }
}

/// 表达式访问者辅助trait - 提供额外的实用方法
pub trait ExpressionVisitorExt: ExpressionVisitor {
    /// 检查表达式是否包含特定类型
    fn contains_expression_type(&mut self, expr: &Expression, expr_type: &ExpressionType) -> bool {
        if &expr.expression_type() == expr_type {
            return true;
        }

        for child in expr.children() {
            if self.contains_expression_type(child, expr_type) {
                return true;
            }
        }

        false
    }

    /// 获取表达式树的最大深度
    fn max_depth(&mut self, expr: &Expression) -> usize {
        if expr.children().is_empty() {
            return 1;
        }

        let max_child_depth = expr
            .children()
            .iter()
            .map(|child| self.max_depth(child))
            .max()
            .unwrap_or(0);

        max_child_depth + 1
    }

    /// 获取表达式树中的所有变量名
    fn collect_variables(&mut self, expr: &Expression) -> Vec<String> {
        let mut variables = Vec::new();

        if let Expression::Variable(name) = expr {
            variables.push(name.clone());
        }

        for child in expr.children() {
            variables.extend(self.collect_variables(child));
        }

        variables
    }

    /// 检查表达式是否有效（无循环引用等）
    fn is_valid_expression(&mut self, expr: &Expression) -> bool {
        let mut visited = std::collections::HashSet::new();
        self.check_no_cycles(expr, &mut visited)
    }

    fn check_no_cycles(
        &mut self,
        expr: &Expression,
        visited: &mut std::collections::HashSet<usize>,
    ) -> bool {
        let expr_id = expr as *const _ as usize;

        if visited.contains(&expr_id) {
            return false;
        }

        visited.insert(expr_id);

        for child in expr.children() {
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
