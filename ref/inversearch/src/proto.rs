pub mod proto {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AddDocumentRequest {
        pub id: u64,
        pub content: String,
        pub metadata: std::collections::HashMap<String, String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AddDocumentResponse {
        pub success: bool,
        pub error: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateDocumentRequest {
        pub id: u64,
        pub content: String,
        pub metadata: std::collections::HashMap<String, String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UpdateDocumentResponse {
        pub success: bool,
        pub error: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RemoveDocumentRequest {
        pub id: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RemoveDocumentResponse {
        pub success: bool,
        pub error: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchRequest {
        pub query: String,
        pub limit: u32,
        pub offset: u32,
        pub context: bool,
        pub suggest: bool,
        pub resolve: bool,
        pub enrich: bool,
        pub cache: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SearchResponse {
        pub results: Vec<u64>,
        pub total: u32,
        pub error: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ClearIndexRequest {}

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ClearIndexResponse {
        pub success: bool,
        pub error: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GetStatsRequest {}

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GetStatsResponse {
        pub document_count: u64,
        pub index_size: u64,
        pub cache_size: u64,
        pub error: Option<String>,
    }
}
