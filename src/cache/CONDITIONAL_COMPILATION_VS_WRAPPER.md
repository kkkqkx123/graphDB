# 缓存统计功能：条件编译 vs Wrapper 层设计

## 背景

**决策前提**：统一使用带统计的缓存，移除非统计版本的 `CacheType` 和 `StatsCacheType` 枚举二重性。

**未来问题**：后续可能需要不带统计的缓存版本。实现策略是什么？

## 对比分析

### 选项 1：条件编译方案（`features`）

在 `Cargo.toml` 中添加特性开关：

```toml
[features]
cache-stats = []
```

```rust
// stats_wrapper.rs
#[cfg(feature = "cache-stats")]
pub struct StatsCacheWrapper<K, V, C> {
    inner: Arc<C>,
    stats: Arc<RwLock<CacheStats>>,
}

#[cfg(not(feature = "cache-stats"))]
pub struct StatsCacheWrapper<K, V, C> {
    inner: Arc<C>,
    // 无统计字段
}

// 两套 impl
#[cfg(feature = "cache-stats")]
impl<K, V, C> Cache<K, V> for StatsCacheWrapper<K, V, C>
where
    C: Cache<K, V>,
{
    fn get(&self, key: &K) -> Option<V> {
        let result = self.inner.get(key);
        // 更新统计
        result
    }
}

#[cfg(not(feature = "cache-stats"))]
impl<K, V, C> Cache<K, V> for StatsCacheWrapper<K, V, C>
where
    C: Cache<K, V>,
{
    fn get(&self, key: &K) -> Option<V> {
        // 直接委托，无统计开销
        self.inner.get(key)
    }
}
```

**优势**：
- ✓ 编译时消除死代码，零运行时开销
- ✓ 单一代码路径，维护简单
- ✓ 对 API 使用者透明

**劣势**：
- ✗ **全局决策** - 整个程序要么启用要么禁用统计
- ✗ **无法混用** - 同一程序不能同时有统计和无统计的缓存
- ✗ **测试困难** - 需要两个构建配置来测试两个代码路径
- ✗ **部署灵活性差** - 无法在运行时调整统计开销

### 选项 2：Wrapper 层统一设计（推荐✓）

保持 `StatsCacheWrapper` 不变，但**在内部实现零开销**：

#### 2.1 让缓存实现自己管理统计

```rust
// 每个缓存实现都带有可选的统计
pub trait Cache<K, V> {
    fn get(&self, key: &K) -> Option<V>;
    
    // 可选的统计方法
    fn stats(&self) -> Option<CacheStats> {
        None
    }
    
    fn reset_stats(&self) {}
}

// 缓存实现选择是否启用统计
pub struct ConcurrentLruCacheWithStats<K, V> {
    cache: Arc<Mutex<LruCache<K, V>>>,
    stats: Arc<RwLock<CacheStats>>,
}

pub struct ConcurrentLroCacheNoStats<K, V> {
    cache: Arc<Mutex<LruCache<K, V>>>,
}

// 两个不同的类型，但都实现 Cache trait
```

**优势**：
- ✓ 灵活 - 每个缓存实例独立选择
- ✓ 零开销 - 无统计版本完全没有开销
- ✓ 可混用 - 同一程序可以同时使用两种

**劣势**：
- ✗ 代码复用差 - 需要为每个缓存类型复制实现
- ✗ 维护困难 - 8个缓存类型 × 2 = 16个独立实现

#### 2.2 使用泛型参数（最优✓）

```rust
// 编译时标记
pub trait StatsMode: Send + Sync {
    const ENABLED: bool;
}

pub struct WithStats;
impl StatsMode for WithStats {
    const ENABLED: bool = true;
}

pub struct NoStats;
impl StatsMode for NoStats {
    const ENABLED: bool = false;
}

// 统一的缓存实现
pub struct ConcurrentLruCache<K, V, S: StatsMode = NoStats> {
    cache: Arc<Mutex<LruCache<K, V>>>,
    #[cfg_attr(feature = "cache-stats", allow(dead_code))]
    stats: Option<Arc<RwLock<CacheStats>>>,
    _marker: std::marker::PhantomData<S>,
}

impl<K, V> Cache<K, V> for ConcurrentLruCache<K, V, NoStats>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    // 编译器优化掉 Option 分支
    fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.lock().expect("...");
        let value = cache.cache.get(key).cloned();
        if value.is_some() {
            cache.move_to_back(key);
        }
        value
    }
}

impl<K, V> Cache<K, V> for ConcurrentLruCache<K, V, WithStats>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    // 带统计的实现
    fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.lock().expect("...");
        let value = cache.cache.get(key).cloned();
        if value.is_some() {
            cache.move_to_back(key);
            // 更新统计
            if let Some(ref stats) = self.stats {
                let mut s = stats.write().unwrap();
                s.total_hits += 1;
            }
        } else if let Some(ref stats) = self.stats {
            let mut s = stats.write().unwrap();
            s.total_misses += 1;
        }
        value
    }
}

// 类型别名
pub type LruCacheWithStats<K, V> = ConcurrentLruCache<K, V, WithStats>;
pub type LruCacheNoStats<K, V> = ConcurrentLruCache<K, V, NoStats>;
```

