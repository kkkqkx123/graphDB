//! 模式 AST 定义
//!
//! 定义图模式匹配的 AST 节点，支持复杂的图遍历模式。

use super::{AstNode, Pattern, Expression, Span, PatternType, node::*};
use std::fmt;

/// 基础模式节点
#[derive(Debug, Clone, PartialEq)]
pub struct BasePattern {
    pub span: Span,
    pub pattern_type: PatternType,
}

impl BasePattern {
    pub fn new(span: Span, pattern_type: PatternType) -> Self {
        Self { span, pattern_type }
    }
}

/// 节点模式
#[derive(Debug, Clone, PartialEq)]
pub struct NodePattern {
    pub base: BasePattern,
    pub identifier: Option<String>,
    pub labels: Vec<String>,
    pub properties: Option<Box<dyn Expression>>,
    pub predicates: Vec<Box<dyn Expression>>,
}

impl NodePattern {
    pub fn new(identifier: Option<String>, labels: Vec<String>, span: Span) -> Self {
        Self {
            base: BasePattern::new(span, PatternType::Node),
            identifier,
            labels,
            properties: None,
            predicates: Vec::new(),
        }
    }
    
    pub fn with_properties(mut self, properties: Box<dyn Expression>) -> Self {
        self.properties = Some(properties);
        self
    }
    
    pub fn with_predicates(mut self, predicates: Vec<Box<dyn Expression>>) -> Self {
        self.predicates = predicates;
        self
    }
}

impl AstNode for NodePattern {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_node_pattern(self)
    }
    
    fn node_type(&self) -> &'static str {
        "NodePattern"
    }
    
    fn to_string(&self) -> String {
        let mut result = String::from("(");
        
        if let Some(ref id) = self.identifier {
            result.push_str(id);
        }
        
        if !self.labels.is_empty() {
            if self.identifier.is_some() {
                result.push(':');
            }
            result.push_str(&self.labels.join(":"));
        }
        
        if let Some(ref props) = self.properties {
            result.push_str(" {");
            result.push_str(&props.to_string());
            result.push('}');
        }
        
        result.push(')');
        result
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(self.clone())
    }
}

impl Pattern for NodePattern {
    fn pattern_type(&self) -> PatternType {
        self.base.pattern_type
    }
    
    fn variables(&self) -> Vec<&str> {
        self.identifier.as_ref().map(|id| id.as_str()).into_iter().collect()
    }
}

/// 边模式
#[derive(Debug, Clone, PartialEq)]
pub struct EdgePattern {
    pub base: BasePattern,
    pub identifier: Option<String>,
    pub edge_type: Option<String>,
    pub direction: EdgeDirection,
    pub properties: Option<Box<dyn Expression>>,
    pub predicates: Vec<Box<dyn Expression>>,
    pub range: Option<EdgeRange>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeDirection {
    Outbound,      // ->
    Inbound,       // <-
    Bidirectional, // -
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeRange {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

impl EdgePattern {
    pub fn new(
        identifier: Option<String>,
        edge_type: Option<String>,
        direction: EdgeDirection,
        span: Span,
    ) -> Self {
        Self {
            base: BasePattern::new(span, PatternType::Edge),
            identifier,
            edge_type,
            direction,
            properties: None,
            predicates: Vec::new(),
            range: None,
        }
    }
    
    pub fn with_properties(mut self, properties: Box<dyn Expression>) -> Self {
        self.properties = Some(properties);
        self
    }
    
    pub fn with_range(mut self, min: Option<u32>, max: Option<u32>) -> Self {
        self.range = Some(EdgeRange { min, max });
        self
    }
}

impl AstNode for EdgePattern {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_edge_pattern(self)
    }
    
    fn node_type(&self) -> &'static str {
        "EdgePattern"
    }
    
    fn to_string(&self) -> String {
        let mut result = String::new();
        
        // 添加方向前缀
        match self.direction {
            EdgeDirection::Inbound => result.push_str("<-"),
            EdgeDirection::Bidirectional => result.push('-'),
            EdgeDirection::Outbound => {} // 默认方向，不显示
        }
        
        result.push('[');
        
        if let Some(ref id) = self.identifier {
            result.push_str(id);
        }
        
        if let Some(ref edge_type) = self.edge_type {
            if self.identifier.is_some() {
                result.push(':');
            }
            result.push_str(edge_type);
        }
        
        if let Some(ref props) = self.properties {
            result.push_str(" {");
            result.push_str(&props.to_string());
            result.push('}');
        }
        
        if let Some(ref range) = self.range {
            result.push_str(" *");
            if let Some(min) = range.min {
                result.push_str(&min.to_string());
            }
            result.push_str("..");
            if let Some(max) = range.max {
                result.push_str(&max.to_string());
            }
        }
        
        result.push(']');
        
        // 添加方向后缀
        match self.direction {
            EdgeDirection::Outbound => result.push_str("->"),
            EdgeDirection::Bidirectional => result.push('-'),
            EdgeDirection::Inbound => {} // 已经显示过了
        }
        
        result
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(self.clone())
    }
}

impl Pattern for EdgePattern {
    fn pattern_type(&self) -> PatternType {
        self.base.pattern_type
    }
    
