//! 模式 AST 定义 (v2)
//!
//! 图模式匹配相关的 AST 定义，支持节点、边和路径模式。

use super::types::*;
use crate::core::types::expression::ContextualExpression;
use crate::core::types::expression::utils::collect_variables_from_contextual;

/// 模式枚举 - 图模式匹配
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Node(NodePattern),
    Edge(EdgePattern),
    Path(PathPattern),
    Variable(VariablePattern),
}

impl Pattern {
    /// 获取模式的位置信息
    pub fn span(&self) -> Span {
        match self {
            Pattern::Node(p) => p.span,
            Pattern::Edge(p) => p.span,
            Pattern::Path(p) => p.span,
            Pattern::Variable(p) => p.span,
        }
    }
}

/// 节点模式
#[derive(Debug, Clone, PartialEq)]
pub struct NodePattern {
    pub span: Span,
    pub variable: Option<String>,
    pub labels: Vec<String>,
    pub properties: Option<ContextualExpression>,
    pub predicates: Vec<ContextualExpression>,
}

impl NodePattern {
    pub fn new(
        variable: Option<String>,
        labels: Vec<String>,
        properties: Option<ContextualExpression>,
        predicates: Vec<ContextualExpression>,
        span: Span,
    ) -> Self {
        Self {
            span,
            variable,
            labels,
            properties,
            predicates,
        }
    }
}

/// 边模式
#[derive(Debug, Clone, PartialEq)]
pub struct EdgePattern {
    pub span: Span,
    pub variable: Option<String>,
    pub edge_types: Vec<String>,
    pub properties: Option<ContextualExpression>,
    pub predicates: Vec<ContextualExpression>,
    pub direction: EdgeDirection,
    pub range: Option<EdgeRange>,
}

impl EdgePattern {
    pub fn new(
        variable: Option<String>,
        edge_types: Vec<String>,
        properties: Option<ContextualExpression>,
        predicates: Vec<ContextualExpression>,
        direction: EdgeDirection,
        range: Option<EdgeRange>,
        span: Span,
    ) -> Self {
        Self {
            span,
            variable,
            edge_types,
            properties,
            predicates,
            direction,
            range,
        }
    }
}

/// 边范围
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeRange {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl EdgeRange {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self { min, max }
    }

    pub fn fixed(steps: usize) -> Self {
        Self {
            min: Some(steps),
            max: Some(steps),
        }
    }

    pub fn range(min: usize, max: usize) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn at_least(min: usize) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    pub fn at_most(max: usize) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }

    pub fn any() -> Self {
        Self {
            min: None,
            max: None,
        }
    }
}

/// 路径模式
#[derive(Debug, Clone, PartialEq)]
pub struct PathPattern {
    pub span: Span,
    pub elements: Vec<PathElement>,
}

impl PathPattern {
    pub fn new(elements: Vec<PathElement>, span: Span) -> Self {
        Self { span, elements }
    }
}

/// 路径元素
#[derive(Debug, Clone, PartialEq)]
pub enum PathElement {
    Node(NodePattern),
    Edge(EdgePattern),
    Alternative(Vec<Pattern>),
    Optional(Box<PathElement>),
    Repeated(Box<PathElement>, RepetitionType),
}

/// 重复类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RepetitionType {
    ZeroOrMore,          // *
    OneOrMore,           // +
    ZeroOrOne,           // ?
    Exactly(usize),      // {n}
    Range(usize, usize), // {n,m}
}

/// 变量模式
#[derive(Debug, Clone, PartialEq)]
pub struct VariablePattern {
    pub span: Span,
    pub name: String,
}

impl VariablePattern {
    pub fn new(name: String, span: Span) -> Self {
        Self { span, name }
    }
}

// 模式工具函数
pub struct PatternUtils;

impl PatternUtils {
    /// 查找模式中使用的所有变量
    pub fn find_variables(pattern: &Pattern) -> Vec<String> {
        let mut variables = Vec::new();
        Self::find_variables_recursive(pattern, &mut variables);
        variables
    }

    fn find_variables_recursive(pattern: &Pattern, variables: &mut Vec<String>) {
        match pattern {
            Pattern::Node(p) => {
                if let Some(ref var) = p.variable {
                    variables.push(var.clone());
                }
                if let Some(ref props) = p.properties {
                    variables.extend(collect_variables_from_contextual(props));
                }
                for predicate in &p.predicates {
                    variables.extend(collect_variables_from_contextual(predicate));
                }
            }
            Pattern::Edge(p) => {
                if let Some(ref var) = p.variable {
                    variables.push(var.clone());
                }
                if let Some(ref props) = p.properties {
                    variables.extend(collect_variables_from_contextual(props));
                }
                for predicate in &p.predicates {
                    variables.extend(collect_variables_from_contextual(predicate));
                }
            }
            Pattern::Path(p) => {
                for element in &p.elements {
                    Self::find_variables_in_element(element, variables);
                }
            }
            Pattern::Variable(p) => {
                variables.push(p.name.clone());
            }
        }
    }

