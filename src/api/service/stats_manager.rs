use dashmap::DashMap;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::time::Instant;

/// 查询执行阶段统计
#[derive(Debug, Clone)]
pub struct StageMetrics {
    pub parse_ms: u64,
    pub validate_ms: u64,
    pub plan_ms: u64,
    pub optimize_ms: u64,
    pub execute_ms: u64,
}

impl Default for StageMetrics {
    fn default() -> Self {
        Self {
            parse_ms: 0,
            validate_ms: 0,
            plan_ms: 0,
            optimize_ms: 0,
            execute_ms: 0,
        }
    }
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

/// 查询执行阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QueryPhase {
    Parse,
    Validate,
    Plan,
    Optimize,
    Execute,
}

impl std::fmt::Display for QueryPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryPhase::Parse => write!(f, "parse"),
            QueryPhase::Validate => write!(f, "validate"),
            QueryPhase::Plan => write!(f, "plan"),
            QueryPhase::Optimize => write!(f, "optimize"),
            QueryPhase::Execute => write!(f, "execute"),
        }
    }
}

/// 错误类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorType {
    ParseError,
    ValidationError,
    PlanningError,
    OptimizationError,
    ExecutionError,
    StorageError,
    TimeoutError,
    MemoryLimitError,
    PermissionError,
    SessionError,
    OtherError,
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::ParseError => write!(f, "parse_error"),
            ErrorType::ValidationError => write!(f, "validation_error"),
            ErrorType::PlanningError => write!(f, "planning_error"),
            ErrorType::OptimizationError => write!(f, "optimization_error"),
            ErrorType::ExecutionError => write!(f, "execution_error"),
            ErrorType::StorageError => write!(f, "storage_error"),
            ErrorType::TimeoutError => write!(f, "timeout_error"),
            ErrorType::MemoryLimitError => write!(f, "memory_limit_error"),
            ErrorType::PermissionError => write!(f, "permission_error"),
            ErrorType::SessionError => write!(f, "session_error"),
            ErrorType::OtherError => write!(f, "other_error"),
        }
    }
}

/// 扩展的错误信息
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub error_type: ErrorType,
    pub error_phase: QueryPhase,
    pub error_message: String,
    pub error_details: Option<String>,
}

impl ErrorInfo {
    pub fn new(error_type: ErrorType, error_phase: QueryPhase, error_message: impl Into<String>) -> Self {
        Self {
            error_type,
            error_phase,
            error_message: error_message.into(),
            error_details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.error_details = Some(details.into());
        self
    }
}

/// 查询画像
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
    /// 创建新的查询画像
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

    /// 标记为失败（兼容旧版本）
    pub fn mark_failed(&mut self, error: String) {
        self.status = QueryStatus::Failed;
        self.error_message = Some(error);
    }

    /// 标记为失败（带详细错误信息）
    pub fn mark_failed_with_info(&mut self, error_info: ErrorInfo) {
        self.status = QueryStatus::Failed;
        self.error_message = Some(error_info.error_message.clone());
        self.error_info = Some(error_info);
    }

    /// 获取错误类型
    pub fn error_type(&self) -> Option<ErrorType> {
        self.error_info.as_ref().map(|e| e.error_type)
    }

    /// 获取错误阶段
    pub fn error_phase(&self) -> Option<QueryPhase> {
        self.error_info.as_ref().map(|e| e.error_phase)
    }

    /// 添加执行器统计
    pub fn add_executor_stat(&mut self, stat: ExecutorStat) {
        self.executor_stats.push(stat);
    }

    /// 计算总执行器耗时
    pub fn total_executor_time_ms(&self) -> u64 {
        self.executor_stats.iter().map(|s| s.duration_ms).sum()
    }
}

#[derive(Debug, Clone)]
pub struct QueryMetrics {
    pub parse_time_us: u64,
    pub validate_time_us: u64,
    pub plan_time_us: u64,
    pub optimize_time_us: u64,
    pub execute_time_us: u64,
    pub total_time_us: u64,
    pub plan_node_count: usize,
    pub result_row_count: usize,
    pub timestamp: Instant,
}

impl Default for QueryMetrics {
    fn default() -> Self {
        Self {
            parse_time_us: 0,
            validate_time_us: 0,
            plan_time_us: 0,
            optimize_time_us: 0,
            execute_time_us: 0,
            total_time_us: 0,
            plan_node_count: 0,
            result_row_count: 0,
            timestamp: Instant::now(),
        }
    }
}

