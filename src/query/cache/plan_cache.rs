//! Query Plan Cache Module
//!
//! Provides Prepared Statement style query plan caching with support for parameterized queries.
//!
//! # Design objectives
//!
//! 1. Cache query plan parsing, validation and planning results
//! 2. Support for parameterized queries (Prepared Statement)
//! 3. Limit memory usage to prevent unlimited growth
//! 4. Thread-safe, supporting highly concurrent access
//!
//! # Scenarios of use
//!
//! - Repeated execution of the same query template
//! - Batch insert/update operations
//! - Applications use Prepared Statements

use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::error::{DBError, DBResult};
use crate::query::planning::plan::ExecutionPlan;

/// Query Plan Cache Configuration
#[derive(Debug, Clone)]
pub struct PlanCacheConfig {
    /// Maximum number of cache entries
    pub max_entries: usize,
    /// Maximum survival time of entries (seconds)
    pub ttl_seconds: u64,
    /// Whether to enable parameterized query support
    pub enable_parameterized: bool,
}

impl Default for PlanCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl_seconds: 3600, // 1 hour
            enable_parameterized: true,
        }
    }
}

/// Cached query plan entries
#[derive(Debug, Clone)]
pub struct CachedPlan {
    /// Query template (parameterized form)
    pub query_template: String,
    /// implementation plan
    pub plan: ExecutionPlan,
    /// Parameter location information (for parameter binding)
    pub param_positions: Vec<ParamPosition>,
    /// Creation time
    pub created_at: Instant,
    /// Last access time
    pub last_accessed: Instant,
    /// Number of visits
    pub access_count: u64,
    /// Average execution time (milliseconds)
    pub avg_execution_time_ms: f64,
    /// Number of executions
    pub execution_count: u64,
}

/// Parameter location information
#[derive(Debug, Clone)]
pub struct ParamPosition {
    /// Parameter Index
    pub index: usize,
    /// Parameter name (named parameter)
    pub name: Option<String>,
    /// Position of the parameter in the query
    pub position: usize,
    /// Desired data types
    pub expected_type: Option<crate::core::types::DataType>,
}

/// Query Plan Cache Key
///
/// Supports fast lookups using the hash of the query text as the key.
/// Also store query text for conflict detection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlanCacheKey {
    /// Query the hash value of the text
    pub hash: u64,
    /// Query text (for conflict detection, not just debugging)
    query_text: String,
}

impl PlanCacheKey {
    /// Creating Cache Keys from Query Text
    pub fn from_query(query: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        let hash = hasher.finish();

        Self {
            hash,
            query_text: query.to_string(),
        }
    }

    /// Verify that the query text matches (for conflict detection)
    pub fn verify_query(&self, query: &str) -> bool {
        self.query_text == query
    }

    /// Get query text (for debugging or logging)
    pub fn query_text(&self) -> &str {
        &self.query_text
    }
}

/// Query Plan Cache Statistics
#[derive(Debug, Clone, Default)]
pub struct PlanCacheStats {
    /// Number of hits
    pub hits: u64,
    /// Number of missed hits
    pub misses: u64,
    /// Number of eliminations
    pub evictions: u64,
    /// Number of expiration dates
    pub expirations: u64,
    /// Number of current cache entries
    pub current_entries: usize,
    /// Average query template size (bytes)
    pub avg_query_template_bytes: usize,
    /// Total query template size (bytes)
    pub total_query_template_bytes: usize,
}

impl PlanCacheStats {
    /// hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Estimate total memory footprint (based on average template size and number of entries)
    pub fn estimated_memory_bytes(&self) -> usize {
        // Base overhead + estimated memory per entry
        // Each entry contains approximately: CachedPlan structure + ExecutionPlan + query template string
        const PER_ENTRY_OVERHEAD: usize = 1024; // Estimates of the overhead for structures and execution plans
        self.total_query_template_bytes + (self.current_entries * PER_ENTRY_OVERHEAD)
    }
}

/// Query plan cache
///
/// Provide a query plan cache in the style of a Prepared Statement
pub struct QueryPlanCache {
    /// Cache storage
    cache: Mutex<LruCache<PlanCacheKey, Arc<CachedPlan>>>,
    /// Configuration
    config: PlanCacheConfig,
    /// Statistical information
    stats: Mutex<PlanCacheStats>,
}

impl std::fmt::Debug for QueryPlanCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.stats.lock();
        f.debug_struct("QueryPlanCache")
            .field("config", &self.config)
            .field("stats", &*stats)
            .finish()
    }
}

