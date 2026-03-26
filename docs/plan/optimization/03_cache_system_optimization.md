# 缓存系统优化方案

**文档版本**: 1.0  
**创建日期**: 2026-03-26  
**相关模块**: query/cache

---

## 一、现状分析

### 1.1 当前缓存架构

当前缓存系统包含两个主要组件：

```
┌─────────────────────────────────────────────────────────────┐
│                      CacheManager                            │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────────┐  ┌──────────────────────────────┐ │
│  │   QueryPlanCache     │  │      CteCacheManager         │ │
│  │   (查询计划缓存)      │  │      (CTE 结果缓存)           │ │
│  │                      │  │                              │ │
│  │  - 默认 1000 条目     │  │  - 默认 64MB 内存限制          │ │
│  │  - TTL: 1 小时        │  │  - 行数限制: 100-100,000      │ │
│  │  - 纯 LRU 淘汰        │  │  - LRU + 大小限制             │ │
│  └──────────────────────┘  └──────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 存在的问题

| 问题 | Plan Cache | CTE Cache | 影响 |
|------|------------|-----------|------|
| 无统一内存管理 | 各自独立 | 各自独立 | 无法全局控制内存使用 |
| 固定配置 | 无法根据负载调整 | 无法根据负载调整 | 不适应不同场景 |
| 无优先级 | 所有计划同等对待 | 所有 CTE 同等对待 | 重要缓存可能被淘汰 |
| 淘汰策略简单 | 纯 LRU | LRU + 大小 | 未考虑访问频率和价值 |
| 无预热机制 | 冷启动为空 | 冷启动为空 | 首次查询性能差 |

---

## 二、优化目标

1. **统一缓存管理**: 全局内存预算，统一监控
2. **分级缓存策略**: 按重要性和访问模式分级
3. **智能淘汰**: 综合考虑访问频率、大小、计算成本
4. **自适应配置**: 根据负载自动调整缓存大小
5. **预热机制**: 减少冷启动影响
6. **预期收益**: 提高 15-30% 缓存命中率，防止内存溢出

---

## 三、具体优化方案

### 3.1 全局缓存管理器

#### 统一缓存管理架构

```rust
// src/query/cache/global_manager.rs (新建)

/// 全局缓存管理器
/// 
/// 统一管理所有缓存，协调内存分配，提供统一监控接口
pub struct GlobalCacheManager {
    /// 总内存预算
    total_budget: usize,
    /// 各缓存分配比例
    allocations: CacheAllocations,
    /// 子缓存管理器
    plan_cache: Arc<QueryPlanCache>,
    cte_cache: Arc<CteCacheManager>,
    /// 当前内存使用
    current_usage: AtomicUsize,
    /// 统计信息
    stats: RwLock<GlobalCacheStats>,
    /// 是否启用紧急淘汰
    emergency_eviction: AtomicBool,
}

/// 缓存分配配置
#[derive(Debug, Clone)]
pub struct CacheAllocations {
    /// 计划缓存分配比例 (0.0 - 1.0)
    pub plan_cache_ratio: f64,
    /// CTE 缓存分配比例 (0.0 - 1.0)
    pub cte_cache_ratio: f64,
    /// 预留比例（用于突发分配）
    pub reserve_ratio: f64,
}

impl Default for CacheAllocations {
    fn default() -> Self {
        Self {
            plan_cache_ratio: 0.4,  // 40%
            cte_cache_ratio: 0.4,   // 40%
            reserve_ratio: 0.2,     // 20%
        }
    }
}

/// 全局缓存统计
#[derive(Debug, Clone, Default)]
pub struct GlobalCacheStats {
    /// 总命中次数
    pub total_hits: u64,
    /// 总未命中次数
    pub total_misses: u64,
    /// 总内存使用（字节）
    pub total_memory: usize,
    /// 总内存预算（字节）
    pub total_budget: usize,
    /// 淘汰次数
    pub evictions: u64,
    /// 紧急淘汰次数
    pub emergency_evictions: u64,
    /// 各缓存统计
    pub plan_cache_stats: PlanCacheStats,
    pub cte_cache_stats: CteCacheStats,
}