impl QueryMetrics {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn record_parse_time(&mut self, duration: Duration) {
        self.parse_time_us = duration.as_micros() as u64;
    }
    
    pub fn record_validate_time(&mut self, duration: Duration) {
        self.validate_time_us = duration.as_micros() as u64;
    }
    
    pub fn record_plan_time(&mut self, duration: Duration) {
        self.plan_time_us = duration.as_micros() as u64;
    }
    
    pub fn record_optimize_time(&mut self, duration: Duration) {
        self.optimize_time_us = duration.as_micros() as u64;
    }
    
    pub fn record_execute_time(&mut self, duration: Duration) {
        self.execute_time_us = duration.as_micros() as u64;
    }
    
    pub fn record_total_time(&mut self, duration: Duration) {
        self.total_time_us = duration.as_micros() as u64;
    }
    
    pub fn set_plan_node_count(&mut self, count: usize) {
        self.plan_node_count = count;
    }
    
    pub fn set_result_row_count(&mut self, count: usize) {
        self.result_row_count = count;
    }
    
    pub fn to_map(&self) -> HashMap<String, u64> {
        let mut map = HashMap::new();
        map.insert("parse_time_us".to_string(), self.parse_time_us);
        map.insert("validate_time_us".to_string(), self.validate_time_us);
        map.insert("plan_time_us".to_string(), self.plan_time_us);
        map.insert("optimize_time_us".to_string(), self.optimize_time_us);
        map.insert("execute_time_us".to_string(), self.execute_time_us);
        map.insert("total_time_us".to_string(), self.total_time_us);
        map.insert("plan_node_count".to_string(), self.plan_node_count as u64);
        map.insert("result_row_count".to_string(), self.result_row_count as u64);
        map
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricType {
    NumAuthFailedSessions,
    NumQueries,
    NumActiveQueries,
    QueryParseTimeUs,
    QueryValidateTimeUs,
    QueryPlanTimeUs,
    QueryOptimizeTimeUs,
    QueryExecuteTimeUs,
    QueryTotalTimeUs,
    QueryPlanNodeCount,
    QueryResultRowCount,
}

pub struct MetricValue {
    pub value: AtomicU64,
    pub timestamp: AtomicU64, // 存储为UNIX时间戳（秒）
}

impl MetricValue {
    pub fn new(value: u64) -> Self {
        let timestamp_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            value: AtomicU64::new(value),
            timestamp: AtomicU64::new(timestamp_secs),
        }
    }

    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();
    }

    pub fn add(&self, amount: u64) {
        self.value.fetch_add(amount, Ordering::Relaxed);
        self.update_timestamp();
    }

    pub fn decrement(&self) {
        let _ = self.value.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
            if v > 0 { Some(v - 1) } else { Some(0) }
        });
        self.update_timestamp();
    }

    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
        self.update_timestamp();
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn get_timestamp(&self) -> u64 {
        self.timestamp.load(Ordering::Relaxed)
    }

    fn update_timestamp(&self) {
        let timestamp_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.timestamp.store(timestamp_secs, Ordering::Relaxed);
    }
}

/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub memory_cache_size: usize,
    pub slow_query_threshold_ms: u64,
    pub slow_query_log_dir: String,
    pub slow_query_log_retention_days: u32,
}

/// 错误统计摘要
#[derive(Debug, Clone)]
pub struct ErrorSummary {
    pub total_errors: u64,
    pub errors_by_type: HashMap<ErrorType, u64>,
    pub errors_by_phase: HashMap<QueryPhase, u64>,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            memory_cache_size: 1000,
            slow_query_threshold_ms: 10000,
            slow_query_log_dir: "logs".to_string(),
            slow_query_log_retention_days: 7,
        }
    }
}

pub struct StatsManager {
    metrics: Arc<DashMap<MetricType, Arc<MetricValue>>>,
    space_metrics: Arc<DashMap<String, Arc<DashMap<MetricType, Arc<MetricValue>>>>>,
    last_query_metrics: Arc<Mutex<Option<QueryMetrics>>>,
    // 新增：查询画像内存缓存
    query_profiles: Arc<Mutex<VecDeque<QueryProfile>>>,
    // 新增：监控配置
    config: MonitoringConfig,
    // 新增：错误统计
    error_counts: Arc<DashMap<ErrorType, AtomicU64>>,
    error_by_phase: Arc<DashMap<QueryPhase, AtomicU64>>,
}

