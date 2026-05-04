# Storage Engine Improvement Plan

## Executive Summary

This document outlines a phased improvement plan for GraphDB's storage engine, focusing on memory management and persistence enhancements based on industry best practices.

## Current State Assessment

### Strengths
1. **Clean Architecture**: Modular design with separate vertex and edge tables
2. **MVCC Support**: Basic multi-version concurrency control
3. **Column Storage**: Columnar layout for vertex properties
4. **CSR Format**: Compressed Sparse Row for edge storage

### Weaknesses
1. **No Memory Limits**: Memory grows unbounded
2. **Full Flush Only**: All data written on every persistence
3. **Poor Cache Locality**: Nested Vec structures
4. **No Compression**: Raw bytes stored directly
5. **No Checksums**: Data integrity not verified

## Improvement Phases

### Phase 1: Memory Management Foundation (Week 1-2)

**Goal**: Implement basic memory budget and monitoring

#### 1.1 Memory Configuration

```rust
// src/storage/config/memory_config.rs
pub struct MemoryConfig {
    pub max_total_memory: usize,
    pub vertex_memory_ratio: f32,    // Default: 0.4
    pub edge_memory_ratio: f32,      // Default: 0.4
    pub cache_memory_ratio: f32,     // Default: 0.2
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_total_memory: 1024 * 1024 * 1024,  // 1GB default
            vertex_memory_ratio: 0.4,
            edge_memory_ratio: 0.4,
            cache_memory_ratio: 0.2,
        }
    }
}
```

#### 1.2 Memory Tracker

```rust
// src/storage/memory/memory_tracker.rs
pub struct MemoryTracker {
    vertex_memory: AtomicUsize,
    edge_memory: AtomicUsize,
    cache_memory: AtomicUsize,
    config: MemoryConfig,
}

impl MemoryTracker {
    pub fn try_allocate_vertex(&self, size: usize) -> bool {
        let max = (self.config.max_total_memory as f32 
            * self.config.vertex_memory_ratio) as usize;
        self.try_allocate(&self.vertex_memory, size, max)
    }
    
    pub fn try_allocate(&self, counter: &AtomicUsize, size: usize, max: usize) -> bool {
        loop {
            let current = counter.load(Ordering::Relaxed);
            if current + size > max {
                return false;
            }
            if counter.compare_exchange_weak(
                current, current + size, 
                Ordering::SeqCst, Ordering::Relaxed
            ).is_ok() {
                return true;
            }
        }
    }
}
```

#### 1.3 Null Bitmap Optimization

```rust
// src/storage/vertex/null_bitmap.rs
pub struct NullBitmap {
    data: Vec<u64>,
    len: usize,
}

impl NullBitmap {
    pub fn new(len: usize) -> Self {
        let words = (len + 63) / 64;
        Self {
            data: vec![0; words],
            len,
        }
    }
    
    pub fn is_null(&self, idx: usize) -> bool {
        if idx >= self.len {
            return true;
        }
        let word = idx / 64;
        let bit = idx % 64;
        (self.data[word] >> bit) & 1 == 1
    }
    
    pub fn set_null(&mut self, idx: usize, is_null: bool) {
        if idx >= self.len {
            return;
        }
        let word = idx / 64;
        let bit = idx % 64;
        if is_null {
            self.data[word] |= 1u64 << bit;
        } else {
            self.data[word] &= !(1u64 << bit);
        }
    }
}
```

**Deliverables**:
- [ ] MemoryConfig with validation
- [ ] MemoryTracker with atomic operations
- [ ] NullBitmap with bit-level packing
- [ ] Unit tests for all components

### Phase 2: Block Cache Implementation (Week 3-4)

**Goal**: Add LRU cache for frequently accessed data

#### 2.1 Block Cache Core

```rust
// src/storage/cache/block_cache.rs
pub struct BlockCache {
    shards: Vec<RwLock<CacheShard>>,
    capacity: usize,
    memory_usage: AtomicUsize,
}

struct CacheShard {
    entries: HashMap<BlockId, LruEntry>,
    lru: VecDeque<BlockId>,
    size: usize,
}

pub struct BlockId {
    pub table_type: TableType,
    pub label_id: LabelId,
    pub block_number: u64,
}

pub enum TableType {
    Vertex,
    Edge,
    Property,
}
```

#### 2.2 Cache Interface

