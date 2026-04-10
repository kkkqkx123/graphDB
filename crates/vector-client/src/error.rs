use thiserror::Error;

pub type Result<T> = std::result::Result<T, VectorClientError>;

#[derive(Error, Debug)]
pub enum VectorClientError {
    #[error("Collection '{0}' not found")]
    CollectionNotFound(String),

    #[error("Collection '{0}' already exists")]
    CollectionAlreadyExists(String),

    #[error("Index '{0}' already exists")]
    IndexAlreadyExists(String),

    #[error("Point '{0}' not found in collection '{1}'")]
    PointNotFound(String, String),

    #[error("Invalid vector dimension: expected {expected}, got {actual}")]
    InvalidVectorDimension { expected: usize, actual: usize },

    #[error("Invalid collection name: '{0}'")]
    InvalidCollectionName(String),

    #[error("Invalid point ID: '{0}'")]
    InvalidPointId(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Operation timeout: {0}")]
    Timeout(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Search error: {0}")]
    SearchError(String),

    #[error("Upsert error: {0}")]
    UpsertError(String),

    #[error("Delete error: {0}")]
    DeleteError(String),

    #[error("Payload error: {0}")]
    PayloadError(String),

    #[error("Filter error: {0}")]
    FilterError(String),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Engine not initialized")]
    EngineNotInitialized,

    #[error("Engine '{0}' is not available (feature not enabled)")]
    EngineNotAvailable(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[cfg(feature = "qdrant")]
    #[error("Qdrant error: {0}")]
    QdrantError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl VectorClientError {
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            VectorClientError::CollectionNotFound(_)
                | VectorClientError::PointNotFound(_, _)
        )
    }

    pub fn is_connection_error(&self) -> bool {
        matches!(
            self,
            VectorClientError::ConnectionFailed(_)
                | VectorClientError::Timeout(_)
                | VectorClientError::HealthCheckFailed(_)
        )
    }
}

#[cfg(feature = "qdrant")]
impl From<qdrant_client::QdrantError> for VectorClientError {
    fn from(err: qdrant_client::QdrantError) -> Self {
        VectorClientError::QdrantError(err.to_string())
    }
}
