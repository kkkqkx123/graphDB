//! Statistics Manager
//!
//! Provides unified management of query metrics, query portraits and error statistics.

use dashmap::DashMap;
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use super::error_stats::{ErrorInfo, ErrorStatsManager, ErrorType, QueryPhase};
use super::metrics::QueryMetrics;
use super::profile::QueryProfile;

/// Space metrics type alias
type SpaceMetrics = Arc<DashMap<MetricType, Arc<MetricValue>>>;

/// Type of indicator
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
    // Query Type Statistics
    NumMatchQueries,
    NumCreateQueries,
    NumUpdateQueries,
    NumDeleteQueries,
    NumInsertQueries,
    NumGoQueries,
    NumFetchQueries,
    NumLookupQueries,
    NumShowQueries,
}

/// metric
pub struct MetricValue {
    pub value: AtomicU64,
    pub timestamp: AtomicU64,
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
        let _ = self
            .value
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
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

/// Statistics Manager
///
/// Unified management of query metrics, query profiling and error statistics.
pub struct StatsManager {
    metrics: Arc<DashMap<MetricType, Arc<MetricValue>>>,
    space_metrics: Arc<DashMap<String, SpaceMetrics>>,
    last_query_metrics: Arc<Mutex<Option<QueryMetrics>>>,
    query_profiles: Arc<Mutex<VecDeque<QueryProfile>>>,
    config: crate::config::MonitoringConfig,
    error_stats: ErrorStatsManager,
}

impl StatsManager {
    pub fn new() -> Self {
        Self::with_config(crate::config::MonitoringConfig::default())
    }

    pub fn with_config(config: crate::config::MonitoringConfig) -> Self {
        let cache_size = config.memory_cache_size;
        Self {
            metrics: Arc::new(DashMap::new()),
            space_metrics: Arc::new(DashMap::new()),
            last_query_metrics: Arc::new(Mutex::new(None)),
            query_profiles: Arc::new(Mutex::new(VecDeque::with_capacity(cache_size))),
            config,
            error_stats: ErrorStatsManager::new(),
        }
    }

    pub fn record_query_profile(&self, profile: QueryProfile) {
        if !self.config.enabled {
            return;
        }

        if profile.total_duration_ms >= self.config.slow_query_threshold_ms {
            self.write_slow_query_log(&profile);
        }

        let mut profiles = self.query_profiles.lock();
        if profiles.len() >= self.config.memory_cache_size {
            profiles.pop_front();
        }
        profiles.push_back(profile);
    }

    fn write_slow_query_log(&self, profile: &QueryProfile) {
        let executor_summary: Vec<String> = profile
            .executor_stats
            .iter()
            .map(|stat| {
                format!(
                    "{}[id={}, {}ms, rows={}, mem={}]",
                    stat.executor_type,
                    stat.executor_id,
                    stat.duration_ms,
                    stat.rows_processed,
                    stat.memory_used
                )
            })
            .collect();

        let error_str = if let Some(ref info) = profile.error_info {
            format!(
                " [error={} phase={}]: {}",
                info.error_type, info.error_phase, info.error_message
            )
        } else if let Some(ref msg) = profile.error_message {
            format!(" [error]: {}", msg)
        } else {
            String::new()
        };

        log::warn!(
            "慢查询 [trace_id={}] [session_id={}] [duration={}ms] [status={}]\n\
             查询: {}\n\
             阶段统计: parse={}ms validate={}ms plan={}ms optimize={}ms execute={}ms\n\
             结果数: {} 执行器数: {} 执行器总时间: {}ms\n\
             执行器详情: {}{}",
            profile.trace_id,
            profile.session_id,
            profile.total_duration_ms,
            match profile.status {
                super::profile::QueryStatus::Success => "success",
                super::profile::QueryStatus::Failed => "failed",
            },
            profile.query_text,
            profile.stages.parse_ms,
            profile.stages.validate_ms,
            profile.stages.plan_ms,
            profile.stages.optimize_ms,
            profile.stages.execute_ms,
            profile.result_count,
            profile.executor_stats.len(),
            profile.total_executor_time_ms(),
            executor_summary.join(", "),
            error_str
        );
    }

    pub fn get_recent_queries(&self, limit: usize) -> Vec<QueryProfile> {
        let profiles = self.query_profiles.lock();
        profiles.iter().rev().take(limit).cloned().collect()
    }

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

    pub fn get_query_profile(&self, trace_id: &str) -> Option<QueryProfile> {
        let profiles = self.query_profiles.lock();
        profiles.iter().find(|p| p.trace_id == trace_id).cloned()
    }

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

    pub fn get_executor_stats_summary(&self) -> HashMap<String, (u64, u64, usize)> {
        let profiles = self.query_profiles.lock();
        let mut stats: HashMap<String, (u64, u64, usize)> = HashMap::new();

        for profile in profiles.iter() {
            for exec_stat in &profile.executor_stats {
                let entry = stats
                    .entry(exec_stat.executor_type.clone())
                    .or_insert((0, 0, 0));
                entry.0 += exec_stat.duration_ms;
                entry.1 += exec_stat.rows_processed as u64;
                entry.2 += 1;
            }
        }

        stats
    }

    pub fn clear_query_cache(&self) {
        let mut profiles = self.query_profiles.lock();
        profiles.clear();
    }

