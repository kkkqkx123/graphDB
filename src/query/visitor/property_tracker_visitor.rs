//! PropertyTrackerVisitor - 用于跟踪表达式中使用的属性
//!
//! 主要功能：
//! - 记录顶点属性使用情况（按标签ID和属性名）
//! - 记录边属性使用情况（按边类型和属性名）
//! - 支持属性更新和别名映射
//! - 用于属性剪裁优化

use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, Expression, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use crate::query::parser::ast::expr::*;
use std::collections::{HashMap, HashSet};

/// 属性跟踪器
///
/// 跟踪查询中使用的所有属性，用于属性剪裁优化
#[derive(Debug, Clone, Default)]
pub struct PropertyTracker {
    /// 顶点属性映射：别名 -> 标签ID -> 属性名集合
    pub vertex_props_map: HashMap<String, HashMap<String, HashSet<String>>>,
    /// 边属性映射：别名 -> 边类型 -> 属性名集合
    pub edge_props_map: HashMap<String, HashMap<String, HashSet<String>>>,
    /// 列集合
    pub cols_set: HashSet<String>,
}

impl PropertyTracker {
    /// 创建新的属性跟踪器
    pub fn new() -> Self {
        Self::default()
    }

    /// 更新属性别名
    pub fn update(&mut self, old_name: &str, new_name: &str) -> Result<(), String> {
        if let Some(vertex_props) = self.vertex_props_map.remove(old_name) {
            self.vertex_props_map.insert(new_name.to_string(), vertex_props);
        }

        if let Some(edge_props) = self.edge_props_map.remove(old_name) {
            self.edge_props_map.insert(new_name.to_string(), edge_props);
        }

        if self.cols_set.contains(old_name) {
            self.cols_set.remove(old_name);
            self.cols_set.insert(new_name.to_string());
        }

        Ok(())
    }

    /// 检查是否存在别名
    pub fn has_alias(&self, name: &str) -> bool {
        self.vertex_props_map.contains_key(name)
            || self.edge_props_map.contains_key(name)
            || self.cols_set.contains(name)
    }

    /// 插入顶点属性
    pub fn insert_vertex_prop(&mut self, name: &str, tag_id: &str, prop_name: &str) {
        self.vertex_props_map
            .entry(name.to_string())
            .or_insert_with(HashMap::new)
            .entry(tag_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop_name.to_string());
    }

    /// 插入边属性
    pub fn insert_edge_prop(&mut self, name: &str, edge_type: &str, prop_name: &str) {
        self.edge_props_map
            .entry(name.to_string())
            .or_insert_with(HashMap::new)
            .entry(edge_type.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop_name.to_string());
    }

    /// 插入列
    pub fn insert_col(&mut self, name: &str) {
        self.cols_set.insert(name.to_string());
    }

    /// 合并另一个 PropertyTracker
    pub fn union(&mut self, other: &PropertyTracker) {
        for (name, tag_map) in &other.vertex_props_map {
            for (tag_id, props) in tag_map {
                for prop in props {
                    self.insert_vertex_prop(name, tag_id, prop);
                }
            }
        }

        for (name, edge_map) in &other.edge_props_map {
            for (edge_type, props) in edge_map {
                for prop in props {
                    self.insert_edge_prop(name, edge_type, prop);
                }
            }
        }

        for col in &other.cols_set {
            self.insert_col(col);
        }
    }

    /// 获取顶点属性
    pub fn get_vertex_props(&self, name: &str) -> Option<&HashMap<String, HashSet<String>>> {
        self.vertex_props_map.get(name)
    }

    /// 获取边属性
    pub fn get_edge_props(&self, name: &str) -> Option<&HashMap<String, HashSet<String>>> {
        self.edge_props_map.get(name)
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.vertex_props_map.is_empty()
            && self.edge_props_map.is_empty()
            && self.cols_set.is_empty()
    }
}

