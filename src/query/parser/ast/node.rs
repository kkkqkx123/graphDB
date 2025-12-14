//! AST 节点基础定义
//!
//! 提供 AST 节点的核心 trait 和基础实现

use crate::core::Value;
use super::{AstNode, Expression, Span, ExpressionType};
use std::fmt;

/// 基础 AST 节点实现
#[derive(Debug, Clone, PartialEq)]
pub struct BaseNode {
    pub span: Span,
    pub node_type: &'static str,
}

impl BaseNode {
    pub fn new(span: Span, node_type: &'static str) -> Self {
        Self { span, node_type }
    }
}

/// 常量表达式节点
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantExpr {
    pub base: BaseNode,
    pub value: Value,
}

impl ConstantExpr {
    pub fn new(value: Value, span: Span) -> Self {
        Self {
            base: BaseNode::new(span, "ConstantExpr"),
            value,
        }
    }
}

impl AstNode for ConstantExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_constant_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        format!("{:?}", self.value)
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(self.clone())
    }
}

impl Expression for ConstantExpr {
    fn expr_type(&self) -> ExpressionType {
        ExpressionType::Constant
    }
    
    fn is_constant(&self) -> bool {
        true
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        vec![]
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 变量表达式节点
#[derive(Debug, Clone, PartialEq)]
pub struct VariableExpr {
    pub base: BaseNode,
    pub name: String,
}

impl VariableExpr {
    pub fn new(name: String, span: Span) -> Self {
        Self {
            base: BaseNode::new(span, "VariableExpr"),
            name,
        }
    }
}

impl AstNode for VariableExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_variable_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        self.name.clone()
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(self.clone())
    }
}

impl Expression for VariableExpr {
    fn expr_type(&self) -> ExpressionType {
        ExpressionType::Variable
    }
    
    fn is_constant(&self) -> bool {
        false
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        vec![]
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 二元表达式节点
#[derive(Debug)]
pub struct BinaryExpr {
    pub base: BaseNode,
    pub left: Box<dyn Expression>,
    pub op: BinaryOp,
    pub right: Box<dyn Expression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // 算术操作符
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp, // 指数运算
    
    // 逻辑操作符
    And,
    Or,
    Xor,
    
    // 关系操作符
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    
    // 字符串操作符
    Regex,      // =~
    In,         // IN
    NotIn,      // NOT IN
    Contains,   // CONTAINS
    StartsWith, // STARTS WITH
    EndsWith,   // ENDS WITH
}

impl BinaryExpr {
    pub fn new(left: Box<dyn Expression>, op: BinaryOp, right: Box<dyn Expression>, span: Span) -> Self {
        Self {
            base: BaseNode::new(span, "BinaryExpr"),
            left,
            op,
            right,
        }
    }
}

impl AstNode for BinaryExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_binary_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        format!("({} {} {})", self.left.to_string(), self.op.to_string(), self.right.to_string())
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        super::AstNode::clone_box(self)
    }
}

impl Expression for BinaryExpr {
    fn expr_type(&self) -> ExpressionType {
        match self.op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod | BinaryOp::Exp => {
                ExpressionType::Binary
            }
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => ExpressionType::Binary,
            _ => ExpressionType::Binary,
        }
    }
    
    fn is_constant(&self) -> bool {
        self.left.is_constant() && self.right.is_constant()
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        vec![super::Expression::clone_box(&*self.left), super::Expression::clone_box(&*self.right)]
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(BinaryExpr {
            base: self.base.clone(),
            left: super::Expression::clone_box(&*self.left),
            op: self.op,
            right: super::Expression::clone_box(&*self.right),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::Exp => write!(f, "**"),
            BinaryOp::And => write!(f, "AND"),
            BinaryOp::Or => write!(f, "OR"),
            BinaryOp::Xor => write!(f, "XOR"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Ne => write!(f, "!="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Le => write!(f, "<="),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Ge => write!(f, ">="),
            BinaryOp::Regex => write!(f, "=~"),
            BinaryOp::In => write!(f, "IN"),
            BinaryOp::NotIn => write!(f, "NOT IN"),
            BinaryOp::Contains => write!(f, "CONTAINS"),
            BinaryOp::StartsWith => write!(f, "STARTS WITH"),
            BinaryOp::EndsWith => write!(f, "ENDS WITH"),
        }
    }
}

/// 一元表达式节点
#[derive(Debug)]
pub struct UnaryExpr {
    pub base: BaseNode,
    pub op: UnaryOp,
    pub operand: Box<dyn Expression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,   // NOT
    Plus,  // +
    Minus, // -
    IsNull,    // IS NULL
    IsNotNull, // IS NOT NULL
    IsEmpty,   // IS EMPTY
    IsNotEmpty, // IS NOT EMPTY
}

impl UnaryExpr {
    pub fn new(op: UnaryOp, operand: Box<dyn Expression>, span: Span) -> Self {
        Self {
            base: BaseNode::new(span, "UnaryExpr"),
            op,
            operand,
        }
    }
}

impl AstNode for UnaryExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_unary_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        format!("{} {}", self.op.to_string(), self.operand.to_string())
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        super::AstNode::clone_box(self)
    }
}