    pub fn query_cache_size(&self) -> usize {
        let profiles = self.query_profiles.lock();
        profiles.len()
    }

    pub fn add_value(&self, metric_type: MetricType) {
        let metric = self
            .metrics
            .entry(metric_type)
            .or_insert_with(|| Arc::new(MetricValue::new(0)));
        metric.increment();
    }

    pub fn add_value_with_amount(&self, metric_type: MetricType, amount: u64) {
        let metric = self
            .metrics
            .entry(metric_type)
            .or_insert_with(|| Arc::new(MetricValue::new(0)));
        metric.add(amount);
    }

    pub fn dec_value(&self, metric_type: MetricType) {
        if let Some(metric) = self.metrics.get(&metric_type) {
            metric.decrement();
        }
    }

    pub fn add_space_metric(&self, space_name: &str, metric_type: MetricType) {
        let space_map = self
            .space_metrics
            .entry(space_name.to_string())
            .or_insert_with(|| Arc::new(DashMap::new()));
        let metric = space_map
            .entry(metric_type)
            .or_insert_with(|| Arc::new(MetricValue::new(0)));
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
        self.space_metrics
            .get(space_name)
            .and_then(|space_map| space_map.get(&metric_type).map(|metric| metric.get()))
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

    pub fn record_error(&self, error_type: ErrorType, phase: QueryPhase) {
        self.error_stats.record_error(error_type, phase);
    }

    pub fn get_error_count(&self, error_type: ErrorType) -> u64 {
        self.error_stats.get_error_count(error_type)
    }

    pub fn get_error_count_by_phase(&self, phase: QueryPhase) -> u64 {
        self.error_stats.get_error_count_by_phase(phase)
    }

    pub fn get_all_error_counts(&self) -> HashMap<ErrorType, u64> {
        self.error_stats.get_all_error_counts()
    }

    pub fn get_all_error_counts_by_phase(&self) -> HashMap<QueryPhase, u64> {
        self.error_stats.get_all_error_counts_by_phase()
    }

    pub fn reset_error_counts(&self) {
        self.error_stats.reset_error_counts();
    }

    pub fn record_failed_query(&self, mut profile: QueryProfile, error_info: ErrorInfo) {
        profile.mark_failed_with_info(error_info.clone());
        self.record_error(error_info.error_type, error_info.error_phase);
        self.record_query_profile(profile);
    }

    pub fn get_error_summary(&self) -> super::error_stats::ErrorSummary {
        self.error_stats.get_error_summary()
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
            (
                MetricType::QueryPlanNodeCount,
                metrics.plan_node_count as u64,
            ),
            (
                MetricType::QueryResultRowCount,
                metrics.result_row_count as u64,
            ),
        ];

        for (metric_type, value) in updates {
            let metric = self
                .metrics
                .entry(metric_type)
                .or_insert_with(|| Arc::new(MetricValue::new(0)));
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
        assert_eq!(stats.get_value(MetricType::NumQueries), None);

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
    fn test_get_all_metrics() {
        let stats = StatsManager::new();
        stats.add_value(MetricType::NumQueries);
        stats.add_value(MetricType::NumActiveQueries);

        let all_metrics = stats.get_all_metrics();
        assert_eq!(all_metrics.get(&MetricType::NumQueries), Some(&1));
        assert_eq!(all_metrics.get(&MetricType::NumActiveQueries), Some(&1));
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
    fn test_record_and_get_query_profile() {
        let config = crate::config::MonitoringConfig {
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
        let config = crate::config::MonitoringConfig {
            enabled: true,
            memory_cache_size: 10,
            slow_query_threshold_ms: 1000,
            slow_query_log_dir: "test_logs".to_string(),
            slow_query_log_retention_days: 1,
        };
        let stats = StatsManager::with_config(config);

        let mut slow_profile = QueryProfile::new(1, "MATCH (n) RETURN n".to_string());
        slow_profile.total_duration_ms = 2000;
        stats.record_query_profile(slow_profile);

        let mut fast_profile = QueryProfile::new(2, "MATCH (n) RETURN n LIMIT 1".to_string());
        fast_profile.total_duration_ms = 100;
        stats.record_query_profile(fast_profile);

        let slow_queries = stats.get_slow_queries(10);
        assert_eq!(slow_queries.len(), 1);
        assert_eq!(slow_queries[0].session_id, 1);
    }

    #[test]
    fn test_query_cache_size_limit() {
        let stats = StatsManager::with_config(crate::config::MonitoringConfig {
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

        assert_eq!(stats.query_cache_size(), 3);

        let recent = stats.get_recent_queries(3);
        assert_eq!(recent[0].session_id, 4);
        assert_eq!(recent[2].session_id, 2);
    }

    #[test]
    fn test_disabled_monitoring() {
        let stats = StatsManager::with_config(crate::config::MonitoringConfig {
            enabled: false,
            memory_cache_size: 10,
            slow_query_threshold_ms: 1000,
            slow_query_log_dir: "test_logs".to_string(),
            slow_query_log_retention_days: 1,
        });

        let profile = QueryProfile::new(123, "MATCH (n) RETURN n".to_string());
        stats.record_query_profile(profile);

        assert_eq!(stats.query_cache_size(), 0);
    }
}
