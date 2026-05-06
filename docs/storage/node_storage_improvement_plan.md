# GraphDB 节点存储优化方案

## 一、现状问题总结

基于对现有代码的分析和业界数据库调研，当前节点存储存在以下主要问题：

| 问题                      | 影响          | 优先级 |
| ------------------------- | ------------- | ------ |
| NULL 位图使用 `Vec<bool>` | 内存浪费 8 倍 | 高     |
| 字符串无压缩              | 存储空间大    | 高     |
| ID 映射固定容量           | 扩展性差      | 中     |
| 删除产生空洞              | 内存碎片      | 中     |
| 无批量操作优化            | 写入性能低    | 低     |

---

## 二、优化方案

### 2.1 NULL 位图优化

#### 问题分析

当前实现：

```rust
// column_store.rs
pub struct Column {
    null_bitmap: Option<Vec<bool>>,  // 每个 NULL 标记占 1 byte
}
```

参考 DuckDB/Arrow 的设计，应使用位图：

```rust
// 每个 NULL 标记仅占 1 bit
null_bitmap: Option<BitVec>,
```

#### 内存节省计算

| 场景               | 当前内存 | 优化后内存 | 节省  |
| ------------------ | -------- | ---------- | ----- |
| 100万行，50% NULL  | 1 MB     | 125 KB     | 87.5% |
| 1000万行，10% NULL | 10 MB    | 1.25 MB    | 87.5% |

#### 实现方案

```rust
use std::bitvec::prelude::*;

pub struct Column {
    name: String,
    data_type: DataType,
    nullable: bool,
    data: Vec<u8>,
    offsets: Vec<usize>,
    null_bitmap: Option<BitVec<u8, Lsb0>>,  // 改用 BitVec
    row_count: usize,
}

impl Column {
    pub fn is_null(&self, row_idx: usize) -> bool {
        self.null_bitmap
            .as_ref()
            .map(|b| b.get(row_idx).map(|&v| v).unwrap_or(false))
            .unwrap_or(false)
    }

    pub fn set_null(&mut self, row_idx: usize, is_null: bool) {
        if let Some(ref mut bitmap) = self.null_bitmap {
            if row_idx >= bitmap.len() {
                bitmap.resize(row_idx + 1, false);
            }
            bitmap.set(row_idx, is_null);
        }
    }
}
```

---

### 2.2 字符串字典编码

#### 问题分析

当前实现：

```rust
// 变长字符串存储
fn write_variable_value(&mut self, value: &Value) {
    let bytes = s.as_bytes();
    let len = bytes.len() as u64;
    self.data.extend_from_slice(&len.to_le_bytes());  // 8 bytes overhead
    self.data.extend_from_slice(bytes);
}
```

问题：

- 每个字符串存储完整副本
- 低基数字符串列（如性别、状态）重复存储

#### 字典编码方案

```rust
pub struct StringColumn {
    dictionary: Vec<Arc<str>>,      // 唯一字符串池
    indices: Vec<u32>,              // 字典索引
    null_bitmap: Option<BitVec>,
}

impl StringColumn {
    pub fn insert(&mut self, value: Option<&str>) {
        match value {
            Some(s) => {
                // 查找或添加到字典
                let idx = self.dictionary.iter()
                    .position(|d| d.as_ref() == s)
                    .unwrap_or_else(|| {
                        self.dictionary.push(Arc::from(s));
                        self.dictionary.len() - 1
                    });
                self.indices.push(idx as u32);
            }
            None => {
                self.indices.push(0);  // 占位
                self.set_null(self.indices.len() - 1, true);
            }
        }
    }

    pub fn get(&self, row_idx: usize) -> Option<&str> {
        if self.is_null(row_idx) {
            return None;
        }
        self.dictionary.get(self.indices[row_idx] as usize)
            .map(|s| s.as_ref())
    }
}
```

#### 压缩效果估算

| 场景                             | 原始大小 | 字典编码后 | 压缩比 |
| -------------------------------- | -------- | ---------- | ------ |
| 100万行，100个唯一值，平均长度20 | 28 MB    | 4.4 MB     | 84%    |
| 100万行，10个唯一值，平均长度10  | 18 MB    | 4.0 MB     | 78%    |

---

### 2.3 ID 映射动态扩容

#### 问题分析

当前实现：