impl Expression for UnaryExpr {
    fn expr_type(&self) -> ExpressionType {
        ExpressionType::Unary
    }
    
    fn is_constant(&self) -> bool {
        self.operand.is_constant()
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        vec![super::Expression::clone_box(&*self.operand)]
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(UnaryExpr {
            base: self.base.clone(),
            op: self.op,
            operand: super::Expression::clone_box(&*self.operand),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "NOT"),
            UnaryOp::Plus => write!(f, "+"),
            UnaryOp::Minus => write!(f, "-"),
            UnaryOp::IsNull => write!(f, "IS NULL"),
            UnaryOp::IsNotNull => write!(f, "IS NOT NULL"),
            UnaryOp::IsEmpty => write!(f, "IS EMPTY"),
            UnaryOp::IsNotEmpty => write!(f, "IS NOT EMPTY"),
        }
    }
}

/// 函数调用表达式节点
#[derive(Debug)]
pub struct FunctionCallExpr {
    pub base: BaseNode,
    pub name: String,
    pub args: Vec<Box<dyn Expression>>,
    pub distinct: bool,
}

impl FunctionCallExpr {
    pub fn new(name: String, args: Vec<Box<dyn Expression>>, distinct: bool, span: Span) -> Self {
        Self {
            base: BaseNode::new(span, "FunctionCallExpr"),
            name,
            args,
            distinct,
        }
    }
}

impl AstNode for FunctionCallExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_function_call_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        let args_str = self.args.iter()
            .map(|arg| arg.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        
        if self.distinct {
            format!("{}(DISTINCT {})", self.name, args_str)
        } else {
            format!("{}({})", self.name, args_str)
        }
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        super::AstNode::clone_box(self)
    }
}

impl Expression for FunctionCallExpr {
    fn expr_type(&self) -> ExpressionType {
        ExpressionType::FunctionCall
    }
    
