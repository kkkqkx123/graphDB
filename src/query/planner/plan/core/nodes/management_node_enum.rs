//! 管理节点枚举定义
//!
//! 包含所有管理节点类型的枚举

// 导入所有管理节点类型
use crate::query::planner::plan::management::admin::config_ops::{
    GetConfig, SetConfig, ShowConfigs,
};
use crate::query::planner::plan::management::admin::host_ops::{
    AddHosts, DropHosts, ShowHosts, ShowHostsStatus,
};
use crate::query::planner::plan::management::admin::index_ops::{
    CreateEdgeIndex, CreateIndex, CreateTagIndex, DescIndex, DropEdgeIndex, DropIndex,
    DropTagIndex, ShowEdgeIndexes, ShowIndexStatus, ShowIndexes, ShowTagIndexes,
};
use crate::query::planner::plan::management::admin::system_ops::{
    CreateSnapshot, DropSnapshot, ShowCharset, ShowCollation, ShowSnapshots, ShowStats, SubmitJob,
};
use crate::query::planner::plan::management::ddl::edge_ops::{
    AlterEdge, CreateEdge, DropEdge, ShowCreateEdge, ShowEdges,
};
use crate::query::planner::plan::management::ddl::space_ops::{
    AlterSpace, ClearSpace, CreateSpace, DescSpace, DropSpace, ShowCreateSpace, ShowSpaces,
    SwitchSpace,
};
use crate::query::planner::plan::management::ddl::tag_ops::{
    AlterTag, CreateTag, DescTag, DropTag, ShowCreateTag, ShowTags,
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

/// 管理节点枚举，包含所有管理操作节点类型
#[derive(Debug, Clone)]
pub enum ManagementNodeEnum {
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
    AlterTag(AlterTag),

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
    AlterEdge(AlterEdge),

    /// 系统管理节点
    SubmitJob(SubmitJob),
    CreateSnapshot(CreateSnapshot),
    DropSnapshot(DropSnapshot),
    ShowSnapshots(ShowSnapshots),
    // ShowMetaLeader(ShowMetaLeader),
    // ShowParts(ShowParts),
    ShowStats(ShowStats),
    ShowCharset(ShowCharset),
    ShowCollation(ShowCollation),

    /// 索引管理节点
    CreateTagIndex(CreateTagIndex),
    CreateEdgeIndex(CreateEdgeIndex),
    DropIndex(DropIndex),
    DropTagIndex(DropTagIndex),
    DropEdgeIndex(DropEdgeIndex),
    ShowIndexes(ShowIndexes),
    ShowTagIndexes(ShowTagIndexes),
    ShowEdgeIndexes(ShowEdgeIndexes),
    ShowIndexStatus(ShowIndexStatus),
    DescIndex(DescIndex),
    CreateIndex(CreateIndex),

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

impl ManagementNodeEnum {
    /// 获取节点类型的名称
    pub fn type_name(&self) -> &'static str {
        match self {
            ManagementNodeEnum::CreateUser(_) => "CreateUser",
            ManagementNodeEnum::DropUser(_) => "DropUser",
            ManagementNodeEnum::UpdateUser(_) => "UpdateUser",
            ManagementNodeEnum::ChangePassword(_) => "ChangePassword",
            ManagementNodeEnum::ListUsers(_) => "ListUsers",
            ManagementNodeEnum::ListUserRoles(_) => "ListUserRoles",
            ManagementNodeEnum::DescribeUser(_) => "DescribeUser",
            ManagementNodeEnum::CreateRole(_) => "CreateRole",
            ManagementNodeEnum::DropRole(_) => "DropRole",
            ManagementNodeEnum::GrantRole(_) => "GrantRole",
            ManagementNodeEnum::RevokeRole(_) => "RevokeRole",
            ManagementNodeEnum::ShowRoles(_) => "ShowRoles",
            ManagementNodeEnum::UpdateVertex(_) => "UpdateVertex",
            ManagementNodeEnum::UpdateEdge(_) => "UpdateEdge",
            ManagementNodeEnum::InsertVertices(_) => "InsertVertices",
            ManagementNodeEnum::InsertEdges(_) => "InsertEdges",
            ManagementNodeEnum::DeleteVertices(_) => "DeleteVertices",
            ManagementNodeEnum::DeleteTags(_) => "DeleteTags",
            ManagementNodeEnum::DeleteEdges(_) => "DeleteEdges",
            ManagementNodeEnum::NewVertex(_) => "NewVertex",
            ManagementNodeEnum::NewTag(_) => "NewTag",
            ManagementNodeEnum::NewProp(_) => "NewProp",
            ManagementNodeEnum::NewEdge(_) => "NewEdge",
            ManagementNodeEnum::CreateTag(_) => "CreateTag",
            ManagementNodeEnum::DescTag(_) => "DescTag",
            ManagementNodeEnum::DropTag(_) => "DropTag",
            ManagementNodeEnum::ShowTags(_) => "ShowTags",
            ManagementNodeEnum::ShowCreateTag(_) => "ShowCreateTag",
            ManagementNodeEnum::AlterTag(_) => "AlterTag",
            ManagementNodeEnum::CreateSpace(_) => "CreateSpace",
            ManagementNodeEnum::DescSpace(_) => "DescSpace",
            ManagementNodeEnum::ShowCreateSpace(_) => "ShowCreateSpace",
            ManagementNodeEnum::ShowSpaces(_) => "ShowSpaces",
            ManagementNodeEnum::SwitchSpace(_) => "SwitchSpace",
            ManagementNodeEnum::DropSpace(_) => "DropSpace",
            ManagementNodeEnum::ClearSpace(_) => "ClearSpace",
            ManagementNodeEnum::AlterSpace(_) => "AlterSpace",
            ManagementNodeEnum::CreateEdge(_) => "CreateEdge",
            ManagementNodeEnum::DropEdge(_) => "DropEdge",
            ManagementNodeEnum::ShowEdges(_) => "ShowEdges",
            ManagementNodeEnum::ShowCreateEdge(_) => "ShowCreateEdge",
            ManagementNodeEnum::AlterEdge(_) => "AlterEdge",
            ManagementNodeEnum::SubmitJob(_) => "SubmitJob",
            ManagementNodeEnum::CreateSnapshot(_) => "CreateSnapshot",
            ManagementNodeEnum::DropSnapshot(_) => "DropSnapshot",
            ManagementNodeEnum::ShowSnapshots(_) => "ShowSnapshots",
            // ManagementNodeEnum::ShowMetaLeader(_) => "ShowMetaLeader",
            // ManagementNodeEnum::ShowParts(_) => "ShowParts",
            ManagementNodeEnum::ShowStats(_) => "ShowStats",
            ManagementNodeEnum::ShowCharset(_) => "ShowCharset",
            ManagementNodeEnum::ShowCollation(_) => "ShowCollation",
            ManagementNodeEnum::CreateTagIndex(_) => "CreateTagIndex",
            ManagementNodeEnum::CreateEdgeIndex(_) => "CreateEdgeIndex",
            ManagementNodeEnum::DropIndex(_) => "DropIndex",
            ManagementNodeEnum::DropTagIndex(_) => "DropTagIndex",
            ManagementNodeEnum::DropEdgeIndex(_) => "DropEdgeIndex",
            ManagementNodeEnum::ShowIndexes(_) => "ShowIndexes",
            ManagementNodeEnum::ShowTagIndexes(_) => "ShowTagIndexes",
            ManagementNodeEnum::ShowEdgeIndexes(_) => "ShowEdgeIndexes",
            ManagementNodeEnum::ShowIndexStatus(_) => "ShowIndexStatus",
            ManagementNodeEnum::DescIndex(_) => "DescIndex",
            ManagementNodeEnum::CreateIndex(_) => "CreateIndex",
            ManagementNodeEnum::AddHosts(_) => "AddHosts",
            ManagementNodeEnum::DropHosts(_) => "DropHosts",
            ManagementNodeEnum::ShowHosts(_) => "ShowHosts",
            ManagementNodeEnum::ShowHostsStatus(_) => "ShowHostsStatus",
            ManagementNodeEnum::ShowConfigs(_) => "ShowConfigs",
            ManagementNodeEnum::SetConfig(_) => "SetConfig",
            ManagementNodeEnum::GetConfig(_) => "GetConfig",
        }
    }
}

impl std::fmt::Display for ManagementNodeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}()", self)
    }
}