```rust
pub fn insert(&mut self, key: K) -> StorageResult<u32> {
    if self.keys.len() >= self.capacity {
        return Err(StorageError::CapacityExceeded);  // 硬性限制
    }
    // ...
}
```

#### 动态扩容方案

```rust
pub struct IdIndexer<K> {
    keys: Vec<K>,
    key_to_index: HashMap<K, u32>,
    growth_factor: f64,  // 扩容因子
    max_capacity: usize, // 最大容量限制 (可选)
}

impl<K: Eq + Hash + Clone> IdIndexer<K> {
    const DEFAULT_GROWTH_FACTOR: f64 = 1.5;
    const DEFAULT_INITIAL_CAPACITY: usize = 1024;

    pub fn insert(&mut self, key: K) -> StorageResult<u32> {
        // 检查是否需要扩容
        if self.keys.len() >= self.key_to_index.capacity() {
            self.grow()?;
        }

        let index = self.keys.len() as u32;
        self.keys.push(key.clone());
        self.key_to_index.insert(key, index);
        Ok(index)
    }

    fn grow(&mut self) -> StorageResult<()> {
        let new_capacity = (self.keys.capacity() as f64 * self.growth_factor) as usize;

        // 检查最大容量限制
        if let Some(max) = self.max_capacity {
            if new_capacity > max {
                return Err(StorageError::CapacityExceeded);
            }
        }

        self.keys.reserve(new_capacity - self.keys.capacity());
        self.key_to_index.reserve(new_capacity - self.key_to_index.len());
        Ok(())
    }
}
```

---

### 2.4 删除空洞回收

#### 问题分析

当前删除操作只是标记删除，不回收内部 ID：

```rust
pub fn remove(&mut self, index: u32, ts: Timestamp) {
    self.end_ts[index] = ts;
    self.deleted[index] = true;  // 仅标记
}
```

#### 方案一：延迟压缩

```rust
pub struct VertexTable {
    // ... 现有字段
    deleted_count: usize,        // 已删除计数
    compact_threshold: f64,      // 压缩阈值 (如 0.3 = 30% 删除)
}

impl VertexTable {
    pub fn delete(&mut self, external_id: &str, ts: Timestamp) -> StorageResult<()> {
        // ... 删除逻辑
        self.deleted_count += 1;

        // 检查是否需要压缩
        if self.deleted_count as f64 / self.total_count() as f64 > self.compact_threshold {
            self.compact();
        }
        Ok(())
    }

    pub fn compact(&mut self) -> StorageResult<()> {
        let mut new_id_indexer = IdIndexer::with_capacity(self.id_indexer.capacity());
        let mut new_columns = ColumnStore::new();
        // ... 初始化新存储

        let mut id_mapping = HashMap::new();  // old_id -> new_id

        // 遍历有效记录
        for (old_id, key) in self.id_indexer.iter() {
            if !self.timestamps.is_deleted(old_id) {
                let new_id = new_id_indexer.insert(key.clone())?;
                id_mapping.insert(old_id, new_id);

                // 复制数据
                let props = self.columns.get(old_id as usize);
                new_columns.set(new_id as usize, &props)?;
            }
        }

        // 替换存储
        self.id_indexer = new_id_indexer;
        self.columns = new_columns;
        self.timestamps = self.timestamps.compact_with_mapping(&id_mapping);
        self.deleted_count = 0;

        Ok(())
    }
}
```

#### 方案二：空闲列表复用

```rust
pub struct IdIndexer<K> {
    keys: Vec<Option<K>>,           // 改为 Option
    key_to_index: HashMap<K, u32>,
    free_list: VecDeque<u32>,       // 空闲 ID 列表
}

impl<K: Eq + Hash + Clone> IdIndexer<K> {
    pub fn insert(&mut self, key: K) -> StorageResult<u32> {
        // 优先复用空闲 ID
        if let Some(reused_id) = self.free_list.pop_front() {
            self.keys[reused_id as usize] = Some(key.clone());
            self.key_to_index.insert(key, reused_id);
            return Ok(reused_id);
        }

        // 无空闲 ID，追加新记录
        let index = self.keys.len() as u32;
        self.keys.push(Some(key.clone()));
        self.key_to_index.insert(key, index);
        Ok(index)
    }

    pub fn remove(&mut self, key: &K) -> Option<u32> {
        if let Some(index) = self.key_to_index.remove(key) {
            self.keys[index as usize] = None;
            self.free_list.push_back(index);
            return Some(index);
        }
        None
    }
}
```

---

