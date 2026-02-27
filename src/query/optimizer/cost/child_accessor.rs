//! 子节点访问器
//!
//! 提供统一的方式来访问各种计划节点的子节点

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::{MultipleInputNode, SingleInputNode};

/// 子节点访问器 trait
///
/// 为不同类型的计划节点提供统一的子节点访问接口
pub trait ChildAccessor {
    /// 获取子节点数量
    fn child_count(&self) -> usize;

    /// 获取指定索引的可变子节点引用
    fn get_child_mut(&mut self, index: usize) -> Option<&mut PlanNodeEnum>;
}

impl ChildAccessor for PlanNodeEnum {
    fn child_count(&self) -> usize {
        self.children().len()
    }

    fn get_child_mut(&mut self, index: usize) -> Option<&mut PlanNodeEnum> {
        match self {
            // ==================== 双输入节点 ====================
            PlanNodeEnum::InnerJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },
            PlanNodeEnum::LeftJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },
            PlanNodeEnum::CrossJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },
            PlanNodeEnum::HashInnerJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },
            PlanNodeEnum::HashLeftJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },
            PlanNodeEnum::FullOuterJoin(n) => match index {
                0 => Some(n.left_input_mut()),
                1 => Some(n.right_input_mut()),
                _ => None,
            },

            // ==================== 单输入节点 ====================
            PlanNodeEnum::Project(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Filter(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Sort(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Limit(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::TopN(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Sample(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Dedup(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::DataCollect(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Aggregate(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Unwind(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Assign(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::PatternApply(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::RollUpApply(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Traverse(n) => {
                if index == 0 { Some(n.input_mut()) } else { None }
            }
            PlanNodeEnum::Union(n) => {
                n.dependencies_mut().get_mut(index).map(|b| b.as_mut())
            }
            PlanNodeEnum::Minus(n) => {
                n.dependencies_mut().get_mut(index).map(|b| b.as_mut())
            }
            PlanNodeEnum::Intersect(n) => {
                n.dependencies_mut().get_mut(index).map(|b| b.as_mut())
            }

            // ==================== 多输入节点 ====================
            PlanNodeEnum::Expand(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),
            PlanNodeEnum::ExpandAll(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),
            PlanNodeEnum::AppendVertices(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),
            PlanNodeEnum::GetVertices(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),
            PlanNodeEnum::GetNeighbors(n) => n.inputs_mut().get_mut(index).map(|b| b.as_mut()),

            // ==================== 控制流节点 ====================
            PlanNodeEnum::Loop(n) => {
                if index == 0 { n.body_mut().as_mut().map(|b| b.as_mut()) } else { None }
            }
            PlanNodeEnum::Select(n) => match index {
                0 => n.if_branch_mut().as_mut().map(|b| b.as_mut()),
                1 => n.else_branch_mut().as_mut().map(|b| b.as_mut()),
                _ => None,
            },

            // ==================== 无输入节点 ====================
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::graph_scan_node::ScanVerticesNode;

    #[test]
    fn test_scan_vertices_child_count() {
        let scan = ScanVerticesNode::new(1);
        let node = PlanNodeEnum::ScanVertices(scan);
        assert_eq!(node.child_count(), 0);
    }

    #[test]
    fn test_scan_vertices_get_child() {
        let scan = ScanVerticesNode::new(1);
        let mut node = PlanNodeEnum::ScanVertices(scan);
        assert!(node.get_child_mut(0).is_none());
    }
}
