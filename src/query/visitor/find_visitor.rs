//! FindVisitor - 用于查找表达式中特定类型子表达式的访问器
//! 对应 NebulaGraph FindVisitor.h/.cpp 的功能

use crate::core::visitor::{VisitorContext, VisitorCore, VisitorResult};
use crate::core::{
    AggregateFunction, BinaryOperator, DataType, Expression, ExpressionVisitor, LiteralValue,
    UnaryOperator,
};
use crate::query::visitor::QueryVisitor;
use std::collections::HashSet;

#[derive(Debug)]
pub struct FindVisitor {
    /// 要查找的表达式类型集合
    target_types: HashSet<ExpressionType>,
    /// 找到的表达式列表
    found_exprs: Vec<Expression>,
    /// 访问器上下文
    context: VisitorContext,
    /// 访问器状态
    state: crate::core::visitor::visitor_state_enum::VisitorStateEnum,
}

/// 表达式类型枚举，用于标识不同类型的表达式
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionType {
    Literal,
    Variable,
    Property,
    Binary,
    Unary,
    Function,
    Aggregate,
    List,
    Map,
    Case,
    TypeCast,
    Subscript,
    Range,
    Path,
    Label,
    TagProperty,
    EdgeProperty,
    InputProperty,
    VariableProperty,
    SourceProperty,
    DestinationProperty,
}

impl FindVisitor {
    pub fn new() -> Self {
        Self {
            target_types: HashSet::new(),
            found_exprs: Vec::new(),
            context: VisitorContext::new(crate::core::visitor::VisitorConfig::new()),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::new(),
        }
    }

    /// 创建带初始深度的 FindVisitor
    pub fn with_depth(depth: usize) -> Self {
        Self {
            target_types: HashSet::new(),
            found_exprs: Vec::new(),
            context: VisitorContext::new(crate::core::visitor::VisitorConfig::new()),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::with_depth(depth),
        }
    }

    /// 创建带配置的 FindVisitor
    pub fn with_config(config: crate::core::visitor::VisitorConfig) -> Self {
        Self {
            target_types: HashSet::new(),
            found_exprs: Vec::new(),
            context: VisitorContext::new(config),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::new(),
        }
    }

    /// 创建带配置和初始深度的 FindVisitor
    pub fn with_config_and_depth(
        config: crate::core::visitor::VisitorConfig,
        depth: usize,
    ) -> Self {
        Self {
            target_types: HashSet::new(),
            found_exprs: Vec::new(),
            context: VisitorContext::new(config),
            state: crate::core::visitor::visitor_state_enum::VisitorStateEnum::with_depth(depth),
        }
    }

    /// 设置要查找的表达式类型
    pub fn set_target_types(&mut self, types: Vec<ExpressionType>) -> &mut Self {
        self.target_types.clear();
        for expr_type in types {
            self.target_types.insert(expr_type);
        }
        self
    }

    /// 添加要查找的表达式类型
    pub fn add_target_type(&mut self, expr_type: ExpressionType) -> &mut Self {
        self.target_types.insert(expr_type);
        self
    }

    /// 搜索表达式中匹配类型的所有子表达式
    pub fn find(&mut self, expr: &Expression) -> Vec<Expression> {
        self.found_exprs.clear();
        self.visit(expr);
        self.found_exprs.clone()
    }

    /// 检查表达式中是否存在匹配类型的子表达式
    pub fn exist(&mut self, expr: &Expression) -> bool {
        self.found_exprs.clear();
        self.visit(expr);
        !self.found_exprs.is_empty()
    }

    fn visit(&mut self, expr: &Expression) {
        // 检查当前表达式是否匹配目标类型
        if self.target_types.contains(&Self::get_expression_type(expr)) {
            self.found_exprs.push(expr.clone());
        }

        // 递归访问子表达式
        self.visit_children(expr);
    }

