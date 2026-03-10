//! 查询结果类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 查询请求
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    pub session_id: i64,
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}

/// 查询响应（结构化）
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub success: bool,
    pub data: Option<QueryData>,
    pub error: Option<QueryError>,
    pub metadata: QueryMetadata,
}

/// 查询数据
#[derive(Debug, Serialize)]
pub struct QueryData {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub row_count: usize,
}

/// 查询元数据
#[derive(Debug, Serialize)]
pub struct QueryMetadata {
    pub execution_time_ms: u64,
    pub rows_scanned: u64,
    pub rows_returned: usize,
    pub space_id: Option<u64>,
}

/// 查询错误
#[derive(Debug, Serialize)]
pub struct QueryError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

/// 验证响应
#[derive(Debug, Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub message: String,
}

impl QueryResponse {
    /// 创建成功响应
    pub fn success(data: QueryData, metadata: QueryMetadata) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata,
        }
    }

    /// 创建错误响应
    pub fn error(code: String, message: String, details: Option<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(QueryError {
                code,
                message,
                details,
            }),
            metadata: QueryMetadata {
                execution_time_ms: 0,
                rows_scanned: 0,
                rows_returned: 0,
                space_id: None,
            },
        }
    }
}

impl QueryData {
    /// 创建空的查询数据
    pub fn empty() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            row_count: 0,
        }
    }

    /// 从列和行创建查询数据
    pub fn new(columns: Vec<String>, rows: Vec<HashMap<String, serde_json::Value>>) -> Self {
        let row_count = rows.len();
        Self {
            columns,
            rows,
            row_count,
        }
    }
}
