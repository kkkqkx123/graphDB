//! PlanNode 枚举实现
//!
//! 使用枚举替代 trait objects，避免动态分发，提高性能



use crate::query::context::validate::types::Variable;
use std::fmt;

// 导入所有具体的节点类型
use super::start_node::StartNode;
use super::project_node::ProjectNode;
use super::sort_node::{SortNode, LimitNode, TopNNode};
use super::join_node::{InnerJoinNode, LeftJoinNode, CrossJoinNode};
use super::graph_scan_node::{GetVerticesNode, GetEdgesNode, GetNeighborsNode, ScanVerticesNode, ScanEdgesNode};
use super::traversal_node::{ExpandNode, ExpandAllNode, TraverseNode, AppendVerticesNode};
use super::placeholder_node::PlaceholderNode;

/// PlanNode 枚举，包含所有可能的节点类型
/// 
/// 这个枚举替代了 `PlanNodeEnum`，避免了动态分发的性能开销
#[derive(Debug, Clone)]
pub enum PlanNodeEnum {
    /// 起始节点
    Start(StartNode),
    /// 投影节点
    Project(ProjectNode),
    /// 排序节点
    Sort(SortNode),
    /// 限制节点
    Limit(LimitNode),
    /// TopN 节点
    TopN(TopNNode),
    /// 内连接节点
    InnerJoin(InnerJoinNode),
    /// 左连接节点
    LeftJoin(LeftJoinNode),
    /// 交叉连接节点
    CrossJoin(CrossJoinNode),
    /// 获取顶点节点
    GetVertices(GetVerticesNode),
    /// 获取边节点
    GetEdges(GetEdgesNode),
    /// 获取邻居节点
    GetNeighbors(GetNeighborsNode),
    /// 扫描顶点节点
    ScanVertices(ScanVerticesNode),
    /// 扫描边节点
    ScanEdges(ScanEdgesNode),
    /// 扩展节点
    Expand(ExpandNode),
    /// 全扩展节点
    ExpandAll(ExpandAllNode),
    /// 遍历节点
    Traverse(TraverseNode),
    /// 追加顶点节点
    AppendVertices(AppendVerticesNode),
    /// 占位符节点
    Placeholder(PlaceholderNode),
}

