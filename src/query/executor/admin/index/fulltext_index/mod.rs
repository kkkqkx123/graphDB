//! Fulltext Index Management Executor
//!
//! Provide functions for creating, deleting, altering, describing, and showing fulltext indexes.

pub mod create_fulltext_index;
pub mod drop_fulltext_index;
pub mod alter_fulltext_index;
pub mod show_fulltext_index;
pub mod describe_fulltext_index;

#[cfg(test)]
mod tests;

pub use create_fulltext_index::CreateFulltextIndexExecutor;
pub use drop_fulltext_index::DropFulltextIndexExecutor;
pub use alter_fulltext_index::AlterFulltextIndexExecutor;
pub use show_fulltext_index::ShowFulltextIndexExecutor;
pub use describe_fulltext_index::DescribeFulltextIndexExecutor;