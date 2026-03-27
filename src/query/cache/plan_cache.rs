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

use moka::sync::Cache;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::error::{DBError, DBResult};
use crate::query::planning::plan::ExecutionPlan;

/// Cache priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for CachePriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// TTL configuration
#[derive(Debug, Clone)]
pub struct TtlConfig {
    pub base_ttl_seconds: u64,
    pub adaptive: bool,
    pub min_ttl_seconds: u64,
    pub max_ttl_seconds: u64,
}

impl Default for TtlConfig {
    fn default() -> Self {
        Self {
            base_ttl_seconds: 3600,
            adaptive: true,
            min_ttl_seconds: 300,
            max_ttl_seconds: 86400,
        }
    }
}

/// Priority configuration
#[derive(Debug, Clone)]
pub struct PriorityConfig {
    pub enable_priority: bool,
    pub track_execution_time: bool,
}

impl Default for PriorityConfig {
    fn default() -> Self {
        Self {
            enable_priority: true,
            track_execution_time: true,
        }
    }
}

/// Query Plan Cache Configuration
#[derive(Debug, Clone)]
pub struct PlanCacheConfig {
    /// Maximum number of cache entries
    pub max_entries: usize,
    /// Memory budget (bytes)
    pub memory_budget: usize,
    /// Whether to enable parameterized query support
    pub enable_parameterized: bool,
    /// TTL configuration
    pub ttl_config: TtlConfig,
    /// Priority configuration
    pub priority_config: PriorityConfig,
}

impl Default for PlanCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            memory_budget: 50 * 1024 * 1024,
            enable_parameterized: true,
            ttl_config: TtlConfig::default(),
            priority_config: PriorityConfig::default(),
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
    /// Cache priority
    pub priority: CachePriority,
    /// Plan complexity score (for eviction decisions)
    pub complexity_score: u32,
    /// Estimated compute cost (milliseconds)
    pub estimated_compute_cost: u64,
    /// Current TTL
    pub current_ttl: Duration,
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
#[derive(Debug, Clone)]
pub struct PlanCacheStats {
    /// Number of hits
    pub hits: Arc<AtomicU64>,
    /// Number of missed hits
    pub misses: Arc<AtomicU64>,
    /// Number of eliminations
    pub evictions: Arc<AtomicU64>,
    /// Number of expiration dates
    pub expirations: Arc<AtomicU64>,
    /// Number of current cache entries
    pub current_entries: Arc<AtomicUsize>,
    /// Average query template size (bytes)
    pub avg_query_template_bytes: Arc<RwLock<usize>>,
    /// Total query template size (bytes)
    pub total_query_template_bytes: Arc<AtomicUsize>,
}

impl Default for PlanCacheStats {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanCacheStats {
    /// Create new statistics
    pub fn new() -> Self {
        Self {
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicU64::new(0)),
            expirations: Arc::new(AtomicU64::new(0)),
            current_entries: Arc::new(AtomicUsize::new(0)),
            avg_query_template_bytes: Arc::new(RwLock::new(0)),
            total_query_template_bytes: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// hit rate
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Estimate total memory footprint (based on average template size and number of entries)
    pub fn estimated_memory_bytes(&self) -> usize {
        const PER_ENTRY_OVERHEAD: usize = 1024;
        let total_bytes = self.total_query_template_bytes.load(Ordering::Relaxed);
        let entries = self.current_entries.load(Ordering::Relaxed);
        total_bytes + (entries * PER_ENTRY_OVERHEAD)
    }
}

/// Query plan cache
///
/// Provide a query plan cache in the style of a Prepared Statement
pub struct QueryPlanCache {
    /// Cache storage - using moka for high-performance concurrent access
    cache: Cache<PlanCacheKey, Arc<CachedPlan>>,
    /// Configuration
    config: PlanCacheConfig,
    /// Statistical information - using RwLock for read-heavy scenarios
    stats: Arc<RwLock<PlanCacheStats>>,
}

impl std::fmt::Debug for QueryPlanCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.stats.read();
        f.debug_struct("QueryPlanCache")
            .field("config", &self.config)
            .field("stats", &*stats)
            .finish()
    }
}

