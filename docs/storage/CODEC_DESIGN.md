# Codec 模块设计方案

## 一、设计背景与目标

### 1.1 项目现状分析

当前 Rust 图数据库项目采用 `serde_json` 和 `bincode` 进行数据序列化，存在以下问题：

1. **存储效率低**：JSON 序列化存在大量冗余字符，空间开销大
2. **访问粒度粗**：整体序列化导致无法进行字段级随机访问
3. **编解码性能差**：每次操作都需要完整序列化/反序列化整个对象
4. **键设计简单**：缺乏结构化的键编码，无法支持高效范围扫描

### 1.2 设计目标

参考 NebulaGraph 的成熟实现，设计一个高效的 Codec 模块：

1. **紧凑存储**：减少 30-50% 存储空间
2. **快速访问**：O(1) 字段随机访问性能
3. **Schema 驱动**：编解码过程由 Schema 定义驱动
4. **向后兼容**：支持平滑迁移现有数据

### 1.3 对比分析

| 特性 | 当前实现 | NebulaGraph | 目标实现 |
|------|----------|-------------|----------|
| 序列化方式 | JSON + Bincode | 自定义二进制 | 自定义二进制 |
| Schema 版本 | 单版本 | 多版本 | 单版本（可扩展） |
| 键设计 | 字符串分隔 | 二进制结构 | 二进制结构 |
| 字段访问 | O(n) 全量解析 | O(1) 直接访问 | O(1) 直接访问 |

## 二、架构设计

### 2.1 模块结构

```
src/core/codec/
├── mod.rs                    # 模块入口
├── error.rs                  # 错误类型
├── row_buffer.rs             # 二进制缓冲区
├── field_accessor.rs         # 字段访问器
├── row_writer.rs             # 编码器
├── row_reader.rs             # 解码器
├── key_utils.rs              # 键编码工具
└── test/
    ├── mod.rs
    └── codec_test.rs
```

### 2.2 二进制格式设计

```
┌─────────────────────────────────────────────────────────────────┐
│                      RowWriterV2 二进制格式                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   头部 (1 字节):                                                 │
│   ┌─────────────────────────────────────────────────────────┐   │
│   │  0x08 (固定值，表示 V2 格式)                              │   │
│   └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│   数据布局:                                                      │
│   +--------+--------+--------+--------+------...                  │
│   | Header |  Data  | NULL区  | 变长内容 | ...                   │
│   +--------+--------+--------+--------+------...                  │
│                                                                  │
│   字段类型编码:                                                   │
│   - BOOL(1) | INT8(1) | INT16(2) | INT32(4) | INT64(8)         │
│   - FLOAT(4) | DOUBLE(8) | STRING(8) | FIXED_STRING(N)         │
│   - DATE(4) | TIME(8) | DATETIME(10) | TIMESTAMP(8)            │
│   - VID(8) | GEOGRAPHY(8) | DURATION(16)                        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 键格式设计

```
┌─────────────────────────────────────────────────────────────────┐
│                         键格式设计                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   顶点键 (12 + vid_len bytes):                                   │
│   +--------+--------+--------+--------+------...                  │
│   | type   | partId |   vid (定长填充)  |                        │
│   +--------+--------+--------+--------+------...                  │
│                                                                  │
│   边键 (21 + vid_len*2 bytes):                                   │
│   +--------+--------+--------+--------+--------+------...         │
│   | type   | partId | srcVID | edgeType | rank | dstVID |       │
│   +--------+--------+--------+--------+--------+------...         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## 三、核心实现

### 3.1 错误类型 (error.rs)

```rust
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Type mismatch: {0}")]
    TypeMismatch(String),

    #[error("Field not found: {0}")]
    FieldNotFound(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Unsupported data type: {0}")]
    UnsupportedDataType(String),
}
```

### 3.2 RowBuffer (row_buffer.rs)

```rust
pub struct RowBuffer {
    buffer: Vec<u8>,
    header_len: usize,
    data_start: usize,
    str_content_start: usize,
}

impl RowBuffer {
    pub fn with_capacity(schema: &Schema) -> Self {
        // 预分配缓冲区
        // 写入头部占位符
    }

    pub fn write_bool(&mut self, offset: usize, value: bool);
    pub fn write_fixed<T: AsBytes>(&mut self, offset: usize, value: &T);
    pub fn write_string_offset(&mut self, offset: usize, str_offset: u32, str_len: u32);
    pub fn append_string_content(&mut self, content: &[u8]) -> usize;
}
```

### 3.3 RowWriter (row_writer.rs)

```rust
pub struct RowWriter<'a> {
    schema: &'a Schema,
    buffer: RowBuffer,
    is_set: Vec<bool>,
    finished: bool,
}

impl<'a> RowWriter<'a> {
    pub fn new(schema: &'a Schema) -> Self;
    pub fn set_null(&mut self, field_name: &str) -> Result<(), CodecError>;
    pub fn set_bool(&mut self, field_name: &str, value: bool) -> Result<(), CodecError>;
    pub fn set_int64(&mut self, field_name: &str, value: i64) -> Result<(), CodecError>;
    pub fn set_string(&mut self, field_name: &str, value: &str) -> Result<(), CodecError>;
    pub fn finish(self) -> Result<Vec<u8>, CodecError>;
}
```

### 3.4 RowReader (row_reader.rs)

