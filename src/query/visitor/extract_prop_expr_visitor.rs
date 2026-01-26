//! ExtractPropExprVisitor - 用于提取属性表达式的访问器
//!
//! 主要功能：
//! - 从表达式中提取属性表达式
//! - 区分源属性、边属性、目标属性、输入属性
//! - 构建属性表达式到列的映射
//! - 支持属性去重

use crate::core::types::expression::Expression;
use crate::core::types::expression::visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::{
    BinaryOperator, DataType, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use std::collections::{HashMap, HashSet};

/// 属性表达式提取结果
#[derive(Debug, Clone, Default)]
pub struct ExtractedProps {
    /// 源属性表达式：$^.tagName.propName
    pub src_props: Vec<(String, String)>,
    /// 边属性表达式：edgeName.propName
    pub edge_props: Vec<(String, String)>,
    /// 目标属性表达式：$$.tagName.propName
    pub dst_props: Vec<(String, String)>,
    /// 输入属性表达式：$-.propName
    pub input_props: Vec<String>,
    /// 属性表达式到列的映射
    pub prop_expr_col_map: HashMap<String, String>,
    /// 唯一的边/顶点列集合
    pub unique_edge_vertex_cols: HashSet<String>,
}

impl ExtractedProps {
    /// 创建新的属性提取结果
    pub fn new() -> Self {
        Self::default()
    }

    /// 插入源属性
    pub fn insert_src_prop(&mut self, tag: String, prop: String) {
        let expression = format!("$^.{}.{}", tag, prop);
        if !self.src_props.contains(&(tag.clone(), prop.clone())) {
            self.src_props.push((tag, prop));
            self.unique_edge_vertex_cols.insert(expression);
        }
    }

    /// 插入边属性
    pub fn insert_edge_prop(&mut self, edge: String, prop: String) {
        let expression = format!("{}.{}", edge, prop);
        if !self.edge_props.contains(&(edge.clone(), prop.clone())) {
            self.edge_props.push((edge, prop));
            self.unique_edge_vertex_cols.insert(expression);
        }
    }

    /// 插入目标属性
    pub fn insert_dst_prop(&mut self, tag: String, prop: String) {
        let expression = format!("$$.{}.{}", tag, prop);
        if !self.dst_props.contains(&(tag.clone(), prop.clone())) {
            self.dst_props.push((tag, prop));
            self.unique_edge_vertex_cols.insert(expression);
        }
    }

    /// 插入输入属性
    pub fn insert_input_prop(&mut self, prop: String) {
        if !self.input_props.contains(&prop) {
            self.input_props.push(prop.clone());
        }
    }

    /// 插入属性表达式到列的映射
    pub fn insert_prop_expr_col(&mut self, expression: String, col: String) {
        self.prop_expr_col_map.insert(expression, col);
    }

    /// 合并另一个 ExtractedProps
    pub fn union(&mut self, other: &ExtractedProps) {
        for (tag, prop) in &other.src_props {
            self.insert_src_prop(tag.clone(), prop.clone());
        }

        for (edge, prop) in &other.edge_props {
            self.insert_edge_prop(edge.clone(), prop.clone());
        }

        for (tag, prop) in &other.dst_props {
            self.insert_dst_prop(tag.clone(), prop.clone());
        }

        for prop in &other.input_props {
            self.insert_input_prop(prop.clone());
        }

        for (expression, col) in &other.prop_expr_col_map {
            self.prop_expr_col_map.insert(expression.clone(), col.clone());
        }

        for col in &other.unique_edge_vertex_cols {
            self.unique_edge_vertex_cols.insert(col.clone());
        }
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.src_props.is_empty()
            && self.edge_props.is_empty()
            && self.dst_props.is_empty()
            && self.input_props.is_empty()
    }
}

/// 属性表达式提取访问器
///
/// 用于从表达式中提取属性表达式，分类存储
#[derive(Debug)]
pub struct ExtractPropExprVisitor {
    /// 提取到的属性
    extracted_props: ExtractedProps,
    /// 错误状态
    error: Option<String>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl ExtractPropExprVisitor {
    /// 创建新的属性表达式提取访问器
    pub fn new() -> Self {
        Self {
            extracted_props: ExtractedProps::new(),
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 提取属性表达式
    pub fn extract(&mut self, expression: &Expression) -> Result<ExtractedProps, String> {
        self.extracted_props = ExtractedProps::new();
        self.error = None;

        self.visit_expression(expression)?;

        if let Some(err) = &self.error {
            Err(err.clone())
        } else {
            Ok(self.extracted_props.clone())
        }
    }

    /// 获取提取到的属性
    pub fn get_extracted_props(&self) -> &ExtractedProps {
        &self.extracted_props
    }
}

impl Default for ExtractPropExprVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionVisitor for ExtractPropExprVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
        Ok(())
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.extracted_props.insert_input_prop(name.to_string());
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
                        self.extracted_props.insert_input_prop(alias.clone());
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
        for (_, expression) in pairs {
            self.visit_expression(expression)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result {
        for (cond, expression) in conditions {
            self.visit_expression(cond)?;
            self.visit_expression(expression)?;
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
        self.visit_expression(index)
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

    fn visit_label(&mut self, _name: &str) -> Self::Result {
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

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}
