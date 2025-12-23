//! PlanNode 枚举定义
//!
//! 简化的PlanNodeEnum定义，只包含枚举和基本方法

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

// 导入管理节点类型
use crate::query::planner::plan::management::admin::config_ops::{
    GetConfig, SetConfig, ShowConfigs,
};
use crate::query::planner::plan::management::admin::host_ops::{
    AddHosts, DropHosts, ShowHosts, ShowHostsStatus,
};
use crate::query::planner::plan::management::admin::index_ops::{
    CreateIndex, DescIndex, DropIndex, ShowIndexes,
};
use crate::query::planner::plan::management::admin::system_ops::{
    CreateSnapshot, DropSnapshot, ShowSnapshots, SubmitJob,
};
use crate::query::planner::plan::management::ddl::edge_ops::{
    CreateEdge, DropEdge, ShowCreateEdge, ShowEdges,
};
use crate::query::planner::plan::management::ddl::space_ops::{
    AlterSpace, ClearSpace, CreateSpace, DescSpace, DropSpace, ShowCreateSpace, ShowSpaces,
    SwitchSpace,
};
use crate::query::planner::plan::management::ddl::tag_ops::{
    CreateTag, DescTag, DropTag, ShowCreateTag, ShowTags,
};
use crate::query::planner::plan::management::dml::data_constructors::{
    NewEdge, NewProp, NewTag, NewVertex,
};
use crate::query::planner::plan::management::dml::delete_ops::{
    DeleteEdges, DeleteTags, DeleteVertices,
};
use crate::query::planner::plan::management::dml::insert_ops::{InsertEdges, InsertVertices};
use crate::query::planner::plan::management::dml::update_ops::{UpdateEdge, UpdateVertex};
use crate::query::planner::plan::management::security::role_ops::{
    CreateRole, DropRole, GrantRole, RevokeRole, ShowRoles,
};
use crate::query::planner::plan::management::security::user_ops::{
    ChangePassword, CreateUser, DescribeUser, DropUser, ListUserRoles, ListUsers, UpdateUser,
};

