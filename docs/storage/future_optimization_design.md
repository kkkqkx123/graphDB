# 后续优化任务设计文档

> 本文档详细描述存储层优化计划中剩余任务的实现方案，供后续开发参考。

---

## 一、VertexIndexManager + EdgeIndexManager 泛型化

### 1.1 问题分析

当前 `VertexIndexManager` 和 `EdgeIndexManager` 存在大量重复代码：

| 文件                    | 行数    | 重复代码估计 |
| ----------------------- | ------- | ------------ |
| vertex_index_manager.rs | ~400 行 | ~60%         |
| edge_index_manager.rs   | ~400 行 | ~60%         |

**重复内容：**

- 数据结构定义（`forward_index`, `reverse_index`, `compressor`）
- 压缩方法（`compress_key`, `decompress_key`, `train_compression`）
- MVCC 处理逻辑
- 持久化逻辑（`flush`, `save`, `load`）
- GC 逻辑（`gc_tombstones`, `gc_tombstones_incremental`）

**唯一差异：**

- Key 构建方法：`KeyBuilder::build_vertex_index_key` vs `KeyBuilder::build_edge_index_key`
- Key 解析方法：`KeyParser::parse_vertex_reverse_key` vs `KeyParser::parse_edge_reverse_key`

### 1.2 设计方案

#### 方案 A：泛型参数化（推荐）

```rust
/// Index key generator trait
/// Abstracts the key generation logic for different index types
pub trait IndexKeyGenerator: Send + Sync + 'static {
    /// Build forward index key (property -> id mapping)
    fn build_forward_key(
        space_id: u64,
        index_name: &str,
        prop_value: &Value,
        ids: &[&Value],
    ) -> Result<ByteKey, StorageError>;

    /// Build reverse index key (id -> property mapping)
    fn build_reverse_key(
        space_id: u64,
        ids: &[&Value],
        index_name: &str,
    ) -> Result<ByteKey, StorageError>;

    /// Build reverse key prefix for range scan
    fn build_reverse_prefix(
        space_id: u64,
        ids: &[&Value],
    ) -> Result<ByteKey, StorageError>;

    /// Parse reverse key to extract id and index name
    fn parse_reverse_key(key: &[u8]) -> Result<(Vec<u8>, String), StorageError>;

    /// Type name for logging/debugging
    fn type_name() -> &'static str;
}

/// Generic index manager
pub struct GenericIndexManager<K: IndexKeyGenerator> {
    forward_index: Arc<RwLock<BTreeMap<SecondaryIndexKey, IndexEntry>>>,
    reverse_index: Arc<RwLock<BTreeMap<SecondaryIndexKey, IndexEntry>>>,
    compressor: Option<Arc<RwLock<IndexCompressor>>>,
    _marker: PhantomData<K>,
}

// Type aliases for backward compatibility
pub type VertexIndexManager = GenericIndexManager<VertexIndexKeyGen>;
pub type EdgeIndexManager = GenericIndexManager<EdgeIndexKeyGen>;
```

#### 方案 B：宏生成

```rust
macro_rules! define_index_manager {
    ($name:ident, $key_gen:ty) => {
        pub struct $name {
            forward_index: Arc<RwLock<BTreeMap<SecondaryIndexKey, IndexEntry>>>,
            reverse_index: Arc<RwLock<BTreeMap<SecondaryIndexKey, IndexEntry>>>,
            compressor: Option<Arc<RwLock<IndexCompressor>>>,
        }

        impl $name {
            // Generated methods using $key_gen
        }
    };
}

define_index_manager!(VertexIndexManager, VertexIndexKeyGen);
define_index_manager!(EdgeIndexManager, EdgeIndexKeyGen);
```

### 1.3 实现步骤

1. **定义 `IndexKeyGenerator` trait**
   - 文件：`src/storage/index/secondary/key_generator.rs`
   - 抽象 key 构建和解析逻辑

2. **实现具体 KeyGenerator**
   - `VertexIndexKeyGen`：使用 `KeyBuilder::build_vertex_*` 方法
   - `EdgeIndexKeyGen`：使用 `KeyBuilder::build_edge_*` 方法

3. **重构为泛型实现**
   - 创建 `GenericIndexManager<K: IndexKeyGenerator>`
   - 将公共逻辑移入泛型实现
   - 使用 `K::build_forward_key()` 替代硬编码方法调用