```rust
impl BlockCache {
    pub fn get(&self, id: &BlockId) -> Option<Arc<[u8]>> {
        let shard = self.get_shard(id);
        let guard = shard.read();
        guard.entries.get(id).map(|e| e.data.clone())
    }
    
    pub fn insert(&self, id: BlockId, data: Vec<u8>) -> bool {
        let shard = self.get_shard(&id);
        let mut guard = shard.write();
        
        // Check memory limit
        if !self.can_allocate(data.len()) {
            self.evict_lru(&mut guard, data.len());
        }
        
        guard.entries.insert(id.clone(), LruEntry {
            data: Arc::from(data.into_boxed_slice()),
            size: data.len(),
        });
        
        true
    }
    
    pub fn evict_lru(&self, shard: &mut CacheShard, required: usize) {
        while shard.size > self.capacity - required {
            if let Some(id) = shard.lru.pop_front() {
                if let Some(entry) = shard.entries.remove(&id) {
                    shard.size -= entry.size;
                    self.memory_usage.fetch_sub(entry.size, Ordering::Relaxed);
                }
            }
        }
    }
}
```

**Deliverables**:
- [ ] BlockCache with sharding
- [ ] LRU eviction policy
- [ ] Memory accounting
- [ ] Integration with VertexTable and EdgeTable

### Phase 3: Persistence Enhancement (Week 5-6)

**Goal**: Implement incremental persistence with compression

#### 3.1 Dirty Page Tracking

```rust
// src/storage/persistence/dirty_tracker.rs
pub struct DirtyPageTracker {
    dirty_pages: RwLock<HashSet<PageId>>,
    last_flush: RwLock<Instant>,
    flush_threshold: usize,
}

impl DirtyPageTracker {
    pub fn mark_dirty(&self, page_id: PageId) {
        self.dirty_pages.write().insert(page_id);
    }
    
    pub fn should_flush(&self) -> bool {
        let dirty = self.dirty_pages.read();
        dirty.len() >= self.flush_threshold 
            || self.last_flush.read().elapsed() > Duration::from_secs(60)
    }
    
    pub fn get_dirty_pages(&self) -> Vec<PageId> {
        self.dirty_pages.write().drain().collect()
    }
}
```

#### 3.2 Compression Support

```rust
// src/storage/persistence/compression.rs
pub enum CompressionType {
    None,
    Snappy,
    Zstd { level: i32 },
}

pub struct Compressor {
    compression: CompressionType,
}

impl Compressor {
    pub fn compress(&self, data: &[u8]) -> Vec<u8> {
        match self.compression {
            CompressionType::None => data.to_vec(),
            CompressionType::Snappy => {
                snap::raw::Encoder::new().compress_vec(data).unwrap()
            }
            CompressionType::Zstd { level } => {
                zstd::encode_all(data, level).unwrap()
            }
        }
    }
    
    pub fn decompress(&self, data: &[u8]) -> Vec<u8> {
        match self.compression {
            CompressionType::None => data.to_vec(),
            CompressionType::Snappy => {
                snap::raw::Decoder::new().decompress_vec(data).unwrap()
            }
            CompressionType::Zstd { .. } => {
                zstd::decode_all(data).unwrap()
            }
        }
    }
}
```

#### 3.3 Incremental Flush

```rust
// src/storage/persistence/flush_manager.rs
pub struct FlushManager {
    dirty_tracker: DirtyPageTracker,
    compressor: Compressor,
    flush_queue: mpsc::Sender<FlushTask>,
}

impl FlushManager {
    pub fn schedule_flush(&self, pages: Vec<PageId>) {
        let _ = self.flush_queue.send(FlushTask { pages });
    }
    
    pub fn background_flush(&self, storage: &PropertyGraph) {
        while let Ok(task) = self.flush_queue.recv() {
            for page_id in task.pages {
                if let Some(data) = storage.read_page(&page_id) {
                    let compressed = self.compressor.compress(&data);
                    self.write_page(&page_id, &compressed);
                }
            }
        }
    }
}
```

**Deliverables**:
- [ ] DirtyPageTracker
- [ ] Compression support (Snappy, Zstd)
- [ ] Incremental flush mechanism
- [ ] Background flush thread

### Phase 4: Storage Format Refactoring (Week 7-8)

**Goal**: Migrate to page-based storage with fixed-size records

#### 4.1 Page Format

```rust
// src/storage/page/mod.rs
pub const PAGE_SIZE: usize = 4096;

#[repr(C)]
pub struct PageHeader {
    pub page_id: u64,
    pub page_type: PageType,
    pub checksum: u32,
    pub record_count: u16,
    pub free_space: u16,
}

pub enum PageType {
    VertexHeader = 1,
    VertexData = 2,
    EdgeHeader = 3,
    EdgeData = 4,
    Property = 5,
    Schema = 6,
    Free = 255,
}

pub struct Page {
    header: PageHeader,
    data: [u8; PAGE_SIZE - size_of::<PageHeader>()],
}
```

#### 4.2 Fixed-Size Records

```rust
// Vertex record: 16 bytes
#[repr(C, packed)]
pub struct VertexRecord {
    pub internal_id: u64,
    pub timestamp: u64,
}

// Edge record: 32 bytes
#[repr(C, packed)]
pub struct EdgeRecord {
    pub src_id: u64,
    pub dst_id: u64,
    pub edge_id: u64,
    pub timestamp: u64,
}
```