impl QueryPlanCache {
    /// Create a new query plan cache.
    pub fn new(config: PlanCacheConfig) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(config.max_entries).expect("缓存条目数必须大于0"),
            )),
            config,
            stats: Mutex::new(PlanCacheStats::default()),
        }
    }

    /// Obtaining the cached plan
    ///
    /// # Parameters
    /// `query`: The text of the query
    ///
    /// # Back
    /// - `Some(Arc<CachedPlan>)`: 缓存的计划
    /// “None”: No results were found, or there was a hash collision.
    pub fn get(&self, query: &str) -> Option<Arc<CachedPlan>> {
        let key = PlanCacheKey::from_query(query);
        let ttl = Duration::from_secs(self.config.ttl_seconds);

        let mut cache = self.cache.lock();
        let mut stats = self.stats.lock();

        if let Some(plan) = cache.get(&key) {
            // Check whether it has expired.
            if plan.created_at.elapsed() > ttl {
                // Expired; remove it and return None.
                cache.pop(&key);
                stats.expirations += 1;
                stats.misses += 1;
                return None;
            }

            // Hash collision detection: Verifying whether the query text matches a certain value.
            if plan.query_template != query {
                // A hash collision occurred; the event was logged, and None was returned.
                log::warn!(
                    "查询计划缓存哈希冲突 detected: hash={}, expected_query={}, cached_query={}",
                    key.hash,
                    query,
                    plan.query_template
                );
                stats.misses += 1;
                return None;
            }

            // Update the access statistics.
            stats.hits += 1;
            return Some(plan.clone());
        }

        stats.misses += 1;
        None
    }

    /// Put the plan in the cache.
    ///
    /// # 参数
    /// - `query`: 查询文本
    /// Execute the plan.
    /// `param_positions`: Information about the positions of the parameters
    pub fn put(&self, query: &str, plan: ExecutionPlan, param_positions: Vec<ParamPosition>) {
        let key = PlanCacheKey::from_query(query);
        let query_bytes = query.len();

        let cached_plan = Arc::new(CachedPlan {
            query_template: query.to_string(),
            plan,
            param_positions,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            avg_execution_time_ms: 0.0,
            execution_count: 0,
        });

        let mut cache = self.cache.lock();
        let old_len = cache.len();

        // Check whether it is an update to an existing entry.
        let is_update = cache.contains(&key);

        cache.put(key, cached_plan);
        let new_len = cache.len();

        // Update the statistics.
        let mut stats = self.stats.lock();
        if new_len <= old_len && old_len > 0 && !is_update {
            // A elimination has taken place.
            stats.evictions += 1;
        }

        // Update the size statistics.
        if is_update {
            // Update the existing entry and recalculate the total size.
            stats.total_query_template_bytes = cache
                .iter()
                .map(|(_, plan)| plan.query_template.len())
                .sum();
        } else {
            // New entry
            stats.total_query_template_bytes += query_bytes;
        }

        stats.current_entries = new_len;
        if stats.current_entries > 0 {
            stats.avg_query_template_bytes =
                stats.total_query_template_bytes / stats.current_entries;
        }
    }

    /// Record the statistics on the execution of the plan.
    ///
    /// # 参数
    /// - `query`: 查询文本
    /// `execution_time_ms`: Execution time (in milliseconds)
    pub fn record_execution(&self, query: &str, execution_time_ms: f64) {
        let key = PlanCacheKey::from_query(query);

        let mut cache = self.cache.lock();
        if let Some(plan) = cache.get_mut(&key) {
            // Use `Arc::make_mut` to obtain a mutable reference.
            let plan_mut = Arc::make_mut(plan);
            plan_mut.execution_count += 1;

            // Update of the average execution time (Exponential Moving Average)
            let alpha = 0.1; // Smoothing factor
            plan_mut.avg_execution_time_ms =
                plan_mut.avg_execution_time_ms * (1.0 - alpha) + execution_time_ms * alpha;
        }
    }

    /// Check whether the query has been cached.
    pub fn contains(&self, query: &str) -> bool {
        let key = PlanCacheKey::from_query(query);
        let cache = self.cache.lock();
        cache.contains(&key)
    }

    /// Invalidate the cache entry
    pub fn invalidate(&self, query: &str) -> bool {
        let key = PlanCacheKey::from_query(query);
        let mut cache = self.cache.lock();
        let removed = cache.pop(&key).is_some();

        if removed {
            let mut stats = self.stats.lock();
            stats.current_entries = cache.len();
        }

        removed
    }

    /// Clear all caches.
    pub fn clear(&self) {
        let mut cache = self.cache.lock();
        cache.clear();

        let mut stats = self.stats.lock();
        stats.current_entries = 0;
        stats.total_query_template_bytes = 0;
        stats.avg_query_template_bytes = 0;
    }

    /// Obtain statistical information
    pub fn stats(&self) -> PlanCacheStats {
        let mut stats = self.stats.lock();
        stats.current_entries = self.cache.lock().len();
        stats.clone()
    }

    /// Clean up expired entries.
    pub fn cleanup_expired(&self) {
        let ttl = Duration::from_secs(self.config.ttl_seconds);
        let mut cache = self.cache.lock();
        let mut stats = self.stats.lock();

        let expired_keys: Vec<_> = cache
            .iter()
            .filter(|(_, plan)| plan.created_at.elapsed() > ttl)
            .map(|(k, _)| k.clone())
            .collect();

        for key in expired_keys {
            cache.pop(&key);
            stats.expirations += 1;
        }

        stats.current_entries = cache.len();
    }

    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.cache.lock().len()
    }

    /// Check whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.lock().is_empty()
    }
}

