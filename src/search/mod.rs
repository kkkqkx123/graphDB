pub mod adapters;
pub mod engine;
pub mod error;
pub mod result;

pub use engine::{EngineType, SearchEngine};
pub use error::{Result, SearchError};
pub use result::{IndexStats, SearchResult};
