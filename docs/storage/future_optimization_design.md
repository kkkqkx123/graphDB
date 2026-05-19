## 三、MutableCsr 扩容性能优化

### 3.1 问题分析

当前 `MutableCsr.expand_vertex_capacity()` 使用 `Vec::splice()` 在 `nbr_list` 中间插入空位：

```rust
fn expand_vertex_capacity(&mut self, src_idx: usize) {
    self.nbr_list.splice(
        insert_pos..insert_pos,
        std::iter::repeat_n(empty_nbr, additional),
    );
    // splice 导致插入位置之后所有元素移位，O(n) 复杂度
    for i in (src_idx + 1)..self.vertex_capacity {
        self.adj_offsets[i] += additional;
    }
}
```

**性能瓶颈：**

- 每次扩容都需要移位后续所有顶点的边，O(total_remaining_edges)
- 对于低编号顶点（如 vertex 0）频繁扩容的场景，每次移位代价随全图边数增长
- 增长因子 1.5x 导致扩容次数较多（O(log₁.₅ n)）
- 无 batched expansion，`batch_insert_parallel` 中每个顶点独立触发 splice

### 3.2 关键约束：为何不能随意改变存储结构

现有架构的核心依赖：

| 组件                                            | 对当前 CSR 的依赖                 | 影响程度 |
| ----------------------------------------------- | --------------------------------- | -------- |
| `MutableCsrIterator` / `MutableCsrEdgeIterator` | 顺序扫描 `nbr_list`               | 高       |
| `edges_of_with_prefetch`                        | 使用 x86 prefetch 预取连续地址    | 必须保留 |
| `batch_insert_parallel`                         | unsafe pointer write 到非重叠区域 | 高       |
| `dump` / `load`                                 | 平铺数组直接序列化                | 高       |
| `compact_with_ts`                               | 重建整个 `nbr_list`               | 中       |

**`Vec<Vec<Nbr>>`（原方案 A）的主要问题是：**

- 全图遍历退化为跨 Vec 跳转，TLB miss 和 cache miss 显著增加
- `batch_insert_parallel` 的 unsafe 指针写入不适用于独立 Vec
- `edges_of_with_prefetch` 无法跨 Vec 预取
- 序列化复杂度增加，需要逐个编码子 Vec
- 每个空顶点多 24+ 字节堆开销

**Slot 池 + 指针链（原方案 B）的问题是：**

- 链表遍历的本质是 pointer chasing，对图遍历场景几乎必定更慢
- `compact` 复杂度极高
- `dump` / `load` 需要处理复杂的有向图结构

### 3.3 新方案：双层 CSR（Two-Level CSR）

#### 核心思路

保持 `nbr_list` 的连续存储不变，但当顶点容量耗尽时，**不在中间 splice，而是在末尾 append overflow 块**：

```
当前 splice 方案：
[v0_e0, v0_e1, v0_e2, v0_e3, ___, ___, ___ | v1_e0, v1_e1 | ... ]
                              ↑ splice 插入3个空位，后续全部移位

双层 CSR 方案：
[v0_e0, v0_e1, v0_e2, v0_e3 | v1_e0, v1_e1 | ... | ___, ___, ___ ]
                                                    ↑ overflow append
```

**优势**：append 是 O(1) 摊销（`Vec::resize` 可能触发整体扩容，但远少于每次 expand），且不改变任何已有元素的位置，无需更新后续顶点的 `adj_offsets`。

#### 数据结构

```rust
pub struct MutableCsr {
    nbr_list: Vec<Nbr>,
    adj_offsets: Vec<usize>,       // primary 块起始
    degrees: Vec<u32>,              // primary 块活跃边数
    primary_capacities: Vec<u32>,   // primary 块保留容量

    overflow_starts: Vec<usize>,    // overflow 块起始（nbr_list 中位置，NO_OVERFLOW = usize::MAX）
    overflow_counts: Vec<u32>,      // overflow 块活跃边数

    edge_count: AtomicU64,
    vertex_capacity: usize,
    total_edge_capacity: usize,
}
```

