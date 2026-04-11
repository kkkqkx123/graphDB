pub mod error;
pub mod fulltext_client;
pub mod trait_def;
pub mod vector_client;

pub use error::ExternalIndexError;
pub use error::IndexResult;
pub use fulltext_client::FulltextClient;
pub use trait_def::{ExternalIndexClient, IndexData, IndexKey, IndexOperation, IndexOptions};
pub use vector_client::VectorClient;
