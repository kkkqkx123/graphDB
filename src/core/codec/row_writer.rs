//! RowWriter - 二进制编码器

use crate::storage::Schema;
use crate::storage::types::FieldDef;
use crate::core::DataType;
use crate::core::Value;
use super::row_buffer::RowBuffer;
use super::error::{CodecError, Result};

pub struct RowWriter<'a> {
    schema: &'a Schema,
    buffer: RowBuffer,
    is_set: Vec<bool>,
    finished: bool,
}

impl<'a> RowWriter<'a> {
    pub fn new(schema: &'a Schema) -> Self {
        let buffer = RowBuffer::with_capacity(schema);
        let num_fields = schema.num_fields();
        let is_set = vec![false; num_fields];

        Self {
            schema,
            buffer,
            is_set,
            finished: false,
        }
    }

    pub fn set_null(&mut self, field_name: &str) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        if !field.nullable {
            return Err(CodecError::TypeMismatch(
                format!("Field '{}' is not nullable", field_name)
            ));
        }

        let null_flag_pos = field.null_flag_pos
            .ok_or_else(|| CodecError::TypeMismatch(format!("Field '{}' has no null flag", field_name)))?;

        let byte_offset = self.buffer.header_len() + (null_flag_pos >> 3);
        let bit_mask = 0x80 >> (null_flag_pos & 0x07);
        self.buffer[byte_offset] |= bit_mask;

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_bool(&mut self, field_name: &str, value: bool) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;

        self.buffer.write_bool(offset, value);

        if field.nullable {
            self.clear_null_bit(field.null_flag_pos.unwrap());
        }

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_int8(&mut self, field_name: &str, value: i8) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;

        self.buffer.write_int8(offset, value);

        if field.nullable {
            self.clear_null_bit(field.null_flag_pos.unwrap());
        }

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_int16(&mut self, field_name: &str, value: i16) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;

        self.buffer.write_int16(offset, value);

        if field.nullable {
            self.clear_null_bit(field.null_flag_pos.unwrap());
        }

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_int32(&mut self, field_name: &str, value: i32) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;

        self.buffer.write_int32(offset, value);

        if field.nullable {
            self.clear_null_bit(field.null_flag_pos.unwrap());
        }

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_int64(&mut self, field_name: &str, value: i64) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;

        self.buffer.write_int64(offset, value);

        if field.nullable {
            self.clear_null_bit(field.null_flag_pos.unwrap());
        }

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_float(&mut self, field_name: &str, value: f32) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;

        self.buffer.write_float(offset, value);

        if field.nullable {
            self.clear_null_bit(field.null_flag_pos.unwrap());
        }

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_double(&mut self, field_name: &str, value: f64) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;

        self.buffer.write_double(offset, value);

        if field.nullable {
            self.clear_null_bit(field.null_flag_pos.unwrap());
        }

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_string(&mut self, field_name: &str, value: &str) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        match field.field_type {
            DataType::String => {
                let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;
                let str_offset = self.buffer.append_string_content(value.as_bytes()) as u32;
                let str_len = value.len() as u32;

                self.buffer.write_string_offset(offset, str_offset, str_len);
            }
            DataType::FixedString(len) => {
                let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;
                self.buffer.write_fixed_string(offset, value, len);
            }
            _ => {
                return Err(CodecError::TypeMismatch(
                    format!("Field '{}' is not a string type", field_name)
                ));
            }
        }

        if field.nullable {
            self.clear_null_bit(field.null_flag_pos.unwrap());
        }

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_timestamp(&mut self, field_name: &str, value: i64) -> Result<()> {
        self.set_int64(field_name, value)
    }

    pub fn set_date(&mut self, field_name: &str, year: i32, month: u32, day: u32) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;

        self.buffer.write_date(offset, year, month, day);

