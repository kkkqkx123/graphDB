# GraphDB CSR 设计分析与改进建议

基于 GraphScope CSR 实现的对标分析，本文档评估当前项目的 CSR 设计合理性，并提出改进方向。

---

## 1. 当前项目 CSR 现状速览

### 1.1 已实现的变体

| 变体 | 类型 | 场景 | 状态 |
|------|------|------|------|
| `MutableCsr` | 多边 | 一般性多边关系 | ✅ 完整实现 |
| `SingleMutableCsr` | 单边 | 一对一关系（spouse, current_employer） | ✅ 完整实现 |
| `CsrVariant` | 枚举包装器 | 无dyn的多态性 | ✅ 完整实现 |
| `EdgeStrategy::None` | 占位符 | 无边关系 | ✅ 完整实现 |
| `ImmutableCsr` | 不可变 | 批量加载/快照 | ❌ **缺失** |
| 标签索引 CSR | 标签分组 | 多标签图查询加速 | ❌ **缺失** |
| 多级 CSR | 压缩结构 | 大规模图压缩 | ❌ **缺失** |

**当前变体数：3（不计None）。GraphScope 约 10 种。**

### 1.2 核心设计特征

```
当前架构                          GraphScope 参考
├─ MutableCsr                    ├─ Flex/MutableCsr + ImmutableCsr
│  ├─ 两层存储（Primary+Overflow) │  ├─ 单层/双层可选
│  ├─ MVCC 时间戳支持             │  ├─ MVCC + SpinLock
│  ├─ 内部碎片化问题              │  ├─ ArenaAllocator 管理
│  └─ Compact 清理                │  └─ mmap 文件同步
├─ SingleMutableCsr             ├─ Flex/SingleMutableCsr + ImmutableSingleCsr
│  ├─ O(1) 访问                  │  ├─ O(1) 访问
│  ├─ 并发更新限制 ⚠️             │  ├─ 并发安全
│  └─ 简单直接                   └─ 带时间戳
└─ CsrVariant（枚举）    └─ dyn CsrBase trait（可选）
```

---

## 2. 优势分析

### 2.1 已有的好的设计

| 设计点 | 优势 | 参考 |
|-------|------|------|
| **EdgeStrategy 分层选择** | 编译期/运行期灵活选择合适的 CSR 变体 | ✅ GraphScope 也用 EdgeStrategy |
| **CsrVariant 枚举** | 避免 `dyn` 性能开销 | ⚠️ GraphScope 用 trait object，项目方案更高效 |
| **MVCC 时间戳** | 支持多版本并发读 | ✅ 与 Flex 引擎设计一致 |
| **Compact 机制** | 清理删除边和碎片 | ✅ 与 GraphScope 原理相同 |
| **Two-level storage** | 避免主块 O(n) 重分配 | ✅ 创新设计，高效扩容 |

### 2.2 项目相比 GraphScope 的创新

1. **CsrVariant（枚举 vs dyn trait）**
   - 当前项目：枚举 + match 分发 → 零虚函数调用开销
   - GraphScope：dyn trait → 动态分发（可能有 vtable 开销）
   - **优势**：项目方案在不需要动态加载 CSR 的场景下更高效

2. **Two-level storage（Primary + Overflow）**
   - 当前项目：扩容时追加 overflow 块，避免主块拷贝
   - GraphScope：ArenaAllocator 做延迟回收
   - **优势**：项目方案从设计上消除了扩容时的内存拷贝

---

## 3. 问题与不足

### 3.1 关键缺失功能

#### ❌ 缺失 1：ImmutableCsr（不可变 CSR）

**GraphScope 中的应用**：
- Flex 有 `ImmutableCsr` 用于批量加载和静态快照
- Interactive Engine 的 `CsrTopo` 也是不可变设计
- 性能优化：零拷贝投影、紧凑内存布局、无锁读取

**当前项目的问题**：
```rust
// 当前的MutableCsr在用作不可变快照时，仍需要维护：
// - 原子操作（AtomicU64 edge_count）
// - 锁机制（虽未实现但留了接口）
// - 扩容逻辑（虽快照不需要）

let snapshot: &MutableCsr = ...;  // ⚠️ 读快照时仍用可变数据结构
```

