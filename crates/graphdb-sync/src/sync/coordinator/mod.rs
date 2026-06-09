#![allow(clippy::module_inception)]

pub mod coordinator;
pub mod error;
pub mod types;

pub use coordinator::RecoveryResult;
pub use coordinator::SyncCoordinator;
pub use coordinator::SyncCoordinatorError;
pub use error::{CoordinatorError, CoordinatorResult, FulltextError, FulltextResult};
pub use types::{ChangeContext, ChangeData, ChangeType, IndexType};