/// PlanNode 枚举，包含所有可能的节点类型
///
/// 这个枚举避免了动态分发的性能开销
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
    /// 哈希内连接节点
    HashInnerJoin(InnerJoinNode),
    /// 哈希左连接节点
    HashLeftJoin(LeftJoinNode),
    /// 笛卡尔积节点
    CartesianProduct(CrossJoinNode),
    /// 索引扫描节点
    IndexScan(crate::query::planner::plan::algorithms::IndexScan),
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

    // 管理节点类型
    /// 用户管理节点
    CreateUser(CreateUser),
    DropUser(DropUser),
    UpdateUser(UpdateUser),
    ChangePassword(ChangePassword),
    ListUsers(ListUsers),
    ListUserRoles(ListUserRoles),
    DescribeUser(DescribeUser),

    /// 角色管理节点
    CreateRole(CreateRole),
    DropRole(DropRole),
    GrantRole(GrantRole),
    RevokeRole(RevokeRole),
    ShowRoles(ShowRoles),

    /// DML操作节点
    UpdateVertex(UpdateVertex),
    UpdateEdge(UpdateEdge),
    InsertVertices(InsertVertices),
    InsertEdges(InsertEdges),
    DeleteVertices(DeleteVertices),
    DeleteTags(DeleteTags),
    DeleteEdges(DeleteEdges),
    NewVertex(NewVertex),
    NewTag(NewTag),
    NewProp(NewProp),
    NewEdge(NewEdge),

    /// DDL操作节点 - 标签
    CreateTag(CreateTag),
    DescTag(DescTag),
    DropTag(DropTag),
    ShowTags(ShowTags),
    ShowCreateTag(ShowCreateTag),

    /// DDL操作节点 - 空间
    CreateSpace(CreateSpace),
    DescSpace(DescSpace),
    ShowCreateSpace(ShowCreateSpace),
    ShowSpaces(ShowSpaces),
    SwitchSpace(SwitchSpace),
    DropSpace(DropSpace),
    ClearSpace(ClearSpace),
    AlterSpace(AlterSpace),

    /// DDL操作节点 - 边
    CreateEdge(CreateEdge),
    DropEdge(DropEdge),
    ShowEdges(ShowEdges),
    ShowCreateEdge(ShowCreateEdge),

    /// 系统管理节点
    SubmitJob(SubmitJob),
    CreateSnapshot(CreateSnapshot),
    DropSnapshot(DropSnapshot),
    ShowSnapshots(ShowSnapshots),

    /// 索引管理节点
    CreateIndex(CreateIndex),
    DropIndex(DropIndex),
    ShowIndexes(ShowIndexes),
    DescIndex(DescIndex),

    /// 主机管理节点
    AddHosts(AddHosts),
    DropHosts(DropHosts),
    ShowHosts(ShowHosts),
    ShowHostsStatus(ShowHostsStatus),

    /// 配置管理节点
    ShowConfigs(ShowConfigs),
    SetConfig(SetConfig),
    GetConfig(GetConfig),
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

    pub fn is_dedup(&self) -> bool {
        matches!(self, PlanNodeEnum::Dedup(_))
    }

    pub fn is_get_vertices(&self) -> bool {
        matches!(self, PlanNodeEnum::GetVertices(_))
    }

    pub fn is_get_edges(&self) -> bool {
        matches!(self, PlanNodeEnum::GetEdges(_))
    }

    pub fn is_get_neighbors(&self) -> bool {
        matches!(self, PlanNodeEnum::GetNeighbors(_))
    }

    pub fn is_scan_vertices(&self) -> bool {
        matches!(self, PlanNodeEnum::ScanVertices(_))
    }

    pub fn is_scan_edges(&self) -> bool {
        matches!(self, PlanNodeEnum::ScanEdges(_))
    }

    pub fn is_index_scan(&self) -> bool {
        matches!(self, PlanNodeEnum::IndexScan(_))
    }

    pub fn is_append_vertices(&self) -> bool {
        matches!(self, PlanNodeEnum::AppendVertices(_))
    }

    pub fn is_inner_join(&self) -> bool {
        matches!(self, PlanNodeEnum::InnerJoin(_))
    }

    pub fn is_left_join(&self) -> bool {
        matches!(self, PlanNodeEnum::LeftJoin(_))
    }

    pub fn is_cross_join(&self) -> bool {
        matches!(self, PlanNodeEnum::CrossJoin(_))
    }

    pub fn is_traverse(&self) -> bool {
        matches!(self, PlanNodeEnum::Traverse(_))
    }

    pub fn is_expand(&self) -> bool {
        matches!(self, PlanNodeEnum::Expand(_))
    }

    pub fn is_hash_inner_join(&self) -> bool {
        matches!(self, PlanNodeEnum::HashInnerJoin(_))
    }

    pub fn is_hash_left_join(&self) -> bool {
        matches!(self, PlanNodeEnum::HashLeftJoin(_))
    }

    pub fn is_cartesian_product(&self) -> bool {
        matches!(self, PlanNodeEnum::CartesianProduct(_))
    }

    pub fn type_name(&self) -> &'static str {
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
            PlanNodeEnum::HashInnerJoin(_) => "HashInnerJoin",
            PlanNodeEnum::HashLeftJoin(_) => "HashLeftJoin",
            PlanNodeEnum::CartesianProduct(_) => "CartesianProduct",
            PlanNodeEnum::IndexScan(_) => "IndexScan",
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
            // 管理节点类型
            PlanNodeEnum::CreateUser(_) => "CreateUser",
            PlanNodeEnum::DropUser(_) => "DropUser",
            PlanNodeEnum::UpdateUser(_) => "UpdateUser",
            PlanNodeEnum::ChangePassword(_) => "ChangePassword",
            PlanNodeEnum::ListUsers(_) => "ListUsers",
            PlanNodeEnum::ListUserRoles(_) => "ListUserRoles",
            PlanNodeEnum::DescribeUser(_) => "DescribeUser",
            PlanNodeEnum::CreateRole(_) => "CreateRole",
            PlanNodeEnum::DropRole(_) => "DropRole",
            PlanNodeEnum::GrantRole(_) => "GrantRole",
            PlanNodeEnum::RevokeRole(_) => "RevokeRole",
            PlanNodeEnum::ShowRoles(_) => "ShowRoles",
            PlanNodeEnum::UpdateVertex(_) => "UpdateVertex",
            PlanNodeEnum::UpdateEdge(_) => "UpdateEdge",
            PlanNodeEnum::InsertVertices(_) => "InsertVertices",
            PlanNodeEnum::InsertEdges(_) => "InsertEdges",
            PlanNodeEnum::DeleteVertices(_) => "DeleteVertices",
            PlanNodeEnum::DeleteTags(_) => "DeleteTags",
            PlanNodeEnum::DeleteEdges(_) => "DeleteEdges",
            PlanNodeEnum::NewVertex(_) => "NewVertex",
            PlanNodeEnum::NewTag(_) => "NewTag",
            PlanNodeEnum::NewProp(_) => "NewProp",
            PlanNodeEnum::NewEdge(_) => "NewEdge",
            PlanNodeEnum::CreateTag(_) => "CreateTag",
            PlanNodeEnum::DescTag(_) => "DescTag",
            PlanNodeEnum::DropTag(_) => "DropTag",
            PlanNodeEnum::ShowTags(_) => "ShowTags",
            PlanNodeEnum::ShowCreateTag(_) => "ShowCreateTag",
            PlanNodeEnum::CreateSpace(_) => "CreateSpace",
            PlanNodeEnum::DescSpace(_) => "DescSpace",
            PlanNodeEnum::ShowCreateSpace(_) => "ShowCreateSpace",
            PlanNodeEnum::ShowSpaces(_) => "ShowSpaces",
            PlanNodeEnum::SwitchSpace(_) => "SwitchSpace",
            PlanNodeEnum::DropSpace(_) => "DropSpace",
            PlanNodeEnum::ClearSpace(_) => "ClearSpace",
            PlanNodeEnum::AlterSpace(_) => "AlterSpace",
            PlanNodeEnum::CreateEdge(_) => "CreateEdge",
            PlanNodeEnum::DropEdge(_) => "DropEdge",
            PlanNodeEnum::ShowEdges(_) => "ShowEdges",
            PlanNodeEnum::ShowCreateEdge(_) => "ShowCreateEdge",
            PlanNodeEnum::SubmitJob(_) => "SubmitJob",
            PlanNodeEnum::CreateSnapshot(_) => "CreateSnapshot",
            PlanNodeEnum::DropSnapshot(_) => "DropSnapshot",
            PlanNodeEnum::ShowSnapshots(_) => "ShowSnapshots",
            PlanNodeEnum::CreateIndex(_) => "CreateIndex",
            PlanNodeEnum::DropIndex(_) => "DropIndex",
            PlanNodeEnum::ShowIndexes(_) => "ShowIndexes",
            PlanNodeEnum::DescIndex(_) => "DescIndex",
            PlanNodeEnum::AddHosts(_) => "AddHosts",
            PlanNodeEnum::DropHosts(_) => "DropHosts",
            PlanNodeEnum::ShowHosts(_) => "ShowHosts",
            PlanNodeEnum::ShowHostsStatus(_) => "ShowHostsStatus",
            PlanNodeEnum::ShowConfigs(_) => "ShowConfigs",
            PlanNodeEnum::SetConfig(_) => "SetConfig",
            PlanNodeEnum::GetConfig(_) => "GetConfig",
        }
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

    pub fn as_get_vertices(&self) -> Option<&GetVerticesNode> {
        match self {
            PlanNodeEnum::GetVertices(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_get_edges(&self) -> Option<&GetEdgesNode> {
        match self {
            PlanNodeEnum::GetEdges(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_get_neighbors(&self) -> Option<&GetNeighborsNode> {
        match self {
            PlanNodeEnum::GetNeighbors(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_scan_vertices(&self) -> Option<&ScanVerticesNode> {
        match self {
            PlanNodeEnum::ScanVertices(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_scan_edges(&self) -> Option<&ScanEdgesNode> {
        match self {
            PlanNodeEnum::ScanEdges(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_index_scan(&self) -> Option<&crate::query::planner::plan::algorithms::IndexScan> {
        match self {
            PlanNodeEnum::IndexScan(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_expand_all(&self) -> Option<&ExpandAllNode> {
        match self {
            PlanNodeEnum::ExpandAll(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_traverse(&self) -> Option<&TraverseNode> {
        match self {
            PlanNodeEnum::Traverse(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_expand(&self) -> Option<&ExpandNode> {
        match self {
            PlanNodeEnum::Expand(node) => Some(node),
            _ => None,
        }
    }

    /// 克隆节点
    pub fn clone_plan_node(&self) -> PlanNodeEnum {
        self.clone()
    }

    /// 判断节点是否是查询节点
    pub fn is_query_node(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::GetVertices(_)
                | PlanNodeEnum::GetEdges(_)
                | PlanNodeEnum::GetNeighbors(_)
                | PlanNodeEnum::Expand(_)
                | PlanNodeEnum::ExpandAll(_)
                | PlanNodeEnum::Traverse(_)
                | PlanNodeEnum::AppendVertices(_)
                | PlanNodeEnum::ScanVertices(_)
                | PlanNodeEnum::ScanEdges(_)
        )
    }

    /// 判断节点是否是数据处理节点
    pub fn is_data_processing_node(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Filter(_)
                | PlanNodeEnum::Union(_)
                | PlanNodeEnum::Project(_)
                | PlanNodeEnum::Unwind(_)
                | PlanNodeEnum::Sort(_)
                | PlanNodeEnum::TopN(_)
                | PlanNodeEnum::Limit(_)
                | PlanNodeEnum::Aggregate(_)
                | PlanNodeEnum::Dedup(_)
                | PlanNodeEnum::DataCollect(_)
                | PlanNodeEnum::InnerJoin(_)
                | PlanNodeEnum::LeftJoin(_)
                | PlanNodeEnum::CrossJoin(_)
                | PlanNodeEnum::RollUpApply(_)
                | PlanNodeEnum::PatternApply(_)
                | PlanNodeEnum::Argument(_)
        )
    }

    /// 判断节点是否是控制流节点
    pub fn is_control_flow_node(&self) -> bool {
        matches!(
            self,
            PlanNodeEnum::Select(_)
                | PlanNodeEnum::Loop(_)
                | PlanNodeEnum::PassThrough(_)
                | PlanNodeEnum::Start(_)
        )
    }
}

impl std::fmt::Display for PlanNodeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}()", self)
    }
}

// 使用操作实现
use super::plan_node_operations::*;

/// PlanNode 访问者trait - 使用泛型避免动态分发
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

impl PlanNodeEnum {
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

            // 管理节点类型 - 暂时使用默认处理
            _ => unimplemented!("管理节点的访问者模式尚未实现"),
        }
    }
}
