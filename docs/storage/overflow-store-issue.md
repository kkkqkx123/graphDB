## OverflowStore 实现分析

这是一个**针对大值（>256字节）的独立溢出存储**，与主列式存储并行。让我深入分析其合理性。

---

## 一、当前设计分析

### 1.1 核心机制

```rust
pub struct OverflowStore {
    data_pool: Vec<u8>,                          // 连续内存池
    index: HashMap<u64, (u64, u32)>,             // id → (offset, size)
    location_index: HashMap<OverflowKey, u64>,   // (col, row) → id
    free_list: Vec<(u64, u32)>,                  // 空闲块 (最佳适配)
    next_id: u64,
    entry_count: usize,
}
```

**工作流程**：
1. 值 > 256 字节时存入 OverflowStore
2. 主列存储只存一个 `prop_offset` (u32)
3. 读取时通过 `(col_idx, row_idx)` 查找大值
4. 删除时回收空间到 free_list

### 1.2 设计特点

| 特点 | 评价 |
|------|------|
| **最佳适配分配** | ✅ 减少碎片，但 O(n) 查找 |
| **空闲块合并** | ✅ 相邻块自动合并 |
| **独立序列化** | ✅ 与主存储分离，便于管理 |
| **CRC校验** | ✅ 数据完整性保护 |
| **列索引重映射** | ✅ 支持 DROP COLUMN |

---

## 二、存在的问题

### 2.1 ⚠️ 严重的碎片问题

```rust
// 场景：不断插入和删除大值
insert("very_long_string_1")  // 分配 300 字节
delete(very_long_string_1)    // 释放 300 字节到 free_list
insert("very_long_string_2")  // 分配 280 字节 → 从 free_list 取 300，留下 20 字节碎片
insert("very_long_string_3")  // 分配 290 字节 → 从 free_list 取 300，留下 10 字节碎片
// 经过多次操作，产生大量微小碎片，无法利用
```

**后果**：
- 内存利用率下降
- 碎片过多导致分配失败，被迫扩容
- 无法 compaction

### 2.2 ⚠️ O(n) 查找开销

```rust
fn allocate_space(&mut self, needed_size: u32) -> (u64, u32) {
    let mut best_idx = None;
    let mut best_size = u32::MAX;
    
    // O(n) 扫描整个 free_list
    for (i, &(_offset, size)) in self.free_list.iter().enumerate() {
        if size >= needed_size && size < best_size {
            best_idx = Some(i);
            best_size = size;
        }
    }
    // ...
}
```

**问题**：频繁的大值操作会导致性能下降

### 2.3 ⚠️ 双重索引开销

```rust
// 查找一个值需要两次哈希查找
fn retrieve(&self, col_idx: usize, row_idx: usize) -> Option<Value> {
    let overflow_id = self.location_index.get(&key)?;  // 第1次查找
    let &(offset, size) = self.index.get(overflow_id)?; // 第2次查找
    // ...
}
```

**问题**：内存和 CPU 开销增加

### 2.4 ⚠️ 无压缩支持

```rust
// 大值直接存储，没有压缩
self.data_pool[offset..end].copy_from_slice(&bytes);
```

**问题**：大字符串、JSON、BLOB 浪费空间

### 2.5 ⚠️ 查询效率低

```rust
// 过滤大值列时，无法利用列式存储的优势
// 需要回表查询 OverflowStore
for row in rows {
    let value = overflow_store.retrieve(col_idx, row_idx);  // 随机访问
    if filter(value) { ... }
}
```

**问题**：大值列的谓词下推失效

---

## 三、与列式存储方案对比

### 3.1 LadybugDB 的方案：分裂为两列

```cpp
// 字符串列存储为两列
class StringColumn {
    Column offset_column;  // 定长 INT，存储每个字符串的起始位置
    Column data_column;    // 变长 CHAR，存储所有字符串数据
};

// 写入
void insert(const std::string& value) {
    offset_column.append(data_column.size());
    data_column.append(value.data(), value.size());
}

// 读取
std::string get(row) {
    size_t start = offset_column.get(row);
    size_t end = offset_column.get(row + 1);
    return data_column.get_range(start, end);
}
```

**优势**：
- 统一存储，无需特殊处理
- 支持列式压缩（前缀压缩、字典压缩）
- 支持向量化读取
- 支持谓词下推（如 `WHERE name LIKE 'A%'`）

### 3.2 对比分析

| 维度 | **OverflowStore** | **列式分裂存储** |
|------|------------------|-----------------|
| **存储位置** | 独立存储池 | 主列存储 |
| **访问模式** | 随机访问 (哈希查找) | 顺序访问 (列式) |
| **压缩支持** | ❌ 无 | ✅ 字典/前缀/RLE |
| **谓词下推** | ❌ 失效 | ✅ 支持 |
| **向量化** | ❌ 不支持 | ✅ 支持 |
| **碎片问题** | ⚠️ 严重 | ✅ 无碎片 (连续追加) |
| **内存开销** | ⚠️ 双重索引 | ✅ 只需存储数据 |
| **删除回收** | ⚠️ 需管理 free_list | ✅ 标记删除，定期清理 |
| **实现复杂度** | 🔴 高 | 🟢 低 |

---

## 四、迁移到列式存储的方案

### 4.1 方案一：分裂为 OFFSET + DATA 列

