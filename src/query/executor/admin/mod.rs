//! 管理执行器模块
//!
//! 提供数据库管理功能，包括空间管理、标签管理、边类型管理、索引管理、数据变更、用户管理等。
//! 针对单节点部署进行了简化，移除了分布式相关功能。

pub mod space;
pub mod tag;
pub mod edge;
pub mod index;
pub mod user;

pub use self::space::{
    CreateSpaceExecutor, DropSpaceExecutor, DescSpaceExecutor, ShowSpacesExecutor,
};

pub use self::tag::{
    CreateTagExecutor, AlterTagExecutor, DescTagExecutor, DropTagExecutor, ShowTagsExecutor,
};

pub use self::tag::alter_tag::{AlterTagInfo, AlterTagItem, AlterTagOp};

pub use self::edge::{
    CreateEdgeExecutor, AlterEdgeExecutor, DescEdgeExecutor, DropEdgeExecutor, ShowEdgesExecutor,
};

pub use self::edge::alter_edge::{AlterEdgeInfo, AlterEdgeItem, AlterEdgeOp};

pub use self::index::{
    CreateTagIndexExecutor, DropTagIndexExecutor, DescTagIndexExecutor, ShowTagIndexesExecutor,
    CreateEdgeIndexExecutor, DropEdgeIndexExecutor, DescEdgeIndexExecutor, ShowEdgeIndexesExecutor,
    RebuildTagIndexExecutor, RebuildEdgeIndexExecutor,
};

pub use self::user::{
    ChangePasswordExecutor,
};

pub use crate::core::types::metadata::PasswordInfo;
