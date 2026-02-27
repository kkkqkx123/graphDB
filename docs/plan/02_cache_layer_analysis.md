# 缓存层分析文档

## 概述

本文档分析 GraphDB 项目中缓存机制的当前实现状态，并提出可进一步改进的方向。

## 当前已实现的缓存

### 1. 查询计划缓存

**实现位置**: `src/query/planner/planner.rs`

**功能说明**:
- 使用 LRU (Least Recently Used) 缓存策略
- 默认缓存 1000 条查询计划
- 支持参数化查询（将具体参数替换为占位符）
- 缓存键包含查询模板、图空间 ID、语句类型和模式指纹

**关键代码**:
```rust
pub struct QueryPlanner {
    plan_cache: Arc<Mutex<LruCache<PlanCacheKey, Arc<ExecutionPlan>>>>,
    config: PlannerConfig,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PlanCacheKey {
    query_template: String,           // 参数化后的查询模板
    space_id: Option<i32>,            // 图空间 ID
    statement_type: SentenceKind,     // 语句类型
    pattern_fingerprint: Option<String>, // 模式指纹（MATCH 查询）
}
```

**配置选项**:
```rust
pub struct PlannerConfig {
    pub enable_caching: bool,         // 启用缓存
    pub cache_size: usize,            // 缓存大小（默认 1000）
    pub enable_rewrite: bool,         // 启用计划重写
    pub max_plan_depth: usize,        // 最大计划深度
}
```

**优化效果**:
- 避免重复解析和规划相同查询
- 参数化查询可复用计划（如 `MATCH (n) WHERE n.id = $id`）
- 减少 CPU 开销，提高查询响应速度

### 2. 表达式缓存

**实现位置**: `src/expression/context/cache_manager.rs`

**功能说明**:
- 正则表达式缓存（避免重复编译）
- 表达式解析缓存（字符串 -> ExpressionMeta）
- 日期时间解析缓存（字符串 -> DateValue/TimeValue/DateTimeValue）

**关键代码**:
```rust
#[derive(Debug, Clone)]
pub struct CacheManager {
    regex_cache: HashMap<String, Regex>,
    expression_cache: HashMap<String, ExpressionMeta>,
    date_cache: HashMap<String, DateValue>,
    time_cache: HashMap<String, TimeValue>,
    datetime_cache: HashMap<String, DateTimeValue>,
}

impl CacheManager {
    /// 获取或编译正则表达式
    pub fn get_regex(&mut self, pattern: &str) -> Option<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            if let Ok(regex) = Regex::new(pattern) {
                self.regex_cache.insert(pattern.to_string(), regex);
            } else {
                return None;
            }
        }
        self.regex_cache.get(pattern)
    }
}
```

**优化效果**:
- 正则表达式编译开销大，缓存后显著提升性能
- 日期时间解析避免重复计算
- 表达式解析结果复用

### 3. 执行器对象池

**实现位置**: `src/query/executor/object_pool.rs`

**功能说明**:
- 缓存执行器实例，减少频繁的内存分配和释放
- 支持多种执行器类型的独立缓存
- 提供命中/未命中统计

**关键代码**:
```rust
pub struct ExecutorObjectPool<S: StorageClient + 'static> {
    config: ObjectPoolConfig,
    pools: HashMap<String, Vec<ExecutorEnum<S>>>,
    stats: PoolStats,
}

#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    pub total_acquires: usize,    // 总获取次数
    pub total_releases: usize,    // 总释放次数
    pub cache_hits: usize,        // 缓存命中次数
    pub cache_misses: usize,      // 缓存未命中次数
}

impl<S: StorageClient> ExecutorObjectPool<S> {
    pub fn acquire(&mut self, executor_type: &str) -> Option<ExecutorEnum<S>> {
        if !self.config.enabled {
            return None;
        }
        
        self.stats.total_acquires += 1;
        
        if let Some(executors) = self.pools.get_mut(executor_type) {
            if let Some(executor) = executors.pop() {
                self.stats.cache_hits += 1;
                return Some(executor);
            }
        }
        
        self.stats.cache_misses += 1;
        None
    }
}
```

**配置选项**:
```rust
pub struct ObjectPoolConfig {
    pub max_pool_size: usize,    // 每种类型最大缓存数量（默认 10）
    pub enabled: bool,           // 是否启用
}
```

**优化效果**:
- 减少内存分配开销
- 减少垃圾回收压力
- 提高查询执行性能

### 4. Schema 缓存

**实现位置**: `src/storage/redb_storage.rs`

**功能说明**:
- RedbStorage 内部维护 Schema 信息
- 空间、标签、边类型等元数据内存缓存
- 减少元数据查询的存储访问

**关键组件**:
```rust
pub struct RedbStorage {
    schema_manager: Arc<RedbSchemaManager>,
    index_metadata_manager: Arc<RedbIndexMetadataManager>,
    extended_schema_manager: Arc<RedbExtendedSchemaManager>,
    // ...
}
```

## 可进一步改进的方向