每个顶点至多一块连续 overflow。`NO_OVERFLOW` 标记表示无 overflow。

#### 关键操作

##### expand_vertex_capacity — O(1) append

```rust
fn expand_vertex_capacity(&mut self, src_idx: usize) {
    let old_cap = self.primary_capacities[src_idx] as usize;
    let new_cap = (old_cap * 2).max(4);
    let additional = new_cap - old_cap;

    // append 到末尾，O(1) 摊销
    let append_pos = self.nbr_list.len();
    self.nbr_list.resize(
        append_pos + additional,
        Nbr::default_empty(),
    );

    self.overflow_starts[src_idx] = append_pos;
    self.primary_capacities[src_idx] = new_cap as u32;
    self.total_edge_capacity += additional;
    // 无需更新 adj_offsets，无需 splice
}
```

**当前对应实现**（mutable_csr.rs:208-229）需要 splice + 循环更新所有后续 adj_offsets，O(n)。

##### insert_edge — 两路写入

```rust
fn insert_edge(&mut self, src: VertexId, dst: VertexId, ...) -> bool {
    let src_idx = src.as_int64().unwrap_or(0) as usize;

    if src_idx >= self.vertex_capacity {
        self.ensure_vertex_capacity(src_idx + 1);
    }

    // duplicate check（O(degree)，不可跳过，但和当前一致）
    let degree = self.degrees[src_idx] as usize;
    let offset = self.adj_offsets[src_idx];
    for i in 0..degree {
        let nbr = &self.nbr_list[offset + i];
        if nbr.neighbor == dst && nbr.timestamp != INVALID_TIMESTAMP {
            return false;
        }
    }
    if self.overflow_starts[src_idx] != usize::MAX {
        let o_start = self.overflow_starts[src_idx];
        let o_count = self.overflow_counts[src_idx] as usize;
        for i in 0..o_count {
            let nbr = &self.nbr_list[o_start + i];
            if nbr.neighbor == dst && nbr.timestamp != INVALID_TIMESTAMP {
                return false;
            }
        }
    }

    // 优先写入 primary
    if (degree as u32) < self.primary_capacities[src_idx] {
        self.nbr_list[offset + degree] = Nbr::new(dst, edge_id, prop_offset, ts);
        self.degrees[src_idx] += 1;
    } else {
        // overflow
        if self.overflow_starts[src_idx] == usize::MAX {
            self.expand_vertex_capacity(src_idx);
        }
        let o_start = self.overflow_starts[src_idx];
        let o_count = self.overflow_counts[src_idx] as usize;
        self.nbr_list[o_start + o_count] = Nbr::new(dst, edge_id, prop_offset, ts);
        self.overflow_counts[src_idx] += 1;
    }

    self.edge_count.fetch_add(1, Ordering::Relaxed);
    true
}
```

注意：overflow 也可持续追加。若 overflow 块自身位于 `nbr_list` 末尾，`overflow_counts[src_idx]` 递增即可——`nbr_list` 后续位置已被之前 `expand_vertex_capacity` 的 `resize` 预留。

##### edges_of / 迭代器 — 扫描 primary + overflow

```rust
fn edges_of(&self, src: VertexId, ts: Timestamp) -> Vec<Nbr> {
    let src_idx = src.as_int64().unwrap_or(0) as usize;
    if src_idx >= self.vertex_capacity { return Vec::new(); }

    let offset = self.adj_offsets[src_idx];
    let degree = self.degrees[src_idx] as usize;

    let mut result = Vec::new();
    for i in 0..degree {
        let nbr = &self.nbr_list[offset + i];
        if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
            result.push(*nbr);
        }
    }

    let o_start = self.overflow_starts[src_idx];
    if o_start != usize::MAX {
        let o_count = self.overflow_counts[src_idx] as usize;
        for i in 0..o_count {
            let nbr = &self.nbr_list[o_start + i];
            if nbr.timestamp <= ts && nbr.timestamp != INVALID_TIMESTAMP {
                result.push(*nbr);
            }
        }
    }
    result
}
```

