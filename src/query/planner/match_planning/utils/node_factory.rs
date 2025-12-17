use crate::query::planner::plan::core::nodes::PlanNodeFactory;
/// 节点工厂模块
/// 提供统一的节点创建逻辑，消除重复代码
use crate::query::planner::plan::PlanNode;
use crate::query::planner::plan::PlanNodeKind;
use crate::query::planner::planner::PlannerError;
use std::sync::Arc;

/// 创建起始节点
///
/// 这是所有查找策略的公共起始节点创建函数
/// 返回一个标准的起始节点，作为执行计划的根节点
pub fn create_start_node() -> Result<Arc<dyn PlanNode>, PlannerError> {
    Ok(PlanNodeFactory::create_start_node()?)
}

/// 创建嵌套起始节点
///
/// 创建一个嵌套的起始节点结构，用于某些需要多层嵌套的场景
pub fn create_nested_start_node() -> Result<Arc<dyn PlanNode>, PlannerError> {
    use crate::query::planner::plan::core::nodes::PlanNodeFactory;

    Ok(PlanNodeFactory::create_placeholder_node()?)
}

/// 创建空节点
///
/// 创建一个空的计划节点作为占位符
pub fn create_empty_node() -> Result<Arc<dyn PlanNode>, PlannerError> {
    Ok(PlanNodeFactory::create_start_node()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_start_node() {
        let result = create_start_node();
        assert!(result.is_ok());

        let start_node = result.unwrap();
        assert_eq!(start_node.kind(), PlanNodeKind::Start);
        assert_eq!(start_node.id(), -1);
        assert_eq!(start_node.dependencies().len(), 0);
        assert_eq!(start_node.cost(), 0.0);
    }

    #[test]
    fn test_create_nested_start_node() {
        let result = create_nested_start_node();
        assert!(result.is_ok());

        let nested_node = result.unwrap();
        assert_eq!(nested_node.kind(), PlanNodeKind::Start);
    }

    #[test]
    fn test_create_empty_node() {
        let result = create_empty_node();
        assert!(result.is_ok());

        let empty_node = result.unwrap();
        assert_eq!(empty_node.kind(), PlanNodeKind::Start);
        assert_eq!(empty_node.id(), -1);
        assert_eq!(empty_node.dependencies().len(), 0);
        assert_eq!(empty_node.cost(), 0.0);
    }
}
