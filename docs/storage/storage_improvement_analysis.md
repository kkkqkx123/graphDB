# 存储架构改进分析报告

## 一、分析概述

本文档基于 `database_storage_research.md` 调研报告，对照现有代码实现，分析改进空间并制定实施计划。

---

## 二、已实现功能对照表

| 调研报告推荐 | 现有实现 | 状态 | 代码位置 |
|-------------|---------|------|----------|
| **DuckDB Validity Mask** | `BitVec<u8, Lsb0>` | ✅ 已实现 | `src/storage/vertex/column_store.rs` |
| **Dictionary Encoding** | `DictionaryColumn` | ✅ 已实现 | `src/storage/vertex/encoding/dictionary.rs` |
| **RLE Encoding** | `RleEncoder` | ✅ 已实现 | `src/storage/vertex/encoding/rle.rs` |
| **Zstd Compression** | `CompressionType::Zstd` | ✅ 已实现 | `src/storage/persistence/compression.rs` |
| **Block Cache** | `PageManager` (Moka) | ✅ 已实现 | `src/storage/page/page_manager.rs` |
| **Record Cache** | `RecordCache` (TinyLFU) | ✅ 已实现 | `src/storage/cache/record_cache.rs` |
| **动态扩容** | `IdIndexerConfig` | ✅ 已实现 | `src/storage/vertex/id_indexer.rs` |
| **空闲列表复用** | `free_list: VecDeque<u32>` | ✅ 已实现 | `src/storage/vertex/id_indexer.rs` |

---

## 三、待改进功能清单

### 3.1 高优先级 ✅ 已完成

| 功能 | 调研来源 | 现状 | 预期收益 |
|------|---------|------|---------|
| **BitPacking 编码** | DuckDB | ✅ 已实现 | 小范围整数节省 50-75% 空间 |
| **Bloom Filter** | RocksDB | ✅ 已实现 | 快速判断键存在性，减少磁盘 IO |
| **Varint 编码** | SQLite | ✅ 已实现 | 紧凑存储小整数，减少 30-50% 空间 |

### 3.2 中优先级 ✅ 已完成

| 功能 | 调研来源 | 现状 | 预期收益 |
|------|---------|------|---------|
| **FSST 字符串压缩** | DuckDB | ✅ 已实现 | 高效压缩长字符串、高基数场景 |
| **ALP 浮点数压缩** | DuckDB | ✅ 已实现 | 浮点数压缩比 70-80% |
| **延迟解压** | DuckDB | ✅ 已实现 | 压缩状态下执行查询，减少解压开销 |
| **分层压缩策略** | RocksDB | ✅ 已实现 | 热数据快速访问，冷数据高压缩比 |

### 3.3 低优先级

| 功能 | 调研来源 | 现状 | 预期收益 |
|------|---------|------|---------|
| **向量化执行** | DuckDB | 未实现 | 批量处理 2048 行，CPU 缓存友好 |
| **SSTable 结构** | RocksDB | 未实现 | 有序键值对文件，适合持久化 |
| **溢出页处理** | SQLite | 未实现 | 处理大记录，不浪费页面空间 |

---

## 四、详细改进方案

### 4.1 BitPacking 编码

#### 问题分析

当前整数存储使用固定大小 (4/8 bytes)，小范围整数浪费空间。

**现有实现** (`column_store.rs`):
```rust
(DataType::Int, Value::Int(i)) => {
    self.data[offset..offset + 4].copy_from_slice(&i.to_le_bytes());
    // 固定 4 bytes，即使值很小
}
```

#### 调研参考 (DuckDB)

```c
// BitPacking: 将值压缩到最小所需位数
// 例如: 值范围 0-100 只需 7 bits 存储
// 压缩流程:
// 1. 分析数据范围，确定 bit_width
// 2. 计算偏移量 (min_value)
// 3. 按位打包存储
```

#### 实现方案

