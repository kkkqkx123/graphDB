use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::client::client_trait::{
    ClientConfig, GraphDbClient, SessionInfo, TransactionInfo, TransactionOptions,
};
use crate::utils::error::{CliError, Result};

/// HTTP client for connecting to remote GraphDB server
pub struct HttpClient {
    inner: reqwest::Client,
    base_url: String,
    config: ClientConfig,
    connected: bool,
    session_info: Option<SessionInfo>,
}

impl HttpClient {
    /// Create a new HTTP client with default settings
    pub fn new(host: &str, port: u16) -> Result<Self> {
        let config = ClientConfig::new().with_host(host).with_port(port);
        Self::with_config(config)
    }

    /// Create a new HTTP client with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let base_url = format!("http://{}:{}/v1", config.host, config.port);
        let inner = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| CliError::connection(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            inner,
            base_url,
            config,
            connected: false,
            session_info: None,
        })
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the underlying reqwest client
    pub fn inner(&self) -> &reqwest::Client {
        &self.inner
    }

    /// Login and authenticate (low-level API)
    async fn login(&self, username: &str, password: &str) -> Result<(i64, String)> {
        let url = format!("{}/auth/login", self.base_url);
        let request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::auth(format!(
                "Login failed ({}): {}",
                status, body
            )));
        }

        let login_resp: LoginResponse = response.json().await?;
        Ok((login_resp.session_id, login_resp.username))
    }
}

#[async_trait]
impl GraphDbClient for HttpClient {
    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn connect(&mut self) -> Result<SessionInfo> {
        let (session_id, username) = self
            .login(&self.config.username, &self.config.password)
            .await?;

        let session_info = SessionInfo {
            session_id,
            username: username.clone(),
            host: self.config.host.clone(),
            port: self.config.port,
        };

        self.session_info = Some(session_info.clone());
        self.connected = true;

        Ok(session_info)
    }