**改进方案**：
```rust
pub struct ImmutableCsr {
    nbr_list: Box<[Nbr]>,                    // 不可变，紧凑
    adj_offsets: Box<[u32]>,                 // 精确映射
    degrees: Box<[u32]>,                     // 无需capacity
    vertex_capacity: u32,
}

impl ImmutableCsr {
    pub fn from_mutable(mutable: &MutableCsr, ts: Timestamp) -> Self {
        // 1. Compact 清理 overflow
        // 2. 压缩到不可变格式
        // 3. 过滤无效边（ts > cutoff）
    }
    
    pub fn batch_put_edge(&mut self, src: u32, dst: VertexId, edge_id: EdgeId) {
        // 构建阶段的可变方法（仅在构建时调用）
    }
}
```

**预期收益**：
- 快照查询性能 +20% (无原子操作)
- 内存使用 -15% (无预留空间 + 更紧凑的 degrees 编码)
- 序列化/反序列化 faster path

---

#### ❌ 缺失 2：标签索引 CSR（Label-aware CSR）

**GraphScope Interactive Engine 的 Label-CSR 原理**：

```
传统 CSR：offsets[v] = start_index
        edges[start:start+degree] = 所有邻边（不分标签）

Label-CSR：offsets[v] = SortedMap {
           label_0 → (start_0, size_0),
           label_2 → (start_2, size_2),
           ...
        }
        edges 内部按标签聚组存储
```

**当前项目缺失的能力**：

```rust
// GraphScope 能做
let edges_to_label_1 = csr.get_adjacent_edges_by_label(v, label_1, Direction::Out);

// 当前项目无法做（需要遍历所有邻边过滤）
let mut edges_to_label_1 = Vec::new();
for nbr in csr.edges_of(v, ts) {
    // 需要从 property_table 查询边标签？成本高
    if get_edge_label(&nbr) == label_1 {
        edges_to_label_1.push(nbr);
    }
}
```

**适用场景**：
- 多标签图查询（如 GraphQL, GQL)
- 按边类型过滤的 traversal
- 标签条件下的图分析

**改进方案**：

```rust
pub struct LabeledMutableCsr {
    // 每个顶点的邻边按标签分组
    nbr_list: Vec<Nbr>,
    adj_offsets: Vec<RangeByLabel>,  // 新增
    degrees: Vec<u32>,
    // 其他字段同 MutableCsr
}

pub struct RangeByLabel {
    // 紧凑的标签→范围映射
    label_ranges: Vec<(EdgeLabelId, u32, u32)>,  // (label, start, size)
}

impl LabeledMutableCsr {
    pub fn edges_of_label(&self, v: u32, label: EdgeLabelId, ts: Timestamp) -> &[Nbr] {
        if let Some((start, size)) = self.get_label_range(v, label) {
            // O(log K) 查询（K = 该顶点的不同标签数）
            return &self.nbr_list[start..start+size];
        }
        &[]
    }
}
```

**预期收益**：
- 按标签过滤查询性能 +50-80% (直接定位而非遍历)
- 支持更复杂的图查询语义

---

#### ❌ 缺失 3：EdgeId 到 Nbr 的反向索引

**当前问题**：

```rust
// 要删除某条边，必须先找到它
fn delete_edge(&mut self, src_vid: u32, edge_id: EdgeId, ts: Timestamp) -> bool {
    let degree = self.degrees[src_idx] as usize;
    for i in 0..degree {
        if self.nbr_list[base + i].edge_id == edge_id {
            // 删除 → O(degree) 线性扫描
            self.mark_as_deleted(...);
            return true;
        }
    }
    // 未找到，还要扫描 overflow 块 → 最坏 O(degree + overflow_size)
    ...
}
```

GraphScope Flex 的解决方案：
```cpp
// 维护 edge_id → (offset_in_primary, offset_in_overflow) 的快速查询表
std::unordered_map<EdgeId, EdgeLocationInfo> edge_index;
```

**改进方案**：

```rust
pub struct EdgeIdIndex {
    // Option<(vertex_idx, offset_in_adj)> 用于快速定位
    edge_to_location: DashMap<EdgeId, (u32, u32)>,  // 支持并发读
}

pub struct MutableCsr {
    // 原有字段
    nbr_list: Vec<Nbr>,
    adj_offsets: Vec<u32>,
    degrees: Vec<u32>,
    
    // 新增：边 ID 快速索引
    edge_id_index: EdgeIdIndex,
}

impl MutableCsr {
    pub fn insert_edge(...) -> bool {
        // ... 原逻辑
        let offset_info = (src_idx as u32, edge_idx);
        self.edge_id_index.insert(edge_id, offset_info);
    }
    
    pub fn delete_edge(&mut self, src_vid: u32, edge_id: EdgeId, ts: Timestamp) -> bool {
        if let Some((_, (_, offset))) = self.edge_id_index.remove(&edge_id) {
            // O(1) 精确定位，而非 O(degree) 扫描
            self.mark_as_deleted(src_vid, offset, ts);
            return true;
        }
        false
    }
}
```

