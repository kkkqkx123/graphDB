# Inversearch 冷热索引 + WAL 默认实现设计分析

## 1. 概述

本文档分析在 inversearch crate 中提供**冷热索引分离 + WAL（Write-Ahead Log）**默认实现的可行性、架构设计和实施方案。

### 1.1 背景

当前 inversearch 提供以下存储方案：
- **内存存储** (`store-memory`)：性能最佳，但数据不持久化
- **文件存储** (`store-file`)：数据持久化，但性能较低
- **Redis 存储** (`store-redis`)：支持分布式，但有外部依赖
- **WAL 日志** (`store-wal`)：预写式日志，提供崩溃恢复能力
- **缓存存储** (`store-cached`)：内存 + 文件的混合方案

### 1.2 目标

设计一个**开箱即用**的默认存储方案，兼顾：
1. **高性能**：热数据在内存中，快速响应查询
2. **数据持久化**：冷数据和变更日志持久化，重启不丢失
3. **自动管理**：冷热分离、WAL 轮转、快照创建自动化
4. **低配置**：默认配置适用于大多数场景，无需调优

---

## 2. 技术架构

### 2.1 冷热索引分离架构

```
┌─────────────────────────────────────────────────────────┐
│                   Query Interface                        │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│              Cold-Warm Cache Manager                     │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Hot Cache (L1 Cache) - 内存                      │  │
│  │  - 最近访问的索引 (LRU 管理)                       │  │
│  │  - 大小限制：100MB-1GB (可配置)                    │  │
│  │  - 淘汰策略：LRU / LFU / ARC                       │  │
│  └───────────────────────────────────────────────────┘  │
│                            │                             │
│                            ▼                             │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Warm Cache (L2 Cache) - 内存映射文件             │  │
│  │  - 频繁访问的索引                                  │  │
│  │  - 使用 mmap 映射到磁盘文件                        │  │
│  │  - 大小限制：1GB-10GB (可配置)                     │  │
│  └───────────────────────────────────────────────────┘  │
│                            │                             │
│                            ▼                             │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Cold Storage - 磁盘文件                          │  │
│  │  - 不常访问的索引                                  │  │
│  │  - 压缩存储 (zstd)                                │  │
│  │  - 无大小限制                                      │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                  WAL Manager                             │
│  - 记录所有索引变更                                      │
│  - 异步刷盘 (batch commit)                              │
│  - 自动轮转 (rotation)                                  │
│  - 崩溃恢复                                              │
└─────────────────────────────────────────────────────────┘
```

### 2.2 数据流向

#### 写入流程：
```
1. 写入请求
   │
   ├─► 2. 写入 WAL (确保持久化)
   │
   ├─► 3. 写入 Hot Cache (立即可查)
   │
   └─► 4. 异步持久化
       ├─► 定期 flush 到 Warm Cache
       └─► 定期合并到 Cold Storage
```

#### 读取流程：
```
1. 读取请求
   │
   ├─► 2. 查找 Hot Cache (命中 → 返回)
   │    └─► 未命中 → 3
   │
   ├─► 3. 查找 Warm Cache (命中 → 返回，并提升到 Hot Cache)
   │    └─► 未命中 → 4
   │
   └─► 4. 从 Cold Storage 加载 (加载到 Hot Cache，返回)
```

---

## 3. 核心组件设计

### 3.1 ColdWarmCacheManager

