//! 计划节点类型枚举定义
//! 
//! 定义了执行计划中所有可能的操作节点类型，包括：
//! - 查询节点：获取顶点、边、邻接信息等
//! - 数据处理节点：过滤、投影、聚合等
//! - 控制流节点：分支、循环等
//! - 模式管理节点：创建、修改、删除模式元素
//! - 索引管理节点：创建、删除索引
//! - 用户管理节点：创建、删除用户，分配角色等
//! - 其他维护节点：快照、配置管理等

use std::fmt;

/// 计划节点类型枚举，表示执行计划中的各种操作
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlanNodeKind {
    // 查询节点
    GetNeighbors,
    GetVertices,
    GetEdges,
    Expand,
    ExpandAll,
    Traverse,
    AppendVertices,
    ShortestPath,
    IndexScan,
    FulltextIndexScan,
    ScanVertices,
    ScanEdges,

    // 数据处理节点
    Filter,
    Union,
    UnionAllVersionVar,
    Intersect,
    Minus,
    Project,
    Unwind,
    Sort,
    TopN,
    Limit,
    Sample,
    Aggregate,
    Dedup,
    Assign,
    BFSShortest,
    MultiShortestPath,
    AllPaths,
    CartesianProduct,
    Subgraph,
    DataCollect,
    InnerJoin,
    HashJoin,
    HashLeftJoin,
    HashInnerJoin,
    CrossJoin,
    RollUpApply,
    PatternApply,
    Argument,

    // 控制流节点
    Select,
    Loop,
    PassThrough,
    Start,

    // 模式相关节点
    CreateSpace,
    CreateTag,
    CreateEdge,
    DescSpace,
    ShowCreateSpace,
    DescTag,
    DescEdge,
    AlterTag,
    AlterEdge,
    ShowSpaces,
    SwitchSpace,
    ShowTags,
    ShowEdges,
    ShowCreateTag,
    ShowCreateEdge,
    DropSpace,
    ClearSpace,
    DropTag,
    DropEdge,
    AlterSpace,

    // 索引相关节点
    CreateTagIndex,
    CreateEdgeIndex,
    CreateFTIndex,
    DropFTIndex,
    DropTagIndex,
    DropEdgeIndex,
    DescTagIndex,
    DescEdgeIndex,
    ShowCreateTagIndex,
    ShowCreateEdgeIndex,
    ShowTagIndexes,
    ShowEdgeIndexes,
    ShowTagIndexStatus,
    ShowEdgeIndexStatus,
    InsertVertices,
    InsertEdges,
    SubmitJob,
    ShowHosts,

    // 用户相关节点
    CreateUser,
    DropUser,
    UpdateUser,
    GrantRole,
    RevokeRole,
    ChangePassword,
    ListUserRoles,
    ListUsers,
    ListRoles,
    DescribeUser,

    // 快照节点
    CreateSnapshot,
    DropSnapshot,
    ShowSnapshots,

    // 更新/删除节点
    DeleteVertices,
    DeleteEdges,
    UpdateVertex,
    DeleteTags,
    UpdateEdge,

    // 显示节点
    ShowParts,
    ShowCharset,
    ShowCollation,
    ShowStats,
    ShowConfigs,
    SetConfig,
    GetConfig,
    ShowMetaLeader,

    // 区域相关节点
    ShowZones,
    MergeZone,
    RenameZone,
    DropZone,
    DivideZone,
    AddHosts,
    DropHosts,
    DescribeZone,
    AddHostsIntoZone,

    // 监听器相关节点
    AddListener,
    RemoveListener,
    ShowListener,

    // 服务相关节点
    ShowServiceClients,
    ShowFTIndexes,
    SignInService,
    SignOutService,
    ShowSessions,
    UpdateSession,
    KillSession,

    ShowQueries,
    KillQuery,

    // 未知节点类型的占位符
    Unknown,
}