**预期收益**：
- 删除边性能 O(degree) → O(1)
- 支持更高效的边更新（不需要遍历）

---

### 3.2 并发安全性问题

#### ⚠️ SingleMutableCsr 的并发限制

**问题描述**（见源码注释）：

```rust
// 当前实现不支持同一时间戳的并发更新
T1: insert_edge(v0, dst=v1, ts=100) ✓ succeeds
T2: insert_edge(v0, dst=v1, ts=99)  ✗ rejected (99 < 100)
T3: insert_edge(v0, dst=v1, ts=100) ✗ rejected (100 == 100, not strictly greater)
```

**GraphScope 的解决方案**：
- Flex 的 `SingleMutableCsr` + `SpinLock` 保护，允许原子时间戳更新
- MVCC 使用原子操作 (`std::atomic<timestamp_t>`)

**改进方案**：

```rust
pub struct SingleMutableCsr {
    // 每条边的时间戳改为原子操作，允许并发更新
    nbr_list: Vec<Nbr>,                    // Nbr 内的 timestamp 改为 AtomicU64
    edge_count: AtomicU64,
    vertex_capacity: usize,
}

// 或者用锁保护单个顶点
pub struct SingleMutableCsrLocked {
    nbr_list: Vec<Nbr>,
    locks: Vec<parking_lot::Mutex<()>>,   // 每个顶点一把轻量级锁
    vertex_capacity: usize,
}
```

**预期收益**：
- 消除当前的并发更新限制
- 支持真实的多线程 ACID 事务

---

### 3.3 内存碎片化问题

#### ⚠️ MutableCsr 的 Overflow Fragmentation

**当前设计**：
- Primary 满时，追加 overflow 块到 `nbr_list` 末尾
- 扩容时，新数据追加到末尾，旧 overflow 块仍占空间

**问题量化**：

```
初始：nbr_list = [v0_primary (8) | v1_primary (8) | ...]  capacity=1024

添加 100 条边到 v0：
  1. v0_overflow_1 = [edges 1-8] → nbr_list末尾 append
  2. v0_overflow_2 = [edges 9-16] → nbr_list末尾 append（旧 overflow_1 变成"僵尸块"）
  3. v0_overflow_3 = [edges 17-24] → nbr_list末尾 append（overflow_2 也变僵尸）
  ...
  
最终：nbr_list 中有 O(log degree) 个"僵尸块"无法被回收，直到 compact()
```

**GraphScope 的解决方案**：
- `ArenaAllocator` 跟踪各时期分配的内存，在所有读线程释放后统一回收
- MVCC snapshot 确保旧块不被并发读者访问

**当前项目已有缓解方案**：
- `compact_with_ts()` 方法可清理碎片
- 文档明确说明何时调用

**改进方向**：

```rust
pub struct MutableCsr {
    nbr_list: Vec<Nbr>,
    // ... 其他字段
    
    // 新增：碎片化统计
    fragmentation_metadata: FragmentationStats,
}

pub struct FragmentationStats {
    zombie_blocks: usize,          // 无法到达的块数量
    wasted_capacity: usize,        // 浪费的总容量
    total_capacity: usize,         // 整体容量
}

impl MutableCsr {
    pub fn fragmentation_ratio(&self) -> f32 {
        self.fragmentation_metadata.wasted_capacity as f32 
            / self.fragmentation_metadata.total_capacity as f32
    }
    
    // 现有方法名改进：清晰表示这是 offline compaction
    pub fn compact_offline(&mut self, ts: Timestamp, reserve_ratio: f32) -> CompactionReport {
        CompactionReport {
            removed_edges: usize,
            reclaimed_bytes: usize,
            new_fragmentation_ratio: f32,
        }
    }
}
```

**预期收益**：
- 提供可观测的碎片化指标
- 支持自动 compact 触发策略

---