```rust
pub struct ColdWarmCacheManager {
    // L1 缓存：热数据
    hot_cache: Arc<DashMap<String, Arc<Index>>>,
    
    // L2 缓存：温数据（使用 mmap 映射）
    warm_cache: Arc<MokaCache<String, MmapIndex>>,
    
    // 冷数据存储
    cold_storage: Arc<ColdStorage>,
    
    // WAL 管理器
    wal_manager: Arc<WALManager>,
    
    // 缓存策略
    eviction_policy: EvictionPolicy,
    
    // 配置
    config: ColdWarmCacheConfig,
    
    // 统计信息
    stats: Arc<CacheStats>,
}

impl ColdWarmCacheManager {
    /// 获取索引（自动处理缓存层级）
    pub async fn get_index(&self, index_name: &str) -> Result<Option<Arc<Index>>> {
        // 1. 尝试热缓存
        if let Some(index) = self.hot_cache.get(index_name) {
            self.stats.hot_hit.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(index.clone()));
        }
        
        // 2. 尝试温缓存
        if let Some(mmap_index) = self.warm_cache.get(index_name).await {
            self.stats.warm_hit.fetch_add(1, Ordering::Relaxed);
            // 提升到热缓存
            let index = mmap_index.load_to_memory()?;
            self.hot_cache.insert(index_name.to_string(), index.clone());
            return Ok(Some(index));
        }
        
        // 3. 从冷存储加载
        if let Some(index) = self.cold_storage.load_index(index_name)? {
            self.stats.cold_hit.fetch_add(1, Ordering::Relaxed);
            // 根据访问频率决定放入哪个缓存
            if self.is_hot_index(index_name) {
                self.hot_cache.insert(index_name.to_string(), index.clone());
            } else {
                self.warm_cache.insert(index_name.to_string(), index.to_mmap()?).await;
            }
            return Ok(Some(index));
        }
        
        self.stats.miss.fetch_add(1, Ordering::Relaxed);
        Ok(None)
    }
    
    /// 插入索引
    pub async fn insert_index(&self, index_name: &str, index: Arc<Index>) -> Result<()> {
        // 1. 写入 WAL
        self.wal_manager.log_change(IndexChange::Add {
            index_name: index_name.to_string(),
            content: serialize_index(&index)?,
        }).await?;
        
        // 2. 写入热缓存
        self.hot_cache.insert(index_name.to_string(), index.clone());
        
        // 3. 检查是否需要淘汰
        self.evict_if_needed().await?;
        
        Ok(())
    }
}
```

### 3.2 缓存淘汰策略

```rust
pub enum EvictionPolicy {
    /// LRU (Least Recently Used)
    LRU,
    
    /// LFU (Least Frequently Used)
    LFU { decay_factor: f64 },
    
    /// ARC (Adaptive Replacement Cache)
    ARC { p_target: usize },
    
    /// 基于时间的 TTL
    TTL { ttl: Duration },
    
    /// 混合策略
    Hybrid {
        hot_policy: EvictionPolicy,
        warm_policy: EvictionPolicy,
    },
}

impl ColdWarmCacheManager {
    /// 检查并执行淘汰
    async fn evict_if_needed(&self) -> Result<()> {
        let hot_size = self.get_hot_cache_size();
        let hot_limit = self.config.hot_cache_max_size;
        
        if hot_size > hot_limit {
            match &self.eviction_policy {
                EvictionPolicy::LRU => self.evict_lru_from_hot().await?,
                EvictionPolicy::LFU { decay_factor } => {
                    self.evict_lfu_from_hot(*decay_factor).await?
                },
                EvictionPolicy::ARC { p_target } => {
                    self.evict_arc_from_hot(*p_target).await?
                },
                _ => self.evict_oldest_from_hot().await?,
            }
        }
        
        Ok(())
    }
    
    /// 淘汰热缓存中最不常用的索引到温缓存
    async fn evict_oldest_from_hot(&self) -> Result<()> {
        // 找到最久未使用的索引
        if let Some((key, index)) = self.find_lru_entry() {
            self.hot_cache.remove(&key);
            
            // 降级到温缓存
            let mmap_index = index.to_mmap()?;
            self.warm_cache.insert(key.clone(), mmap_index).await;
            
            self.stats.evict_to_warm.fetch_add(1, Ordering::Relaxed);
        }
        
        Ok(())
    }
}
```

### 3.3 WAL 增强设计