### 1. 查询结果缓存（Result Cache）

**需求场景**:
- 配置表、字典表等变化少的查询
- 热点数据重复查询
- 计算复杂的聚合查询

**建议实现**:

```rust
use dashmap::DashMap;
use std::time::{Duration, Instant};

/// 查询结果缓存
pub struct ResultCache {
    /// 缓存存储（查询指纹 -> 缓存结果）
    cache: DashMap<QueryFingerprint, CachedResult>,
    /// 默认 TTL
    default_ttl: Duration,
    /// 最大缓存条目数
    max_entries: usize,
    /// 当前缓存大小（字节）
    current_size: AtomicUsize,
    /// 最大缓存大小（字节）
    max_size: usize,
}

/// 查询指纹（唯一标识查询）
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct QueryFingerprint {
    /// 查询字符串哈希
    query_hash: u64,
    /// 参数哈希
    params_hash: u64,
    /// 空间 ID
    space_id: Option<i64>,
}

/// 缓存结果
pub struct CachedResult {
    /// 结果数据
    data: Arc<QueryResult>,
    /// 创建时间
    created_at: Instant,
    /// 过期时间
    expires_at: Instant,
    /// 访问次数
    access_count: AtomicU64,
}

impl ResultCache {
    /// 获取缓存结果
    pub fn get(&self, fingerprint: &QueryFingerprint) -> Option<Arc<QueryResult>> {
        if let Some(entry) = self.cache.get(fingerprint) {
            if entry.expires_at > Instant::now() {
                entry.access_count.fetch_add(1, Ordering::Relaxed);
                return Some(Arc::clone(&entry.data));
            }
        }
        None
    }
    
    /// 存入缓存
    pub fn put(&self, fingerprint: QueryFingerprint, result: QueryResult, ttl: Option<Duration>) {
        let ttl = ttl.unwrap_or(self.default_ttl);
        let cached = CachedResult {
            data: Arc::new(result),
            created_at: Instant::now(),
            expires_at: Instant::now() + ttl,
            access_count: AtomicU64::new(0),
        };
        
        self.cache.insert(fingerprint, cached);
        
        // 检查是否需要清理
        if self.cache.len() > self.max_entries {
            self.evict_entries();
        }
    }
    
    /// 使缓存失效（数据变更时调用）
    pub fn invalidate(&self, space_id: i64, affected_tags: Option<Vec<String>>) {
        // 移除相关空间的缓存
        self.cache.retain(|key, _| {
            key.space_id != Some(space_id)
        });
    }
}
```

**缓存策略**:

```rust
/// 缓存策略配置
pub struct CachePolicy {
    /// 是否启用结果缓存
    pub enabled: bool,
    /// 默认 TTL
    pub default_ttl: Duration,
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 最大缓存大小（字节）
    pub max_size: usize,
    /// 缓存键模式（哪些查询可以缓存）
    pub key_patterns: Vec<CacheKeyPattern>,
}

pub enum CacheKeyPattern {
    /// 精确匹配
    Exact(String),
    /// 前缀匹配
    Prefix(String),
    /// 正则匹配
    Regex(String),
    /// 表名匹配（缓存涉及特定表的查询）
    Table(String),
}
```

**实现复杂度**: 低
**预期收益**: 高（热点查询场景）

### 2. 图数据缓存（Graph Data Cache）

**需求场景**:
- 热点顶点/边的频繁访问
- 邻居查询的重复访问
- 属性访问的局部性

**建议实现**:

```rust
use lru::LruCache;
use parking_lot::Mutex;
use std::sync::Arc;

/// 图数据缓存
pub struct GraphDataCache {
    /// 顶点缓存
    vertex_cache: Mutex<LruCache<Value, Arc<CachedVertex>>>,
    /// 边缓存
    edge_cache: Mutex<LruCache<EdgeKey, Arc<CachedEdge>>>,
    /// 邻居缓存（顶点 ID -> 邻居列表）
    neighbor_cache: Mutex<LruCache<NeighborCacheKey, Arc<Vec<Value>>>>,
    /// 统计信息
    stats: CacheStats,
}

/// 缓存的顶点
pub struct CachedVertex {
    pub vertex: Vertex,
    pub cached_at: Instant,
    pub version: u64,  // 用于一致性检查
}

/// 边缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct EdgeKey {
    pub src_id: Value,
    pub dst_id: Value,
    pub edge_type: String,
}

/// 邻居缓存键
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct NeighborCacheKey {
    pub vertex_id: Value,
    pub edge_type: Option<String>,
    pub direction: EdgeDirection,
}

impl GraphDataCache {
    /// 获取顶点
    pub fn get_vertex(&self, id: &Value) -> Option<Arc<CachedVertex>> {
        let mut cache = self.vertex_cache.lock();
        cache.get(id).cloned()
    }
    
    /// 缓存顶点
    pub fn put_vertex(&self, id: Value, vertex: Vertex) {
        let mut cache = self.vertex_cache.lock();
        let cached = CachedVertex {
            vertex,
            cached_at: Instant::now(),
            version: 0,
        };
        cache.put(id, Arc::new(cached));
    }
    
    /// 获取邻居
    pub fn get_neighbors(&self, key: &NeighborCacheKey) -> Option<Arc<Vec<Value>>> {
        let mut cache = self.neighbor_cache.lock();
        cache.get(key).cloned()
    }
    
    /// 使顶点失效（更新时调用）
    pub fn invalidate_vertex(&self, id: &Value) {
        let mut cache = self.vertex_cache.lock();
        cache.pop(id);
        
        // 同时使相关邻居缓存失效
        let mut neighbor_cache = self.neighbor_cache.lock();
        neighbor_cache.retain(|key, _| &key.vertex_id != id);
    }
}
```

