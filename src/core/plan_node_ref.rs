//! 计划节点引用模块
//!
//! 提供轻量级的计划节点引用，用于查询计划中的节点标识和依赖跟踪
//! 这是一个核心基础类型，供整个查询引擎使用

use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// 计划节点引用
///
/// 轻量级标识符，用于在查询处理过程中引用计划节点
/// 使用 newtype 模式简化实现，提高性能
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanNodeRef(pub i64);

impl PlanNodeRef {
    /// 创建新的计划节点引用
    pub fn new(node_id: i64) -> Self {
        Self(node_id)
    }

    /// 获取节点ID
    pub fn node_id(&self) -> i64 {
        self.0
    }

    /// 从节点ID创建引用
    pub fn from_node_id(node_id: i64) -> Self {
        Self(node_id)
    }
}

impl Deref for PlanNodeRef {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i64> for PlanNodeRef {
    fn from(node_id: i64) -> Self {
        Self(node_id)
    }
}

impl From<PlanNodeRef> for i64 {
    fn from(ref_: PlanNodeRef) -> Self {
        ref_.0
    }
}

impl fmt::Display for PlanNodeRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PlanNodeRef({})", self.0)
    }
}

impl Hash for PlanNodeRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_node_ref_creation() {
        let node_ref = PlanNodeRef::new(42);
        assert_eq!(node_ref.node_id(), 42);
        assert_eq!(node_ref.0, 42);
    }

    #[test]
    fn test_plan_node_ref_display() {
        let node_ref = PlanNodeRef::new(42);
        assert_eq!(format!("{}", node_ref), "PlanNodeRef(42)");
    }

    #[test]
    fn test_plan_node_ref_equality() {
        let ref1 = PlanNodeRef::new(42);
        let ref2 = PlanNodeRef::new(42);
        let ref3 = PlanNodeRef::new(43);

        assert_eq!(ref1, ref2);
        assert_ne!(ref1, ref3);
    }

    #[test]
    fn test_plan_node_ref_from() {
        let ref_: PlanNodeRef = 42.into();
        assert_eq!(ref_.node_id(), 42);

        let id: i64 = ref_.into();
        assert_eq!(id, 42);
    }

    #[test]
    fn test_plan_node_ref_deref() {
        let ref_ = PlanNodeRef::new(42);
        assert_eq!(*ref_, 42);
    }

    #[test]
    fn test_plan_node_ref_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PlanNodeRef::new(1));
        set.insert(PlanNodeRef::new(1)); // 重复，应该只存在一个
        set.insert(PlanNodeRef::new(2));
        assert_eq!(set.len(), 2);
    }
}
