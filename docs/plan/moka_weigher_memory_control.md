# Moka Weigher 内存控制扩展方案

## 1. 背景和问题

### 1.1 当前状态

当前缓存系统使用 moka 的 `max_capacity` 限制条目数量，但存在以下问题：

- **条目数量限制 ≠ 内存限制**：`max_capacity` 限制的是条目数量，而不是实际内存使用量
- **内存估算不准确**：`estimated_memory_bytes()` 只是基于条目数量和平均大小的估算
- **无法精确控制**：单个大条目可能占用大量内存，但只计为一个条目
- **已移除的内存压力检查**：之前的手动内存压力检查已被移除，因为与 moka 的 LRU 机制不匹配

### 1.2 目标

使用 moka 的 `weigher` 功能实现基于实际内存大小的精确控制：
- 按内存字节数限制缓存大小
- 自动淘汰最不重要的条目（LRU + 权重）
- 简化内存管理逻辑
- 提高内存利用率

## 2. Moka Weigher 功能介绍

### 2.1 基本概念

`weigher` 是 moka 提供的一个功能，用于为每个缓存条目分配权重（weight）：

```rust
Cache::builder()
    .weigher(|_key, value: &Arc<CachedPlan>| -> u32 {
        // 计算并返回该条目的权重（通常是内存大小）
        value.estimate_memory() as u32
    })
    .max_weight(max_memory_bytes as u64)  // 设置最大总权重
    .build()
```

### 2.2 工作原理

1. **权重计算**：每次插入或更新条目时，调用 `weigher` 计算权重
2. **总权重跟踪**：moka 维护当前所有条目的总权重
3. **自动淘汰**：当总权重超过 `max_weight` 时，自动淘汰条目
4. **淘汰策略**：基于 LRU（最久未使用）+ 权重考虑

### 2.3 优势

- **精确控制**：按实际内存使用量限制，而不是条目数量
- **自动管理**：无需手动检查内存压力，moka 自动处理
- **高效淘汰**：基于 LRU 和权重的智能淘汰
- **线程安全**：moka 内部处理并发访问

## 3. 当前缓存结构分析

### 3.1 QueryPlanCache 条目结构

```rust
pub struct CachedPlan {
    pub query_template: String,           // 查询模板字符串
    pub plan: ExecutionPlan,              // 执行计划（复杂结构）
    pub param_positions: Vec<ParamPosition>, // 参数位置信息
    pub created_at: Instant,              // 创建时间
    pub last_accessed: Instant,           // 最后访问时间
    pub access_count: u64,                // 访问次数
    pub avg_execution_time_ms: f64,        // 平均执行时间
    pub execution_count: u64,               // 执行次数
    pub priority: CachePriority,            // 缓存优先级
    pub complexity_score: u32,              // 复杂度分数
    pub estimated_compute_cost: u64,         // 估算计算成本
    pub current_ttl: Duration,              // 当前 TTL
}
```

### 3.2 CteCacheEntry 条目结构

```rust
pub struct CteCacheEntry {
    pub data: Arc<Vec<u8>>,              // 结果数据（字节数组）
    pub row_count: u64,                   // 行数
    pub data_size: usize,                  // 数据大小（已知）
    pub created_at: Instant,               // 创建时间
    pub last_accessed: Instant,            // 最后访问时间
    pub access_count: u64,                 // 访问次数
    pub reuse_probability: f64,             // 重用概率
    pub cte_hash: String,                 // CTE 哈希
    pub cte_definition: String,           // CTE 定义
    pub priority: CachePriority,            // 缓存优先级
    pub compute_cost_ms: u64,             // 计算成本
    pub access_frequency: f64,             // 访问频率
    pub dependent_tables: Vec<String>,      // 依赖表
}
```

## 4. 内存估算策略

### 4.1 QueryPlanCache 内存估算

需要估算 `CachedPlan` 的内存占用：

