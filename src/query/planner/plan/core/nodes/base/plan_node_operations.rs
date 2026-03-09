//! PlanNode 操作实现
//!
//! 实现 PlanNodeEnum 的各种操作方法

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::{MultipleInputNode, PlanNode, SingleInputNode};

/// 为 PlanNodeEnum 生成 match 分支的宏（带默认值）
///
/// 这个宏生成对所有节点类型的匹配，自动调用指定的方法，对管理节点返回默认值
macro_rules! match_all_nodes_with_default {
    ($self:expr, $method:ident, $default:expr) => {
        match $self {
            PlanNodeEnum::Start(node) => node.$method(),
            PlanNodeEnum::Project(node) => node.$method(),
            PlanNodeEnum::Sort(node) => node.$method(),
            PlanNodeEnum::Limit(node) => node.$method(),
            PlanNodeEnum::TopN(node) => node.$method(),
            PlanNodeEnum::Sample(node) => node.$method(),
            PlanNodeEnum::InnerJoin(node) => node.$method(),
            PlanNodeEnum::LeftJoin(node) => node.$method(),
            PlanNodeEnum::CrossJoin(node) => node.$method(),
            PlanNodeEnum::HashInnerJoin(node) => node.$method(),
            PlanNodeEnum::HashLeftJoin(node) => node.$method(),
            PlanNodeEnum::FullOuterJoin(node) => node.$method(),
            PlanNodeEnum::IndexScan(node) => node.$method(),
            PlanNodeEnum::EdgeIndexScan(node) => node.$method(),
            PlanNodeEnum::GetVertices(node) => node.$method(),
            PlanNodeEnum::GetEdges(node) => node.$method(),
            PlanNodeEnum::GetNeighbors(node) => node.$method(),
            PlanNodeEnum::ScanVertices(node) => node.$method(),
            PlanNodeEnum::ScanEdges(node) => node.$method(),
            PlanNodeEnum::Expand(node) => node.$method(),
            PlanNodeEnum::ExpandAll(node) => node.$method(),
            PlanNodeEnum::Traverse(node) => node.$method(),
            PlanNodeEnum::AppendVertices(node) => node.$method(),
            PlanNodeEnum::Filter(node) => node.$method(),
            PlanNodeEnum::Aggregate(node) => node.$method(),
            PlanNodeEnum::Argument(node) => node.$method(),
            PlanNodeEnum::Loop(node) => node.$method(),
            PlanNodeEnum::PassThrough(node) => node.$method(),
            PlanNodeEnum::Select(node) => node.$method(),
            PlanNodeEnum::DataCollect(node) => node.$method(),
            PlanNodeEnum::Dedup(node) => node.$method(),
            PlanNodeEnum::PatternApply(node) => node.$method(),
            PlanNodeEnum::RollUpApply(node) => node.$method(),
            PlanNodeEnum::Union(node) => node.$method(),
            PlanNodeEnum::Minus(node) => node.$method(),
            PlanNodeEnum::Intersect(node) => node.$method(),
            PlanNodeEnum::Unwind(node) => node.$method(),
            PlanNodeEnum::Assign(node) => node.$method(),
            PlanNodeEnum::MultiShortestPath(node) => node.$method(),
            PlanNodeEnum::BFSShortest(node) => node.$method(),
            PlanNodeEnum::AllPaths(node) => node.$method(),
            PlanNodeEnum::ShortestPath(node) => node.$method(),
            // 管理节点返回默认值
            _ => $default,
        }
    };
}

impl PlanNodeEnum {
    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        match_all_nodes_with_default!(self, id, 0)
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
            PlanNodeEnum::Sample(_) => "Sample",
            PlanNodeEnum::InnerJoin(_) => "InnerJoin",
            PlanNodeEnum::LeftJoin(_) => "LeftJoin",
            PlanNodeEnum::CrossJoin(_) => "CrossJoin",
            PlanNodeEnum::HashInnerJoin(_) => "HashInnerJoin",
            PlanNodeEnum::HashLeftJoin(_) => "HashLeftJoin",
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
            PlanNodeEnum::Assign(_) => "Assign",
            PlanNodeEnum::MultiShortestPath(_) => "MultiShortestPath",
            PlanNodeEnum::BFSShortest(_) => "BFSShortest",
            PlanNodeEnum::AllPaths(_) => "AllPaths",
            PlanNodeEnum::ShortestPath(_) => "ShortestPath",

