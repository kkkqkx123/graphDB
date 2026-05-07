# 存储引擎改进方案分析

## 一、设计冲突分析

根据对 `database_storage_research.md` 文档和现有实现的分析，以下设计参考与当前项目目标存在冲突：

### 1.1 项目核心目标回顾

- **单机架构**：消除分布式复杂性
- **轻量级**：最小化外部依赖
- **高性能**：图遍历和点查优化
- **简单部署**：单一可执行文件

### 1.2 设计冲突矩阵

| 参考设计 | 来源 | 冲突程度 | 冲突原因 |
|----------|------|----------|----------|
| LSM-Tree 架构 | RocksDB | 🔴 高 | 图数据库查询模式不同于 KV 存储，LSM-Tree 增加复杂度 |
| MemTable + SSTable | RocksDB | 🔴 高 | 当前已有 MVCC/WAL，图写入涉及节点边关联性 |
| 分层存储 (Level 0-N) | RocksDB | 🔴 高 | 单机场景数据量有限，分层增加复杂度 |
| 向量化执行引擎 | DuckDB | 🟡 中 | 图查询是点查/遍历，非批量分析；但批量操作可受益 |
| 固定大小记录 | Neo4j | 🟡 中 | 属性数量可变，固定大小浪费空间；但 ID 映射可优化 |
| B+Tree 行存储 | SQLite | 🔴 高 | 图数据库属性访问模式不同，列式存储更合适 |
| 溢出页链表 | SQLite | 🟡 中 | 增加实现复杂度，可简化 |
| Compaction 后台线程 | RocksDB | 🟡 中 | 需要 LSM-Tree 架构支持，当前架构不需要 |

### 1.3 详细冲突分析

#### 1.3.1 LSM-Tree 架构 (不采用)

**RocksDB 设计**:
```
MemTable → Immutable MemTable → L0 SSTable → L1 → L2 → ... → Ln
         写入路径                    Compaction 路径
```

**冲突原因**:

1. **查询模式不同**:
   - RocksDB: 范围扫描、点查、前缀扫描
   - GraphDB: 点查 (O(1))、邻居遍历 (O(degree))、路径查询

2. **写入模式不同**:
   - RocksDB: 独立 KV 写入
   - GraphDB: 节点+边+属性关联写入，需要事务一致性

3. **复杂度问题**:
   - LSM-Tree 需要: MemTable、SSTable、Compaction、Level Manager
   - 当前架构: 列式存储 + CSR，已足够

**结论**: 保持当前列式存储 + CSR 架构，不引入 LSM-Tree。

---

#### 1.3.2 向量化执行引擎 (部分采用)

**DuckDB 设计**:
```
DataChunk (2048 rows)
├── Vector 1 (col 1)
├── Vector 2 (col 2)
└── Vector 3 (col 3)

向量化执行: 批量处理 2048 行，CPU 缓存友好
```

**冲突分析**:

| 场景 | 向量化收益 | 图数据库适用性 |
|------|-----------|----------------|
| 点查 (Get by ID) | 低 | 单行操作，向量化无收益 |
| 邻居遍历 | 低 | 度数通常 < 1000，批量收益有限 |
| 批量导入 | 高 | ✅ 可优化 |
| 批量导出 | 高 | ✅ 可优化 |
| 全表扫描 | 高 | ✅ 可优化 |

**结论**: 不实现完整向量化执行引擎，但优化批量操作接口。

---

#### 1.3.3 MemTable 写入缓冲 (不采用)

**RocksDB 设计**:
```
写入 → MemTable (内存跳表) → 达到阈值 → Immutable → Flush to SSTable
```

**冲突原因**:

1. **当前已有机制**:
   - MVCC 时间戳管理
   - WAL 持久化
   - 写入事务支持

2. **图写入特点**:
   - 节点和边需要原子写入
   - 属性更新涉及多列
   - MemTable 的 KV 模型不适用

**结论**: 不引入 MemTable，优化现有写入路径。

---

#### 1.3.4 分层存储 (不采用)

**RocksDB 设计**:
```
L0: 最近写入，可能有重叠
L1-Ln: 有序无重叠，每层大小倍增
Compaction: 后台合并，减少读放大
```

**冲突原因**:

1. **单机数据量**: 通常 < 100GB，单层足够
2. **复杂度**: 需要后台线程、合并策略、空间管理
3. **查询模式**: 图遍历需要随机访问，分层增加 IO

**结论**: 保持单层存储，简化架构。

---

#### 1.3.5 Block Cache (需要，但设计不同)

**RocksDB 设计**:
```
Block Cache (LRU)
├── Data Block 缓存
├── Index Block 缓存
└── Filter Block 缓存
```

**适配方案**:

图数据库的访问模式不同，需要设计 **Graph-aware Cache**:

```
Graph Cache
├── Vertex Cache (热点节点)
├── Edge Cache (热点边)
├── Neighbor Cache (邻居列表)
└── Property Cache (属性值)
```

**结论**: 需要缓存机制，但设计要适配图遍历模式。

---

## 二、可借鉴的设计

### 2.1 完全兼容的设计

| 设计 | 来源 | 现有实现 | 改进建议 |
|------|------|----------|----------|
| Validity Bitmap | DuckDB/Arrow | ✅ 已实现 [null_bitmap.rs](../src/storage/memory/null_bitmap.rs) | 保持 |
| Varint 编码 | SQLite | ✅ 已实现 [varint.rs](../src/storage/vertex/encoding/varint.rs) | 扩展到持久化格式 |
| 字典压缩 | DuckDB | ✅ 已实现 [dictionary.rs](../src/storage/vertex/encoding/dictionary.rs) | 保持 |
| RLE 压缩 | DuckDB | ✅ 已实现 [rle.rs](../src/storage/vertex/encoding/rle.rs) | 保持 |
| BitPacking | DuckDB | ✅ 已实现 [bitpacking.rs](../src/storage/vertex/encoding/bitpacking.rs) | 保持 |
| Bloom Filter | RocksDB | ✅ 已实现 [bloom_filter.rs](../src/utils/bloom_filter.rs) | 保持 |
| SSTable 持久化 | RocksDB | ✅ 已实现 [sstable.rs](../src/storage/persistence/sstable.rs) | 优化格式 |
| CSR 边存储 | Neo4j | ✅ 已实现 [csr.rs](../src/storage/edge/csr.rs) | 保持 |
| 延迟解压 | DuckDB | ✅ 已实现 [lazy.rs](../src/storage/vertex/encoding/lazy.rs) | 保持 |
| 分层压缩策略 | DuckDB | ✅ 已实现 [selector.rs](../src/storage/vertex/encoding/selector.rs) | 保持 |

### 2.2 需要适配的设计

| 设计 | 来源 | 适配方案 |
|------|------|----------|
| Block Cache | RocksDB | 改为 Graph-aware Cache，缓存热点节点和邻居 |
| Page 格式 | SQLite | 简化，去掉溢出页链表，大对象单独存储 |
| 统计信息 | 各数据库 | 收集列统计信息用于查询优化 |
| 检查点 | RocksDB | 实现增量检查点，减少恢复时间 |

---

## 三、分阶段改进方案

### Phase 1: 基础优化 (已完成 ✅)

**目标**: 完善基础存储组件

**已完成项目**:
- [x] Null Bitmap 优化 (8x 内存节省)
- [x] 多种压缩编码 (Dictionary, RLE, BitPacking, FSST, ALP)
- [x] Varint 编码
- [x] Bloom Filter
- [x] SSTable 持久化
- [x] CSR 边存储
- [x] 延迟解压支持
- [x] 分层压缩策略
- [x] 内存追踪器

---

### Phase 2: 缓存机制 (优先级: 高)

**目标**: 实现图感知缓存，减少重复数据访问

#### 2.1 Graph Cache 设计

```rust
// src/storage/cache/graph_cache.rs

pub struct GraphCache {
    vertex_cache: VertexCache,
    neighbor_cache: NeighborCache,
    property_cache: PropertyCache,
    config: CacheConfig,
}

pub struct VertexCache {
    entries: LruCache<VertexId, Arc<VertexEntry>>,
    memory_usage: AtomicUsize,
    max_memory: usize,
}

pub struct NeighborCache {
    entries: LruCache<VertexId, Arc<Vec<Nbr>>>,
    memory_usage: AtomicUsize,
    max_memory: usize,
}

pub struct CacheConfig {
    pub max_total_memory: usize,
    pub vertex_cache_ratio: f32,   // 0.3
    pub neighbor_cache_ratio: f32, // 0.5
    pub property_cache_ratio: f32, // 0.2
}
```

#### 2.2 缓存策略

```rust
pub enum CacheStrategy {
    Lru,           // 最近最少使用
    Lfu,           // 最不经常使用
    Arc,           // 自适应替换缓存
    GraphAware,    // 图感知：优先缓存高度节点
}

impl GraphCache {
    pub fn get_vertex(&self, vid: VertexId) -> Option<Arc<VertexEntry>> {
        self.vertex_cache.get(&vid)
    }
    
    pub fn get_neighbors(&self, vid: VertexId) -> Option<Arc<Vec<Nbr>>> {
        self.neighbor_cache.get(&vid)
    }
    
    pub fn prefetch_neighbors(&self, vid: VertexId, storage: &EdgeTable) {
        if !self.neighbor_cache.contains(&vid) {
            if let Some(neighbors) = storage.get_neighbors(vid) {
                self.neighbor_cache.insert(vid, Arc::new(neighbors));
            }
        }
    }
}
```

