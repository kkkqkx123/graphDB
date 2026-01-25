//! DeducePropsVisitor - 用于推导表达式属性的访问器
//! 对应 NebulaGraph DeducePropsVisitor.h/.cpp 的功能

use crate::core::types::expression::Expression;
use crate::core::expression_visitor::{ExpressionVisitor, ExpressionVisitorState};
use crate::core::Value;
use crate::core::{AggregateFunction, BinaryOperator, DataType, UnaryOperator};
use std::collections::{HashMap, HashSet};

/// 属性定义
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropDef {
    pub name: String,
    pub type_: DataType,
}

/// 节点信息 - 记录查询中涉及的节点及其属性
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub alias: String,
    pub props: HashSet<String>,
    pub vid: Option<Box<Expression>>,
    pub tags: Vec<String>, // 节点上的标签列表
}

/// 边信息 - 记录查询中涉及的边及其属性
#[derive(Debug, Clone)]
pub struct EdgeInfo {
    pub alias: String,
    pub props: HashSet<String>,
    pub type_name: String,
    pub src: Option<Box<Expression>>,
    pub dst: Option<Box<Expression>>,
    pub rank: Option<Box<Expression>>,
    pub steps: String, // "1" or "*"
}

/// 表达式属性集合 - 统一管理所有类型的属性
#[derive(Debug, Clone)]
pub struct ExpressionProps {
    /// 输入列属性 - 来自上一步的输出（$-）
    pub input_props: HashSet<String>,
    /// 变量属性 - 从变量中获取（$varName.prop）
    pub var_props: HashMap<String, HashSet<String>>,
    /// 源标签属性 - 起点节点属性（$^.tagName.prop）
    pub src_tag_props: HashMap<String, HashSet<String>>,
    /// 目标标签属性 - 终点节点属性（$$.tagName.prop）
    pub dst_tag_props: HashMap<String, HashSet<String>>,
    /// 普通标签属性 - 节点标签属性（tagName.prop）
    pub tag_props: HashMap<String, HashSet<String>>,
    /// 边属性 - 边的属性（edgeName.prop）
    pub edge_props: HashMap<String, HashSet<String>>,
    /// 标签名称映射 - tagName -> tagId
    pub tag_name_ids: HashMap<String, String>,
}

impl ExpressionProps {
    pub fn new() -> Self {
        Self {
            input_props: HashSet::new(),
            var_props: HashMap::new(),
            src_tag_props: HashMap::new(),
            dst_tag_props: HashMap::new(),
            tag_props: HashMap::new(),
            edge_props: HashMap::new(),
            tag_name_ids: HashMap::new(),
        }
    }

    /// 插入输入属性
    pub fn insert_input_prop(&mut self, prop: &str) {
        self.input_props.insert(prop.to_string());
    }

