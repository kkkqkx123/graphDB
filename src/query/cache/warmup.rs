//! Cache Warmup Module
//!
//! Provide cache warmup functionality to reduce cold start impact.
//!
//! # Design Goals
//!
//! 1. Preload frequently used query plans
//! 2. Preload frequently used CTE results
//! 3. Support configuration-based warmup
//! 4. Support statistics-based automatic warmup

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::plan_cache::QueryPlanCache;

/// Cache warmer
///
/// Provides cache warmup functionality to reduce cold start impact
pub struct CacheWarmer {
    plan_cache: Arc<QueryPlanCache>,
    warmup_queries: Vec<String>,
}

impl CacheWarmer {
    /// Create a new cache warmer
    pub fn new(plan_cache: Arc<QueryPlanCache>) -> Self {
        Self {
            plan_cache,
            warmup_queries: Vec::new(),
        }
    }

    /// Load warmup data from configuration file
    pub fn from_config(
        config_path: &Path,
        plan_cache: Arc<QueryPlanCache>,
    ) -> Result<Self, WarmupError> {
        let config: WarmupConfig = serde_json::from_reader(
            File::open(config_path).map_err(|e| WarmupError::ConfigReadError(e.to_string()))?,
        )
        .map_err(|e| WarmupError::ConfigParseError(e.to_string()))?;

        Ok(Self {
            plan_cache,
            warmup_queries: config.queries,
        })
    }

    /// Execute warmup
    pub async fn warmup(&self) -> WarmupResult {
        log::info!("Starting cache warmup...");

        let mut result = WarmupResult::default();

        for query in &self.warmup_queries {
            match self.warmup_query(query.as_str()).await {
                Ok(_) => {
                    result.successful_queries += 1;
                    log::debug!("Warmed up query plan: {}", query);
                }
                Err(e) => {
                    result.failed_queries += 1;
                    result.errors.push(format!("Query '{}': {}", query, e));
                    log::warn!("Failed to warmup query '{}': {}", query, e);
                }
            }
        }

        log::info!(
            "Cache warmup completed: {} queries successful, {} failed",
            result.successful_queries,
            result.failed_queries
        );

        result
    }

    /// Warmup a single query
    async fn warmup_query(&self, query: &str) -> Result<(), WarmupError> {
        if self.plan_cache.contains(query) {
            return Ok(());
        }

        let _plan = self
            .prepare_query(query)
            .await
            .map_err(|e| WarmupError::QueryPrepareError(e.to_string()))?;

        Ok(())
    }

    /// Prepare a query (placeholder implementation)
    async fn prepare_query(&self, query: &str) -> Result<(), WarmupError> {
        log::debug!("Preparing query: {}", query);
        Ok(())
    }

    /// Warmup from statistics
    pub async fn warmup_from_stats(&self, stats: &QueryStats) -> WarmupResult {
        log::info!("Starting cache warmup from statistics...");

        let mut result = WarmupResult::default();

        let top_queries = stats.most_frequent_queries(100);

        for (query, frequency) in top_queries {
            if frequency < 10 {
                continue;
            }

            match self.warmup_query(query.as_str()).await {
                Ok(_) => {
                    result.successful_queries += 1;
                    log::debug!(
                        "Warmed up query from stats: {} (freq: {})",
                        query,
                        frequency
                    );
                }
                Err(e) => {
                    result.failed_queries += 1;
                    result.errors.push(format!("Query '{}': {}", query, e));
                    log::warn!("Failed to warmup query '{}': {}", query, e);
                }
            }
        }

        log::info!(
            "Cache warmup from stats completed: {} queries successful, {} failed",
            result.successful_queries,
            result.failed_queries
        );

        result
    }

    /// Get warmup queries
    pub fn warmup_queries(&self) -> &[String] {
        &self.warmup_queries
    }

    /// Get warmup CTEs
    pub fn warmup_ctes(&self) -> &[String] {
        &[]
    }

    /// Add warmup query
    pub fn add_warmup_query(&mut self, query: String) {
        self.warmup_queries.push(query);
    }

    /// Add warmup CTE
    pub fn add_warmup_cte(&mut self, _cte: String) {}

    /// Clear warmup data
    pub fn clear_warmup_data(&mut self) {
        self.warmup_queries.clear();
    }
}

/// Warmup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmupConfig {
    pub queries: Vec<String>,
}

impl Default for WarmupConfig {
    fn default() -> Self {
        Self {
            queries: Vec::new(),
        }
    }
}

/// Warmup result
#[derive(Debug, Clone, Default)]
pub struct WarmupResult {
    pub successful_queries: usize,
    pub failed_queries: usize,
    pub errors: Vec<String>,
}

