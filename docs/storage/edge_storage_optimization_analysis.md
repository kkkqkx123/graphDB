# Edge Property 存储优化分析

> 本文档分析 Edge Property 存储的缓存策略、OverflowStore 性能瓶颈及大数据量场景下的磁盘存储优化方向。基于当前代码实现（PropertyTable + CSR + OverflowStore）展开分析。

---

## 一、Edge Property Cache 分析

### 1.1 历史背景

早期的设计文档中曾提出 `EdgeTableCache` trait，意图为 Edge Property 添加缓存层：

```rust
pub trait EdgeTableCache: Send + Sync + Debug {
    fn get_by_offset(&self, prop_offset: u32, prop_name: &str) -> Option<Value>;
    fn put_by_offset(&self, prop_offset: u32, prop_name: &str, value: Value);
    fn invalidate_by_offset(&self, prop_offset: u32);
}
```

该设计的出发点是：参考传统数据库的 page cache，对"从磁盘加载"的数据做缓存。

### 1.2 分析结论：不需要 Edge 缓存

经过对当前代码实现的深入分析，**Edge Property 不需要额外的缓存层**。原因如下：

| 原因 | 说明 |
|------|------|
| **CSR 已是读优化结构** | CSR 提供 O(1) 的边列表访问 + 连续内存布局，CPU 缓存友好。`edges_of()` 使用 x86 prefetch 预取连续 Nbr 数据 |
| **PropertyTable 是内存结构** | Column 的内部存储是 `Vec<Value>`，不是磁盘 page。访问属性是 `Vec::get()` 直接索引，已是 O(1) |
| **缓存反而降低性能** | `HashMap::get()`（缓存查找）比 `Vec::get()`（直接索引）慢。引入缓存层增加了一次间接跳转 |
| **大数据量下内存开销高** | 边数据通常远大于顶点数据，缓存全量 PropertyTable 的内存开销不可接受 |
| **缓存失效复杂** | 边属性频繁更新（insert/delete/update），缓存失效逻辑散布在多个方法中，增加维护成本 |

### 1.3 record_cache 的设计印证

