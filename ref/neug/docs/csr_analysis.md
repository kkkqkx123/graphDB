# NeuG CSR (Compressed Sparse Row) 模块完整分析

## 1. 模块概述

CSR（Compressed Sparse Row，压缩稀疏行）是 NeuG 图存储引擎中用于表示和遍历图结构的核心数据结构。该模块位于 `src/storages/csr/` 和 `include/neug/storages/csr/` 下，提供了完整的图邻接表存储方案，支持多种边存储策略（单边/多边、可变/不可变）以及 MVCC 并发控制。

```
csr/
├── csr_base.h            # CSR 抽象基类与类型枚举
├── mutable_csr.h/.cc     # 可变 CSR 实现（支持动态增删边）
├── immutable_csr.h/.cc   # 不可变 CSR 实现（快照只读优化）
├── generic_view.h        # 通用遍历视图（MVCC 可见性过滤）
├── nbr.h                 # 邻居节点结构体定义
└── generic_view_utils.h/.cc  # 视图工具函数（偏移查找、记录转换）
```

## 2. 类型体系

### 2.1 CsrType 枚举

```cpp
enum class CsrType {
    kImmutable,          // 不可变多边形 CSR
    kMutable,            // 可变多边形 CSR
    kSingleMutable,      // 可变单边形 CSR
    kSingleImmutable,    // 不可变单边形 CSR
    kEmpty,              // 空 CSR
};
```

### 2.2 类继承结构

```
CsrBase (抽象基类)
└── TypedCsrBase<EDATA_T> (类型化基类)
    ├── MutableCsr<EDATA_T>
    │   └── 适用场景：高并发写入，支持动态扩容
    ├── SingleMutableCsr<EDATA_T>
    │   └── 适用场景：每对顶点最多一条边，写入场景
    ├── ImmutableCsr<EDATA_T>
    │   └── 适用场景：快照读取，紧凑存储
    ├── SingleImmutableCsr<EDATA_T>
    │   └── 适用场景：单边快照，极致紧凑
    └── EmptyCsr<EDATA_T>
        └── 适用场景：无边的占位符
```

### 2.3 实例化的数据类型

| 类型 | MutableCsr | SingleMutableCsr | ImmutableCsr | SingleImmutableCsr |
|------|:----------:|:----------------:|:------------:|:------------------:|
| `EmptyType` | ✅ | ✅ | ✅ | ✅ |
| `int32_t` | ✅ | ✅ | ✅ | ✅ |
| `uint32_t` | ✅ | ✅ | ✅ | ✅ |
| `int64_t` | ✅ | ✅ | ✅ | ✅ |
| `uint64_t` | ✅ | ✅ | ✅ | ✅ |
| `float` | ✅ | ✅ | ✅ | ✅ |
| `double` | ✅ | ✅ | ✅ | ✅ |
| `bool` | ✅ | ✅ | ✅ | ✅ |
| `Date` | ✅ | ✅ | ✅ | ✅ |
| `DateTime` | ✅ | ✅ | ✅ | ✅ |
| `Interval` | ✅ | ✅ | ✅ | ✅ |

> 边属性只支持单一非字符串类型（`kVarchar` 不支持），这是由 `determine_search_prop_type()` 函数决定的。

## 3. 邻居结构 (nbr.h)

### 3.1 ImmutableNbr — 不可变邻居

```cpp
template <typename EDATA_T>
struct ImmutableNbr {
    vid_t neighbor;     // 目标顶点 ID
    EDATA_T data;       // 边属性数据
};
```

- 无时间戳字段，存储紧凑
- 通过 `neighbor == std::numeric_limits<vid_t>::max()` 表示已删除

**EmptyType 特化：** 使用 `union` 复用内存，`neighbor` 和 `data` 共享同一空间。

### 3.2 MutableNbr — 可变邻居

```cpp
template <typename EDATA_T>
struct MutableNbr {
    vid_t neighbor;                     // 目标顶点 ID
    std::atomic<timestamp_t> timestamp; // MVCC 时间戳
    EDATA_T data;                       // 边属性数据
};
```

- `timestamp` 为 `std::atomic`，保证并发安全
- `timestamp == std::numeric_limits<timestamp_t>::max()` 表示已删除（逻辑删除标记）
- `timestamp == INVALID_TIMESTAMP` 表示特殊无效状态