```rust
impl CachedPlan {
    /// 估算内存占用（字节）
    pub fn estimate_memory(&self) -> usize {
        let mut total = 0;

        // 查询模板字符串
        total += self.query_template.len();

        // 参数位置信息
        total += self.param_positions.capacity() * std::mem::size_of::<ParamPosition>();
        for pos in &self.param_positions {
            total += std::mem::size_of::<ParamPosition>();
            if let Some(ref name) = pos.name {
                total += name.capacity();
            }
        }

        // 执行计划（递归估算）
        total += self.estimate_plan_memory(&self.plan);

        // 其他字段
        total += std::mem::size_of::<Instant>() * 2;  // created_at, last_accessed
        total += std::mem::size_of::<u64>() * 2;      // access_count, execution_count
        total += std::mem::size_of::<f64>() * 2;      // avg_execution_time_ms
        total += std::mem::size_of::<CachePriority>();
        total += std::mem::size_of::<u32>();          // complexity_score
        total += std::mem::size_of::<u64>();          // estimated_compute_cost
        total += std::mem::size_of::<Duration>();       // current_ttl

        total
    }

    /// 估算执行计划的内存
    fn estimate_plan_memory(&self, plan: &ExecutionPlan) -> usize {
        // 根据执行计划的类型和结构估算
        // 这里需要根据实际的 ExecutionPlan 结构实现
        // 示例：
        let base_size = std::mem::size_of::<ExecutionPlan>();
        let overhead = 1024; // 估算的额外开销
        base_size + overhead
    }
}
```

**估算公式**：
```
总内存 = 查询模板 + 参数位置 + 执行计划 + 元数据
```

### 4.2 CteCacheEntry 内存估算

`CteCacheEntry` 已经有 `data_size` 字段，可以直接使用：

```rust
impl CteCacheEntry {
    /// 估算内存占用（字节）
    pub fn estimate_memory(&self) -> usize {
        let mut total = 0;

        // 数据大小（已知）
        total += self.data_size;

        // 字符串字段
        total += self.cte_hash.capacity();
        total += self.cte_definition.capacity();

        // 向量字段
        total += self.dependent_tables.capacity() * std::mem::size_of::<String>();
        for table in &self.dependent_tables {
            total += table.capacity();
        }

        // 其他字段
        total += std::mem::size_of::<Instant>() * 2;  // created_at, last_accessed
        total += std::mem::size_of::<u64>() * 2;      // row_count, access_count, compute_cost_ms
        total += std::mem::size_of::<f64>() * 2;      // reuse_probability, access_frequency
        total += std::mem::size_of::<CachePriority>();

        total
    }
}
```

**估算公式**：
```
总内存 = 数据大小 + 字符串 + 向量 + 元数据
```

### 4.3 估算精度

- **精确部分**：字符串长度、向量容量、基本类型大小
- **估算部分**：执行计划的复杂结构、Arc 共享数据
- **保守策略**：略微高估，避免内存超限

## 5. Weigher 实现方案

### 5.1 QueryPlanCache 实现

```rust
impl QueryPlanCache {
    pub fn new(config: PlanCacheConfig) -> Self {
        let max_weight = config.max_weight.unwrap_or(config.memory_budget as u64);

        let cache = Cache::builder()
            .max_capacity(config.max_entries as u64)
            .weigher(|_key, value: &Arc<CachedPlan>| -> u32 {
                // 计算缓存条目的权重（内存大小）
                value.estimate_memory() as u32
            })
            .max_weight(max_weight)
            .time_to_live(Duration::from_secs(config.ttl_config.base_ttl_seconds))
            .build();

        Self {
            cache,
            config,
            stats: Arc::new(RwLock::new(PlanCacheStats::new())),
        }
    }
}
```

### 5.2 CteCacheManager 实现

由于 `CteCacheManager` 使用 `DashMap` 而不是 moka，需要考虑两种方案：