**一致性策略**:

```rust
/// 缓存一致性策略
pub enum ConsistencyStrategy {
    /// 写穿（写入时同时更新缓存）
    WriteThrough,
    /// 写回（写入时使缓存失效）
    WriteBack,
    /// 时间到期
    TTL(Duration),
    /// 版本号检查
    VersionCheck,
}
```

**实现复杂度**: 中
**预期收益**: 中（减少存储 IO）

### 3. 缓存预热（Cache Warmup）

**需求场景**:
- 系统启动时预加载热点数据
- 定时刷新缓存
- 避免冷启动性能问题

**建议实现**:

```rust
/// 缓存预热管理器
pub struct CacheWarmupManager {
    graph_cache: Arc<GraphDataCache>,
    result_cache: Arc<ResultCache>,
    warmup_config: WarmupConfig,
}

pub struct WarmupConfig {
    /// 预热顶点 ID 列表
    pub hot_vertices: Vec<Value>,
    /// 预热查询列表
    pub hot_queries: Vec<String>,
    /// 预热时间窗口
    pub warmup_window: Duration,
    /// 并发预热数量
    pub concurrency: usize,
}

impl CacheWarmupManager {
    /// 异步预热
    pub async fn warmup(&self, storage: &dyn StorageClient) -> Result<WarmupStats, Error> {
        let start = Instant::now();
        let mut stats = WarmupStats::default();
        
        // 预热顶点
        let vertex_futures: Vec<_> = self.warmup_config.hot_vertices
            .iter()
            .map(|id| self.load_vertex(storage, id.clone()))
            .collect();
        
        let results = futures::future::join_all(vertex_futures).await;
        stats.vertices_loaded = results.iter().filter(|r| r.is_ok()).count();
        
        // 预热查询
        let query_futures: Vec<_> = self.warmup_config.hot_queries
            .iter()
            .map(|query| self.execute_warmup_query(storage, query.clone()))
            .collect();
        
        let results = futures::future::join_all(query_futures).await;
        stats.queries_cached = results.iter().filter(|r| r.is_ok()).count();
        
        stats.duration = start.elapsed();
        Ok(stats)
    }
    
    /// 定时刷新
    pub async fn schedule_refresh(&self, interval: Duration) {
        let mut interval = tokio::time::interval(interval);
        loop {
            interval.tick().await;
            self.refresh_hot_data().await;
        }
    }
}
```

**实现复杂度**: 中
**预期收益**: 中（避免冷启动）

### 4. 多级缓存架构

**架构设计**:

```
┌─────────────────────────────────────────────────────────────┐
│                      查询请求                                │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  L1: 结果缓存 (Result Cache)                                │
│  - 内存中的查询结果                                          │
│  - TTL: 60s                                                 │
│  - 适用: 重复查询、配置表                                     │
└─────────────────────────────────────────────────────────────┘
                            │ 未命中
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  L2: 图数据缓存 (Graph Data Cache)                          │
│  - 顶点、边、邻居缓存                                        │
│  - LRU 策略                                                  │
│  - 适用: 热点数据访问                                        │
└─────────────────────────────────────────────────────────────┘
                            │ 未命中
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  L3: 计划缓存 (Plan Cache)                                  │
│  - 查询执行计划                                              │
│  - 适用: 避免重复规划                                        │
└─────────────────────────────────────────────────────────────┘
                            │ 未命中
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  存储层 (Redb)                                              │
└─────────────────────────────────────────────────────────────┘
```

**实现复杂度**: 中
**预期收益**: 高（综合性能提升）

## 总结

### 已实现的优势

1. **完善的计划缓存** - LRU 策略，支持参数化查询
2. **表达式级缓存** - 正则、日期时间解析缓存
3. **对象池机制** - 减少内存分配开销
4. **Schema 内存缓存** - 减少元数据查询

### 建议优先级

| 优先级 | 优化项 | 预期收益 | 实现复杂度 |
|-------|--------|---------|-----------|
| P0 | 结果缓存 | 高 | 低 |
| P1 | 图数据缓存 | 中 | 中 |
| P2 | 缓存预热 | 中 | 中 |
| P3 | 多级缓存架构 | 高 | 中 |

**建议**: 优先实现结果缓存，投入产出比最高，可显著提升热点查询性能。