impl GlobalCacheManager {
    /// 创建新的全局缓存管理器
    pub fn new(
        total_budget: usize,
        allocations: CacheAllocations,
    ) -> Self {
        // 计算各缓存的预算
        let plan_budget = (total_budget as f64 * allocations.plan_cache_ratio) as usize;
        let cte_budget = (total_budget as f64 * allocations.cte_cache_ratio) as usize;
        
        let plan_cache = Arc::new(QueryPlanCache::with_budget(plan_budget));
        let cte_cache = Arc::CteCacheManager::with_budget(cte_budget);
        
        Self {
            total_budget,
            allocations,
            plan_cache,
            cte_cache,
            current_usage: AtomicUsize::new(0),
            stats: RwLock::new(GlobalCacheStats::default()),
            emergency_eviction: AtomicBool::new(false),
        }
    }

    /// 检查内存使用，必要时触发淘汰
    pub fn check_memory_pressure(&self) {
        let usage = self.current_usage.load(Ordering::Relaxed);
        let threshold = (self.total_budget as f64 * 0.9) as usize;
        
        if usage > threshold {
            // 触发紧急淘汰
            self.emergency_evict();
        }
    }

    /// 紧急淘汰 - 按优先级淘汰缓存
    fn emergency_evict(&self) {
        if self.emergency_eviction.swap(true, Ordering::SeqCst) {
            return; // 已有紧急淘汰在进行
        }
        
        let target = (self.total_budget as f64 * 0.7) as usize; // 淘汰到 70%
        let mut evicted = 0;
        
        // 1. 先淘汰 CTE 缓存中的低优先级条目
        evicted += self.cte_cache.evict_low_priority(target - evicted);
        
        // 2. 再淘汰计划缓存中的低命中率条目
        if evicted < target {
            evicted += self.plan_cache.evict_low_hit_rate(target - evicted);
        }
        
        // 3. 最后强制淘汰最老的条目
        if evicted < target {
            evicted += self.force_evict_oldest(target - evicted);
        }
        
        self.stats.write().emergency_evictions += 1;
        self.emergency_eviction.store(false, Ordering::SeqCst);
        
        log::warn!("Emergency cache eviction completed: {} bytes freed", evicted);
    }

    /// 强制淘汰最老的条目
    fn force_evict_oldest(&self, target: usize) -> usize {
        // 实现最老条目淘汰逻辑
        0
    }

    /// 获取统计信息
    pub fn stats(&self) -> GlobalCacheStats {
        let mut stats = self.stats.read().clone();
        stats.plan_cache_stats = self.plan_cache.stats();
        stats.cte_cache_stats = self.cte_cache.get_stats();
        stats.total_memory = self.current_usage.load(Ordering::Relaxed);
        stats.total_budget = self.total_budget;
        stats
    }
}
```

---

### 3.2 查询计划缓存优化

#### 优化后的计划缓存

```rust
// src/query/cache/plan_cache.rs

/// 增强的计划缓存配置
#[derive(Debug, Clone)]
pub struct PlanCacheConfig {
    /// 最大条目数
    pub max_entries: usize,
    /// 内存预算（字节）
    pub memory_budget: usize,
    /// 基础 TTL（秒）
    pub ttl_seconds: u64,
    /// 是否启用自适应 TTL
    pub adaptive_ttl: bool,
    /// 最小 TTL（秒）
    pub min_ttl_seconds: u64,
    /// 最大 TTL（秒）
    pub max_ttl_seconds: u64,
    /// 是否启用优先级
    pub enable_priority: bool,
    /// 是否统计执行时间
    pub track_execution_time: bool,
}

impl Default for PlanCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            memory_budget: 50 * 1024 * 1024, // 50MB
            ttl_seconds: 3600,
            adaptive_ttl: true,
            min_ttl_seconds: 300,   // 5 分钟
            max_ttl_seconds: 86400, // 24 小时
            enable_priority: true,
            track_execution_time: true,
        }
    }
}

/// 带优先级的缓存条目
#[derive(Debug, Clone)]
pub struct CachedPlan {
    // 原有字段...
    pub query_template: String,
    pub plan: ExecutionPlan,
    pub param_positions: Vec<ParamPosition>,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub avg_execution_time_ms: f64,
    pub execution_count: u64,
    
    // 新增字段
    /// 缓存优先级
    pub priority: CachePriority,
    /// 计划复杂度评分（用于淘汰决策）
    pub complexity_score: u32,
    /// 计算成本估算（毫秒）
    pub estimated_compute_cost: u64,
    /// 当前 TTL
    pub current_ttl: Duration,
}

