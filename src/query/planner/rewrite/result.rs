//! 重写结果定义
//!
//! 定义重写规则的返回结果类型。
//! 这是从 optimizer 层独立出来的简化版本。

/// 重写错误类型
#[derive(Debug, thiserror::Error)]
pub enum RewriteError {
    #[error("无效的计划节点: {0}")]
    InvalidNode(String),

    #[error("重写失败: {0}")]
    RewriteFailed(String),

    #[error("不支持的节点类型: {0}")]
    UnsupportedNodeType(String),

    #[error("优化器错误: {0}")]
    OptimizerError(String),

    #[error("循环检测: 节点 {0}")]
    CycleDetected(usize),

    #[error("无效的计划结构: {0}")]
    InvalidPlanStructure(String),
}

impl RewriteError {
    pub fn invalid_node(msg: impl Into<String>) -> Self {
        Self::InvalidNode(msg.into())
    }

    pub fn rewrite_failed(msg: impl Into<String>) -> Self {
        Self::RewriteFailed(msg.into())
    }

    pub fn unsupported_node_type(name: impl Into<String>) -> Self {
        Self::UnsupportedNodeType(name.into())
    }

    pub fn optimizer_error(msg: impl Into<String>) -> Self {
        Self::OptimizerError(msg.into())
    }

    pub fn cycle_detected(node_id: usize) -> Self {
        Self::CycleDetected(node_id)
    }

    pub fn invalid_plan_structure(msg: impl Into<String>) -> Self {
        Self::InvalidPlanStructure(msg.into())
    }
}

/// 重写结果类型
pub type RewriteResult<T> = std::result::Result<T, RewriteError>;

/// 转换结果
///
/// 记录重写规则应用后的结果
#[derive(Debug, Default, Clone)]
pub struct TransformResult {
    /// 是否删除当前节点
    pub erase_curr: bool,
    /// 是否删除所有相关节点
    pub erase_all: bool,
    /// 新的计划节点列表
    pub new_nodes: Vec<crate::query::planner::plan::PlanNodeEnum>,
    /// 新的依赖关系
    pub new_dependencies: Vec<usize>,
}

impl TransformResult {
    /// 创建新的转换结果
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否删除当前节点
    pub fn with_erase_curr(mut self, erase_curr: bool) -> Self {
        self.erase_curr = erase_curr;
        self
    }

    /// 设置是否删除所有节点
    pub fn with_erase_all(mut self, erase_all: bool) -> Self {
        self.erase_all = erase_all;
        self
    }

    /// 添加新的计划节点
    pub fn add_new_node(&mut self, node: crate::query::planner::plan::PlanNodeEnum) {
        self.new_nodes.push(node);
    }

    /// 添加新的依赖
    pub fn add_new_dependency(&mut self, dep_id: usize) {
        self.new_dependencies.push(dep_id);
    }

    /// 设置已删除标记
    pub fn with_erased(mut self) -> Self {
        self.erase_curr = true;
        self
    }

    /// 检查是否有新节点
    pub fn has_new_nodes(&self) -> bool {
        !self.new_nodes.is_empty()
    }

    /// 获取第一个新节点（如果存在）
    pub fn first_new_node(&self) -> Option<&crate::query::planner::plan::PlanNodeEnum> {
        self.new_nodes.first()
    }
}

/// 匹配结果
///
/// 记录模式匹配的结果
#[derive(Debug, Default, Clone)]
pub struct MatchedResult {
    /// 匹配的节点列表
    pub nodes: Vec<crate::query::planner::plan::PlanNodeEnum>,
    /// 依赖节点列表
    pub dependencies: Vec<crate::query::planner::plan::PlanNodeEnum>,
    /// 根节点
    pub root_node: Option<crate::query::planner::plan::PlanNodeEnum>,
}

impl MatchedResult {
    /// 创建新的匹配结果
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加匹配的节点
    pub fn add_node(&mut self, node: crate::query::planner::plan::PlanNodeEnum) {
        self.nodes.push(node);
    }

    /// 添加依赖节点
    pub fn add_dependency(&mut self, node: crate::query::planner::plan::PlanNodeEnum) {
        self.dependencies.push(node);
    }

    /// 设置根节点
    pub fn set_root_node(&mut self, node: crate::query::planner::plan::PlanNodeEnum) {
        self.root_node = Some(node);
    }

    /// 检查是否有匹配的节点
    pub fn has_matches(&self) -> bool {
        !self.nodes.is_empty()
    }

    /// 获取第一个匹配的节点
    pub fn first_node(&self) -> Option<&crate::query::planner::plan::PlanNodeEnum> {
        self.nodes.first()
    }

    /// 获取第一个依赖节点
    pub fn first_dependency(&self) -> Option<&crate::query::planner::plan::PlanNodeEnum> {
        self.dependencies.first()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::graph_scan_node::ScanVerticesNode;

    #[test]
    fn test_transform_result() {
        let mut result = TransformResult::new();
        assert!(!result.has_new_nodes());

        let node = crate::query::planner::plan::PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        result.add_new_node(node);
        
        assert!(result.has_new_nodes());
        assert!(result.first_new_node().is_some());
    }

    #[test]
    fn test_matched_result() {
        let mut result = MatchedResult::new();
        assert!(!result.has_matches());

        let node = crate::query::planner::plan::PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        result.add_node(node);
        
        assert!(result.has_matches());
        assert!(result.first_node().is_some());
    }

    #[test]
    fn test_rewrite_error() {
        let err = RewriteError::invalid_node("test node");
        assert!(err.to_string().contains("test node"));

        let err = RewriteError::cycle_detected(42);
        assert!(err.to_string().contains("42"));
    }
}