/// 属性跟踪访问器
///
/// 用于遍历表达式并跟踪所有使用的属性
#[derive(Debug)]
pub struct PropertyTrackerVisitor {
    /// 属性跟踪器
    props_used: PropertyTracker,
    /// 当前实体别名
    entity_alias: Option<String>,
    /// 错误状态
    error: Option<String>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl PropertyTrackerVisitor {
    /// 创建新的属性跟踪访问器
    pub fn new() -> Self {
        Self {
            props_used: PropertyTracker::new(),
            entity_alias: None,
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 创建带有实体别名的访问器
    pub fn with_alias(alias: String) -> Self {
        Self {
            props_used: PropertyTracker::new(),
            entity_alias: Some(alias),
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 跟踪表达式中的属性
    pub fn track(&mut self, expr: &Expression) -> Result<PropertyTracker, String> {
        self.props_used = PropertyTracker::new();
        self.error = None;

        self.visit_expression(expr)?;

        if let Some(err) = &self.error {
            Err(err.clone())
        } else {
            Ok(self.props_used.clone())
        }
    }

    /// 获取属性跟踪器
    pub fn get_props_used(&self) -> &PropertyTracker {
        &self.props_used
    }

    /// 设置错误信息
    fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }
}

impl Default for PropertyTrackerVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionVisitor for PropertyTrackerVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
        Ok(())
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.props_used.insert_col(name);
        Ok(())
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
        self.visit_expression(right)
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        let name_upper = name.to_uppercase();

        match name_upper.as_str() {
            "ID" | "SRC" | "DST" => {
                if !args.is_empty() {
                    if let Expression::Variable(alias) = &args[0] {
                        self.props_used.insert_col(alias);
                    }
                }
            }
            _ => {}
        }

        for arg in args {
            self.visit_expression(arg)?;
        }
        Ok(())
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
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_, expr) in pairs {
            self.visit_expression(expr)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) -> Self::Result {
        for (cond, expr) in conditions {
            self.visit_expression(cond)?;
            self.visit_expression(expr)?;
        }
        if let Some(default_expr) = default {
            self.visit_expression(default_expr)?;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expr: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expr)
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
        if let Some(start_expr) = start {
            self.visit_expression(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit_expression(end_expr)?;
        }
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }

    fn visit_tag_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if let Some(alias) = &self.entity_alias {
            self.props_used.insert_vertex_prop(alias, tag, prop);
        }
        Ok(())
    }

    fn visit_edge_property(&mut self, edge: &str, prop: &str) -> Self::Result {
        if let Some(alias) = &self.entity_alias {
            self.props_used.insert_edge_prop(alias, edge, prop);
        }
        Ok(())
    }

    fn visit_input_property(&mut self, prop: &str) -> Self::Result {
        self.props_used.insert_col(prop);
        Ok(())
    }

    fn visit_variable_property(&mut self, var: &str, prop: &str) -> Self::Result {
        self.props_used.insert_col(var);
        self.props_used.insert_col(prop);
        Ok(())
    }

    fn visit_source_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if let Some(alias) = &self.entity_alias {
            self.props_used.insert_vertex_prop(alias, tag, prop);
        }
        Ok(())
    }

    fn visit_destination_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        if let Some(alias) = &self.entity_alias {
            self.props_used.insert_vertex_prop(alias, tag, prop);
        }
        Ok(())
    }