`MutableCsrEdgeIterator` 增加一个 overflow 扫描阶段：

```rust
impl<'a> Iterator for MutableCsrEdgeIterator<'a> {
    type Item = Nbr;

    fn next(&mut self) -> Option<Self::Item> {
        // Phase 1: primary
        while self.pos < self.degree {
            let nbr = self.csr.nbr_list[self.offset + self.pos];
            self.pos += 1;
            if nbr.timestamp <= self.ts && nbr.timestamp != INVALID_TIMESTAMP {
                return Some(nbr);
            }
        }
        // Phase 2: overflow (如有)
        if let Some(o_pos) = self.overflow_scan.as_mut() {
            while o_pos.idx < o_pos.count {
                let nbr = self.csr.nbr_list[o_pos.start + o_pos.idx];
                o_pos.idx += 1;
                if nbr.timestamp <= self.ts && nbr.timestamp != INVALID_TIMESTAMP {
                    return Some(nbr);
                }
            }
        }
        None
    }
}
```

##### compact — 合并为单层 CSR

```rust
fn compact(&mut self) {
    // 遍历所有顶点，计算出每个顶点的总活跃边数和有效偏移
    // 在 nbr_list 中从前往后原地压缩，重建 adj_offsets
    // 清空 overflow_starts / overflow_counts
    // compact 后回归纯平 CSR 结构，遍历性能无损失
}
```

`compact_with_ts` 同理：重建 `nbr_list`，忽略 overflow 标记。

##### batch_insert_parallel — 保持兼容

Phase 1 预先为每个顶点展开 capacity（此时会生成 overflow 块）。Phase 2 的 unsafe 指针写入沿用原有逻辑——优先级：先填满 primary，再写入 overflow 区域。不同顶点的 overflow 块在 `nbr_list` 末尾的非重叠区域中，安全。

### 3.4 额外可确定的优化

#### 优化 1：Growth factor 1.5x → 2x

变更 `mutable_csr.rs:211`：

```rust
// 当前
let new_capacity = ((old_capacity as f64) * 1.5).max(4.0) as usize;
// 改为
let new_capacity = (old_capacity * 2).max(4);
```

效果：扩容次数从 O(log₁.₅ n) 降到 O(log₂ n)。对于 100 万条边/顶点的场景，从约 28 次降到约 19 次。代价是稳态内存开销从 33% 增到 50%，无代码复杂度增加。

#### 优化 2：修复 `batch_put_edges` 静默丢边

当前 `batch_put_edges`（mutable_csr.rs:604-608）在 capacity 满时静默丢弃边。双层 CSR 下，多余边直接写入 overflow：

```rust
if degree < capacity {
    // 写入 primary
} else {
    // 写入 overflow（无丢边）
}
```

#### 优化 3：`edges_of_with_prefetch` 保留

overflow 块与 primary 块相同，也是连续内存区域，prefetch 逻辑可直接扩展覆盖 overflow 范围的预取。

### 3.5 对现有代码的影响总览

| 组件                                            | 变更程度 | 说明                                         |
| ----------------------------------------------- | -------- | -------------------------------------------- |
| `insert_edge`                                   | 小改     | duplicate check 扩展到 overflow；写入分两路  |
| `expand_vertex_capacity`                        | 重写     | splice → append；无需更新后续 offset         |
| `delete_edge*` / `revert*`                      | 小改     | 搜索范围从 primary 扩展到 primary + overflow |
| `edges_of` / `degree` / `has_edge` / `get_edge` | 中改     | 所有读方法需要检查 overflow block            |
| `MutableCsrIterator` / `MutableCsrEdgeIterator` | 中改     | 迭代逻辑追加 overflow 扫描阶段               |
| `batch_insert_parallel`                         | 小改     | Phase 2 写入 primary/overflow 两区域         |
| `batch_put_edges`                               | 小改     | 修复静默丢边，写入 overflow                  |
| `compact` / `compact_with_ts`                   | 重写     | 合并 overflow 回 primary 并重建              |
| `dump` / `load`                                 | 中改     | 增加 overflow_starts / overflow_counts 字段  |
| `dump` 版本号                                   | 小改     | 需要添加格式版本标识以兼容旧数据             |
| `clear`                                         | 小改     | 额外清空 overflow 数组                       |
| `memory_size` / `used_memory_size`              | 小改     | 计入 overflow 数组                           |