impl PlanNodeEnum {
    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        match self {
            PlanNodeEnum::Start(node) => node.id(),
            PlanNodeEnum::Project(node) => node.id(),
            PlanNodeEnum::Sort(node) => node.id(),
            PlanNodeEnum::Limit(node) => node.id(),
            PlanNodeEnum::TopN(node) => node.id(),
            PlanNodeEnum::InnerJoin(node) => node.id(),
            PlanNodeEnum::LeftJoin(node) => node.id(),
            PlanNodeEnum::CrossJoin(node) => node.id(),
            PlanNodeEnum::GetVertices(node) => node.id(),
            PlanNodeEnum::GetEdges(node) => node.id(),
            PlanNodeEnum::GetNeighbors(node) => node.id(),
            PlanNodeEnum::ScanVertices(node) => node.id(),
            PlanNodeEnum::ScanEdges(node) => node.id(),
            PlanNodeEnum::Expand(node) => node.id(),
            PlanNodeEnum::ExpandAll(node) => node.id(),
            PlanNodeEnum::Traverse(node) => node.id(),
            PlanNodeEnum::AppendVertices(node) => node.id(),
            PlanNodeEnum::Placeholder(node) => node.id(),
        }
    }

    /// 获取节点的类型
    pub fn kind(&self) -> PlanNodeKind {
        match self {
            PlanNodeEnum::Start(node) => node.kind(),
            PlanNodeEnum::Project(node) => node.kind(),
            PlanNodeEnum::Sort(node) => node.kind(),
            PlanNodeEnum::Limit(node) => node.kind(),
            PlanNodeEnum::TopN(node) => node.kind(),
            PlanNodeEnum::InnerJoin(node) => node.kind(),
            PlanNodeEnum::LeftJoin(node) => node.kind(),
            PlanNodeEnum::CrossJoin(node) => node.kind(),
            PlanNodeEnum::GetVertices(node) => node.kind(),
            PlanNodeEnum::GetEdges(node) => node.kind(),
            PlanNodeEnum::GetNeighbors(node) => node.kind(),
            PlanNodeEnum::ScanVertices(node) => node.kind(),
            PlanNodeEnum::ScanEdges(node) => node.kind(),
            PlanNodeEnum::Expand(node) => node.kind(),
            PlanNodeEnum::ExpandAll(node) => node.kind(),
            PlanNodeEnum::Traverse(node) => node.kind(),
            PlanNodeEnum::AppendVertices(node) => node.kind(),
            PlanNodeEnum::Placeholder(node) => node.kind(),
        }
    }

    /// 获取节点的输出变量
    pub fn output_var(&self) -> Option<&Variable> {
        match self {
            PlanNodeEnum::Start(node) => node.output_var(),
            PlanNodeEnum::Project(node) => node.output_var(),
            PlanNodeEnum::Sort(node) => node.output_var(),
            PlanNodeEnum::Limit(node) => node.output_var(),
            PlanNodeEnum::TopN(node) => node.output_var(),
            PlanNodeEnum::InnerJoin(node) => node.output_var(),
            PlanNodeEnum::LeftJoin(node) => node.output_var(),
            PlanNodeEnum::CrossJoin(node) => node.output_var(),
            PlanNodeEnum::GetVertices(node) => node.output_var(),
            PlanNodeEnum::GetEdges(node) => node.output_var(),
            PlanNodeEnum::GetNeighbors(node) => node.output_var(),
            PlanNodeEnum::ScanVertices(node) => node.output_var(),
            PlanNodeEnum::ScanEdges(node) => node.output_var(),
            PlanNodeEnum::Expand(node) => node.output_var(),
            PlanNodeEnum::ExpandAll(node) => node.output_var(),
            PlanNodeEnum::Traverse(node) => node.output_var(),
            PlanNodeEnum::AppendVertices(node) => node.output_var(),
            PlanNodeEnum::Placeholder(node) => node.output_var(),
        }
    }

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        match self {
            PlanNodeEnum::Start(node) => node.col_names(),
            PlanNodeEnum::Project(node) => node.col_names(),
            PlanNodeEnum::Sort(node) => node.col_names(),
            PlanNodeEnum::Limit(node) => node.col_names(),
            PlanNodeEnum::TopN(node) => node.col_names(),
            PlanNodeEnum::InnerJoin(node) => node.col_names(),
            PlanNodeEnum::LeftJoin(node) => node.col_names(),
            PlanNodeEnum::CrossJoin(node) => node.col_names(),
            PlanNodeEnum::GetVertices(node) => node.col_names(),
            PlanNodeEnum::GetEdges(node) => node.col_names(),
            PlanNodeEnum::GetNeighbors(node) => node.col_names(),
            PlanNodeEnum::ScanVertices(node) => node.col_names(),
            PlanNodeEnum::ScanEdges(node) => node.col_names(),
            PlanNodeEnum::Expand(node) => node.col_names(),
            PlanNodeEnum::ExpandAll(node) => node.col_names(),
            PlanNodeEnum::Traverse(node) => node.col_names(),
            PlanNodeEnum::AppendVertices(node) => node.col_names(),
            PlanNodeEnum::Placeholder(node) => node.col_names(),
        }
    }

    /// 获取节点的成本估计值
    pub fn cost(&self) -> f64 {
        match self {
            PlanNodeEnum::Start(node) => node.cost(),
            PlanNodeEnum::Project(node) => node.cost(),
            PlanNodeEnum::Sort(node) => node.cost(),
            PlanNodeEnum::Limit(node) => node.cost(),
            PlanNodeEnum::TopN(node) => node.cost(),
            PlanNodeEnum::InnerJoin(node) => node.cost(),
            PlanNodeEnum::LeftJoin(node) => node.cost(),
            PlanNodeEnum::CrossJoin(node) => node.cost(),
            PlanNodeEnum::GetVertices(node) => node.cost(),
            PlanNodeEnum::GetEdges(node) => node.cost(),
            PlanNodeEnum::GetNeighbors(node) => node.cost(),
            PlanNodeEnum::ScanVertices(node) => node.cost(),
            PlanNodeEnum::ScanEdges(node) => node.cost(),
            PlanNodeEnum::Expand(node) => node.cost(),
            PlanNodeEnum::ExpandAll(node) => node.cost(),
            PlanNodeEnum::Traverse(node) => node.cost(),
            PlanNodeEnum::AppendVertices(node) => node.cost(),
            PlanNodeEnum::Placeholder(node) => node.cost(),
        }
    }

    /// 获取节点的依赖节点列表
    pub fn dependencies(&self) -> Vec<PlanNodeEnum> {
        match self {
            PlanNodeEnum::Start(node) => {
                // StartNode 没有依赖
                vec![]
            }
            PlanNodeEnum::Project(node) => {
                // 从具体节点获取依赖，需要重构具体节点实现
                vec![]
            }
            PlanNodeEnum::Sort(node) => {
                vec![]
            }
            PlanNodeEnum::Limit(node) => {
                vec![]
            }
            PlanNodeEnum::TopN(node) => {
                vec![]
            }
            PlanNodeEnum::InnerJoin(node) => {
                vec![]
            }
            PlanNodeEnum::LeftJoin(node) => {
                vec![]
            }
            PlanNodeEnum::CrossJoin(node) => {
                vec![]
            }
            PlanNodeEnum::GetVertices(node) => {
                vec![]
            }
            PlanNodeEnum::GetEdges(node) => {
                vec![]
            }
            PlanNodeEnum::GetNeighbors(node) => {
                vec![]
            }
            PlanNodeEnum::ScanVertices(node) => {
                vec![]
            }
            PlanNodeEnum::ScanEdges(node) => {
                vec![]
            }
            PlanNodeEnum::Expand(node) => {
                vec![]
            }
            PlanNodeEnum::ExpandAll(node) => {
                vec![]
            }
            PlanNodeEnum::Traverse(node) => {
                vec![]
            }
            PlanNodeEnum::AppendVertices(node) => {
                vec![]
            }
            PlanNodeEnum::Placeholder(node) => {
                vec![]
            }
        }
    }

    /// 设置节点的输出变量
    pub fn set_output_var(&mut self, var: Variable) {
        match self {
            PlanNodeEnum::Start(node) => node.set_output_var(var),
            PlanNodeEnum::Project(node) => node.set_output_var(var),
            PlanNodeEnum::Sort(node) => node.set_output_var(var),
            PlanNodeEnum::Limit(node) => node.set_output_var(var),
            PlanNodeEnum::TopN(node) => node.set_output_var(var),
            PlanNodeEnum::InnerJoin(node) => node.set_output_var(var),
            PlanNodeEnum::LeftJoin(node) => node.set_output_var(var),
            PlanNodeEnum::CrossJoin(node) => node.set_output_var(var),
            PlanNodeEnum::GetVertices(node) => node.set_output_var(var),
            PlanNodeEnum::GetEdges(node) => node.set_output_var(var),
            PlanNodeEnum::GetNeighbors(node) => node.set_output_var(var),
            PlanNodeEnum::ScanVertices(node) => node.set_output_var(var),
            PlanNodeEnum::ScanEdges(node) => node.set_output_var(var),
            PlanNodeEnum::Expand(node) => node.set_output_var(var),
            PlanNodeEnum::ExpandAll(node) => node.set_output_var(var),
            PlanNodeEnum::Traverse(node) => node.set_output_var(var),
            PlanNodeEnum::AppendVertices(node) => node.set_output_var(var),
            PlanNodeEnum::Placeholder(node) => node.set_output_var(var),
        }
    }

    /// 设置列名
    pub fn set_col_names(&mut self, names: Vec<String>) {
        match self {
            PlanNodeEnum::Start(node) => node.set_col_names(names),
            PlanNodeEnum::Project(node) => node.set_col_names(names),
            PlanNodeEnum::Sort(node) => node.set_col_names(names),
            PlanNodeEnum::Limit(node) => node.set_col_names(names),
            PlanNodeEnum::TopN(node) => node.set_col_names(names),
            PlanNodeEnum::InnerJoin(node) => node.set_col_names(names),
            PlanNodeEnum::LeftJoin(node) => node.set_col_names(names),
            PlanNodeEnum::CrossJoin(node) => node.set_col_names(names),
            PlanNodeEnum::GetVertices(node) => node.set_col_names(names),
            PlanNodeEnum::GetEdges(node) => node.set_col_names(names),
            PlanNodeEnum::GetNeighbors(node) => node.set_col_names(names),
            PlanNodeEnum::ScanVertices(node) => node.set_col_names(names),
            PlanNodeEnum::ScanEdges(node) => node.set_col_names(names),
            PlanNodeEnum::Expand(node) => node.set_col_names(names),
            PlanNodeEnum::ExpandAll(node) => node.set_col_names(names),
            PlanNodeEnum::Traverse(node) => node.set_col_names(names),
            PlanNodeEnum::AppendVertices(node) => node.set_col_names(names),
            PlanNodeEnum::Placeholder(node) => node.set_col_names(names),
        }
    }

    /// 使用访问者模式访问节点
    pub fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        match self {
            PlanNodeEnum::Start(node) => node.accept(visitor),
            PlanNodeEnum::Project(node) => node.accept(visitor),
            PlanNodeEnum::Sort(node) => node.accept(visitor),
            PlanNodeEnum::Limit(node) => node.accept(visitor),
            PlanNodeEnum::TopN(node) => node.accept(visitor),
            PlanNodeEnum::InnerJoin(node) => node.accept(visitor),
            PlanNodeEnum::LeftJoin(node) => node.accept(visitor),
            PlanNodeEnum::CrossJoin(node) => node.accept(visitor),
            PlanNodeEnum::GetVertices(node) => node.accept(visitor),
            PlanNodeEnum::GetEdges(node) => node.accept(visitor),
            PlanNodeEnum::GetNeighbors(node) => node.accept(visitor),
            PlanNodeEnum::ScanVertices(node) => node.accept(visitor),
            PlanNodeEnum::ScanEdges(node) => node.accept(visitor),
            PlanNodeEnum::Expand(node) => node.accept(visitor),
            PlanNodeEnum::ExpandAll(node) => node.accept(visitor),
            PlanNodeEnum::Traverse(node) => node.accept(visitor),
            PlanNodeEnum::AppendVertices(node) => node.accept(visitor),
            PlanNodeEnum::Placeholder(node) => node.accept(visitor),
        }
    }

    /// 克隆节点
    pub fn clone_plan_node(&self) -> PlanNodeEnum {
        self.clone()
    }

    /// 克隆节点并分配新的ID
    pub fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        match self {
            PlanNodeEnum::Start(node) => {
                let mut cloned = node.clone();
                // 这里需要访问内部字段来设置新ID
                // 暂时使用简单克隆，后续需要为每个节点实现 set_id 方法
                PlanNodeEnum::Start(cloned)
            }
            PlanNodeEnum::Project(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Project(cloned)
            }
            PlanNodeEnum::Sort(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Sort(cloned)
            }
            PlanNodeEnum::Limit(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Limit(cloned)
            }
            PlanNodeEnum::TopN(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::TopN(cloned)
            }
            PlanNodeEnum::InnerJoin(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::InnerJoin(cloned)
            }
            PlanNodeEnum::LeftJoin(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::LeftJoin(cloned)
            }
            PlanNodeEnum::CrossJoin(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::CrossJoin(cloned)
            }
            PlanNodeEnum::GetVertices(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::GetVertices(cloned)
            }
            PlanNodeEnum::GetEdges(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::GetEdges(cloned)
            }
            PlanNodeEnum::GetNeighbors(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::GetNeighbors(cloned)
            }
            PlanNodeEnum::ScanVertices(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::ScanVertices(cloned)
            }
            PlanNodeEnum::ScanEdges(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::ScanEdges(cloned)
            }
            PlanNodeEnum::Expand(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Expand(cloned)
            }
            PlanNodeEnum::ExpandAll(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::ExpandAll(cloned)
            }
            PlanNodeEnum::Traverse(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Traverse(cloned)
            }
            PlanNodeEnum::AppendVertices(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::AppendVertices(cloned)
            }
            PlanNodeEnum::Placeholder(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Placeholder(cloned)
            }
        }
    }
}

impl fmt::Display for PlanNodeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.kind(), self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_node_enum_creation() {
        let start_node = StartNode::new();
        let enum_node = PlanNodeEnum::Start(start_node);

        assert_eq!(enum_node.kind(), PlanNodeKind::Start);
        assert_eq!(enum_node.id(), -1);
    }

    #[test]
    fn test_plan_node_enum_display() {
        let start_node = StartNode::new();
        let enum_node = PlanNodeEnum::Start(start_node);

        assert_eq!(format!("{}", enum_node), "Start(-1)");
    }
}