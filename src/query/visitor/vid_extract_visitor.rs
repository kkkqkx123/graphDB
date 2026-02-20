//! VidExtractVisitor - 用于提取顶点ID模式的访问器
//! 对应 NebulaGraph VidExtractVisitor.h/.cpp 的功能
//!
//! 主要功能：
//! - 从过滤表达式中提取顶点ID模式
//! - 识别 id(V) IN [vid1, vid2, ...] 或 id(V) == vid 形式的表达式
//! - 支持多个节点的ID提取和交集运算
//! - 用于查询优化，将过滤条件转换为索引查找

use crate::core::types::expression::Expression;
use crate::core::types::expression::visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::{
    BinaryOperator, DataType, UnaryOperator, Value,
};
use crate::core::types::operators::AggregateFunction;
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
    pub fn extract(&mut self, expression: &Expression) -> Result<VidPattern, String> {
        self.vid_pattern = VidPattern::new();
        self.current_alias = None;
        self.error = None;

        self.visit_expression(expression)?;

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
                    if let Expression::List(vids_expression) = right {
                        let mut vids = HashSet::new();
                        for vid_expression in vids_expression {
                            if let Expression::Literal(vid) = vid_expression {
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
        for (_, expression) in pairs {
            self.visit_expression(expression)?;
        }
        Ok(())
    }

    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> Self::Result {
        if let Some(test) = test_expr {
            self.visit_expression(test)?;
        }
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

    fn visit_parameter(&mut self, _name: &str) -> Self::Result {
        Ok(())
    }
}