```rust
pub struct WALManager {
    config: WALConfig,
    
    // 当前 WAL 写入器
    current_wal: Arc<Mutex<WALWriter>>,
    
    // WAL 文件列表（用于轮转）
    wal_files: Arc<Mutex<Vec<WALFileInfo>>>,
    
    // 检查点信息
    checkpoint: Arc<Mutex<CheckpointInfo>>,
    
    // 后台任务句柄
    background_tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
}

impl WALManager {
    /// 记录变更
    pub async fn log_change(&self, change: IndexChange) -> Result<()> {
        let mut writer = self.current_wal.lock().await;
        writer.write_change(&change).await?;
        
        // 检查是否需要轮转
        if writer.size() > self.config.max_wal_size {
            self.rotate_wal(&mut writer).await?;
        }
        
        Ok(())
    }
    
    /// WAL 轮转
    async fn rotate_wal(&self, writer: &mut WALWriter) -> Result<()> {
        // 1. 刷盘并关闭当前文件
        writer.flush().await?;
        writer.close().await?;
        
        // 2. 创建检查点
        self.create_checkpoint().await?;
        
        // 3. 创建新的 WAL 文件
        let new_wal_path = self.get_next_wal_path();
        *writer = WALWriter::new(new_wal_path, &self.config).await?;
        
        // 4. 更新文件列表
        self.wal_files.lock().await.push(WALFileInfo {
            path: writer.path().to_path_buf(),
            created_at: Utc::now(),
            size: 0,
        });
        
        // 5. 清理旧的 WAL 文件
        self.cleanup_old_wals().await?;
        
        Ok(())
    }
    
    /// 创建检查点
    async fn create_checkpoint(&self) -> Result<()> {
        let checkpoint_info = CheckpointInfo {
            timestamp: Utc::now(),
            wal_sequence: self.get_current_sequence(),
            index_state: self.capture_index_state()?,
        };
        
        // 原子写入检查点文件
        let checkpoint_path = self.config.base_path.join("checkpoint.json");
        atomic_write(&checkpoint_path, &serde_json::to_vec(&checkpoint_info)?).await?;
        
        *self.checkpoint.lock().await = checkpoint_info;
        
        Ok(())
    }
    
    /// 崩溃恢复
    pub async fn recover(&self) -> Result<RecoveryResult> {
        let checkpoint = self.load_checkpoint()?;
        let mut recovery_result = RecoveryResult::default();
        
        // 从检查点恢复索引状态
        self.restore_from_checkpoint(&checkpoint).await?;
        
        // 重放检查点后的 WAL
        let wal_files = self.get_wal_files_after(&checkpoint)?;
        for wal_file in wal_files {
            let changes = self.read_wal_file(&wal_file).await?;
            for change in changes {
                self.apply_change(&change).await?;
                recovery_result.replayed_changes += 1;
            }
        }
        
        Ok(recovery_result)
    }
}
```

### 3.4 后台管理任务

```rust
impl ColdWarmCacheManager {
    /// 启动后台任务
    pub fn start_background_tasks(&self) -> Vec<JoinHandle<()>> {
        vec![
            // 1. 定期 flush 热缓存到温缓存
            self.start_flush_task(),
            
            // 2. 定期合并温缓存到冷存储
            self.start_merge_task(),
            
            // 3. 定期 WAL 轮转
            self.start_wal_rotate_task(),
            
            // 4. 定期清理过期数据
            self.start_cleanup_task(),
            
            // 5. 统计信息上报
            self.start_stats_report_task(),
        ]
    }
    
    /// 定期 flush 任务
    fn start_flush_task(&self) -> JoinHandle<()> {
        let hot_cache = self.hot_cache.clone();
        let warm_cache = self.warm_cache.clone();
        let interval = self.config.flush_interval;
        
        tokio::spawn(async move {
            let mut timer = interval(interval);
            loop {
                timer.tick().await;
                
                // 将热缓存中久未访问的数据降级到温缓存
                Self::flush_hot_to_warm(&hot_cache, &warm_cache).await;
            }
        })
    }
    
    /// 定期合并任务
    fn start_merge_task(&self) -> JoinHandle<()> {
        let warm_cache = self.warm_cache.clone();
        let cold_storage = self.cold_storage.clone();
        let interval = self.config.merge_interval;
        
        tokio::spawn(async move {
            let mut timer = interval(interval);
            loop {
                timer.tick().await;
                
                // 将温缓存中久未访问的数据合并到冷存储
                Self::merge_warm_to_cold(&warm_cache, &cold_storage).await;
            }
        })
    }
}
```

