//! Codec 模块 - 二进制编解码
//!
//! 提供高效的二进制数据编解码功能，参考 NebulaGraph 3.8.0 实现
//!
//! ## 架构
//!
//! ```
//! ┌─────────────────────────────────────┐
//! │            codec::mod.rs            │
//! │       模块入口和公共类型导出         │
//! └─────────────────────────────────────┘
//!              │
//!    ┌─────────┼─────────┬──────────┐
//!    ▼         ▼         ▼          ▼
//! ┌──────┐ ┌────────┐ ┌─────────┐ ┌──────┐
//! │error │ │row_buf │ │field_   │ │row_  │
//! │      │ │        │ │accessor │ │writer│
//! └──────┘ └────────┘ └─────────┘ └──────┘
//!                    │
//!              ┌─────┴─────┐
//!              ▼           ▼
//!          ┌────────┐ ┌─────────┐
//!          │key_    │ │row_     │
//!          │utils   │ │reader   │
//!          └────────┘ └─────────┘
//! ```
//!
//! ## 二进制格式
//!
//! RowWriterV2 采用紧凑的二进制格式：
//! - 头部：1字节（固定值 0x08）
//! - NULL标记：位图存储可空字段
//! - 数据区：固定长度字段
//! - 字符串内容：变长字符串内容存储在末尾
//!
//! ## 使用示例
//!
//! ```ignore
//! use crate::core::codec::{RowWriter, RowReader, KeyUtils};
//!
//! // 编码
//! let mut writer = RowWriter::new(&schema);
//! writer.set_int64("age", 25).unwrap();
//! writer.set_string("name", "test").unwrap();
//! let data = writer.finish().unwrap();
//!
//! // 解码
//! let reader = RowReader::new(&data, &schema).unwrap();
//! let age = reader.get_int64("age").unwrap();
//! ```

pub mod error;
pub mod row_buffer;
pub mod field_accessor;
pub mod row_writer;
pub mod row_reader;
pub mod key_utils;

pub use error::{CodecError, Result, CodecResult};
pub use row_writer::RowWriter;
pub use row_reader::RowReader;
pub use key_utils::KeyUtils;

use crate::storage::Schema;
use crate::storage::types::FieldDef;
use crate::core::DataType;

pub enum FormatVersion {
    V1,
    V2,
    Unknown,
}

pub fn detect_format_version(data: &[u8]) -> FormatVersion {
    if data.is_empty() {
        return FormatVersion::Unknown;
    }
    if (data[0] & 0x08) != 0 {
        FormatVersion::V2
    } else {
        FormatVersion::V1
    }
}

pub fn create_row_writer<'a>(schema: &'a Schema) -> RowWriter<'a> {
    RowWriter::new(schema)
}

pub fn create_row_reader<'a>(data: &'a [u8], schema: &'a Schema) -> Result<RowReader<'a>> {
    RowReader::new(data, schema)
}

pub fn estimate_row_size(schema: &Schema) -> usize {
    let header_size = 1;
    let data_size = schema.estimated_data_size();
    let null_size = if schema.num_nullable_fields() > 0 {
        ((schema.num_nullable_fields() - 1) >> 3) + 1
    } else {
        0
    };
    header_size + null_size + data_size + 128
}
