//! PlanNode 枚举实现
//!
//! 使用枚举替代 trait objects，避免动态分发，提高性能
//! 实现零成本抽象的核心系统

use crate::core::error::PlanNodeVisitError;
use crate::query::context::validate::types::Variable;

// 导入所有具体的节点类型
use super::aggregate_node::AggregateNode;
use super::control_flow_node::{ArgumentNode, LoopNode, PassThroughNode, SelectNode};
use super::data_processing_node::{
    DataCollectNode, DedupNode, PatternApplyNode, RollUpApplyNode, UnionNode, UnwindNode,
};
use super::filter_node::FilterNode;
use super::graph_scan_node::{
    GetEdgesNode, GetNeighborsNode, GetVerticesNode, ScanEdgesNode, ScanVerticesNode,
};
use super::join_node::{CrossJoinNode, InnerJoinNode, LeftJoinNode};
use super::project_node::ProjectNode;
use super::sort_node::{LimitNode, SortNode, TopNNode};
use super::start_node::StartNode;
use super::traversal_node::{AppendVerticesNode, ExpandAllNode, ExpandNode, TraverseNode};

/// 零成本访问者trait - 使用泛型避免动态分发
pub trait PlanNodeVisitor {
    type Result;

    /// 访问Start节点 - 编译时分发
    fn visit_start(&mut self, node: &StartNode) -> Self::Result;

    /// 访问Project节点 - 编译时分发
    fn visit_project(&mut self, node: &ProjectNode) -> Self::Result;

    /// 访问Sort节点 - 编译时分发
    fn visit_sort(&mut self, node: &SortNode) -> Self::Result;

    /// 访问Limit节点 - 编译时分发
    fn visit_limit(&mut self, node: &LimitNode) -> Self::Result;

    /// 访问TopN节点 - 编译时分发
    fn visit_topn(&mut self, node: &TopNNode) -> Self::Result;

    /// 访问InnerJoin节点 - 编译时分发
    fn visit_inner_join(&mut self, node: &InnerJoinNode) -> Self::Result;

    /// 访问LeftJoin节点 - 编译时分发
    fn visit_left_join(&mut self, node: &LeftJoinNode) -> Self::Result;

    /// 访问CrossJoin节点 - 编译时分发
    fn visit_cross_join(&mut self, node: &CrossJoinNode) -> Self::Result;

    /// 访问GetVertices节点 - 编译时分发
    fn visit_get_vertices(&mut self, node: &GetVerticesNode) -> Self::Result;

    /// 访问GetEdges节点 - 编译时分发
    fn visit_get_edges(&mut self, node: &GetEdgesNode) -> Self::Result;

    /// 访问GetNeighbors节点 - 编译时分发
    fn visit_get_neighbors(&mut self, node: &GetNeighborsNode) -> Self::Result;

    /// 访问ScanVertices节点 - 编译时分发
    fn visit_scan_vertices(&mut self, node: &ScanVerticesNode) -> Self::Result;

    /// 访问ScanEdges节点 - 编译时分发
    fn visit_scan_edges(&mut self, node: &ScanEdgesNode) -> Self::Result;

    /// 访问Expand节点 - 编译时分发
    fn visit_expand(&mut self, node: &ExpandNode) -> Self::Result;

    /// 访问ExpandAll节点 - 编译时分发
    fn visit_expand_all(&mut self, node: &ExpandAllNode) -> Self::Result;

    /// 访问Traverse节点 - 编译时分发
    fn visit_traverse(&mut self, node: &TraverseNode) -> Self::Result;

    /// 访问AppendVertices节点 - 编译时分发
    fn visit_append_vertices(&mut self, node: &AppendVerticesNode) -> Self::Result;

    /// 访问Filter节点 - 编译时分发
    fn visit_filter(&mut self, node: &FilterNode) -> Self::Result;

    /// 访问Aggregate节点 - 编译时分发
    fn visit_aggregate(&mut self, node: &AggregateNode) -> Self::Result;

    /// 访问Argument节点 - 编译时分发
    fn visit_argument(&mut self, node: &ArgumentNode) -> Self::Result;

    /// 访问Loop节点 - 编译时分发
    fn visit_loop(&mut self, node: &LoopNode) -> Self::Result;

    /// 访问PassThrough节点 - 编译时分发
    fn visit_pass_through(&mut self, node: &PassThroughNode) -> Self::Result;

