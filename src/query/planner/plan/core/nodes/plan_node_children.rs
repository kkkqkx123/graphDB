//! PlanNode 子节点遍历实现

use super::plan_node_enum::PlanNodeEnum;

impl PlanNodeEnum {
    /// 获取节点的所有子节点
    /// 用于遍历执行计划树
    pub fn children(&self) -> Vec<&PlanNodeEnum> {
        match self {
            // ZeroInputNode: 没有子节点
            PlanNodeEnum::Start(_) => vec![],
            PlanNodeEnum::CreateSpace(_) => vec![],
            PlanNodeEnum::DropSpace(_) => vec![],
            PlanNodeEnum::DescSpace(_) => vec![],
            PlanNodeEnum::ShowSpaces(_) => vec![],
            PlanNodeEnum::CreateTag(_) => vec![],
            PlanNodeEnum::AlterTag(_) => vec![],
            PlanNodeEnum::DescTag(_) => vec![],
            PlanNodeEnum::DropTag(_) => vec![],
            PlanNodeEnum::ShowTags(_) => vec![],
            PlanNodeEnum::CreateEdge(_) => vec![],
            PlanNodeEnum::AlterEdge(_) => vec![],
            PlanNodeEnum::DescEdge(_) => vec![],
            PlanNodeEnum::DropEdge(_) => vec![],
            PlanNodeEnum::ShowEdges(_) => vec![],
            PlanNodeEnum::CreateTagIndex(_) => vec![],
            PlanNodeEnum::DropTagIndex(_) => vec![],
            PlanNodeEnum::DescTagIndex(_) => vec![],
            PlanNodeEnum::ShowTagIndexes(_) => vec![],
            PlanNodeEnum::CreateEdgeIndex(_) => vec![],
            PlanNodeEnum::DropEdgeIndex(_) => vec![],
            PlanNodeEnum::DescEdgeIndex(_) => vec![],
            PlanNodeEnum::ShowEdgeIndexes(_) => vec![],
            PlanNodeEnum::RebuildTagIndex(_) => vec![],
            PlanNodeEnum::RebuildEdgeIndex(_) => vec![],
            PlanNodeEnum::CreateUser(_) => vec![],
            PlanNodeEnum::AlterUser(_) => vec![],
            PlanNodeEnum::DropUser(_) => vec![],
            PlanNodeEnum::ChangePassword(_) => vec![],
            PlanNodeEnum::InsertVertices(_) => vec![],
            PlanNodeEnum::InsertEdges(_) => vec![],
            PlanNodeEnum::IndexScan(_) => vec![],
            PlanNodeEnum::ScanVertices(_) => vec![],
            PlanNodeEnum::ScanEdges(_) => vec![],
            PlanNodeEnum::EdgeIndexScan(_) => vec![],
            PlanNodeEnum::GetVertices(_) => vec![],
            PlanNodeEnum::GetEdges(_) => vec![],
            PlanNodeEnum::GetNeighbors(_) => vec![],
            PlanNodeEnum::ShortestPath(_) => vec![],
            PlanNodeEnum::AllPaths(_) => vec![],
            PlanNodeEnum::BFSShortest(_) => vec![],
            PlanNodeEnum::MultiShortestPath(_) => vec![],

            // SingleInputNode: 有一个子节点
            PlanNodeEnum::Project(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Filter(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Sort(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Limit(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::TopN(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Sample(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Dedup(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::DataCollect(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Aggregate(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Unwind(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Assign(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::PatternApply(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::RollUpApply(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Traverse(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],

            // BinaryInputNode: 有两个子节点
            PlanNodeEnum::InnerJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],
            PlanNodeEnum::LeftJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],
            PlanNodeEnum::CrossJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],
            PlanNodeEnum::HashInnerJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],
            PlanNodeEnum::HashLeftJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],
            PlanNodeEnum::FullOuterJoin(node) => vec![super::plan_node_traits::BinaryInputNode::left_input(node), super::plan_node_traits::BinaryInputNode::right_input(node)],

            // MultipleInputNode: 有多个子节点
            PlanNodeEnum::Expand(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),
            PlanNodeEnum::ExpandAll(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),
            PlanNodeEnum::AppendVertices(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),

            // UnionNode: 使用 dependencies() 获取所有子节点
            PlanNodeEnum::Union(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),
            PlanNodeEnum::Minus(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),
            PlanNodeEnum::Intersect(node) => node.dependencies().iter().map(|b| b.as_ref()).collect(),

            // ControlFlowNode
            PlanNodeEnum::Argument(_) => vec![],
            PlanNodeEnum::Loop(node) => {
                let mut children = Vec::new();
                if let Some(body) = node.body() {
                    children.push(body.as_ref());
                }
                children
            },
            PlanNodeEnum::PassThrough(_) => vec![],
            PlanNodeEnum::Select(node) => {
                let mut children = Vec::new();
                if let Some(if_branch) = node.if_branch() {
                    children.push(if_branch.as_ref());
                }
                if let Some(else_branch) = node.else_branch() {
                    children.push(else_branch.as_ref());
                }
                children
            },
        }
    }
}