**EmptyType 特化：** `timestamp` 和 `data` 通过 `union` 复用，节省内存。

## 4. 核心 CSR 实现详解

### 4.1 MutableCsr — 可变 CSR

#### 4.1.1 内存布局

```
adj_list_buffer_    // nbr_t** 指针数组，每个元素指向一个顶点的邻接表
degree_list_        // int[] 每个顶点的当前边数 (实际大小)
cap_list_           // int[] 每个顶点的邻接表容量
nbr_list_           // nbr_t[] 连续的邻居数据存储区
locks_              // SpinLock[] 每顶点自旋锁
```

```
顶点 i:
┌─────────────────────────────────────┐
│ adj_list_buffer_[i] ──→ nbr_t[nbr1] │
│ degree_list_[i] = 2     nbr_t[nbr2] │
│ cap_list_[i] = 4        (空闲)      │
│                         (空闲)      │
│ locks_[i] (SpinLock)               │
└─────────────────────────────────────┘
```

#### 4.1.2 单边插入 (put_edge)

```cpp
int32_t put_edge(vid_t src, vid_t dst, const EDATA_T& data, timestamp_t ts, Allocator& alloc) {
    locks_[src].lock();                          // 1. 获取顶点锁
    int sz = sizes[src];
    int cap = caps[src];
    if (sz == cap) {                             // 2. 容量不足，扩容
        cap += (cap >> 1);                       //    增长 1.5 倍
        cap = std::max(cap, 8);                  //    最小容量为 8
        nbr_t* new_buffer = alloc.allocate(cap * sizeof(nbr_t));
        if (sz > 0) memcpy(new_buffer, buffers[src], sz * sizeof(nbr_t));
        buffers[src] = new_buffer;
        caps[src] = cap;
    }
    int32_t prev_size = sizes[src]++;            // 3. 追加边
    buffers[src][prev_size].neighbor = dst;
    buffers[src][prev_size].data = data;
    buffers[src][prev_size].timestamp.store(ts);
    edge_num_.fetch_add(1);
    locks_[src].unlock();                        // 4. 释放锁
    return prev_size;                            // 返回偏移量
}
```

#### 4.1.3 批量边插入 (batch_put_edges)

- 预计算每个顶点的目标容量（考虑 `DEFAULT_RESERVE_RATIO` 预留比例）
- 一次性分配所有新邻接表空间
- 复制旧数据到新位置，追加新边
- 避免逐条插入时的反复分配

#### 4.1.4 删除操作

| 方法 | 行为 | 实现方式 |
|------|------|----------|
| `delete_edge` | 单条删除 | 将 `timestamp` 设为 `MAX` |
| `revert_delete_edge` | 撤销删除 | 恢复原始 `timestamp` |
| `batch_delete_edges` | 批量按偏移删除 | 按 offset 标记 timestamp 为 MAX |
| `batch_delete_edges` | 批量按顶点对删除 | 遍历匹配 dst，标记 timestamp 为 MAX |
| `batch_delete_vertices` | 删除关联顶点的所有边 | 源顶点清空度数；目标顶点过滤邻接表 |

#### 4.1.5 压缩 (compact)

```cpp
void compact() {
    // 原地压缩：移除 timestamp == INVALID_TIMESTAMP 的已删除边
    // 不缩小邻接表容量，只调整 degree
    for each vertex i:
        read_ptr → write_ptr 双指针压缩
        degree[i] -= removed_count
}
```

#### 4.1.6 排序 (batch_sort_by_edge_data)

```cpp
void batch_sort_by_edge_data(timestamp_t ts) {
    // 对每个顶点的邻接表按边属性值排序
    for each vertex i:
        std::sort(begin, begin + degree[i], compare by data)
    unsorted_since_ = ts;  // 记录排序时间点
}
```

排序后，`unsorted_since_` 用于优化范围查询（`foreach_nbr_gt/lt`）。

### 4.2 SingleMutableCsr — 单边形可变 CSR

#### 内存布局

```
nbr_list_    // nbr_t[] 定长数组，顶点 i 的边固定在位置 i
```

- 每个顶点最多一条边，无需 degree/cap 数组
- 无 locks（单线程或上层保证）
- 插入时直接覆盖：`nbrs[src].neighbor = dst; nbrs[src].data = data;`

