//! FieldAccessor - 字段访问器

use crate::storage::Schema;
use crate::core::Value;
use crate::core::value::{DateValue, TimeValue, DateTimeValue};
use super::error::{CodecError, Result};

pub struct FieldAccessor<'a> {
    data: &'a [u8],
    schema: &'a Schema,
    header_len: usize,
    null_bytes: usize,
}

impl<'a> FieldAccessor<'a> {
    pub fn new(data: &'a [u8], schema: &'a Schema) -> Result<Self> {
        if data.is_empty() {
            return Err(CodecError::InvalidData("Empty row data".to_string()));
        }

        let ver_bytes = (data[0] & 0x07) as usize;
        let header_len = ver_bytes + 1;

        if data.len() < header_len {
            return Err(CodecError::InvalidData("Data too short for header".to_string()));
        }

        let num_nullables = schema.num_nullable_fields();
        let null_bytes = if num_nullables > 0 {
            ((num_nullables - 1) >> 3) + 1
        } else {
            0
        };

        Ok(Self {
            data,
            schema,
            header_len,
            null_bytes,
        })
    }

    pub fn header_len(&self) -> usize {
        self.header_len
    }

    pub fn null_bytes(&self) -> usize {
        self.null_bytes
    }

    pub fn data_start(&self) -> usize {
        self.header_len + self.null_bytes
    }

    pub fn is_null(&self, field_index: usize) -> bool {
        let field = match self.schema.get_field_by_index(field_index) {
            Some(f) => f,
            None => return false,
        };

        if !field.nullable {
            return false;
        }

        let null_flag_pos = match field.null_flag_pos {
            Some(pos) => pos,
            None => return false,
        };

        let byte_offset = self.header_len + (null_flag_pos >> 3);
        let bit_mask = 0x80 >> (null_flag_pos & 0x07);
        (self.data[byte_offset] & bit_mask) != 0
    }

    pub fn get_bool(&self, field_index: usize) -> Result<bool> {
        let offset = self.get_field_offset(field_index)?;
        Ok(self.data[offset] != 0)
    }

    pub fn get_int8(&self, field_index: usize) -> Result<i8> {
        let offset = self.get_field_offset(field_index)?;
        Ok(self.data[offset] as i8)
    }

    pub fn get_int16(&self, field_index: usize) -> Result<i16> {
        let offset = self.get_field_offset(field_index)?;
        let bytes: [u8; 2] = self.data[offset..offset + 2].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read int16".to_string())
        })?;
        Ok(i16::from_le_bytes(bytes))
    }

    pub fn get_int32(&self, field_index: usize) -> Result<i32> {
        let offset = self.get_field_offset(field_index)?;
        let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read int32".to_string())
        })?;
        Ok(i32::from_le_bytes(bytes))
    }

    pub fn get_int64(&self, field_index: usize) -> Result<i64> {
        let offset = self.get_field_offset(field_index)?;
        let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read int64".to_string())
        })?;
        Ok(i64::from_le_bytes(bytes))
    }

    pub fn get_float(&self, field_index: usize) -> Result<f32> {
        let offset = self.get_field_offset(field_index)?;
        let bytes: [u8; 4] = self.data[offset..offset + 4].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read float".to_string())
        })?;
        Ok(f32::from_le_bytes(bytes))
    }

    pub fn get_double(&self, field_index: usize) -> Result<f64> {
        let offset = self.get_field_offset(field_index)?;
        let bytes: [u8; 8] = self.data[offset..offset + 8].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read double".to_string())
        })?;
        Ok(f64::from_le_bytes(bytes))
    }

    pub fn get_string(&self, field_index: usize) -> Result<String> {
        let offset = self.get_field_offset(field_index)?;

        let str_offset = u32::from_le_bytes(self.data[offset..offset + 4].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read string offset".to_string())
        })?) as usize;
        let str_len = u32::from_le_bytes(self.data[offset + 4..offset + 8].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read string length".to_string())
        })?) as usize;

        if str_offset == self.data.len() && str_len == 0 {
            return Ok(String::new());
        }

        if str_offset + str_len > self.data.len() {
            return Err(CodecError::InvalidData("String offset out of bounds".to_string()));
        }

        std::str::from_utf8(&self.data[str_offset..str_offset + str_len])
            .map(|s| s.to_string())
            .map_err(|e| CodecError::EncodingError(e.to_string()))
    }

    pub fn get_fixed_string(&self, field_index: usize, len: usize) -> Result<String> {
        let offset = self.get_field_offset(field_index)?;
        let end = std::cmp::min(offset + len, self.data.len());
        let slice = &self.data[offset..end];

        let null_pos = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
        std::str::from_utf8(&slice[..null_pos])
            .map(|s| s.to_string())
            .map_err(|e| CodecError::EncodingError(e.to_string()))
    }

    pub fn get_timestamp(&self, field_index: usize) -> Result<i64> {
        self.get_int64(field_index)
    }

    pub fn get_date(&self, field_index: usize) -> Result<DateValue> {
        let offset = self.get_field_offset(field_index)?;

        let year = i16::from_le_bytes(self.data[offset..offset + 2].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read date year".to_string())
        })?) as i32;
        let month = self.data[offset + 2] as u32;
        let day = self.data[offset + 3] as u32;

        Ok(DateValue { year, month, day })
    }

    pub fn get_time(&self, field_index: usize) -> Result<TimeValue> {
        let offset = self.get_field_offset(field_index)?;

        let hour = self.data[offset] as u32;
        let minute = self.data[offset + 1] as u32;
        let sec = self.data[offset + 2] as u32;
        let microsec = u32::from_le_bytes(self.data[offset + 4..offset + 8].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read time microsec".to_string())
        })?);

        Ok(TimeValue { hour, minute, sec, microsec })
    }

    pub fn get_datetime(&self, field_index: usize) -> Result<DateTimeValue> {
        let offset = self.get_field_offset(field_index)?;

        let year = i16::from_le_bytes(self.data[offset..offset + 2].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read datetime year".to_string())
        })?) as i32;
        let month = self.data[offset + 2] as u32;
        let day = self.data[offset + 3] as u32;
        let hour = self.data[offset + 4] as u32;
        let minute = self.data[offset + 5] as u32;
        let sec = self.data[offset + 6] as u32;
        let microsec = u32::from_le_bytes(self.data[offset + 7..offset + 11].try_into().map_err(|_| {
            CodecError::InvalidData("Failed to read datetime microsec".to_string())
        })?);

        Ok(DateTimeValue {
            year,
            month,
            day,
            hour,
            minute,
            sec,
            microsec,
        })
    }

    fn get_field_offset(&self, field_index: usize) -> Result<usize> {
        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(format!("Field at index {}", field_index)))?;

        let offset = self.header_len + self.null_bytes + field.offset;
        Ok(offset)
    }
}