    fn variables(&self) -> Vec<&str> {
        self.identifier.as_ref().map(|id| id.as_str()).into_iter().collect()
    }
}

/// 路径模式
#[derive(Debug, Clone, PartialEq)]
pub struct PathPattern {
    pub base: BasePattern,
    pub elements: Vec<PathElement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathElement {
    Node(NodePattern),
    Edge(EdgePattern),
    Alternative(Vec<PathPattern>), // (pattern1 | pattern2)
    Optional(Box<PathElement>),      // [pattern]
    Repeated(Box<PathElement>, RepetitionType), // pattern*
}

#[derive(Debug, Clone, PartialEq)]
pub enum RepetitionType {
    ZeroOrMore,  // *
    OneOrMore,   // +
    ZeroOrOne,   // ?
    Exact(u32),  // {n}
    Range(Option<u32>, Option<u32>), // {n,m}
}

impl PathPattern {
    pub fn new(elements: Vec<PathElement>, span: Span) -> Self {
        Self {
            base: BasePattern::new(span, PatternType::Path),
            elements,
        }
    }
    
    pub fn simple(node: NodePattern, edge: EdgePattern, next_node: NodePattern, span: Span) -> Self {
        Self {
            base: BasePattern::new(span, PatternType::Path),
            elements: vec![
                PathElement::Node(node),
                PathElement::Edge(edge),
                PathElement::Node(next_node),
            ],
        }
    }
}

impl AstNode for PathPattern {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_path_pattern(self)
    }
    
    fn node_type(&self) -> &'static str {
        "PathPattern"
    }
    
    fn to_string(&self) -> String {
        self.elements.iter()
            .map(|elem| elem.to_string())
            .collect::<Vec<_>>()
            .join("")
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(self.clone())
    }
}

impl Pattern for PathPattern {
    fn pattern_type(&self) -> PatternType {
        self.base.pattern_type
    }
    
    fn variables(&self) -> Vec<&str> {
        let mut variables = Vec::new();
        
        for element in &self.elements {
            match element {
                PathElement::Node(node) => {
                    variables.extend(node.variables());
                }
                PathElement::Edge(edge) => {
                    variables.extend(edge.variables());
                }
                PathElement::Alternative(patterns) => {
                    // 对于替代模式，我们取第一个模式的变量
                    if let Some(first) = patterns.first() {
                        variables.extend(first.variables());
                    }
                }
                PathElement::Optional(elem) => {
                    variables.extend(elem.variables());
                }
                PathElement::Repeated(elem, _) => {
                    variables.extend(elem.variables());
                }
            }
        }
        
        variables
    }
}

impl fmt::Display for PathElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathElement::Node(node) => write!(f, "{}", node.to_string()),
            PathElement::Edge(edge) => write!(f, "{}", edge.to_string()),
            PathElement::Alternative(patterns) => {
                let pattern_strs: Vec<String> = patterns.iter()
                    .map(|p| p.to_string())
                    .collect();
                write!(f, "({})", pattern_strs.join(" | "))
            }
            PathElement::Optional(elem) => write!(f, "[{}]", elem.to_string()),
            PathElement::Repeated(elem, rep_type) => {
                write!(f, "{}{}", elem.to_string(), rep_type.to_string())
            }
        }
    }
}

impl fmt::Display for RepetitionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepetitionType::ZeroOrMore => write!(f, "*"),
            RepetitionType::OneOrMore => write!(f, "+"),
            RepetitionType::ZeroOrOne => write!(f, "?"),
            RepetitionType::Exact(n) => write!(f, "{{{}}}", n),
            RepetitionType::Range(None, None) => write!(f, "*"),
            RepetitionType::Range(Some(min), None) => write!(f, "{{{min},}}"),
            RepetitionType::Range(None, Some(max)) => write!(f, "{{,{max}}}"),
            RepetitionType::Range(Some(min), Some(max)) => write!(f, "{{{min},{max}}}"),
        }
    }
}