---

## 4. 配置设计

### 4.1 默认配置

```rust
pub struct ColdWarmCacheConfig {
    // 热缓存配置
    pub hot_cache_max_size: usize,           // 默认：500MB
    pub hot_cache_eviction_policy: EvictionPolicy, // 默认：LRU
    
    // 温缓存配置
    pub warm_cache_max_size: usize,          // 默认：2GB
    pub warm_cache_mmap_enabled: bool,       // 默认：true
    
    // 冷存储配置
    pub cold_storage_path: PathBuf,          // 默认：./data/cold
    pub cold_storage_compression: bool,      // 默认：true
    pub cold_storage_compression_level: i32, // 默认：3 (zstd)
    
    // WAL 配置
    pub wal_enabled: bool,                   // 默认：true
    pub wal_path: PathBuf,                   // 默认：./data/wal
    pub wal_max_size: usize,                 // 默认：100MB
    pub wal_max_files: usize,                // 默认：10
    pub wal_flush_interval: Duration,        // 默认：100ms
    pub wal_auto_rotate: bool,               // 默认：true
    
    // 后台任务配置
    pub flush_interval: Duration,            // 默认：10s
    pub merge_interval: Duration,            // 默认：60s
    pub cleanup_interval: Duration,          // 默认：1h
    pub checkpoint_interval: Duration,       // 默认：5min
    
    // 性能调优
    pub write_buffer_size: usize,            // 默认：4MB
    pub read_ahead_enabled: bool,            // 默认：true
    pub pre_fetch_enabled: bool,             // 默认：true
}

impl Default for ColdWarmCacheConfig {
    fn default() -> Self {
        Self {
            hot_cache_max_size: 500 * 1024 * 1024, // 500MB
            hot_cache_eviction_policy: EvictionPolicy::LRU,
            
            warm_cache_max_size: 2 * 1024 * 1024 * 1024, // 2GB
            warm_cache_mmap_enabled: true,
            
            cold_storage_path: PathBuf::from("./data/cold"),
            cold_storage_compression: true,
            cold_storage_compression_level: 3,
            
            wal_enabled: true,
            wal_path: PathBuf::from("./data/wal"),
            wal_max_size: 100 * 1024 * 1024, // 100MB
            wal_max_files: 10,
            wal_flush_interval: Duration::from_millis(100),
            wal_auto_rotate: true,
            
            flush_interval: Duration::from_secs(10),
            merge_interval: Duration::from_secs(60),
            cleanup_interval: Duration::from_secs(3600),
            checkpoint_interval: Duration::from_secs(300),
            
            write_buffer_size: 4 * 1024 * 1024, // 4MB
            read_ahead_enabled: true,
            pre_fetch_enabled: true,
        }
    }
}
```

### 4.2 配置文件示例

```toml
# config.toml

[storage.cold_warm_cache]
# 热缓存配置
hot_cache_max_size_mb = 500
hot_cache_eviction_policy = "lru"

# 温缓存配置
warm_cache_max_size_gb = 2
warm_cache_mmap_enabled = true

# 冷存储配置
cold_storage_path = "./data/cold"
cold_storage_compression = true
cold_storage_compression_level = 3

# WAL 配置
wal_enabled = true
wal_path = "./data/wal"
wal_max_size_mb = 100
wal_max_files = 10
wal_flush_interval_ms = 100
wal_auto_rotate = true

# 后台任务配置
flush_interval_secs = 10
merge_interval_secs = 60
cleanup_interval_secs = 3600
checkpoint_interval_secs = 300

# 性能调优
write_buffer_size_mb = 4
read_ahead_enabled = true
pre_fetch_enabled = true
```

---

## 5. 实现方案

### 5.1 新增模块结构