    fn is_constant(&self) -> bool {
        false // 函数调用通常不是常量
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        self.args.iter().map(|arg| super::Expression::clone_box(&**arg)).collect()
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(FunctionCallExpr {
            base: self.base.clone(),
            name: self.name.clone(),
            args: self.args.iter().map(|arg| super::Expression::clone_box(&**arg)).collect(),
            distinct: self.distinct,
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 属性访问表达式节点
#[derive(Debug)]
pub struct PropertyAccessExpr {
    pub base: BaseNode,
    pub object: Box<dyn Expression>,
    pub property: String,
}

impl PropertyAccessExpr {
    pub fn new(object: Box<dyn Expression>, property: String, span: Span) -> Self {
        Self {
            base: BaseNode::new(span, "PropertyAccessExpr"),
            object,
            property,
        }
    }
}

impl AstNode for PropertyAccessExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_property_access_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        format!("{}.{}", self.object.to_string(), self.property)
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        super::AstNode::clone_box(self)
    }
}

impl Expression for PropertyAccessExpr {
    fn expr_type(&self) -> ExpressionType {
        ExpressionType::PropertyAccess
    }
    
    fn is_constant(&self) -> bool {
        false
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        vec![super::Expression::clone_box(&*self.object)]
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(PropertyAccessExpr {
            base: self.base.clone(),
            object: super::Expression::clone_box(&*self.object),
            property: self.property.clone(),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 列表表达式节点
#[derive(Debug)]
pub struct ListExpr {
    pub base: BaseNode,
    pub elements: Vec<Box<dyn Expression>>,
}

impl ListExpr {
    pub fn new(elements: Vec<Box<dyn Expression>>, span: Span) -> Self {
        Self {
            base: BaseNode::new(span, "ListExpr"),
            elements,
        }
    }
}

impl AstNode for ListExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_list_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        let elements_str = self.elements.iter()
            .map(|elem| elem.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{}]", elements_str)
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        super::AstNode::clone_box(self)
    }
}

impl Expression for ListExpr {
    fn expr_type(&self) -> ExpressionType {
        ExpressionType::List
    }
    
    fn is_constant(&self) -> bool {
        self.elements.iter().all(|elem| elem.is_constant())
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        self.elements.iter().map(|elem| super::Expression::clone_box(&**elem)).collect()
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(ListExpr {
            base: self.base.clone(),
            elements: self.elements.iter().map(|elem| super::Expression::clone_box(&**elem)).collect(),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 映射表达式节点
#[derive(Debug)]
pub struct MapExpr {
    pub base: BaseNode,
    pub pairs: Vec<(String, Box<dyn Expression>)>,
}

impl MapExpr {
    pub fn new(pairs: Vec<(String, Box<dyn Expression>)>, span: Span) -> Self {
        Self {
            base: BaseNode::new(span, "MapExpr"),
            pairs,
        }
    }
}

impl AstNode for MapExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_map_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        let pairs_str = self.pairs.iter()
            .map(|(key, value)| format!("{}: {}", key, value.to_string()))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{{{}}}", pairs_str)
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        super::AstNode::clone_box(self)
    }
}

impl Expression for MapExpr {
    fn expr_type(&self) -> ExpressionType {
        ExpressionType::Map
    }
    
    fn is_constant(&self) -> bool {
        self.pairs.iter().all(|(_, value)| value.is_constant())
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        self.pairs.iter().map(|(_, value)| super::Expression::clone_box(&**value)).collect()
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(MapExpr {
            base: self.base.clone(),
            pairs: self.pairs.iter().map(|(k, v)| (k.clone(), super::Expression::clone_box(&**v))).collect(),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// CASE 表达式节点
#[derive(Debug)]
pub struct CaseExpr {
    pub base: BaseNode,
    pub match_expr: Option<Box<dyn Expression>>,
    pub when_then_pairs: Vec<(Box<dyn Expression>, Box<dyn Expression>)>,
    pub default: Option<Box<dyn Expression>>,
}

impl CaseExpr {
    pub fn new(
        match_expr: Option<Box<dyn Expression>>,
        when_then_pairs: Vec<(Box<dyn Expression>, Box<dyn Expression>)>,
        default: Option<Box<dyn Expression>>,
        span: Span,
    ) -> Self {
        Self {
            base: BaseNode::new(span, "CaseExpr"),
            match_expr,
            when_then_pairs,
            default,
        }
    }
}

impl AstNode for CaseExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_case_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        let mut result = String::from("CASE");
        
        if let Some(ref expr) = self.match_expr {
            result.push_str(&format!(" {}", expr.to_string()));
        }
        
        for (when, then) in &self.when_then_pairs {
            result.push_str(&format!(" WHEN {} THEN {}", when.to_string(), then.to_string()));
        }
        
        if let Some(ref default) = self.default {
            result.push_str(&format!(" ELSE {}", default.to_string()));
        }
        
        result.push_str(" END");
        result
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        super::AstNode::clone_box(self)
    }
}

impl Expression for CaseExpr {
    fn expr_type(&self) -> ExpressionType {
        ExpressionType::Case
    }
    
    fn is_constant(&self) -> bool {
        let all_when_constant = self.when_then_pairs.iter()
            .all(|(when, then)| when.is_constant() && then.is_constant());
        
        let default_constant = self.default.as_ref()
            .map(|d| d.is_constant())
            .unwrap_or(true);
        
        let match_constant = self.match_expr.as_ref()
            .map(|m| m.is_constant())
            .unwrap_or(true);
        
        all_when_constant && default_constant && match_constant
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        let mut children = Vec::new();
        
        if let Some(ref expr) = self.match_expr {
            children.push(super::Expression::clone_box(&**expr));
        }
        
        for (when, then) in &self.when_then_pairs {
            children.push(super::Expression::clone_box(&**when));
            children.push(super::Expression::clone_box(&**then));
        }
        
        if let Some(ref default) = self.default {
            children.push(super::Expression::clone_box(&**default));
        }
        
        children
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(CaseExpr {
            base: self.base.clone(),
            match_expr: self.match_expr.as_ref().map(|expr| super::Expression::clone_box(&**expr)),
            when_then_pairs: self.when_then_pairs.iter()
                .map(|(when, then)| (super::Expression::clone_box(&**when), super::Expression::clone_box(&**then)))
                .collect(),
            default: self.default.as_ref().map(|expr| super::Expression::clone_box(&**expr)),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 下标表达式节点
#[derive(Debug)]
pub struct SubscriptExpr {
    pub base: BaseNode,
    pub collection: Box<dyn Expression>,
    pub index: Box<dyn Expression>,
}

impl SubscriptExpr {
    pub fn new(collection: Box<dyn Expression>, index: Box<dyn Expression>, span: Span) -> Self {
        Self {
            base: BaseNode::new(span, "SubscriptExpr"),
            collection,
            index,
        }
    }
}

impl AstNode for SubscriptExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_subscript_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        format!("{}[{}]", self.collection.to_string(), self.index.to_string())
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        super::AstNode::clone_box(self)
    }
}

impl Expression for SubscriptExpr {
    fn expr_type(&self) -> ExpressionType {
        ExpressionType::Subscript
    }
    
    fn is_constant(&self) -> bool {
        self.collection.is_constant() && self.index.is_constant()
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        vec![super::Expression::clone_box(&*self.collection), super::Expression::clone_box(&*self.index)]
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(SubscriptExpr {
            base: self.base.clone(),
            collection: super::Expression::clone_box(&*self.collection),
            index: super::Expression::clone_box(&*self.index),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 谓词表达式节点
#[derive(Debug)]
pub struct PredicateExpr {
    pub base: BaseNode,
    pub predicate: PredicateType,
    pub list: Box<dyn Expression>,
    pub condition: Box<dyn Expression>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateType {
    All,    // ALL
    Any,    // ANY
    Single, // SINGLE
    None,   // NONE
    Exists, // EXISTS
}

impl PredicateExpr {
    pub fn new(
        predicate: PredicateType,
        list: Box<dyn Expression>,
        condition: Box<dyn Expression>,
        span: Span,
    ) -> Self {
        Self {
            base: BaseNode::new(span, "PredicateExpr"),
            predicate,
            list,
            condition,
        }
    }
}

impl AstNode for PredicateExpr {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_predicate_expr(self)
    }
    
    fn node_type(&self) -> &'static str {
        self.base.node_type
    }
    
    fn to_string(&self) -> String {
        format!("{}({} IN {} WHERE {})", 
            self.predicate.to_string(),
            "x", // 变量名需要额外处理
            self.list.to_string(),
            self.condition.to_string()
        )
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        super::AstNode::clone_box(self)
    }
}

impl Expression for PredicateExpr {
    fn expr_type(&self) -> ExpressionType {
        ExpressionType::Predicate
    }
    
    fn is_constant(&self) -> bool {
        false // 谓词表达式通常不是常量
    }
    
    fn children(&self) -> Vec<Box<dyn Expression>> {
        vec![super::Expression::clone_box(&*self.list), super::Expression::clone_box(&*self.condition)]
    }
    
    fn clone_box(&self) -> Box<dyn Expression> {
        Box::new(PredicateExpr {
            base: self.base.clone(),
            predicate: self.predicate,
            list: super::Expression::clone_box(&*self.list),
            condition: super::Expression::clone_box(&*self.condition),
        })
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl fmt::Display for PredicateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PredicateType::All => write!(f, "ALL"),
            PredicateType::Any => write!(f, "ANY"),
            PredicateType::Single => write!(f, "SINGLE"),
            PredicateType::None => write!(f, "NONE"),
            PredicateType::Exists => write!(f, "EXISTS"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constant_expr() {
        let expr = ConstantExpr::new(Value::Int(42), Span::default());
        assert_eq!(expr.expr_type(), ExpressionType::Constant);
        assert!(expr.is_constant());
        assert_eq!(expr.to_string(), "Int(42)");
    }
    
    #[test]
    fn test_variable_expr() {
        let expr = VariableExpr::new("x".to_string(), Span::default());
        assert_eq!(expr.expr_type(), ExpressionType::Variable);
        assert!(!expr.is_constant());
        assert_eq!(expr.to_string(), "x");
    }
    
    #[test]
    fn test_binary_expr() {
        let left = Box::new(ConstantExpr::new(Value::Int(5), Span::default()));
        let right = Box::new(ConstantExpr::new(Value::Int(3), Span::default()));
        let expr = BinaryExpr::new(left, BinaryOp::Add, right, Span::default());
        
        assert_eq!(expr.expr_type(), ExpressionType::Binary);
        assert!(expr.is_constant());
        assert_eq!(expr.to_string(), "(Int(5) + Int(3))");
    }
}