4. **保持向后兼容**
   - 使用 type alias：`pub type VertexIndexManager = GenericIndexManager<VertexIndexKeyGen>`
   - 导出原有类型名

### 1.4 风险评估

| 风险                 | 等级 | 缓解措施              |
| -------------------- | ---- | --------------------- |
| 泛型增加类型复杂度   | 中   | 保持清晰的 trait 文档 |
| PhantomData 约束传播 | 低   | 仅在构造函数使用      |
| 测试覆盖             | 中   | 复用现有测试用例      |

### 1.5 工作量估计

- 设计 + 实现：1-2 天
- 测试验证：0.5 天
- 总计：1.5-2.5 天

---

## 二、Column 变长/定长拆分

### 2.1 问题分析

当前 `Column` 结构体在 `set()` 和 `get()` 方法中需要运行时判断变长/定长类型：

```rust
// 当前实现中的分支判断
fn set(&mut self, row_idx: usize, value: &Value) {
    if self.is_variable_length() {
        // 变长类型逻辑：使用 offsets 数组
    } else {
        // 定长类型逻辑：直接计算偏移
    }
}
```

**问题：**

- 每次读写都有分支判断开销
- 变长和定长的优化路径不同，难以各自优化
- 添加新变长类型（JSON、Bytes）需要多处修改

### 2.2 设计方案

```rust
/// Column storage unified interface
pub trait ColumnStorage: Send + Sync {
    fn get(&self, row_idx: usize) -> Option<Value>;
    fn set(&mut self, row_idx: usize, value: &Value) -> Result<(), StorageError>;
    fn len(&self) -> usize;
    fn is_null(&self, row_idx: usize) -> bool;
    fn memory_usage(&self) -> usize;
}

/// Fixed-width column for primitive types
pub struct FixedWidthColumn {
    data: Vec<u8>,
    element_size: usize,
    null_bitmap: Option<BitVec>,
    row_count: usize,
    data_type: DataType,
    encoding: ColumnEncoding,
}

/// Variable-width column for String, Bytes, JSON
pub struct VariableWidthColumn {
    data: Vec<u8>,           // Concatenated value data
    offsets: Vec<usize>,     // Start offset for each row
    lengths: Vec<usize>,     // Length for each row (for variable-length)
    null_bitmap: Option<BitVec>,
    row_count: usize,
    data_type: DataType,
    encoding: ColumnEncoding,
}

/// Column enum for runtime dispatch
pub enum Column {
    Fixed(FixedWidthColumn),
    Variable(VariableWidthColumn),
}
```

### 2.3 实现步骤

1. **定义 `ColumnStorage` trait**
   - 统一列存储接口
   - 提供默认实现

2. **实现 `FixedWidthColumn`**
   - 直接偏移计算：`offset = row_idx * element_size`
   - 支持 RLE、BitPacking 等定长编码
   - O(1) 随机访问

3. **实现 `VariableWidthColumn`**
   - 使用 offsets 数组定位
   - 支持 Dictionary、FSST 等变长编码
   - O(1) 随机访问（通过 offsets）

4. **重构 `Column` 为枚举**
   - 构造时根据 DataType 选择实现
   - 委托方法调用到具体实现

5. **更新 `ColumnStore` 和 `PropertyTable`**
   - 使用新的 Column 接口
   - 保持 API 兼容

### 2.4 性能影响

| 操作     | 当前        | 优化后   |
| -------- | ----------- | -------- |
| 定长 get | 分支 + 计算 | 直接计算 |
| 定长 set | 分支 + 计算 | 直接计算 |
| 变长 get | 分支 + 查表 | 直接查表 |
| 变长 set | 分支 + 查表 | 直接查表 |

### 2.5 工作量估计

- 设计 + 实现：1-2 天
- 测试验证：0.5 天
- 总计：1.5-2.5 天

---

## 三、MutableCsr 扩容性能优化

### 3.1 问题分析

当前 `MutableCsr.expand_vertex_capacity()` 使用 `Vec::splice()`：

```rust
fn expand_vertex_capacity(&mut self, src_idx: usize) {
    self.nbr_list.splice(
        insert_pos..insert_pos,
        std::iter::repeat_n(empty_nbr, additional),
    );
    // splice 导致插入位置之后所有元素移位，O(n) 复杂度
}
```