#### 2.3 实现任务

- [ ] VertexCache 实现
- [ ] NeighborCache 实现
- [ ] PropertyCache 实现
- [ ] CacheConfig 配置
- [ ] 集成到 VertexTable 和 EdgeTable
- [ ] 缓存命中率统计

---

### Phase 3: 持久化优化 (优先级: 中)

**目标**: 优化持久化格式和增量写入

#### 3.1 优化 SSTable 格式

当前问题:
- 字符串长度使用 8 字节固定前缀
- 没有 Varint 编码

改进方案:

```rust
// 改进前
fn write_string(data: &mut Vec<u8>, s: &str) {
    let len = s.len() as u64;
    data.extend_from_slice(&len.to_le_bytes()); // 8 bytes
    data.extend_from_slice(s.as_bytes());
}

// 改进后
fn write_string(data: &mut Vec<u8>, s: &str) {
    write_varint(data, s.len() as u64); // 1-9 bytes, 通常 1-2 bytes
    data.extend_from_slice(s.as_bytes());
}
```

#### 3.2 增量持久化

```rust
// src/storage/persistence/incremental_flush.rs

pub struct IncrementalFlush {
    dirty_tracker: DirtyPageTracker,
    flush_threshold: usize,
    last_flush: Instant,
}

impl IncrementalFlush {
    pub fn should_flush(&self) -> bool {
        self.dirty_tracker.dirty_count() >= self.flush_threshold
            || self.last_flush.elapsed() > Duration::from_secs(60)
    }
    
    pub fn flush_dirty(&mut self, storage: &mut PropertyGraph) -> Result<()> {
        let dirty_pages = self.dirty_tracker.drain_dirty();
        
        for page_id in dirty_pages {
            let data = storage.serialize_page(&page_id)?;
            let compressed = self.compressor.compress(&data)?;
            self.write_page(&page_id, &compressed)?;
        }
        
        self.last_flush = Instant::now();
        Ok(())
    }
}
```

#### 3.3 实现任务

- [ ] SSTable 格式优化 (Varint 长度编码)
- [ ] DirtyPageTracker 实现
- [ ] IncrementalFlush 实现
- [ ] 检查点机制

---

### Phase 4: 批量操作优化 (优先级: 中)

**目标**: 优化批量导入、导出、扫描操作

#### 4.1 批量读取接口

```rust
// src/storage/vertex/batch_reader.rs

pub const BATCH_SIZE: usize = 1024;

pub struct VertexBatchReader<'a> {
    table: &'a VertexTable,
    current_idx: usize,
    batch: Vec<VertexRecord>,
}

impl<'a> Iterator for VertexBatchReader<'a> {
    type Item = Vec<VertexRecord>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mut batch = Vec::with_capacity(BATCH_SIZE);
        
        for _ in 0..BATCH_SIZE {
            if let Some(record) = self.table.get_by_internal_id(self.current_idx) {
                batch.push(record);
                self.current_idx += 1;
            } else {
                break;
            }
        }
        
        if batch.is_empty() { None } else { Some(batch) }
    }
}
```

#### 4.2 批量写入优化

```rust
// src/storage/vertex/batch_writer.rs

pub struct VertexBatchWriter<'a> {
    table: &'a mut VertexTable,
    buffer: Vec<(String, Vec<(String, Value)>)>,
    buffer_size: usize,
}

impl<'a> VertexBatchWriter<'a> {
    pub fn insert(&mut self, external_id: String, properties: Vec<(String, Value)>) {
        self.buffer.push((external_id, properties));
        
        if self.buffer.len() >= self.buffer_size {
            self.flush();
        }
    }
    
    pub fn flush(&mut self) {
        if self.buffer.is_empty() { return; }
        
        // 批量预分配内存
        self.table.ensure_capacity(self.table.total_count() + self.buffer.len());
        
        // 批量写入
        for (id, props) in self.buffer.drain(..) {
            self.table.insert(&id, &props, self.ts);
        }
    }
}
```

#### 4.3 实现任务

- [ ] VertexBatchReader 实现
- [ ] VertexBatchWriter 实现
- [ ] EdgeBatchReader 实现
- [ ] EdgeBatchWriter 实现
- [ ] 性能基准测试

---

### Phase 5: 统计信息收集 (优先级: 低)

**目标**: 收集数据分布统计，支持查询优化

#### 5.1 列统计信息