    fn visit_children(&mut self, expr: &Expression) {
        match expr {
            Expression::Literal(_) => {}
            Expression::Variable(_) => {}
            Expression::Property { object, .. } => {
                self.visit(object);
            }
            Expression::Binary { left, right, .. } => {
                self.visit(left);
                self.visit(right);
            }
            Expression::Unary { operand, .. } => {
                self.visit(operand);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.visit(arg);
                }
            }
            Expression::Aggregate { arg, .. } => {
                self.visit(arg);
            }
            Expression::List(elements) => {
                for elem in elements {
                    self.visit(elem);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.visit(value);
                }
            }
            Expression::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    self.visit(condition);
                    self.visit(value);
                }
                if let Some(default_expr) = default {
                    self.visit(default_expr);
                }
            }
            Expression::TypeCast { expr, .. } => {
                self.visit(expr);
            }
            Expression::Subscript { collection, index } => {
                self.visit(collection);
                self.visit(index);
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                self.visit(collection);
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr);
                }
            }
            Expression::Path(elements) => {
                for elem in elements {
                    self.visit(elem);
                }
            }
            Expression::Label(_) => {}
            Expression::TagProperty { .. } => {}
            Expression::EdgeProperty { .. } => {}
            Expression::InputProperty(_) => {}
            Expression::VariableProperty { .. } => {}
            Expression::SourceProperty { .. } => {}
            Expression::DestinationProperty { .. } => {}

            // 新增表达式类型的处理
            Expression::UnaryPlus(expr)
            | Expression::UnaryNegate(expr)
            | Expression::UnaryNot(expr)
            | Expression::UnaryIncr(expr)
            | Expression::UnaryDecr(expr)
            | Expression::IsNull(expr)
            | Expression::IsNotNull(expr)
            | Expression::IsEmpty(expr)
            | Expression::IsNotEmpty(expr) => {
                self.visit(expr);
            }
            Expression::TypeCasting { expr, .. } => {
                self.visit(expr);
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                self.visit(generator);
                if let Some(cond) = condition {
                    self.visit(cond);
                }
            }
            Expression::Predicate { list, condition } => {
                self.visit(list);
                self.visit(condition);
            }
            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => {
                self.visit(list);
                self.visit(initial);
                self.visit(expr);
            }
            Expression::PathBuild(elements) => {
                for elem in elements {
                    self.visit(elem);
                }
            }
            Expression::ESQuery(_) => {}
            Expression::UUID => {}
            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                self.visit(collection);
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr);
                }
            }
            Expression::MatchPathPattern { patterns, .. } => {
                for pattern in patterns {
                    self.visit(pattern);
                }
            }
        }
    }

    /// 获取表达式的类型
    fn get_expression_type(expr: &Expression) -> ExpressionType {
        match expr {
            Expression::Literal(_) => ExpressionType::Literal,
            Expression::Variable(_) => ExpressionType::Variable,
            Expression::Property { .. } => ExpressionType::Property,
            Expression::Binary { .. } => ExpressionType::Binary,
            Expression::Unary { .. } => ExpressionType::Unary,
            Expression::Function { .. } => ExpressionType::Function,
            Expression::Aggregate { .. } => ExpressionType::Aggregate,
            Expression::List(_) => ExpressionType::List,
            Expression::Map(_) => ExpressionType::Map,
            Expression::Case { .. } => ExpressionType::Case,
            Expression::TypeCast { .. } => ExpressionType::TypeCast,
            Expression::Subscript { .. } => ExpressionType::Subscript,
            Expression::Range { .. } => ExpressionType::Range,
            Expression::Path(_) => ExpressionType::Path,
            Expression::Label(_) => ExpressionType::Label,
            Expression::TagProperty { .. } => ExpressionType::TagProperty,
            Expression::EdgeProperty { .. } => ExpressionType::EdgeProperty,
            Expression::InputProperty(_) => ExpressionType::InputProperty,
            Expression::VariableProperty { .. } => ExpressionType::VariableProperty,
            Expression::SourceProperty { .. } => ExpressionType::SourceProperty,
            Expression::DestinationProperty { .. } => ExpressionType::DestinationProperty,

            // 新增表达式类型的处理
            Expression::UnaryPlus(_)
            | Expression::UnaryNegate(_)
            | Expression::UnaryNot(_)
            | Expression::UnaryIncr(_)
            | Expression::UnaryDecr(_)
            | Expression::IsNull(_)
            | Expression::IsNotNull(_)
            | Expression::IsEmpty(_)
            | Expression::IsNotEmpty(_) => ExpressionType::Unary,
            Expression::TypeCasting { .. } => ExpressionType::TypeCast,
            Expression::ListComprehension { .. } => ExpressionType::List,
            Expression::Predicate { .. } => ExpressionType::Property,
            Expression::Reduce { .. } => ExpressionType::Aggregate,
            Expression::PathBuild(_) => ExpressionType::Path,
            Expression::ESQuery(_) => ExpressionType::Function,
            Expression::UUID => ExpressionType::Literal,
            Expression::SubscriptRange { .. } => ExpressionType::Subscript,
            Expression::MatchPathPattern { .. } => ExpressionType::Path,
        }
    }

    /// 搜索表达式中匹配特定条件的子表达式
    pub fn find_if<F>(&mut self, expr: &Expression, predicate: F) -> Vec<Expression>
    where
        F: Fn(&Expression) -> bool,
    {
        let mut results = Vec::new();
        self.visit_with_predicate(expr, &predicate, &mut results);
        results
    }

    fn visit_with_predicate<F>(
        &self,
        expr: &Expression,
        predicate: &F,
        results: &mut Vec<Expression>,
    ) where
        F: Fn(&Expression) -> bool,
    {
        if predicate(expr) {
            results.push(expr.clone());
        }

        // 递归访问子表达式
        match expr {
            Expression::Literal(_) => {}
            Expression::Variable(_) => {}
            Expression::Property { object, .. } => {
                self.visit_with_predicate(object, predicate, results);
            }
            Expression::Binary { left, right, .. } => {
                self.visit_with_predicate(left, predicate, results);
                self.visit_with_predicate(right, predicate, results);
            }
            Expression::Unary { operand, .. } => {
                self.visit_with_predicate(operand, predicate, results);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.visit_with_predicate(arg, predicate, results);
                }
            }
            Expression::Aggregate { arg, .. } => {
                self.visit_with_predicate(arg, predicate, results);
            }
            Expression::List(elements) => {
                for elem in elements {
                    self.visit_with_predicate(elem, predicate, results);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.visit_with_predicate(value, predicate, results);
                }
            }
            Expression::Case {
                conditions,
                default,
            } => {
                for (condition, value) in conditions {
                    self.visit_with_predicate(condition, predicate, results);
                    self.visit_with_predicate(value, predicate, results);
                }
                if let Some(default_expr) = default {
                    self.visit_with_predicate(default_expr, predicate, results);
                }
            }
            Expression::TypeCast { expr, .. } => {
                self.visit_with_predicate(expr, predicate, results);
            }
            Expression::Subscript { collection, index } => {
                self.visit_with_predicate(collection, predicate, results);
                self.visit_with_predicate(index, predicate, results);
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                self.visit_with_predicate(collection, predicate, results);
                if let Some(start_expr) = start {
                    self.visit_with_predicate(start_expr, predicate, results);
                }
                if let Some(end_expr) = end {
                    self.visit_with_predicate(end_expr, predicate, results);
                }
            }
            Expression::Path(elements) => {
                for elem in elements {
                    self.visit_with_predicate(elem, predicate, results);
                }
            }
            Expression::Label(_) => {}
            Expression::TagProperty { .. } => {}
            Expression::EdgeProperty { .. } => {}
            Expression::InputProperty(_) => {}
            Expression::VariableProperty { .. } => {}
            Expression::SourceProperty { .. } => {}
            Expression::DestinationProperty { .. } => {}

            // 新增表达式类型的处理
            Expression::UnaryPlus(expr)
            | Expression::UnaryNegate(expr)
            | Expression::UnaryNot(expr)
            | Expression::UnaryIncr(expr)
            | Expression::UnaryDecr(expr)
            | Expression::IsNull(expr)
            | Expression::IsNotNull(expr)
            | Expression::IsEmpty(expr)
            | Expression::IsNotEmpty(expr) => {
                self.visit_with_predicate(expr, predicate, results);
            }
            Expression::TypeCasting { expr, .. } => {
                self.visit_with_predicate(expr, predicate, results);
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                self.visit_with_predicate(generator, predicate, results);
                if let Some(cond) = condition {
                    self.visit_with_predicate(cond, predicate, results);
                }
            }
            Expression::Predicate { list, condition } => {
                self.visit_with_predicate(list, predicate, results);
                self.visit_with_predicate(condition, predicate, results);
            }
            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => {
                self.visit_with_predicate(list, predicate, results);
                self.visit_with_predicate(initial, predicate, results);
                self.visit_with_predicate(expr, predicate, results);
            }
            Expression::PathBuild(elements) => {
                for elem in elements {
                    self.visit_with_predicate(elem, predicate, results);
                }
            }
            Expression::ESQuery(_) => {}
            Expression::UUID => {}
            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                self.visit_with_predicate(collection, predicate, results);
                if let Some(start_expr) = start {
                    self.visit_with_predicate(start_expr, predicate, results);
                }
                if let Some(end_expr) = end {
                    self.visit_with_predicate(end_expr, predicate, results);
                }
            }
            Expression::MatchPathPattern { patterns, .. } => {
                for pattern in patterns {
                    self.visit_with_predicate(pattern, predicate, results);
                }
            }
        }
    }
}

