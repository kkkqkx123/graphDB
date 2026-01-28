use std::collections::HashMap;
use std::sync::{Arc, RwLock};
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
    NumAuthFailedSessionsBadUserNamePassword,
    NumAuthFailedSessionsOutOfMaxAllowed,
    NumOpenedSessions,
    NumActiveSessions,
    NumQueries,
    NumActiveQueries,
    NumKilledQueries,
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
    pub value: u64,
    pub timestamp: Instant,
}

impl MetricValue {
    pub fn new(value: u64) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
        }
    }
}

pub struct StatsManager {
    metrics: Arc<RwLock<HashMap<MetricType, MetricValue>>>,
    space_metrics: Arc<RwLock<HashMap<String, HashMap<MetricType, MetricValue>>>>,
    last_query_metrics: Arc<RwLock<Option<QueryMetrics>>>,
}

impl StatsManager {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            space_metrics: Arc::new(RwLock::new(HashMap::new())),
            last_query_metrics: Arc::new(RwLock::new(None)),
        }
    }

    pub fn add_value(&self, metric_type: MetricType) {
        let mut metrics = self.metrics.write().expect("获取指标写锁失败");
        let entry = metrics.entry(metric_type).or_insert_with(|| MetricValue::new(0));
        entry.value += 1;
        entry.timestamp = Instant::now();
    }

    pub fn add_value_with_amount(&self, metric_type: MetricType, amount: u64) {
        let mut metrics = self.metrics.write().expect("获取指标写锁失败");
        let entry = metrics.entry(metric_type).or_insert_with(|| MetricValue::new(0));
        entry.value += amount;
        entry.timestamp = Instant::now();
    }

    pub fn dec_value(&self, metric_type: MetricType) {
        let mut metrics = self.metrics.write().expect("获取指标写锁失败");
        if let Some(entry) = metrics.get_mut(&metric_type) {
            if entry.value > 0 {
                entry.value -= 1;
                entry.timestamp = Instant::now();
            }
        }
    }

    pub fn add_space_metric(&self, space_name: &str, metric_type: MetricType) {
        let mut space_metrics = self.space_metrics.write().expect("获取空间指标写锁失败");
        let space_entry = space_metrics
            .entry(space_name.to_string())
            .or_insert_with(HashMap::new);

        let metric_entry = space_entry
            .entry(metric_type)
            .or_insert_with(|| MetricValue::new(0));
        metric_entry.value += 1;
        metric_entry.timestamp = Instant::now();
    }

    pub fn dec_space_metric(&self, space_name: &str, metric_type: MetricType) {
        let mut space_metrics = self.space_metrics.write().expect("获取空间指标写锁失败");
        if let Some(space_entry) = space_metrics.get_mut(space_name) {
            if let Some(metric_entry) = space_entry.get_mut(&metric_type) {
                if metric_entry.value > 0 {
                    metric_entry.value -= 1;
                    metric_entry.timestamp = Instant::now();
                }
            }
        }
    }

    pub fn get_value(&self, metric_type: MetricType) -> Option<u64> {
        let metrics = self.metrics.read().expect("获取指标读锁失败");
        Some(metrics.get(&metric_type).map(|v| v.value).unwrap_or(0))
    }

    pub fn get_space_value(&self, space_name: &str, metric_type: MetricType) -> Option<u64> {
        let space_metrics = self.space_metrics.read().expect("获取空间指标读锁失败");
        space_metrics
            .get(space_name)
            .and_then(|space| space.get(&metric_type).map(|v| v.value))
    }

    pub fn get_all_metrics(&self) -> HashMap<MetricType, u64> {
        let metrics = self.metrics.read().expect("获取指标读锁失败");
        metrics
            .iter()
            .map(|(k, v)| (*k, v.value))
            .collect()
    }

    pub fn get_all_space_metrics(&self, space_name: &str) -> Option<HashMap<MetricType, u64>> {
        let space_metrics = self.space_metrics.read().expect("获取空间指标读锁失败");
        space_metrics.get(space_name).map(|space| {
            space.iter().map(|(k, v)| (*k, v.value)).collect()
        })
    }

    pub fn reset_metric(&self, metric_type: MetricType) {
        let mut metrics = self.metrics.write().expect("获取指标写锁失败");
        if let Some(entry) = metrics.get_mut(&metric_type) {
            entry.value = 0;
            entry.timestamp = Instant::now();
        }
    }

    pub fn reset_all_metrics(&self) {
        let mut metrics = self.metrics.write().expect("获取指标写锁失败");
        for entry in metrics.values_mut() {
            entry.value = 0;
            entry.timestamp = Instant::now();
        }
    }

    pub fn reset_space_metrics(&self, space_name: &str) {
        let mut space_metrics = self.space_metrics.write().expect("获取空间指标写锁失败");
        if let Some(space_entry) = space_metrics.get_mut(space_name) {
            for entry in space_entry.values_mut() {
                entry.value = 0;
                entry.timestamp = Instant::now();
            }
        }
    }
    
    pub fn record_query_metrics(&self, metrics: &QueryMetrics) {
        let mut last_metrics = self.last_query_metrics.write().expect("获取查询指标写锁失败");
        *last_metrics = Some(metrics.clone());
        
        let mut global_metrics = self.metrics.write().expect("获取指标写锁失败");
        
        let entry = global_metrics.entry(MetricType::QueryParseTimeUs).or_insert_with(|| MetricValue::new(0));
        entry.value = metrics.parse_time_us;
        entry.timestamp = Instant::now();
        
        let entry = global_metrics.entry(MetricType::QueryValidateTimeUs).or_insert_with(|| MetricValue::new(0));
        entry.value = metrics.validate_time_us;
        entry.timestamp = Instant::now();
        
        let entry = global_metrics.entry(MetricType::QueryPlanTimeUs).or_insert_with(|| MetricValue::new(0));
        entry.value = metrics.plan_time_us;
        entry.timestamp = Instant::now();
        
        let entry = global_metrics.entry(MetricType::QueryOptimizeTimeUs).or_insert_with(|| MetricValue::new(0));
        entry.value = metrics.optimize_time_us;
        entry.timestamp = Instant::now();
        
        let entry = global_metrics.entry(MetricType::QueryExecuteTimeUs).or_insert_with(|| MetricValue::new(0));
        entry.value = metrics.execute_time_us;
        entry.timestamp = Instant::now();
        
        let entry = global_metrics.entry(MetricType::QueryTotalTimeUs).or_insert_with(|| MetricValue::new(0));
        entry.value = metrics.total_time_us;
        entry.timestamp = Instant::now();
        
        let entry = global_metrics.entry(MetricType::QueryPlanNodeCount).or_insert_with(|| MetricValue::new(0));
        entry.value = metrics.plan_node_count as u64;
        entry.timestamp = Instant::now();
        
        let entry = global_metrics.entry(MetricType::QueryResultRowCount).or_insert_with(|| MetricValue::new(0));
        entry.value = metrics.result_row_count as u64;
        entry.timestamp = Instant::now();
    }
    
    pub fn get_last_query_metrics(&self) -> Option<QueryMetrics> {
        let last_metrics = self.last_query_metrics.read().expect("获取查询指标读锁失败");
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
        assert_eq!(stats.get_value(MetricType::NumQueries), Some(0));
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
        stats.add_value(MetricType::NumOpenedSessions);
        stats.add_value(MetricType::NumActiveSessions);

        let all_metrics = stats.get_all_metrics();
        assert_eq!(all_metrics.get(&MetricType::NumQueries), Some(&1));
        assert_eq!(all_metrics.get(&MetricType::NumOpenedSessions), Some(&1));
        assert_eq!(all_metrics.get(&MetricType::NumActiveSessions), Some(&1));
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
        stats.add_value_with_amount(MetricType::NumOpenedSessions, 5);
        stats.add_value_with_amount(MetricType::NumActiveSessions, 3);

        stats.reset_all_metrics();

        assert_eq!(stats.get_value(MetricType::NumQueries), Some(0));
        assert_eq!(stats.get_value(MetricType::NumOpenedSessions), Some(0));
        assert_eq!(stats.get_value(MetricType::NumActiveSessions), Some(0));
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