impl PlanNodeKind {
    /// 获取节点类型的名称
    pub fn name(&self) -> &'static str {
        match self {
            // 查询节点
            PlanNodeKind::GetNeighbors => "GetNeighbors",
            PlanNodeKind::GetVertices => "GetVertices",
            PlanNodeKind::GetEdges => "GetEdges",
            PlanNodeKind::Expand => "Expand",
            PlanNodeKind::ExpandAll => "ExpandAll",
            PlanNodeKind::Traverse => "Traverse",
            PlanNodeKind::AppendVertices => "AppendVertices",
            PlanNodeKind::ShortestPath => "ShortestPath",
            PlanNodeKind::IndexScan => "IndexScan",
            PlanNodeKind::FulltextIndexScan => "FulltextIndexScan",
            PlanNodeKind::ScanVertices => "ScanVertices",
            PlanNodeKind::ScanEdges => "ScanEdges",

            // 数据处理节点
            PlanNodeKind::Filter => "Filter",
            PlanNodeKind::Union => "Union",
            PlanNodeKind::UnionAllVersionVar => "UnionAllVersionVar",
            PlanNodeKind::Intersect => "Intersect",
            PlanNodeKind::Minus => "Minus",
            PlanNodeKind::Project => "Project",
            PlanNodeKind::Unwind => "Unwind",
            PlanNodeKind::Sort => "Sort",
            PlanNodeKind::TopN => "TopN",
            PlanNodeKind::Limit => "Limit",
            PlanNodeKind::Sample => "Sample",
            PlanNodeKind::Aggregate => "Aggregate",
            PlanNodeKind::Dedup => "Dedup",
            PlanNodeKind::Assign => "Assign",
            PlanNodeKind::BFSShortest => "BFSShortest",
            PlanNodeKind::MultiShortestPath => "MultiShortestPath",
            PlanNodeKind::AllPaths => "AllPaths",
            PlanNodeKind::CartesianProduct => "CartesianProduct",
            PlanNodeKind::Subgraph => "Subgraph",
            PlanNodeKind::DataCollect => "DataCollect",
            PlanNodeKind::InnerJoin => "InnerJoin",
            PlanNodeKind::HashJoin => "HashJoin",
            PlanNodeKind::HashLeftJoin => "HashLeftJoin",
            PlanNodeKind::HashInnerJoin => "HashInnerJoin",
            PlanNodeKind::CrossJoin => "CrossJoin",
            PlanNodeKind::RollUpApply => "RollUpApply",
            PlanNodeKind::PatternApply => "PatternApply",
            PlanNodeKind::Argument => "Argument",

            // 控制流节点
            PlanNodeKind::Select => "Select",
            PlanNodeKind::Loop => "Loop",
            PlanNodeKind::PassThrough => "PassThrough",
            PlanNodeKind::Start => "Start",

            // 模式相关节点
            PlanNodeKind::CreateSpace => "CreateSpace",
            PlanNodeKind::CreateTag => "CreateTag",
            PlanNodeKind::CreateEdge => "CreateEdge",
            PlanNodeKind::DescSpace => "DescSpace",
            PlanNodeKind::ShowCreateSpace => "ShowCreateSpace",
            PlanNodeKind::DescTag => "DescTag",
            PlanNodeKind::DescEdge => "DescEdge",
            PlanNodeKind::AlterTag => "AlterTag",
            PlanNodeKind::AlterEdge => "AlterEdge",
            PlanNodeKind::ShowSpaces => "ShowSpaces",
            PlanNodeKind::SwitchSpace => "SwitchSpace",
            PlanNodeKind::ShowTags => "ShowTags",
            PlanNodeKind::ShowEdges => "ShowEdges",
            PlanNodeKind::ShowCreateTag => "ShowCreateTag",
            PlanNodeKind::ShowCreateEdge => "ShowCreateEdge",
            PlanNodeKind::DropSpace => "DropSpace",
            PlanNodeKind::ClearSpace => "ClearSpace",
            PlanNodeKind::DropTag => "DropTag",
            PlanNodeKind::DropEdge => "DropEdge",
            PlanNodeKind::AlterSpace => "AlterSpace",

            // 索引相关节点
            PlanNodeKind::CreateTagIndex => "CreateTagIndex",
            PlanNodeKind::CreateEdgeIndex => "CreateEdgeIndex",
            PlanNodeKind::CreateFTIndex => "CreateFTIndex",
            PlanNodeKind::DropFTIndex => "DropFTIndex",
            PlanNodeKind::DropTagIndex => "DropTagIndex",
            PlanNodeKind::DropEdgeIndex => "DropEdgeIndex",
            PlanNodeKind::DescTagIndex => "DescTagIndex",
            PlanNodeKind::DescEdgeIndex => "DescEdgeIndex",
            PlanNodeKind::ShowCreateTagIndex => "ShowCreateTagIndex",
            PlanNodeKind::ShowCreateEdgeIndex => "ShowCreateEdgeIndex",
            PlanNodeKind::ShowTagIndexes => "ShowTagIndexes",
            PlanNodeKind::ShowEdgeIndexes => "ShowEdgeIndexes",
            PlanNodeKind::ShowTagIndexStatus => "ShowTagIndexStatus",
            PlanNodeKind::ShowEdgeIndexStatus => "ShowEdgeIndexStatus",
            PlanNodeKind::InsertVertices => "InsertVertices",
            PlanNodeKind::InsertEdges => "InsertEdges",
            PlanNodeKind::SubmitJob => "SubmitJob",
            PlanNodeKind::ShowHosts => "ShowHosts",

            // 用户相关节点
            PlanNodeKind::CreateUser => "CreateUser",
            PlanNodeKind::DropUser => "DropUser",
            PlanNodeKind::UpdateUser => "UpdateUser",
            PlanNodeKind::GrantRole => "GrantRole",
            PlanNodeKind::RevokeRole => "RevokeRole",
            PlanNodeKind::ChangePassword => "ChangePassword",
            PlanNodeKind::ListUserRoles => "ListUserRoles",
            PlanNodeKind::ListUsers => "ListUsers",
            PlanNodeKind::ListRoles => "ListRoles",
            PlanNodeKind::DescribeUser => "DescribeUser",

            // 快照节点
            PlanNodeKind::CreateSnapshot => "CreateSnapshot",
            PlanNodeKind::DropSnapshot => "DropSnapshot",
            PlanNodeKind::ShowSnapshots => "ShowSnapshots",

            // 更新/删除节点
            PlanNodeKind::DeleteVertices => "DeleteVertices",
            PlanNodeKind::DeleteEdges => "DeleteEdges",
            PlanNodeKind::UpdateVertex => "UpdateVertex",
            PlanNodeKind::DeleteTags => "DeleteTags",
            PlanNodeKind::UpdateEdge => "UpdateEdge",

            // 显示节点
            PlanNodeKind::ShowParts => "ShowParts",
            PlanNodeKind::ShowCharset => "ShowCharset",
            PlanNodeKind::ShowCollation => "ShowCollation",
            PlanNodeKind::ShowStats => "ShowStats",
            PlanNodeKind::ShowConfigs => "ShowConfigs",
            PlanNodeKind::SetConfig => "SetConfig",
            PlanNodeKind::GetConfig => "GetConfig",
            PlanNodeKind::ShowMetaLeader => "ShowMetaLeader",

            // 区域相关节点
            PlanNodeKind::ShowZones => "ShowZones",
            PlanNodeKind::MergeZone => "MergeZone",
            PlanNodeKind::RenameZone => "RenameZone",
            PlanNodeKind::DropZone => "DropZone",
            PlanNodeKind::DivideZone => "DivideZone",
            PlanNodeKind::AddHosts => "AddHosts",
            PlanNodeKind::DropHosts => "DropHosts",
            PlanNodeKind::DescribeZone => "DescribeZone",
            PlanNodeKind::AddHostsIntoZone => "AddHostsIntoZone",

            // 监听器相关节点
            PlanNodeKind::AddListener => "AddListener",
            PlanNodeKind::RemoveListener => "RemoveListener",
            PlanNodeKind::ShowListener => "ShowListener",

            // 服务相关节点
            PlanNodeKind::ShowServiceClients => "ShowServiceClients",
            PlanNodeKind::ShowFTIndexes => "ShowFTIndexes",
            PlanNodeKind::SignInService => "SignInService",
            PlanNodeKind::SignOutService => "SignOutService",
            PlanNodeKind::ShowSessions => "ShowSessions",
            PlanNodeKind::UpdateSession => "UpdateSession",
            PlanNodeKind::KillSession => "KillSession",
            PlanNodeKind::ShowQueries => "ShowQueries",
            PlanNodeKind::KillQuery => "KillQuery",

            // 未知节点类型
            PlanNodeKind::Unknown => "Unknown",
        }
    }

    /// 判断节点是否是查询节点
    pub fn is_query_node(&self) -> bool {
        matches!(
            self,
            PlanNodeKind::GetNeighbors
                | PlanNodeKind::GetVertices
                | PlanNodeKind::GetEdges
                | PlanNodeKind::Expand
                | PlanNodeKind::ExpandAll
                | PlanNodeKind::Traverse
                | PlanNodeKind::AppendVertices
                | PlanNodeKind::ShortestPath
                | PlanNodeKind::IndexScan
                | PlanNodeKind::FulltextIndexScan
                | PlanNodeKind::ScanVertices
                | PlanNodeKind::ScanEdges
        )
    }

    /// 判断节点是否是数据处理节点
    pub fn is_data_processing_node(&self) -> bool {
        matches!(
            self,
            PlanNodeKind::Filter
                | PlanNodeKind::Union
                | PlanNodeKind::UnionAllVersionVar
                | PlanNodeKind::Intersect
                | PlanNodeKind::Minus
                | PlanNodeKind::Project
                | PlanNodeKind::Unwind
                | PlanNodeKind::Sort
                | PlanNodeKind::TopN
                | PlanNodeKind::Limit
                | PlanNodeKind::Sample
                | PlanNodeKind::Aggregate
                | PlanNodeKind::Dedup
                | PlanNodeKind::Assign
                | PlanNodeKind::BFSShortest
                | PlanNodeKind::MultiShortestPath
                | PlanNodeKind::AllPaths
                | PlanNodeKind::CartesianProduct
                | PlanNodeKind::Subgraph
                | PlanNodeKind::DataCollect
                | PlanNodeKind::InnerJoin
                | PlanNodeKind::HashJoin
                | PlanNodeKind::HashLeftJoin
                | PlanNodeKind::HashInnerJoin
                | PlanNodeKind::CrossJoin
                | PlanNodeKind::RollUpApply
                | PlanNodeKind::PatternApply
                | PlanNodeKind::Argument
        )
    }

    /// 判断节点是否是控制流节点
    pub fn is_control_flow_node(&self) -> bool {
        matches!(
            self,
            PlanNodeKind::Select
                | PlanNodeKind::Loop
                | PlanNodeKind::PassThrough
                | PlanNodeKind::Start
        )
    }

    /// 判断节点是否是管理类节点
    pub fn is_admin_node(&self) -> bool {
        matches!(
            self,
            PlanNodeKind::CreateSpace
                | PlanNodeKind::CreateTag
                | PlanNodeKind::CreateEdge
                | PlanNodeKind::DescSpace
                | PlanNodeKind::ShowCreateSpace
                | PlanNodeKind::DescTag
                | PlanNodeKind::DescEdge
                | PlanNodeKind::AlterTag
                | PlanNodeKind::AlterEdge
                | PlanNodeKind::ShowSpaces
                | PlanNodeKind::SwitchSpace
                | PlanNodeKind::ShowTags
                | PlanNodeKind::ShowEdges
                | PlanNodeKind::ShowCreateTag
                | PlanNodeKind::ShowCreateEdge
                | PlanNodeKind::DropSpace
                | PlanNodeKind::ClearSpace
                | PlanNodeKind::DropTag
                | PlanNodeKind::DropEdge
                | PlanNodeKind::AlterSpace
                | PlanNodeKind::CreateTagIndex
                | PlanNodeKind::CreateEdgeIndex
                | PlanNodeKind::CreateFTIndex
                | PlanNodeKind::DropFTIndex
                | PlanNodeKind::DropTagIndex
                | PlanNodeKind::DropEdgeIndex
                | PlanNodeKind::DescTagIndex
                | PlanNodeKind::DescEdgeIndex
                | PlanNodeKind::ShowCreateTagIndex
                | PlanNodeKind::ShowCreateEdgeIndex
                | PlanNodeKind::ShowTagIndexes
                | PlanNodeKind::ShowEdgeIndexes
                | PlanNodeKind::ShowTagIndexStatus
                | PlanNodeKind::ShowEdgeIndexStatus
                | PlanNodeKind::CreateUser
                | PlanNodeKind::DropUser
                | PlanNodeKind::UpdateUser
                | PlanNodeKind::GrantRole
                | PlanNodeKind::RevokeRole
                | PlanNodeKind::ChangePassword
                | PlanNodeKind::ListUserRoles
                | PlanNodeKind::ListUsers
                | PlanNodeKind::ListRoles
                | PlanNodeKind::DescribeUser
                | PlanNodeKind::CreateSnapshot
                | PlanNodeKind::DropSnapshot
                | PlanNodeKind::ShowSnapshots
        )
    }

    /// 判断节点是否是修改类节点
    pub fn is_mutate_node(&self) -> bool {
        matches!(
            self,
            PlanNodeKind::InsertVertices
                | PlanNodeKind::InsertEdges
                | PlanNodeKind::DeleteVertices
                | PlanNodeKind::DeleteEdges
                | PlanNodeKind::UpdateVertex
                | PlanNodeKind::DeleteTags
                | PlanNodeKind::UpdateEdge
        )
    }
}