#### 与 MutableCsr 的差异

| 特性 | MutableCsr | SingleMutableCsr |
|------|------------|------------------|
| 内存结构 | 指针数组 + 动态缓冲区 | 单一连续数组 |
| 并发控制 | SpinLock 数组 | 无 |
| 扩容 | 支持 | 不支持 |
| 删除标记 | timestamp → MAX | timestamp → MAX |
| 紧凑性 | 较低（多数组） | 高（单数组） |

### 4.3 ImmutableCsr — 不可变 CSR

#### 内存布局

```
adj_list_buffer_    // nbr_t** 指针数组
degree_list_buffer_ // int[] 度数数组
nbr_list_buffer_    // nbr_t[] 连续邻居数据
```

- 无 `cap_list_`，无 `locks_`
- 所有边紧凑排列在 `nbr_list_buffer_` 中
- 通过 `batch_put_edges` 支持批量写入（重新分配并重新排列）

#### 批量插入策略

```cpp
void batch_put_edges(src_list, dst_list, data_list, ts) {
    // 1. 保存旧度数
    // 2. 增加新度数
    // 3. 扩容 nbr_list_buffer
    // 4. 从后向前 memmove 旧数据到新位置
    // 5. 追加新边到每个顶点的尾部
}
```

从后向前移动确保在原地扩容时数据不被覆盖。

#### 删除操作

- 使用 `neighbor == std::numeric_limits<vid_t>::max()` 作为删除标记（而非 timestamp）
- `compact()` 时物理移除已删除边，重新计算指针

### 4.4 SingleImmutableCsr — 单边形不可变 CSR

- 最简单的实现：单一 `nbr_list_buffer_` 数组
- 删除标记：`neighbor = MAX`
- 无 compact（空操作）

### 4.5 EmptyCsr — 空 CSR

- 所有方法为空操作或返回零值
- 用于不存在边的 label 三元组占位

## 5. 通用遍历视图 (GenericView)

### 5.1 内存布局配置

```cpp
struct NbrIterConfig {
    int stride : 16;     // 相邻条目间的字节步长
    int ts_offset : 8;   // 时间戳字段的字节偏移 (0 表示不可变)
    int data_offset : 8; // 边属性字段的字节偏移
};
```

位域设计使配置结构仅占用 4 字节。

### 5.2 视图类型判断

```cpp
CsrViewType type() const {
    if (degrees_ == nullptr) {        // 单边模式
        return cfg_.ts_offset != 0 ? kSingleMutable : kSingleImmutable;
    } else {                          // 多边形模式
        return cfg_.ts_offset != 0 ? kMultipleMutable : kMultipleImmutable;
    }
}
```

### 5.3 get_edges — 获取邻接表

```cpp
NbrList get_edges(vid_t v) const {
    if (degrees_ == nullptr) {
        // 单边：固定位置
        start_ptr = adjlists_ + v * stride;
        end_ptr = start_ptr + stride;
    } else {
        // 多边：通过指针数组定位
        start_ptr = (int64_t*)adjlists_[v];
        end_ptr = start_ptr + degrees_[v] * stride;
    }
    return NbrList{start_ptr, end_ptr, cfg_, timestamp_};
}
```

### 5.4 NbrIterator — MVCC 迭代器

```cpp
NbrIterator& operator++() {
    cur += cfg.stride;
    while (cur != end && get_timestamp() > timestamp) {
        cur += cfg.stride;  // 跳过不可见边
    }
    return *this;
}
```

**关键行为：**
- `*it` 返回邻居顶点 ID
- `it.get_data_ptr()` 返回边属性指针
- `it.get_timestamp()` 返回边时间戳
- 自动跳过 `timestamp > read_ts` 的边

### 5.5 NbrList — 邻接表容器

```cpp
struct NbrList {
    const void* start_ptr;
    const void* end_ptr;
    NbrIterConfig cfg;
    timestamp_t timestamp;

    NbrIterator begin() const;
    NbrIterator end() const;
    bool empty() const;
};
```

POD 结构，适合高性能遍历。

### 5.6 EdgeDataAccessor — 边属性访问器

