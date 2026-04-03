pub mod fulltext;
pub mod types;

#[cfg(test)]
pub mod fulltext_test;

pub use fulltext::{FulltextCoordinator, ChangeType};
pub use types::*;