    fn visit_unary_plus(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_unary_negate(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_unary_not(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_unary_incr(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_unary_decr(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_is_null(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_is_not_null(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_is_empty(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_is_not_empty(&mut self, expr: &Expression) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_list_comprehension(
        &mut self,
        generator: &Expression,
        condition: &Option<Box<Expression>>,
    ) -> Self::Result {
        self.visit_expression(generator)?;
        if let Some(cond) = condition {
            self.visit_expression(cond)?;
        }
        Ok(())
    }

    fn visit_predicate(&mut self, list: &Expression, condition: &Expression) -> Self::Result {
        self.visit_expression(list)?;
        self.visit_expression(condition)
    }

    fn visit_reduce(
        &mut self,
        list: &Expression,
        _var: &str,
        initial: &Expression,
        expr: &Expression,
    ) -> Self::Result {
        self.visit_expression(list)?;
        self.visit_expression(initial)?;
        self.visit_expression(expr)
    }

    fn visit_es_query(&mut self, _query: &str) -> Self::Result {
        Ok(())
    }

    fn visit_uuid(&mut self) -> Self::Result {
        Ok(())
    }

    fn visit_match_path_pattern(
        &mut self,
        _path_alias: &str,
        _patterns: &[Expression],
    ) -> Self::Result {
        Ok(())
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }

    fn visit_type_casting(&mut self, expr: &Expression, _target_type: &str) -> Self::Result {
        self.visit_expression(expr)
    }

    fn visit_path_build(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_subscript_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(start_expr) = start {
            self.visit_expression(start_expr)?;
        }
        if let Some(end_expr) = end {
            self.visit_expression(end_expr)?;
        }
        Ok(())
    }

    fn visit_constant_expr(&mut self, _expr: &ConstantExpr) -> Self::Result {
        Ok(())
    }

    fn visit_variable_expr(&mut self, expr: &VariableExpr) -> Self::Result {
        self.visit_variable(&expr.name)
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Self::Result {
        self.visit_expr(expr.left.as_ref())?;
        self.visit_expr(expr.right.as_ref())?;
        Ok(())
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Self::Result {
        self.visit_expr(expr.operand.as_ref())?;
        Ok(())
    }

    fn visit_function_call_expr(&mut self, expr: &FunctionCallExpr) -> Self::Result {
        for arg in &expr.args {
            self.visit_expr(arg)?;
        }
        Ok(())
    }

    fn visit_property_access_expr(&mut self, expr: &PropertyAccessExpr) -> Self::Result {
        self.visit_expr(expr.object.as_ref())?;
        Ok(())
    }

    fn visit_list_expr(&mut self, expr: &ListExpr) -> Self::Result {
        for element in &expr.elements {
            self.visit_expr(element)?;
        }
        Ok(())
    }

    fn visit_map_expr(&mut self, expr: &MapExpr) -> Self::Result {
        for (_key, value) in &expr.pairs {
            self.visit_expr(value)?;
        }
        Ok(())
    }

    fn visit_case_expr(&mut self, expr: &CaseExpr) -> Self::Result {
        for (when_expr, then_expr) in &expr.when_then_pairs {
            self.visit_expr(when_expr)?;
            self.visit_expr(then_expr)?;
        }
        if let Some(default_expr) = &expr.default {
            self.visit_expr(default_expr.as_ref())?;
        }
        Ok(())
    }

    fn visit_subscript_expr(&mut self, expr: &SubscriptExpr) -> Self::Result {
        self.visit_expr(expr.collection.as_ref())?;
        self.visit_expr(expr.index.as_ref())?;
        Ok(())
    }

    fn visit_predicate_expr(&mut self, expr: &PredicateExpr) -> Self::Result {
        self.visit_expr(expr.list.as_ref())?;
        self.visit_expr(expr.condition.as_ref())?;
        Ok(())
    }

    fn visit_tag_property_expr(&mut self, expr: &TagPropertyExpr) -> Self::Result {
        self.visit_tag_property(&expr.tag, &expr.prop)
    }

    fn visit_edge_property_expr(&mut self, expr: &EdgePropertyExpr) -> Self::Result {
        self.visit_edge_property(&expr.edge, &expr.prop)
    }

    fn visit_input_property_expr(&mut self, expr: &InputPropertyExpr) -> Self::Result {
        self.visit_input_property(&expr.prop)
    }

    fn visit_variable_property_expr(&mut self, expr: &VariablePropertyExpr) -> Self::Result {
        self.visit_variable_property(&expr.var, &expr.prop)
    }

    fn visit_source_property_expr(&mut self, expr: &SourcePropertyExpr) -> Self::Result {
        self.visit_source_property(&expr.tag, &expr.prop)
    }

    fn visit_destination_property_expr(&mut self, expr: &DestinationPropertyExpr) -> Self::Result {
        self.visit_destination_property(&expr.tag, &expr.prop)
    }

    fn visit_type_cast_expr(&mut self, expr: &TypeCastExpr) -> Self::Result {
        self.visit_expr(expr.expr.as_ref())
    }

    fn visit_range_expr(&mut self, expr: &RangeExpr) -> Self::Result {
        self.visit_expr(expr.collection.as_ref())?;
        if let Some(start_expr) = &expr.start {
            self.visit_expr(start_expr.as_ref())?;
        }
        if let Some(end_expr) = &expr.end {
            self.visit_expr(end_expr.as_ref())?;
        }
        Ok(())
    }

    fn visit_path_expr(&mut self, expr: &PathExpr) -> Self::Result {
        for element in &expr.elements {
            self.visit_expr(element)?;
        }
        Ok(())
    }

    fn visit_label_expr(&mut self, _expr: &LabelExpr) -> Self::Result {
        Ok(())
    }

    fn visit_reduce_expr(&mut self, expr: &ReduceExpr) -> Self::Result {
        self.visit_expr(expr.list.as_ref())?;
        self.visit_expr(expr.initial.as_ref())?;
        self.visit_expr(expr.expr.as_ref())
    }

    fn visit_list_comprehension_expr(&mut self, expr: &ListComprehensionExpr) -> Self::Result {
        self.visit_expr(expr.generator.as_ref())?;
        if let Some(condition_expr) = &expr.condition {
            self.visit_expr(condition_expr.as_ref())?;
        }
        Ok(())
    }
}