```
crates/inversearch/src/storage/
├── mod.rs                    # 导出冷热缓存
├── cold_warm_cache/
│   ├── mod.rs               # 主模块，导出 ColdWarmCacheManager
│   ├── manager.rs           # ColdWarmCacheManager 实现
│   ├── config.rs            # 配置结构
│   ├── policy.rs            # 淘汰策略实现
│   ├── stats.rs             # 统计信息
│   └── background.rs        # 后台任务
├── wal/
│   ├── mod.rs               # 导出 WAL 相关类型
│   ├── writer.rs            # WALWriter 实现
│   ├── reader.rs            # WALReader 实现
│   ├── rotation.rs          # WAL 轮转逻辑
│   └── recovery.rs          # 崩溃恢复逻辑
└── mmap/
    ├── mod.rs               # 内存映射工具
    ├── index.rs             # MmapIndex 实现
    └── utils.rs             # mmap 工具函数
```

### 5.2 Feature Flag 设计

在 `Cargo.toml` 中新增 feature：

```toml
[features]
default = ["service", "store"]

# 现有 feature
service = ["tonic", "prost", "tokio/full"]
store = ["store-cold-warm-cache"]  # 修改默认存储
store-memory = []
store-file = []
store-redis = ["redis"]
store-wal = []

# 新增 feature
store-cold-warm-cache = ["store-wal", "mmap"]
mmap = ["memmap2"]
```

### 5.3 实现优先级

#### Phase 1: 基础架构 (1-2 周)
- [ ] 创建模块结构
- [ ] 实现 `ColdWarmCacheManager` 基础框架
- [ ] 实现三层缓存数据结构
- [ ] 实现基本的热温冷数据流转

#### Phase 2: WAL 增强 (1 周)
- [ ] 实现 WAL 轮转机制
- [ ] 实现检查点机制
- [ ] 实现崩溃恢复逻辑
- [ ] 集成到 `ColdWarmCacheManager`

#### Phase 3: 后台任务 (1 周)
- [ ] 实现定期 flush 任务
- [ ] 实现定期 merge 任务
- [ ] 实现定期 cleanup 任务
- [ ] 实现统计信息收集

#### Phase 4: 优化和测试 (1-2 周)
- [ ] 性能基准测试
- [ ] 压力测试
- [ ] 崩溃恢复测试
- [ ] 配置参数调优

---

## 6. 性能预期

### 6.1 性能指标

| 场景 | 当前方案 | 冷热缓存方案 | 提升 |
|------|---------|-------------|------|
| 热数据读取延迟 | ~10μs | ~10μs | - |
| 温数据读取延迟 | ~1ms | ~100μs | 10x |
| 冷数据读取延迟 | ~10ms | ~5ms | 2x |
| 写入延迟 | ~100μs | ~150μs | -33% |
| 崩溃恢复时间 | N/A | ~1s | - |
| 内存占用 | 全量 | 热数据 | 50-80% |

### 6.2 内存使用优化

```
场景：10GB 索引数据，100 万文档

当前方案 (纯内存):
- 内存占用：~10GB
- 启动时间：~30s (从文件加载)
- 重启后数据丢失 (无 WAL)

冷热缓存方案:
- 热缓存：~500MB (最近访问的 5% 数据)
- 温缓存：~2GB (频繁访问的 20% 数据，mmap)
- 冷存储：~7.5GB (剩余 75% 数据，压缩)
- 总内存占用：~2.5GB (减少 75%)
- 启动时间：~2s (仅加载元数据)
- 重启后数据完整 (WAL 保证)
```

---

## 7. 优势与风险

### 7.1 优势

1. **开箱即用**
   - 默认配置适用于大多数场景
   - 无需手动调优
   - 自动管理缓存层级

2. **性能优异**
   - 热数据访问速度与纯内存方案相同
   - 温数据访问速度提升 10 倍
   - 冷数据访问速度提升 2 倍

3. **数据可靠**
   - WAL 保证数据不丢失
   - 崩溃后自动恢复
   - 定期检查点加速恢复

4. **内存友好**
   - 仅热数据占用内存
   - 自动淘汰冷数据
   - 支持超大索引（超出内存容量）