/// 缓存优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// 增强的计划缓存
pub struct QueryPlanCache {
    cache: RwLock<LruCache<PlanCacheKey, Arc<CachedPlan>>>,
    config: PlanCacheConfig,
    stats: RwLock<PlanCacheStats>,
    /// 当前内存使用（估算）
    current_memory: AtomicUsize,
}

impl QueryPlanCache {
    /// 根据查询特征计算优先级
    fn calculate_priority(&self, plan: &ExecutionPlan) -> CachePriority {
        // 基于计划复杂度、预期使用频率等计算优先级
        let complexity = plan.complexity_score();
        
        if complexity > 1000 {
            CachePriority::High
        } else if complexity > 100 {
            CachePriority::Normal
        } else {
            CachePriority::Low
        }
    }

    /// 自适应 TTL 更新
    fn update_ttl(&self, entry: &mut CachedPlan) {
        if !self.config.adaptive_ttl {
            return;
        }
        
        let hit_rate = entry.access_count as f64 / 
            (entry.created_at.elapsed().as_secs() as f64 / 60.0 + 1.0);
        
        // 访问频率高，延长 TTL
        if hit_rate > 10.0 {
            entry.current_ttl = Duration::from_secs(
                (entry.current_ttl.as_secs() as f64 * 1.5)
                    .min(self.config.max_ttl_seconds as f64) as u64
            );
        } 
        // 访问频率低，缩短 TTL
        else if hit_rate < 1.0 {
            entry.current_ttl = Duration::from_secs(
                (entry.current_ttl.as_secs() as f64 * 0.8)
                    .max(self.config.min_ttl_seconds as f64) as u64
            );
        }
    }

    /// 基于价值的淘汰
    pub fn evict_low_value(&self, target_bytes: usize) -> usize {
        let mut freed = 0;
        let mut to_remove = Vec::new();
        
        {
            let cache = self.cache.read();
            
            // 计算每个条目的价值分数
            let mut entries: Vec<_> = cache.iter()
                .map(|(k, v)| {
                    let value_score = v.access_count as f64 * v.estimated_compute_cost as f64 
                        / v.query_template.len() as f64;
                    (k.clone(), value_score, v.query_template.len())
                })
                .collect();
            
            // 按价值排序，淘汰低价值条目
            entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            for (key, _, size) in entries {
                if freed >= target_bytes {
                    break;
                }
                to_remove.push(key);
                freed += size;
            }
        }
        
        // 执行淘汰
        let mut cache = self.cache.write();
        for key in to_remove {
            cache.pop(&key);
        }
        
        freed
    }

    /// 淘汰低命中率条目
    pub fn evict_low_hit_rate(&self, target_bytes: usize) -> usize {
        // 实现低命中率淘汰逻辑
        self.evict_low_value(target_bytes)
    }
}
```

---

### 3.3 CTE 缓存优化

#### 增强的 CTE 缓存

```rust
// src/query/cache/cte_cache.rs

/// 增强的 CTE 缓存配置
#[derive(Debug, Clone)]
pub struct CteCacheConfig {
    /// 最大内存（字节）
    pub max_size: usize,
    /// 单个条目最大大小
    pub max_entry_size: usize,
    /// 最小行数
    pub min_row_count: u64,
    /// 最大行数
    pub max_row_count: u64,
    /// 基础 TTL
    pub entry_ttl_seconds: u64,
    /// 是否启用自适应
    pub adaptive: bool,
    /// 是否启用优先级
    pub enable_priority: bool,
    /// 启用缓存
    pub enabled: bool,
}

impl Default for CteCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 64 * 1024 * 1024,       // 64MB
            max_entry_size: 10 * 1024 * 1024, // 10MB
            min_row_count: 100,
            max_row_count: 100_000,
            entry_ttl_seconds: 3600,
            adaptive: true,
            enable_priority: true,
            enabled: true,
        }
    }
}

/// 增强的 CTE 缓存条目
#[derive(Debug, Clone)]
pub struct CteCacheEntry {
    // 原有字段...
    pub data: Arc<Vec<u8>>,
    pub row_count: u64,
    pub data_size: usize,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub reuse_probability: f64,
    pub cte_hash: String,
    pub cte_definition: String,
    