```rust
// 类似于 LadybugDB
pub struct StringColumn {
    offset_column: Column,  // 存储每个值的起始位置
    data_column: ByteColumn, // 存储所有值的连续数据
    null_bitmap: Bitmap,     // NULL 标记
}

impl StringColumn {
    pub fn insert(&mut self, value: Option<&str>) {
        match value {
            Some(v) => {
                let start = self.data_column.len();
                self.data_column.append(v.as_bytes());
                self.offset_column.append(start);
                self.null_bitmap.set(false);
            }
            None => {
                self.offset_column.append(self.data_column.len());
                self.null_bitmap.set(true);
            }
        }
    }
    
    pub fn get(&self, row: usize) -> Option<&str> {
        if self.null_bitmap.get(row) {
            return None;
        }
        let start = self.offset_column.get(row);
        let end = self.offset_column.get(row + 1);
        Some(self.data_column.get_range(start, end))
    }
}
```

**优势**：
- 统一存储，无需 OverflowStore
- 支持字典压缩（重复字符串）
- 支持前缀压缩
- 向量化读取

### 4.2 方案二：FSST 压缩

```rust
// 使用 FSST (Fast Static Symbol Table) 压缩字符串
pub struct FsstColumn {
    symbol_table: Vec<u8>,     // 符号表
    compressed_data: Column,   // 压缩后的符号序列
    offset_column: Column,     // 每个值的符号偏移
}

// FSST 压缩率可达 2x-5x，适合字符串列
```

### 4.3 方案三：字典压缩

```rust
pub struct DictionaryColumn {
    dictionary: Vec<String>,   // 字典
    indices: Column,           // 每个值的字典索引
    // 适合低基数字符串列 (如国家、城市)
}
```

---

## 五、迁移建议

### 5.1 短期方案（兼容性优先）

保留 OverflowStore，但做以下改进：

```rust
// 1. 使用更高效的分配器
use buddy_system_allocator::Heap;  // 伙伴系统，减少碎片

// 2. 定期 compaction
pub fn compact(&mut self) {
    // 重新分配所有活动值，消除碎片
    let mut new_pool = Vec::new();
    for ((col, row), id) in &self.location_index {
        let value = self.retrieve(*col, *row).unwrap();
        let offset = new_pool.len();
        new_pool.extend_from_slice(&value.to_bytes());
        self.index.insert(*id, (offset, value.size));
    }
    self.data_pool = new_pool;
    self.free_list.clear();
}

// 3. LRU 缓存热点大值
pub struct CachedOverflowStore {
    store: OverflowStore,
    cache: LruCache<OverflowKey, Value>,  // 缓存热点大值
}
```

### 5.2 长期方案（迁移到列式）

```rust
// 逐步迁移：在 PropertyTable 中增加大值列支持
pub struct PropertyTable {
    // 原有列存储 (小值)
    columns: Vec<Column>,
    
    // 新增：大值列 (使用 OFFSET + DATA)
    large_value_columns: Vec<LargeValueColumn>,
}

// 写入时自动判断
pub fn insert(&mut self, values: &[(String, Value)]) -> StorageResult<u32> {
    for (name, value) in values {
        if value.to_bytes().len() > OVERFLOW_THRESHOLD {
            // 使用大值列存储
            self.large_value_columns[col_idx].insert(value);
        } else {
            // 使用普通列存储
            self.columns[col_idx].insert(value);
        }
    }
}
```

### 5.3 最终方案（统一列式）

```rust
// 完全去掉 OverflowStore，所有值都存列式
pub struct PropertyTable {
    columns: Vec<ColumnVariant>,  // 每种列类型独立实现
}

pub enum ColumnVariant {
    Small(Column),          // 小值列
    String(StringColumn),   // 字符串列 (OFFSET+DATA)
    Json(JsonColumn),       // JSON 列
    Blob(BlobColumn),       // BLOB 列
}
```

---

## 六、结论与建议

### 6.1 OverflowStore 是否合理？

| 方面 | 结论 |
|------|------|
| **设计质量** | ⚠️ 合理但存在缺陷 |
| **是否可用** | ✅ 可用，小规模场景 OK |
| **可维护性** | ⚠️ 复杂，需额外维护 |
| **性能** | ⚠️ 碎片 + O(n) 分配 |
| **扩展性** | ❌ 不利于列式优化 |

### 6.2 迁移建议

```
优先级排序：

1. 🔴 高优先级：添加 compaction
   → 解决碎片问题，避免内存膨胀

2. 🟡 中优先级：优化分配策略
   → 使用伙伴系统替代最佳适配

3. 🟢 低优先级：迁移到列式存储
   → 长期方案，统一存储架构
```

### 6.3 最终判断

**OverflowStore 作为一个独立的溢出存储，在以下场景合理**：
- 小规模图数据库（< 100万边）
- 大值占比低（< 5%）
- 查询模式以点查为主

**但在以下场景应迁移到列式**：
- 大规模图数据库（> 1亿边）
- 大值列频繁查询
- 需要谓词下推和向量化
- 追求统一存储架构

**我的建议**：采用渐进式迁移
1. 第一阶段：保留 OverflowStore，增加 compaction
2. 第二阶段：对大值列增加列式存储支持
3. 第三阶段：完全替换为列式存储