    async fn disconnect(&mut self) -> Result<()> {
        // Call logout endpoint if we have a session
        if let Some(ref session_info) = self.session_info {
            let url = format!("{}/auth/logout", self.base_url);
            let request = LogoutRequest {
                session_id: session_info.session_id,
            };

            // Attempt to logout, but don't fail if the server is unreachable
            match self.inner.post(&url).json(&request).send().await {
                Ok(response) => {
                    if !response.status().is_success() {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        eprintln!("Warning: Logout failed ({}): {}", status, body);
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to contact server during logout: {}", e);
                }
            }
        }

        self.connected = false;
        self.session_info = None;
        Ok(())
    }

    async fn execute_query(&self, query: &str, session_id: i64) -> Result<QueryResult> {
        let url = format!("{}/query", self.base_url);
        let request = QueryRequest {
            query: query.to_string(),
            session_id,
            parameters: HashMap::new(),
        };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Query failed ({}): {}",
                status, body
            )));
        }

        let query_resp: QueryResponse = response.json().await?;

        if !query_resp.success {
            let err = query_resp.error.unwrap_or(QueryError {
                code: "UNKNOWN".to_string(),
                message: "Unknown error".to_string(),
                details: None,
            });
            return Err(CliError::query(format!("{}: {}", err.code, err.message)));
        }

        let data = query_resp.data.unwrap_or(QueryData {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 0,
        });

        let metadata = query_resp.metadata.unwrap_or(QueryMetadata {
            execution_time_ms: 0,
            rows_scanned: 0,
            rows_returned: 0,
            space_id: None,
        });

        Ok(QueryResult {
            columns: data.columns,
            rows: data.rows,
            row_count: data.row_count,
            execution_time_ms: metadata.execution_time_ms,
            rows_scanned: metadata.rows_scanned,
            error: None,
        })
    }

    async fn execute_query_raw(&self, query: &str, session_id: i64) -> Result<QueryResult> {
        // HTTP mode doesn't do variable substitution, so same as execute_query
        self.execute_query(query, session_id).await
    }

    async fn list_spaces(&self) -> Result<Vec<SpaceInfo>> {
        let url = format!("{}/schema/spaces", self.base_url);
        let response = self.inner.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to list spaces ({}): {}",
                status, body
            )));
        }

        let body: serde_json::Value = response.json().await?;
        let spaces = body
            .get("spaces")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(spaces)
    }

    async fn switch_space(&self, space: &str) -> Result<()> {
        let url = format!("{}/schema/spaces/{}", self.base_url, space);
        let response = self.inner.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to use space '{}' ({}): {}",
                space, status, body
            )));
        }

        Ok(())
    }

    async fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>> {
        let url = format!("{}/schema/spaces/{}/tags", self.base_url, space);
        let response = self.inner.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to list tags ({}): {}",
                status, body
            )));
        }

        let body: serde_json::Value = response.json().await?;
        let tags = body
            .get("tags")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(tags)
    }

    async fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>> {
        let url = format!("{}/schema/spaces/{}/edge-types", self.base_url, space);
        let response = self.inner.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to list edge types ({}): {}",
                status, body
            )));
        }

        let body: serde_json::Value = response.json().await?;
        let edge_types = body
            .get("edge_types")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(edge_types)
    }

    async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let response = self.inner.get(&url).send().await;
        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    fn connection_string(&self) -> String {
        self.base_url.clone()
    }

    async fn begin_transaction(&self, options: TransactionOptions) -> Result<TransactionInfo> {
        let url = format!("{}/transactions", self.base_url);

        // Get session_id from session_info
        let session_id = self
            .session_info
            .as_ref()
            .map(|s| s.session_id)
            .ok_or_else(|| CliError::session("Not connected".to_string()))?;

        let request = BeginTransactionRequest {
            session_id,
            read_only: options.read_only,
            timeout_seconds: options.timeout_seconds,
        };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::transaction(format!(
                "Failed to begin transaction ({}): {}",
                status, body
            )));
        }

        let txn_resp: TransactionResponse = response.json().await?;
        Ok(TransactionInfo {
            transaction_id: txn_resp.transaction_id,
            status: txn_resp.status,
        })
    }

    async fn commit_transaction(&self, txn_id: u64) -> Result<()> {
        let url = format!("{}/transactions/{}/commit", self.base_url, txn_id);

        // Get session_id from session_info
        let session_id = self
            .session_info
            .as_ref()
            .map(|s| s.session_id)
            .ok_or_else(|| CliError::session("Not connected".to_string()))?;

        let request = TransactionActionRequest { session_id };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::transaction(format!(
                "Failed to commit transaction ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn rollback_transaction(&self, txn_id: u64) -> Result<()> {
        let url = format!("{}/transactions/{}/rollback", self.base_url, txn_id);

        // Get session_id from session_info
        let session_id = self
            .session_info
            .as_ref()
            .map(|s| s.session_id)
            .ok_or_else(|| CliError::session("Not connected".to_string()))?;

        let request = TransactionActionRequest { session_id };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::transaction(format!(
                "Failed to rollback transaction ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn create_space(
        &self,
        name: &str,
        vid_type: Option<&str>,
        comment: Option<&str>,
    ) -> Result<()> {
        let url = format!("{}/schema/spaces", self.base_url);
        let request = CreateSpaceRequest {
            name: name.to_string(),
            vid_type: vid_type.map(|s| s.to_string()),
            comment: comment.map(|s| s.to_string()),
        };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to create space ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn drop_space(&self, name: &str) -> Result<()> {
        let url = format!("{}/schema/spaces/{}", self.base_url, name);

        let response = self.inner.delete(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to drop space ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn create_tag(
        &self,
        space: &str,
        name: &str,
        properties: Vec<crate::client::client_trait::PropertyDef>,
    ) -> Result<()> {
        let url = format!("{}/schema/spaces/{}/tags", self.base_url, space);

        let props: Vec<PropertyDefInput> = properties
            .into_iter()
            .map(|p| PropertyDefInput {
                name: p.name,
                data_type: p.data_type.to_string(),
                nullable: p.nullable,
            })
            .collect();

        let request = CreateTagRequest {
            name: name.to_string(),
            properties: props,
        };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to create tag ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn create_edge_type(
        &self,
        space: &str,
        name: &str,
        properties: Vec<crate::client::client_trait::PropertyDef>,
    ) -> Result<()> {
        let url = format!("{}/schema/spaces/{}/edge-types", self.base_url, space);

        let props: Vec<PropertyDefInput> = properties
            .into_iter()
            .map(|p| PropertyDefInput {
                name: p.name,
                data_type: p.data_type.to_string(),
                nullable: p.nullable,
            })
            .collect();

        let request = CreateEdgeTypeRequest {
            name: name.to_string(),
            properties: props,
        };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to create edge type ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn create_batch(
        &self,
        space_id: u64,
        batch_type: crate::client::client_trait::BatchType,
        batch_size: usize,
    ) -> Result<String> {
        let url = format!("{}/batch", self.base_url);

        let batch_type_str = match batch_type {
            crate::client::client_trait::BatchType::Vertex => "vertex",
            crate::client::client_trait::BatchType::Edge => "edge",
            crate::client::client_trait::BatchType::Mixed => "mixed",
        };

        let request = CreateBatchRequest {
            space_id,
            batch_type: batch_type_str.to_string(),
            batch_size,
        };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to create batch ({}): {}",
                status, body
            )));
        }

        let batch_resp: CreateBatchResponse = response.json().await?;
        Ok(batch_resp.batch_id)
    }

    async fn add_batch_items(
        &self,
        batch_id: &str,
        items: Vec<crate::client::client_trait::BatchItem>,
    ) -> Result<usize> {
        let url = format!("{}/batch/{}/items", self.base_url, batch_id);

        let batch_items: Vec<BatchItem> = items
            .into_iter()
            .map(|item| match item {
                crate::client::client_trait::BatchItem::Vertex(v) => {
                    BatchItem::Vertex(VertexData {
                        vid: v.vid,
                        tags: v.tags,
                        properties: v.properties,
                    })
                }
                crate::client::client_trait::BatchItem::Edge(e) => BatchItem::Edge(EdgeData {
                    edge_type: e.edge_type,
                    src_vid: e.src_vid,
                    dst_vid: e.dst_vid,
                    properties: e.properties,
                }),
            })
            .collect();

        let request = AddBatchItemsRequest { items: batch_items };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to add batch items ({}): {}",
                status, body
            )));
        }

        let add_resp: AddBatchItemsResponse = response.json().await?;
        Ok(add_resp.accepted)
    }

    async fn execute_batch(
        &self,
        batch_id: &str,
    ) -> Result<crate::client::client_trait::BatchResult> {
        let url = format!("{}/batch/{}/execute", self.base_url, batch_id);

        let response = self.inner.post(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to execute batch ({}): {}",
                status, body
            )));
        }

        let exec_resp: ExecuteBatchResponse = response.json().await?;
        Ok(crate::client::client_trait::BatchResult {
            batch_id: exec_resp.batch_id,
            status: format!("{:?}", exec_resp.status),
            vertices_inserted: exec_resp.result.vertices_inserted,
            edges_inserted: exec_resp.result.edges_inserted,
            errors: exec_resp
                .result
                .errors
                .into_iter()
                .map(|e| crate::client::client_trait::BatchError {
                    index: e.index,
                    item_type: format!("{:?}", e.item_type),
                    error: e.error,
                })
                .collect(),
        })
    }

    async fn get_batch_status(
        &self,
        batch_id: &str,
    ) -> Result<crate::client::client_trait::BatchStatus> {
        let url = format!("{}/batch/{}", self.base_url, batch_id);

        let response = self.inner.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to get batch status ({}): {}",
                status, body
            )));
        }

        let status_resp: BatchStatusResponse = response.json().await?;
        Ok(crate::client::client_trait::BatchStatus {
            batch_id: status_resp.batch_id,
            status: format!("{:?}", status_resp.status),
            total: status_resp.progress.total,
            processed: status_resp.progress.processed,
            succeeded: status_resp.progress.succeeded,
            failed: status_resp.progress.failed,
        })
    }

    async fn cancel_batch(&self, batch_id: &str) -> Result<()> {
        let url = format!("{}/batch/{}/cancel", self.base_url, batch_id);

        let response = self.inner.post(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to cancel batch ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn get_session_statistics(
        &self,
        session_id: i64,
    ) -> Result<crate::client::client_trait::SessionStatistics> {
        let url = format!("{}/statistics/sessions/{}", self.base_url, session_id);

        let response = self.inner.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to get session statistics ({}): {}",
                status, body
            )));
        }

        let stats: serde_json::Value = response.json().await?;
        let stats_obj = stats
            .get("statistics")
            .ok_or_else(|| CliError::query("Missing statistics field"))?;

        Ok(crate::client::client_trait::SessionStatistics {
            total_queries: stats_obj
                .get("total_queries")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            total_changes: stats_obj
                .get("total_changes")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            avg_execution_time_ms: stats_obj
                .get("avg_execution_time_ms")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        })
    }

    async fn get_query_statistics(&self) -> Result<crate::client::client_trait::QueryStatistics> {
        let url = format!("{}/statistics/queries", self.base_url);

        let response = self.inner.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to get query statistics ({}): {}",
                status, body
            )));
        }

        let stats: serde_json::Value = response.json().await?;

        let slow_queries: Vec<crate::client::client_trait::SlowQueryInfo> = stats
            .get("slow_queries")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|q| {
                        Some(crate::client::client_trait::SlowQueryInfo {
                            trace_id: q.get("trace_id")?.as_str()?.to_string(),
                            session_id: q.get("session_id")?.as_i64()?,
                            query: q.get("query")?.as_str()?.to_string(),
                            duration_ms: q.get("duration_ms")?.as_f64()?,
                            status: q.get("status")?.as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let query_types = stats
            .get("query_types")
            .map(|qt| crate::client::client_trait::QueryTypeStatistics {
                match_queries: qt.get("MATCH").and_then(|v| v.as_u64()).unwrap_or(0),
                create_queries: qt.get("CREATE").and_then(|v| v.as_u64()).unwrap_or(0),
                update_queries: qt.get("UPDATE").and_then(|v| v.as_u64()).unwrap_or(0),
                delete_queries: qt.get("DELETE").and_then(|v| v.as_u64()).unwrap_or(0),
                insert_queries: qt.get("INSERT").and_then(|v| v.as_u64()).unwrap_or(0),
                go_queries: qt.get("GO").and_then(|v| v.as_u64()).unwrap_or(0),
                fetch_queries: qt.get("FETCH").and_then(|v| v.as_u64()).unwrap_or(0),
                lookup_queries: qt.get("LOOKUP").and_then(|v| v.as_u64()).unwrap_or(0),
                show_queries: qt.get("SHOW").and_then(|v| v.as_u64()).unwrap_or(0),
            })
            .unwrap_or(crate::client::client_trait::QueryTypeStatistics {
                match_queries: 0,
                create_queries: 0,
                update_queries: 0,
                delete_queries: 0,
                insert_queries: 0,
                go_queries: 0,
                fetch_queries: 0,
                lookup_queries: 0,
                show_queries: 0,
            });

        Ok(crate::client::client_trait::QueryStatistics {
            total_queries: stats
                .get("total_queries")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            slow_queries,
            query_types,
        })
    }

    async fn get_database_statistics(
        &self,
    ) -> Result<crate::client::client_trait::DatabaseStatistics> {
        let url = format!("{}/statistics/database", self.base_url);

        let response = self.inner.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to get database statistics ({}): {}",
                status, body
            )));
        }

        let stats: serde_json::Value = response.json().await?;

        let spaces = stats.get("spaces");
        let performance = stats.get("performance");

        Ok(crate::client::client_trait::DatabaseStatistics {
            space_count: spaces
                .and_then(|s| s.get("count"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            total_vertices: spaces
                .and_then(|s| s.get("total_vertices"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            total_edges: spaces
                .and_then(|s| s.get("total_edges"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            total_queries: performance
                .and_then(|p| p.get("total_queries"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            active_queries: performance
                .and_then(|p| p.get("active_queries"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            queries_per_second: performance
                .and_then(|p| p.get("queries_per_second"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            avg_latency_ms: performance
                .and_then(|p| p.get("avg_latency_ms"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        })
    }

    async fn validate_query(
        &self,
        query: &str,
    ) -> Result<crate::client::client_trait::ValidationResult> {
        let url = format!("{}/query/validate", self.base_url);

        let session_id = self.session_info.as_ref().map(|s| s.session_id);

        let request = ValidateQueryRequest {
            query: query.to_string(),
            session_id,
        };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to validate query ({}): {}",
                status, body
            )));
        }

        let validate_resp: ValidateQueryResponse = response.json().await?;

        let errors = validate_resp
            .errors
            .into_iter()
            .map(|e| crate::client::client_trait::ValidationError {
                code: e.code,
                message: e.message,
                position: e.position,
                line: e.line,
                column: e.column,
            })
            .collect();

        let warnings = validate_resp
            .warnings
            .into_iter()
            .map(|w| crate::client::client_trait::ValidationWarning {
                code: w.code,
                message: w.message,
                suggestion: w.suggestion,
            })
            .collect();

        Ok(crate::client::client_trait::ValidationResult {
            valid: validate_resp.valid,
            errors,
            warnings,
            estimated_cost: validate_resp.estimated_cost,
        })
    }

    async fn get_config(&self) -> Result<crate::client::client_trait::ServerConfig> {
        let url = format!("{}/config", self.base_url);

        let response = self.inner.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to get config ({}): {}",
                status, body
            )));
        }

        let config_resp: ServerConfigResponse = response.json().await?;

        let sections = config_resp
            .sections
            .into_iter()
            .map(|s| crate::client::client_trait::ConfigSection {
                name: s.name,
                description: s.description,
                items: s
                    .items
                    .into_iter()
                    .map(|i| crate::client::client_trait::ConfigItem {
                        key: i.key,
                        value: i.value,
                        default_value: i.default_value,
                        description: i.description,
                        mutable: i.mutable,
                    })
                    .collect(),
            })
            .collect();

        Ok(crate::client::client_trait::ServerConfig {
            version: config_resp.version,
            sections,
        })
    }

    async fn update_config(
        &self,
        section: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<()> {
        let url = format!("{}/config", self.base_url);

        let request = UpdateConfigRequest {
            section: section.to_string(),
            key: key.to_string(),
            value,
        };

        let response = self.inner.put(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to update config ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn create_vector_index(
        &self,
        space: &str,
        name: &str,
        tag: &str,
        field: &str,
        dimension: usize,
        metric: &str,
    ) -> Result<()> {
        let url = format!("{}/schema/spaces/{}/vector-indexes", self.base_url, space);

        let request = CreateVectorIndexRequest {
            name: name.to_string(),
            tag: tag.to_string(),
            field: field.to_string(),
            dimension,
            metric: metric.to_string(),
        };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to create vector index ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn drop_vector_index(&self, space: &str, name: &str) -> Result<()> {
        let url = format!(
            "{}/schema/spaces/{}/vector-indexes/{}",
            self.base_url, space, name
        );

        let response = self.inner.delete(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to drop vector index ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    async fn vector_search(
        &self,
        space: &str,
        index_name: &str,
        vector: Vec<f32>,
        top_k: usize,
    ) -> Result<crate::client::client_trait::VectorSearchResult> {
        let url = format!(
            "{}/schema/spaces/{}/vector-indexes/{}/search",
            self.base_url, space, index_name
        );

        let request = VectorSearchRequest { vector, top_k };

        let response = self.inner.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to search vectors ({}): {}",
                status, body
            )));
        }

        let search_resp: VectorSearchResponse = response.json().await?;

        let results = search_resp
            .results
            .into_iter()
            .map(|r| crate::client::client_trait::VectorMatch {
                vid: r.vid,
                score: r.score,
                properties: r.properties,
            })
            .collect();

        Ok(crate::client::client_trait::VectorSearchResult {
            total: search_resp.total,
            results,
        })
    }
}

// Legacy GraphDBHttpClient for backward compatibility
// Deprecated: Use HttpClient instead
pub struct GraphDBHttpClient {
    client: reqwest::Client,
    base_url: String,
}

impl GraphDBHttpClient {
    pub fn new(host: &str, port: u16) -> Self {
        let base_url = format!("http://{}:{}/v1", host, port);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self { client, base_url }
    }

    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await;
        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<(i64, String)> {
        let url = format!("{}/auth/login", self.base_url);
        let request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::auth(format!(
                "Login failed ({}): {}",
                status, body
            )));
        }

        let login_resp: LoginResponse = response.json().await?;
        Ok((login_resp.session_id, login_resp.username))
    }

    pub async fn create_session(&self, username: &str) -> Result<i64> {
        let url = format!("{}/sessions", self.base_url);
        let request = CreateSessionRequest {
            username: username.to_string(),
            client_ip: "127.0.0.1".to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::session(format!(
                "Failed to create session ({}): {}",
                status, body
            )));
        }

        let session_resp: SessionResponse = response.json().await?;
        Ok(session_resp.session_id)
    }

    pub async fn execute_query(&self, query: &str, session_id: i64) -> Result<QueryResult> {
        let url = format!("{}/query", self.base_url);
        let request = QueryRequest {
            query: query.to_string(),
            session_id,
            parameters: HashMap::new(),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Query failed ({}): {}",
                status, body
            )));
        }

        let query_resp: QueryResponse = response.json().await?;

        if !query_resp.success {
            let err = query_resp.error.unwrap_or(QueryError {
                code: "UNKNOWN".to_string(),
                message: "Unknown error".to_string(),
                details: None,
            });
            return Err(CliError::query(format!("{}: {}", err.code, err.message)));
        }

        let data = query_resp.data.unwrap_or(QueryData {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 0,
        });

        let metadata = query_resp.metadata.unwrap_or(QueryMetadata {
            execution_time_ms: 0,
            rows_scanned: 0,
            rows_returned: 0,
            space_id: None,
        });

        Ok(QueryResult {
            columns: data.columns,
            rows: data.rows,
            row_count: data.row_count,
            execution_time_ms: metadata.execution_time_ms,
            rows_scanned: metadata.rows_scanned,
            error: None,
        })
    }

    pub async fn list_spaces(&self) -> Result<Vec<SpaceInfo>> {
        let url = format!("{}/schema/spaces", self.base_url);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to list spaces ({}): {}",
                status, body
            )));
        }

        let body: serde_json::Value = response.json().await?;
        let spaces = body
            .get("spaces")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(spaces)
    }

    pub async fn use_space(&self, space_name: &str) -> Result<()> {
        let url = format!("{}/schema/spaces/{}", self.base_url, space_name);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to use space '{}' ({}): {}",
                space_name, status, body
            )));
        }

        Ok(())
    }

    pub async fn list_tags(&self, space_name: &str) -> Result<Vec<TagInfo>> {
        let url = format!("{}/schema/spaces/{}/tags", self.base_url, space_name);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to list tags ({}): {}",
                status, body
            )));
        }

        let body: serde_json::Value = response.json().await?;
        let tags = body
            .get("tags")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(tags)
    }

    pub async fn list_edge_types(&self, space_name: &str) -> Result<Vec<EdgeTypeInfo>> {
        let url = format!("{}/schema/spaces/{}/edge-types", self.base_url, space_name);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(CliError::query(format!(
                "Failed to list edge types ({}): {}",
                status, body
            )));
        }

        let body: serde_json::Value = response.json().await?;
        let edge_types = body
            .get("edge_types")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(edge_types)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