    // 新增字段
    /// 缓存优先级
    pub priority: CachePriority,
    /// 计算成本（毫秒）
    pub compute_cost_ms: u64,
    /// 访问频率（每分钟）
    pub access_frequency: f64,
    /// 依赖的表（用于失效检测）
    pub dependent_tables: Vec<String>,
}

impl CteCacheEntry {
    /// 计算缓存价值分数（用于淘汰决策）
    pub fn value_score(&self) -> f64 {
        let frequency_factor = self.access_frequency.sqrt();
        let cost_factor = (self.compute_cost_ms as f64 / 1000.0).sqrt();
        let size_factor = (self.data_size as f64 / 1024.0 / 1024.0).max(0.1);
        let priority_factor = self.priority as i32 as f64 + 1.0;
        
        // 价值 = (访问频率 * 计算成本 * 优先级) / 大小
        (frequency_factor * cost_factor * priority_factor) / size_factor
    }
}

/// 增强的 CTE 缓存管理器
pub struct CteCacheManager {
    cache: RwLock<HashMap<String, CteCacheEntry>>,
    config: RwLock<CteCacheConfig>,
    stats: RwLock<CteCacheStats>,
    current_memory: AtomicUsize,
    /// 按优先级组织的条目索引
    priority_index: RwLock<HashMap<CachePriority, Vec<String>>>,
}

impl CteCacheManager {
    /// 智能缓存决策
    pub fn should_cache(&self, row_count: u64, estimated_cost_ms: u64) -> CteCacheDecision {
        let config = self.config.read();
        
        if !config.enabled {
            return CteCacheDecision::no("Caching disabled");
        }
        
        // 行数检查
        if row_count < config.min_row_count {
            return CteCacheDecision::no("Row count too small");
        }
        if row_count > config.max_row_count {
            return CteCacheDecision::no("Row count too large");
        }
        
        // 基于计算成本的决策
        let reuse_probability = if estimated_cost_ms > 1000 {
            0.9 // 高成本查询，高概率复用
        } else if estimated_cost_ms > 100 {
            0.7
        } else {
            0.5
        };
        
        CteCacheDecision {
            should_cache: true,
            reason: "Passed all checks".to_string(),
            reuse_probability,
            estimated_benefit: estimated_cost_ms as f64 * reuse_probability,
            suggested_priority: if estimated_cost_ms > 1000 {
                CachePriority::High
            } else {
                CachePriority::Normal
            },
        }
    }

    /// 淘汰低优先级条目
    pub fn evict_low_priority(&self, target_bytes: usize) -> usize {
        let mut freed = 0;
        let mut to_remove = Vec::new();
        
        {
            let cache = self.cache.read();
            let priority_index = self.priority_index.read();
            
            // 按优先级从低到高淘汰
            for priority in [CachePriority::Low, CachePriority::Normal] {
                if let Some(keys) = priority_index.get(&priority) {
                    for key in keys {
                        if freed >= target_bytes {
                            break;
                        }
                        if let Some(entry) = cache.get(key) {
                            to_remove.push(key.clone());
                            freed += entry.data_size;
                        }
                    }
                }
                if freed >= target_bytes {
                    break;
                }
            }
        }
        
        // 执行淘汰
        let mut cache = self.cache.write();
        let mut priority_index = self.priority_index.write();
        
        for key in to_remove {
            if let Some(entry) = cache.remove(&key) {
                *self.current_memory.get_mut() -= entry.data_size;
                
                // 更新优先级索引
                if let Some(keys) = priority_index.get_mut(&entry.priority) {
                    keys.retain(|k| k != &key);
                }
            }
        }
        
        freed
    }
}
```

---

### 3.4 缓存预热机制

```rust
// src/query/cache/warmup.rs (新建)

/// 缓存预热器
pub struct CacheWarmer {
    plan_cache: Arc<QueryPlanCache>,
    cte_cache: Arc<CteCacheManager>,
    /// 预热查询列表
    warmup_queries: Vec<String>,
    /// 预热 CTE 定义列表
    warmup_ctes: Vec<String>,
}

impl CacheWarmer {
    /// 从配置文件加载预热数据
    pub fn from_config(config_path: &Path) -> Result<Self, CacheError> {
        // 加载预热配置
        let config: WarmupConfig = serde_json::from_reader(
            File::open(config_path)?
        )?;
        
        Ok(Self {
            plan_cache: Arc::new(QueryPlanCache::default()),
            cte_cache: Arc::new(CteCacheManager::default()),
            warmup_queries: config.queries,
            warmup_ctes: config.ctes,
        })
    }