    /// 插入变量属性
    pub fn insert_var_prop(&mut self, var: &str, prop: &str) {
        self.var_props
            .entry(var.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 插入源标签属性
    pub fn insert_src_tag_prop(&mut self, tag: &str, prop: &str) {
        self.src_tag_props
            .entry(tag.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 插入目标标签属性
    pub fn insert_dst_tag_prop(&mut self, tag: &str, prop: &str) {
        self.dst_tag_props
            .entry(tag.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 插入普通标签属性
    pub fn insert_tag_prop(&mut self, tag: &str, prop: &str) {
        self.tag_props
            .entry(tag.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 插入边属性
    pub fn insert_edge_prop(&mut self, edge: &str, prop: &str) {
        self.edge_props
            .entry(edge.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    /// 记录标签名称到ID的映射
    pub fn insert_tag_name_id(&mut self, name: &str, id: &str) {
        self.tag_name_ids.insert(name.to_string(), id.to_string());
    }

    /// 检查是否有输入或变量属性
    pub fn has_input_var_property(&self) -> bool {
        !self.input_props.is_empty() || !self.var_props.is_empty()
    }

    /// 检查是否有源标签属性
    pub fn has_src_tag_property(&self) -> bool {
        !self.src_tag_props.is_empty()
    }

    /// 检查是否有源或目标标签属性
    pub fn has_src_dst_tag_property(&self) -> bool {
        !self.src_tag_props.is_empty() || !self.dst_tag_props.is_empty()
    }

    /// 检查所有属性集合是否为空
    pub fn is_all_props_empty(&self) -> bool {
        self.input_props.is_empty()
            && self.var_props.is_empty()
            && self.src_tag_props.is_empty()
            && self.dst_tag_props.is_empty()
            && self.tag_props.is_empty()
            && self.edge_props.is_empty()
    }

    /// 检查给定属性集合是否为输入属性的子集
    pub fn is_subset_of_input(&self, props: &HashSet<String>) -> bool {
        props.iter().all(|p| self.input_props.contains(p))
    }

    /// 检查给定属性映射是否为变量属性的子集
    pub fn is_subset_of_var(&self, props: &HashMap<String, HashSet<String>>) -> bool {
        props.iter().all(|(var, var_props)| {
            self.var_props.get(var).map_or(false, |my_props| {
                var_props.iter().all(|p| my_props.contains(p))
            })
        })
    }

    /// 合并另一个ExpressionProps的属性
    pub fn union_props(&mut self, other: &ExpressionProps) {
        self.input_props.extend(other.input_props.iter().cloned());

        for (var, props) in &other.var_props {
            self.var_props
                .entry(var.clone())
                .or_insert_with(HashSet::new)
                .extend(props.iter().cloned());
        }

        for (tag, props) in &other.src_tag_props {
            self.src_tag_props
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .extend(props.iter().cloned());
        }

        for (tag, props) in &other.dst_tag_props {
            self.dst_tag_props
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .extend(props.iter().cloned());
        }

        for (tag, props) in &other.tag_props {
            self.tag_props
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .extend(props.iter().cloned());
        }

        for (edge, props) in &other.edge_props {
            self.edge_props
                .entry(edge.clone())
                .or_insert_with(HashSet::new)
                .extend(props.iter().cloned());
        }

        for (name, id) in &other.tag_name_ids {
            self.tag_name_ids.insert(name.clone(), id.clone());
        }
    }
}

/// 属性推导访问器
/// 用于递归遍历表达式树，收集所有涉及的属性信息
#[derive(Debug)]
pub struct DeducePropsVisitor {
    /// 推导出的表达式属性集合
    props: ExpressionProps,
    /// 收集的节点信息
    node_info: Vec<NodeInfo>,
    /// 收集的边信息
    edge_info: Vec<EdgeInfo>,
    /// 用户定义的变量名称列表
    user_defined_vars: HashSet<String>,
    /// 错误状态
    error: Option<String>,
    /// 访问者状态
    state: ExpressionVisitorState,
}

impl DeducePropsVisitor {
    /// 创建新的属性推导访问器
    pub fn new() -> Self {
        Self {
            props: ExpressionProps::new(),
            node_info: Vec::new(),
            edge_info: Vec::new(),
            user_defined_vars: HashSet::new(),
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 创建带有用户定义变量列表的访问器
    pub fn with_user_vars(user_defined_vars: HashSet<String>) -> Self {
        Self {
            props: ExpressionProps::new(),
            node_info: Vec::new(),
            edge_info: Vec::new(),
            user_defined_vars,
            error: None,
            state: ExpressionVisitorState::new(),
        }
    }

    /// 执行属性推导
    pub fn deduce(&mut self, expression: &Expression) -> Result<(), String> {
        self.visit_expression(expression)
    }

    /// 获取推导出的表达式属性
    pub fn get_props(&self) -> &ExpressionProps {
        &self.props
    }

    /// 获取可变的表达式属性引用
    pub fn get_props_mut(&mut self) -> &mut ExpressionProps {
        &mut self.props
    }

    /// 获取收集的节点信息
    pub fn get_node_info(&self) -> &[NodeInfo] {
        &self.node_info
    }

    /// 获取收集的边信息
    pub fn get_edge_info(&self) -> &[EdgeInfo] {
        &self.edge_info
    }

    /// 获取用户定义变量列表
    pub fn get_user_defined_vars(&self) -> &HashSet<String> {
        &self.user_defined_vars
    }

    /// 获取错误信息
    pub fn get_error(&self) -> &Option<String> {
        &self.error
    }

    /// 是否推导成功
    pub fn is_ok(&self) -> bool {
        self.error.is_none()
    }

    /// 设置错误信息
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
    }

    /// 向节点信息列表添加节点
    pub fn add_node_info(&mut self, node: NodeInfo) {
        self.node_info.push(node);
    }

    /// 向边信息列表添加边
    pub fn add_edge_info(&mut self, edge: EdgeInfo) {
        self.edge_info.push(edge);
    }
}

impl Default for DeducePropsVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionVisitor for DeducePropsVisitor {
    type Result = Result<(), String>;

    fn visit_literal(&mut self, _value: &Value) -> Self::Result {
        Ok(())
    }

    fn visit_variable(&mut self, name: &str) -> Self::Result {
        self.props.insert_input_prop(name);
        Ok(())
    }

    fn visit_property(&mut self, object: &Expression, property: &str) -> Self::Result {
        self.visit_expression(object)?;
        self.props.insert_input_prop(property);
        Ok(())
    }

    fn visit_binary(
        &mut self,
        left: &Expression,
        _op: &BinaryOperator,
        right: &Expression,
    ) -> Self::Result {
        self.visit_expression(left)?;
        self.visit_expression(right)?;
        Ok(())
    }

    fn visit_unary(&mut self, _op: &UnaryOperator, operand: &Expression) -> Self::Result {
        self.visit_expression(operand)
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) -> Self::Result {
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
        default: &Option<Box<Expression>>,
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
        self.visit_expression(index)?;
        Ok(())
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: &Option<Box<Expression>>,
        end: &Option<Box<Expression>>,
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

    fn state(&self) -> &ExpressionVisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut ExpressionVisitorState {
        &mut self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_props_input() {
        let mut props = ExpressionProps::new();
        props.insert_input_prop("name");
        props.insert_input_prop("age");

        assert!(props.input_props.contains("name"));
        assert!(props.input_props.contains("age"));
        assert_eq!(props.input_props.len(), 2);
    }

    #[test]
    fn test_expression_props_var() {
        let mut props = ExpressionProps::new();
        props.insert_var_prop("x", "value");
        props.insert_var_prop("x", "name");
        props.insert_var_prop("y", "age");

        assert_eq!(props.var_props.get("x").map(|p| p.len()), Some(2));
        assert_eq!(props.var_props.get("y").map(|p| p.len()), Some(1));
    }

    #[test]
    fn test_expression_props_tag() {
        let mut props = ExpressionProps::new();
        props.insert_tag_prop("Person", "name");
        props.insert_tag_prop("Person", "age");
        props.insert_src_tag_prop("Person", "id");
        props.insert_dst_tag_prop("Animal", "type");

        assert_eq!(props.tag_props.get("Person").map(|p| p.len()), Some(2));
        assert_eq!(props.src_tag_props.get("Person").map(|p| p.len()), Some(1));
        assert_eq!(props.dst_tag_props.get("Animal").map(|p| p.len()), Some(1));
    }

    #[test]
    fn test_expression_props_edge() {
        let mut props = ExpressionProps::new();
        props.insert_edge_prop("follow", "weight");
        props.insert_edge_prop("follow", "time");
        props.insert_edge_prop("like", "score");

        assert_eq!(props.edge_props.get("follow").map(|p| p.len()), Some(2));
        assert_eq!(props.edge_props.get("like").map(|p| p.len()), Some(1));
    }

    #[test]
    fn test_deduce_visitor_constant() {
        let mut visitor = DeducePropsVisitor::new();
        let expression = Expression::Literal(crate::core::Value::Int(42));

        assert!(visitor.deduce(&expression).is_ok());
        assert!(visitor.get_props().is_all_props_empty());
    }

    #[test]
    fn test_deduce_visitor_property_variable() {
        let mut visitor = DeducePropsVisitor::new();
        let expression = Expression::Variable("name".to_string());

        assert!(visitor.deduce(&expression).is_ok());
        assert!(visitor.get_props().input_props.contains("name"));
    }

    #[test]
    fn test_deduce_visitor_property() {
        let mut visitor = DeducePropsVisitor::new();
        let expression = Expression::Property {
            object: Box::new(Expression::Variable("person".to_string())),
            property: "name".to_string(),
        };

        assert!(visitor.deduce(&expression).is_ok());
    }

    #[test]
    fn test_deduce_visitor_binary_op() {
        let mut visitor = DeducePropsVisitor::new();
        let expression = Expression::Binary {
            left: Box::new(Expression::Variable("age".to_string())),
            op: crate::core::BinaryOperator::Add,
            right: Box::new(Expression::Variable("bonus".to_string())),
        };

        assert!(visitor.deduce(&expression).is_ok());
        let props = visitor.get_props();
        assert!(props.input_props.contains("age"));
        assert!(props.input_props.contains("bonus"));
    }

    #[test]
    fn test_deduce_visitor_union() {
        let mut props1 = ExpressionProps::new();
        props1.insert_input_prop("name");
        props1.insert_tag_prop("Person", "age");

        let mut props2 = ExpressionProps::new();
        props2.insert_input_prop("email");
        props2.insert_tag_prop("Person", "city");
        props2.insert_edge_prop("follow", "weight");

        props1.union_props(&props2);

        assert_eq!(props1.input_props.len(), 2);
        assert!(props1
            .tag_props
            .get("Person")
            .map_or(false, |p| p.len() == 2));
        assert!(props1
            .edge_props
            .get("follow")
            .map_or(false, |p| p.len() == 1));
    }
}
