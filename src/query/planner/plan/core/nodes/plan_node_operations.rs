//! PlanNode 操作实现
//!
//! 实现 PlanNodeEnum 的各种操作方法

use super::plan_node_enum::PlanNodeEnum;
use crate::query::context::validate::types::Variable;

impl PlanNodeEnum {
    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        match self {
            // 基础节点类型 - 这些节点实现了 PlanNode trait
            PlanNodeEnum::Start(node) => node.id(),
            PlanNodeEnum::Project(node) => node.id(),
            PlanNodeEnum::Sort(node) => node.id(),
            PlanNodeEnum::Limit(node) => node.id(),
            PlanNodeEnum::TopN(node) => node.id(),
            PlanNodeEnum::InnerJoin(node) => node.id(),
            PlanNodeEnum::LeftJoin(node) => node.id(),
            PlanNodeEnum::CrossJoin(node) => node.id(),
            PlanNodeEnum::HashInnerJoin(node) => node.id(),
            PlanNodeEnum::HashLeftJoin(node) => node.id(),
            PlanNodeEnum::CartesianProduct(node) => node.id(),
            PlanNodeEnum::IndexScan(node) => node.id(),
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

            // 管理节点类型 - 使用默认实现，因为这些节点没有实现 PlanNode trait
            // 使用基于节点类型的哈希值作为默认 ID
            PlanNodeEnum::CreateUser(_) => self.default_node_id("CreateUser"),
            PlanNodeEnum::DropUser(_) => self.default_node_id("DropUser"),
            PlanNodeEnum::UpdateUser(_) => self.default_node_id("UpdateUser"),
            PlanNodeEnum::ChangePassword(_) => self.default_node_id("ChangePassword"),
            PlanNodeEnum::ListUsers(_) => self.default_node_id("ListUsers"),
            PlanNodeEnum::ListUserRoles(_) => self.default_node_id("ListUserRoles"),
            PlanNodeEnum::DescribeUser(_) => self.default_node_id("DescribeUser"),
            PlanNodeEnum::CreateRole(_) => self.default_node_id("CreateRole"),
            PlanNodeEnum::DropRole(_) => self.default_node_id("DropRole"),
            PlanNodeEnum::GrantRole(_) => self.default_node_id("GrantRole"),
            PlanNodeEnum::RevokeRole(_) => self.default_node_id("RevokeRole"),
            PlanNodeEnum::ShowRoles(_) => self.default_node_id("ShowRoles"),
            PlanNodeEnum::UpdateVertex(_) => self.default_node_id("UpdateVertex"),
            PlanNodeEnum::UpdateEdge(_) => self.default_node_id("UpdateEdge"),
            PlanNodeEnum::InsertVertices(_) => self.default_node_id("InsertVertices"),
            PlanNodeEnum::InsertEdges(_) => self.default_node_id("InsertEdges"),
            PlanNodeEnum::DeleteVertices(_) => self.default_node_id("DeleteVertices"),
            PlanNodeEnum::DeleteTags(_) => self.default_node_id("DeleteTags"),
            PlanNodeEnum::DeleteEdges(_) => self.default_node_id("DeleteEdges"),
            PlanNodeEnum::NewVertex(_) => self.default_node_id("NewVertex"),
            PlanNodeEnum::NewTag(_) => self.default_node_id("NewTag"),
            PlanNodeEnum::NewProp(_) => self.default_node_id("NewProp"),
            PlanNodeEnum::NewEdge(_) => self.default_node_id("NewEdge"),
            PlanNodeEnum::CreateTag(_) => self.default_node_id("CreateTag"),
            PlanNodeEnum::DescTag(_) => self.default_node_id("DescTag"),
            PlanNodeEnum::DropTag(_) => self.default_node_id("DropTag"),
            PlanNodeEnum::ShowTags(_) => self.default_node_id("ShowTags"),
            PlanNodeEnum::ShowCreateTag(_) => self.default_node_id("ShowCreateTag"),
            PlanNodeEnum::CreateSpace(_) => self.default_node_id("CreateSpace"),
            PlanNodeEnum::DescSpace(_) => self.default_node_id("DescSpace"),
            PlanNodeEnum::ShowCreateSpace(_) => self.default_node_id("ShowCreateSpace"),
            PlanNodeEnum::ShowSpaces(_) => self.default_node_id("ShowSpaces"),
            PlanNodeEnum::SwitchSpace(_) => self.default_node_id("SwitchSpace"),
            PlanNodeEnum::DropSpace(_) => self.default_node_id("DropSpace"),
            PlanNodeEnum::ClearSpace(_) => self.default_node_id("ClearSpace"),
            PlanNodeEnum::AlterSpace(_) => self.default_node_id("AlterSpace"),
            PlanNodeEnum::CreateEdge(_) => self.default_node_id("CreateEdge"),
            PlanNodeEnum::DropEdge(_) => self.default_node_id("DropEdge"),
            PlanNodeEnum::ShowEdges(_) => self.default_node_id("ShowEdges"),
            PlanNodeEnum::ShowCreateEdge(_) => self.default_node_id("ShowCreateEdge"),
            PlanNodeEnum::SubmitJob(_) => self.default_node_id("SubmitJob"),
            PlanNodeEnum::CreateSnapshot(_) => self.default_node_id("CreateSnapshot"),
            PlanNodeEnum::DropSnapshot(_) => self.default_node_id("DropSnapshot"),
            PlanNodeEnum::ShowSnapshots(_) => self.default_node_id("ShowSnapshots"),
            PlanNodeEnum::CreateIndex(_) => self.default_node_id("CreateIndex"),
            PlanNodeEnum::DropIndex(_) => self.default_node_id("DropIndex"),
            PlanNodeEnum::ShowIndexes(_) => self.default_node_id("ShowIndexes"),
            PlanNodeEnum::DescIndex(_) => self.default_node_id("DescIndex"),
            PlanNodeEnum::AddHosts(_) => self.default_node_id("AddHosts"),
            PlanNodeEnum::DropHosts(_) => self.default_node_id("DropHosts"),
            PlanNodeEnum::ShowHosts(_) => self.default_node_id("ShowHosts"),
            PlanNodeEnum::ShowHostsStatus(_) => self.default_node_id("ShowHostsStatus"),
            PlanNodeEnum::ShowConfigs(_) => self.default_node_id("ShowConfigs"),
            PlanNodeEnum::SetConfig(_) => self.default_node_id("SetConfig"),
            PlanNodeEnum::GetConfig(_) => self.default_node_id("GetConfig"),
        }
    }

    /// 为管理节点提供默认的 ID 实现
    fn default_node_id(&self, _node_type: &str) -> i64 {
        // 使用节点类型的哈希值作为默认 ID
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        _node_type.hash(&mut hasher);
        (hasher.finish() % i64::MAX as u64) as i64
    }

    /// 获取节点类型的名称
    pub fn name(&self) -> &'static str {
        match self {
            // 基础节点类型
            PlanNodeEnum::Start(_) => "Start",
            PlanNodeEnum::Project(_) => "Project",
            PlanNodeEnum::Sort(_) => "Sort",
            PlanNodeEnum::Limit(_) => "Limit",
            PlanNodeEnum::TopN(_) => "TopN",
            PlanNodeEnum::InnerJoin(_) => "InnerJoin",
            PlanNodeEnum::LeftJoin(_) => "LeftJoin",
            PlanNodeEnum::CrossJoin(_) => "CrossJoin",
            PlanNodeEnum::HashInnerJoin(_) => "HashInnerJoin",
            PlanNodeEnum::HashLeftJoin(_) => "HashLeftJoin",
            PlanNodeEnum::CartesianProduct(_) => "CartesianProduct",
            PlanNodeEnum::IndexScan(_) => "IndexScan",
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

    /// 获取节点的输出变量
    pub fn output_var(&self) -> Option<&Variable> {
        match self {
            // 基础节点类型 - 这些节点实现了 PlanNode trait
            PlanNodeEnum::Start(node) => node.output_var(),
            PlanNodeEnum::Project(node) => node.output_var(),
            PlanNodeEnum::Sort(node) => node.output_var(),
            PlanNodeEnum::Limit(node) => node.output_var(),
            PlanNodeEnum::TopN(node) => node.output_var(),
            PlanNodeEnum::InnerJoin(node) => node.output_var(),
            PlanNodeEnum::LeftJoin(node) => node.output_var(),
            PlanNodeEnum::CrossJoin(node) => node.output_var(),
            PlanNodeEnum::HashInnerJoin(node) => node.output_var(),
            PlanNodeEnum::HashLeftJoin(node) => node.output_var(),
            PlanNodeEnum::CartesianProduct(node) => node.output_var(),
            PlanNodeEnum::IndexScan(node) => node.output_var(),
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

            // 管理节点类型 - 默认返回 None
            PlanNodeEnum::CreateUser(_) => None,
            PlanNodeEnum::DropUser(_) => None,
            PlanNodeEnum::UpdateUser(_) => None,
            PlanNodeEnum::ChangePassword(_) => None,
            PlanNodeEnum::ListUsers(_) => None,
            PlanNodeEnum::ListUserRoles(_) => None,
            PlanNodeEnum::DescribeUser(_) => None,
            PlanNodeEnum::CreateRole(_) => None,
            PlanNodeEnum::DropRole(_) => None,
            PlanNodeEnum::GrantRole(_) => None,
            PlanNodeEnum::RevokeRole(_) => None,
            PlanNodeEnum::ShowRoles(_) => None,
            PlanNodeEnum::UpdateVertex(_) => None,
            PlanNodeEnum::UpdateEdge(_) => None,
            PlanNodeEnum::InsertVertices(_) => None,
            PlanNodeEnum::InsertEdges(_) => None,
            PlanNodeEnum::DeleteVertices(_) => None,
            PlanNodeEnum::DeleteTags(_) => None,
            PlanNodeEnum::DeleteEdges(_) => None,
            PlanNodeEnum::NewVertex(_) => None,
            PlanNodeEnum::NewTag(_) => None,
            PlanNodeEnum::NewProp(_) => None,
            PlanNodeEnum::NewEdge(_) => None,
            PlanNodeEnum::CreateTag(_) => None,
            PlanNodeEnum::DescTag(_) => None,
            PlanNodeEnum::DropTag(_) => None,
            PlanNodeEnum::ShowTags(_) => None,
            PlanNodeEnum::ShowCreateTag(_) => None,
            PlanNodeEnum::CreateSpace(_) => None,
            PlanNodeEnum::DescSpace(_) => None,
            PlanNodeEnum::ShowCreateSpace(_) => None,
            PlanNodeEnum::ShowSpaces(_) => None,
            PlanNodeEnum::SwitchSpace(_) => None,
            PlanNodeEnum::DropSpace(_) => None,
            PlanNodeEnum::ClearSpace(_) => None,
            PlanNodeEnum::AlterSpace(_) => None,
            PlanNodeEnum::CreateEdge(_) => None,
            PlanNodeEnum::DropEdge(_) => None,
            PlanNodeEnum::ShowEdges(_) => None,
            PlanNodeEnum::ShowCreateEdge(_) => None,
            PlanNodeEnum::SubmitJob(_) => None,
            PlanNodeEnum::CreateSnapshot(_) => None,
            PlanNodeEnum::DropSnapshot(_) => None,
            PlanNodeEnum::ShowSnapshots(_) => None,
            PlanNodeEnum::CreateIndex(_) => None,
            PlanNodeEnum::DropIndex(_) => None,
            PlanNodeEnum::ShowIndexes(_) => None,
            PlanNodeEnum::DescIndex(_) => None,
            PlanNodeEnum::AddHosts(_) => None,
            PlanNodeEnum::DropHosts(_) => None,
            PlanNodeEnum::ShowHosts(_) => None,
            PlanNodeEnum::ShowHostsStatus(_) => None,
            PlanNodeEnum::ShowConfigs(_) => None,
            PlanNodeEnum::SetConfig(_) => None,
            PlanNodeEnum::GetConfig(_) => None,
        }
    }

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        match self {
            // 基础节点类型 - 这些节点实现了 PlanNode trait
            PlanNodeEnum::Start(node) => node.col_names(),
            PlanNodeEnum::Project(node) => node.col_names(),
            PlanNodeEnum::Sort(node) => node.col_names(),
            PlanNodeEnum::Limit(node) => node.col_names(),
            PlanNodeEnum::TopN(node) => node.col_names(),
            PlanNodeEnum::InnerJoin(node) => node.col_names(),
            PlanNodeEnum::LeftJoin(node) => node.col_names(),
            PlanNodeEnum::CrossJoin(node) => node.col_names(),
            PlanNodeEnum::HashInnerJoin(node) => node.col_names(),
            PlanNodeEnum::HashLeftJoin(node) => node.col_names(),
            PlanNodeEnum::CartesianProduct(node) => node.col_names(),
            PlanNodeEnum::IndexScan(node) => node.col_names(),
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

            // 管理节点类型 - 默认返回空列表
            PlanNodeEnum::CreateUser(_) => &[],
            PlanNodeEnum::DropUser(_) => &[],
            PlanNodeEnum::UpdateUser(_) => &[],
            PlanNodeEnum::ChangePassword(_) => &[],
            PlanNodeEnum::ListUsers(_) => &[],
            PlanNodeEnum::ListUserRoles(_) => &[],
            PlanNodeEnum::DescribeUser(_) => &[],
            PlanNodeEnum::CreateRole(_) => &[],
            PlanNodeEnum::DropRole(_) => &[],
            PlanNodeEnum::GrantRole(_) => &[],
            PlanNodeEnum::RevokeRole(_) => &[],
            PlanNodeEnum::ShowRoles(_) => &[],
            PlanNodeEnum::UpdateVertex(_) => &[],
            PlanNodeEnum::UpdateEdge(_) => &[],
            PlanNodeEnum::InsertVertices(_) => &[],
            PlanNodeEnum::InsertEdges(_) => &[],
            PlanNodeEnum::DeleteVertices(_) => &[],
            PlanNodeEnum::DeleteTags(_) => &[],
            PlanNodeEnum::DeleteEdges(_) => &[],
            PlanNodeEnum::NewVertex(_) => &[],
            PlanNodeEnum::NewTag(_) => &[],
            PlanNodeEnum::NewProp(_) => &[],
            PlanNodeEnum::NewEdge(_) => &[],
            PlanNodeEnum::CreateTag(_) => &[],
            PlanNodeEnum::DescTag(_) => &[],
            PlanNodeEnum::DropTag(_) => &[],
            PlanNodeEnum::ShowTags(_) => &[],
            PlanNodeEnum::ShowCreateTag(_) => &[],
            PlanNodeEnum::CreateSpace(_) => &[],
            PlanNodeEnum::DescSpace(_) => &[],
            PlanNodeEnum::ShowCreateSpace(_) => &[],
            PlanNodeEnum::ShowSpaces(_) => &[],
            PlanNodeEnum::SwitchSpace(_) => &[],
            PlanNodeEnum::DropSpace(_) => &[],
            PlanNodeEnum::ClearSpace(_) => &[],
            PlanNodeEnum::AlterSpace(_) => &[],
            PlanNodeEnum::CreateEdge(_) => &[],
            PlanNodeEnum::DropEdge(_) => &[],
            PlanNodeEnum::ShowEdges(_) => &[],
            PlanNodeEnum::ShowCreateEdge(_) => &[],
            PlanNodeEnum::SubmitJob(_) => &[],
            PlanNodeEnum::CreateSnapshot(_) => &[],
            PlanNodeEnum::DropSnapshot(_) => &[],
            PlanNodeEnum::ShowSnapshots(_) => &[],
            PlanNodeEnum::CreateIndex(_) => &[],
            PlanNodeEnum::DropIndex(_) => &[],
            PlanNodeEnum::ShowIndexes(_) => &[],
            PlanNodeEnum::DescIndex(_) => &[],
            PlanNodeEnum::AddHosts(_) => &[],
            PlanNodeEnum::DropHosts(_) => &[],
            PlanNodeEnum::ShowHosts(_) => &[],
            PlanNodeEnum::ShowHostsStatus(_) => &[],
            PlanNodeEnum::ShowConfigs(_) => &[],
            PlanNodeEnum::SetConfig(_) => &[],
            PlanNodeEnum::GetConfig(_) => &[],
        }
    }

    /// 获取节点的成本估计值
    pub fn cost(&self) -> f64 {
        match self {
            // 基础节点类型 - 这些节点实现了 PlanNode trait
            PlanNodeEnum::Start(node) => node.cost(),
            PlanNodeEnum::Project(node) => node.cost(),
            PlanNodeEnum::Sort(node) => node.cost(),
            PlanNodeEnum::Limit(node) => node.cost(),
            PlanNodeEnum::TopN(node) => node.cost(),
            PlanNodeEnum::InnerJoin(node) => node.cost(),
            PlanNodeEnum::LeftJoin(node) => node.cost(),
            PlanNodeEnum::CrossJoin(node) => node.cost(),
            PlanNodeEnum::HashInnerJoin(node) => node.cost(),
            PlanNodeEnum::HashLeftJoin(node) => node.cost(),
            PlanNodeEnum::CartesianProduct(node) => node.cost(),
            PlanNodeEnum::IndexScan(node) => node.cost(),
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

            // 管理节点类型 - 默认返回 1.0
            PlanNodeEnum::CreateUser(_) => 1.0,
            PlanNodeEnum::DropUser(_) => 1.0,
            PlanNodeEnum::UpdateUser(_) => 1.0,
            PlanNodeEnum::ChangePassword(_) => 1.0,
            PlanNodeEnum::ListUsers(_) => 1.0,
            PlanNodeEnum::ListUserRoles(_) => 1.0,
            PlanNodeEnum::DescribeUser(_) => 1.0,
            PlanNodeEnum::CreateRole(_) => 1.0,
            PlanNodeEnum::DropRole(_) => 1.0,
            PlanNodeEnum::GrantRole(_) => 1.0,
            PlanNodeEnum::RevokeRole(_) => 1.0,
            PlanNodeEnum::ShowRoles(_) => 1.0,
            PlanNodeEnum::UpdateVertex(_) => 1.0,
            PlanNodeEnum::UpdateEdge(_) => 1.0,
            PlanNodeEnum::InsertVertices(_) => 1.0,
            PlanNodeEnum::InsertEdges(_) => 1.0,
            PlanNodeEnum::DeleteVertices(_) => 1.0,
            PlanNodeEnum::DeleteTags(_) => 1.0,
            PlanNodeEnum::DeleteEdges(_) => 1.0,
            PlanNodeEnum::NewVertex(_) => 1.0,
            PlanNodeEnum::NewTag(_) => 1.0,
            PlanNodeEnum::NewProp(_) => 1.0,
            PlanNodeEnum::NewEdge(_) => 1.0,
            PlanNodeEnum::CreateTag(_) => 1.0,
            PlanNodeEnum::DescTag(_) => 1.0,
            PlanNodeEnum::DropTag(_) => 1.0,
            PlanNodeEnum::ShowTags(_) => 1.0,
            PlanNodeEnum::ShowCreateTag(_) => 1.0,
            PlanNodeEnum::CreateSpace(_) => 1.0,
            PlanNodeEnum::DescSpace(_) => 1.0,
            PlanNodeEnum::ShowCreateSpace(_) => 1.0,
            PlanNodeEnum::ShowSpaces(_) => 1.0,
            PlanNodeEnum::SwitchSpace(_) => 1.0,
            PlanNodeEnum::DropSpace(_) => 1.0,
            PlanNodeEnum::ClearSpace(_) => 1.0,
            PlanNodeEnum::AlterSpace(_) => 1.0,
            PlanNodeEnum::CreateEdge(_) => 1.0,
            PlanNodeEnum::DropEdge(_) => 1.0,
            PlanNodeEnum::ShowEdges(_) => 1.0,
            PlanNodeEnum::ShowCreateEdge(_) => 1.0,
            PlanNodeEnum::SubmitJob(_) => 1.0,
            PlanNodeEnum::CreateSnapshot(_) => 1.0,
            PlanNodeEnum::DropSnapshot(_) => 1.0,
            PlanNodeEnum::ShowSnapshots(_) => 1.0,
            PlanNodeEnum::CreateIndex(_) => 1.0,
            PlanNodeEnum::DropIndex(_) => 1.0,
            PlanNodeEnum::ShowIndexes(_) => 1.0,
            PlanNodeEnum::DescIndex(_) => 1.0,
            PlanNodeEnum::AddHosts(_) => 1.0,
            PlanNodeEnum::DropHosts(_) => 1.0,
            PlanNodeEnum::ShowHosts(_) => 1.0,
            PlanNodeEnum::ShowHostsStatus(_) => 1.0,
            PlanNodeEnum::ShowConfigs(_) => 1.0,
            PlanNodeEnum::SetConfig(_) => 1.0,
            PlanNodeEnum::GetConfig(_) => 1.0,
        }
    }

    /// 获取节点的依赖节点列表
    pub fn dependencies(&self) -> &[Box<PlanNodeEnum>] {
        static EMPTY: [Box<PlanNodeEnum>; 0] = [];
        match self {
            // 基础节点类型 - 这些节点实现了 PlanNode trait
            PlanNodeEnum::Start(_node) => {
                // StartNode 没有依赖
                &EMPTY
            }
            PlanNodeEnum::Project(node) => node.dependencies(),
            PlanNodeEnum::Sort(node) => node.dependencies(),
            PlanNodeEnum::Limit(node) => node.dependencies(),
            PlanNodeEnum::TopN(node) => node.dependencies(),
            PlanNodeEnum::InnerJoin(node) => node.dependencies(),
            PlanNodeEnum::LeftJoin(node) => node.dependencies(),
            PlanNodeEnum::CrossJoin(node) => node.dependencies(),
            PlanNodeEnum::HashInnerJoin(node) => node.dependencies(),
            PlanNodeEnum::HashLeftJoin(node) => node.dependencies(),
            PlanNodeEnum::CartesianProduct(node) => node.dependencies(),
            PlanNodeEnum::IndexScan(node) => node.dependencies(),
            PlanNodeEnum::GetVertices(node) => node.dependencies(),
            PlanNodeEnum::GetEdges(node) => node.dependencies(),
            PlanNodeEnum::GetNeighbors(node) => node.dependencies(),
            PlanNodeEnum::ScanVertices(node) => node.dependencies(),
            PlanNodeEnum::ScanEdges(node) => node.dependencies(),
            PlanNodeEnum::Expand(node) => node.dependencies(),
            PlanNodeEnum::ExpandAll(node) => node.dependencies(),
            PlanNodeEnum::Traverse(node) => node.dependencies(),
            PlanNodeEnum::AppendVertices(node) => node.dependencies(),
            PlanNodeEnum::Filter(node) => node.dependencies(),
            PlanNodeEnum::Aggregate(node) => node.dependencies(),
            PlanNodeEnum::Argument(node) => node.dependencies(),
            PlanNodeEnum::Loop(node) => node.dependencies(),
            PlanNodeEnum::PassThrough(node) => node.dependencies(),
            PlanNodeEnum::Select(node) => node.dependencies(),
            PlanNodeEnum::DataCollect(node) => node.dependencies(),
            PlanNodeEnum::Dedup(node) => node.dependencies(),
            PlanNodeEnum::PatternApply(node) => node.dependencies(),
            PlanNodeEnum::RollUpApply(node) => node.dependencies(),
            PlanNodeEnum::Union(node) => node.dependencies(),
            PlanNodeEnum::Unwind(node) => node.dependencies(),

            // 管理节点类型 - 默认返回空列表
            PlanNodeEnum::CreateUser(_) => &EMPTY,
            PlanNodeEnum::DropUser(_) => &EMPTY,
            PlanNodeEnum::UpdateUser(_) => &EMPTY,
            PlanNodeEnum::ChangePassword(_) => &EMPTY,
            PlanNodeEnum::ListUsers(_) => &EMPTY,
            PlanNodeEnum::ListUserRoles(_) => &EMPTY,
            PlanNodeEnum::DescribeUser(_) => &EMPTY,
            PlanNodeEnum::CreateRole(_) => &EMPTY,
            PlanNodeEnum::DropRole(_) => &EMPTY,
            PlanNodeEnum::GrantRole(_) => &EMPTY,
            PlanNodeEnum::RevokeRole(_) => &EMPTY,
            PlanNodeEnum::ShowRoles(_) => &EMPTY,
            PlanNodeEnum::UpdateVertex(_) => &EMPTY,
            PlanNodeEnum::UpdateEdge(_) => &EMPTY,
            PlanNodeEnum::InsertVertices(_) => &EMPTY,
            PlanNodeEnum::InsertEdges(_) => &EMPTY,
            PlanNodeEnum::DeleteVertices(_) => &EMPTY,
            PlanNodeEnum::DeleteTags(_) => &EMPTY,
            PlanNodeEnum::DeleteEdges(_) => &EMPTY,
            PlanNodeEnum::NewVertex(_) => &EMPTY,
            PlanNodeEnum::NewTag(_) => &EMPTY,
            PlanNodeEnum::NewProp(_) => &EMPTY,
            PlanNodeEnum::NewEdge(_) => &EMPTY,
            PlanNodeEnum::CreateTag(_) => &EMPTY,
            PlanNodeEnum::DescTag(_) => &EMPTY,
            PlanNodeEnum::DropTag(_) => &EMPTY,
            PlanNodeEnum::ShowTags(_) => &EMPTY,
            PlanNodeEnum::ShowCreateTag(_) => &EMPTY,
            PlanNodeEnum::CreateSpace(_) => &EMPTY,
            PlanNodeEnum::DescSpace(_) => &EMPTY,
            PlanNodeEnum::ShowCreateSpace(_) => &EMPTY,
            PlanNodeEnum::ShowSpaces(_) => &EMPTY,
            PlanNodeEnum::SwitchSpace(_) => &EMPTY,
            PlanNodeEnum::DropSpace(_) => &EMPTY,
            PlanNodeEnum::ClearSpace(_) => &EMPTY,
            PlanNodeEnum::AlterSpace(_) => &EMPTY,
            PlanNodeEnum::CreateEdge(_) => &EMPTY,
            PlanNodeEnum::DropEdge(_) => &EMPTY,
            PlanNodeEnum::ShowEdges(_) => &EMPTY,
            PlanNodeEnum::ShowCreateEdge(_) => &EMPTY,
            PlanNodeEnum::SubmitJob(_) => &EMPTY,
            PlanNodeEnum::CreateSnapshot(_) => &EMPTY,
            PlanNodeEnum::DropSnapshot(_) => &EMPTY,
            PlanNodeEnum::ShowSnapshots(_) => &EMPTY,
            PlanNodeEnum::CreateIndex(_) => &EMPTY,
            PlanNodeEnum::DropIndex(_) => &EMPTY,
            PlanNodeEnum::ShowIndexes(_) => &EMPTY,
            PlanNodeEnum::DescIndex(_) => &EMPTY,
            PlanNodeEnum::AddHosts(_) => &EMPTY,
            PlanNodeEnum::DropHosts(_) => &EMPTY,
            PlanNodeEnum::ShowHosts(_) => &EMPTY,
            PlanNodeEnum::ShowHostsStatus(_) => &EMPTY,
            PlanNodeEnum::ShowConfigs(_) => &EMPTY,
            PlanNodeEnum::SetConfig(_) => &EMPTY,
            PlanNodeEnum::GetConfig(_) => &EMPTY,
        }
    }

    /// 设置节点的输出变量
    pub fn set_output_var(&mut self, var: Variable) {
        match self {
            // 基础节点类型 - 这些节点实现了 PlanNode trait
            PlanNodeEnum::Start(node) => node.set_output_var(var),
            PlanNodeEnum::Project(node) => node.set_output_var(var),
            PlanNodeEnum::Sort(node) => node.set_output_var(var),
            PlanNodeEnum::Limit(node) => node.set_output_var(var),
            PlanNodeEnum::TopN(node) => node.set_output_var(var),
            PlanNodeEnum::InnerJoin(node) => node.set_output_var(var),
            PlanNodeEnum::LeftJoin(node) => node.set_output_var(var),
            PlanNodeEnum::CrossJoin(node) => node.set_output_var(var),
            PlanNodeEnum::HashInnerJoin(node) => node.set_output_var(var),
            PlanNodeEnum::HashLeftJoin(node) => node.set_output_var(var),
            PlanNodeEnum::CartesianProduct(node) => node.set_output_var(var),
            PlanNodeEnum::IndexScan(node) => node.set_output_var(var),
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

            // 管理节点类型 - 默认不执行任何操作
            PlanNodeEnum::CreateUser(_) => {}
            PlanNodeEnum::DropUser(_) => {}
            PlanNodeEnum::UpdateUser(_) => {}
            PlanNodeEnum::ChangePassword(_) => {}
            PlanNodeEnum::ListUsers(_) => {}
            PlanNodeEnum::ListUserRoles(_) => {}
            PlanNodeEnum::DescribeUser(_) => {}
            PlanNodeEnum::CreateRole(_) => {}
            PlanNodeEnum::DropRole(_) => {}
            PlanNodeEnum::GrantRole(_) => {}
            PlanNodeEnum::RevokeRole(_) => {}
            PlanNodeEnum::ShowRoles(_) => {}
            PlanNodeEnum::UpdateVertex(_) => {}
            PlanNodeEnum::UpdateEdge(_) => {}
            PlanNodeEnum::InsertVertices(_) => {}
            PlanNodeEnum::InsertEdges(_) => {}
            PlanNodeEnum::DeleteVertices(_) => {}
            PlanNodeEnum::DeleteTags(_) => {}
            PlanNodeEnum::DeleteEdges(_) => {}
            PlanNodeEnum::NewVertex(_) => {}
            PlanNodeEnum::NewTag(_) => {}
            PlanNodeEnum::NewProp(_) => {}
            PlanNodeEnum::NewEdge(_) => {}
            PlanNodeEnum::CreateTag(_) => {}
            PlanNodeEnum::DescTag(_) => {}
            PlanNodeEnum::DropTag(_) => {}
            PlanNodeEnum::ShowTags(_) => {}
            PlanNodeEnum::ShowCreateTag(_) => {}
            PlanNodeEnum::CreateSpace(_) => {}
            PlanNodeEnum::DescSpace(_) => {}
            PlanNodeEnum::ShowCreateSpace(_) => {}
            PlanNodeEnum::ShowSpaces(_) => {}
            PlanNodeEnum::SwitchSpace(_) => {}
            PlanNodeEnum::DropSpace(_) => {}
            PlanNodeEnum::ClearSpace(_) => {}
            PlanNodeEnum::AlterSpace(_) => {}
            PlanNodeEnum::CreateEdge(_) => {}
            PlanNodeEnum::DropEdge(_) => {}
            PlanNodeEnum::ShowEdges(_) => {}
            PlanNodeEnum::ShowCreateEdge(_) => {}
            PlanNodeEnum::SubmitJob(_) => {}
            PlanNodeEnum::CreateSnapshot(_) => {}
            PlanNodeEnum::DropSnapshot(_) => {}
            PlanNodeEnum::ShowSnapshots(_) => {}
            PlanNodeEnum::CreateIndex(_) => {}
            PlanNodeEnum::DropIndex(_) => {}
            PlanNodeEnum::ShowIndexes(_) => {}
            PlanNodeEnum::DescIndex(_) => {}
            PlanNodeEnum::AddHosts(_) => {}
            PlanNodeEnum::DropHosts(_) => {}
            PlanNodeEnum::ShowHosts(_) => {}
            PlanNodeEnum::ShowHostsStatus(_) => {}
            PlanNodeEnum::ShowConfigs(_) => {}
            PlanNodeEnum::SetConfig(_) => {}
            PlanNodeEnum::GetConfig(_) => {}
        }
    }

    /// 设置列名
    pub fn set_col_names(&mut self, names: Vec<String>) {
        match self {
            // 基础节点类型 - 这些节点实现了 PlanNode trait
            PlanNodeEnum::Start(node) => node.set_col_names(names),
            PlanNodeEnum::Project(node) => node.set_col_names(names),
            PlanNodeEnum::Sort(node) => node.set_col_names(names),
            PlanNodeEnum::Limit(node) => node.set_col_names(names),
            PlanNodeEnum::TopN(node) => node.set_col_names(names),
            PlanNodeEnum::InnerJoin(node) => node.set_col_names(names),
            PlanNodeEnum::LeftJoin(node) => node.set_col_names(names),
            PlanNodeEnum::CrossJoin(node) => node.set_col_names(names),
            PlanNodeEnum::HashInnerJoin(node) => node.set_col_names(names),
            PlanNodeEnum::HashLeftJoin(node) => node.set_col_names(names),
            PlanNodeEnum::CartesianProduct(node) => node.set_col_names(names),
            PlanNodeEnum::IndexScan(node) => node.set_col_names(names),
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

            // 管理节点类型 - 默认不执行任何操作
            PlanNodeEnum::CreateUser(_) => {}
            PlanNodeEnum::DropUser(_) => {}
            PlanNodeEnum::UpdateUser(_) => {}
            PlanNodeEnum::ChangePassword(_) => {}
            PlanNodeEnum::ListUsers(_) => {}
            PlanNodeEnum::ListUserRoles(_) => {}
            PlanNodeEnum::DescribeUser(_) => {}
            PlanNodeEnum::CreateRole(_) => {}
            PlanNodeEnum::DropRole(_) => {}
            PlanNodeEnum::GrantRole(_) => {}
            PlanNodeEnum::RevokeRole(_) => {}
            PlanNodeEnum::ShowRoles(_) => {}
            PlanNodeEnum::UpdateVertex(_) => {}
            PlanNodeEnum::UpdateEdge(_) => {}
            PlanNodeEnum::InsertVertices(_) => {}
            PlanNodeEnum::InsertEdges(_) => {}
            PlanNodeEnum::DeleteVertices(_) => {}
            PlanNodeEnum::DeleteTags(_) => {}
            PlanNodeEnum::DeleteEdges(_) => {}
            PlanNodeEnum::NewVertex(_) => {}
            PlanNodeEnum::NewTag(_) => {}
            PlanNodeEnum::NewProp(_) => {}
            PlanNodeEnum::NewEdge(_) => {}
            PlanNodeEnum::CreateTag(_) => {}
            PlanNodeEnum::DescTag(_) => {}
            PlanNodeEnum::DropTag(_) => {}
            PlanNodeEnum::ShowTags(_) => {}
            PlanNodeEnum::ShowCreateTag(_) => {}
            PlanNodeEnum::CreateSpace(_) => {}
            PlanNodeEnum::DescSpace(_) => {}
            PlanNodeEnum::ShowCreateSpace(_) => {}
            PlanNodeEnum::ShowSpaces(_) => {}
            PlanNodeEnum::SwitchSpace(_) => {}
            PlanNodeEnum::DropSpace(_) => {}
            PlanNodeEnum::ClearSpace(_) => {}
            PlanNodeEnum::AlterSpace(_) => {}
            PlanNodeEnum::CreateEdge(_) => {}
            PlanNodeEnum::DropEdge(_) => {}
            PlanNodeEnum::ShowEdges(_) => {}
            PlanNodeEnum::ShowCreateEdge(_) => {}
            PlanNodeEnum::SubmitJob(_) => {}
            PlanNodeEnum::CreateSnapshot(_) => {}
            PlanNodeEnum::DropSnapshot(_) => {}
            PlanNodeEnum::ShowSnapshots(_) => {}
            PlanNodeEnum::CreateIndex(_) => {}
            PlanNodeEnum::DropIndex(_) => {}
            PlanNodeEnum::ShowIndexes(_) => {}
            PlanNodeEnum::DescIndex(_) => {}
            PlanNodeEnum::AddHosts(_) => {}
            PlanNodeEnum::DropHosts(_) => {}
            PlanNodeEnum::ShowHosts(_) => {}
            PlanNodeEnum::ShowHostsStatus(_) => {}
            PlanNodeEnum::ShowConfigs(_) => {}
            PlanNodeEnum::SetConfig(_) => {}
            PlanNodeEnum::GetConfig(_) => {}
        }
    }
}
