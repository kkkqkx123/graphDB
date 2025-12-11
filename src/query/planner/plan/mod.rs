//! 执行计划节点相关定义和结构
//! 包含PlanNode特征、各种计划节点类型和执行计划结构

pub mod plan_node;
pub mod execution_plan;
pub mod plan_node_visitor;
pub mod query_nodes;
pub mod logic_nodes;
pub mod admin_nodes;
pub mod algo_nodes;
pub mod mutate_nodes;
pub mod maintain_nodes;
pub mod scan_nodes;
pub mod common;
pub mod graph_scan;
pub mod data_ops;
pub mod join_ops;
pub mod traverse_ops;
pub mod aggregation_ops;
pub mod sort_limit_ops;
pub mod other_ops;

// 重新导出主要的类型
pub use plan_node::{PlanNode, PlanNodeKind, SingleDependencyNode, SingleInputNode, BinaryInputNode, VariableDependencyNode};
pub use execution_plan::{ExecutionPlan, SubPlan};
pub use plan_node_visitor::{PlanNodeVisitor, PlanNodeVisitError};

// 导出新拆分的节点类型
pub use common::*;
pub use graph_scan::*;
pub use data_ops::*;
pub use join_ops::*;
pub use traverse_ops::*;
pub use aggregation_ops::*;
pub use sort_limit_ops::*;
pub use other_ops::*;

// 导出逻辑节点
pub use logic_nodes::{
    StartNode, SelectNode, LoopNode, PassThroughNode, ArgumentNode, BinarySelectNode
};

// 导出管理节点
pub use admin_nodes::{
    CreateNode, DropNode, CreateSpace, CreateTag, CreateEdge, DescSpace,
    ShowCreateSpace, DescTag, ShowSpaces, SwitchSpace, CreateUser, DropUser
};

// 导出算法节点
pub use algo_nodes::{
    MultiShortestPath, BFSShortest, AllPaths, ShortestPath
};

// 导出数据修改节点
pub use mutate_nodes::{
    InsertVertices, InsertEdges, UpdateVertex, UpdateEdge,
    DeleteVertices, DeleteEdges, DeleteTags, NewVertex, NewTag, NewProp, NewEdge
};

// 导出维护节点
pub use maintain_nodes::{
    SubmitJob, CreateSnapshot, DropSnapshot, ShowSnapshots,
    ShowConfigs, SetConfig, JobType
};

// 导出查询节点
pub use query_nodes::{
    GetNeighbors, Traverse, AppendVertices, Union, Unwind,
    Aggregate, HashJoin, Sort, Limit, TopN, Sample, DataCollect
};

// 导出扫描节点
pub use scan_nodes::{
    ScanVertices, ScanEdges, IndexScan, FulltextIndexScan
};
