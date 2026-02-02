//! RowBuffer - 二进制缓冲区管理

use crate::storage::Schema;
use crate::storage::types::FieldDef;
use crate::core::DataType;
use super::error::{CodecError, Result};

pub struct RowBuffer {
    buffer: Vec<u8>,
    header_len: usize,
    data_start: usize,
    str_content_start: usize,
    null_bytes: usize,
    schema: Schema,
}

impl RowBuffer {
    pub fn with_capacity(schema: &Schema) -> Self {
        let mut buffer = Vec::with_capacity(schema.estimated_row_size());

        buffer.push(0x08);

        let header_len = 1;

        let num_nullables = schema.num_nullable_fields();
        let null_bytes = if num_nullables > 0 {
            ((num_nullables - 1) >> 3) + 1
        } else {
            0
        };

        buffer.resize(header_len + null_bytes + schema.estimated_data_size(), 0);

        let data_start = header_len + null_bytes;
        let str_content_start = header_len + null_bytes + schema.estimated_data_size();

        Self {
            buffer,
            header_len,
            data_start,
            str_content_start,
            null_bytes,
            schema: schema.clone(),
        }
    }

    pub fn header_len(&self) -> usize {
        self.header_len
    }

    pub fn null_bytes(&self) -> usize {
        self.null_bytes
    }

    pub fn data_start(&self) -> usize {
        self.data_start
    }

    pub fn write_bool(&mut self, offset: usize, value: bool) {
        self.buffer[offset] = if value { 0x01 } else { 0x00 };
    }

    pub fn write_int8(&mut self, offset: usize, value: i8) {
        self.buffer[offset] = value as u8;
    }

    pub fn write_int16(&mut self, offset: usize, value: i16) {
        let bytes = value.to_le_bytes();
        self.buffer[offset..offset + 2].copy_from_slice(&bytes);
    }

    pub fn write_int32(&mut self, offset: usize, value: i32) {
        let bytes = value.to_le_bytes();
        self.buffer[offset..offset + 4].copy_from_slice(&bytes);
    }

    pub fn write_int64(&mut self, offset: usize, value: i64) {
        let bytes = value.to_le_bytes();
        self.buffer[offset..offset + 8].copy_from_slice(&bytes);
    }

    pub fn write_float(&mut self, offset: usize, value: f32) {
        let bytes = value.to_le_bytes();
        self.buffer[offset..offset + 4].copy_from_slice(&bytes);
    }

    pub fn write_double(&mut self, offset: usize, value: f64) {
        let bytes = value.to_le_bytes();
        self.buffer[offset..offset + 8].copy_from_slice(&bytes);
    }

    pub fn write_string_offset(&mut self, offset: usize, str_offset: u32, str_len: u32) {
        self.buffer[offset..offset + 4].copy_from_slice(&str_offset.to_le_bytes());
        self.buffer[offset + 4..offset + 8].copy_from_slice(&str_len.to_le_bytes());
    }

    pub fn append_string_content(&mut self, content: &[u8]) -> usize {
        let start = self.str_content_start;
        self.buffer.extend_from_slice(content);
        self.str_content_start = self.buffer.len();
        start
    }

    pub fn write_fixed_string(&mut self, offset: usize, value: &str, len: usize) {
        let write_len = std::cmp::min(value.len(), len);
        self.buffer[offset..offset + write_len].copy_from_slice(&value.as_bytes()[..write_len]);
        if write_len < len {
            self.buffer[offset + write_len..offset + len].fill(b'\0');
        }
    }

    pub fn write_date(&mut self, offset: usize, year: i32, month: u32, day: u32) {
        self.buffer[offset..offset + 2].copy_from_slice(&(year as i16).to_le_bytes());
        self.buffer[offset + 2] = month as u8;
        self.buffer[offset + 3] = day as u8;
    }

    pub fn write_time(&mut self, offset: usize, hour: u32, minute: u32, sec: u32, microsec: u32) {
        self.buffer[offset] = hour as u8;
        self.buffer[offset + 1] = minute as u8;
        self.buffer[offset + 2] = sec as u8;
        self.buffer[offset + 3] = 0;
        self.buffer[offset + 4..offset + 8].copy_from_slice(&microsec.to_le_bytes());
    }

    pub fn write_timestamp(&mut self, offset: usize, timestamp: i64) {
        let bytes = timestamp.to_le_bytes();
        self.buffer[offset..offset + 8].copy_from_slice(&bytes);
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buffer
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl std::ops::Index<usize> for RowBuffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buffer[index]
    }
}

impl std::ops::IndexMut<usize> for RowBuffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}

impl std::ops::Index<std::ops::Range<usize>> for RowBuffer {
    type Output = [u8];

    fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
        &self.buffer[index]
    }
}

impl std::ops::IndexMut<std::ops::Range<usize>> for RowBuffer {
    fn index_mut(&mut self, index: std::ops::Range<usize>) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_buffer_basic() {
        let mut schema = Schema::new("test".to_string(), 1);
        schema = schema.add_field(FieldDef::new("age".to_string(), DataType::Int64));
        schema = schema.add_field(FieldDef::new("name".to_string(), DataType::String));

        let mut buffer = RowBuffer::with_capacity(&schema);

        buffer.write_int64(3, 25);

        assert_eq!(buffer[3], 25);
        assert_eq!(buffer[4], 0);
        assert_eq!(buffer[5], 0);
        assert_eq!(buffer[6], 0);
        assert_eq!(buffer[7], 0);
        assert_eq!(buffer[8], 0);
        assert_eq!(buffer[9], 0);
        assert_eq!(buffer[10], 0);
    }
}