```cpp
struct EdgeDataAccessor {
    DataTypeId data_type_;
    ColumnBase* data_column_;  // nullptr 表示内联 (bundled) 存储

    Property get_data(const NbrIterator& it) const;
    T get_typed_data<T>(const NbrIterator& it) const;
    void set_data(const NbrIterator& it, const Property& prop, timestamp_t ts);
};
```

**两种存储模式：**
1. **Bundled（内联）：** 数据直接存储在 `MutableNbr` / `ImmutableNbr` 中
2. **Column（列式）：** 数据存储在独立的 `ColumnBase` 中，边中只存索引

### 5.7 TypedView — 类型化视图

```cpp
template <typename T, CsrViewType TYPE>
struct TypedView {
    void foreach_nbr_gt(vid_t v, const T& threshold, const FUNC_T& func);
    void foreach_nbr_lt(vid_t v, const T& threshold, const FUNC_T& func);
};
```

**优化策略：**
- 当边按 `data` 排序后（`unsorted_since_` 之后未写入），可使用二分查找
- `foreach_nbr_gt`：从后向前遍历，遇到 `< threshold` 即停止
- `foreach_nbr_lt`：先用二分定位边界，再向前遍历

## 6. 视图工具函数 (generic_view_utils)

### 6.1 偏移查找

```cpp
int32_t fuzzy_search_offset_from_nbr_list(
    const NbrList& nbr_list, vid_t expected_nbr,
    const void* expected_prop, const DataTypeId& type);
```

- 先遍历所有匹配的 `neighbor` 收集候选偏移
- 如果候选 > 1，进一步比较属性值和时间戳精确匹配
- 失败返回 `std::numeric_limits<int32_t>::max()`

### 6.2 EdgeRecord 到 CSR 偏移对转换

```cpp
std::pair<int32_t, int32_t> record_to_csr_offset_pair(
    const GenericView& oe, const GenericView& ie,
    const EdgeRecord& record, const std::vector<DataType>& props);
```

- 根据 `EdgeRecord` 的方向（出边/入边）获取对应的 NbrList
- 在当前视图中计算 `cur_offset`
- 在反向视图中 fuzzy search `another_offset`
- 返回 `(oe_offset, ie_offset)` 对

### 6.3 搜索属性类型决策

```cpp
DataTypeId determine_search_prop_type(const std::vector<DataType>& props) {
    // 单属性且非字符串 → 使用该类型
    // 多属性或字符串 → 使用 uint64 (指针/索引比较)
    return (props.size() == 1 && props[0].id() != DataTypeId::kVarchar)
               ? props[0].id()
               : DataTypeId::kUInt64;
}
```

### 6.4 跨视图偏移查找

```cpp
int32_t search_other_offset_with_cur_offset(
    const GenericView& cur_view, const GenericView& other_view,
    vid_t src_lid, vid_t other_lid, int32_t cur_offset,
    const std::vector<DataType>& props);
```

- 已知当前视图中的偏移，在反向视图中查找对应偏移
- 不能简单 `begin() + offset`，因为可能有已删除边需要跳过

## 7. CsrBase 抽象接口

```cpp
class CsrBase {
    static constexpr size_t INFINITE_CAPACITY = max;

    virtual CsrType csr_type() const = 0;
    virtual GenericView get_generic_view(timestamp_t ts) const = 0;
    virtual timestamp_t unsorted_since() const;
    virtual size_t size() const = 0;
    virtual size_t edge_num() const = 0;
    virtual void open(...) = 0;
    virtual void open_in_memory(...) = 0;
    virtual void open_with_hugepages(...) = 0;
    virtual void dump(...) = 0;
    virtual void reset_timestamp() = 0;
    virtual void compact() = 0;
    virtual void resize(vid_t vnum) = 0;
    virtual size_t capacity() const = 0;
    virtual void close() = 0;
    virtual void batch_sort_by_edge_data(timestamp_t ts);
    virtual void batch_delete_vertices(...) = 0;
    virtual void batch_delete_edges(...) = 0;
    virtual void delete_edge(...) = 0;
    virtual void revert_delete_edge(...) = 0;
    virtual int32_t put_generic_edge(...) = 0;
    virtual std::tuple<...> batch_export(...) const = 0;
};
```

## 8. 持久化文件格式

### 8.1 MutableCsr 文件

