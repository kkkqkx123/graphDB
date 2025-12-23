//! 计划节点引用模块
//!
//! 提供轻量级的计划节点引用，用于符号表中的依赖跟踪

use std::fmt;

/// 计划节点引用
///
/// 用于在符号表中轻量级地引用计划节点，避免存储完整的节点对象
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

    /// 获取节点类型名称（通过节点ID查找）
    ///
    /// 注意：这是一个简化实现，实际使用中可能需要通过全局注册表或上下文查找
    pub fn type_name(&self) -> &'static str {
        // 这里可以根据 node_id 查找实际类型
        // 为了简化，暂时返回通用名称
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
    fn test_plan_node_ref_from_node_id() {
        let node_ref = PlanNodeRef::from_node_id("test_node".to_string(), 123);
        assert_eq!(node_ref.id(), "test_node");
        assert_eq!(node_ref.node_id(), 123);
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

    #[test]
    fn test_plan_node_ref_hash() {
        use std::collections::HashSet;
        
        let ref1 = PlanNodeRef::new("node_1".to_string(), 42);
        let ref2 = PlanNodeRef::new("node_1".to_string(), 42);
        let ref3 = PlanNodeRef::new("node_2".to_string(), 42);
        
        let mut set = HashSet::new();
        set.insert(ref1.clone());
        set.insert(ref2);
        set.insert(ref3);
        
        assert_eq!(set.len(), 2); // ref1 和 ref2 相同，ref3 不同
    }
}