5. **易于维护**
   - 自动轮转 WAL 文件
   - 自动清理过期数据
   - 详细的统计信息

### 7.2 风险与挑战

1. **实现复杂度**
   - 需要管理三层缓存
   - WAL 轮转和恢复逻辑复杂
   - 后台任务需要仔细设计

2. **并发控制**
   - 多层缓存一致性问题
   - mmap 文件的并发访问
   - WAL 写入的原子性保证

3. **边界情况**
   - 缓存穿透（大量未命中）
   - 缓存雪崩（同时淘汰）
   - WAL 文件损坏

4. **测试难度**
   - 崩溃恢复测试复杂
   - 并发场景难以复现
   - 性能测试需要真实数据

### 7.3 缓解措施

1. **分阶段实现**
   - 先实现基础功能
   - 逐步添加优化
   - 每个阶段充分测试

2. **充分测试**
   - 单元测试覆盖核心逻辑
   - 集成测试验证整体行为
   - 压力测试发现边界问题

3. **监控和告警**
   - 详细的统计信息
   - 异常情况日志
   - 性能指标监控

---

## 8. 后续扩展

### 8.1 短期扩展

1. **自适应缓存大小**
   - 根据系统内存自动调整
   - 根据访问模式动态调整

2. **预取优化**
   - 基于访问模式预测
   - 批量加载相关索引

3. **压缩算法选择**
   - 支持多种压缩算法
   - 根据数据类型自动选择

### 8.2 长期扩展

1. **分布式支持**
   - 多节点缓存同步
   - 分布式 WAL

2. **云原生支持**
   - 对象存储作为冷存储
   - 容器化部署

3. **智能缓存**
   - 机器学习预测访问模式
   - 自动优化缓存策略

---

## 9. 结论

在 inversearch 中提供**冷热索引 + WAL 默认实现**是可行且必要的：

### 9.1 技术可行性 ✅
- 已有 WAL 和缓存存储的基础实现
- 三层缓存架构成熟（类似数据库 buffer pool）
- Rust 生态提供必要的工具库（mmap、LRU 等）

### 9.2 业务价值 ✅
- 显著提升性能（特别是温数据访问）
- 大幅降低内存占用（75%+）
- 提供数据持久化保证
- 降低用户使用门槛（开箱即用）

### 9.3 实施建议 ✅
- 分 4 个阶段实施，总计 4-6 周
- 先实现基础功能，再优化性能
- 充分测试，特别是崩溃恢复场景
- 提供详细文档和配置说明

### 9.4 推荐方案
**强烈建议实施**，这将使 inversearch 成为一个：
- 高性能（热数据内存访问）
- 低内存占用（冷热分离）
- 数据可靠（WAL 保证）
- 易于使用（默认配置）

的生产级全文搜索存储引擎。

---

## 附录

### A. 相关参考资料

1. **缓存淘汰算法**
   - LRU: Least Recently Used
   - LFU: Least Frequently Used
   - ARC: Adaptive Replacement Cache
   - TinyLFU: W-TinyLFU (Caffeine 使用)

2. **内存映射文件**
   - Linux: `mmap(2)`
   - Windows: `CreateFileMapping`
   - Rust crate: `memmap2`

3. **WAL 实现参考**
   - RocksDB: WAL + Manifest
   - PostgreSQL: WAL + Checkpoint
   - Redis: AOF + RDB

### B. 依赖 crate 推荐

```toml
[dependencies]
# 内存映射
memmap2 = "0.9"

# LRU 缓存
lru = "0.12"
moka = { version = "0.12", features = ["future"] }

# 压缩
zstd = "0.13"
lz4_flex = "0.11"

# 序列化
bincode = "1.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 异步运行时
tokio = { version = "1.48", features = ["full"] }

# 并发容器
dashmap = "5.5"
crossbeam = "0.8"
```

### C. 性能测试工具

```rust
// 使用 criterion 进行基准测试
[dev-dependencies]
criterion = "0.5"

// 压力测试
[dev-dependencies]
proptest = "1.4"
```