当前 [`record_cache.rs`](file:///d:/项目/database/graphDB/src/storage/cache/record_cache.rs) 和 [`types.rs`](file:///d:/项目/database/graphDB/src/storage/cache/types.rs) 中的设计注释明确说明：

> **Why No Edge Cache?**
> 1. CSR is already read-optimized
> 2. High memory cost
> 3. Frequent updates
> 4. Property access is O(1)

缓存层只负责 **Vertex Cache**（顶点记录）和 **ID Index Cache**（external_id → internal_id 映射），不缓存任何边数据。这与分析结论完全一致。

### 1.4 EdgeTable 中的 Cache 解耦

虽然 `EdgeTable` 当前没有嵌入 cache 字段，但如果在未来的实现中需要添加缓存（如业务层面的热点边缓存），建议使用 **trait 注入**方式，而非内嵌：

```rust
// 推荐：可插拔的 cache trait
trait EdgePropertyCache: Send + Sync {
    fn get(&self, offset: u32) -> Option<Vec<(String, Value)>>;
    fn set(&self, offset: u32, props: Vec<(String, Value)>);
    fn invalidate(&self, offset: u32);
}

// EdgeTable 只关注存储逻辑
pub struct EdgeTable {
    // ... 存储字段
    property_cache: Option<Box<dyn EdgePropertyCache>>, // 可选注入
}
```

---

## 二、OverflowStore 优化

### 2.1 当前实现

[OverflowStore](file:///d:/项目/database/graphDB/src/storage/edge/property_table.rs#L104-L109) 用于存储超过 256 字节的大值属性（长文本、序列化对象等）：

```rust
pub struct OverflowStore {
    data: HashMap<u64, Vec<u8>>,           // 大值数据存储
    index: HashMap<OverflowKey, OverflowPointer>, // 索引：(col_idx, row_idx) → overflow_id
    next_id: u64,
}
```

### 2.2 性能瓶颈

| 瓶颈 | 根因 | 影响 |
|------|------|------|
| **双层 HashMap 查找** | 每次 `retrieve()` 先查 index 再查 data，两次 hash 计算 + 两次指针跳转 | 延迟增加 50-100ns |
| **内存碎片化** | 每个大值独立分配 `Vec<u8>`，堆上分散存储 | 内存利用率低，TLB miss 增加 |
| **无缓存机制** | 频繁访问同一大值属性会重复序列化/反序列化 | CPU 开销高 |
| **全量 dump/load** | 持久化时一次序列化所有数据，无分片/流式处理 | 大数据量时内存峰值高 |

### 2.3 优化方案

#### 方案 A：连续内存池（推荐优先实施）

将分散的 `HashMap<u64, Vec<u8>>` 替换为连续内存池：

```rust
pub struct OverflowStore {
    /// 连续内存池（替代 HashMap<u64, Vec<u8>>）
    data_pool: Vec<u8>,
    /// 索引：overflow_id → (offset_in_pool, size)
    index: HashMap<u64, (u64, u32)>,
    /// 位置索引：(col_idx, row_idx) → overflow_id
    location_index: HashMap<OverflowKey, u64>,
    next_id: u64,
    /// 空闲列表（复用已删除大值的空间）
    free_list: Vec<(u64, u32)>, // (offset, size)
}
```

**优势：**
- **内存连续**：消除碎片化，CPU 缓存友好
- **减少分配次数**：`Vec<u8>` 预分配大块，避免每次 store 都触发堆分配
- **支持 mmap**：连续内存可直接 mmap 到文件，为磁盘存储做准备

**代价：**
- 删除大值时产生空闲碎片，需要 free_list 管理
- `data_pool` 扩容时需要拷贝已有数据（类似 `Vec::resize`）

**工作量：** 1-2 天

#### 方案 B：热点缓存层（可选增强）

在方案 A 基础上，为频繁访问的大值添加 LRU 缓存：

```rust
pub struct OverflowStore {
    // ... 连续内存池字段 ...
    /// 热点缓存：反序列化后的 Value，避免重复 to_bytes/from_bytes
    hot_cache: Option<LruCache<OverflowKey, Arc<Value>>>,
}
```

**适用场景：**
- 某些大值属性被高频读取（如长文本描述、JSON 属性）
- 读取远多于写入

**代价：**
- 增加内存开销
- 写入时需要 invalidate 或 update 缓存

**工作量：** 额外 0.5-1 天

#### 方案 C：mmap 磁盘存储（大数据量场景）

当数据量超过内存容量时，不再将数据全量加载到内存，而是使用 mmap 文件映射：

```rust
pub struct PersistentOverflowStore {
    mmap: MmapMut,
    file: File,
    header: FileHeader,     // magic, version, checksum
    /// 内存中的索引（仅索引，不存数据）
    index: HashMap<OverflowKey, FileOffset>,
}
```

**参考实现：**
- NeuG 的 [mmap 容器设计](file:///d:/项目/database/graphDB/ref/neug/storages/container/mmap_container.cc)
- 项目已有的 [PersistentContainer](file:///d:/项目/database/graphDB/src/storage/container/persistent/mod.rs)

**工作量：** 2-3 天

### 2.4 选择建议

| 场景 | 建议方案 | 理由 |
|------|----------|------|
| 边数 < 100 万 | 保持现有实现 | 现有实现足够，优化收益低 |
| 边数 100 万 - 1000 万 | 方案 A（内存池） | 内存碎片成为主要瓶颈 |
| 边数 1000 万 - 1 亿 | 方案 A + B（内存池 + 热缓存） | 大值属性访问频率差异大 |
| 边数 > 1 亿 | 方案 A + C（内存池 + mmap） | 内存容量不足，需要磁盘卸载 |

---

## 三、磁盘存储优化

### 3.1 当前问题

当前 Edge Property 的持久化采用 **全量序列化** 方式（见 [`EdgeTable::flush()`](file:///d:/项目/database/graphDB/src/storage/edge/edge_table.rs#L644-L705)）：

1. **全量 dump**：每次 flush 将整个 PropertyTable 序列化为一个二进制块
2. **无分片**：一次性加载全部数据，数据集超过内存即无法工作
3. **无校验**：无 magic number、无字段级版本号、无校验和
4. **逐行序列化**：`PropertyTable::dump()` 逐行调用 `Value::to_bytes()`，未利用批量优化

### 3.2 优化方向

#### 方向一：序列化规范化（短期）

参考 [persistence_redesign.md](file:///d:/项目/database/graphDB/docs/storage/persistence_redesign.md) 的 Phase 1：

- 引入 magic number + version + section_id 头部
- 统一编解码错误处理（消除 unwrap）
- PropertyTable dump 改用批量路径（复用 Column 的批量序列化路径）

**工作量：** 小（0.5 天）

#### 方向二：RowGroup 级分片存储（中期）

PropertyTable 已有 [RowGroup](file:///d:/项目/database/graphDB/src/storage/edge/property_table.rs#L236-L248) 结构（默认 2048 rows/group），但未用于持久化：

```rust
pub struct RowGroup {
    pub start_row: usize,
    pub end_row: usize,
    pub column_indices: Vec<usize>,
}
```

改造方案：

```
properties.bin 结构：
[file_header]
  ├── magic (4B) | version (4B) | row_group_count (4B)
  ├── [row_group_0_header]
  │     ├── start_row, end_row, data_offset, data_size
  │     ├── column_0_encoding
  │     └── column_1_encoding ...
  ├── [row_group_1_header] ...
  └── [row_group_data]
        ├── row_group_0: [column_0_data] [column_1_data] ...
        └── row_group_1: ...
```

**优势：**
- 按 RowGroup 粒度加载，无需全量读入内存
- 支持 LRU 缓存：最近使用的 RowGroup 常驻内存
- 写入时只 flush 变脏的 RowGroup

**参考：** DuckDB 的 RowGroup 设计

**工作量：** 中型（3-5 天）

#### 方向三：mmap 内存映射（长期）

对于超大数据集，使用 mmap 将文件直接映射到进程地址空间：

```
磁盘文件 ↔ mmap 映射 → 虚拟内存 → 缺页中断 → 按需加载
```

**适用场景：**
- 数据集超过物理内存
- 读多写少（mmap 写操作涉及缺页中断 + dirty page 回刷）
- 随机访问模式（mmap 由 OS 管理 page cache）

**注意事项：**
- mmap 写性能通常不如 `pwrite()`（需处理 SIGBUS）
- Windows 上使用 `MapViewOfFile` / `FlushViewOfFile`
- 需要处理文件截断和扩容

**工作量：** 大（5-7 天）

### 3.3 Edge 正反 CSR 数据冗余

[persistence_redesign.md](file:///d:/项目/database/graphDB/docs/storage/persistence_redesign.md) 中已指出的问题：

out_csr 和 in_csr 存储相同的 `prop_offset`，占用 2× 空间。更新时用 `assert_eq!` 校验一致性。

**优化方向（Phase 3）：**
- 从 Nbr 结构体中移除 `prop_offset`
- 新增 `EdgePropertyIndex: HashMap<EdgeId, u32>`
- Property 查找统一走 EdgePropertyIndex

**工作量：** 大（3-5 天），建议延后实施

---

## 四、总结与建议

### 优先级路线图

| 优先级 | 优化项 | 工作量化 | 影响范围 | 阶段 |
|--------|--------|----------|----------|------|
| P0 | OverflowStore 内存池优化（方案 A） | 1-2d | edge 模块 | 阶段一 |
| P1 | 序列化规范化（magic/version/checksum） | 0.5d | 持久化层 | 阶段一 |
| P1 | RowGroup 级分片存储 | 3-5d | edge 模块 | 阶段二 |
| P2 | OverflowStore 热缓存（方案 B） | 0.5-1d | edge 模块 | 阶段二 |
| P2 | OverflowStore mmap（方案 C） | 2-3d | edge 模块 | 阶段三 |
| P3 | EdgePropertyIndex 消除 prop_offset 冗余 | 3-5d | CSR + edge | 阶段三 |
| P3 | 全量 mmap 内存映射存储 | 5-7d | 持久化层 | 阶段三 |

### 关键决策

1. **不添加 Edge Property Cache**：CSR + PropertyTable 已提供 O(1) 访问，引入缓存弊大于利
2. **优先优化 OverflowStore**：内存碎片是当前最突出的性能瓶颈，连续内存池方案成熟且风险低
3. **持久化走 RowGroup 粒度**：PropertyTable 已有 RowGroup 结构，将其扩展到持久化层是自然演进方向
4. **mmap 作为长期目标**：仅在数据集超过内存容量时有必要，目前阶段优先级较低