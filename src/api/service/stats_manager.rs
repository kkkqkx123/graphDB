use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

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
}

impl StatsManager {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            space_metrics: Arc::new(RwLock::new(HashMap::new())),
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
        metrics.get(&metric_type).map(|v| v.value)
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
        stats.add_space_metric("test_space".to_string(), MetricType::NumQueries);
        stats.add_space_metric("test_space".to_string(), MetricType::NumActiveQueries);

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
        stats.add_space_metric("test_space".to_string(), MetricType::NumQueries);
        stats.add_space_metric("test_space".to_string(), MetricType::NumActiveQueries);

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
