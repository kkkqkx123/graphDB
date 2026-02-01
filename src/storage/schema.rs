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
        self.fields.insert(field.name.clone(), field);
        self
    }

    pub fn get_field(&self, name: &str) -> Option<&FieldDef> {
        self.fields.get(name)
    }

    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }
}