impl VisitorCore<Expression> for FindVisitor {
    type Result = ();

    fn visit(&mut self, target: &Expression) -> Self::Result {
        // 使用表达式访问器模式进行访问
        match target {
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
            } => {
                let default_cloned = default.map(|b| (**b).clone());
                self.visit_case(conditions, &default_cloned)
            }
            Expression::TypeCast { expr, target_type } => self.visit_type_cast(expr, target_type),
            Expression::Subscript { collection, index } => self.visit_subscript(collection, index),
            Expression::Range {
                collection,
                start,
                end,
            } => {
                let start_cloned = start.map(|b| (**b).clone());
                let end_cloned = end.map(|b| (**b).clone());
                self.visit_range(collection, &start_cloned, &end_cloned)
            }
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

            // 处理新增的表达式类型
            Expression::UnaryPlus(expr) => self.visit_unary(&UnaryOperator::Plus, expr),
            Expression::UnaryNegate(expr) => self.visit_unary(&UnaryOperator::Minus, expr),
            Expression::UnaryNot(expr) => self.visit_unary(&UnaryOperator::Not, expr),
            Expression::UnaryIncr(expr) => self.visit_unary(&UnaryOperator::Increment, expr),
            Expression::UnaryDecr(expr) => self.visit_unary(&UnaryOperator::Decrement, expr),
            Expression::IsNull(expr) => self.visit_unary(&UnaryOperator::IsNull, expr),
            Expression::IsNotNull(expr) => self.visit_unary(&UnaryOperator::IsNotNull, expr),
            Expression::IsEmpty(expr) => self.visit_unary(&UnaryOperator::IsEmpty, expr),
            Expression::IsNotEmpty(expr) => self.visit_unary(&UnaryOperator::IsNotEmpty, expr),

            Expression::TypeCasting { expr, .. } => self.visit_type_cast(expr, &DataType::String),
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                // 简化为函数调用
                let cond_expr = condition
                    .map(|c| (**c).clone())
                    .unwrap_or(Expression::bool(true));
                self.visit_function("list_comprehension", &[(**generator).clone(), cond_expr])
            }
            Expression::Predicate { list, condition } => {
                self.visit_function("predicate", &[(**list).clone(), (**condition).clone()])
            }
            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => self.visit_function(
                "reduce",
                &[(**list).clone(), (**initial).clone(), (**expr).clone()],
            ),
            Expression::PathBuild(items) => self.visit_path(items),
            Expression::ESQuery(query) => {
                self.visit_function("es_query", &[Expression::string(query)])
            }
            Expression::UUID => self.visit_function("uuid", &[]),
            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                let start_cloned = start.map(|b| (**b).clone());
                let end_cloned = end.map(|b| (**b).clone());
                self.visit_range(collection, &start_cloned, &end_cloned)
            }
            Expression::MatchPathPattern { patterns, .. } => self.visit_list(patterns),
        }
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

