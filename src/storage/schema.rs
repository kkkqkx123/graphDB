//! Schema定义
//!
//! 定义了存储层的Schema结构和相关操作

use super::types::FieldDef;

use std::collections::BTreeMap;

/// Schema定义
#[derive(Debug, Clone, Default)]
pub struct Schema {
    pub name: String,
    pub fields: BTreeMap<String, FieldDef>,
    pub version: i32,
}

impl Schema {
    pub fn new(name: String, version: i32) -> Self {
        Self {
            name,
            fields: BTreeMap::new(),
            version,
        }
    }

    pub fn add_field(mut self, field: FieldDef) -> Self {
        let offset = self.fields.values()
            .map(|f| f.estimated_size())
            .sum::<usize>();
        
        let field_with_offset = FieldDef {
            offset,
            ..field
        };
        
        self.fields.insert(field_with_offset.name.clone(), field_with_offset);
        self
    }

    pub fn get_field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.get(name)
    }

    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    pub fn get_field_index(&self, name: &str) -> Option<usize> {
        self.fields.keys().position(|k| k == name)
    }

    pub fn get_field_by_index(&self, index: usize) -> Option<&FieldDef> {
        self.fields.values().nth(index)
    }

    pub fn field_name(&self, index: usize) -> Option<String> {
        self.fields.keys().nth(index).cloned()
    }

    pub fn num_fields(&self) -> usize {
        self.fields.len()
    }

    pub fn num_nullable_fields(&self) -> usize {
        self.fields.values().filter(|f| f.nullable).count()
    }

    pub fn estimated_data_size(&self) -> usize {
        self.fields.values().map(|f| f.estimated_size()).sum()
    }

    pub fn estimated_row_size(&self) -> usize {
        let header_size = 1;
        let null_bytes = if self.num_nullable_fields() > 0 {
            ((self.num_nullable_fields() - 1) >> 3) + 1
        } else {
            0
        };
        header_size + null_bytes + self.estimated_data_size() + 128
    }
}