// Request/Response types
#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Debug, Serialize)]
struct LogoutRequest {
    session_id: i64,
}

#[derive(Debug, Serialize)]
struct BeginTransactionRequest {
    session_id: i64,
    read_only: bool,
    timeout_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TransactionResponse {
    transaction_id: u64,
    status: String,
}

#[derive(Debug, Serialize)]
struct TransactionActionRequest {
    session_id: i64,
}

// Schema DDL request/response types
#[derive(Debug, Serialize)]
struct CreateSpaceRequest {
    name: String,
    vid_type: Option<String>,
    comment: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateTagRequest {
    name: String,
    properties: Vec<PropertyDefInput>,
}

#[derive(Debug, Serialize)]
struct CreateEdgeTypeRequest {
    name: String,
    properties: Vec<PropertyDefInput>,
}

#[derive(Debug, Serialize)]
struct PropertyDefInput {
    name: String,
    data_type: String,
    nullable: bool,
}

// Batch operation request/response types
#[derive(Debug, Serialize)]
struct CreateBatchRequest {
    space_id: u64,
    batch_type: String,
    batch_size: usize,
}

#[derive(Debug, Deserialize)]
struct CreateBatchResponse {
    batch_id: String,
}

#[derive(Debug, Serialize)]
struct AddBatchItemsRequest {
    items: Vec<BatchItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
enum BatchItem {
    #[serde(rename = "vertex")]
    Vertex(VertexData),
    #[serde(rename = "edge")]
    Edge(EdgeData),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct VertexData {
    vid: serde_json::Value,
    tags: Vec<String>,
    properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct EdgeData {
    edge_type: String,
    src_vid: serde_json::Value,
    dst_vid: serde_json::Value,
    properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct AddBatchItemsResponse {
    accepted: usize,
}

#[derive(Debug, Deserialize)]
struct ExecuteBatchResponse {
    batch_id: String,
    status: BatchStatusEnum,
    result: BatchResultData,
}

#[derive(Debug, Deserialize)]
struct BatchResultData {
    vertices_inserted: usize,
    edges_inserted: usize,
    errors: Vec<BatchErrorData>,
}

#[derive(Debug, Deserialize)]
struct BatchErrorData {
    index: usize,
    item_type: BatchItemType,
    error: String,
}

#[derive(Debug, Deserialize)]
enum BatchItemType {
    Vertex,
    Edge,
}

#[derive(Debug, Deserialize)]
struct BatchStatusResponse {
    batch_id: String,
    status: BatchStatusEnum,
    progress: BatchProgress,
}

#[derive(Debug, Deserialize)]
enum BatchStatusEnum {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Deserialize)]
struct BatchProgress {
    total: usize,
    processed: usize,
    succeeded: usize,
    failed: usize,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoginResponse {
    session_id: i64,
    username: String,
    #[serde(default)]
    expires_at: Option<u64>,
}

#[derive(Debug, Serialize)]
struct CreateSessionRequest {
    username: String,
    client_ip: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SessionResponse {
    session_id: i64,
    username: String,
    created_at: u64,
}

#[derive(Debug, Serialize)]
struct QueryRequest {
    query: String,
    session_id: i64,
    #[serde(default)]
    parameters: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct QueryResponse {
    success: bool,
    data: Option<QueryData>,
    error: Option<QueryError>,
    metadata: Option<QueryMetadata>,
}

#[derive(Debug, Deserialize)]
struct QueryData {
    columns: Vec<String>,
    rows: Vec<HashMap<String, serde_json::Value>>,
    row_count: usize,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct QueryError {
    code: String,
    message: String,
    #[serde(default)]
    details: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct QueryMetadata {
    #[serde(default)]
    execution_time_ms: u64,
    #[serde(default)]
    rows_scanned: u64,
    #[serde(default)]
    rows_returned: usize,
    #[serde(default)]
    space_id: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HealthResponse {
    status: String,
    service: String,
    version: String,
}

// Public data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceInfo {
    pub id: u64,
    pub name: String,
    pub vid_type: String,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTypeInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    #[serde(default)]
    pub default_value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub row_count: usize,
    pub execution_time_ms: u64,
    pub rows_scanned: u64,
    pub error: Option<QueryErrorInfo>,
}

#[derive(Debug, Clone)]
pub struct QueryErrorInfo {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

// Query validation request/response types
#[derive(Debug, Serialize)]
struct ValidateQueryRequest {
    query: String,
    session_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ValidateQueryResponse {
    valid: bool,
    errors: Vec<ValidationErrorData>,
    warnings: Vec<ValidationWarningData>,
    estimated_cost: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ValidationErrorData {
    code: String,
    message: String,
    position: Option<usize>,
    line: Option<usize>,
    column: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct ValidationWarningData {
    code: String,
    message: String,
    suggestion: Option<String>,
}

// Configuration request/response types
#[derive(Debug, Deserialize)]
struct ServerConfigResponse {
    version: String,
    sections: Vec<ConfigSectionData>,
}

#[derive(Debug, Deserialize)]
struct ConfigSectionData {
    name: String,
    description: Option<String>,
    items: Vec<ConfigItemData>,
}

#[derive(Debug, Deserialize)]
struct ConfigItemData {
    key: String,
    value: serde_json::Value,
    default_value: Option<serde_json::Value>,
    description: Option<String>,
    mutable: bool,
}

#[derive(Debug, Serialize)]
struct UpdateConfigRequest {
    section: String,
    key: String,
    value: serde_json::Value,
}

// Vector operations request/response types
#[derive(Debug, Serialize)]
struct CreateVectorIndexRequest {
    name: String,
    tag: String,
    field: String,
    dimension: usize,
    metric: String,
}

#[derive(Debug, Serialize)]
struct VectorSearchRequest {
    vector: Vec<f32>,
    top_k: usize,
}

#[derive(Debug, Deserialize)]
struct VectorSearchResponse {
    total: usize,
    results: Vec<VectorMatchData>,
}

#[derive(Debug, Deserialize)]
struct VectorMatchData {
    vid: serde_json::Value,
    score: f32,
    properties: HashMap<String, serde_json::Value>,
}