| 文件后缀 | 内容 |
|---------|------|
| `.meta` | `unsorted_since` (timestamp) + `edge_num` (uint64) |
| `.deg` | 度数数组 (int[]) |
| `.cap` | 容量数组 (int[]) |
| `.nbr` | 邻居数据连续存储 (nbr_t[])，含 MD5 校验的 FileHeader |
| `.buf` | 邻接表指针数组 (临时文件) |

### 8.2 ImmutableCsr 文件

| 文件后缀 | 内容 |
|---------|------|
| `.meta` | `unsorted_since` (timestamp) + `edge_num` (uint64) |
| `.deg` | 度数数组 (int[]) |
| `.nbr` | 邻居数据连续存储 (nbr_t[]) |
| `.adj` | 邻接表指针数组 (临时文件) |

### 8.3 Single* CSR 文件

| 文件后缀 | 内容 |
|---------|------|
| `.meta` | `edge_num` (uint64) |
| `.snbr` | 单边形邻居数组 (nbr_t[]) |

### 8.4 Dump 过程 (MutableCsr)

```
1. dump_meta() → 写入 .meta
2. degree_list_->Dump() → 写入 .deg
3. 顺序写入所有邻接表到 .nbr (含 MD5 计算)
4. 更新 .nbr 文件头中的 MD5 校验
5. cap_list_->Dump() → 写入 .cap
```

## 9. 加载与打开流程

### 9.1 MutableCsr open_internal

```
1. load_meta() → 读取 unsorted_since 和 edge_num
2. 打开/创建容器：
   - nbr_list_   ← snapshot/.nbr + tmp/.nbr
   - degree_list_ ← snapshot/.deg + tmp/.deg
   - adj_list_buffer_ ← tmp/.buf
   - cap_list_   ← snapshot/.cap + tmp/.cap
3. 调整 adj_list_buffer_ 大小为 v_cap * sizeof(nbr_t*)
4. 分配 locks_ = new SpinLock[v_cap]
5. 从 nbr_list_ 和 degree_list_ 重建 adj_list 指针数组
6. 验证 edge_num 一致性
```

### 9.2 ImmutableCsr open_internal

```
1. load_meta() → 读取 unsorted_since 和 edge_num
2. 打开/创建容器：
   - degree_list_buffer_ ← snapshot/.deg + tmp/.deg
   - nbr_list_buffer_    ← snapshot/.nbr + tmp/.nbr
   - adj_list_buffer_    ← tmp/.adj (或匿名映射)
3. 重建 adj_list 指针数组
```

### 9.3 三种打开模式

| 模式 | MemoryLevel | 用途 |
|------|-------------|------|
| `open()` | kSyncToFile | 生产环境，文件-backed + 临时文件 |
| `open_in_memory()` | kInMemory | 测试/开发，纯内存 |
| `open_with_hugepages()` | kHugePagePreferred | 高性能场景，大页内存 |

## 10. MVCC 机制在 CSR 中的实现

### 10.1 可见性规则

```
读事务时间戳 = read_ts
边可见当且仅当: edge.timestamp <= read_ts
边已删除当且仅当: edge.timestamp == MAX_TIMESTAMP
```

### 10.2 写入隔离

| CSR 类型 | 并发写入保护 |
|----------|-------------|
| MutableCsr | 每顶点 SpinLock |
| SingleMutableCsr | 无（假设单线程或上层序列化） |
| ImmutableCsr | 不可变，无需保护 |

### 10.3 生命周期管理

- `MutableCsr` 通过 `ArenaAllocator` 分配邻接表缓冲区
- 旧缓冲区不会立即释放，由 ArenaAllocator 统一管理
- 读取线程持有旧缓冲区引用直到遍历完成

### 10.4 reset_timestamp

```cpp
void reset_timestamp() {
    // 将所有非 INVALID_TIMESTAMP 的边 timestamp 重置为 0
    // 用于快照后重新开始新的事务
}
```

## 11. 容量与 Resize 策略

### 11.1 MutableCsr

- 单顶点扩容：1.5 倍增长，最小 8
- 全局 `resize(vnum)`：扩展顶点数量，重新分配 locks_
- `capacity()` 返回 `INFINITE_CAPACITY`（假设可无限扩展）

### 11.2 ImmutableCsr

