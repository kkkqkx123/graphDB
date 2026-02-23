//! 模式匹配定义
//!
//! 提供计划节点的模式匹配功能，用于重写规则识别特定计划结构。
//! 这是从 optimizer 层独立出来的简化版本，专注于启发式重写规则的需求。

use crate::query::planner::plan::PlanNodeEnum;

/// 模式结构体
///
/// 用于匹配计划树的特定结构。
/// 包含当前节点的匹配条件和子节点的模式。
#[derive(Debug, Clone)]
pub struct Pattern {
    /// 当前节点的匹配条件
    pub node: Option<MatchNode>,
    /// 子节点的模式列表
    pub dependencies: Vec<Pattern>,
}

impl Pattern {
    /// 创建空模式（匹配任何节点）
    pub fn new() -> Self {
        Self::default()
    }

    /// 使用指定节点创建模式
    pub fn with_node(node: MatchNode) -> Self {
        Self {
            node: Some(node),
            dependencies: Vec::new(),
        }
    }

    /// 使用节点名称创建模式
    pub fn new_with_name(name: &'static str) -> Self {
        Self::with_node(MatchNode::Single(name))
    }

    /// 使用多个可能的节点名称创建模式
    pub fn multi(node_names: Vec<&'static str>) -> Self {
        Self::with_node(MatchNode::Multi(node_names))
    }

    /// 添加子节点模式
    pub fn with_dependency(mut self, dependency: Pattern) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// 使用节点名称添加子节点模式
    pub fn with_dependency_name(mut self, name: &'static str) -> Self {
        self.dependencies.push(Self::new_with_name(name));
        self
    }

    /// 添加依赖模式（可变引用版本）
    pub fn add_dependency(&mut self, dependency: Pattern) {
        self.dependencies.push(dependency);
    }

    /// 检查模式是否匹配给定的计划节点
    pub fn matches(&self, plan_node: &PlanNodeEnum) -> bool {
        // 检查当前节点
        if let Some(ref node) = self.node {
            if !node.matches(plan_node.name()) {
                return false;
            }
        }

        // 如果没有依赖模式，直接匹配成功
        if self.dependencies.is_empty() {
            return true;
        }

        // 获取所有依赖节点的名称
        let dep_names: Vec<&str> = self.dependencies
            .iter()
            .filter_map(|d| d.node.as_ref())
            .filter_map(|n| n.as_single())
            .collect();

        if dep_names.is_empty() {
            return true;
        }

        // 检查每个依赖模式是否匹配
        for dep_pattern in &self.dependencies {
            let dep_matches = plan_node.dependencies().iter().any(|input| {
                if let Some(ref node) = dep_pattern.node {
                    node.matches(input.name())
                } else {
                    true
                }
            });

            if !dep_matches && !dep_pattern.dependencies.is_empty() {
                return false;
            }
        }

        true
    }

    // ==================== 便捷构造方法 ====================

    /// 创建匹配 Project 节点的模式
    pub fn with_project_matcher() -> Self {
        Self::new_with_name("Project")
    }

    /// 创建匹配 Filter 节点的模式
    pub fn with_filter_matcher() -> Self {
        Self::new_with_name("Filter")
    }

    /// 创建匹配 ScanVertices 节点的模式
    pub fn with_scan_vertices_matcher() -> Self {
        Self::new_with_name("ScanVertices")
    }

    /// 创建匹配 GetVertices 节点的模式
    pub fn with_get_vertices_matcher() -> Self {
        Self::new_with_name("GetVertices")
    }

    /// 创建匹配 Limit 节点的模式
    pub fn with_limit_matcher() -> Self {
        Self::new_with_name("Limit")
    }

    /// 创建匹配 Sort 节点的模式
    pub fn with_sort_matcher() -> Self {
        Self::new_with_name("Sort")
    }

    /// 创建匹配 Aggregate 节点的模式
    pub fn with_aggregate_matcher() -> Self {
        Self::new_with_name("Aggregate")
    }

    /// 创建匹配 Dedup 节点的模式
    pub fn with_dedup_matcher() -> Self {
        Self::new_with_name("Dedup")
    }

    /// 创建匹配 GetNeighbors 节点的模式
    pub fn with_get_neighbors_matcher() -> Self {
        Self::new_with_name("GetNeighbors")
    }

    /// 创建匹配 Traverse 节点的模式
    pub fn with_traverse_matcher() -> Self {
        Self::new_with_name("Traverse")
    }

    /// 创建匹配 Join 节点的模式（匹配任何连接类型）
    pub fn with_join_matcher() -> Self {
        Self::multi(vec!["HashInnerJoin", "HashLeftJoin", "InnerJoin", "LeftJoin", "CrossJoin", "FullOuterJoin"])
    }
}

impl Default for Pattern {
    fn default() -> Self {
        Self {
            node: None,
            dependencies: Vec::new(),
        }
    }
}

/// 节点匹配枚举
///
/// 定义如何匹配单个计划节点
#[derive(Debug, Clone)]
pub enum MatchNode {
    /// 匹配单个特定名称的节点
    Single(&'static str),
    /// 匹配多个可能名称中的任意一个
    Multi(Vec<&'static str>),
    /// 匹配任何节点
    Any,
}

impl MatchNode {
    /// 检查节点名称是否匹配
    pub fn matches(&self, node_name: &str) -> bool {
        match self {
            MatchNode::Single(name) => *name == node_name,
            MatchNode::Multi(names) => names.contains(&node_name),
            MatchNode::Any => true,
        }
    }

    /// 获取单个名称（如果是 Single 变体）
    pub fn as_single(&self) -> Option<&'static str> {
        match self {
            MatchNode::Single(name) => Some(name),
            _ => None,
        }
    }

    /// 获取多个名称（如果是 Multi 变体）
    pub fn as_multi(&self) -> Option<&Vec<&'static str>> {
        match self {
            MatchNode::Multi(names) => Some(names),
            _ => None,
        }
    }
}

