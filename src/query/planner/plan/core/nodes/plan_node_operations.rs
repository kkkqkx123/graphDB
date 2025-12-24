//! PlanNode 操作实现
//!
//! 实现 PlanNodeEnum 的各种操作方法

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::{BinaryInputNode, MultipleInputNode, PlanNode, SingleInputNode, ZeroInputNode};
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
        }
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
    pub fn dependencies(&self) -> Vec<Box<PlanNodeEnum>> {
        match self {
            // 零输入节点
            PlanNodeEnum::Start(_node) => vec![],
            PlanNodeEnum::GetVertices(_node) => vec![],
            PlanNodeEnum::GetEdges(_node) => vec![],
            PlanNodeEnum::GetNeighbors(_node) => vec![],
            PlanNodeEnum::ScanVertices(_node) => vec![],
            PlanNodeEnum::ScanEdges(_node) => vec![],
            PlanNodeEnum::IndexScan(_node) => vec![],
            PlanNodeEnum::FulltextIndexScan(_node) => vec![],
            PlanNodeEnum::MultiShortestPath(_node) => vec![],
            PlanNodeEnum::BFSShortest(_node) => vec![],
            PlanNodeEnum::AllPaths(_node) => vec![],
            PlanNodeEnum::ShortestPath(_node) => vec![],

            // 单输入节点
            PlanNodeEnum::Project(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Sort(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Limit(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::TopN(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Filter(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Aggregate(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::DataCollect(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Dedup(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::PatternApply(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::RollUpApply(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Union(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Unwind(node) => vec![Box::new(node.input().clone())],

            // 双输入节点
            PlanNodeEnum::InnerJoin(node) => vec![Box::new(node.left_input().clone()), Box::new(node.right_input().clone())],
            PlanNodeEnum::LeftJoin(node) => vec![Box::new(node.left_input().clone()), Box::new(node.right_input().clone())],
            PlanNodeEnum::CrossJoin(node) => vec![Box::new(node.left_input().clone()), Box::new(node.right_input().clone())],
            PlanNodeEnum::HashInnerJoin(node) => vec![Box::new(node.left_input().clone()), Box::new(node.right_input().clone())],
            PlanNodeEnum::HashLeftJoin(node) => vec![Box::new(node.left_input().clone()), Box::new(node.right_input().clone())],
            PlanNodeEnum::CartesianProduct(node) => vec![Box::new(node.left_input().clone()), Box::new(node.right_input().clone())],

            // 多输入节点
            PlanNodeEnum::Expand(node) => node.inputs().iter().map(|input| input.as_ref().clone()).collect(),
            PlanNodeEnum::ExpandAll(node) => node.inputs().iter().map(|input| input.as_ref().clone()).collect(),
            PlanNodeEnum::Traverse(node) => node.inputs().iter().map(|input| input.as_ref().clone()).collect(),
            PlanNodeEnum::AppendVertices(node) => node.inputs().iter().map(|input| input.as_ref().clone()).collect(),

            // 其他节点
            PlanNodeEnum::Argument(_node) => vec![],
            PlanNodeEnum::Loop(_node) => vec![],
            PlanNodeEnum::PassThrough(_node) => vec![],
            PlanNodeEnum::Select(_node) => vec![],

            // 管理节点类型 - 默认返回空列表
            PlanNodeEnum::CreateUser(_) => vec![],
            PlanNodeEnum::DropUser(_) => vec![],
            PlanNodeEnum::UpdateUser(_) => vec![],
            PlanNodeEnum::ChangePassword(_) => vec![],
            PlanNodeEnum::ListUsers(_) => vec![],
            PlanNodeEnum::ListUserRoles(_) => vec![],
            PlanNodeEnum::DescribeUser(_) => vec![],
            PlanNodeEnum::CreateRole(_) => vec![],
            PlanNodeEnum::DropRole(_) => vec![],
            PlanNodeEnum::GrantRole(_) => vec![],
            PlanNodeEnum::RevokeRole(_) => vec![],
            PlanNodeEnum::ShowRoles(_) => vec![],
            PlanNodeEnum::UpdateVertex(_) => vec![],
            PlanNodeEnum::UpdateEdge(_) => vec![],
            PlanNodeEnum::InsertVertices(_) => vec![],
            PlanNodeEnum::InsertEdges(_) => vec![],
            PlanNodeEnum::DeleteVertices(_) => vec![],
            PlanNodeEnum::DeleteTags(_) => vec![],
            PlanNodeEnum::DeleteEdges(_) => vec![],
            PlanNodeEnum::NewVertex(_) => vec![],
            PlanNodeEnum::NewTag(_) => vec![],
            PlanNodeEnum::NewProp(_) => vec![],
            PlanNodeEnum::NewEdge(_) => vec![],
            PlanNodeEnum::CreateTag(_) => vec![],
            PlanNodeEnum::DescTag(_) => vec![],
            PlanNodeEnum::DropTag(_) => vec![],
            PlanNodeEnum::ShowTags(_) => vec![],
            PlanNodeEnum::ShowCreateTag(_) => vec![],
            PlanNodeEnum::CreateSpace(_) => vec![],
            PlanNodeEnum::DescSpace(_) => vec![],
            PlanNodeEnum::ShowCreateSpace(_) => vec![],
            PlanNodeEnum::ShowSpaces(_) => vec![],
            PlanNodeEnum::SwitchSpace(_) => vec![],
            PlanNodeEnum::DropSpace(_) => vec![],
            PlanNodeEnum::ClearSpace(_) => vec![],
            PlanNodeEnum::AlterSpace(_) => vec![],
            PlanNodeEnum::CreateEdge(_) => vec![],
            PlanNodeEnum::DropEdge(_) => vec![],
            PlanNodeEnum::ShowEdges(_) => vec![],
            PlanNodeEnum::ShowCreateEdge(_) => vec![],
            PlanNodeEnum::SubmitJob(_) => vec![],
            PlanNodeEnum::CreateSnapshot(_) => vec![],
            PlanNodeEnum::DropSnapshot(_) => vec![],
            PlanNodeEnum::ShowSnapshots(_) => vec![],
            PlanNodeEnum::CreateIndex(_) => vec![],
            PlanNodeEnum::DropIndex(_) => vec![],
            PlanNodeEnum::ShowIndexes(_) => vec![],
            PlanNodeEnum::DescIndex(_) => vec![],
            PlanNodeEnum::AddHosts(_) => vec![],
            PlanNodeEnum::DropHosts(_) => vec![],
            PlanNodeEnum::ShowHosts(_) => vec![],
            PlanNodeEnum::ShowHostsStatus(_) => vec![],
            PlanNodeEnum::ShowConfigs(_) => vec![],
            PlanNodeEnum::SetConfig(_) => vec![],
            PlanNodeEnum::GetConfig(_) => vec![],
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