    /// 执行预热
    pub async fn warmup(&self, query_engine: &QueryEngine) {
        log::info!("Starting cache warmup...");
        
        // 预热查询计划
        for query in &self.warmup_queries {
            match query_engine.prepare(query).await {
                Ok(plan) => {
                    self.plan_cache.put(query, plan, vec![]);
                    log::debug!("Warmed up query plan: {}", query);
                }
                Err(e) => {
                    log::warn!("Failed to warmup query '{}': {}", query, e);
                }
            }
        }
        
        log::info!("Cache warmup completed: {} queries", self.warmup_queries.len());
    }

    /// 基于历史统计的自动预热
    pub async fn warmup_from_stats(&self, stats: &QueryStats) {
        // 获取最频繁的查询
        let top_queries = stats.most_frequent_queries(100);
        
        for (query, _) in top_queries {
            // 预热逻辑...
        }
    }
}

/// 预热配置
#[derive(Debug, Clone, Deserialize)]
pub struct WarmupConfig {
    pub queries: Vec<String>,
    pub ctes: Vec<String>,
}
```

---

## 四、配置预设

```rust
impl GlobalCacheManager {
    /// 最小内存配置 - 适用于嵌入式环境
    pub fn minimal() -> Self {
        Self::new(
            32 * 1024 * 1024, // 32MB 总预算
            CacheAllocations {
                plan_cache_ratio: 0.5,
                cte_cache_ratio: 0.3,
                reserve_ratio: 0.2,
            },
        )
    }

    /// 平衡配置 - 默认
    pub fn balanced() -> Self {
        Self::new(
            128 * 1024 * 1024, // 128MB
            CacheAllocations::default(),
        )
    }

    /// 高性能配置 - 适用于服务器环境
    pub fn high_performance() -> Self {
        Self::new(
            512 * 1024 * 1024, // 512MB
            CacheAllocations {
                plan_cache_ratio: 0.35,
                cte_cache_ratio: 0.45,
                reserve_ratio: 0.2,
            },
        )
    }
}
```

---

## 五、实施步骤

### 阶段一：创建全局缓存管理器（中风险）

1. 创建 `src/query/cache/global_manager.rs`
2. 实现 `GlobalCacheManager` 结构体
3. 实现内存预算和紧急淘汰逻辑

### 阶段二：增强现有缓存（中风险）

1. 修改 `PlanCacheConfig`，添加新字段
2. 修改 `CteCacheConfig`，添加优先级支持
3. 实现自适应 TTL 和价值评分

### 阶段三：实现预热机制（低风险）

1. 创建 `src/query/cache/warmup.rs`
2. 实现 `CacheWarmer` 结构体
3. 添加配置文件支持

### 阶段四：集成与测试（中风险）

1. 在 `QueryExecutionManager` 中集成全局缓存管理器
2. 添加监控指标收集
3. 进行性能测试

---

## 六、监控指标

```rust
/// 缓存性能指标
#[derive(Debug, Clone)]
pub struct CacheMetrics {
    /// 全局指标
    pub global: GlobalCacheStats,
    /// 计划缓存指标
    pub plan_cache: PlanCacheMetrics,
    /// CTE 缓存指标
    pub cte_cache: CteCacheMetrics,
    /// 时间戳
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct PlanCacheMetrics {
    pub hit_rate: f64,
    pub avg_ttl_seconds: f64,
    pub priority_distribution: HashMap<CachePriority, usize>,
    pub memory_usage: usize,
}

#[derive(Debug, Clone)]
pub struct CteCacheMetrics {
    pub hit_rate: f64,
    pub avg_value_score: f64,
    pub priority_distribution: HashMap<CachePriority, usize>,
    pub memory_usage: usize,
}
```

---

## 七、预期收益

| 指标 | 优化前 | 优化后 | 改善 |
|------|--------|--------|------|
| 计划缓存命中率 | ~70% | ~85% | +15% |
| CTE 缓存命中率 | ~60% | ~80% | +20% |
| 内存使用控制 | 无上限 | 严格预算 | 可控 |
| 冷启动性能 | 差 | 良好 | +40% |
| 自适应能力 | 无 | 完整支持 | 新功能 |
