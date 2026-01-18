//! ExtractPropExprVisitor - 用于提取属性表达式的访问器
//!
//! 主要功能：
//! - 从表达式中提取属性表达式
//! - 区分源属性、边属性、目标属性、输入属性
//! - 构建属性表达式到列的映射
//! - 支持属性去重

use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, Expression, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use crate::query::parser::ast::expr::*;
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
        let expr = format!("$^.{}.{}", tag, prop);
        if !self.src_props.contains(&(tag.clone(), prop.clone())) {
            self.src_props.push((tag, prop));
            self.unique_edge_vertex_cols.insert(expr);
        }
    }

    /// 插入边属性
    pub fn insert_edge_prop(&mut self, edge: String, prop: String) {
        let expr = format!("{}.{}", edge, prop);
        if !self.edge_props.contains(&(edge.clone(), prop.clone())) {
            self.edge_props.push((edge, prop));
            self.unique_edge_vertex_cols.insert(expr);
        }
    }

    /// 插入目标属性
    pub fn insert_dst_prop(&mut self, tag: String, prop: String) {
        let expr = format!("$$.{}.{}", tag, prop);
        if !self.dst_props.contains(&(tag.clone(), prop.clone())) {
            self.dst_props.push((tag, prop));
            self.unique_edge_vertex_cols.insert(expr);
        }
    }

    /// 插入输入属性
    pub fn insert_input_prop(&mut self, prop: String) {
        if !self.input_props.contains(&prop) {
            self.input_props.push(prop.clone());
        }
    }

    /// 插入属性表达式到列的映射
    pub fn insert_prop_expr_col(&mut self, expr: String, col: String) {
        self.prop_expr_col_map.insert(expr, col);
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

        for (expr, col) in &other.prop_expr_col_map {
            self.prop_expr_col_map.insert(expr.clone(), col.clone());
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
    pub fn extract(&mut self, expr: &Expression) -> Result<ExtractedProps, String> {
        self.extracted_props = ExtractedProps::new();
        self.error = None;

        self.visit_expression(expr)?;

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

    /// 设置错误信息
    fn set_error(&mut self, error: String) {
        self.error = Some(error);
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
        let expr = format!("{}.{}", tag, prop);
        self.extracted_props.insert_prop_expr_col(expr, prop.to_string());
        Ok(())
    }

    fn visit_edge_property(&mut self, edge: &str, prop: &str) -> Self::Result {
        self.extracted_props.insert_edge_prop(edge.to_string(), prop.to_string());
        let expr = format!("{}.{}", edge, prop);
        self.extracted_props.insert_prop_expr_col(expr, prop.to_string());
        Ok(())
    }

    fn visit_input_property(&mut self, prop: &str) -> Self::Result {
        self.extracted_props.insert_input_prop(prop.to_string());
        let expr = format!("$-.{}", prop);
        self.extracted_props.insert_prop_expr_col(expr, prop.to_string());
        Ok(())
    }

    fn visit_variable_property(&mut self, var: &str, prop: &str) -> Self::Result {
        self.extracted_props.insert_input_prop(var.to_string());
        let expr = format!("${}.{}", var, prop);
        self.extracted_props.insert_prop_expr_col(expr, prop.to_string());
        Ok(())
    }

    fn visit_source_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        self.extracted_props.insert_src_prop(tag.to_string(), prop.to_string());
        let expr = format!("$^.{}.{}", tag, prop);
        self.extracted_props.insert_prop_expr_col(expr, prop.to_string());
        Ok(())
    }

    fn visit_destination_property(&mut self, tag: &str, prop: &str) -> Self::Result {
        self.extracted_props.insert_dst_prop(tag.to_string(), prop.to_string());
        let expr = format!("$$.{}.{}", tag, prop);
        self.extracted_props.insert_prop_expr_col(expr, prop.to_string());
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