            // 管理节点
            _ => "AdminNode",
        }
    }

    /// 获取节点的输出变量
    pub fn output_var(&self) -> Option<&str> {
        match self {
            // 基础节点类型 - 这些节点实现了 PlanNode trait
            PlanNodeEnum::Start(node) => node.output_var(),
            PlanNodeEnum::Project(node) => node.output_var(),
            PlanNodeEnum::Sort(node) => node.output_var(),
            PlanNodeEnum::Limit(node) => node.output_var(),
            PlanNodeEnum::TopN(node) => node.output_var(),
            PlanNodeEnum::Sample(node) => node.output_var(),
            PlanNodeEnum::InnerJoin(node) => node.output_var(),
            PlanNodeEnum::LeftJoin(node) => node.output_var(),
            PlanNodeEnum::CrossJoin(node) => node.output_var(),
            PlanNodeEnum::HashInnerJoin(node) => node.output_var(),
            PlanNodeEnum::HashLeftJoin(node) => node.output_var(),
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
            PlanNodeEnum::Assign(node) => node.output_var(),
            PlanNodeEnum::MultiShortestPath(node) => node.output_var(),
            PlanNodeEnum::BFSShortest(node) => node.output_var(),
            PlanNodeEnum::AllPaths(node) => node.output_var(),
            PlanNodeEnum::ShortestPath(node) => node.output_var(),

            // 管理节点 - 无输出变量
            _ => None,
        }
    }

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        match_all_nodes_with_default!(self, col_names, &[])
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
            PlanNodeEnum::MultiShortestPath(_node) => vec![],
            PlanNodeEnum::BFSShortest(_node) => vec![],
            PlanNodeEnum::AllPaths(_node) => vec![],
            PlanNodeEnum::ShortestPath(_node) => vec![],

            // 单输入节点
            PlanNodeEnum::Project(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Sort(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Limit(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::TopN(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Sample(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Filter(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Aggregate(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::DataCollect(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Dedup(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::PatternApply(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::RollUpApply(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Union(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Unwind(node) => vec![Box::new(node.input().clone())],
            PlanNodeEnum::Assign(node) => vec![Box::new(node.input().clone())],

            // 双输入节点
            PlanNodeEnum::InnerJoin(node) => vec![
                Box::new(node.left_input().clone()),
                Box::new(node.right_input().clone()),
            ],
            PlanNodeEnum::LeftJoin(node) => vec![
                Box::new(node.left_input().clone()),
                Box::new(node.right_input().clone()),
            ],
            PlanNodeEnum::CrossJoin(node) => vec![
                Box::new(node.left_input().clone()),
                Box::new(node.right_input().clone()),
            ],
            PlanNodeEnum::HashInnerJoin(node) => vec![
                Box::new(node.left_input().clone()),
                Box::new(node.right_input().clone()),
            ],
            PlanNodeEnum::HashLeftJoin(node) => vec![
                Box::new(node.left_input().clone()),
                Box::new(node.right_input().clone()),
            ],

            // 多输入节点
            PlanNodeEnum::Expand(node) => node.inputs().iter().map(|input| input.clone()).collect(),
            PlanNodeEnum::ExpandAll(node) => {
                node.inputs().iter().map(|input| input.clone()).collect()
            }
            PlanNodeEnum::Traverse(node) => {
                vec![Box::new(node.input().clone())]
            }
            PlanNodeEnum::AppendVertices(node) => {
                node.inputs().iter().map(|input| input.clone()).collect()
            }

            // 其他节点
            PlanNodeEnum::Argument(_node) => vec![],
            PlanNodeEnum::Loop(_node) => vec![],
            PlanNodeEnum::PassThrough(_node) => vec![],
            PlanNodeEnum::Select(_node) => vec![],

            // 管理节点 - 无输入依赖
            _ => vec![],
        }
    }

    /// 获取第一个依赖节点（如果存在）
    pub fn first_dependency(&self) -> Option<PlanNodeEnum> {
        let deps = self.dependencies();
        if deps.is_empty() {
            None
        } else {
            Some(*deps[0].clone())
        }
    }

    /// 设置节点的输出变量
    pub fn set_output_var(&mut self, var: String) {
        match self {
            // 基础节点类型 - 这些节点实现了 PlanNode trait
            PlanNodeEnum::Start(node) => node.set_output_var(var),
            PlanNodeEnum::Project(node) => node.set_output_var(var),
            PlanNodeEnum::Sort(node) => node.set_output_var(var),
            PlanNodeEnum::Limit(node) => node.set_output_var(var),
            PlanNodeEnum::TopN(node) => node.set_output_var(var),
            PlanNodeEnum::Sample(node) => node.set_output_var(var),
            PlanNodeEnum::InnerJoin(node) => node.set_output_var(var),
            PlanNodeEnum::LeftJoin(node) => node.set_output_var(var),
            PlanNodeEnum::CrossJoin(node) => node.set_output_var(var),
            PlanNodeEnum::HashInnerJoin(node) => node.set_output_var(var),
            PlanNodeEnum::HashLeftJoin(node) => node.set_output_var(var),
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
            PlanNodeEnum::Assign(node) => node.set_output_var(var),
            PlanNodeEnum::MultiShortestPath(node) => node.set_output_var(var),
            PlanNodeEnum::BFSShortest(node) => node.set_output_var(var),
            PlanNodeEnum::AllPaths(node) => node.set_output_var(var),
            PlanNodeEnum::ShortestPath(node) => node.set_output_var(var),

            // 管理节点 - 不需要设置输出变量
            _ => {}
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
            PlanNodeEnum::Sample(node) => node.set_col_names(names),
            PlanNodeEnum::InnerJoin(node) => node.set_col_names(names),
            PlanNodeEnum::LeftJoin(node) => node.set_col_names(names),
            PlanNodeEnum::CrossJoin(node) => node.set_col_names(names),
            PlanNodeEnum::HashInnerJoin(node) => node.set_col_names(names),
            PlanNodeEnum::HashLeftJoin(node) => node.set_col_names(names),
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
            PlanNodeEnum::Assign(node) => node.set_col_names(names),
            PlanNodeEnum::MultiShortestPath(node) => node.set_col_names(names),
            PlanNodeEnum::BFSShortest(node) => node.set_col_names(names),
            PlanNodeEnum::AllPaths(node) => node.set_col_names(names),
            PlanNodeEnum::ShortestPath(node) => node.set_col_names(names),

            // 管理节点 - 不需要设置列名
            _ => {}
        }
    }
}