/// 变量模式
#[derive(Debug, Clone, PartialEq)]
pub struct VariablePattern {
    pub base: BasePattern,
    pub name: String,
}

impl VariablePattern {
    pub fn new(name: String, span: Span) -> Self {
        Self {
            base: BasePattern::new(span, PatternType::Variable),
            name,
        }
    }
}

impl AstNode for VariablePattern {
    fn span(&self) -> Span {
        self.base.span
    }
    
    fn accept(&self, visitor: &mut dyn super::Visitor) -> super::VisitorResult {
        visitor.visit_variable_pattern(self)
    }
    
    fn node_type(&self) -> &'static str {
        "VariablePattern"
    }
    
    fn to_string(&self) -> String {
        self.name.clone()
    }
    
    fn clone_box(&self) -> Box<dyn AstNode> {
        Box::new(self.clone())
    }
}

impl Pattern for VariablePattern {
    fn pattern_type(&self) -> PatternType {
        self.base.pattern_type
    }
    
    fn variables(&self) -> Vec<&str> {
        vec![self.name.as_str()]
    }
}

/// 模式工具函数
pub struct PatternUtils;

impl PatternUtils {
    /// 检查模式是否包含循环
    pub fn has_cycle(pattern: &dyn Pattern) -> bool {
        // 简化的循环检测，实际实现会更复杂
        false
    }
    
    /// 获取模式中的所有变量
    pub fn collect_variables(pattern: &dyn Pattern) -> Vec<String> {
        pattern.variables().iter().map(|s| s.to_string()).collect()
    }
    
    /// 检查模式是否有效
    pub fn is_valid(pattern: &dyn Pattern) -> bool {
        // 检查模式的基本有效性
        match pattern.pattern_type() {
            PatternType::Path => {
                // 路径模式应该至少包含一个节点
                true // 简化检查
            }
            _ => true,
        }
    }
    
    /// 合并两个模式
    pub fn merge_patterns(pattern1: &dyn Pattern, pattern2: &dyn Pattern) -> Option<Box<dyn Pattern>> {
        // 模式合并逻辑，需要处理变量冲突等问题
        None // 简化实现
    }
}

/// 模式工厂
pub struct PatternFactory;

impl PatternFactory {
    /// 创建简单的节点模式
    pub fn node(identifier: Option<String>, labels: Vec<String>, span: Span) -> Box<dyn Pattern> {
        Box::new(NodePattern::new(identifier, labels, span))
    }
    
    /// 创建简单的边模式
    pub fn edge(
        identifier: Option<String>,
        edge_type: Option<String>,
        direction: EdgeDirection,
        span: Span,
    ) -> Box<dyn Pattern> {
        Box::new(EdgePattern::new(identifier, edge_type, direction, span))
    }
    
    /// 创建简单的路径模式
    pub fn path(elements: Vec<PathElement>, span: Span) -> Box<dyn Pattern> {
        Box::new(PathPattern::new(elements, span))
    }
    
    /// 创建变量模式
    pub fn variable(name: String, span: Span) -> Box<dyn Pattern> {
        Box::new(VariablePattern::new(name, span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_pattern() {
        let span = Span::default();
        let node = NodePattern::new(
            Some("n".to_string()),
            vec!["Person".to_string(), "Student".to_string()],
            span,
        );
        
        assert_eq!(node.pattern_type(), PatternType::Node);
        assert_eq!(node.variables(), vec!["n"]);
        assert_eq!(node.to_string(), "(n:Person:Student)");
    }
    
    #[test]
    fn test_edge_pattern() {
        let span = Span::default();
        let edge = EdgePattern::new(
            Some("e".to_string()),
            Some("friend".to_string()),
            EdgeDirection::Outbound,
            span,
        );
        
        assert_eq!(edge.pattern_type(), PatternType::Edge);
        assert_eq!(edge.variables(), vec!["e"]);
        assert_eq!(edge.to_string(), "[e:friend]->");
    }
    
    #[test]
    fn test_path_pattern() {
        let span = Span::default();
        let node1 = NodePattern::new(Some("a".to_string()), vec![], span);
        let edge = EdgePattern::new(None, Some("knows".to_string()), EdgeDirection::Outbound, span);
        let node2 = NodePattern::new(Some("b".to_string()), vec![], span);
        
        let path = PathPattern::new(vec![
            PathElement::Node(node1),
            PathElement::Edge(edge),
            PathElement::Node(node2),
        ], span);
        
        assert_eq!(path.pattern_type(), PatternType::Path);
        assert_eq!(path.variables(), vec!["a", "b"]);
    }
}