# PropertyTable 缓存设计文档

## 1. 澄清：存储方式与缓存需求

### 1.1 PropertyTable 存储方式

**PropertyTable 是列式存储**，但提供行式 API：

```rust
pub struct PropertyTable {
    schema: Vec<PropertySchema>,
    columns: Vec<Column>,        // ← 每个属性一列（列式存储）
    row_count: usize,
    free_list: Vec<u32>,
    overflow_store: OverflowStore,
    row_groups: Vec<RowGroup>,   // ← 行组用于压缩管理
    row_group_size: usize,       // 默认 2048
}
```

**列式存储特点**：

- 每个属性独立存储为一列 (`columns: Vec<Column>`)
- 同一属性的所有值连续存储
- 支持列级压缩（字典编码、RLE、FSST、ALP 等）
- 压缩单元是 RowGroup（2048 行）

**行式 API**：

- `get(prop_offset)` → 返回一行的所有属性
- `insert(values)` → 插入一行
- `update(offset, values)` → 更新一行

### 1.2 EdgeTable Cache 是什么？

**EdgeTable Cache 是属性值缓存**，不是缓存整个 EdgeTable：

```rust
pub trait EdgeTableCache: Send + Sync + Debug {
    // 缓存单个属性值
    fn get_by_offset(&self, prop_offset: u32, prop_name: &str) -> Option<Value>;
    fn put_by_offset(&self, prop_offset: u32, prop_name: &str, value: Value);
    fn invalidate_by_offset(&self, prop_offset: u32);
}
```

**设计目的**：

- 缓存热点属性值，减少 PropertyTable 访问
- 适用于高频访问特定边属性的场景
- 默认是 `NoOpEdgeTableCache`（不缓存）

**为什么 EdgeTable 本身不需要缓存**：

1. **CSR 已经优化遍历**：O(1) 边列表访问，连续内存布局
2. **PropertyTable 已经是内存结构**：Column 是 `Vec<Value>`，不是磁盘页
3. **边数据量大**：缓存整个 EdgeTable 内存开销大

### 1.3 PropertyTable 是否需要缓存？

**当前 PropertyTable 已经是内存结构**：

- `Column` 内部是 `Vec<Value>`
- 不需要像 RDB 那样的 buffer pool（那是磁盘页缓存）

**可能的缓存场景**：

1. **热点行缓存**：某些边被频繁访问
2. **解压缓存**：RowGroup 解压后的数据缓存
3. **当前不需要**：因为已经是内存结构

## 2. 如果需要缓存的设计（未来扩展）

如果 PropertyTable 数据量大到需要持久化到磁盘，则需要设计 buffer pool。

### 2.1 RDB Buffer Pool 参考

**PostgreSQL**：

- Clock-Sweep 算法 + Usage Count
- Dirty Flag + Pinning 机制

**MySQL InnoDB**：

- 改进的 LRU（young/old sublist）
- Midpoint Insertion（防止全表扫描污染缓存）

### 2.2 设计方案（未来）

如果需要实现 PropertyTable 缓存：

**缓存单位**：RowGroup（2048 行）

- 与压缩单元对齐
- 减少缓存碎片

**淘汰策略**：改进的 LRU

- Young sublist (5/8) + Old sublist (3/8)
- Midpoint insertion

**核心结构**：

```rust
pub struct PropertyTableCache {
    cache: HashMap<u32, CachedRowGroup>,
    young_list: VecDeque<u32>,
    old_list: VecDeque<u32>,
    current_size: usize,
    config: PropertyTableCacheConfig,
}
```

## 3. 结论

### 3.1 当前状态

| 组件            | 存储方式            | 是否需要缓存 | 原因                                   |
| --------------- | ------------------- | ------------ | -------------------------------------- |
| PropertyTable   | 列式存储（内存）    | **不需要**   | Column 是 Vec，已是内存结构            |
| EdgeTable       | CSR + PropertyTable | **不需要**   | CSR 已优化遍历，PropertyTable 已是内存 |
| EdgeTable Cache | 属性值缓存          | **不必要**   | 引入额外开销，过度设计                 |

### 3.2 EdgeTable Cache 是否需要？

**结论：EdgeTableCache 可能是不必要的**

**原因分析**：

1. **CSR 本身缓存友好**
   - 连续内存布局
   - CPU 缓存命中率高
   - O(1) 边列表访问

2. **PropertyTable 已是内存结构**

   ```rust
   pub fn get(&self, offset: u32) -> Option<Vec<(String, Option<Value>)>> {
       let row_idx = prop_offset_to_index(offset)?;
       // 直接 Vec 访问，O(1)，无解压开销
       self.columns.iter().map(|col| col.get(row_idx)).collect()
   }
   ```

   - `columns: Vec<Column>` 是内存结构
   - `Column` 内部是 `Vec<Value>`
   - 访问是 O(1)，无解压开销

