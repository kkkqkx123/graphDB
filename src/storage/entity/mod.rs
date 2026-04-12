//! Entity Storage Module
//!
//! Manages storage operations for different entity types (vertices, edges, users, events).

pub mod edge_storage;
pub mod event_storage;
pub mod user_storage;
pub mod vertex_storage;

pub use edge_storage::EdgeStorage;
pub use event_storage::SyncStorage;
pub use user_storage::UserStorage;
pub use vertex_storage::VertexStorage;
