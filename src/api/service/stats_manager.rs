use dashmap::DashMap;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::time::Instant;

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

pub struct StatsManager {
    metrics: Arc<DashMap<MetricType, Arc<MetricValue>>>,
    space_metrics: Arc<DashMap<String, Arc<DashMap<MetricType, Arc<MetricValue>>>>>,
    last_query_metrics: Arc<Mutex<Option<QueryMetrics>>>,
}

impl StatsManager {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(DashMap::new()),
            space_metrics: Arc::new(DashMap::new()),
            last_query_metrics: Arc::new(Mutex::new(None)),
        }
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
}
