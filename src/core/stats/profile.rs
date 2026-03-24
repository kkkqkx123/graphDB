//! 查询画像和执行器统计
//!
//! 用于详细监控和分析的查询画像，使用毫秒级精度。

use std::time::Instant;

use super::error_stats::{ErrorInfo, ErrorType, QueryPhase};

/// 查询执行阶段统计（毫秒级）
#[derive(Debug, Clone, Default)]
pub struct StageMetrics {
    pub parse_ms: u64,
    pub validate_ms: u64,
    pub plan_ms: u64,
    pub optimize_ms: u64,
    pub execute_ms: u64,
}

/// 执行器统计
#[derive(Debug, Clone)]
pub struct ExecutorStat {
    pub executor_type: String,
    pub executor_id: i64,
    pub duration_ms: u64,
    pub rows_processed: usize,
    pub memory_used: usize,
}

/// 查询状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStatus {
    Success,
    Failed,
}

/// 查询画像
///
/// 用于详细监控和分析的查询画像，使用毫秒级精度。
/// 与 QueryMetrics 的区别：
/// - QueryProfile: 详细监控，用于内部分析和日志（毫秒级）
/// - QueryMetrics: 轻量级，用于返回给客户端（微秒级）
#[derive(Debug, Clone)]
pub struct QueryProfile {
    pub trace_id: String,
    pub session_id: i64,
    pub query_text: String,
    pub start_time: Instant,
    pub total_duration_ms: u64,
    pub stages: StageMetrics,
    pub executor_stats: Vec<ExecutorStat>,
    pub result_count: usize,
    pub status: QueryStatus,
    pub error_message: Option<String>,
    pub error_info: Option<ErrorInfo>,
}

impl QueryProfile {
    pub fn new(session_id: i64, query_text: String) -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            session_id,
            query_text,
            start_time: Instant::now(),
            total_duration_ms: 0,
            stages: StageMetrics::default(),
            executor_stats: Vec::new(),
            result_count: 0,
            status: QueryStatus::Success,
            error_message: None,
            error_info: None,
        }
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = QueryStatus::Failed;
        self.error_message = Some(error);
    }

    pub fn mark_failed_with_info(&mut self, error_info: ErrorInfo) {
        self.status = QueryStatus::Failed;
        self.error_message = Some(error_info.error_message.clone());
        self.error_info = Some(error_info);
    }

    pub fn error_type(&self) -> Option<ErrorType> {
        self.error_info.as_ref().map(|e| e.error_type)
    }

    pub fn error_phase(&self) -> Option<QueryPhase> {
        self.error_info.as_ref().map(|e| e.error_phase)
    }

    pub fn add_executor_stat(&mut self, stat: ExecutorStat) {
        self.executor_stats.push(stat);
    }

    pub fn total_executor_time_ms(&self) -> u64 {
        self.executor_stats.iter().map(|s| s.duration_ms).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_profile_creation() {
        let profile = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());
        assert_eq!(profile.session_id, 123);
        assert_eq!(profile.query_text, "MATCH (n) RETURN n");
        assert!(!profile.trace_id.is_empty());
        assert!(matches!(profile.status, QueryStatus::Success));
    }

    #[test]
    fn test_query_profile_mark_failed() {
        let mut profile = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());
        profile.mark_failed("Syntax error".to_string());
        assert!(matches!(profile.status, QueryStatus::Failed));
        assert_eq!(profile.error_message, Some("Syntax error".to_string()));
    }

    #[test]
    fn test_query_profile_add_executor_stat() {
        let mut profile = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());
        let stat = ExecutorStat {
            executor_type: "ScanVerticesExecutor".to_string(),
            executor_id: 1,
            duration_ms: 100,
            rows_processed: 50,
            memory_used: 1024,
        };
        profile.add_executor_stat(stat);
        assert_eq!(profile.executor_stats.len(), 1);
        assert_eq!(profile.total_executor_time_ms(), 100);
    }

    #[test]
    fn test_stage_metrics_default() {
        let metrics = StageMetrics::default();
        assert_eq!(metrics.parse_ms, 0);
        assert_eq!(metrics.execute_ms, 0);
    }
}