## 4. 设计模式对比

### 4.1 Trait 多态 vs Enum 多态

| 维度 | Trait Object (`dyn`) | Enum 多态 |
|------|-------|---------|
| **虚函数开销** | ✅ 单一 vtable | ❌ match 分发（通常还是内联） |
| **扩展性** | ✅ 编译后新增 impl | ❌ 需修改 enum 定义和 match 分支 |
| **类型安全** | ⚠️ 动态检查 | ✅ 编译期确定 |
| **内存布局** | ⚠️ 胖指针（data + vtable） | ✅ 单一枚举值 |
| **当前项目** | ❌ 未用 | ✅ CsrVariant |

**评价**：当前项目的 enum 方案在性能上更优，但牺牲了扩展性。如果未来可能增加新的 CSR 变体（如 `LabeledMutableCsr`），可考虑：

```rust
// 方案 A：继续扩展 enum（当前）
pub enum CsrVariant {
    Multiple(MutableCsr),
    Single(SingleMutableCsr),
    Labeled(LabeledMutableCsr),  // 新增
    None { vertex_capacity: usize },
}

// 方案 B：加入 trait object（更灵活）
pub type MutableCsrDyn = Box<dyn MutableCsrTrait>;

// 方案 C：泛型包装（编译期多态）
pub struct MutableCsrStore<T: MutableCsrTrait> {
    inner: T,
}
```

---

### 4.2 时间戳管理模式对比

| 项 | 当前项目 | GraphScope Flex | 交互引擎 |
|----|--------|-----------------|---------|
| **时间戳形式** | u64（从核心库）| `timestamp_t` | 序列化后（无MVCC）|
| **MVCC 支持** | ✅ 时间戳过滤 | ✅ 原子时间戳 | ❌ 无（batch 式）|
| **删除策略** | ❌ 物理删除（改 timestamp） | ✅ 原子删除标记 | ❌ 无 |
| **版本链** | ❌ 无（仅一份数据） | ✅ 历史版本保留 | ❌ 无 |

**当前项目的 MVCC 实现**：
```rust
// 删除：只是改时间戳为 INVALID_TIMESTAMP
pub fn delete_edge(&mut self, ...) -> bool {
    self.nbr_list[...].timestamp = INVALID_TIMESTAMP;
    // 问题：这破坏了原有时间戳信息，无法恢复删除前的状态
}

// 读：时间戳过滤
pub fn edges_of(&self, src_vid: u32, ts: Timestamp) -> Vec<Nbr> {
    result.retain(|nbr| nbr.timestamp <= ts);  // 简单过滤
}
```

**改进方向**（参考 Flex 设计）：

```rust
pub struct Nbr {
    pub neighbor: VertexId,
    pub edge_id: EdgeId,
    pub prop_offset: u32,
    pub create_ts: Timestamp,       // 新增：创建时间戳
    pub delete_ts: Timestamp,       // 新增：删除时间戳（u64::MAX = 未删除）
}

// 删除仅更新 delete_ts，不破坏 create_ts
pub fn delete_edge(&mut self, ..., ts: Timestamp) -> bool {
    self.nbr_list[...].delete_ts = ts;
    true
}

// 读：检查 create_ts <= query_ts < delete_ts
pub fn edges_of(&self, src_vid: u32, ts: Timestamp) -> Vec<Nbr> {
    result.retain(|nbr| nbr.create_ts <= ts && ts < nbr.delete_ts);
}

// 支持撤销删除（revert_delete）
pub fn revert_delete(&mut self, ..., ts: Timestamp) -> bool {
    // 只要改回 delete_ts = u64::MAX
    if self.nbr_list[...].delete_ts < u64::MAX {
        self.nbr_list[...].delete_ts = u64::MAX;
        return true;
    }
    false
}
```

**预期收益**：
- 支持 revert_delete（已有接口但实现有问题）
- 完整的事务回滚语义

---

## 5. 优先级改进建议

### 第一阶段（关键，1-2周）

1. **ImmutableCsr 实现** ⭐⭐⭐⭐⭐
   - 快照性能优化
   - 序列化路径加速
   - 批量加载优化

2. **SingleMutableCsr 并发修复** ⭐⭐⭐⭐
   - 消除当前并发限制
   - 需改动 `Nbr` 结构和时间戳管理

3. **MVCC 时间戳完善** ⭐⭐⭐⭐
   - 分离 create_ts / delete_ts
   - 修复 revert_delete 语义