#### 方案 A：迁移到 moka（推荐）

```rust
pub struct CteCacheManager {
    cache: Cache<String, Arc<CteCacheEntry>>,
    config: CteCacheConfig,
    stats: Arc<RwLock<CteCacheStats>>,
}

impl CteCacheManager {
    pub fn with_config(config: CteCacheConfig) -> Self {
        let max_weight = config.max_size as u64;

        let cache = Cache::builder()
            .weigher(|_key, value: &Arc<CteCacheEntry>| -> u32 {
                value.estimate_memory() as u32
            })
            .max_weight(max_weight)
            .time_to_live(Duration::from_secs(config.entry_ttl_seconds))
            .build();

        Self {
            cache,
            config,
            stats: Arc::new(RwLock::new(CteCacheStats::new())),
        }
    }
}
```

#### 方案 B：保持 DashMap，手动管理

如果必须使用 DashMap，可以手动实现 weigher 逻辑：

```rust
impl CteCacheManager {
    pub fn insert(&self, key: String, entry: Arc<CteCacheEntry>) -> DBResult<()> {
        let weight = entry.estimate_memory();

        // 检查是否超过最大权重
        if self.current_weight() + weight > self.config.max_size {
            self.evict_by_weight(weight);
        }

        self.cache.insert(key, entry);
        self.update_stats(weight);
        Ok(())
    }

    fn evict_by_weight(&self, target_weight: usize) {
        // 按访问时间排序，淘汰最久未使用的条目
        let mut entries: Vec<_> = self.cache.iter().collect();
        entries.sort_by_key(|(_, v)| v.last_accessed);

        let mut freed = 0;
        for (key, _) in entries {
            if freed >= target_weight {
                break;
            }
            if let Some((_, entry)) = self.cache.remove(&key) {
                freed += entry.estimate_memory();
            }
        }
    }
}
```

**推荐方案 A**：统一使用 moka，简化代码，利用其成熟的 weigher 功能。

## 6. 配置接口设计

### 6.1 PlanCacheConfig 扩展

```rust
pub struct PlanCacheConfig {
    /// 最大条目数量
    pub max_entries: usize,

    /// 内存预算（字节）
    pub memory_budget: usize,

    /// 最大权重（字节），优先于 max_entries
    /// 如果为 None，使用 memory_budget
    pub max_weight: Option<u64>,

    /// 是否启用参数化查询
    pub enable_parameterized: bool,

    /// TTL 配置
    pub ttl_config: TtlConfig,

    /// 优先级配置
    pub priority_config: PriorityConfig,
}

impl Default for PlanCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            memory_budget: 50 * 1024 * 1024,  // 50 MB
            max_weight: None,  // 使用 memory_budget
            enable_parameterized: true,
            ttl_config: TtlConfig::default(),
            priority_config: PriorityConfig::default(),
        }
    }
}
```

### 6.2 CteCacheConfig 扩展

```rust
pub struct CteCacheConfig {
    /// 最大大小（字节）- 作为 max_weight
    pub max_size: usize,

    /// 最大条目数量（可选）
    pub max_entries: Option<usize>,

    /// 单个条目最大大小（字节）
    pub max_entry_size: usize,

    /// 最小行数
    pub min_row_count: u64,

    /// 最大行数
    pub max_row_count: u64,

    /// 条目 TTL（秒）
    pub entry_ttl_seconds: u64,

    /// 是否启用
    pub enabled: bool,

    /// 是否自适应
    pub adaptive: bool,

    /// 是否启用优先级
    pub enable_priority: bool,
}

impl Default for CteCacheConfig {
    fn default() -> Self {
        Self {
            max_size: 40 * 1024 * 1024,  // 40 MB
            max_entries: Some(10000),
            max_entry_size: 10 * 1024 * 1024,  // 10 MB
            min_row_count: 100,
            max_row_count: 100_000,
            entry_ttl_seconds: 3600,
            enabled: true,
            adaptive: true,
            enable_priority: true,
        }
    }
}
```