```rust
pub struct BitPackedColumn {
    data: Vec<u8>,
    bit_width: u8,      // 每个值的位数 (1-64)
    min_value: i64,     // 偏移量，用于减少 bit_width
    row_count: usize,
    null_bitmap: Option<BitVec<u8, Lsb0>>,
}

impl BitPackedColumn {
    pub fn analyze(values: &[i64]) -> Self {
        let min_val = *values.iter().min().unwrap_or(&0);
        let max_val = *values.iter().max().unwrap_or(&0);
        let range = (max_val - min_val) as u64;
        let bit_width = Self::calculate_bit_width(range);
        // ...
    }

    fn calculate_bit_width(range: u64) -> u8 {
        if range == 0 { return 1; }
        (64 - range.leading_zeros()) as u8
    }

    pub fn get(&self, row_idx: usize) -> Option<i64> {
        let bit_offset = row_idx * self.bit_width as usize;
        let byte_offset = bit_offset / 8;
        let bit_offset_in_byte = bit_offset % 8;
        // Extract bits and add min_value
    }
}
```

#### 预期效果

| 数据范围 | 原始大小 | BitPacking | 节省 |
|---------|---------|------------|------|
| 0-100 | 4 bytes/值 | 7 bits/值 | 78% |
| 0-1000 | 4 bytes/值 | 10 bits/值 | 69% |
| -1000~1000 | 4 bytes/值 | 11 bits/值 | 66% |

---

### 4.2 Bloom Filter

#### 问题分析

查询不存在的键时需要完整扫描或索引查找，造成不必要的 IO 开销。

#### 调研参考 (RocksDB)

```
SSTable 结构:
[data block 1]
[data block 2]
...
[meta block: filter block]    <- Bloom Filter
[meta block: index block]
[Footer]
```

#### 实现方案

```rust
pub struct BloomFilter {
    bitmap: BitVec<u8, Lsb0>,
    hash_count: usize,    // 哈希函数数量
    bit_count: usize,     // 总位数
}

impl BloomFilter {
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        // 计算最优参数
        let ln2 = std::f64::consts::LN_2;
        let bit_count = (-1.0 * expected_items as f64 * false_positive_rate.ln() / (ln2 * ln2)) as usize;
        let hash_count = (bit_count as f64 / expected_items as f64 * ln2).ceil() as usize;

        Self {
            bitmap: BitVec::repeat(false, bit_count),
            hash_count: hash_count.max(1),
            bit_count,
        }
    }

    pub fn insert(&mut self, key: &[u8]) {
        let hashes = self.hash_key(key);
        for h in hashes {
            self.bitmap.set(h % self.bit_count, true);
        }
    }

    pub fn might_contain(&self, key: &[u8]) -> bool {
        let hashes = self.hash_key(key);
        hashes.iter().all(|&h| self.bitmap[h % self.bit_count])
    }

    fn hash_key(&self, key: &[u8]) -> Vec<usize> {
        // 使用双重哈希技术生成多个哈希值
        let h1 = Self::murmur_hash(key, 0);
        let h2 = Self::murmur_hash(key, h1 as u32);
        (0..self.hash_count)
            .map(|i| (h1.wrapping_add(i as u64 * h2)) as usize)
            .collect()
    }

    fn murmur_hash(data: &[u8], seed: u32) -> u64 {
        // MurmurHash3 实现
    }
}
```

#### 应用场景

1. **IdIndexer**: 快速判断外部 ID 是否存在
2. **索引查询**: 前置过滤，减少不必要的查找
3. **持久化**: 存储到文件头部，加载时快速判断

---

### 4.3 Varint 编码

#### 问题分析

字符串长度使用固定 8 bytes 存储，短字符串浪费空间。

**现有实现** (`column_store.rs`):
```rust
let len = bytes.len() as u64;
self.data.extend_from_slice(&len.to_le_bytes());  // 固定 8 bytes
self.data.extend_from_slice(bytes);
```

#### 调研参考 (SQLite)

```
Varint 编码规则:
- 值 0-127: 1 byte (最高位 0)
- 值 128-16383: 2 bytes (最高位 1, 后续最高位 0)
- 最大支持 9 bytes

示例:
- 0x00 -> 0x00 (1 byte)
- 0x7F -> 0x7F (1 byte)
- 0x80 -> 0x81 0x00 (2 bytes)
- 0x3FFF -> 0xFF 0x7F (2 bytes)
```

#### 实现方案