#### 4.3 Flat CSR Migration

```rust
// src/storage/edge/flat_csr.rs
pub struct FlatCsr {
    offsets: Vec<usize>,
    edges: Vec<EdgeRecord>,
    degrees: Vec<usize>,
    capacity: usize,
}

impl FlatCsr {
    pub fn insert(&mut self, src: VertexId, edge: EdgeRecord) {
        let src_idx = src as usize;
        if src_idx >= self.capacity {
            self.resize(src_idx + 1);
        }
        
        let offset = self.offsets[src_idx];
        let degree = self.degrees[src_idx];
        
        // Insert into contiguous array
        self.edges.insert(offset + degree, edge);
        self.degrees[src_idx] += 1;
        
        // Update subsequent offsets
        for i in (src_idx + 1)..self.capacity {
            self.offsets[i] += 1;
        }
    }
    
    pub fn iter_edges(&self, src: VertexId) -> &[EdgeRecord] {
        let src_idx = src as usize;
        let offset = self.offsets[src_idx];
        let degree = self.degrees[src_idx];
        &self.edges[offset..offset + degree]
    }
}
```

**Deliverables**:
- [ ] Page format with header and checksum
- [ ] Fixed-size record definitions
- [ ] FlatCsr implementation
- [ ] Migration tool from old format

### Phase 5: Advanced Features (Week 9-10)

**Goal**: Add mmap support and huge pages

#### 5.1 Memory-Mapped I/O

```rust
// src/storage/io/mmap.rs
pub struct MmapFile {
    file: File,
    mmap: Mmap,
    path: PathBuf,
}

impl MmapFile {
    pub fn open(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        
        let mmap = unsafe { MmapMut::map_mut(&file)? };
        
        Ok(Self {
            file,
            mmap,
            path: path.to_path_buf(),
        })
    }
    
    pub fn read(&self, offset: usize, len: usize) -> &[u8] {
        &self.mmap[offset..offset + len]
    }
    
    pub fn write(&mut self, offset: usize, data: &[u8]) {
        self.mmap[offset..offset + data.len()].copy_from_slice(data);
    }
    
    pub fn flush(&self) -> io::Result<()> {
        self.mmap.flush()
    }
}
```

#### 5.2 Huge Pages Support

```rust
// src/storage/memory/huge_pages.rs
#[cfg(target_os = "linux")]
pub struct HugePageAllocator {
    huge_page_size: usize,
}

#[cfg(target_os = "linux")]
impl HugePageAllocator {
    pub fn allocate(&self, size: usize) -> *mut u8 {
        use std::ptr::null_mut;
        use libc::{mmap, MAP_ANONYMOUS, MAP_HUGETLB, PROT_READ, PROT_WRITE};
        
        let aligned_size = (size + self.huge_page_size - 1) 
            & !(self.huge_page_size - 1);
        
        unsafe {
            mmap(
                null_mut(),
                aligned_size,
                PROT_READ | PROT_WRITE,
                MAP_ANONYMOUS | MAP_HUGETLB,
                -1,
                0,
            ) as *mut u8
        }
    }
}
```

**Deliverables**:
- [ ] MmapFile implementation
- [ ] Huge page allocator (Linux only)
- [ ] Memory level configuration
- [ ] Performance benchmarks

## Testing Strategy

### Unit Tests
- Memory tracker edge cases
- Cache eviction correctness
- Compression roundtrip
- Page format serialization

### Integration Tests
- Memory limit enforcement
- Incremental persistence
- Recovery from checkpoint
- Migration from old format

### Performance Tests
- Cache hit rate benchmarks
- Flush latency measurements
- Memory usage profiling
- Comparison with baseline

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Memory regression | High | Comprehensive benchmarks before/after |
| Data corruption | Critical | Checksums, recovery tests |
| Performance degradation | Medium | Incremental rollout, feature flags |
| Migration failures | High | Backup mechanism, rollback support |

## Success Metrics

1. **Memory Efficiency**: 30% reduction in memory usage for same dataset
2. **Persistence Speed**: 50% reduction in flush time via incremental writes
3. **Cache Hit Rate**: >80% for typical workloads
4. **Recovery Time**: <5 seconds for 1GB database

## Timeline

| Phase | Duration | Start | End |
|-------|----------|-------|-----|
| Phase 1: Memory Foundation | 2 weeks | Week 1 | Week 2 |
| Phase 2: Block Cache | 2 weeks | Week 3 | Week 4 |
| Phase 3: Persistence | 2 weeks | Week 5 | Week 6 |
| Phase 4: Storage Format | 2 weeks | Week 7 | Week 8 |
| Phase 5: Advanced Features | 2 weeks | Week 9 | Week 10 |

## Conclusion

This improvement plan addresses the critical gaps in GraphDB's storage engine while maintaining backward compatibility. The phased approach allows for incremental delivery and validation of each component.