3. **引入 LRU 缓存是过度设计**
   - Hash 查找开销：`HashMap::get()` 比 `Vec::get()` 慢
   - 额外的内存开销
   - 缓存失效复杂性

**EdgeTableCache 可能的唯一价值**：

- 如果 `overflow_store` 访问频繁（大值属性）
- 但这应该通过优化 `overflow_store` 本身解决

**建议**：

- 移除 `EdgeTableCache` trait 和相关实现
- 如果未来需要，考虑在 PropertyTable 层面优化

### 3.3 当前缓存架构总结

如果 PropertyTable 需要持久化到磁盘：

1. 实现 RowGroup 级别的 buffer pool
2. 参考 InnoDB 的改进 LRU 算法
3. 支持 dirty tracking 和 checkpoint

## 4. 参考资源

- PostgreSQL Buffer Cache: https://www.postgresql.org/docs/current/pgbuffercache.html
- MySQL InnoDB Buffer Pool: https://dev.mysql.com/doc/refman/8.0/en/innodb-buffer-pool.html
- DuckDB Row Groups: https://duckdb.org/docs/internals/storage
  /// LRU old 列表（较少访问）
  old_list: VecDeque<u32>,
  /// 当前缓存大小
  current_size: usize,
  /// 统计信息
  stats: CacheStats,
  }

/// 缓存统计信息
pub struct CacheStats {
pub hits: AtomicU64,
pub misses: AtomicU64,
pub evictions: AtomicU64,
pub dirty_writes: AtomicU64,
}

````

### 3.3 核心操作

#### 3.3.1 获取行组

```rust
impl PropertyTableCache {
    pub fn get_or_load(
        &mut self,
        row_group_id: u32,
        loader: impl FnOnce(u32) -> Vec<Vec<Option<Value>>>,
    ) -> CacheResult<&CachedRowGroup> {
        // 1. 尝试从缓存获取
        if let Some(entry) = self.cache.get_mut(&row_group_id) {
            entry.access_count += 1;
            entry.last_access = Instant::now();
            self.stats.hits.fetch_add(1, Ordering::Relaxed);

            // 如果在 old list，提升到 young list
            self.promote_to_young(row_group_id);

            return Ok(entry);
        }

        // 2. 缓存未命中
        self.stats.misses.fetch_add(1, Ordering::Relaxed);

        // 3. 加载行组
        let data = loader(row_group_id);
        let entry = CachedRowGroup {
            row_group_id,
            data,
            is_dirty: false,
            access_count: 1,
            pin_count: 0,
            last_access: Instant::now(),
        };

        // 4. 检查是否需要淘汰
        self.evict_if_needed(entry.size())?;

        // 5. 插入到 old list（midpoint insertion）
        self.insert_to_old(entry);

        Ok(self.cache.get(&row_group_id).unwrap())
    }
}
````

#### 3.3.2 淘汰策略

```rust
impl PropertyTableCache {
    fn evict_if_needed(&mut self, required_size: usize) -> CacheResult<()> {
        while self.current_size + required_size > self.config.max_size {
            // 1. 优先从 old list 尾部淘汰
            if let Some(victim_id) = self.old_list.pop_back() {
                if let Some(entry) = self.cache.get(&victim_id) {
                    // 不能淘汰被 pin 的条目
                    if entry.pin_count > 0 {
                        self.old_list.push_front(victim_id);
                        continue;
                    }

                    // 如果是脏页，需要写回
                    if entry.is_dirty {
                        self.flush_row_group(victim_id)?;
                        self.stats.dirty_writes.fetch_add(1, Ordering::Relaxed);
                    }

                    // 淘汰
                    self.current_size -= entry.size();
                    self.cache.remove(&victim_id);
                    self.stats.evictions.fetch_add(1, Ordering::Relaxed);
                }
            } else {
                // old list 为空，从 young list 淘汰
                if let Some(victim_id) = self.young_list.pop_back() {
                    // 同上处理
                } else {
                    // 缓存为空但仍然超出限制
                    return Err(CacheError::CapacityExceeded);
                }
            }
        }
        Ok(())
    }
}
```

#### 3.3.3 Pin/Unpin 操作

```rust
impl PropertyTableCache {
    /// Pin 行组（防止被淘汰）
    pub fn pin(&mut self, row_group_id: u32) -> CacheResult<()> {
        let entry = self.cache.get_mut(&row_group_id)
            .ok_or(CacheError::NotFound)?;
        entry.pin_count += 1;
        Ok(())
    }

    /// Unpin 行组
    pub fn unpin(&mut self, row_group_id: u32) -> CacheResult<()> {
        let entry = self.cache.get_mut(&row_group_id)
            .ok_or(CacheError::NotFound)?;
        entry.pin_count = entry.pin_count.saturating_sub(1);
        Ok(())
    }
}
```

### 3.4 与 PropertyTable 集成

```rust
pub struct PropertyTable {
    schema: Vec<PropertySchema>,
    name_indexer: NameIndexer,
    columns: Vec<Column>,
    row_count: usize,
    free_list: Vec<u32>,
    overflow_store: OverflowStore,
    row_groups: Vec<RowGroup>,
    row_group_size: usize,
    /// 新增：行组缓存
    row_group_cache: Option<Ack<PropertyTableCache>>,
}