### 第二阶段（增强功能，2-4周）

4. **EdgeId 反向索引** ⭐⭐⭐
   - O(1) 边定位
   - 支持高性能边删除

5. **碎片化可观测性** ⭐⭐⭐
   - 暴露 fragmentation_ratio
   - 支持自动 compact 策略

6. **性能监控指标** ⭐⭐
   - 追踪缓存行为
   - 内存使用统计

### 第三阶段（高级功能，4-8周）

7. **LabeledMutableCsr** ⭐⭐⭐
   - 支持多标签图查询
   - 按标签快速过滤

8. **MultiSingleMutableCsr** ⭐⭐
   - 单源多目标（比如 one-to-many）
   - 扩展 Single 语义

9. **多级 CSR（MCSR/BMCSR）** ⭐⭐
   - 大规模图压缩存储
   - 分区支持

---

## 6. 实现路线图示例

### ImmutableCsr 快速原型（1周）

```rust
// Step 1: 定义结构
pub struct ImmutableCsr {
    nbr_list: Box<[Nbr]>,
    adj_offsets: Box<[u32]>,
    degrees: Box<[u32]>,
}

// Step 2: 从 MutableCsr 转换
impl From<&MutableCsr> for ImmutableCsr {
    fn from(mutable: &MutableCsr) -> Self {
        // compact + 转换逻辑
    }
}

// Step 3: 实现 CsrBase trait
impl CsrBase for ImmutableCsr { ... }

// Step 4: 加入 CsrVariant
pub enum CsrVariant {
    Multiple(MutableCsr),
    Single(SingleMutableCsr),
    Immutable(ImmutableCsr),  // 新增
    None { vertex_capacity: usize },
}
```

### 并发修复（1周）

```rust
// 改 Nbr 结构
pub struct Nbr {
    pub neighbor: VertexId,
    pub edge_id: EdgeId,
    pub prop_offset: u32,
    pub create_ts: Timestamp,
    pub delete_ts: Timestamp,  // 新增
}

// 修改 SingleMutableCsr 逻辑
impl SingleMutableCsr {
    pub fn insert_edge(&mut self, ..., ts: Timestamp) -> bool {
        // 检查 delete_ts，允许覆盖已删除的边
        if self.nbr_list[src_idx].delete_ts <= ts {
            self.nbr_list[src_idx] = Nbr { create_ts: ts, delete_ts: u64::MAX, ... };
            return true;
        }
        false  // 边仍活跃，拒绝
    }
}
```

---

## 7. 总体评价与结论

### 优点（相比 GraphScope）

✅ **Enum 枚举多态** — 零虚函数开销  
✅ **Two-level overflow** — 创新的扩容设计  
✅ **Compact 机制** — 清晰的碎片化处理  
✅ **代码简洁** — 逻辑清晰，易维护  

### 不足与缺失

❌ **无 ImmutableCsr** — 快照查询无法优化  
❌ **无标签索引** — 多标签图查询性能差  
❌ **并发安全性** — SingleMutableCsr 并发限制  
❌ **时间戳管理** — 删除时不保留历史  
❌ **无边 ID 索引** — 边定位成本高  

### 总体建议

| 优先级 | 方向 | 预期影响 |
|-------|------|---------|
| **立即** | ImmutableCsr + 并发修复 | +20% 快照性能，ACID完整性 |
| **1 月内** | 时间戳完善 + 边索引 | +30% 边操作性能，事务回滚 |
| **3 月内** | LabeledMutableCsr | +50-80% 多标签查询性能 |
| **6 月内** | 多级 CSR 探索 | 压缩比 +2-5x |

---

## 附录：文件清单

### 现有文件
- `src/storage/edge/mutable_csr.rs` — 主 CSR 实现（2.8KB 有效代码）
- `src/storage/edge/single_mutable_csr.rs` — 单边 CSR（1.5KB）
- `src/storage/edge/mutable_csr_variant.rs` — 枚举包装（0.4KB）
- `src/storage/edge/csr_trait.rs` — trait 定义（0.1KB）

### 建议新增文件
- `immutable_csr.rs` — 不可变 CSR
- `labeled_mutable_csr.rs` — 标签索引 CSR（可选，第三阶段）
- `edge_id_index.rs` — 边 ID 反向索引
- `csr_tests/` — 集中测试目录

