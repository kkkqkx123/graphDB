//! 模式定义语言(DDL)相关的计划节点
//! 包括创建/删除空间、标签、边等操作

mod edge_ops;
mod space_ops;
mod tag_ops;

pub use edge_ops::*;
pub use space_ops::*;
pub use tag_ops::*;

// 重新导出新增的空间管理节点
pub use space_ops::{
    DropSpace, ClearSpace, AlterSpace, AlterSpaceOption
};
