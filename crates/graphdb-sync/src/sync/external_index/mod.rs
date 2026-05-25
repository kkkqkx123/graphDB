pub mod error;
pub mod fulltext_client;
pub mod fulltext_error;
pub mod trait_def;
#[cfg(feature = "qdrant")]
pub mod vector_client;
pub mod vector_error;

pub use error::ExternalIndexError;
pub use error::IndexResult;
pub use fulltext_client::FulltextClient;
pub use fulltext_error::{CoordinatorError, CoordinatorResult, FulltextError, FulltextResult};
pub use trait_def::{ExternalIndexClient, IndexData, IndexKey, IndexOperation, IndexOptions};
#[cfg(feature = "qdrant")]
pub use vector_client::{VectorClient, VectorClientConfig};
pub use vector_error::{
    VectorCoordinatorError, VectorCoordinatorResult, VectorError, VectorResult,
};