        if field.nullable {
            self.clear_null_bit(field.null_flag_pos.unwrap());
        }

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_time(&mut self, field_name: &str, hour: u32, minute: u32, sec: u32, microsec: u32) -> Result<()> {
        self.check_not_finished();

        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        let offset = self.buffer.data_start() + self.buffer.null_bytes() + field.offset;

        self.buffer.write_time(offset, hour, minute, sec, microsec);

        if field.nullable {
            self.clear_null_bit(field.null_flag_pos.unwrap());
        }

        self.is_set[field_index] = true;
        Ok(())
    }

    pub fn set_value(&mut self, field_name: &str, value: &Value) -> Result<()> {
        match value {
            Value::Null(_) => self.set_null(field_name),
            Value::Bool(v) => self.set_bool(field_name, *v),
            Value::Int(v) => {
                if *v >= i8::MIN as i64 && *v <= i8::MAX as i64 {
                    self.set_int8(field_name, *v as i8)
                } else if *v >= i16::MIN as i64 && *v <= i16::MAX as i64 {
                    self.set_int16(field_name, *v as i16)
                } else if *v >= i32::MIN as i64 && *v <= i32::MAX as i64 {
                    self.set_int32(field_name, *v as i32)
                } else {
                    self.set_int64(field_name, *v)
                }
            }
            Value::Float(v) => {
                let abs = v.abs();
                if abs <= f32::MAX as f64 && abs >= f32::MIN_POSITIVE as f64 {
                    self.set_float(field_name, *v as f32)
                } else {
                    self.set_double(field_name, *v)
                }
            }
            Value::String(s) => self.set_string(field_name, s),
            Value::Date(d) => self.set_date(field_name, d.year, d.month, d.day),
            Value::Time(t) => self.set_time(field_name, t.hour, t.minute, t.sec, t.microsec),
            Value::DateTime(dt) => {
                let offset = self.buffer.data_start() + self.buffer.null_bytes() + {
                    let field = self.schema.get_field_by_index(
                        self.schema.get_field_index(field_name).unwrap()
                    ).unwrap();
                    field.offset
                };
                self.buffer.write_date(offset, dt.year, dt.month, dt.day);
                self.buffer.write_time(offset + 4, dt.hour, dt.minute, dt.sec, dt.microsec);
                self.is_set[self.schema.get_field_index(field_name).unwrap()] = true;
                Ok(())
            }
            _ => Err(CodecError::UnsupportedDataType(format!("{:?}", value))),
        }
    }

    pub fn finish(mut self) -> Result<Vec<u8>> {
        self.finished = true;

        for (index, is_set) in self.is_set.iter().enumerate() {
            if !is_set {
                let field = match self.schema.get_field_by_index(index) {
                    Some(f) => f,
                    None => continue,
                };
                if !field.nullable && field.default_value.is_none() {
                    return Err(CodecError::TypeMismatch(
                        format!("Required field '{}' not set", field.name)
                    ));
                }
            }
        }

        Ok(self.buffer.into_inner())
    }

    fn check_not_finished(&self) {
        assert!(!self.finished, "Cannot write to finished RowWriter");
    }

    fn clear_null_bit(&mut self, pos: usize) {
        let byte_offset = self.buffer.header_len() + (pos >> 3);
        let and_mask = !(0x80 >> (pos & 0x07));
        self.buffer[byte_offset] &= and_mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::types::FieldDef;

    #[test]
    fn test_row_writer_basic() {
        let mut schema = Schema::new("test".to_string(), 1);
        schema = schema.add_field(FieldDef::new("age".to_string(), DataType::Int64));
        schema = schema.add_field(FieldDef::new("name".to_string(), DataType::String));

        let mut writer = RowWriter::new(&schema);
        writer.set_int64("age", 25).unwrap();
        writer.set_string("name", "test").unwrap();

        let data = writer.finish().unwrap();

        assert!(!data.is_empty());
        assert_eq!(data[0], 0x08);
    }
}