impl QueryPlanCache {
    /// Create a new query plan cache.
    pub fn new(config: PlanCacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_entries as u64)
            .time_to_live(Duration::from_secs(config.ttl_config.base_ttl_seconds))
            .build();

        Self {
            cache,
            config,
            stats: Arc::new(RwLock::new(PlanCacheStats::new())),
        }
    }

    /// Obtaining the cached plan
    ///
    /// # Parameters
    /// - `query`: The text of the query
    ///
    /// # Returns
    /// - `Some(Arc<CachedPlan>)`: Cached plan
    /// - `None`: No results were found, or there was a hash collision.
    pub fn get(&self, query: &str) -> Option<Arc<CachedPlan>> {
        let key = PlanCacheKey::from_query(query);

        if let Some(plan) = self.cache.get(&key) {
            let stats = self.stats.read();

            // Hash collision detection: Verifying whether the query text matches a certain value.
            if plan.query_template != query {
                // A hash collision occurred; the event was logged, and None was returned.
                log::warn!(
                    "查询计划缓存哈希冲突 detected: hash={}, expected_query={}, cached_query={}",
                    key.hash,
                    query,
                    plan.query_template
                );
                stats.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }

            // Update the access statistics.
            stats.hits.fetch_add(1, Ordering::Relaxed);
            return Some(plan);
        }

        let stats = self.stats.read();
        stats.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Put the plan in the cache.
    ///
    /// # Parameters
    /// - `query`: Query text
    /// - `plan`: Execution plan
    /// - `param_positions`: Information about the positions of the parameters
    pub fn put(&self, query: &str, plan: ExecutionPlan, param_positions: Vec<ParamPosition>) {
        let key = PlanCacheKey::from_query(query);
        let query_bytes = query.len();

        let priority = if self.config.priority_config.enable_priority {
            self.calculate_priority(&plan)
        } else {
            CachePriority::Normal
        };

        let complexity_score = self.calculate_complexity_score(&plan);
        let estimated_compute_cost = self.estimate_compute_cost(&plan);
        let current_ttl = Duration::from_secs(self.config.ttl_config.base_ttl_seconds);

        let cached_plan = Arc::new(CachedPlan {
            query_template: query.to_string(),
            plan,
            param_positions,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            avg_execution_time_ms: 0.0,
            execution_count: 0,
            priority,
            complexity_score,
            estimated_compute_cost,
            current_ttl,
        });

        let is_update = self.cache.contains_key(&key);
        self.cache.insert(key, cached_plan);

        let stats = self.stats.read();
        if !is_update {
            stats.total_query_template_bytes.fetch_add(query_bytes, Ordering::Relaxed);
        }

        let current_entries = self.cache.entry_count() as usize;
        stats.current_entries.store(current_entries, Ordering::Relaxed);
        if current_entries > 0 {
            let total_bytes = stats.total_query_template_bytes.load(Ordering::Relaxed);
            let mut avg_bytes = stats.avg_query_template_bytes.write();
            *avg_bytes = total_bytes / current_entries;
        }
    }

    /// Calculate priority based on query characteristics
    fn calculate_priority(&self, plan: &ExecutionPlan) -> CachePriority {
        let complexity = self.calculate_complexity_score(plan);

        if complexity > 1000 {
            CachePriority::High
        } else if complexity > 100 {
            CachePriority::Normal
        } else {
            CachePriority::Low
        }
    }

    /// Calculate complexity score for a plan
    fn calculate_complexity_score(&self, _plan: &ExecutionPlan) -> u32 {
        let mut score = 0u32;

        score += 10;
        score += 5;
        score += 20;
        score += 15;
        score += 30;
        score += 25;

        score
    }

    /// Estimate compute cost in milliseconds
    fn estimate_compute_cost(&self, plan: &ExecutionPlan) -> u64 {
        let complexity = self.calculate_complexity_score(plan);
        (complexity as u64 * 10).max(100)
    }

    /// Update TTL adaptively based on access patterns
    fn update_ttl(&self, entry: &mut CachedPlan) {
        if !self.config.ttl_config.adaptive {
            return;
        }

        let hit_rate =
            entry.access_count as f64 / (entry.created_at.elapsed().as_secs() as f64 / 60.0 + 1.0);

        if hit_rate > 10.0 {
            entry.current_ttl = Duration::from_secs(
                (entry.current_ttl.as_secs() as f64 * 1.5)
                    .min(self.config.ttl_config.max_ttl_seconds as f64) as u64,
            );
        } else if hit_rate < 1.0 {
            entry.current_ttl = Duration::from_secs(
                (entry.current_ttl.as_secs() as f64 * 0.8)
                    .max(self.config.ttl_config.min_ttl_seconds as f64) as u64,
            );
        }
    }

    /// Record the statistics on the execution of the plan.
    ///
    /// # Parameter
    /// - `query`: Query content
    /// - `execution_time_ms`: Execution time (in milliseconds)
    pub fn record_execution(&self, query: &str, execution_time_ms: f64) {
        let key = PlanCacheKey::from_query(query);

        if let Some(plan) = self.cache.get(&key) {
            // Update the average execution time (Exponential Moving Average)
            let alpha = 0.1; // Smoothing factor
            let new_avg = plan.avg_execution_time_ms * (1.0 - alpha) + execution_time_ms * alpha;

            // Create updated plan with new stats
            let updated_plan = Arc::new(CachedPlan {
                execution_count: plan.execution_count + 1,
                avg_execution_time_ms: new_avg,
                ..(*plan).clone()
            });

            self.cache.insert(key, updated_plan);
        }
    }

    /// Check whether the query has been cached.
    pub fn contains(&self, query: &str) -> bool {
        let key = PlanCacheKey::from_query(query);
        self.cache.contains_key(&key)
    }

    /// Invalidate the cache entry
    pub fn invalidate(&self, query: &str) -> bool {
        let key = PlanCacheKey::from_query(query);
        let removed = self.cache.remove(&key).is_some();

        if removed {
            let stats = self.stats.read();
            stats.current_entries.store(self.cache.entry_count() as usize, Ordering::Relaxed);
        }

        removed
    }

    /// Get cache entries for eviction (internal use)
    pub fn get_cache_entries(&self) -> Vec<(Arc<PlanCacheKey>, f64, usize)> {
        self.cache
            .iter()
            .map(|(k, v)| {
                let value_score = v.access_count as f64 * 0.5
                    + v.estimated_compute_cost as f64 * 0.3
                    - v.query_template.len() as f64 * 0.2;
                (k.clone(), value_score, v.query_template.len())
            })
            .collect()
    }

    /// Increment eviction count (internal use)
    pub fn increment_eviction_count(&self, count: u64) {
        let stats = self.stats.read();
        stats.evictions.fetch_add(count, Ordering::Relaxed);
    }

    /// Clear all caches.
    pub fn clear(&self) {
        self.cache.invalidate_all();

        let stats = self.stats.read();
        stats.current_entries.store(0, Ordering::Relaxed);
        stats.total_query_template_bytes.store(0, Ordering::Relaxed);
    }

    /// Obtain statistical information
    pub fn stats(&self) -> PlanCacheStats {
        let stats = self.stats.read();
        stats.current_entries.store(self.cache.entry_count() as usize, Ordering::Relaxed);
        stats.clone()
    }

    /// Clean up expired entries.
    /// Note: moka handles TTL automatically, so this is a no-op
    pub fn cleanup_expired(&self) {
        // moka handles TTL automatically, no manual cleanup needed
    }

    /// Get the number of cached entries
    pub fn len(&self) -> usize {
        self.cache.entry_count() as usize
    }

    /// Check whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.entry_count() == 0
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
    use std::sync::atomic::Ordering;

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

        assert_eq!(stats.hits.load(Ordering::Relaxed), 0);
        assert_eq!(stats.misses.load(Ordering::Relaxed), 0);
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
