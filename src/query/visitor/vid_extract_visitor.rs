//! VidExtractVisitor - 用于提取顶点ID模式的访问器
//! 对应 NebulaGraph VidExtractVisitor.h/.cpp 的功能
//!
//! 主要功能：
//! - 从过滤表达式中提取顶点ID模式
//! - 识别 id(V) IN [vid1, vid2, ...] 或 id(V) == vid 形式的表达式
//! - 支持多个节点的ID提取和交集运算
//! - 用于查询优化，将过滤条件转换为索引查找

use crate::core::{
    expression_visitor::{ExpressionVisitor, ExpressionVisitorState},
    BinaryOperator, DataType, Expression, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
use crate::query::parser::ast::expr::*;
use std::collections::{HashMap, HashSet};

/// Vids 类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VidsKind {
    /// 来自其他源（如 PropertiesSeek）
    OtherSource,
    /// IN 操作符
    In,
    /// NOT IN 操作符
    NotIn,
}

/// 顶点ID集合
#[derive(Debug, Clone)]
pub struct Vids {
    /// 类型
    pub kind: VidsKind,
    /// 顶点ID集合
    pub vids: HashSet<Value>,
}

impl Vids {
    pub fn new(kind: VidsKind) -> Self {
        Self {
            kind,
            vids: HashSet::new(),
        }
    }

    pub fn with_vids(kind: VidsKind, vids: HashSet<Value>) -> Self {
        Self { kind, vids }
    }

    /// 计算交集
    pub fn intersect(&self, other: &Vids) -> Vids {
        let vids = self.vids.intersection(&other.vids).cloned().collect();
        Vids {
            kind: VidsKind::In,
            vids,
        }
    }
}

/// 顶点ID模式
#[derive(Debug, Clone)]
pub struct VidPattern {
    /// 节点别名 -> Vids 映射
    pub nodes: HashMap<String, Vids>,
}

impl VidPattern {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// 插入节点ID模式
    pub fn insert_node(&mut self, alias: String, vids: Vids) {
        self.nodes.insert(alias, vids);
    }

    /// 获取节点的Vids
    pub fn get_node(&self, alias: &str) -> Option<&Vids> {
        self.nodes.get(alias)
    }

    /// 计算两个VidPattern的交集
    pub fn intersect(left: VidPattern, right: VidPattern) -> VidPattern {
        let mut result = VidPattern::new();

        for (alias, vids) in left.nodes {
            if let Some(other_vids) = right.nodes.get(&alias) {
                let intersected = vids.intersect(other_vids);
                result.insert_node(alias, intersected);
            }
        }

        result
    }

    /// 计算VidPattern与单个节点Vids的交集
    pub fn intersect_with_node(
        mut pattern: VidPattern,
        alias: String,
        vids: Vids,
    ) -> VidPattern {
        if let Some(existing_vids) = pattern.nodes.get_mut(&alias) {
            *existing_vids = existing_vids.intersect(&vids);
        } else {
            pattern.insert_node(alias, vids);
        }
        pattern
    }
}

impl Default for VidPattern {
    fn default() -> Self {
        Self::new()
    }
}