    fn find_variables_in_element(element: &PathElement, variables: &mut Vec<String>) {
        match element {
            PathElement::Node(p) => {
                if let Some(ref var) = p.variable {
                    variables.push(var.clone());
                }
                if let Some(ref props) = p.properties {
                    variables.extend(collect_variables_from_contextual(props));
                }
                for predicate in &p.predicates {
                    variables.extend(collect_variables_from_contextual(predicate));
                }
            }
            PathElement::Edge(p) => {
                if let Some(ref var) = p.variable {
                    variables.push(var.clone());
                }
                if let Some(ref props) = p.properties {
                    variables.extend(collect_variables_from_contextual(props));
                }
                for predicate in &p.predicates {
                    variables.extend(collect_variables_from_contextual(predicate));
                }
            }
            PathElement::Alternative(patterns) => {
                for pattern in patterns {
                    Self::find_variables_recursive(pattern, variables);
                }
            }
            PathElement::Optional(elem) => {
                Self::find_variables_in_element(elem, variables);
            }
            PathElement::Repeated(elem, _) => {
                Self::find_variables_in_element(elem, variables);
            }
        }
    }

    /// 检查模式是否包含变量
    pub fn has_variables(pattern: &Pattern) -> bool {
        !Self::find_variables(pattern).is_empty()
    }

    /// 获取模式中的所有标签
    pub fn get_labels(pattern: &Pattern) -> Vec<String> {
        let mut labels = Vec::new();
        Self::get_labels_recursive(pattern, &mut labels);
        labels
    }

    fn get_labels_recursive(pattern: &Pattern, labels: &mut Vec<String>) {
        match pattern {
            Pattern::Node(p) => {
                labels.extend(p.labels.clone());
            }
            Pattern::Path(p) => {
                for element in &p.elements {
                    Self::get_labels_in_element(element, labels);
                }
            }
            _ => {}
        }
    }

    fn get_labels_in_element(element: &PathElement, labels: &mut Vec<String>) {
        match element {
            PathElement::Node(p) => {
                labels.extend(p.labels.clone());
            }
            PathElement::Alternative(patterns) => {
                for pattern in patterns {
                    Self::get_labels_recursive(pattern, labels);
                }
            }
            PathElement::Optional(elem) => {
                Self::get_labels_in_element(elem, labels);
            }
            PathElement::Repeated(elem, _) => {
                Self::get_labels_in_element(elem, labels);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_pattern() {
        let pattern = Pattern::Node(NodePattern::new(
            Some("n".to_string()),
            vec!["Person".to_string()],
            None,
            vec![],
            Span::default(),
        ));

        assert!(matches!(pattern, Pattern::Node(_)));
        let vars = PatternUtils::find_variables(&pattern);
        assert_eq!(vars, vec!["n"]);
    }

    #[test]
    fn test_edge_pattern() {
        let pattern = Pattern::Edge(EdgePattern::new(
            Some("e".to_string()),
            vec!["KNOWS".to_string()],
            None,
            vec![],
            EdgeDirection::Out,
            None,
            Span::default(),
        ));

        assert!(matches!(pattern, Pattern::Edge(_)));
        let vars = PatternUtils::find_variables(&pattern);
        assert_eq!(vars, vec!["e"]);
    }

    #[test]
    fn test_path_pattern() {
        let elements = vec![
            PathElement::Node(NodePattern::new(
                Some("a".to_string()),
                vec![],
                None,
                vec![],
                Span::default(),
            )),
            PathElement::Edge(EdgePattern::new(
                Some("e".to_string()),
                vec![],
                None,
                vec![],
                EdgeDirection::Out,
                None,
                Span::default(),
            )),
            PathElement::Node(NodePattern::new(
                Some("b".to_string()),
                vec![],
                None,
                vec![],
                Span::default(),
            )),
        ];

        let pattern = Pattern::Path(PathPattern::new(elements, Span::default()));
        let vars = PatternUtils::find_variables(&pattern);
        assert_eq!(vars, vec!["a", "e", "b"]);
    }

    #[test]
    fn test_edge_range() {
        let range1 = EdgeRange::fixed(2);
        assert_eq!(range1.min, Some(2));
        assert_eq!(range1.max, Some(2));

        let range2 = EdgeRange::range(1, 3);
        assert_eq!(range2.min, Some(1));
        assert_eq!(range2.max, Some(3));

        let range3 = EdgeRange::at_least(1);
        assert_eq!(range3.min, Some(1));
        assert_eq!(range3.max, None);

        let range4 = EdgeRange::any();
        assert_eq!(range4.min, None);
        assert_eq!(range4.max, None);
    }
}