```rust
pub struct RowReader<'a> {
    data: &'a [u8],
    schema: &'a Schema,
    accessor: FieldAccessor<'a>,
    cache: RwLock<HashMap<usize, Value>>,
}

impl<'a> RowReader<'a> {
    pub fn new(data: &'a [u8], schema: &'a Schema) -> Result<Self, CodecError>;
    pub fn get_value(&self, field_name: &str) -> Result<Value, CodecError>;
    pub fn get_value_by_index(&self, index: usize) -> Result<Value, CodecError>;
}
```

### 3.5 KeyUtils (key_utils.rs)

```rust
pub struct KeyUtils;

impl KeyUtils {
    pub fn encode_vertex_key(vid_len: usize, space_id: u32, part_id: u32, vid: &[u8]) -> Vec<u8>;
    pub fn encode_tag_key(vid_len: usize, space_id: u32, part_id: u32, vid: &[u8], tag_id: u32) -> Vec<u8>;
    pub fn encode_edge_key(vid_len: usize, space_id: u32, part_id: u32, src_vid: &[u8], edge_type: u32, rank: i64, dst_vid: &[u8]) -> Vec<u8>;
    pub fn decode_vertex_key(key: &[u8], vid_len: usize) -> Result<(u32, Vec<u8>), CodecError>;
    pub fn vertex_prefix(vid_len: usize, part_id: u32) -> Vec<u8>;
    pub fn edge_prefix(vid_len: usize, part_id: u32, src_vid: &[u8]) -> Vec<u8>;
}
```

## 四、集成方案

### 4.1 与现有模块的关系

```
src/core/
├── codec/          # 新增编解码模块
├── types/          # 类型定义（DataType, Value 等）
└── ...

src/storage/
├── serializer/     # 保留（用于向后兼容）
├── redb_storage.rs # 集成新的 codec
└── ...
```

### 4.2 存储引擎集成

修改 `src/storage/redb_storage.rs`：

```rust
// 使用新的 codec 模块
use crate::core::codec::{RowWriter, RowReader, KeyUtils, FormatVersion};

// 序列化
fn serialize_vertex_with_codec(vertex: &Vertex, schema: &Schema) -> Result<Vec<u8>, StorageError> {
    let mut writer = RowWriter::new(schema);
    // 设置各字段...
    writer.finish().map_err(|e| StorageError::SerializeError(e.to_string()))
}

// 反序列化
fn deserialize_vertex_with_codec(data: &[u8], schema: &Schema) -> Result<Vertex, StorageError> {
    let reader = RowReader::new(data, schema)?;
    // 读取各字段...
}
```

### 4.3 迁移策略

采用渐进式迁移策略：

1. **第一阶段**：实现 codec 模块，不修改现有代码
2. **第二阶段**：存储引擎支持双格式读写
3. **第三阶段**：提供数据迁移工具

```rust
// 格式版本检测
pub fn detect_format_version(data: &[u8]) -> FormatVersion {
    if data.is_empty() {
        return FormatVersion::Unknown;
    }
    // 检查是否为新版 Codec 格式（头部字节第4位为1）
    if (data[0] & 0x08) != 0 {
        FormatVersion::V2
    } else {
        FormatVersion::V1  // JSON 或 Bincode
    }
}
```

## 五、性能优化

### 5.1 优化策略

| 优化点 | 策略 | 预期收益 |
|--------|------|----------|
| 内存分配 | 预分配缓冲区 | 减少分配次数 |
| 字段访问 | 固定偏移量 + 直接内存访问 | O(1) 访问 |
| 字符串存储 | 偏移量 + 内容分离 | 紧凑存储 |
| 缓存解码 | 字段结果缓存 | 避免重复解码 |

### 5.2 性能对比（预期）

| 操作 | JSON | Bincode | Codec V2 |
|------|------|---------|----------|
| 单字段读取 | 100ms | 50ms | 10ms |
| 单字段写入 | 80ms | 40ms | 8ms |
| 存储空间 | 100% | 60% | 40% |
| 随机访问 | 否 | 否 | 是 |

## 六、实施计划

### 6.1 实施阶段

| 阶段 | 任务 | 优先级 |
|------|------|--------|
| 1 | 创建模块结构 | 高 |
| 2 | 实现 error.rs, row_buffer.rs | 高 |
| 3 | 实现 field_accessor.rs | 高 |
| 4 | 实现 row_writer.rs | 高 |
| 5 | 实现 row_reader.rs | 高 |
| 6 | 实现 key_utils.rs | 中 |
| 7 | 编写测试用例 | 中 |
| 8 | 集成到 storage | 高 |
| 9 | 数据迁移工具 | 低 |

### 6.2 依赖添加

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
thiserror = "1.0"
```

## 七、总结

本设计方案参考 NebulaGraph 的成熟实现，为项目设计了一个高效的 Codec 模块。主要特点：

1. **紧凑二进制格式**：相比 JSON 减少 50-60% 存储空间
2. **O(1) 字段访问**：直接内存访问，无需全量解析
3. **Schema 驱动**：编解码过程类型安全
4. **渐进式迁移**：支持平滑升级

## 参考资料

- NebulaGraph 3.8.0 `src/codec/` 目录源码
- NebulaGraph 3.8.0 `src/common/utils/NebulaKeyUtils.h` 键设计
