//! 空间管理执行器
//!
//! 提供图空间的创建、删除、修改、清空、描述、列出和切换功能。

pub mod create_space;
pub mod drop_space;
pub mod desc_space;
pub mod show_spaces;
pub mod switch_space;
pub mod alter_space;
pub mod clear_space;

#[cfg(test)]
mod tests;

pub use create_space::CreateSpaceExecutor;
pub use drop_space::DropSpaceExecutor;
pub use desc_space::DescSpaceExecutor;
pub use show_spaces::ShowSpacesExecutor;
pub use switch_space::SwitchSpaceExecutor;
pub use alter_space::{AlterSpaceExecutor, SpaceAlterOption};
pub use clear_space::ClearSpaceExecutor;