/// 顶点ID提取访问器
///
/// 用于从过滤表达式中提取顶点ID模式，支持查询优化
#[derive(Debug)]
pub struct VidExtractVisitor {
    /// 提取到的顶点ID模式
    vid_pattern: VidPattern,
    /// 当前节点别名
    current_alias: Option<String>,
    /// 错误状态
    error: Option<String>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl VidExtractVisitor {
    /// 创建新的顶点ID提取访问器
    pub fn new() -> Self {
        Self {
            vid_pattern: VidPattern::new(),
            current_alias: None,
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 提取顶点ID模式
    pub fn extract(&mut self, expr: &Expression) -> Result<VidPattern, String> {
        self.vid_pattern = VidPattern::new();
        self.current_alias = None;
        self.error = None;

        self.visit_expression(expr)?;

        if let Some(err) = &self.error {
            Err(err.clone())
        } else {
            Ok(self.vid_pattern.clone())
        }
    }

    /// 获取提取到的顶点ID模式
    pub fn get_vid_pattern(&self) -> &VidPattern {
        &self.vid_pattern
    }

    /// 设置错误信息
    fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// 检查是否为id函数调用
    fn is_id_function(&self, name: &str) -> bool {
        name.eq_ignore_ascii_case("id")
    }

    /// 尝试提取id(V) IN [vid1, vid2, ...]或id(V) == vid
    fn try_extract_id_comparison(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) {
        match op {
            BinaryOperator::Equal => {
                self.try_extract_id_equal(left, right);
            }
            BinaryOperator::In => {
                self.try_extract_id_in(left, right);
            }
            _ => {}
        }
    }

    /// 尝试提取 id(V) == vid
    fn try_extract_id_equal(&mut self, left: &Expression, right: &Expression) {
        if let Expression::Function { name, args } = left {
            if self.is_id_function(name) && args.len() == 1 {
                if let Expression::Variable(alias) = &args[0] {
                    if let Expression::Literal(vid) = right {
                        let mut vids = HashSet::new();
                        vids.insert(vid.clone());
                        self.vid_pattern
                            .insert_node(alias.clone(), Vids::with_vids(VidsKind::In, vids));
                    }
                }
            }
        }
    }

    /// 尝试提取 id(V) IN [vid1, vid2, ...]
    fn try_extract_id_in(&mut self, left: &Expression, right: &Expression) {
        if let Expression::Function { name, args } = left {
            if self.is_id_function(name) && args.len() == 1 {
                if let Expression::Variable(alias) = &args[0] {
                    if let Expression::List(vids_expr) = right {
                        let mut vids = HashSet::new();
                        for vid_expr in vids_expr {
                            if let Expression::Literal(vid) = vid_expr {
                                vids.insert(vid.clone());
                            }
                        }
                        if !vids.is_empty() {
                            self.vid_pattern
                                .insert_node(alias.clone(), Vids::with_vids(VidsKind::In, vids));
                        }
                    }
                }
            }
        }
    }
}

impl Default for VidExtractVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionVisitor for VidExtractVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
        Ok(())
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.current_alias = Some(name.to_string());
        Ok(())
    }

    fn visit_property(&mut self, object: &Expression, _property: &str) -> Self::Result {
        self.visit_expression(object)
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        match op {
            BinaryOperator::And => {
                let left_pattern = {
                    let mut left_visitor = VidExtractVisitor::new();
                    left_visitor.extract(left)?;
                    left_visitor.vid_pattern
                };

                let right_pattern = {
                    let mut right_visitor = VidExtractVisitor::new();
                    right_visitor.extract(right)?;
                    right_visitor.vid_pattern
                };

                self.vid_pattern = VidPattern::intersect(left_pattern, right_pattern);
            }
            BinaryOperator::Equal | BinaryOperator::In => {
                self.try_extract_id_comparison(left, op, right);
            }
            _ => {
                self.visit_expression(left)?;
                self.visit_expression(right)?;
            }
        }
        Ok(())
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand)
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) -> Self::Result {
        if self.is_id_function(name) && args.len() == 1 {
            if let Expression::Variable(alias) = &args[0] {
                self.current_alias = Some(alias.clone());
            }
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

    fn visit_tag_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        Ok(())
    }

    fn visit_edge_property(&mut self, _edge: &str, _prop: &str) -> Self::Result {
        Ok(())
    }

    fn visit_input_property(&mut self, _prop: &str) -> Self::Result {
        Ok(())
    }

    fn visit_variable_property(&mut self, var: &str, _prop: &str) -> Self::Result {
        self.visit_variable(var)
    }

    fn visit_source_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
        Ok(())
    }

    fn visit_destination_property(&mut self, _tag: &str, _prop: &str) -> Self::Result {
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

    fn visit_tag_property_expr(&mut self, _expr: &TagPropertyExpr) -> Self::Result {
        Ok(())
    }

    fn visit_edge_property_expr(&mut self, _expr: &EdgePropertyExpr) -> Self::Result {
        Ok(())
    }

    fn visit_input_property_expr(&mut self, _expr: &InputPropertyExpr) -> Self::Result {
        Ok(())
    }

    fn visit_variable_property_expr(&mut self, expr: &VariablePropertyExpr) -> Self::Result {
        self.visit_variable_property(&expr.var, &expr.prop)
    }

    fn visit_source_property_expr(&mut self, _expr: &SourcePropertyExpr) -> Self::Result {
        Ok(())
    }

    fn visit_destination_property_expr(&mut self, _expr: &DestinationPropertyExpr) -> Self::Result {
        Ok(())
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