**性能问题：**

- 高频插入场景下，每次扩容都是 O(n)
- 内存移动开销大
- 缓存不友好

### 3.2 设计方案

#### 方案 A：分段存储（推荐）

```rust
/// Per-vertex edge list storage
pub struct MutableCsrV2 {
    /// Each vertex has its own edge list
    nbr_lists: Vec<Vec<Nbr>>,
    /// Vertex degrees for fast lookup
    degrees: Vec<u32>,
    /// Total edge count
    edge_count: AtomicU64,
    /// Vertex capacity
    vertex_capacity: usize,
}

impl MutableCsrV2 {
    fn insert_edge(&mut self, src: VertexId, dst: VertexId, ...) -> bool {
        let src_idx = src.as_int64().unwrap_or(0) as usize;
        if src_idx >= self.vertex_capacity {
            self.expand_vertex_capacity(src_idx + 1);
        }
        // O(1) amortized push
        self.nbr_lists[src_idx].push(Nbr { neighbor: dst, ... });
        self.degrees[src_idx] += 1;
        true
    }

    fn expand_vertex_capacity(&mut self, new_capacity: usize) {
        self.nbr_lists.resize(new_capacity, Vec::new());
        self.degrees.resize(new_capacity, 0);
        self.vertex_capacity = new_capacity;
    }
}
```

**优点：**

- 插入 O(1) 摊销复杂度
- 扩容仅影响单个 Vec
- 删除使用软删除（timestamp 标记）

**缺点：**

- 内存碎片化
- 遍历所有边需要遍历所有 Vec

#### 方案 B：Slot 池 + 指针链

```rust
/// Slot-based edge storage
pub struct MutableCsrV3 {
    /// Contiguous edge pool
    edge_pool: Vec<Nbr>,
    /// Head index for each vertex
    vertex_heads: Vec<usize>,
    /// Edge count for each vertex
    vertex_counts: Vec<u32>,
    /// Free slots for reuse
    free_slots: Vec<usize>,
    /// Next pointer in each slot (for linked list)
    next_ptrs: Vec<usize>,
}
```

**优点：**

- 内存连续
- 支持链表遍历

**缺点：**

- 实现复杂
- 需要维护 next 指针

### 3.3 实现步骤

1. **创建 `MutableCsrV2` 原型**
   - 实现 `MutableCsrTrait` 接口
   - 使用分段存储

2. **性能基准测试**
   - 对比 `MutableCsr` 和 `MutableCsrV2`
   - 测试场景：批量插入、随机访问、遍历

3. **根据测试结果决定方案**
   - 如果分段存储性能提升明显，替换实现
   - 否则保持现有实现

4. **持久化格式兼容**
   - 如需修改持久化格式，添加版本号
   - 提供迁移工具

### 3.4 工作量估计

- 原型实现：1 天
- 性能测试：0.5 天
- 完整实现（如采用）：1-1.5 天
- 总计：2.5-3 天

---

## 四、实施优先级建议

| 优先级 | 任务                      | 风险 | 收益                 | 建议             |
| ------ | ------------------------- | ---- | -------------------- | ---------------- |
| 1      | VertexIndexManager 泛型化 | 中   | 消除 500+ 行重复代码 | 优先实施         |
| 2      | Column 变长/定长拆分      | 中   | 提升列存性能         | 次优先           |
| 3      | MutableCsr 扩容优化       | 高   | 高频写入性能         | 需要性能测试验证 |

---

## 五、附录：已完成的优化

| 任务                               | 状态 | 描述                      |
| ---------------------------------- | ---- | ------------------------- |
| MutableCsrVariant 重复代码消除     | ✅   | 使用 delegate! 宏         |
| ExtendedSchemaManager stub cleanup | ✅   | 移除未实现方法            |
| PrimaryIndex trait 移除            | ✅   | 转为固有方法              |
| ID 计数器 DashMap 优化             | ✅   | 消除锁竞争                |
| IndexDataManager trait 拆分        | ✅   | 拆分为三个小 trait        |
| Schema 结构体清理                  | ✅   | 移除未使用方法            |
| EdgeTable cache 解耦               | ✅   | 使用 EdgeTableCache trait |
