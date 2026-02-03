//! PropertyTrackerVisitor - 用于跟踪表达式中使用的属性
//!
//! 主要功能：
//! - 记录顶点属性使用情况（按标签ID和属性名）
//! - 记录边属性使用情况（按边类型和属性名）
//! - 支持属性更新和别名映射
//! - 用于属性剪裁优化

use crate::core::types::expression::Expression;
use crate::core::types::expression::visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::{
    BinaryOperator, DataType, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
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
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 创建带有实体别名的访问器
    pub fn with_alias(_alias: String) -> Self {
        Self {
            props_used: PropertyTracker::new(),
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 跟踪表达式中的属性
    pub fn track(&mut self, expression: &Expression) -> Result<PropertyTracker, String> {
        self.props_used = PropertyTracker::new();
        self.error = None;

        self.visit_expression(expression)?;

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

    fn visit_label(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }

    fn visit_list(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_list_comprehension(
        &mut self,
        _variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(source)?;
        if let Some(f) = filter {
            self.visit_expression(f)?;
        }
        if let Some(m) = map {
            self.visit_expression(m)?;
        }
        Ok(())
    }

    fn visit_map(&mut self, pairs: &[(String, Expression)]) -> Self::Result {
        for (_key, value) in pairs {
            self.visit_expression(value)?;
        }
        Ok(())
    }

    fn visit_case(&mut self, test_expr: Option<&Expression>, conditions: &[(Expression, Expression)], default: Option<&Expression>) -> Self::Result {
        if let Some(test) = test_expr {
            self.visit_expression(test)?;
        }
        for (when_expression, then_expression) in conditions {
            self.visit_expression(when_expression)?;
            self.visit_expression(then_expression)?;
        }
        if let Some(default_expression) = default {
            self.visit_expression(default_expression)?;
        }
        Ok(())
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: &DataType) -> Self::Result {
        self.visit_expression(expression)
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) -> Self::Result {
        self.visit_expression(collection)?;
        self.visit_expression(index)?;
        Ok(())
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) -> Self::Result {
        self.visit_expression(collection)?;
        if let Some(start_expression) = start {
            self.visit_expression(start_expression)?;
        }
        if let Some(end_expression) = end {
            self.visit_expression(end_expression)?;
        }
        Ok(())
    }

    fn visit_path(&mut self, items: &[Expression]) -> Self::Result {
        for item in items {
            self.visit_expression(item)?;
        }
        Ok(())
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        self.visit_expression(object)?;
        self.props_used.insert_col(property);
        Ok(())
    }

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}
