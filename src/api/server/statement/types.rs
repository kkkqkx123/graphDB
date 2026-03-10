//! 预编译语句类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 预编译语句ID
pub type StatementId = String;

/// 创建预编译语句请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateStatementRequest {
    /// 查询语句
    pub query: String,
    /// 图空间ID
    pub space_id: u64,
}

/// 创建预编译语句响应
#[derive(Debug, Clone, Serialize)]
pub struct CreateStatementResponse {
    /// 语句ID
    pub statement_id: StatementId,
    /// 参数列表
    pub parameters: Vec<String>,
    /// 创建时间
    pub created_at: String,
}

/// 执行预编译语句请求
#[derive(Debug, Clone, Deserialize)]
pub struct ExecuteStatementRequest {
    /// 参数映射
    pub parameters: HashMap<String, serde_json::Value>,
}

/// 执行预编译语句响应
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteStatementResponse {
    /// 查询数据
    pub data: Option<StatementQueryData>,
    /// 元数据
    pub metadata: StatementMetadata,
}

/// 语句查询数据
#[derive(Debug, Clone, Serialize)]
pub struct StatementQueryData {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub row_count: usize,
}

/// 语句元数据
#[derive(Debug, Clone, Serialize)]
pub struct StatementMetadata {
    pub execution_time_ms: u64,
    pub rows_returned: usize,
}

/// 批量执行预编译语句请求
#[derive(Debug, Clone, Deserialize)]
pub struct BatchExecuteStatementRequest {
    /// 参数列表
    pub batch_parameters: Vec<HashMap<String, serde_json::Value>>,
}

/// 批量执行预编译语句响应
#[derive(Debug, Clone, Serialize)]
pub struct BatchExecuteStatementResponse {
    /// 结果列表
    pub results: Vec<ExecuteStatementResponse>,
    /// 摘要
    pub summary: BatchSummary,
}

/// 批量执行摘要
#[derive(Debug, Clone, Serialize)]
pub struct BatchSummary {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

/// 预编译语句信息响应
#[derive(Debug, Clone, Serialize)]
pub struct StatementInfoResponse {
    /// 语句ID
    pub statement_id: StatementId,
    /// 查询语句
    pub query: String,
    /// 参数列表
    pub parameters: Vec<String>,
    /// 执行次数
    pub execution_count: u64,
    /// 平均执行时间（毫秒）
    pub avg_execution_time_ms: f64,
    /// 创建时间
    pub created_at: String,
    /// 最后使用时间
    pub last_used_at: String,
}

/// 预编译语句信息（内部使用）
#[derive(Debug, Clone)]
pub struct StatementInfo {
    /// 语句ID
    pub id: StatementId,
    /// 查询语句
    pub query: String,
    /// 参数列表
    pub parameters: Vec<String>,
    /// 图空间ID
    pub space_id: u64,
    /// 执行次数
    pub execution_count: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 最后使用时间
    pub last_used_at: chrono::DateTime<chrono::Utc>,
}

impl StatementInfo {
    /// 创建新的语句信息
    pub fn new(id: StatementId, query: String, space_id: u64, parameters: Vec<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            query,
            space_id,
            parameters,
            execution_count: 0,
            total_execution_time_ms: 0,
            created_at: now,
            last_used_at: now,
        }
    }

    /// 记录执行
    pub fn record_execution(&mut self, execution_time_ms: u64) {
        self.execution_count += 1;
        self.total_execution_time_ms += execution_time_ms;
        self.last_used_at = chrono::Utc::now();
    }

    /// 获取平均执行时间
    pub fn avg_execution_time_ms(&self) -> f64 {
        if self.execution_count == 0 {
            0.0
        } else {
            self.total_execution_time_ms as f64 / self.execution_count as f64
        }
    }

    /// 转换为响应
    pub fn to_response(&self) -> StatementInfoResponse {
        StatementInfoResponse {
            statement_id: self.id.clone(),
            query: self.query.clone(),
            parameters: self.parameters.clone(),
            execution_count: self.execution_count,
            avg_execution_time_ms: self.avg_execution_time_ms(),
            created_at: self.created_at.to_rfc3339(),
            last_used_at: self.last_used_at.to_rfc3339(),
        }
    }
}
