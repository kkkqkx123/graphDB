pub mod batch;
pub mod delete;
pub mod document;
pub mod manager;
pub mod persistence;
pub mod schema;
pub mod search;
pub mod stats;
pub mod tests;

pub use manager::{IndexManager, IndexManagerConfig};
pub use schema::IndexSchema;
