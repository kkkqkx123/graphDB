use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::utils::error::{CliError, Result};

#[derive(Debug, Clone)]
pub struct GraphDBHttpClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct LoginRequest {
    username: String,
    password: String,
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
