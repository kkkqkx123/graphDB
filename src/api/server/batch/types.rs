//! 批量操作类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 批量任务ID
pub type BatchId = String;

/// 批量任务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    /// 已创建
    Created,
    /// 运行中
    Running,
    /// 已完成
    Completed,
    /// 已失败
    Failed,
    /// 已取消
    Cancelled,
}

/// 批量任务类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchType {
    /// 顶点批量插入
    Vertex,
    /// 边批量插入
    Edge,
    /// 混合批量插入
    Mixed,
}

/// 批量项类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchItemType {
    /// 顶点
    Vertex,
    /// 边
    Edge,
}

/// 批量项数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum BatchItem {
    #[serde(rename = "vertex")]
    Vertex(VertexData),
    #[serde(rename = "edge")]
    Edge(EdgeData),
}

/// 顶点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexData {
    /// 顶点ID
    pub vid: serde_json::Value,
    /// 标签列表
    #[serde(default)]
    pub tags: Vec<String>,
    /// 属性
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// 边数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    /// 边类型
    pub edge_type: String,
    /// 起始顶点ID
    pub src_vid: serde_json::Value,
    /// 目标顶点ID
    pub dst_vid: serde_json::Value,
    /// 属性
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// 创建批量任务请求
#[derive(Debug, Clone, Deserialize)]
pub struct CreateBatchRequest {
    /// 图空间ID
    pub space_id: u64,
    /// 批量任务类型
    pub batch_type: BatchType,
    /// 批次大小
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_batch_size() -> usize {
    1000
}

/// 创建批量任务响应
#[derive(Debug, Clone, Serialize)]
pub struct CreateBatchResponse {
    /// 批量任务ID
    pub batch_id: BatchId,
    /// 任务状态
    pub status: BatchStatus,
    /// 创建时间
    pub created_at: String,
}

/// 添加批量项请求
#[derive(Debug, Clone, Deserialize)]
pub struct AddBatchItemsRequest {
    /// 批量项列表
    pub items: Vec<BatchItem>,
}

/// 添加批量项响应
#[derive(Debug, Clone, Serialize)]
pub struct AddBatchItemsResponse {
    /// 已接受数量
    pub accepted: usize,
    /// 已缓冲数量
    pub buffered: usize,
    /// 总缓冲数量
    pub total_buffered: usize,
}

/// 执行批量任务响应
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteBatchResponse {
    /// 批量任务ID
    pub batch_id: BatchId,
    /// 任务状态
    pub status: BatchStatus,
    /// 执行结果
    pub result: BatchResultData,
    /// 完成时间
    pub completed_at: Option<String>,
}

/// 批量结果数据
#[derive(Debug, Clone, Serialize)]
pub struct BatchResultData {
    /// 插入的顶点数量
    pub vertices_inserted: usize,
    /// 插入的边数量
    pub edges_inserted: usize,
    /// 错误列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<BatchErrorData>,
}

/// 批量错误数据
#[derive(Debug, Clone, Serialize)]
pub struct BatchErrorData {
    /// 错误发生的索引
    pub index: usize,
    /// 错误项类型
    pub item_type: BatchItemType,
    /// 错误信息
    pub error: String,
}

/// 批量任务状态响应
#[derive(Debug, Clone, Serialize)]
pub struct BatchStatusResponse {
    /// 批量任务ID
    pub batch_id: BatchId,
    /// 任务状态
    pub status: BatchStatus,
    /// 进度信息
    pub progress: BatchProgress,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
}

/// 批量任务进度
#[derive(Debug, Clone, Serialize)]
pub struct BatchProgress {
    /// 总数量
    pub total: usize,
    /// 已处理数量
    pub processed: usize,
    /// 成功数量
    pub succeeded: usize,
    /// 失败数量
    pub failed: usize,
    /// 缓冲数量
    pub buffered: usize,
}

/// 批量任务信息（内部使用）
#[derive(Debug, Clone)]
pub struct BatchTask {
    /// 任务ID
    pub id: BatchId,
    /// 图空间ID
    pub space_id: u64,
    /// 任务类型
    pub batch_type: BatchType,
    /// 批次大小
    pub batch_size: usize,
    /// 任务状态
    pub status: BatchStatus,
    /// 缓冲的项
    pub buffered_items: Vec<BatchItem>,
    /// 进度
    pub progress: BatchProgress,
    /// 结果
    pub result: Option<BatchResultData>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl BatchTask {
    /// 创建新的批量任务
    pub fn new(id: BatchId, space_id: u64, batch_type: BatchType, batch_size: usize) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            space_id,
            batch_type,
            batch_size,
            status: BatchStatus::Created,
            buffered_items: Vec::new(),
            progress: BatchProgress {
                total: 0,
                processed: 0,
                succeeded: 0,
                failed: 0,
                buffered: 0,
            },
            result: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新状态
    pub fn update_status(&mut self, status: BatchStatus) {
        self.status = status;
        self.updated_at = chrono::Utc::now();
    }

    /// 添加缓冲项
    pub fn add_items(&mut self, items: Vec<BatchItem>) -> usize {
        let count = items.len();
        self.buffered_items.extend(items);
        self.progress.buffered = self.buffered_items.len();
        self.progress.total += count;
        self.updated_at = chrono::Utc::now();
        count
    }

    /// 获取并清空缓冲项
    pub fn take_buffered_items(&mut self) -> Vec<BatchItem> {
        let items = std::mem::take(&mut self.buffered_items);
        self.progress.buffered = 0;
        self.updated_at = chrono::Utc::now();
        items
    }

    /// 更新进度
    pub fn update_progress(&mut self, succeeded: usize, failed: usize) {
        self.progress.succeeded += succeeded;
        self.progress.failed += failed;
        self.progress.processed += succeeded + failed;
        self.updated_at = chrono::Utc::now();
    }

    /// 设置结果
    pub fn set_result(&mut self, result: BatchResultData) {
        self.result = Some(result);
        self.updated_at = chrono::Utc::now();
    }
}