impl PropertyTable {
    /// 获取单行属性（通过缓存）
    pub fn get(&self, prop_offset: u32) -> Option<Vec<(String, Option<Value>)>> {
        let row_idx = prop_offset_to_index(prop_offset)?;
        let row_group_id = (row_idx / self.row_group_size) as u32;
        let row_in_group = row_idx % self.row_group_size;

        // 如果有缓存，从缓存获取
        if let Some(cache) = &self.row_group_cache {
            let entry = cache.lock().get_or_load(row_group_id, |id| {
                self.load_row_group_from_disk(id)
            }).ok()?;

            return Some(self.extract_row_from_group(&entry.data, row_in_group));
        }

        // 无缓存，直接从磁盘读取
        self.get_from_disk(prop_offset)
    }

    /// 更新属性（标记脏页）
    pub fn update(&mut self, prop_offset: u32, values: &[(String, Value)]) -> StorageResult<()> {
        // ... 更新逻辑 ...

        // 标记缓存为脏
        if let Some(cache) = &self.row_group_cache {
            let row_idx = prop_offset_to_index(prop_offset).unwrap();
            let row_group_id = (row_idx / self.row_group_size) as u32;
            cache.lock().mark_dirty(row_group_id);
        }

        Ok(())
    }
}
```

## 4. 性能考虑

### 4.1 缓存大小估算

假设：

- 平均每条边有 3 个属性
- 每个属性平均 50 字节
- 行组大小 2048 行

每个行组大小 ≈ 2048 × 3 × 50 = 307,200 字节 ≈ 300 KB

建议缓存配置：

- 小型图（<100 万边）：缓存 100 个行组 ≈ 30 MB
- 中型图（100 万-1 亿边）：缓存 500 个行组 ≈ 150 MB
- 大型图（>1 亿边）：缓存 2000 个行组 ≈ 600 MB

### 4.2 并发控制

```rust
/// 线程安全的缓存包装
pub struct ThreadSafePropertyTableCache {
    inner: RwLock<PropertyTableCache>,
}

impl ThreadSafePropertyTableCache {
    pub fn get_or_load(&self, row_group_id: u32, loader: impl FnOnce(u32) -> Vec<Vec<Option<Value>>>) -> CacheResult<Arc<CachedRowGroup>> {
        // 读锁尝试获取
        {
            let read = self.inner.read().unwrap();
            if let Some(entry) = read.cache.get(&row_group_id) {
                return Ok(Arc::clone(entry));
            }
        }

        // 写锁加载
        let mut write = self.inner.write().unwrap();
        write.get_or_load(row_group_id, loader)
    }
}
```

### 4.3 预取策略

```rust
/// 顺序扫描时的预取
pub fn prefetch_sequential(&mut self, current_row_group: u32, prefetch_count: usize) {
    for i in 1..=prefetch_count {
        let next_id = current_row_group + i as u32;
        if !self.cache.contains_key(&next_id) {
            // 异步预取
            self.prefetch_queue.push(next_id);
        }
    }
}
```

## 5. 实现计划

### Phase 1：基础缓存框架

- [ ] 实现 `CachedRowGroup` 结构
- [ ] 实现 `PropertyTableCache` 基础结构
- [ ] 实现 LRU 淘汰策略
- [ ] 实现 pin/unpin 机制

### Phase 2：集成与优化

- [ ] 集成到 PropertyTable
- [ ] 实现 dirty tracking
- [ ] 添加统计信息
- [ ] 性能测试

### Phase 3：高级特性

- [ ] 多缓存实例（减少竞争）
- [ ] 预取策略
- [ ] 自适应缓存大小

## 6. 与 EdgeTable 缓存的对比

| 特性     | EdgeTable Cache          | PropertyTable Cache    |
| -------- | ------------------------ | ---------------------- |
| 缓存单位 | 属性偏移量 (prop_offset) | 行组 (RowGroup)        |
| 淘汰策略 | LRU                      | 改进的 LRU (young/old) |
| 脏页跟踪 | 有                       | 有                     |
| Pin 支持 | 有                       | 有                     |
| 适用场景 | 随机访问单条边属性       | 批量扫描 + 随机访问    |

## 7. 参考资源

- PostgreSQL Buffer Cache: https://www.postgresql.org/docs/current/pgbuffercache.html
- MySQL InnoDB Buffer Pool: https://dev.mysql.com/doc/refman/8.0/en/innodb-buffer-pool.html
- DuckDB Row Groups: https://duckdb.org/docs/internals/storage
