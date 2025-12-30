//! 计划节点引用模块
//!
//! 提供轻量级的计划节点引用，用于查询计划中的节点标识和依赖跟踪
//! 这是一个核心基础类型，供整个查询引擎使用

use std::fmt;

/// 计划节点引用
///
/// 用于在查询处理过程中轻量级地引用计划节点，避免存储完整的节点对象
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanNodeRef {
    /// 节点标识符
    pub id: String,
    /// 节点ID（来自 PlanNodeEnum.id()）
    pub node_id: i64,
}

impl PlanNodeRef {
    /// 创建新的计划节点引用
    pub fn new(id: String, node_id: i64) -> Self {
        Self { id, node_id }
    }

    /// 从节点ID创建引用
    pub fn from_node_id(id: String, node_id: i64) -> Self {
        Self { id, node_id }
    }

    /// 获取节点标识符
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取节点ID
    pub fn node_id(&self) -> i64 {
        self.node_id
    }

    /// 获取节点类型名称
    pub fn type_name(&self) -> &'static str {
        "PlanNode"
    }
}

impl fmt::Display for PlanNodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlanNodeRef({}, {})", self.id, self.node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_node_ref_creation() {
        let node_ref = PlanNodeRef::new("node_1".to_string(), 42);
        assert_eq!(node_ref.id(), "node_1");
        assert_eq!(node_ref.node_id(), 42);
    }

    #[test]
    fn test_plan_node_ref_display() {
        let node_ref = PlanNodeRef::new("node_1".to_string(), 42);
        assert_eq!(format!("{}", node_ref), "PlanNodeRef(node_1, 42)");
    }

    #[test]
    fn test_plan_node_ref_equality() {
        let ref1 = PlanNodeRef::new("node_1".to_string(), 42);
        let ref2 = PlanNodeRef::new("node_1".to_string(), 42);
        let ref3 = PlanNodeRef::new("node_2".to_string(), 42);

        assert_eq!(ref1, ref2);
        assert_ne!(ref1, ref3);
    }
}