- `resize(vnum)`：仅扩展 degree 和 adj_list 指针数组
- 不重新分配 nbr_list_buffer_（不可变假设）

### 11.3 Single* CSR

- `resize(vnum)`：直接调整 nbr_list 大小
- `capacity()` 返回 `vertex_capacity()`（固定大小）

## 12. 与 EdgeTable 的集成

`EdgeTable` 使用 CSR 的方式：

```cpp
class EdgeTable {
    std::unique_ptr<CsrBase> out_csr_;    // 出边 CSR
    std::unique_ptr<CsrBase> in_csr_;     // 入边 CSR
    std::unique_ptr<Table> table_;        // 边属性表 (列存)
};
```

**CSR 类型选择逻辑（由 EdgeSchema 决定）：**

| oe_strategy | ie_strategy | out_csr 类型 | in_csr 类型 |
|-------------|-------------|-------------|-------------|
| kMultiple | kMultiple | MutableCsr | MutableCsr |
| kSingle | kMultiple | SingleMutableCsr | MutableCsr |
| kMultiple | kSingle | MutableCsr | SingleMutableCsr |
| kSingle | kSingle | SingleMutableCsr | SingleMutableCsr |
| kNone | * | EmptyCsr | * |
| * | kNone | * | EmptyCsr |

**Compaction 时的 CSR 替换：**
- `dropAndCreateNewBundledCSR()`：创建新的 CSR 并替换
- `csr_alter_version_` 原子计数器追踪版本

## 13. 性能优化要点

### 13.1 内存布局优化

| 优化 | 描述 |
|------|------|
| 连续存储 | 同一顶点的边连续存放，缓存友好 |
| 紧凑邻居结构 | `ImmutableNbr` 无 timestamp 字段 |
| EmptyType 特化 | union 复用 neighbor/data 空间 |
| 位域配置 | `NbrIterConfig` 仅 4 字节 |

### 13.2 并发优化

| 优化 | 描述 |
|------|------|
| 细粒度锁 | 每顶点 SpinLock，非全局锁 |
| 原子计数器 | `edge_num_` 使用 atomic |
| 无锁读取 | NbrIterator 不获取任何锁 |
| relaxed 内存序 | edge_num 操作使用 relaxed |

### 13.3 遍历优化

| 优化 | 描述 |
|------|------|
| 排序 + 二分 | 排序后范围查询可提前终止 |
| unsorted_since_ | 区分已排序和未排序区域 |
| TypedView | 编译期类型特化，避免虚函数 |
| always_inline | 关键路径方法标记内联 |
| POD 结构 | NbrIterator/NbrList 为 POD，适合寄存器 |

### 13.4 批量操作优化

| 优化 | 描述 |
|------|------|
| batch_put_edges | 预分配 + 批量复制 |
| batch_export | 单次遍历导出所有边 |
| 预留比例 | DEFAULT_RESERVE_RATIO 减少频繁扩容 |

## 14. 典型使用场景

### 14.1 图遍历

```cpp
// 获取出边视图
GenericView view = graph.GetGenericOutgoingGraphView(src_label, dst_label, edge_label, read_ts);

// 遍历邻居
NbrList neighbors = view.get_edges(vertex_id);
for (auto it = neighbors.begin(); it != neighbors.end(); ++it) {
    vid_t neighbor_id = *it;
    const void* data_ptr = it.get_data_ptr();
}
```

### 14.2 边属性访问

```cpp
EdgeDataAccessor accessor = graph.GetEdgeDataAccessor(src_label, dst_label, edge_label, "weight");

for (auto it = neighbors.begin(); it != neighbors.end(); ++it) {
    double weight = accessor.get_typed_data<double>(it);
}
```

### 14.3 范围查询 (需要排序)

```cpp
TypedView<double, CsrViewType::kMultipleMutable> typed_view = view.get_typed_view<double, CsrViewType::kMultipleMutable>();

typed_view.foreach_nbr_gt(vertex_id, 0.5, [](vid_t nbr, double weight) {
    // 处理权重大于 0.5 的边
});
```

### 14.4 边定位 (用于更新/删除)

```cpp
auto [oe_offset, ie_offset] = record_to_csr_offset_pair(oe_view, ie_view, edge_record, props);
edge_table->DeleteEdge(src_lid, dst_lid, oe_offset, ie_offset, ts);
```