impl ExpressionVisitor for FindVisitor {
    fn visit_literal(&mut self, value: &LiteralValue) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Literal) {
            self.found_exprs.push(Expression::Literal(value.clone()));
        }
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Variable) {
            self.found_exprs
                .push(Expression::Variable(name.to_string()));
        }
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Property) {
            self.found_exprs.push(Expression::Property {
                object: Box::new(object.clone()),
                property: property.to_string(),
            });
        }
        self.visit(object);
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Binary) {
            self.found_exprs.push(Expression::Binary {
                left: Box::new(left.clone()),
                op: op.clone(),
                right: Box::new(right.clone()),
            });
        }
        self.visit(left);
        self.visit(right);
    }

    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Unary) {
            self.found_exprs.push(Expression::Unary {
                op: op.clone(),
                operand: Box::new(operand.clone()),
            });
        }
        self.visit(operand);
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Function) {
            self.found_exprs.push(Expression::Function {
                name: name.to_string(),
                args: args.to_vec(),
            });
        }
        for arg in args {
            self.visit(arg);
        }
    }

    fn visit_aggregate(
        &mut self,
        func: &AggregateFunction,
        arg: &Expression,
        _distinct: bool,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Aggregate) {
            self.found_exprs.push(Expression::Aggregate {
                func: func.clone(),
                arg: Box::new(arg.clone()),
                distinct: false,
            });
        }
        self.visit(arg);
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        if self.target_types.contains(&ExpressionType::List) {
            self.found_exprs.push(Expression::List(items.to_vec()));
        }
        for item in items {
            self.visit(item);
        }
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Map) {
            self.found_exprs.push(Expression::Map(pairs.to_vec()));
        }
        for (_, value) in pairs {
            self.visit(value);
        }
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Expression>,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Case) {
            self.found_exprs.push(Expression::Case {
                conditions: conditions.to_vec(),
                default: default.map(|e| Box::new(e.clone())),
            });
        }
        for (condition, value) in conditions {
            self.visit(condition);
            self.visit(value);
        }
        if let Some(default_expr) = default {
            self.visit(default_expr);
        }
    }

    fn visit_type_cast(&mut self, expr: &Expression, target_type: &DataType) -> Self::Result {
        if self.target_types.contains(&ExpressionType::TypeCast) {
            self.found_exprs.push(Expression::TypeCast {
                expr: Box::new(expr.clone()),
                target_type: target_type.clone(),
            });
        }
        self.visit(expr);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Subscript) {
            self.found_exprs.push(Expression::Subscript {
                collection: Box::new(collection.clone()),
                index: Box::new(index.clone()),
            });
        }
        self.visit(collection);
        self.visit(index);
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Expression>,
        end: &Option<Expression>,
    ) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Range) {
            self.found_exprs.push(Expression::Range {
                collection: Box::new(collection.clone()),
                start: start.map(|e| Box::new(e.clone())),
                end: end.map(|e| Box::new(e.clone())),
            });
        }
        self.visit(collection);
        if let Some(start_expr) = start {
            self.visit(start_expr);
        }
        if let Some(end_expr) = end {
            self.visit(end_expr);
        }
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Path) {
            self.found_exprs.push(Expression::Path(items.to_vec()));
        }
        for item in items {
            self.visit(item);
        }
    }

    fn visit_label(&mut self, name: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::Label) {
            self.found_exprs.push(Expression::Label(name.to_string()));
        }
    }

    fn visit_tag_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::TagProperty) {
            self.found_exprs.push(Expression::TagProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
    }

    fn visit_edge_property(&mut self, edge: &str, prop: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::EdgeProperty) {
            self.found_exprs.push(Expression::EdgeProperty {
                edge: edge.to_string(),
                prop: prop.to_string(),
            });
        }
    }

    fn visit_input_property(&mut self, prop: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::InputProperty) {
            self.found_exprs
                .push(Expression::InputProperty(prop.to_string()));
        }
    }

    fn visit_variable_property(&mut self, var: &str, prop: &str) -> Self::Result {
        if self
            .target_types
            .contains(&ExpressionType::VariableProperty)
        {
            self.found_exprs.push(Expression::VariableProperty {
                var: var.to_string(),
                prop: prop.to_string(),
            });
        }
    }

    fn visit_source_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if self.target_types.contains(&ExpressionType::SourceProperty) {
            self.found_exprs.push(Expression::SourceProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
    }

    fn visit_destination_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if self
            .target_types
            .contains(&ExpressionType::DestinationProperty)
        {
            self.found_exprs.push(Expression::DestinationProperty {
                tag: tag.to_string(),
                prop: prop.to_string(),
            });
        }
    }
}