    /// 访问Select节点 - 编译时分发
    fn visit_select(&mut self, node: &SelectNode) -> Self::Result;

    /// 访问DataCollect节点 - 编译时分发
    fn visit_data_collect(&mut self, node: &DataCollectNode) -> Self::Result;

    /// 访问Dedup节点 - 编译时分发
    fn visit_dedup(&mut self, node: &DedupNode) -> Self::Result;

    /// 访问PatternApply节点 - 编译时分发
    fn visit_pattern_apply(&mut self, node: &PatternApplyNode) -> Self::Result;

    /// 访问RollUpApply节点 - 编译时分发
    fn visit_roll_up_apply(&mut self, node: &RollUpApplyNode) -> Self::Result;

    /// 访问Union节点 - 编译时分发
    fn visit_union(&mut self, node: &UnionNode) -> Self::Result;

    /// 访问Unwind节点 - 编译时分发
    fn visit_unwind(&mut self, node: &UnwindNode) -> Self::Result;
}

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
    /// 过滤节点
    Filter(FilterNode),
    /// 聚合节点
    Aggregate(AggregateNode),
    /// 参数节点
    Argument(ArgumentNode),
    /// 循环节点
    Loop(LoopNode),
    /// 透传节点
    PassThrough(PassThroughNode),
    /// 选择节点
    Select(SelectNode),
    /// 数据收集节点
    DataCollect(DataCollectNode),
    /// 去重节点
    Dedup(DedupNode),
    /// 模式应用节点
    PatternApply(PatternApplyNode),
    /// 滚动应用节点
    RollUpApply(RollUpApplyNode),
    /// 并集节点
    Union(UnionNode),
    /// 展开节点
    Unwind(UnwindNode),
}

impl PlanNodeEnum {
    /// 零成本类型检查 - 直接使用模式匹配
    pub fn is_start(&self) -> bool {
        matches!(self, PlanNodeEnum::Start(_))
    }

    pub fn is_project(&self) -> bool {
        matches!(self, PlanNodeEnum::Project(_))
    }

    pub fn is_filter(&self) -> bool {
        matches!(self, PlanNodeEnum::Filter(_))
    }

    pub fn is_sort(&self) -> bool {
        matches!(self, PlanNodeEnum::Sort(_))
    }

    pub fn is_limit(&self) -> bool {
        matches!(self, PlanNodeEnum::Limit(_))
    }

