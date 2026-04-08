//! Implementation of PlanNode child node traversal

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::{MultipleInputNode, SingleInputNode};

impl PlanNodeEnum {
    /// Get all the child nodes of a node
    /// Used for traversing the execution plan tree
    pub fn children(&self) -> Vec<&PlanNodeEnum> {
        match self {
            // ZeroInputNode: Has no child nodes.
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
            PlanNodeEnum::ShowCreateTag(_) => vec![],
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
            PlanNodeEnum::GrantRole(_) => vec![],
            PlanNodeEnum::RevokeRole(_) => vec![],
            PlanNodeEnum::SwitchSpace(_) => vec![],
            PlanNodeEnum::AlterSpace(_) => vec![],
            PlanNodeEnum::ClearSpace(_) => vec![],
            PlanNodeEnum::ShowStats(_) => vec![],
            PlanNodeEnum::InsertVertices(_) => vec![],
            PlanNodeEnum::InsertEdges(_) => vec![],
            PlanNodeEnum::DeleteVertices(_) => vec![],
            PlanNodeEnum::DeleteEdges(_) => vec![],
            PlanNodeEnum::Update(_) => vec![],
            PlanNodeEnum::UpdateVertices(_) => vec![],
            PlanNodeEnum::UpdateEdges(_) => vec![],
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

            // SingleInputNode: There is a child node.
            PlanNodeEnum::Project(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::Filter(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::Sort(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Limit(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::TopN(node) => vec![super::plan_node_traits::SingleInputNode::input(node)],
            PlanNodeEnum::Sample(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::Dedup(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::DataCollect(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::Aggregate(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::Unwind(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::Assign(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::PatternApply(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::RollUpApply(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::Remove(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::Materialize(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }
            PlanNodeEnum::Traverse(node) => {
                vec![super::plan_node_traits::SingleInputNode::input(node)]
            }

            // BinaryInputNode: It has two child nodes.
            PlanNodeEnum::InnerJoin(node) => vec![
                super::plan_node_traits::BinaryInputNode::left_input(node),
                super::plan_node_traits::BinaryInputNode::right_input(node),
            ],
            PlanNodeEnum::LeftJoin(node) => vec![
                super::plan_node_traits::BinaryInputNode::left_input(node),
                super::plan_node_traits::BinaryInputNode::right_input(node),
            ],
            PlanNodeEnum::CrossJoin(node) => vec![
                super::plan_node_traits::BinaryInputNode::left_input(node),
                super::plan_node_traits::BinaryInputNode::right_input(node),
            ],
            PlanNodeEnum::HashInnerJoin(node) => vec![
                super::plan_node_traits::BinaryInputNode::left_input(node),
                super::plan_node_traits::BinaryInputNode::right_input(node),
            ],
            PlanNodeEnum::HashLeftJoin(node) => vec![
                super::plan_node_traits::BinaryInputNode::left_input(node),
                super::plan_node_traits::BinaryInputNode::right_input(node),
            ],
            PlanNodeEnum::FullOuterJoin(node) => vec![
                super::plan_node_traits::BinaryInputNode::left_input(node),
                super::plan_node_traits::BinaryInputNode::right_input(node),
            ],

            // MultipleInputNode: It has multiple child nodes.
            PlanNodeEnum::Expand(node) => node.inputs().iter().collect(),
            PlanNodeEnum::ExpandAll(node) => node.inputs().iter().collect(),
            PlanNodeEnum::AppendVertices(node) => node.inputs().iter().collect(),

            // UnionNode: 使用 dependencies() 获取所有子节点
            PlanNodeEnum::Union(node) => node.dependencies().iter().collect(),
            PlanNodeEnum::Minus(node) => {
                vec![node.input(), node.minus_input()]
            }
            PlanNodeEnum::Intersect(node) => {
                vec![node.input(), node.intersect_input()]
            }

            // ControlFlowNode
            PlanNodeEnum::Argument(_) => vec![],
            PlanNodeEnum::Loop(node) => {
                let mut children = Vec::new();
                if let Some(body) = node.body() {
                    children.push(body.as_ref());
                }
                children
            }
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
            }
            // Fulltext nodes - ZeroInputNode
            PlanNodeEnum::CreateFulltextIndex(_) => vec![],
            PlanNodeEnum::DropFulltextIndex(_) => vec![],
            PlanNodeEnum::AlterFulltextIndex(_) => vec![],
            PlanNodeEnum::ShowFulltextIndex(_) => vec![],
            PlanNodeEnum::DescribeFulltextIndex(_) => vec![],
            PlanNodeEnum::FulltextSearch(_) => vec![],
            PlanNodeEnum::FulltextLookup(_) => vec![],
            PlanNodeEnum::MatchFulltext(_) => vec![],
            // Vector Search Nodes
            PlanNodeEnum::VectorSearch(_) => vec![],
            PlanNodeEnum::CreateVectorIndex(_) => vec![],
            PlanNodeEnum::DropVectorIndex(_) => vec![],
            PlanNodeEnum::VectorLookup(_) => vec![],
            PlanNodeEnum::VectorMatch(_) => vec![],
        }
    }
}
