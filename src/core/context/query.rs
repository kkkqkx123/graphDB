//! 查询上下文定义
//!
//! 提供查询执行过程中的上下文管理

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::types::query::{QueryType, QueryResult};
use crate::core::Value;
use super::base::{ContextBase, ContextType, MutableContext};
use super::session::SessionInfo;

/// 查询上下文
#[derive(Debug, Clone)]
pub struct QueryContext {
    /// 查询ID
    pub query_id: String,
    /// 查询类型
    pub query_type: QueryType,
    /// 查询语句
    pub query_text: String,
    /// 查询参数
    pub parameters: HashMap<String, QueryParameter>,
    /// 查询选项
    pub options: QueryOptions,
    /// 会话信息
    pub session_info: SessionInfo,
    /// 开始时间
    pub start_time: std::time::Instant,
}

/// 查询参数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryParameter {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

/// 查询选项
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryOptions {
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 是否启用 profiling
    pub enable_profiling: bool,
    /// 最大返回行数
    pub max_rows: Option<usize>,
    /// 是否跳过验证
    pub skip_validation: bool,
    /// 是否只读模式
    pub read_only: bool,
}

// SessionInfo 现在从 session 模块导入

// 会话变量（简化版本）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionVariable {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}

impl QueryContext {
    /// 创建新的查询上下文
    pub fn new(
        query_id: impl Into<String>,
        query_type: QueryType,
        query_text: impl Into<String>,
        session_info: SessionInfo,
    ) -> Self {
        Self {
            query_id: query_id.into(),
            query_type,
            query_text: query_text.into(),
            parameters: HashMap::new(),
            options: QueryOptions::default(),
            session_info,
            start_time: std::time::Instant::now(),
        }
    }
    
    /// 添加查询参数
    pub fn add_parameter(&mut self, name: impl Into<String>, value: QueryParameter) {
        self.parameters.insert(name.into(), value);
    }
    
    /// 获取查询参数
    pub fn get_parameter(&self, name: &str) -> Option<&QueryParameter> {
        self.parameters.get(name)
    }
    
    /// 设置查询选项
    pub fn set_options(&mut self, options: QueryOptions) {
        self.options = options;
    }
    
    /// 获取执行时间（毫秒）
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
    
    /// 检查是否超时
    pub fn is_timeout(&self) -> bool {
        if let Some(timeout_ms) = self.options.timeout_ms {
            self.elapsed_ms() > timeout_ms
        } else {
            false
        }
    }
    
    /// 检查用户是否有指定角色
    pub fn has_role(&self, role: &str) -> bool {
        self.session_info.roles.contains(&role.to_string())
    }
}

impl ContextBase for QueryContext {
    fn id(&self) -> &str {
        &self.query_id
    }

    fn context_type(&self) -> ContextType {
        ContextType::Query
    }

    fn created_at(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH + std::time::Duration::from_millis(0) // 使用固定时间作为创建时间
    }

    fn updated_at(&self) -> std::time::SystemTime {
        std::time::UNIX_EPOCH + std::time::Duration::from_millis(0) // 使用固定时间作为更新时间
    }

    fn is_valid(&self) -> bool {
        !self.is_timeout()
    }
}

impl MutableContext for QueryContext {
    fn touch(&mut self) {
        // 更新时间戳
    }

    fn invalidate(&mut self) {
        // 标记为无效
    }

    fn revalidate(&mut self) -> bool {
        !self.is_timeout()
    }
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            timeout_ms: Some(30000), // 默认30秒超时
            enable_profiling: false,
            max_rows: None,
            skip_validation: false,
            read_only: false,
        }
    }
}


/// 查询状态
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryStatus {
    /// 准备中
    Preparing,
    /// 验证中
    Validating,
    /// 优化中
    Optimizing,
    /// 执行中
    Executing,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 错误
    Error,
}

/// 查询统计信息
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueryStatistics {
    /// 查询状态
    pub status: QueryStatus,
    /// 开始时间
    pub start_time: std::time::SystemTime,
    /// 结束时间
    pub end_time: Option<std::time::SystemTime>,
    /// 执行计划
    pub execution_plan: Option<String>,
    /// 扫描的顶点数
    pub vertices_scanned: usize,
    /// 扫描的边数
    pub edges_scanned: usize,
    /// 返回的行数
    pub rows_returned: usize,
    /// 内存使用量（字节）
    pub memory_used_bytes: usize,
    /// 错误信息
    pub error_message: Option<String>,
}

impl QueryStatistics {
    /// 创建新的查询统计信息
    pub fn new() -> Self {
        Self {
            status: QueryStatus::Preparing,
            start_time: std::time::SystemTime::now(),
            end_time: None,
            execution_plan: None,
            vertices_scanned: 0,
            edges_scanned: 0,
            rows_returned: 0,
            memory_used_bytes: 0,
            error_message: None,
        }
    }
    
    /// 设置状态
    pub fn set_status(&mut self, status: QueryStatus) {
        self.status = status.clone();
        if matches!(status, QueryStatus::Completed | QueryStatus::Cancelled | QueryStatus::Error) {
            self.end_time = Some(std::time::SystemTime::now());
        }
    }
    
    /// 设置执行计划
    pub fn set_execution_plan(&mut self, plan: impl Into<String>) {
        self.execution_plan = Some(plan.into());
    }
    
    /// 增加扫描的顶点数
    pub fn add_vertices_scanned(&mut self, count: usize) {
        self.vertices_scanned += count;
    }
    
    /// 增加扫描的边数
    pub fn add_edges_scanned(&mut self, count: usize) {
        self.edges_scanned += count;
    }
    
    /// 设置返回的行数
    pub fn set_rows_returned(&mut self, count: usize) {
        self.rows_returned = count;
    }
    
    /// 设置内存使用量
    pub fn set_memory_used(&mut self, bytes: usize) {
        self.memory_used_bytes = bytes;
    }
    
    /// 设置错误信息
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error_message = Some(error.into());
        self.set_status(QueryStatus::Error);
    }
    
    /// 获取执行时间（毫秒）
    pub fn execution_time_ms(&self) -> Option<u64> {
        self.end_time.and_then(|end| {
            end.duration_since(self.start_time)
                .ok()
                .map(|duration| duration.as_millis() as u64)
        })
    }
    
    /// 检查是否已完成
    pub fn is_completed(&self) -> bool {
        matches!(self.status, QueryStatus::Completed | QueryStatus::Cancelled | QueryStatus::Error)
    }
}

impl Default for QueryStatistics {
    fn default() -> Self {
        Self::new()
    }
}