```rust
pub struct Varint;

impl Varint {
    pub fn encode(value: u64) -> Vec<u8> {
        if value < 0x80 {
            return vec![value as u8];
        }

        let mut result = Vec::new();
        let mut v = value;

        while v >= 0x80 {
            result.push((v as u8) | 0x80);
            v >>= 7;
        }
        result.push(v as u8);

        result
    }

    pub fn decode(data: &[u8]) -> (u64, usize) {
        let mut result = 0u64;
        let mut shift = 0;
        let mut bytes_read = 0;

        for &byte in data {
            bytes_read += 1;
            result |= ((byte & 0x7F) as u64) << shift;

            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        (result, bytes_read)
    }

    pub fn encoded_len(value: u64) -> usize {
        if value == 0 { return 1; }
        let bits = 64 - value.leading_zeros();
        ((bits + 6) / 7) as usize
    }
}
```

#### 预期效果

| 值范围 | 固定长度 | Varint | 节省 |
|--------|---------|--------|------|
| 0-127 | 8 bytes | 1 byte | 87.5% |
| 128-16383 | 8 bytes | 2 bytes | 75% |
| 16384-2097151 | 8 bytes | 3 bytes | 62.5% |

---

### 4.4 FSST 字符串压缩

#### 问题分析

字典编码对高基数字符串效果差，需要补充方案。

#### 调研参考 (DuckDB)

```
FSST (Fast Static Symbol Table):
- 构建符号表，将常见子串映射为短编码
- 适合长字符串、高基数场景
- 压缩比中等 (30-50%)，解压速度极快 (10GB/s+)
```

#### 实现方案

```rust
pub struct FsstEncoder {
    symbol_table: Vec<(Vec<u8>, u8)>,  // (symbol, code)
    code_table: Vec<Vec<u8>>,          // code -> symbol
}

impl FsstEncoder {
    pub fn train(strings: &[&str]) -> Self {
        // 1. 统计所有 1-8 字节的子串频率
        // 2. 选择高频子串构建符号表
        // 3. 使用贪心算法优化编码
    }

    pub fn compress(&self, s: &str) -> Vec<u8> {
        // 使用符号表编码
    }

    pub fn decompress(&self, compressed: &[u8]) -> String {
        // 查表解码
    }
}
```

---

### 4.5 ALP 浮点数压缩

#### 问题分析

浮点数存储无压缩，占用空间大。

#### 调研参考 (DuckDB ALP)

```
ALP (Adaptive Lossless floating-Point compression):
- 识别浮点数的整数模式
- 乘以 10^k 转换为整数
- 使用 BitPacking 存储
- 压缩比可达 70-80%
```

#### 实现方案

```rust
pub struct AlpEncoder {
    factor: i32,        // 10^k
    exponent: i8,       // k
    bit_packed: BitPackedColumn,
}

impl AlpEncoder {
    pub fn analyze(values: &[f64]) -> Self {
        // 找到最优的 k 值，使得转换后的整数范围最小
    }

    pub fn compress(&self, value: f64) -> i64 {
        (value * 10f64.powi(self.exponent as i32)) as i64
    }

    pub fn decompress(&self, value: i64) -> f64 {
        value as f64 / 10f64.powi(self.exponent as i32)
    }
}
```

---

### 4.6 延迟解压

#### 问题分析

当前读取时必须完全解压，增加 CPU 开销。

#### 调研参考 (DuckDB)

```c
// Dictionary Vector 可在压缩状态下执行查询
// 例如: WHERE col = 'apple' 可直接比较索引
```

#### 实现方案

```rust
pub trait EncodedColumn {
    fn get(&self, row_idx: usize) -> Option<Value>;

    // 新增: 压缩状态下的操作
    fn equals(&self, row_idx: usize, value: &Value) -> bool;
    fn compare(&self, row_idx: usize, value: &Value) -> std::cmp::Ordering;
    fn find_value(&self, value: &Value) -> Vec<usize>;
}

impl EncodedColumn for DictionaryColumn {
    fn equals(&self, row_idx: usize, value: &Value) -> bool {
        // 直接比较字典索引，无需解压
        if let Value::String(s) = value {
            if let Some(idx) = self.reverse_lookup.get(s) {
                return self.indices[row_idx] == *idx;
            }
        }
        false
    }
}
```

---

### 4.7 分层压缩策略

#### 问题分析

当前压缩策略较为单一，未考虑数据特征和访问模式。

#### 调研参考 (RocksDB)

```cpp
options.compression_per_level = {
    kNoCompression,   // Level 0: 写入频繁，不压缩
    kSnappy,          // Level 1-2: 快速压缩
    kLZ4,             // Level 3-4: 中等压缩
    kZSTD             // Level 5+: 高压缩比
};
```