### 3.6 与原方案对比

| 对比维度                 | 方案 A (`Vec<Vec<Nbr>>`) | 方案 B (Slot 池) | **本方案 (双层 CSR)** |
| ------------------------ | ------------------------ | ---------------- | --------------------- |
| 扩容复杂度               | O(1) 摊销                | O(1) 摊销        | **O(1) 摊销**         |
| 顶点遍历                 | 跨 Vec 指针追踪          | pointer chasing  | **连续块扫描**        |
| 全图遍历                 | 大量 TLB miss            | 大量 cache miss  | **顺序 + 少量跳转**   |
| `edges_of_with_prefetch` | 不支持                   | 不支持           | **保留**              |
| `batch_insert_parallel`  | 需重写                   | 极复杂           | **兼容**              |
| dump/load 复杂度         | 编码 N 个 Vec            | 序列化链表       | **平铺 + 2 数组**     |
| compact 复杂度           | 逐个 Vec 压缩            | 重建链表         | **合并重建**          |
| 额外内存/顶点            | 24+24=48 字节            | 8-16 字节        | **8+4=12 字节**       |
| 实现复杂度               | 低                       | 极高             | **中**                |

### 3.7 实现步骤

1. **扩展 `LoadFromPartsParams`**
   - 增加 `overflow_starts` / `overflow_counts`
   - 添加格式版本号

2. **实现新 `expand_vertex_capacity`**
   - 删除 splice 逻辑
   - 改为 append + 记录位置

3. **更新所有读/写/删方法**
   - `insert_edge`、`delete_edge*`、`revert*`
   - `edges_of`、`degree`、`has_edge`、`get_edge`

4. **更新迭代器**
   - `MutableCsrIterator`、`MutableCsrEdgeIterator`

5. **实现 `compact` / `compact_with_ts`**
   - 合并 overflow 回 primary
   - 重建平坦 CSR

6. **更新 `dump` / `load`**
   - 序列化 overflow 数组
   - 兼容旧格式或升级版本

7. **调整 growth factor 为 2x**

8. **修复 `batch_put_edges` 丢边问题**

### 3.8 工作量估计

- 数据结构变更 + expand 重写：1 天
- 读方法适配（edges_of, has_edge, get_edge, degree）：0.5 天
- 迭代器适配：0.5 天
- compact 重写：0.5 天
- dump/load + 序列化兼容：0.5 天
- 测试覆盖 + 验证：0.5 天
- 总计：3.5 天

---

## 四、实施优先级建议

| 优先级 | 任务                      | 风险 | 收益                 | 建议           |
| ------ | ------------------------- | ---- | -------------------- | -------------- |
| 1      | VertexIndexManager 泛型化 | 中   | 消除 500+ 行重复代码 | 优先实施       |
| 2      | Column 变长/定长拆分      | 中   | 提升列存性能         | 次优先         |
| 3      | **MutableCsr 双层 CSR**   | 中   | 高频写入性能         | 实现复杂度较高 |

### 关于 MutableCsr 双层 CSR 的风险说明

与原先标注为"高风险"的 design doc 不同，双层 CSR 的风险等级应为**中**：

- **降低风险的因素**：保留连续存储核心优势，不改变遍历和序列化的基本模式，`batch_insert_parallel` 兼容性好
- **需要关注的**：`dump` 格式版本需兼容旧数据；edges_of 等读方法多一个 overflow 分支（branch 预测影响未知，但至多一个 taken/not-taken 分支）

建议先实现原型，通过 `cargo test --lib -- mutable_csr` 验证语义正确性。