**优势**：
- ✓ 单一实现 - 一份代码处理两种情况
- ✓ **灵活 - 可混用，编译时决定**
- ✓ 零开销 - 编译器特化并优化每个版本
- ✓ 类型安全 - 编译时多态

**劣势**：
- ✗ 代码复杂度稍高 - 需要理解泛型特化
- ✗ 编译时间增加 - 单态化

## 性能对比

### 条件编译方案（Features）

| 场景 | 编译时间 | 运行时性能 | 灵活性 |
|------|--------|---------|-------|
| 启用统计 | 基准 | 基准（含统计开销） | 全局固定 |
| 禁用统计 | 基准 | +0% | 全局固定 |

### Wrapper 层重新实现

| 场景 | 编译时间 | 运行时性能 | 灵活性 |
|------|--------|---------|-------|
| 同时使用两种 | 基准×2 | 各自最优 | **最高** |

### 泛型参数方案

| 场景 | 编译时间 | 运行时性能 | 灵活性 |
|------|--------|---------|-------|
| 同时使用两种 | +20-30% | 各自最优 | **最高** |
| 仅使用一种 | 基准+5% | 最优 | 灵活 |

## 决策矩阵

| 决策因素 | 条件编译 | Wrapper 重实现 | 泛型参数 |
|--------|--------|-----------|--------|
| **代码维护** | ⭐⭐⭐ | ⭐ | ⭐⭐⭐ |
| **运行时性能** | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **灵活性（混用）** | ⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **部署简便性** | ⭐⭐⭐ | ⭐ | ⭐⭐ |
| **编译速度** | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| **易理解度** | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |

## 推荐方案

### 对于您的场景：条件编译（短期）+ 泛型参数（长期）

#### 短期（立即可实施）

采用**条件编译**方案，但**范围限制在 `stats_wrapper.rs`**：

```rust
// stats_wrapper.rs 中
#[cfg(feature = "cache-stats")]
pub struct StatsCacheWrapper<K, V, C> {
    inner: Arc<C>,
    stats: Arc<RwLock<CacheStats>>,
}

#[cfg(not(feature = "cache-stats"))]
pub struct StatsCacheWrapper<K, V, C> {
    inner: Arc<C>,
}
```

**理由**：
1. 现在已经**统一了 `CacheType`**，只有一套枚举
2. 条件编译只发生在 wrapper 层，不影响缓存实现本身
3. `ParserCache` 的二重性问题得到解决
4. 后续如需无统计版本，仅改 feature 开关即可

**优点**：
- 实施简单
- 使用者无感知
- 编译时完全消除统计代码
- 没有运行时条件判断

**缺点**：
- 同一程序不能同时使用两个版本
- 需要多个构建配置测试

#### 长期（v2.0 或后续版本）

如果需要在同一程序中支持有/无统计的混合模式：

```rust
// 升级到泛型参数
pub struct ConcurrentLruCache<K, V, S: StatsMode = NoStats> { ... }

// 同时支持两种模式
let cache_with_stats: ConcurrentLruCache<String, String, WithStats> = ...;
let cache_no_stats: ConcurrentLruCache<String, String, NoStats> = ...;
```

## 实施建议

### 立即行动

1. **在 `Cargo.toml` 添加**：
```toml
[features]
default = ["cache-stats"]
cache-stats = []
```

2. **在 `stats_wrapper.rs` 应用条件编译**

3. **在 `docs/archive/dynamic.md` 记录**：
```markdown
## 文件：src/cache/stats_wrapper.rs

### 已解决的动态分发问题

通过条件编译特性 `cache-stats` 在编译时决定是否启用统计功能，
完全消除了运行时的条件分支。

- **启用**（`--features cache-stats`）：完整统计功能
- **禁用**（`--no-default-features`）：零统计开销，仅保留基础缓存

### 测试覆盖

```bash
cargo test --features cache-stats
cargo test --no-default-features
```
```

4. **更新 `DYNAMIC_DISPATCH_ANALYSIS.md`**：
   标记为已解决（✓）

### 未来升级路径

- 如需混合模式，参考 `DYNAMIC_DISPATCH_ANALYSIS.md` 中的**方案 A（泛型参数）**
- 成本：编译时间 +20-30%，单态化代码 +10-15%
- 收益：完全灵活的缓存配置

## 总结表格

| 实施时间 | 方案 | 实施范围 | 成本 | 收益 |
|--------|------|--------|------|------|
| 现在 | 条件编译 | wrapper 层 | 低 | 消除条件分发，简化二重性 |
| 将来 | 泛型参数 | 所有缓存实现 | 高 | 支持混合模式，完全灵活 |

**建议**：采用**短期 + 长期**两阶段方案，既解决当前问题，也为未来扩展预留空间。
