//! 管理操作相关的计划节点模块
//! 包括模式管理、数据管理和系统管理等操作

mod schema_ops;
mod data_ops;
mod system_ops;

// 重新导出管理节点类型
pub use schema_ops::{CreateNode, DropNode, CreateSpace, CreateTag, CreateEdge, DescSpace, ShowCreateSpace, DescTag, ShowSpaces, SwitchSpace, CreateUser, DropUser};
pub use data_ops::{InsertVertices, InsertEdges, UpdateVertex, UpdateEdge, DeleteVertices, DeleteEdges, DeleteTags, NewVertex, NewTag, NewProp, NewEdge};
pub use system_ops::{SubmitJob, CreateSnapshot, DropSnapshot, ShowSnapshots, ShowConfigs, SetConfig, JobType};