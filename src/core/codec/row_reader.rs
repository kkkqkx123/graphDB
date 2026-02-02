//! RowReader - 二进制解码器

use std::sync::RwLock;
use std::collections::HashMap;
use crate::storage::Schema;
use crate::core::DataType;
use crate::core::Value;
use crate::core::value::{DateValue, TimeValue, DateTimeValue};
use super::field_accessor::FieldAccessor;
use super::error::{CodecError, Result};

pub struct RowReader<'a> {
    data: &'a [u8],
    schema: &'a Schema,
    accessor: FieldAccessor<'a>,
    cache: RwLock<HashMap<usize, Value>>,
}

impl<'a> RowReader<'a> {
    pub fn new(data: &'a [u8], schema: &'a Schema) -> Result<Self> {
        let accessor = FieldAccessor::new(data, schema)?;
        let cache = RwLock::new(HashMap::new());

        Ok(Self {
            data,
            schema,
            accessor,
            cache,
        })
    }

    pub fn get_value(&self, field_name: &str) -> Result<Value> {
        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;

        self.get_value_by_index(field_index)
    }

    pub fn get_value_by_index(&self, index: usize) -> Result<Value> {
        if let Some(cached) = self.cache.read().unwrap().get(&index) {
            return Ok(cached.clone());
        }

        let value = self.decode_field(index)?;
        self.cache.write().unwrap().insert(index, value.clone());

        Ok(value)
    }

    pub fn is_null(&self, field_name: &str) -> Result<bool> {
        let field_index = self.schema.get_field_index(field_name)
            .ok_or_else(|| CodecError::FieldNotFound(field_name.to_string()))?;
        Ok(self.accessor.is_null(field_index))
    }

    pub fn get_bool(&self, field_name: &str) -> Result<bool> {
        let value = self.get_value(field_name)?;
        match value {
            Value::Bool(b) => Ok(b),
            _ => Err(CodecError::TypeMismatch("Not a bool".to_string())),
        }
    }

    pub fn get_int64(&self, field_name: &str) -> Result<i64> {
        let value = self.get_value(field_name)?;
        match value {
            Value::Int(i) => Ok(i),
            _ => Err(CodecError::TypeMismatch("Not an int".to_string())),
        }
    }

    pub fn get_float(&self, field_name: &str) -> Result<f64> {
        let value = self.get_value(field_name)?;
        match value {
            Value::Float(f) => Ok(f),
            _ => Err(CodecError::TypeMismatch("Not a float".to_string())),
        }
    }

    pub fn get_string(&self, field_name: &str) -> Result<String> {
        let value = self.get_value(field_name)?;
        match value {
            Value::String(s) => Ok(s),
            _ => Err(CodecError::TypeMismatch("Not a string".to_string())),
        }
    }

    pub fn schema(&self) -> &Schema {
        self.schema
    }

    pub fn data(&self) -> &[u8] {
        self.data
    }

    fn decode_field(&self, field_index: usize) -> Result<Value> {
        if self.accessor.is_null(field_index) {
            return Ok(Value::Null(crate::core::value::NullType::Null));
        }

        let field = self.schema.get_field_by_index(field_index)
            .ok_or_else(|| CodecError::FieldNotFound(format!("Field at index {}", field_index)))?;

        match field.field_type {
            DataType::Bool => {
                Ok(Value::Bool(self.accessor.get_bool(field_index)?))
            }
            DataType::Int8 => {
                let v = self.accessor.get_int8(field_index)?;
                Ok(Value::Int(v as i64))
            }
            DataType::Int16 => {
                let v = self.accessor.get_int16(field_index)?;
                Ok(Value::Int(v as i64))
            }
            DataType::Int32 => {
                let v = self.accessor.get_int32(field_index)?;
                Ok(Value::Int(v as i64))
            }
            DataType::Int64 => {
                let v = self.accessor.get_int64(field_index)?;
                Ok(Value::Int(v))
            }
            DataType::Float => {
                let v = self.accessor.get_float(field_index)?;
                Ok(Value::Float(v as f64))
            }
            DataType::Double => {
                let v = self.accessor.get_double(field_index)?;
                Ok(Value::Float(v))
            }
            DataType::String => {
                let s = self.accessor.get_string(field_index)?;
                Ok(Value::String(s))
            }
            DataType::FixedString(len) => {
                let s = self.accessor.get_fixed_string(field_index, len)?;
                Ok(Value::String(s))
            }
            DataType::Timestamp => {
                let ts = self.accessor.get_timestamp(field_index)?;
                Ok(Value::Int(ts))
            }
            DataType::Date => {
                let date = self.accessor.get_date(field_index)?;
                Ok(Value::Date(date))
            }
            DataType::Time => {
                let time = self.accessor.get_time(field_index)?;
                Ok(Value::Time(time))
            }
            DataType::DateTime => {
                let dt = self.accessor.get_datetime(field_index)?;
                Ok(Value::DateTime(dt))
            }
            DataType::VID => {
                let v = self.accessor.get_fixed_string(field_index, 8)?;
                Ok(Value::String(v))
            }
            _ => Err(CodecError::UnsupportedDataType(format!("{:?}", field.field_type))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::types::FieldDef;
    use super::super::RowWriter;

    #[test]
    fn test_row_reader_basic() {
        let mut schema = Schema::new("test".to_string(), 1);
        schema = schema.add_field(FieldDef::new("age".to_string(), DataType::Int64));
        schema = schema.add_field(FieldDef::new("name".to_string(), DataType::String));

        let mut writer = RowWriter::new(&schema);
        writer.set_int64("age", 25).unwrap();
        writer.set_string("name", "test").unwrap();
        let data = writer.finish().unwrap();

        let reader = RowReader::new(&data, &schema).unwrap();

        let age = reader.get_int64("age").unwrap();
        assert_eq!(age, 25);

        let name = reader.get_string("name").unwrap();
        assert_eq!(name, "test");
    }
}
