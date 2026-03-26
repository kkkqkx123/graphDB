# 对象池优化方案

**文档版本**: 1.0  
**创建日期**: 2026-03-26  
**相关模块**: query/executor

---

## 一、现状分析

### 1.1 当前实现

当前对象池实现位于 [src/query/executor/object_pool.rs](file:///d:/项目/database/graphDB/src/query/executor/object_pool.rs)：

```rust
pub struct ObjectPoolConfig {
    pub max_pool_size: usize,  // 默认 10
    pub enabled: bool,         // 默认 true
}

pub struct ExecutorObjectPool<S: StorageClient + 'static> {
    config: ObjectPoolConfig,
    pools: HashMap<String, Vec<ExecutorEnum<S>>>,
    stats: PoolStats,
}
```

### 1.2 存在的问题

| 问题 | 说明 | 影响 |
|------|------|------|
| 池大小固定 | 所有执行器类型使用相同的 max_pool_size | 高频执行器可能不足，低频执行器浪费 |
| 无内存限制 | 只限制单个类型数量，无总内存预算 | 可能导致内存无限增长 |
| 无预热机制 | 冷启动时对象池为空 | 首次查询性能不佳 |
| 无优先级 | 所有执行器同等对待 | 重要执行器可能被淘汰 |
| 无动态调整 | 配置固定，不随负载变化 | 无法适应不同工作负载 |

---

## 二、优化目标

1. **分级对象池**: 按执行器类型设置不同池大小和优先级
2. **内存预算**: 添加总内存限制，防止内存无限增长
3. **预热机制**: 启动时预创建常用执行器
4. **自适应调整**: 根据负载动态调整池大小
5. **预期收益**: 提高 20-40% 高并发场景性能，防止内存溢出

---

## 三、具体优化方案

### 3.1 分级对象池配置

#### 优化后的配置结构

```rust
// src/query/executor/object_pool.rs

/// 对象池全局配置
#[derive(Debug, Clone)]
pub struct ObjectPoolConfig {
    /// 是否启用对象池
    pub enabled: bool,
    /// 默认池大小
    pub default_pool_size: usize,
    /// 总内存预算（字节）
    pub memory_budget: usize,
    /// 是否启用预热
    pub enable_warmup: bool,
    /// 按类型配置
    pub type_configs: HashMap<String, TypePoolConfig>,
    /// 是否启用自适应调整
    pub enable_adaptive: bool,
    /// 自适应调整间隔（秒）
    pub adaptive_interval_secs: u64,
}

impl Default for ObjectPoolConfig {
    fn default() -> Self {
        let mut type_configs = HashMap::new();
        
        // 高频执行器 - 大池
        type_configs.insert(
            "FilterExecutor".to_string(),
            TypePoolConfig {
                max_size: 50,
                priority: PoolPriority::High,
                warmup_count: 10,
            },
        );
        type_configs.insert(
            "ProjectExecutor".to_string(),
            TypePoolConfig {
                max_size: 50,
                priority: PoolPriority::High,
                warmup_count: 10,
            },
        );
        
        // 中频执行器 - 中等池
        type_configs.insert(
            "ScanVerticesExecutor".to_string(),
            TypePoolConfig {
                max_size: 20,
                priority: PoolPriority::Medium,
                warmup_count: 5,
            },
        );
        type_configs.insert(
            "GetNeighborsExecutor".to_string(),
            TypePoolConfig {
                max_size: 20,
                priority: PoolPriority::Medium,
                warmup_count: 5,
            },
        );
        
        // 低频执行器 - 小池
        type_configs.insert(
            "AggregateExecutor".to_string(),
            TypePoolConfig {
                max_size: 5,
                priority: PoolPriority::Low,
                warmup_count: 2,
            },
        );
        
        Self {
            enabled: true,
            default_pool_size: 10,
            memory_budget: 64 * 1024 * 1024, // 64MB
            enable_warmup: true,
            type_configs,
            enable_adaptive: true,
            adaptive_interval_secs: 60,
        }
    }
}

/// 执行器类型池配置
#[derive(Debug, Clone)]
pub struct TypePoolConfig {
    /// 最大池大小
    pub max_size: usize,
    /// 优先级（用于内存不足时淘汰决策）
    pub priority: PoolPriority,
    /// 预热数量
    pub warmup_count: usize,
}

/// 池优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PoolPriority {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}
```

---

### 3.2 内存预算管理

```rust
/// 带内存预算的对象池
pub struct ExecutorObjectPool<S: StorageClient + 'static> {
    config: ObjectPoolConfig,
    pools: HashMap<String, Vec<ExecutorEnum<S>>>,
    /// 当前内存使用（估算）
    current_memory: usize,
    /// 执行器类型大小缓存
    type_sizes: HashMap<String, usize>,
    stats: PoolStats,
}

impl<S: StorageClient + 'static> ExecutorObjectPool<S> {
    /// 释放执行器回池，带内存检查
    pub fn release(&mut self, executor_type: &str, executor: ExecutorEnum<S>) {
        if !self.config.enabled {
            return;
        }

        self.stats.total_releases += 1;

        // 获取或估算执行器大小
        let size = self.estimate_size(&executor);
        
        // 检查内存预算
        if self.current_memory + size > self.config.memory_budget {
            // 尝试淘汰低优先级池中的执行器
            if !self.evict_for_memory(size) {
                // 无法腾出足够空间，丢弃当前执行器
                self.stats.memory_discarded += 1;
                return;
            }
        }

        let pool = self.pools.entry(executor_type.to_string()).or_default();
        let max_size = self.get_max_size(executor_type);

        if pool.len() < max_size {
            pool.push(executor);
            self.current_memory += size;
            self.type_sizes.insert(executor_type.to_string(), size);
        } else {
            self.stats.pool_full_discarded += 1;
        }
    }

    /// 为指定大小的执行器腾出内存
    fn evict_for_memory(&mut self, required_size: usize) -> bool {
        let mut freed = 0;
        
        // 按优先级排序，先淘汰低优先级
        let mut types: Vec<_> = self.pools.keys().cloned().collect();
        types.sort_by_key(|t| self.get_priority(t));
        
        for type_name in types {
            if freed >= required_size {
                break;
            }
            
            if let Some(pool) = self.pools.get_mut(&type_name) {
                while let Some(executor) = pool.pop() {
                    freed += self.type_sizes.get(&type_name).copied().unwrap_or(0);
                    self.stats.evicted += 1;
                    
                    if freed >= required_size {
                        break;
                    }
                }
            }
        }
        
        self.current_memory = self.current_memory.saturating_sub(freed);
        freed >= required_size
    }

    /// 估算执行器大小（简化版）
    fn estimate_size(&self, executor: &ExecutorEnum<S>) -> usize {
        // 基于执行器类型的估算
        match executor {
            ExecutorEnum::Filter(_) => 256,
            ExecutorEnum::Project(_) => 128,
            ExecutorEnum::ScanVertices(_) => 512,
            ExecutorEnum::GetNeighbors(_) => 384,
            ExecutorEnum::Aggregate(_) => 1024,
            _ => 256, // 默认值
        }
    }
}
```

---

### 3.3 预热机制

```rust
/// 对象池预热器
pub struct PoolWarmer<S: StorageClient + 'static> {
    pool: Arc<Mutex<ExecutorObjectPool<S>>>,
    factory: Arc<ExecutorFactory<S>>,
}

impl<S: StorageClient + 'static> PoolWarmer<S> {
    /// 执行预热
    pub fn warmup(&self) {
        let mut pool = self.pool.lock();
        let config = pool.config().clone();
        
        if !config.enable_warmup {
            return;
        }
        
        for (type_name, type_config) in &config.type_configs {
            if type_config.warmup_count == 0 {
                continue;
            }
            
            // 创建预热执行器
            for _ in 0..type_config.warmup_count {
                if let Some(executor) = self.factory.create_for_warmup(type_name) {
                    pool.release(type_name, executor);
                }
            }
        }
        
        log::info!("Object pool warmup completed: {} types warmed", config.type_configs.len());
    }
}

/// 在执行器工厂中添加预热创建方法
impl<S: StorageClient + 'static> ExecutorFactory<S> {
    /// 为预热创建执行器（简化配置）
    pub fn create_for_warmup(&self, executor_type: &str) -> Option<ExecutorEnum<S>> {
        // 创建最小配置的执行器，仅用于填充池
        match executor_type {
            "FilterExecutor" => Some(ExecutorEnum::Filter(FilterExecutor::default())),
            "ProjectExecutor" => Some(ExecutorEnum::Project(ProjectExecutor::default())),
            // ... 其他类型
            _ => None,
        }
    }
}
```

---

### 3.4 自适应调整

```rust
/// 自适应对象池管理器
pub struct AdaptivePoolManager<S: StorageClient + 'static> {
    pool: Arc<Mutex<ExecutorObjectPool<S>>>,
    stats_history: VecDeque<PoolStats>,
    last_adjustment: Instant,
}

impl<S: StorageClient + 'static> AdaptivePoolManager<S> {
    /// 根据负载调整池大小
    pub fn adjust_if_needed(&mut self) {
        let config = self.pool.lock().config().clone();
        
        if !config.enable_adaptive {
            return;
        }
        
        let elapsed = self.last_adjustment.elapsed();
        if elapsed.as_secs() < config.adaptive_interval_secs {
            return;
        }
        
        let current_stats = self.pool.lock().stats().clone();
        self.stats_history.push_back(current_stats.clone());
        
        // 只保留最近 10 个统计点
        if self.stats_history.len() > 10 {
            self.stats_history.pop_front();
        }
        
        // 分析命中率趋势
        let hit_rate = current_stats.hit_rate();
        
        if hit_rate < 0.5 {
            // 命中率低，增加池大小
            self.increase_pool_sizes();
        } else if hit_rate > 0.95 && self.is_memory_pressure() {
            // 命中率高但内存压力大，减少池大小
            self.decrease_pool_sizes();
        }
        
        self.last_adjustment = Instant::now();
    }

    /// 增加池大小
    fn increase_pool_sizes(&self) {
        let mut pool = self.pool.lock();
        // 增加 20%，但不超过上限
        // 具体实现...
    }

    /// 减少池大小
    fn decrease_pool_sizes(&self) {
        let mut pool = self.pool.lock();
        // 减少 10%
        // 具体实现...
    }

    /// 检查内存压力
    fn is_memory_pressure(&self) -> bool {
        // 检查系统内存使用情况
        // 具体实现...
        false
    }
}
```

---

## 四、配置预设

### 4.1 预设配置

```rust
impl ObjectPoolConfig {
    /// 最小内存配置 - 适用于嵌入式/低内存环境
    pub fn minimal() -> Self {
        let mut config = Self::default();
        config.memory_budget = 16 * 1024 * 1024; // 16MB
        config.default_pool_size = 5;
        config.enable_warmup = false;
        config.enable_adaptive = false;
        
        // 减少所有类型的池大小
        for (_, type_config) in &mut config.type_configs {
            type_config.max_size = type_config.max_size / 2;
            type_config.warmup_count = 0;
        }
        
        config
    }

    /// 高并发配置 - 适用于高性能服务器
    pub fn high_concurrency() -> Self {
        let mut config = Self::default();
        config.memory_budget = 256 * 1024 * 1024; // 256MB
        config.default_pool_size = 20;
        config.enable_adaptive = true;
        config.adaptive_interval_secs = 30;
        
        // 增加高频执行器的池大小
        if let Some(cfg) = config.type_configs.get_mut("FilterExecutor") {
            cfg.max_size = 100;
            cfg.warmup_count = 20;
        }
        if let Some(cfg) = config.type_configs.get_mut("ProjectExecutor") {
            cfg.max_size = 100;
            cfg.warmup_count = 20;
        }
        
        config
    }

    /// 禁用对象池
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}
```

---

## 五、实施步骤

### 阶段一：扩展配置结构（低风险）

1. 修改 `ObjectPoolConfig`，添加新字段
2. 添加 `TypePoolConfig` 和 `PoolPriority`
3. 添加预设配置方法

### 阶段二：实现内存预算（中风险）

1. 添加 `current_memory` 和 `type_sizes` 字段
2. 修改 `release` 方法，添加内存检查
3. 实现 `evict_for_memory` 方法

### 阶段三：实现预热机制（低风险）

1. 创建 `PoolWarmer` 结构体
2. 在 `ExecutorFactory` 中添加 `create_for_warmup` 方法
3. 在系统启动时调用预热

### 阶段四：实现自适应调整（高风险）

1. 创建 `AdaptivePoolManager` 结构体
2. 实现统计历史记录
3. 实现自动调整逻辑
4. 添加后台任务定期执行调整

---

## 六、集成建议

### 6.1 与查询执行流程集成

```rust
// 在 QueryExecutionManager 中集成
pub struct QueryExecutionManager<S: StorageClient> {
    object_pool: Arc<Mutex<ExecutorObjectPool<S>>>,
    pool_warmer: Option<PoolWarmer<S>>,
    adaptive_manager: Option<AdaptivePoolManager<S>>,
}

impl<S: StorageClient> QueryExecutionManager<S> {
    pub fn new(config: ExecutionConfig) -> Self {
        let pool = Arc::new(Mutex::new(ExecutorObjectPool::new(config.object_pool)));
        
        // 预热
        let warmer = if config.object_pool.enable_warmup {
            let w = PoolWarmer::new(pool.clone(), config.factory.clone());
            w.warmup();
            Some(w)
        } else {
            None
        };
        
        // 自适应管理
        let adaptive = if config.object_pool.enable_adaptive {
            Some(AdaptivePoolManager::new(pool.clone()))
        } else {
            None
        };
        
        Self {
            object_pool: pool,
            pool_warmer: warmer,
            adaptive_manager: adaptive,
        }
    }
}
```

---

## 七、监控指标

```rust
/// 扩展的池统计信息
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    // 原有字段...
    pub total_acquires: usize,
    pub total_releases: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    
    // 新增字段
    /// 因内存不足丢弃的数量
    pub memory_discarded: usize,
    /// 因池满丢弃的数量
    pub pool_full_discarded: usize,
    /// 被淘汰的数量
    pub evicted: usize,
    /// 当前内存使用（字节）
    pub current_memory: usize,
    /// 内存预算
    pub memory_budget: usize,
    /// 按类型统计
    pub type_stats: HashMap<String, TypePoolStats>,
}

#[derive(Debug, Clone, Default)]
pub struct TypePoolStats {
    pub current_size: usize,
    pub max_size: usize,
    pub hits: usize,
    pub misses: usize,
    pub estimated_memory: usize,
}
```

---

## 八、预期收益

| 指标 | 优化前 | 优化后 | 改善 |
|------|--------|--------|------|
| 高并发命中率 | ~60% | ~85% | +25% |
| 内存使用上限 | 无限制 | 可配置 | 可控 |
| 冷启动延迟 | 高 | 低 | -50% |
| 自适应能力 | 无 | 有 | 新功能 |