#### 实现方案

```rust
pub struct CompressionSelector {
    hot_threshold: usize,     // 热数据判定阈值
    cold_threshold: usize,    // 冷数据判定阈值
}

impl CompressionSelector {
    pub fn select(&self, stats: &ColumnStats) -> EncodingType {
        // 热数据: 优先访问速度
        if stats.access_count > self.hot_threshold {
            return EncodingType::None;
        }

        // 冷数据: 优先压缩比
        if stats.access_count < self.cold_threshold {
            return self.select_best_compression(&stats);
        }

        // 温数据: 平衡速度和压缩比
        EncodingType::Rle
    }

    fn select_best_compression(&self, stats: &ColumnStats) -> EncodingType {
        match stats.data_type {
            DataType::String if stats.cardinality_ratio < 0.5 => EncodingType::Dictionary,
            DataType::Int | DataType::BigInt if stats.run_ratio < 0.3 => EncodingType::Rle,
            DataType::Int | DataType::BigInt if stats.value_range < 1000 => EncodingType::BitPacking,
            _ => EncodingType::None,
        }
    }
}
```

---

## 五、实施计划

### 阶段一：基础编码优化 (高优先级) ✅ 已完成

| 任务 | 预计工作量 | 文件 | 状态 |
|------|-----------|------|------|
| BitPacking 编码实现 | 2 天 | `encoding/bitpacking.rs` | ✅ 已完成 |
| Bloom Filter 实现 | 1 天 | `utils/bloom_filter.rs` | ✅ 已完成 |
| Varint 编码实现 | 1 天 | `encoding/varint.rs` | ✅ 已完成 |
| 单元测试 | 1 天 | 各模块 test | ✅ 已完成 |

### 阶段二：高级压缩优化 (中优先级) ✅ 已完成

| 任务 | 预计工作量 | 文件 | 状态 |
|------|-----------|------|------|
| FSST 字符串压缩 | 3 天 | `encoding/fsst.rs` | ✅ 已完成 |
| ALP 浮点数压缩 | 2 天 | `encoding/alp.rs` | ✅ 已完成 |
| 延迟解压支持 | 2 天 | `encoding/lazy.rs` | ✅ 已完成 |
| 分层压缩策略 | 1 天 | `encoding/selector.rs` | ✅ 已完成 |

### 阶段三：性能优化 (低优先级)

| 任务 | 预计工作量 | 文件 |
|------|-----------|------|
| 向量化执行引擎 | 5 天 | `execution/vector.rs` |
| SSTable 持久化 | 3 天 | `persistence/sstable.rs` |
| 溢出页处理 | 2 天 | `page/overflow.rs` |

---

## 六、风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| BitPacking 边界处理 | 数据损坏 | 充分的单元测试，边界用例覆盖 |
| Bloom Filter 误判 | 查询遗漏 | 设置合理的误判率，文档说明 |
| Varint 兼容性 | 数据迁移 | 版本号标识，迁移脚本 |
| FSST 训练开销 | 写入延迟 | 后台异步训练，缓存符号表 |
| ALP 精度问题 | 数据精度丢失 | 仅用于特定场景，保留原始数据 |

---

## 七、总结

### 完成度统计

| 类别 | 已实现 | 待实现 | 完成度 |
|------|--------|--------|--------|
| **NULL 位图** | ✅ BitVec | - | 100% |
| **编码压缩** | Dictionary, RLE, BitPacking, FSST, ALP | - | 100% |
| **整数存储** | Varint, BitPacking | - | 100% |
| **查询优化** | Bloom Filter, 延迟解压 | - | 100% |
| **缓存系统** | RecordCache, PageManager | - | 100% |
| **ID 管理** | 动态扩容, 空闲列表 | - | 100% |

### 核心差距

1. **BitPacking** - 小整数压缩缺失，影响存储效率
2. **Bloom Filter** - 快速过滤缺失，影响查询性能
3. **Varint** - 变长编码缺失，影响字符串存储效率
4. **FSST/ALP** - 高级压缩缺失，影响特定场景性能

### 预期收益

完成全部优化后：
- 存储空间减少 40-60%
- 查询性能提升 20-30%
- 内存占用减少 30-50%