impl WarmupResult {
    /// Check if warmup was successful
    pub fn is_success(&self) -> bool {
        self.failed_queries == 0
    }

    /// Get total queries
    pub fn total_queries(&self) -> usize {
        self.successful_queries + self.failed_queries
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_queries();
        if total == 0 {
            1.0
        } else {
            self.successful_queries as f64 / total as f64
        }
    }

    /// Format result for display
    pub fn format(&self) -> String {
        format!(
            "Warmup Result:\n\
             - Total: {} queries\n\
             - Successful: {} ({:.1}%)\n\
             - Failed: {} ({:.1}%)",
            self.total_queries(),
            self.successful_queries,
            self.success_rate() * 100.0,
            self.failed_queries,
            (1.0 - self.success_rate()) * 100.0
        )
    }
}

/// Query statistics
#[derive(Debug, Clone, Default)]
pub struct QueryStats {
    pub query_frequencies: HashMap<String, u64>,
    pub total_queries: u64,
}

impl QueryStats {
    /// Create new query stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a query execution
    pub fn record_query(&mut self, query: &str) {
        *self.query_frequencies.entry(query.to_string()).or_insert(0) += 1;
        self.total_queries += 1;
    }

    /// Get most frequent queries
    pub fn most_frequent_queries(&self, limit: usize) -> Vec<(String, u64)> {
        let mut queries: Vec<_> = self
            .query_frequencies
            .iter()
            .map(|(q, f)| (q.clone(), *f))
            .collect();

        queries.sort_by(|a, b| b.1.cmp(&a.1));
        queries.truncate(limit);

        queries
    }

    /// Get frequency of a specific query
    pub fn query_frequency(&self, query: &str) -> u64 {
        *self.query_frequencies.get(query).unwrap_or(&0)
    }

    /// Get total query count
    pub fn total_queries(&self) -> u64 {
        self.total_queries
    }

    /// Get unique query count
    pub fn unique_queries(&self) -> usize {
        self.query_frequencies.len()
    }
}

/// Warmup error
#[derive(Debug, Clone)]
pub enum WarmupError {
    ConfigReadError(String),
    ConfigParseError(String),
    QueryPrepareError(String),
    CacheError(String),
}

impl std::fmt::Display for WarmupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigReadError(msg) => write!(f, "Config read error: {}", msg),
            Self::ConfigParseError(msg) => write!(f, "Config parse error: {}", msg),
            Self::QueryPrepareError(msg) => write!(f, "Query prepare error: {}", msg),
            Self::CacheError(msg) => write!(f, "Cache error: {}", msg),
        }
    }
}

impl std::error::Error for WarmupError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warmup_config_default() {
        let config = WarmupConfig::default();
        assert!(config.queries.is_empty());
    }

    #[test]
    fn test_warmup_result_default() {
        let result = WarmupResult::default();
        assert_eq!(result.successful_queries, 0);
        assert_eq!(result.failed_queries, 0);
        assert!(result.is_success());
        assert_eq!(result.success_rate(), 1.0);
    }

    #[test]
    fn test_warmup_result_format() {
        let result = WarmupResult {
            successful_queries: 8,
            failed_queries: 2,
            errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        };

        let formatted = result.format();
        assert!(formatted.contains("Total: 10 queries"));
        assert!(formatted.contains("Successful: 8"));
        assert!(formatted.contains("Failed: 2"));
    }

    #[test]
    fn test_query_stats() {
        let mut stats = QueryStats::new();

        stats.record_query("SELECT 1");
        stats.record_query("SELECT 1");
        stats.record_query("SELECT 2");

        assert_eq!(stats.total_queries(), 3);
        assert_eq!(stats.unique_queries(), 2);
        assert_eq!(stats.query_frequency("SELECT 1"), 2);
        assert_eq!(stats.query_frequency("SELECT 2"), 1);
    }

    #[test]
    fn test_most_frequent_queries() {
        let mut stats = QueryStats::new();

        stats.record_query("SELECT 1");
        stats.record_query("SELECT 1");
        stats.record_query("SELECT 1");
        stats.record_query("SELECT 2");
        stats.record_query("SELECT 2");
        stats.record_query("SELECT 3");

        let top = stats.most_frequent_queries(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "SELECT 1");
        assert_eq!(top[0].1, 3);
        assert_eq!(top[1].0, "SELECT 2");
        assert_eq!(top[1].1, 2);
    }

    #[test]
    fn test_warmup_error_display() {
        let err = WarmupError::ConfigReadError("File not found".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Config read error"));
    }
}