impl QueryVisitor for FindVisitor {
    type QueryResult = Vec<Expression>;

    fn get_result(&self) -> Self::QueryResult {
        self.found_exprs.clone()
    }

    fn reset(&mut self) {
        self.found_exprs.clear();
    }

    fn is_success(&self) -> bool {
        true // FindVisitor 总是成功，即使没有找到任何表达式
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{BinaryOperator, LiteralValue};

    #[test]
    fn test_find_literals() {
        let mut visitor = FindVisitor::new();

        // 创建一个包含字面量的表达式: 1 + 2 * 3
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(LiteralValue::Int(2))),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(LiteralValue::Int(3))),
            }),
        };

        let literals = visitor.add_target_type(ExpressionType::Literal).find(&expr);

        // 应该找到3个字面量
        assert_eq!(literals.len(), 3);
    }

    #[test]
    fn test_find_with_predicate() {
        let mut visitor = FindVisitor::new();

        // 创建一个包含整数字面量的表达式: 1 + 2 * 3
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(LiteralValue::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Binary {
                left: Box::new(Expression::Literal(LiteralValue::Int(2))),
                op: BinaryOperator::Multiply,
                right: Box::new(Expression::Literal(LiteralValue::Int(3))),
            }),
        };

        let literals = visitor.find_if(&expr, |e| {
            matches!(e, Expression::Literal(LiteralValue::Int(_)))
        });

        // 应该找到3个整数字面量
        assert_eq!(literals.len(), 3);
    }
}