```rust
// src/storage/stats/column_stats.rs

pub struct ColumnStatistics {
    pub null_count: usize,
    pub distinct_count: usize,
    pub min_value: Option<Value>,
    pub max_value: Option<Value>,
    pub avg_length: f64,
    pub histogram: Option<Histogram>,
}

pub struct Histogram {
    pub buckets: Vec<HistogramBucket>,
    pub most_common_values: Vec<(Value, usize)>,
}

pub struct HistogramBucket {
    pub lower: Value,
    pub upper: Value,
    pub count: usize,
    pub distinct_count: usize,
}
```

#### 5.2 统计收集器

```rust
// src/storage/stats/stats_collector.rs

pub struct StatsCollector {
    column_stats: HashMap<String, ColumnStatistics>,
    sample_rate: f64,
}

impl StatsCollector {
    pub fn collect(&mut self, column: &Column) {
        let stats = ColumnStatistics {
            null_count: column.null_count(),
            distinct_count: self.estimate_distinct(column),
            min_value: self.find_min(column),
            max_value: self.find_max(column),
            avg_length: self.calculate_avg_length(column),
            histogram: self.build_histogram(column),
        };
        
        self.column_stats.insert(column.name.clone(), stats);
    }
}
```

#### 5.3 实现任务

- [ ] ColumnStatistics 实现
- [ ] Histogram 实现
- [ ] StatsCollector 实现
- [ ] 集成到查询优化器

---

### Phase 6: 高级特性 (优先级: 低)

**目标**: 添加可选的高级特性

#### 6.1 内存映射 I/O (可选)

```rust
// src/storage/io/mmap.rs

pub struct MmapFile {
    file: File,
    mmap: MmapMut,
}

impl MmapFile {
    pub fn open(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        
        Ok(Self { file, mmap })
    }
}
```

#### 6.2 大对象存储 (可选)

```rust
// src/storage/large_object.rs

pub struct LargeObjectStore {
    objects: HashMap<u64, Vec<u8>>,
    threshold: usize,  // 超过此大小的属性单独存储
}

impl LargeObjectStore {
    pub fn store(&mut self, data: Vec<u8>) -> u64 {
        let id = self.next_id();
        self.objects.insert(id, data);
        id
    }
    
    pub fn load(&self, id: u64) -> Option<&[u8]> {
        self.objects.get(&id).map(|v| v.as_slice())
    }
}
```

#### 6.3 实现任务

- [ ] MmapFile 实现 (可选)
- [ ] LargeObjectStore 实现 (可选)
- [ ] 性能基准测试

---

## 四、实施路线图

```
Timeline:
├── Phase 1: 基础优化 ✅ (已完成)
│
├── Phase 2: 缓存机制 (2-3 周)
│   ├── VertexCache
│   ├── NeighborCache
│   └── 集成测试
│
├── Phase 3: 持久化优化 (1-2 周)
│   ├── SSTable 格式优化
│   ├── 增量持久化
│   └── 检查点
│
├── Phase 4: 批量操作优化 (1-2 周)
│   ├── BatchReader
│   ├── BatchWriter
│   └── 性能测试
│
├── Phase 5: 统计信息收集 (1 周)
│   ├── ColumnStatistics
│   └── StatsCollector
│
└── Phase 6: 高级特性 (可选)
    ├── MmapFile
    └── LargeObjectStore
```

---

## 五、风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 缓存一致性 | 高 | MVCC 时间戳验证 |
| 内存溢出 | 高 | 内存追踪器 + 硬限制 |
| 持久化数据损坏 | 严重 | Checksum + WAL |
| 性能回归 | 中 | 基准测试 + 特性开关 |
| 迁移兼容性 | 中 | 版本号 + 迁移工具 |

---

## 六、成功指标

| 指标 | 当前 | 目标 |
|------|------|------|
| 缓存命中率 | N/A | > 80% |
| 批量导入速度 | 基准 | +50% |
| 内存使用效率 | 基准 | +30% |
| 持久化时间 | 基准 | -50% |
| 恢复时间 | 基准 | < 5s (1GB) |

---

## 七、总结

### 不采用的设计

1. **LSM-Tree 架构**: 图数据库查询模式不同，当前架构更合适
2. **MemTable + SSTable**: 当前已有 MVCC/WAL，不需要额外写入缓冲
3. **分层存储**: 单机场景复杂度过高
4. **向量化执行引擎**: 图查询是点查/遍历，非批量分析
5. **B+Tree 行存储**: 列式存储更适合图数据库

### 需要实现的设计

1. **Graph-aware Cache**: 适配图遍历模式的缓存
2. **增量持久化**: 减少写入开销
3. **批量操作优化**: 提升导入导出性能
4. **统计信息收集**: 支持查询优化

### 保持现有设计

1. **列式存储 + CSR**: 已是最优选择
2. **MVCC 时间戳**: 已实现
3. **压缩编码**: 已实现多种算法
4. **延迟解压**: 已实现