/// 计划节点匹配器（复杂条件）
#[derive(Debug, Clone)]
pub enum PlanNodeMatcher {
    /// 匹配特定名称
    MatchNode(&'static str),
    /// 不匹配
    Not(Box<PlanNodeMatcher>),
    /// 所有条件都匹配
    And(Vec<PlanNodeMatcher>),
    /// 任意条件匹配
    Or(Vec<PlanNodeMatcher>),
}

impl PlanNodeMatcher {
    /// 检查是否匹配计划节点
    pub fn matches(&self, plan_node: &PlanNodeEnum) -> bool {
        match self {
            PlanNodeMatcher::MatchNode(name) => plan_node.name() == *name,
            PlanNodeMatcher::Not(matcher) => !matcher.matches(plan_node),
            PlanNodeMatcher::And(matchers) => matchers.iter().all(|m| m.matches(plan_node)),
            PlanNodeMatcher::Or(matchers) => matchers.iter().any(|m| m.matches(plan_node)),
        }
    }

    /// 与另一个匹配器组合（AND）
    pub fn and(self, other: PlanNodeMatcher) -> Self {
        PlanNodeMatcher::And(vec![self, other])
    }

    /// 与另一个匹配器组合（OR）
    pub fn or(self, other: PlanNodeMatcher) -> Self {
        PlanNodeMatcher::Or(vec![self, other])
    }
}

/// 模式构建器 trait
///
/// 允许自定义类型实现模式构建
pub trait PatternBuilder {
    /// 构建模式
    fn build(&self) -> Pattern;
}

/// 节点访问者 trait
///
/// 用于遍历计划树
pub trait NodeVisitor {
    /// 访问节点
    /// 返回 true 继续遍历，false 停止遍历
    fn visit(&mut self, node: &PlanNodeEnum) -> bool;
}

/// 节点记录访问者
///
/// 记录所有访问过的节点
#[derive(Debug, Default)]
pub struct NodeVisitorRecorder {
    pub nodes: Vec<PlanNodeEnum>,
}

impl NodeVisitorRecorder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, node: &PlanNodeEnum) {
        self.nodes.push(node.clone());
    }
}

impl NodeVisitor for NodeVisitorRecorder {
    fn visit(&mut self, node: &PlanNodeEnum) -> bool {
        self.record(node);
        true
    }
}

/// 节点查找访问者
///
/// 查找特定名称的节点
#[derive(Debug)]
pub struct NodeVisitorFinder {
    pub target_name: String,
    pub found_node: Option<PlanNodeEnum>,
}

impl NodeVisitorFinder {
    pub fn new(target_name: &str) -> Self {
        Self {
            target_name: target_name.to_string(),
            found_node: None,
        }
    }
}

impl NodeVisitor for NodeVisitorFinder {
    fn visit(&mut self, node: &PlanNodeEnum) -> bool {
        if node.name() == self.target_name {
            self.found_node = Some(node.clone());
            return false; // 找到后停止遍历
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::graph_scan_node::ScanVerticesNode;
    use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
    use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
    use crate::core::Value;
    use crate::core::Expression;

    #[test]
    fn test_pattern_matches() {
        let pattern = Pattern::new_with_name("Project");
        let input_node = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        let project_node = PlanNodeEnum::Project(
            ProjectNode::new(input_node.clone(), Vec::new()).expect("创建ProjectNode应该成功")
        );
        let filter_node = PlanNodeEnum::Filter(
            FilterNode::new(input_node, Expression::Literal(Value::Bool(true)))
                .expect("创建FilterNode应该成功")
        );
        
        assert!(pattern.matches(&project_node));
        assert!(!pattern.matches(&filter_node));
    }

    #[test]
    fn test_match_node_single() {
        let matcher = MatchNode::Single("Project");
        assert!(matcher.matches("Project"));
        assert!(!matcher.matches("Filter"));
    }

    #[test]
    fn test_match_node_multi() {
        let matcher = MatchNode::Multi(vec!["Project", "Filter"]);
        assert!(matcher.matches("Project"));
        assert!(matcher.matches("Filter"));
        assert!(!matcher.matches("ScanVertices"));
    }

    #[test]
    fn test_match_node_any() {
        let matcher = MatchNode::Any;
        assert!(matcher.matches("Project"));
        assert!(matcher.matches("Filter"));
        assert!(matcher.matches("ScanVertices"));
    }

    #[test]
    fn test_pattern_with_dependency() {
        let pattern = Pattern::new_with_name("Filter")
            .with_dependency_name("Project");
        
        let scan = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        let project = PlanNodeEnum::Project(
            ProjectNode::new(scan.clone(), Vec::new()).expect("创建ProjectNode应该成功")
        );
        let filter = PlanNodeEnum::Filter(
            FilterNode::new(project.clone(), Expression::Literal(Value::Bool(true)))
                .expect("创建FilterNode应该成功")
        );
        
        assert!(pattern.matches(&filter));
        
        // Filter -> Scan 不应该匹配
        let filter2 = PlanNodeEnum::Filter(
            FilterNode::new(scan, Expression::Literal(Value::Bool(true)))
                .expect("创建FilterNode应该成功")
        );
        assert!(!pattern.matches(&filter2));
    }
}