## 7. 实现步骤

### 7.1 第一阶段：内存估算实现

1. 为 `CachedPlan` 实现 `estimate_memory()` 方法
2. 为 `CteCacheEntry` 实现 `estimate_memory()` 方法
3. 编写单元测试验证估算准确性

### 7.2 第二阶段：配置扩展

1. 扩展 `PlanCacheConfig`，添加 `max_weight` 字段
2. 扩展 `CteCacheConfig`，添加 `max_entries` 字段
3. 更新默认配置值

### 7.3 第三阶段：Weigher 集成

1. 修改 `QueryPlanCache::new()`，添加 weigher
2. 修改 `CteCacheManager::with_config()`，添加 weigher（或迁移到 moka）
3. 更新统计信息，跟踪权重使用

### 7.4 第四阶段：测试和优化

1. 编写集成测试，验证内存限制
2. 性能测试，对比优化前后的效果
3. 调整估算策略，提高精度

### 7.5 第五阶段：清理和文档

1. 移除不再需要的 `estimated_memory_bytes()` 方法
2. 更新文档和注释
3. 添加使用示例

## 8. 优势和注意事项

### 8.1 优势

1. **精确内存控制**
   - 按实际内存使用量限制，避免内存溢出
   - 自动淘汰，无需手动干预

2. **简化代码**
   - 移除复杂的内存压力检查逻辑
   - 统一使用 moka 的管理机制

3. **提高效率**
   - LRU + 权重的智能淘汰
   - 减少不必要的内存占用

4. **更好的可观测性**
   - 可以跟踪实际内存使用量
   - 更准确的统计信息

### 8.2 注意事项

1. **估算精度**
   - 内存估算需要准确，否则可能导致限制失效
   - 建议定期验证和调整估算公式

2. **性能影响**
   - weigher 在每次插入时调用，需要保持高效
   - 避免复杂的递归计算

3. **迁移成本**
   - 如果 CteCacheManager 从 DashMap 迁移到 moka，需要测试兼容性
   - 可能需要调整并发访问模式

4. **配置调整**
   - 需要根据实际使用情况调整 `max_weight`
   - 建议提供配置工具和建议值

### 8.3 监控和调优

1. **监控指标**
   - 当前总权重
   - 淘汰频率
   - 缓存命中率

2. **调优策略**
   - 根据监控数据调整 `max_weight`
   - 优化内存估算公式
   - 调整 TTL 和其他参数

## 9. 示例代码

### 9.1 使用 weigher 的 QueryPlanCache

```rust
let config = PlanCacheConfig {
    max_entries: 1000,
    memory_budget: 50 * 1024 * 1024,  // 50 MB
    max_weight: Some(50 * 1024 * 1024),  // 使用 weigher
    enable_parameterized: true,
    ttl_config: TtlConfig::default(),
    priority_config: PriorityConfig::default(),
};

let cache = QueryPlanCache::new(config);

// 插入时会自动计算权重
cache.put("SELECT * FROM users WHERE id = $1", plan, params);

// 当总权重超过 50 MB 时，自动淘汰条目
```

### 9.2 监控内存使用

```rust
let stats = cache.stats();
println!("Estimated memory: {} bytes", stats.estimated_memory_bytes());
println!("Current entries: {}", stats.current_entries.load(Ordering::Relaxed));
```

## 10. 总结

使用 moka 的 weigher 功能可以：

1. **实现精确的内存控制**：按实际内存使用量限制缓存
2. **简化代码逻辑**：移除手动内存管理，依赖成熟的 moka 机制
3. **提高性能**：基于 LRU 和权重的智能淘汰
4. **增强可维护性**：统一的缓存管理策略

建议按照实现步骤逐步推进，先实现内存估算，再集成 weigher，最后进行测试和优化。