impl StatsManager {
    pub fn new() -> Self {
        Self::with_config(MonitoringConfig::default())
    }

    pub fn with_config(config: MonitoringConfig) -> Self {
        let cache_size = config.memory_cache_size;
        Self {
            metrics: Arc::new(DashMap::new()),
            space_metrics: Arc::new(DashMap::new()),
            last_query_metrics: Arc::new(Mutex::new(None)),
            query_profiles: Arc::new(Mutex::new(VecDeque::with_capacity(cache_size))),
            config,
            error_counts: Arc::new(DashMap::new()),
            error_by_phase: Arc::new(DashMap::new()),
        }
    }

    /// 记录查询画像
    pub fn record_query_profile(&self, profile: QueryProfile) {
        if !self.config.enabled {
            return;
        }

        // 检查是否是慢查询
        if profile.total_duration_ms >= self.config.slow_query_threshold_ms {
            self.write_slow_query_log(&profile);
        }

        // 保存到内存缓存
        let mut profiles = self.query_profiles.lock();
        if profiles.len() >= self.config.memory_cache_size {
            profiles.pop_front();
        }
        profiles.push_back(profile);
    }

    /// 写入慢查询日志
    fn write_slow_query_log(&self, profile: &QueryProfile) {
        let log_entry = serde_json::json!({
            "timestamp": chrono::Local::now().to_rfc3339(),
            "trace_id": &profile.trace_id,
            "session_id": profile.session_id,
            "query_text": &profile.query_text,
            "duration_ms": profile.total_duration_ms,
            "stages": {
                "parse_ms": profile.stages.parse_ms,
                "validate_ms": profile.stages.validate_ms,
                "plan_ms": profile.stages.plan_ms,
                "optimize_ms": profile.stages.optimize_ms,
                "execute_ms": profile.stages.execute_ms,
            },
            "result_count": profile.result_count,
            "status": match profile.status {
                QueryStatus::Success => "success",
                QueryStatus::Failed => "failed",
            },
            "error": &profile.error_message,
        });

        let log_line = format!("{}\n", log_entry.to_string());
        
        // 确保日志目录存在
        let log_dir = std::path::Path::new(&self.config.slow_query_log_dir);
        if let Err(e) = std::fs::create_dir_all(log_dir) {
            log::warn!("Failed to create slow query log directory: {}", e);
            return;
        }

        // 按天轮转日志文件
        let date = chrono::Local::now().format("%Y-%m-%d");
        let log_file = log_dir.join(format!("slow_queries_{}.log", date));
        
        if let Err(e) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)
            .and_then(|mut file| {
                use std::io::Write;
                file.write_all(log_line.as_bytes())
            })
        {
            log::warn!("Failed to write slow query log: {}", e);
        }
    }

    /// 获取最近的查询画像
    pub fn get_recent_queries(&self, limit: usize) -> Vec<QueryProfile> {
        let profiles = self.query_profiles.lock();
        profiles.iter().rev().take(limit).cloned().collect()
    }

    /// 获取慢查询列表（从内存缓存中筛选）
    pub fn get_slow_queries(&self, limit: usize) -> Vec<QueryProfile> {
        let profiles = self.query_profiles.lock();
        profiles
            .iter()
            .filter(|p| p.total_duration_ms >= self.config.slow_query_threshold_ms)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// 获取指定查询的画像
    pub fn get_query_profile(&self, trace_id: &str) -> Option<QueryProfile> {
        let profiles = self.query_profiles.lock();
        profiles.iter().find(|p| p.trace_id == trace_id).cloned()
    }

    /// 按会话ID获取查询
    pub fn get_session_queries(&self, session_id: i64, limit: usize) -> Vec<QueryProfile> {
        let profiles = self.query_profiles.lock();
        profiles
            .iter()
            .filter(|p| p.session_id == session_id)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// 获取执行器统计汇总
    pub fn get_executor_stats_summary(&self) -> HashMap<String, (u64, u64, usize)> {
        let profiles = self.query_profiles.lock();
        let mut stats: HashMap<String, (u64, u64, usize)> = HashMap::new();

        for profile in profiles.iter() {
            for exec_stat in &profile.executor_stats {
                let entry = stats.entry(exec_stat.executor_type.clone()).or_insert((0, 0, 0));
                entry.0 += exec_stat.duration_ms;
                entry.1 += exec_stat.rows_processed as u64;
                entry.2 += 1;
            }
        }

        stats
    }

    /// 清空内存缓存
    pub fn clear_query_cache(&self) {
        let mut profiles = self.query_profiles.lock();
        profiles.clear();
    }

    /// 获取当前缓存的查询数量
    pub fn query_cache_size(&self) -> usize {
        let profiles = self.query_profiles.lock();
        profiles.len()
    }

    pub fn add_value(&self, metric_type: MetricType) {
        let metric = self.metrics.entry(metric_type).or_insert_with(|| {
            Arc::new(MetricValue::new(0))
        });
        metric.increment();
    }

    pub fn add_value_with_amount(&self, metric_type: MetricType, amount: u64) {
        let metric = self.metrics.entry(metric_type).or_insert_with(|| {
            Arc::new(MetricValue::new(0))
        });
        metric.add(amount);
    }

    pub fn dec_value(&self, metric_type: MetricType) {
        if let Some(metric) = self.metrics.get(&metric_type) {
            metric.decrement();
        }
    }

    pub fn add_space_metric(&self, space_name: &str, metric_type: MetricType) {
        let space_map = self.space_metrics.entry(space_name.to_string()).or_insert_with(|| {
            Arc::new(DashMap::new())
        });
        let metric = space_map.entry(metric_type).or_insert_with(|| {
            Arc::new(MetricValue::new(0))
        });
        metric.increment();
    }

    pub fn dec_space_metric(&self, space_name: &str, metric_type: MetricType) {
        if let Some(space_map) = self.space_metrics.get(space_name) {
            if let Some(metric) = space_map.get(&metric_type) {
                metric.decrement();
            }
        }
    }

    pub fn get_value(&self, metric_type: MetricType) -> Option<u64> {
        self.metrics.get(&metric_type).map(|metric| metric.get())
    }

    pub fn get_space_value(&self, space_name: &str, metric_type: MetricType) -> Option<u64> {
        self.space_metrics.get(space_name).and_then(|space_map| {
            space_map.get(&metric_type).map(|metric| metric.get())
        })
    }

    pub fn get_all_metrics(&self) -> HashMap<MetricType, u64> {
        self.metrics
            .iter()
            .map(|entry| (*entry.key(), entry.value().get()))
            .collect()
    }

    pub fn get_all_space_metrics(&self, space_name: &str) -> Option<HashMap<MetricType, u64>> {
        self.space_metrics.get(space_name).map(|space_map| {
            space_map
                .iter()
                .map(|entry| (*entry.key(), entry.value().get()))
                .collect()
        })
    }

    pub fn reset_metric(&self, metric_type: MetricType) {
        if let Some(metric) = self.metrics.get(&metric_type) {
            metric.set(0);
        }
    }

    pub fn reset_all_metrics(&self) {
        for metric in self.metrics.iter() {
            metric.value().set(0);
        }
    }

    pub fn reset_space_metrics(&self, space_name: &str) {
        if let Some(space_map) = self.space_metrics.get(space_name) {
            for metric in space_map.iter() {
                metric.value().set(0);
            }
        }
    }

    // 错误统计方法

    /// 记录错误
    pub fn record_error(&self, error_type: ErrorType, phase: QueryPhase) {
        let counter = self.error_counts.entry(error_type).or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::Relaxed);

        let phase_counter = self.error_by_phase.entry(phase).or_insert_with(|| AtomicU64::new(0));
        phase_counter.fetch_add(1, Ordering::Relaxed);

        log::warn!("查询错误: type={}, phase={}", error_type, phase);
    }

    /// 获取指定错误类型的计数
    pub fn get_error_count(&self, error_type: ErrorType) -> u64 {
        self.error_counts.get(&error_type).map(|c| c.load(Ordering::Relaxed)).unwrap_or(0)
    }

    /// 获取指定阶段的错误计数
    pub fn get_error_count_by_phase(&self, phase: QueryPhase) -> u64 {
        self.error_by_phase.get(&phase).map(|c| c.load(Ordering::Relaxed)).unwrap_or(0)
    }

    /// 获取所有错误统计
    pub fn get_all_error_counts(&self) -> HashMap<ErrorType, u64> {
        self.error_counts.iter().map(|entry| (*entry.key(), entry.value().load(Ordering::Relaxed))).collect()
    }

    /// 获取所有阶段的错误统计
    pub fn get_all_error_counts_by_phase(&self) -> HashMap<QueryPhase, u64> {
        self.error_by_phase.iter().map(|entry| (*entry.key(), entry.value().load(Ordering::Relaxed))).collect()
    }

    /// 重置错误统计
    pub fn reset_error_counts(&self) {
        for counter in self.error_counts.iter() {
            counter.value().store(0, Ordering::Relaxed);
        }
        for counter in self.error_by_phase.iter() {
            counter.value().store(0, Ordering::Relaxed);
        }
    }

    /// 记录失败的查询画像（带错误信息）
    pub fn record_failed_query(&self, mut profile: QueryProfile, error_info: ErrorInfo) {
        profile.mark_failed_with_info(error_info.clone());
        self.record_error(error_info.error_type, error_info.error_phase);
        self.record_query_profile(profile);
    }

    /// 获取错误统计摘要
    pub fn get_error_summary(&self) -> ErrorSummary {
        ErrorSummary {
            total_errors: self.error_counts.iter().map(|e| e.value().load(Ordering::Relaxed)).sum(),
            errors_by_type: self.get_all_error_counts(),
            errors_by_phase: self.get_all_error_counts_by_phase(),
        }
    }

    pub fn record_query_metrics(&self, metrics: &QueryMetrics) {
        let mut last_metrics = self.last_query_metrics.lock();
        *last_metrics = Some(metrics.clone());
        drop(last_metrics);
        
        let updates = [
            (MetricType::QueryParseTimeUs, metrics.parse_time_us),
            (MetricType::QueryValidateTimeUs, metrics.validate_time_us),
            (MetricType::QueryPlanTimeUs, metrics.plan_time_us),
            (MetricType::QueryOptimizeTimeUs, metrics.optimize_time_us),
            (MetricType::QueryExecuteTimeUs, metrics.execute_time_us),
            (MetricType::QueryTotalTimeUs, metrics.total_time_us),
            (MetricType::QueryPlanNodeCount, metrics.plan_node_count as u64),
            (MetricType::QueryResultRowCount, metrics.result_row_count as u64),
        ];
        
        for (metric_type, value) in updates {
            let metric = self.metrics.entry(metric_type).or_insert_with(|| {
                Arc::new(MetricValue::new(0))
            });
            metric.set(value);
        }
    }
    
    pub fn get_last_query_metrics(&self) -> Option<QueryMetrics> {
        let last_metrics = self.last_query_metrics.lock();
        last_metrics.clone()
    }
    
    pub fn get_query_metrics(&self) -> Option<QueryMetrics> {
        self.get_last_query_metrics()
    }
}