impl fmt::Display for PlanNodeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_kind_name() {
        assert_eq!(PlanNodeKind::GetVertices.name(), "GetVertices");
        assert_eq!(PlanNodeKind::Filter.name(), "Filter");
        assert_eq!(PlanNodeKind::Start.name(), "Start");
    }

    #[test]
    fn test_node_kind_display() {
        assert_eq!(format!("{}", PlanNodeKind::GetVertices), "GetVertices");
    }

    #[test]
    fn test_is_query_node() {
        assert!(PlanNodeKind::GetVertices.is_query_node());
        assert!(PlanNodeKind::Expand.is_query_node());
        assert!(!PlanNodeKind::Filter.is_query_node());
    }

    #[test]
    fn test_is_data_processing_node() {
        assert!(PlanNodeKind::Filter.is_data_processing_node());
        assert!(PlanNodeKind::Project.is_data_processing_node());
        assert!(!PlanNodeKind::GetVertices.is_data_processing_node());
    }

    #[test]
    fn test_is_control_flow_node() {
        assert!(PlanNodeKind::Start.is_control_flow_node());
        assert!(PlanNodeKind::Select.is_control_flow_node());
        assert!(!PlanNodeKind::Filter.is_control_flow_node());
    }

    #[test]
    fn test_is_admin_node() {
        assert!(PlanNodeKind::CreateSpace.is_admin_node());
        assert!(PlanNodeKind::DropTag.is_admin_node());
        assert!(!PlanNodeKind::Filter.is_admin_node());
    }

    #[test]
    fn test_is_mutate_node() {
        assert!(PlanNodeKind::InsertVertices.is_mutate_node());
        assert!(PlanNodeKind::DeleteEdges.is_mutate_node());
        assert!(!PlanNodeKind::Filter.is_mutate_node());
    }

    #[test]
    fn test_hash_and_eq() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(PlanNodeKind::GetVertices);
        set.insert(PlanNodeKind::Filter);
        
        assert!(set.contains(&PlanNodeKind::GetVertices));
        assert_eq!(set.len(), 2);
    }
}