impl Default for QueryPlanCache {
    fn default() -> Self {
        Self::new(PlanCacheConfig::default())
    }
}

/// Parameterized query processor
///
/// Handling the parsing and binding of parameterized queries
pub struct ParameterizedQueryHandler {
    /// Parameter placeholder pattern
    placeholder_pattern: regex::Regex,
}

impl ParameterizedQueryHandler {
    /// Create a new parametric query processor.
    pub fn new() -> Self {
        Self {
            placeholder_pattern: regex::Regex::new(r"\$(\d+|[a-zA-Z_][a-zA-Z0-9_]*)")
                .expect("占位符正则表达式编译失败"),
        }
    }

    /// Extract the parameter positions from the query.
    ///
    /// # 参数
    /// - `query`: 查询文本
    ///
    /// # 返回
    /// Parameter Location List
    pub fn extract_params(&self, query: &str) -> Vec<ParamPosition> {
        let mut positions = Vec::new();

        for (idx, cap) in self.placeholder_pattern.captures_iter(query).enumerate() {
            let mat = cap.get(0).expect("正则表达式捕获组不应为空");
            let param_str = &cap[1];

            // Determine if it is a positional or named parameter
            let (name, index) = if let Ok(num) = param_str.parse::<usize>() {
                (None, num.saturating_sub(1)) // $1 对应索引 0
            } else {
                (Some(param_str.to_string()), idx)
            };

            positions.push(ParamPosition {
                index,
                name,
                position: mat.start(),
                expected_type: None, // Types are determined during the validation phase
            });
        }

        positions
    }

    /// Binding parameters to a query template
    ///
    /// # 参数
    /// - `template`: query template
    /// - `params`: parameter values
    ///
    /// # 返回
    /// Full query after binding
    pub fn bind_params(&self, template: &str, params: &[crate::core::Value]) -> DBResult<String> {
        let positions = self.extract_params(template);

        if positions.len() != params.len() {
            return Err(DBError::Validation(format!(
                "参数数量不匹配: 期望 {}, 实际 {}",
                positions.len(),
                params.len()
            )));
        }

        let mut result = template.to_string();
        let param_strings: Vec<String> = params.iter().map(|v| format!("{}", v)).collect();

        // Replacement from back to front to avoid positional shifts
        for (pos, value) in positions.iter().zip(param_strings.iter()).rev() {
            result.replace_range(pos.position..pos.position + 2, value);
        }

        Ok(result)
    }
}

impl Default for ParameterizedQueryHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_cache_creation() {
        let cache = QueryPlanCache::default();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_plan_cache_key() {
        let key1 = PlanCacheKey::from_query("MATCH (n) RETURN n");
        let key2 = PlanCacheKey::from_query("MATCH (n) RETURN n");
        let key3 = PlanCacheKey::from_query("MATCH (m) RETURN m");

        assert_eq!(key1.hash, key2.hash);
        assert_ne!(key1.hash, key3.hash);
    }

    #[test]
    fn test_cache_stats() {
        let cache = QueryPlanCache::default();
        let stats = cache.stats();

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate(), 0.0);
    }

    #[test]
    fn test_param_handler_creation() {
        let handler = ParameterizedQueryHandler::new();
        let positions = handler.extract_params("SELECT * FROM t WHERE id = $1 AND name = $2");

        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].index, 0);
        assert_eq!(positions[1].index, 1);
    }
}
