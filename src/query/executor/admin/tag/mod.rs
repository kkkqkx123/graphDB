//! 标签管理执行器
//!
//! 提供标签的创建、修改、描述、删除和列出功能。

pub mod create_tag;
pub mod alter_tag;
pub mod desc_tag;
pub mod drop_tag;
pub mod show_tags;

pub use create_tag::CreateTagExecutor;
pub use alter_tag::AlterTagExecutor;
pub use desc_tag::DescTagExecutor;
pub use drop_tag::DropTagExecutor;
pub use show_tags::ShowTagsExecutor;
