use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

// Define statistics counters
pub struct GraphStats {
    stats: Arc<Mutex<HashMap<StatType, Arc<AtomicU64>>>>,
    query_stats: QueryStats,
    session_stats: SessionStats,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StatType {
    NumQueries,
    NumActiveQueries,
    NumSlowQueries,
    NumQueryErrors,
    NumQueryErrorsLeaderChanges,
    NumSentences,
    QueryLatencyUs,
    SlowQueryLatencyUs,
    NumKilledQueries,
    NumQueriesHitMemoryWatermark,
    OptimizerLatencyUs,
    NumAggregateExecutors,
    NumSortExecutors,
    NumIndexScanExecutors,
    NumOpenedSessions,
    NumAuthFailedSessions,
    NumAuthFailedSessionsBadUserNamePassword,
    NumAuthFailedSessionsOutOfMaxAllowed,
    NumActiveSessions,
    NumReclaimedExpiredSessions,
}

#[derive(Debug)]
struct QueryStats {
    slow_query_threshold_us: u64,
    enable_space_level_metrics: bool,
}

#[derive(Debug)]
struct SessionStats {
    num_opened_sessions: Arc<AtomicUsize>,
    num_active_sessions: Arc<AtomicUsize>,
    num_auth_failed_sessions: Arc<AtomicUsize>,
    num_reclaimed_expired_sessions: Arc<AtomicUsize>,
}

impl GraphStats {
    pub fn new() -> Self {
        let mut stats = HashMap::new();
        // Initialize all counters to 0
        for stat_type in [
            StatType::NumQueries,
            StatType::NumActiveQueries,
            StatType::NumSlowQueries,
            StatType::NumQueryErrors,
            StatType::NumQueryErrorsLeaderChanges,
            StatType::NumSentences,
            StatType::QueryLatencyUs,
            StatType::SlowQueryLatencyUs,
            StatType::NumKilledQueries,
            StatType::NumQueriesHitMemoryWatermark,
            StatType::OptimizerLatencyUs,
            StatType::NumAggregateExecutors,
            StatType::NumSortExecutors,
            StatType::NumIndexScanExecutors,
            StatType::NumOpenedSessions,
            StatType::NumAuthFailedSessions,
            StatType::NumAuthFailedSessionsBadUserNamePassword,
            StatType::NumAuthFailedSessionsOutOfMaxAllowed,
            StatType::NumActiveSessions,
            StatType::NumReclaimedExpiredSessions,
        ] {
            stats.insert(stat_type, Arc::new(AtomicU64::new(0)));
        }

        Self {
            stats: Arc::new(Mutex::new(stats)),
            query_stats: QueryStats {
                slow_query_threshold_us: 5_000_000, // 5 seconds in microseconds
                enable_space_level_metrics: false,
            },
            session_stats: SessionStats {
                num_opened_sessions: Arc::new(AtomicUsize::new(0)),
                num_active_sessions: Arc::new(AtomicUsize::new(0)),
                num_auth_failed_sessions: Arc::new(AtomicUsize::new(0)),
                num_reclaimed_expired_sessions: Arc::new(AtomicUsize::new(0)),
            },
        }
    }