impl Default for StatsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_manager_creation() {
        let stats = StatsManager::new();
        // DashMap 是懒加载的，只有在第一次 add_value 时才会创建 metric
        assert_eq!(stats.get_value(MetricType::NumQueries), None);
        
        // 添加值后会创建 metric
        stats.add_value(MetricType::NumQueries);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(1));
    }

    #[test]
    fn test_add_value() {
        let stats = StatsManager::new();
        stats.add_value(MetricType::NumQueries);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(1));

        stats.add_value(MetricType::NumQueries);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(2));
    }

    #[test]
    fn test_add_value_with_amount() {
        let stats = StatsManager::new();
        stats.add_value_with_amount(MetricType::NumQueries, 5);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(5));

        stats.add_value_with_amount(MetricType::NumQueries, 3);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(8));
    }

    #[test]
    fn test_dec_value() {
        let stats = StatsManager::new();
        stats.add_value_with_amount(MetricType::NumQueries, 10);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(10));

        stats.dec_value(MetricType::NumQueries);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(9));

        stats.dec_value(MetricType::NumQueries);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(8));
    }

    #[test]
    fn test_dec_value_zero() {
        let stats = StatsManager::new();
        stats.add_value(MetricType::NumQueries);
        stats.dec_value(MetricType::NumQueries);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(0));

        stats.dec_value(MetricType::NumQueries);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(0));
    }

    #[test]
    fn test_space_metrics() {
        let stats = StatsManager::new();
        stats.add_space_metric("test_space", MetricType::NumQueries);
        assert_eq!(
            stats.get_space_value("test_space", MetricType::NumQueries),
            Some(1)
        );

        stats.add_space_metric("test_space", MetricType::NumQueries);
        assert_eq!(
            stats.get_space_value("test_space", MetricType::NumQueries),
            Some(2)
        );

        stats.add_space_metric("other_space", MetricType::NumQueries);
        assert_eq!(
            stats.get_space_value("other_space", MetricType::NumQueries),
            Some(1)
        );
    }

    #[test]
    fn test_dec_space_metric() {
        let stats = StatsManager::new();
        stats.add_space_metric("test_space", MetricType::NumQueries);
        stats.add_space_metric("test_space", MetricType::NumQueries);
        assert_eq!(
            stats.get_space_value("test_space", MetricType::NumQueries),
            Some(2)
        );

        stats.dec_space_metric("test_space", MetricType::NumQueries);
        assert_eq!(
            stats.get_space_value("test_space", MetricType::NumQueries),
            Some(1)
        );
    }

    #[test]
    fn test_get_all_metrics() {
        let stats = StatsManager::new();
        stats.add_value(MetricType::NumQueries);
        stats.add_value(MetricType::NumActiveQueries);

        let all_metrics = stats.get_all_metrics();
        assert_eq!(all_metrics.get(&MetricType::NumQueries), Some(&1));
        assert_eq!(all_metrics.get(&MetricType::NumActiveQueries), Some(&1));
    }

    #[test]
    fn test_get_all_space_metrics() {
        let stats = StatsManager::new();
        stats.add_space_metric("test_space", MetricType::NumQueries);
        stats.add_space_metric("test_space", MetricType::NumActiveQueries);

        let space_metrics = stats.get_all_space_metrics("test_space");
        assert!(space_metrics.is_some());
        let metrics = space_metrics.expect("空间指标应该存在");
        assert_eq!(metrics.get(&MetricType::NumQueries), Some(&1));
        assert_eq!(metrics.get(&MetricType::NumActiveQueries), Some(&1));
    }

    #[test]
    fn test_reset_metric() {
        let stats = StatsManager::new();
        stats.add_value_with_amount(MetricType::NumQueries, 10);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(10));

        stats.reset_metric(MetricType::NumQueries);
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(0));
    }

    #[test]
    fn test_reset_all_metrics() {
        let stats = StatsManager::new();
        stats.add_value_with_amount(MetricType::NumQueries, 10);
        stats.add_value_with_amount(MetricType::NumActiveQueries, 3);

        stats.reset_all_metrics();

        assert_eq!(stats.get_value(MetricType::NumQueries), Some(0));
        assert_eq!(stats.get_value(MetricType::NumActiveQueries), Some(0));
    }

    #[test]
    fn test_reset_space_metrics() {
        let stats = StatsManager::new();
        stats.add_space_metric("test_space", MetricType::NumQueries);
        stats.add_space_metric("test_space", MetricType::NumActiveQueries);

        stats.reset_space_metrics("test_space");

        assert_eq!(
            stats.get_space_value("test_space", MetricType::NumQueries),
            Some(0)
        );
        assert_eq!(
            stats.get_space_value("test_space", MetricType::NumActiveQueries),
            Some(0)
        );
    }

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
    fn test_record_and_get_query_profile() {
        let config = MonitoringConfig {
            enabled: true,
            memory_cache_size: 10,
            slow_query_threshold_ms: 1000,
            slow_query_log_dir: "test_logs".to_string(),
            slow_query_log_retention_days: 1,
        };
        let stats = StatsManager::with_config(config);

        let mut profile = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());
        profile.total_duration_ms = 500;
        profile.result_count = 10;

        stats.record_query_profile(profile.clone());

        assert_eq!(stats.query_cache_size(), 1);

        let recent = stats.get_recent_queries(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].session_id, 123);
    }

    #[test]
    fn test_get_slow_queries() {
        let config = MonitoringConfig {
            enabled: true,
            memory_cache_size: 10,
            slow_query_threshold_ms: 1000,
            slow_query_log_dir: "test_logs".to_string(),
            slow_query_log_retention_days: 1,
        };
        let stats = StatsManager::with_config(config);

        // 添加一个慢查询
        let mut slow_profile = QueryProfile::new(1, "MATCH (n) RETURN n".to_string());
        slow_profile.total_duration_ms = 2000; // 超过阈值
        stats.record_query_profile(slow_profile);

        // 添加一个快查询
        let mut fast_profile = QueryProfile::new(2, "MATCH (n) RETURN n LIMIT 1".to_string());
        fast_profile.total_duration_ms = 100; // 低于阈值
        stats.record_query_profile(fast_profile);

        let slow_queries = stats.get_slow_queries(10);
        assert_eq!(slow_queries.len(), 1);
        assert_eq!(slow_queries[0].session_id, 1);
    }

    #[test]
    fn test_get_query_profile_by_trace_id() {
        let stats = StatsManager::with_config(MonitoringConfig {
            enabled: true,
            memory_cache_size: 10,
            slow_query_threshold_ms: 1000,
            slow_query_log_dir: "test_logs".to_string(),
            slow_query_log_retention_days: 1,
        });

        let profile = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());
        let trace_id = profile.trace_id.clone();
        stats.record_query_profile(profile);

        let found = stats.get_query_profile(&trace_id);
        assert!(found.is_some());
        assert_eq!(found.expect("查询画像应该存在").session_id, 123);

        let not_found = stats.get_query_profile("non-existent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_session_queries() {
        let stats = StatsManager::with_config(MonitoringConfig {
            enabled: true,
            memory_cache_size: 10,
            slow_query_threshold_ms: 1000,
            slow_query_log_dir: "test_logs".to_string(),
            slow_query_log_retention_days: 1,
        });

        let profile1 = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());
        let profile2 = QueryProfile::new(123, "MATCH (n) RETURN n LIMIT 10".to_string());
        let profile3 = QueryProfile::new(456, "MATCH (n) RETURN n".to_string());

        stats.record_query_profile(profile1);
        stats.record_query_profile(profile2);
        stats.record_query_profile(profile3);

        let session_queries = stats.get_session_queries(123, 10);
        assert_eq!(session_queries.len(), 2);
    }

    #[test]
    fn test_clear_query_cache() {
        let stats = StatsManager::with_config(MonitoringConfig {
            enabled: true,
            memory_cache_size: 10,
            slow_query_threshold_ms: 1000,
            slow_query_log_dir: "test_logs".to_string(),
            slow_query_log_retention_days: 1,
        });

        let profile = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());
        stats.record_query_profile(profile);

        assert_eq!(stats.query_cache_size(), 1);

        stats.clear_query_cache();

        assert_eq!(stats.query_cache_size(), 0);
    }

    #[test]
    fn test_query_cache_size_limit() {
        let stats = StatsManager::with_config(MonitoringConfig {
            enabled: true,
            memory_cache_size: 3,
            slow_query_threshold_ms: 1000,
            slow_query_log_dir: "test_logs".to_string(),
            slow_query_log_retention_days: 1,
        });

        for i in 0..5 {
            let profile = QueryProfile::new(i as i64, format!("Query {}", i));
            stats.record_query_profile(profile);
        }

        // 缓存大小应该被限制为 3
        assert_eq!(stats.query_cache_size(), 3);

        // 最近的查询应该是最后添加的
        let recent = stats.get_recent_queries(3);
        assert_eq!(recent[0].session_id, 4);
        assert_eq!(recent[2].session_id, 2);
    }

    #[test]
    fn test_disabled_monitoring() {
        let stats = StatsManager::with_config(MonitoringConfig {
            enabled: false,
            memory_cache_size: 10,
            slow_query_threshold_ms: 1000,
            slow_query_log_dir: "test_logs".to_string(),
            slow_query_log_retention_days: 1,
        });

        let profile = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());
        stats.record_query_profile(profile);

        // 禁用监控时不应该记录任何查询
        assert_eq!(stats.query_cache_size(), 0);
    }

    // 新增错误统计测试

    #[test]
    fn test_error_stats_recording() {
        let stats = StatsManager::new();

        // 记录一些错误
        stats.record_error(ErrorType::ParseError, QueryPhase::Parse);
        stats.record_error(ErrorType::ParseError, QueryPhase::Parse);
        stats.record_error(ErrorType::ExecutionError, QueryPhase::Execute);

        // 验证错误计数
        assert_eq!(stats.get_error_count(ErrorType::ParseError), 2);
        assert_eq!(stats.get_error_count(ErrorType::ExecutionError), 1);
        assert_eq!(stats.get_error_count(ErrorType::StorageError), 0);

        // 验证阶段错误计数
        assert_eq!(stats.get_error_count_by_phase(QueryPhase::Parse), 2);
        assert_eq!(stats.get_error_count_by_phase(QueryPhase::Execute), 1);
        assert_eq!(stats.get_error_count_by_phase(QueryPhase::Validate), 0);
    }

    #[test]
    fn test_error_stats_summary() {
        let stats = StatsManager::new();

        stats.record_error(ErrorType::ParseError, QueryPhase::Parse);
        stats.record_error(ErrorType::ValidationError, QueryPhase::Validate);
        stats.record_error(ErrorType::ExecutionError, QueryPhase::Execute);

        let summary = stats.get_error_summary();
        assert_eq!(summary.total_errors, 3);
        assert_eq!(summary.errors_by_type.get(&ErrorType::ParseError), Some(&1));
        assert_eq!(summary.errors_by_type.get(&ErrorType::ValidationError), Some(&1));
        assert_eq!(summary.errors_by_type.get(&ErrorType::ExecutionError), Some(&1));
    }

    #[test]
    fn test_error_stats_reset() {
        let stats = StatsManager::new();

        stats.record_error(ErrorType::ParseError, QueryPhase::Parse);
        assert_eq!(stats.get_error_count(ErrorType::ParseError), 1);

        stats.reset_error_counts();
        assert_eq!(stats.get_error_count(ErrorType::ParseError), 0);
        assert_eq!(stats.get_error_summary().total_errors, 0);
    }

    #[test]
    fn test_query_profile_with_error_info() {
        let mut profile = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());

        let error_info = ErrorInfo::new(
            ErrorType::ParseError,
            QueryPhase::Parse,
            "语法错误: 缺少右括号"
        ).with_details("在位置 15 处发现语法错误");

        profile.mark_failed_with_info(error_info);

        assert!(matches!(profile.status, QueryStatus::Failed));
        assert_eq!(profile.error_message, Some("语法错误: 缺少右括号".to_string()));
        assert!(profile.error_info.is_some());

        let info = profile.error_info.unwrap();
        assert!(matches!(info.error_type, ErrorType::ParseError));
        assert!(matches!(info.error_phase, QueryPhase::Parse));
        assert_eq!(info.error_details, Some("在位置 15 处发现语法错误".to_string()));
    }

    #[test]
    fn test_query_profile_error_type_and_phase() {
        let mut profile = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());

        // 初始状态没有错误
        assert!(profile.error_type().is_none());
        assert!(profile.error_phase().is_none());

        let error_info = ErrorInfo::new(
            ErrorType::ExecutionError,
            QueryPhase::Execute,
            "执行超时"
        );
        profile.mark_failed_with_info(error_info);

        assert_eq!(profile.error_type(), Some(ErrorType::ExecutionError));
        assert_eq!(profile.error_phase(), Some(QueryPhase::Execute));
    }

    #[test]
    fn test_record_failed_query() {
        let stats = StatsManager::with_config(MonitoringConfig {
            enabled: true,
            memory_cache_size: 10,
            slow_query_threshold_ms: 1000,
            slow_query_log_dir: "test_logs".to_string(),
            slow_query_log_retention_days: 1,
        });

        let profile = QueryProfile::new(123, "INVALID QUERY".to_string());
        let error_info = ErrorInfo::new(
            ErrorType::ParseError,
            QueryPhase::Parse,
            "无效的查询语法"
        );

        stats.record_failed_query(profile, error_info);

        // 验证查询被记录
        assert_eq!(stats.query_cache_size(), 1);

        // 验证错误统计被更新
        assert_eq!(stats.get_error_count(ErrorType::ParseError), 1);
        assert_eq!(stats.get_error_count_by_phase(QueryPhase::Parse), 1);

        // 验证查询画像包含错误信息
        let recent = stats.get_recent_queries(1);
        assert_eq!(recent[0].error_message, Some("无效的查询语法".to_string()));
    }
}
