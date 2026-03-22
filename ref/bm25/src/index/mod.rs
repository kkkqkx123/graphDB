pub mod manager;
pub mod schema;
pub mod document;
pub mod delete;
pub mod batch;
pub mod stats;
pub mod search;
pub mod cache;
pub mod persistence;
pub mod tests;

pub use manager::IndexManager;
pub use schema::IndexSchema;