    pub fn increment_counter(&self, stat_type: StatType) {
        if let Ok(stats) = self.stats.lock() {
            if let Some(counter) = stats.get(&stat_type) {
                counter.fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    pub fn add_value(&self, stat_type: StatType, value: u64) {
        if let Ok(stats) = self.stats.lock() {
            if let Some(counter) = stats.get(&stat_type) {
                counter.fetch_add(value, Ordering::SeqCst);
            }
        }
    }

    pub fn get_counter(&self, stat_type: StatType) -> u64 {
        if let Ok(stats) = self.stats.lock() {
            if let Some(counter) = stats.get(&stat_type) {
                return counter.load(Ordering::SeqCst);
            }
        }
        0
    }

    pub fn record_query_execution(&self, latency_us: u64) {
        self.increment_counter(StatType::NumQueries);
        self.add_value(StatType::QueryLatencyUs, latency_us);
        
        if latency_us > self.query_stats.slow_query_threshold_us {
            self.increment_counter(StatType::NumSlowQueries);
            self.add_value(StatType::SlowQueryLatencyUs, latency_us);
        }
    }

    pub fn record_session_opened(&self) {
        self.session_stats.num_opened_sessions.fetch_add(1, Ordering::SeqCst);
        self.increment_counter(StatType::NumOpenedSessions);
    }

    pub fn record_session_closed(&self) {
        self.increment_counter(StatType::NumActiveSessions);
    }

    pub fn record_auth_failure(&self) {
        self.session_stats.num_auth_failed_sessions.fetch_add(1, Ordering::SeqCst);
        self.increment_counter(StatType::NumAuthFailedSessions);
    }

    pub fn record_expired_session_reclaimed(&self) {
        self.session_stats
            .num_reclaimed_expired_sessions
            .fetch_add(1, Ordering::SeqCst);
        self.increment_counter(StatType::NumReclaimedExpiredSessions);
    }

    pub fn get_session_stats(&self) -> (usize, usize, usize, usize) {
        (
            self.session_stats.num_opened_sessions.load(Ordering::SeqCst),
            self.session_stats.num_active_sessions.load(Ordering::SeqCst),
            self.session_stats.num_auth_failed_sessions.load(Ordering::SeqCst),
            self.session_stats
                .num_reclaimed_expired_sessions
                .load(Ordering::SeqCst),
        )
    }
}

pub fn init_graph_stats() -> GraphStats {
    GraphStats::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_stats_creation() {
        let stats = GraphStats::new();
        
        // Check initial values are 0
        assert_eq!(stats.get_counter(StatType::NumQueries), 0);
        assert_eq!(stats.get_counter(StatType::NumActiveQueries), 0);
    }

    #[test]
    fn test_increment_counter() {
        let stats = GraphStats::new();
        
        stats.increment_counter(StatType::NumQueries);
        assert_eq!(stats.get_counter(StatType::NumQueries), 1);
        
        stats.increment_counter(StatType::NumQueries);
        assert_eq!(stats.get_counter(StatType::NumQueries), 2);
    }

    #[test]
    fn test_add_value() {
        let stats = GraphStats::new();
        
        stats.add_value(StatType::QueryLatencyUs, 1000);
        assert_eq!(stats.get_counter(StatType::QueryLatencyUs), 1000);
        
        stats.add_value(StatType::QueryLatencyUs, 500);
        assert_eq!(stats.get_counter(StatType::QueryLatencyUs), 1500);
    }

    #[test]
    fn test_record_query_execution() {
        let stats = GraphStats::new();
        
        // Record a fast query
        stats.record_query_execution(1000); // 1000 microseconds = 1ms
        assert_eq!(stats.get_counter(StatType::NumQueries), 1);
        assert_eq!(stats.get_counter(StatType::QueryLatencyUs), 1000);
        assert_eq!(stats.get_counter(StatType::NumSlowQueries), 0);
        
        // Record a slow query
        stats.record_query_execution(10_000_000); // 10 seconds
        assert_eq!(stats.get_counter(StatType::NumQueries), 2);
        assert_eq!(stats.get_counter(StatType::QueryLatencyUs), 10_001_000); // 1000 + 10_000_000
        assert_eq!(stats.get_counter(StatType::NumSlowQueries), 1);
    }

    #[test]
    fn test_session_stats() {
        let stats = GraphStats::new();
        
        stats.record_session_opened();
        let (opened, active, failed, reclaimed) = stats.get_session_stats();
        assert_eq!(opened, 1);
        assert_eq!(failed, 0);
        
        stats.record_auth_failure();
        let (opened, active, failed, reclaimed) = stats.get_session_stats();
        assert_eq!(failed, 1);
        
        stats.record_expired_session_reclaimed();
        let (opened, active, failed, reclaimed) = stats.get_session_stats();
        assert_eq!(reclaimed, 1);
    }
}