    /// 零成本类型转换 - 直接使用模式匹配
    pub fn as_start(&self) -> Option<&StartNode> {
        match self {
            PlanNodeEnum::Start(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_project(&self) -> Option<&ProjectNode> {
        match self {
            PlanNodeEnum::Project(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_filter(&self) -> Option<&FilterNode> {
        match self {
            PlanNodeEnum::Filter(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_sort(&self) -> Option<&SortNode> {
        match self {
            PlanNodeEnum::Sort(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_limit(&self) -> Option<&LimitNode> {
        match self {
            PlanNodeEnum::Limit(node) => Some(node),
            _ => None,
        }
    }

    /// 零成本类型转换（可变） - 直接使用模式匹配
    pub fn as_start_mut(&mut self) -> Option<&mut StartNode> {
        match self {
            PlanNodeEnum::Start(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_project_mut(&mut self) -> Option<&mut ProjectNode> {
        match self {
            PlanNodeEnum::Project(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_filter_mut(&mut self) -> Option<&mut FilterNode> {
        match self {
            PlanNodeEnum::Filter(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_sort_mut(&mut self) -> Option<&mut SortNode> {
        match self {
            PlanNodeEnum::Sort(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_limit_mut(&mut self) -> Option<&mut LimitNode> {
        match self {
            PlanNodeEnum::Limit(node) => Some(node),
            _ => None,
        }
    }

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
            PlanNodeEnum::Filter(node) => node.id(),
            PlanNodeEnum::Aggregate(node) => node.id(),
            PlanNodeEnum::Argument(node) => node.id(),
            PlanNodeEnum::Loop(node) => node.id(),
            PlanNodeEnum::PassThrough(node) => node.id(),
            PlanNodeEnum::Select(node) => node.id(),
            PlanNodeEnum::DataCollect(node) => node.id(),
            PlanNodeEnum::Dedup(node) => node.id(),
            PlanNodeEnum::PatternApply(node) => node.id(),
            PlanNodeEnum::RollUpApply(node) => node.id(),
            PlanNodeEnum::Union(node) => node.id(),
            PlanNodeEnum::Unwind(node) => node.id(),
        }
    }

    /// 获取节点类型的名称
    pub fn name(&self) -> &'static str {
        match self {
            PlanNodeEnum::Start(_) => "Start",
            PlanNodeEnum::Project(_) => "Project",
            PlanNodeEnum::Sort(_) => "Sort",
            PlanNodeEnum::Limit(_) => "Limit",
            PlanNodeEnum::TopN(_) => "TopN",
            PlanNodeEnum::InnerJoin(_) => "InnerJoin",
            PlanNodeEnum::LeftJoin(_) => "LeftJoin",
            PlanNodeEnum::CrossJoin(_) => "CrossJoin",
            PlanNodeEnum::GetVertices(_) => "GetVertices",
            PlanNodeEnum::GetEdges(_) => "GetEdges",
            PlanNodeEnum::GetNeighbors(_) => "GetNeighbors",
            PlanNodeEnum::ScanVertices(_) => "ScanVertices",
            PlanNodeEnum::ScanEdges(_) => "ScanEdges",
            PlanNodeEnum::Expand(_) => "Expand",
            PlanNodeEnum::ExpandAll(_) => "ExpandAll",
            PlanNodeEnum::Traverse(_) => "Traverse",
            PlanNodeEnum::AppendVertices(_) => "AppendVertices",
            PlanNodeEnum::Filter(_) => "Filter",
            PlanNodeEnum::Aggregate(_) => "Aggregate",
            PlanNodeEnum::Argument(_) => "Argument",
            PlanNodeEnum::Loop(_) => "Loop",
            PlanNodeEnum::PassThrough(_) => "PassThrough",
            PlanNodeEnum::Select(_) => "Select",
            PlanNodeEnum::DataCollect(_) => "DataCollect",
            PlanNodeEnum::Dedup(_) => "Dedup",
            PlanNodeEnum::PatternApply(_) => "PatternApply",
            PlanNodeEnum::RollUpApply(_) => "RollUpApply",
            PlanNodeEnum::Union(_) => "Union",
            PlanNodeEnum::Unwind(_) => "Unwind",
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
            PlanNodeEnum::Filter(node) => node.output_var(),
            PlanNodeEnum::Aggregate(node) => node.output_var(),
            PlanNodeEnum::Argument(node) => node.output_var(),
            PlanNodeEnum::Loop(node) => node.output_var(),
            PlanNodeEnum::PassThrough(node) => node.output_var(),
            PlanNodeEnum::Select(node) => node.output_var(),
            PlanNodeEnum::DataCollect(node) => node.output_var(),
            PlanNodeEnum::Dedup(node) => node.output_var(),
            PlanNodeEnum::PatternApply(node) => node.output_var(),
            PlanNodeEnum::RollUpApply(node) => node.output_var(),
            PlanNodeEnum::Union(node) => node.output_var(),
            PlanNodeEnum::Unwind(node) => node.output_var(),
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
            PlanNodeEnum::Filter(node) => node.col_names(),
            PlanNodeEnum::Aggregate(node) => node.col_names(),
            PlanNodeEnum::Argument(node) => node.col_names(),
            PlanNodeEnum::Loop(node) => node.col_names(),
            PlanNodeEnum::PassThrough(node) => node.col_names(),
            PlanNodeEnum::Select(node) => node.col_names(),
            PlanNodeEnum::DataCollect(node) => node.col_names(),
            PlanNodeEnum::Dedup(node) => node.col_names(),
            PlanNodeEnum::PatternApply(node) => node.col_names(),
            PlanNodeEnum::RollUpApply(node) => node.col_names(),
            PlanNodeEnum::Union(node) => node.col_names(),
            PlanNodeEnum::Unwind(node) => node.col_names(),
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
            PlanNodeEnum::Filter(node) => node.cost(),
            PlanNodeEnum::Aggregate(node) => node.cost(),
            PlanNodeEnum::Argument(node) => node.cost(),
            PlanNodeEnum::Loop(node) => node.cost(),
            PlanNodeEnum::PassThrough(node) => node.cost(),
            PlanNodeEnum::Select(node) => node.cost(),
            PlanNodeEnum::DataCollect(node) => node.cost(),
            PlanNodeEnum::Dedup(node) => node.cost(),
            PlanNodeEnum::PatternApply(node) => node.cost(),
            PlanNodeEnum::RollUpApply(node) => node.cost(),
            PlanNodeEnum::Union(node) => node.cost(),
            PlanNodeEnum::Unwind(node) => node.cost(),
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
            PlanNodeEnum::Filter(node) => node.dependencies(),
            PlanNodeEnum::Aggregate(node) => {
                vec![]
            }
            PlanNodeEnum::Argument(node) => {
                vec![]
            }
            PlanNodeEnum::Loop(node) => {
                vec![]
            }
            PlanNodeEnum::PassThrough(node) => {
                vec![]
            }
            PlanNodeEnum::Select(node) => {
                vec![]
            }
            PlanNodeEnum::DataCollect(node) => {
                vec![]
            }
            PlanNodeEnum::Dedup(node) => {
                vec![]
            }
            PlanNodeEnum::PatternApply(node) => {
                vec![]
            }
            PlanNodeEnum::RollUpApply(node) => {
                vec![]
            }
            PlanNodeEnum::Union(node) => {
                vec![]
            }
            PlanNodeEnum::Unwind(node) => {
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
            PlanNodeEnum::Filter(node) => node.set_output_var(var),
            PlanNodeEnum::Aggregate(node) => node.set_output_var(var),
            PlanNodeEnum::Argument(node) => node.set_output_var(var),
            PlanNodeEnum::Loop(node) => node.set_output_var(var),
            PlanNodeEnum::PassThrough(node) => node.set_output_var(var),
            PlanNodeEnum::Select(node) => node.set_output_var(var),
            PlanNodeEnum::DataCollect(node) => node.set_output_var(var),
            PlanNodeEnum::Dedup(node) => node.set_output_var(var),
            PlanNodeEnum::PatternApply(node) => node.set_output_var(var),
            PlanNodeEnum::RollUpApply(node) => node.set_output_var(var),
            PlanNodeEnum::Union(node) => node.set_output_var(var),
            PlanNodeEnum::Unwind(node) => node.set_output_var(var),
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
            PlanNodeEnum::Filter(node) => node.set_col_names(names),
            PlanNodeEnum::Aggregate(node) => node.set_col_names(names),
            PlanNodeEnum::Argument(node) => node.set_col_names(names),
            PlanNodeEnum::Loop(node) => node.set_col_names(names),
            PlanNodeEnum::PassThrough(node) => node.set_col_names(names),
            PlanNodeEnum::Select(node) => node.set_col_names(names),
            PlanNodeEnum::DataCollect(node) => node.set_col_names(names),
            PlanNodeEnum::Dedup(node) => node.set_col_names(names),
            PlanNodeEnum::PatternApply(node) => node.set_col_names(names),
            PlanNodeEnum::RollUpApply(node) => node.set_col_names(names),
            PlanNodeEnum::Union(node) => node.set_col_names(names),
            PlanNodeEnum::Unwind(node) => node.set_col_names(names),
        }
    }

    /// 零成本访问者模式 - 编译时分发
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: PlanNodeVisitor,
    {
        match self {
            PlanNodeEnum::Start(node) => visitor.visit_start(node),
            PlanNodeEnum::Project(node) => visitor.visit_project(node),
            PlanNodeEnum::Sort(node) => visitor.visit_sort(node),
            PlanNodeEnum::Limit(node) => visitor.visit_limit(node),
            PlanNodeEnum::TopN(node) => visitor.visit_topn(node),
            PlanNodeEnum::InnerJoin(node) => visitor.visit_inner_join(node),
            PlanNodeEnum::LeftJoin(node) => visitor.visit_left_join(node),
            PlanNodeEnum::CrossJoin(node) => visitor.visit_cross_join(node),
            PlanNodeEnum::GetVertices(node) => visitor.visit_get_vertices(node),
            PlanNodeEnum::GetEdges(node) => visitor.visit_get_edges(node),
            PlanNodeEnum::GetNeighbors(node) => visitor.visit_get_neighbors(node),
            PlanNodeEnum::ScanVertices(node) => visitor.visit_scan_vertices(node),
            PlanNodeEnum::ScanEdges(node) => visitor.visit_scan_edges(node),
            PlanNodeEnum::Expand(node) => visitor.visit_expand(node),
            PlanNodeEnum::ExpandAll(node) => visitor.visit_expand_all(node),
            PlanNodeEnum::Traverse(node) => visitor.visit_traverse(node),
            PlanNodeEnum::AppendVertices(node) => visitor.visit_append_vertices(node),
            PlanNodeEnum::Filter(node) => visitor.visit_filter(node),
            PlanNodeEnum::Aggregate(node) => visitor.visit_aggregate(node),
            PlanNodeEnum::Argument(node) => visitor.visit_argument(node),
            PlanNodeEnum::Loop(node) => visitor.visit_loop(node),
            PlanNodeEnum::PassThrough(node) => visitor.visit_pass_through(node),
            PlanNodeEnum::Select(node) => visitor.visit_select(node),
            PlanNodeEnum::DataCollect(node) => visitor.visit_data_collect(node),
            PlanNodeEnum::Dedup(node) => visitor.visit_dedup(node),
            PlanNodeEnum::PatternApply(node) => visitor.visit_pattern_apply(node),
            PlanNodeEnum::RollUpApply(node) => visitor.visit_roll_up_apply(node),
            PlanNodeEnum::Union(node) => visitor.visit_union(node),
            PlanNodeEnum::Unwind(node) => visitor.visit_unwind(node),
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
            PlanNodeEnum::Filter(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Filter(cloned)
            }
            PlanNodeEnum::Aggregate(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Aggregate(cloned)
            }
            PlanNodeEnum::Argument(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Argument(cloned)
            }
            PlanNodeEnum::Loop(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Loop(cloned)
            }
            PlanNodeEnum::PassThrough(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::PassThrough(cloned)
            }
            PlanNodeEnum::Select(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Select(cloned)
            }
            PlanNodeEnum::DataCollect(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::DataCollect(cloned)
            }
            PlanNodeEnum::Dedup(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Dedup(cloned)
            }
            PlanNodeEnum::PatternApply(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::PatternApply(cloned)
            }
            PlanNodeEnum::RollUpApply(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::RollUpApply(cloned)
            }
            PlanNodeEnum::Union(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Union(cloned)
            }
            PlanNodeEnum::Unwind(node) => {
                let mut cloned = node.clone();
                PlanNodeEnum::Unwind(cloned)
            }
        }
    }

    /// 判断节点是否是查询节点
    pub fn is_query_node(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::GetVertices
                | PlanNodeEnum::GetEdges
                | PlanNodeEnum::GetNeighbors
                | PlanNodeEnum::Expand
                | PlanNodeEnum::ExpandAll
                | PlanNodeEnum::Traverse
                | PlanNodeEnum::AppendVertices
                | PlanNodeEnum::ScanVertices
                | PlanNodeEnum::ScanEdges
        )
    }

    /// 判断节点是否是数据处理节点
    pub fn is_data_processing_node(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Filter
                | PlanNodeEnum::Union
                | PlanNodeEnum::Project
                | PlanNodeEnum::Unwind
                | PlanNodeEnum::Sort
                | PlanNodeEnum::TopN
                | PlanNodeEnum::Limit
                | PlanNodeEnum::Aggregate
                | PlanNodeEnum::Dedup
                | PlanNodeEnum::DataCollect
                | PlanNodeEnum::InnerJoin
                | PlanNodeEnum::LeftJoin
                | PlanNodeEnum::CrossJoin
                | PlanNodeEnum::RollUpApply
                | PlanNodeEnum::PatternApply
                | PlanNodeEnum::Argument
        )
    }

    /// 判断节点是否是控制流节点
    pub fn is_control_flow_node(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Select
                | PlanNodeEnum::Loop
                | PlanNodeEnum::PassThrough
                | PlanNodeEnum::Start
        )
    }
}

impl fmt::Display for PlanNodeEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name(), self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_node_enum_creation() {
        let start_node = StartNode::new();
        let enum_node = PlanNodeEnum::Start(start_node);

        assert_eq!(enum_node.name(), "Start");
        assert_eq!(enum_node.id(), -1);
    }

    #[test]
    fn test_plan_node_enum_display() {
        let start_node = StartNode::new();
        let enum_node = PlanNodeEnum::Start(start_node);

        assert_eq!(format!("{}", enum_node), "Start(-1)");
    }

    #[test]
    fn test_zero_cost_type_check() {
        let start_node = StartNode::new();
        let enum_node = PlanNodeEnum::Start(start_node);

        // 零成本类型检查
        assert!(enum_node.is_start());
        assert!(!enum_node.is_project());
        assert!(!enum_node.is_filter());
        assert!(!enum_node.is_sort());
        assert!(!enum_node.is_limit());
    }

    #[test]
    fn test_zero_cost_type_conversion() {
        let start_node = StartNode::new();
        let enum_node = PlanNodeEnum::Start(start_node);

        // 零成本类型转换
        let start_ref = enum_node.as_start();
        assert!(start_ref.is_some());

        let project_ref = enum_node.as_project();
        assert!(project_ref.is_none());

        let filter_ref = enum_node.as_filter();
        assert!(filter_ref.is_none());
    }

    #[test]
    fn test_zero_cost_visitor_pattern() {
        use super::*;

        struct CostCalculator {
            total_cost: f64,
        }

        impl PlanNodeVisitor for CostCalculator {
            type Result = f64;

            fn visit_start(&mut self, node: &StartNode) -> Self::Result {
                self.total_cost += node.cost();
                self.total_cost
            }

            fn visit_project(&mut self, node: &ProjectNode) -> Self::Result {
                self.total_cost += node.cost();
                self.total_cost
            }

            fn visit_filter(&mut self, node: &FilterNode) -> Self::Result {
                self.total_cost += node.cost();
                self.total_cost
            }

            fn visit_sort(&mut self, node: &SortNode) -> Self::Result {
                self.total_cost += node.cost();
                self.total_cost
            }

            fn visit_limit(&mut self, node: &LimitNode) -> Self::Result {
                self.total_cost += node.cost();
                self.total_cost
            }

            // 为其他节点类型提供默认实现
            fn visit_topn(&mut self, node: &TopNNode) -> Self::Result {
                self.total_cost
            }
            fn visit_inner_join(&mut self, node: &InnerJoinNode) -> Self::Result {
                self.total_cost
            }
            fn visit_left_join(&mut self, node: &LeftJoinNode) -> Self::Result {
                self.total_cost
            }
            fn visit_cross_join(&mut self, node: &CrossJoinNode) -> Self::Result {
                self.total_cost
            }
            fn visit_get_vertices(&mut self, node: &GetVerticesNode) -> Self::Result {
                self.total_cost
            }
            fn visit_get_edges(&mut self, node: &GetEdgesNode) -> Self::Result {
                self.total_cost
            }
            fn visit_get_neighbors(&mut self, node: &GetNeighborsNode) -> Self::Result {
                self.total_cost
            }
            fn visit_scan_vertices(&mut self, node: &ScanVerticesNode) -> Self::Result {
                self.total_cost
            }
            fn visit_scan_edges(&mut self, node: &ScanEdgesNode) -> Self::Result {
                self.total_cost
            }
            fn visit_expand(&mut self, node: &ExpandNode) -> Self::Result {
                self.total_cost
            }
            fn visit_expand_all(&mut self, node: &ExpandAllNode) -> Self::Result {
                self.total_cost
            }
            fn visit_traverse(&mut self, node: &TraverseNode) -> Self::Result {
                self.total_cost
            }
            fn visit_append_vertices(&mut self, node: &AppendVerticesNode) -> Self::Result {
                self.total_cost
            }
            fn visit_aggregate(&mut self, node: &AggregateNode) -> Self::Result {
                self.total_cost
            }
            fn visit_argument(&mut self, node: &ArgumentNode) -> Self::Result {
                self.total_cost
            }
            fn visit_loop(&mut self, node: &LoopNode) -> Self::Result {
                self.total_cost
            }
            fn visit_pass_through(&mut self, node: &PassThroughNode) -> Self::Result {
                self.total_cost
            }
            fn visit_select(&mut self, node: &SelectNode) -> Self::Result {
                self.total_cost
            }
            fn visit_data_collect(&mut self, node: &DataCollectNode) -> Self::Result {
                self.total_cost
            }
            fn visit_dedup(&mut self, node: &DedupNode) -> Self::Result {
                self.total_cost
            }
            fn visit_pattern_apply(&mut self, node: &PatternApplyNode) -> Self::Result {
                self.total_cost
            }
            fn visit_roll_up_apply(&mut self, node: &RollUpApplyNode) -> Self::Result {
                self.total_cost
            }
            fn visit_union(&mut self, node: &UnionNode) -> Self::Result {
                self.total_cost
            }
            fn visit_unwind(&mut self, node: &UnwindNode) -> Self::Result {
                self.total_cost
            }
        }

        let start_node = StartNode::new();
        let enum_node = PlanNodeEnum::Start(start_node);

        let mut calculator = CostCalculator { total_cost: 0.0 };
        let cost = enum_node.accept(&mut calculator);

        assert!(cost > 0.0);
    }
}