### 2.5 批量操作优化

#### 问题分析

当前批量插入是逐条处理：

```rust
pub fn batch_insert_vertices(&self, vertices: Vec<Vertex>) -> Result<Vec<Value>> {
    for vertex in &vertices {
        // 逐条插入，每次都获取写锁
        graph.insert_vertex(...)?;
    }
}
```

#### 批量优化方案

```rust
impl VertexTable {
    pub fn batch_insert(
        &mut self,
        records: &[(String, Vec<(String, Value)>)],
        ts: Timestamp,
    ) -> StorageResult<Vec<u32>> {
        // 1. 预分配空间
        let new_count = records.len();
        self.ensure_capacity(self.total_count() + new_count);

        // 2. 批量构建 ID 映射
        let mut internal_ids = Vec::with_capacity(records.len());
        for (external_id, _) in records {
            let internal_id = self.id_indexer.insert(external_id.clone())?;
            internal_ids.push(internal_id);
        }

        // 3. 批量设置时间戳
        for &internal_id in &internal_ids {
            self.timestamps.insert(internal_id, ts);
        }

        // 4. 批量写入列数据
        for (i, (_, properties)) in records.iter().enumerate() {
            self.columns.set(internal_ids[i] as usize, properties)?;
        }

        Ok(internal_ids)
    }
}
```

---

## 三、实施计划

### 阶段一：基础优化 (优先级高)

| 任务                 | 预计工作量 | 风险 |
| -------------------- | ---------- | ---- |
| NULL 位图改为 BitVec | 2 天       | 低   |
| ID 映射动态扩容      | 1 天       | 低   |
| 添加单元测试         | 1 天       | 低   |

### 阶段二：压缩优化 (优先级中)

| 任务           | 预计工作量 | 风险 |
| -------------- | ---------- | ---- |
| 字符串字典编码 | 3 天       | 中   |
| RLE 编码支持   | 2 天       | 中   |
| 压缩算法选择器 | 1 天       | 低   |

### 阶段三：高级优化 (优先级低)

| 任务         | 预计工作量 | 风险 |
| ------------ | ---------- | ---- |
| 删除空洞回收 | 3 天       | 中   |
| 批量操作优化 | 2 天       | 低   |
| 性能基准测试 | 2 天       | 低   |

---

## 四、代码改动清单

### 4.1 新增文件

```
src/storage/vertex/
├── bitmap.rs           # BitVec 封装
├── encoding.rs         # 编码工具 (字典、RLE)
└── compact.rs          # 压缩逻辑
```

### 4.2 修改文件

| 文件                  | 改动内容                               |
| --------------------- | -------------------------------------- |
| `column_store.rs`     | NULL 位图改用 BitVec，添加字典编码支持 |
| `id_indexer.rs`       | 添加动态扩容和空闲列表复用             |
| `vertex_table.rs`     | 添加批量操作接口，集成压缩逻辑         |
| `vertex_timestamp.rs` | 添加压缩映射支持                       |

---

## 五、性能预期

| 指标                | 当前      | 优化后   | 提升        |
| ------------------- | --------- | -------- | ----------- |
| NULL 存储           | 1 byte/值 | 1 bit/值 | 8x          |
| 字符串存储 (低基数) | 原始大小  | 20-30%   | 3-5x        |
| 批量插入            | 逐条处理  | 批量处理 | 2-3x        |
| 内存碎片            | 无回收    | 自动压缩 | 减少 30-50% |

---

## 六、风险评估

| 风险              | 影响       | 缓解措施          |
| ----------------- | ---------- | ----------------- |
| BitVec API 兼容性 | 代码改动大 | 封装统一接口      |
| 字典编码解码开销  | 读性能下降 | 热点数据缓存      |
| 压缩期间服务中断  | 可用性下降 | 后台异步压缩      |
| 数据迁移          | 兼容性问题 | 版本号 + 迁移脚本 |

---

## 七、总结

本优化方案基于业界主流数据库的设计经验，针对 GraphDB 节点存储的痛点问题提出了具体的改进措施。主要优化方向：

1. **内存效率**：NULL 位图优化可节省 87.5% 内存
2. **存储压缩**：字典编码可减少 70-80% 字符串存储
3. **扩展性**：动态扩容解决固定容量限制
4. **维护性**：自动压缩减少内存碎片

建议按阶段逐步实施，优先完成高优先级的基础优化，再逐步引入压缩和高级特性。
