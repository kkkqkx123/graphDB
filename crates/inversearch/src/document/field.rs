//! Field Definitions
//!
//! Define the configuration and operation of document fields

use crate::document::tree::{extract_value, parse_tree, TreePath};
use crate::index::IndexOptions;
use crate::Index;
use crate::{DocId, EncoderOptions};
use serde_json::Value;
use std::collections::HashMap;

type FieldFilterFn = Box<dyn Fn(&Value) -> bool + Send + Sync>;

/// Field type
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Bool,
    Array,
    Object,
}

/// Field Configuration
pub struct FieldConfig {
    pub name: String,
    pub field_type: FieldType,
    pub extract: Vec<TreePath>,
    pub encoder: Option<EncoderOptions>,
    pub filter: Option<FieldFilterFn>,
    pub boost: Option<i32>,
}

impl Default for FieldConfig {
    fn default() -> Self {
        FieldConfig {
            name: String::new(),
            field_type: FieldType::String,
            extract: Vec::new(),
            encoder: None,
            filter: None,
            boost: None,
        }
    }
}

impl FieldConfig {
    /// Creating a new field configuration
    pub fn new(name: &str) -> Self {
        let mut marker = vec![];
        let extract = parse_tree(name, &mut marker);

        FieldConfig {
            name: name.to_string(),
            field_type: FieldType::String,
            extract,
            encoder: None,
            filter: None,
            boost: None,
        }
    }

    /// Setting the field type
    pub fn with_type(mut self, field_type: FieldType) -> Self {
        self.field_type = field_type;
        self
    }

    /// Setting Encoder Options
    pub fn with_encoder(mut self, encoder: EncoderOptions) -> Self {
        self.encoder = Some(encoder);
        self
    }

    /// Setting up filters
    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Fn(&Value) -> bool + 'static + Send + Sync,
    {
        self.filter = Some(Box::new(filter));
        self
    }

    /// weights
    pub fn with_boost(mut self, boost: i32) -> Self {
        self.boost = Some(boost);
        self
    }

    /// Extracting field values from documents
    pub fn extract_value(&self, document: &Value) -> Option<String> {
        extract_value(document, &self.extract)
    }
}

/// Examples of fields
pub struct Field {
    config: FieldConfig,
    index: Index,
}

impl Field {
    /// Creating a new field instance
    pub fn new(config: FieldConfig) -> Result<Self, crate::error::InversearchError> {
        let index_options = IndexOptions {
            encoder: config.encoder.clone(),
            fastupdate: Some(false),
            ..Default::default()
        };

        let index = Index::new(index_options)?;

        Ok(Field { config, index })
    }

    /// Get field name
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// Getting field weights
    pub fn boost(&self) -> Option<i32> {
        self.config.boost
    }

    /// Adding Documents to a Field Index
    pub fn add(
        &mut self,
        id: DocId,
        document: &Value,
    ) -> Result<(), crate::error::InversearchError> {
        if let Some(value) = self.config.extract_value(document) {
            if let Some(ref filter) = self.config.filter {
                if !filter(document) {
                    return Ok(());
                }
            }
            self.index.add(id, &value, false)?;
        }
        Ok(())
    }

    /// Removing documents from a field index
    pub fn remove(&mut self, id: DocId) -> Result<(), crate::error::InversearchError> {
        self.index.remove(id, false)?;
        Ok(())
    }

    /// Clearing field indexes
    pub fn clear(&mut self) {
        self.index.clear();
    }

    /// Get internal index reference (for search coordinator)
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// Get variable internal index reference
    pub fn index_mut(&mut self) -> &mut Index {
        &mut self.index
    }
}

/// set of fields
pub struct Fields {
    fields: Vec<Field>,
    name_to_index: HashMap<String, usize>,
}

impl Fields {
    /// Creating a new collection of fields
    pub fn new() -> Self {
        Fields {
            fields: Vec::new(),
            name_to_index: HashMap::new(),
        }
    }

    /// Adding Fields
    pub fn add(&mut self, field: Field) {
        let name = field.name().to_string();
        self.name_to_index.insert(name.clone(), self.fields.len());
        self.fields.push(field);
    }

    /// Get fields by name
    pub fn get(&self, name: &str) -> Option<&Field> {
        self.name_to_index.get(name).map(|&idx| &self.fields[idx])
    }

    /// Get variable fields by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Field> {
        self.name_to_index
            .get(name)
            .map(|&idx| &mut self.fields[idx])
    }

    /// Get all fields
    pub fn all(&self) -> &[Field] {
        &self.fields
    }

    /// Get the number of fields
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Check if it is empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Clear all fields
    pub fn clear(&mut self) {
        self.fields.clear();
        self.name_to_index.clear();
    }

    /// Iterate over all fields
    pub fn iter(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter()
    }

    /// (math.) variable iteration
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Field> {
        self.fields.iter_mut()
    }
}

impl Default for Fields {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_field_config_new() {
        let config = FieldConfig::new("title");
        assert_eq!(config.name, "title");
        assert!(!config.extract.is_empty());
    }

    #[test]
    fn test_field_add() {
        let config = FieldConfig::new("user.name");
        let mut field = Field::new(config).unwrap();

        let doc = json!({"user": {"name": "John"}});
        field.add(1, &doc).unwrap();

        assert!(field.index.contains(1));
    }

    #[test]
    fn test_field_with_filter() {
        let config =
            FieldConfig::new("status").with_filter(|v| v.get("status") == Some(&json!("active")));

        let mut field = Field::new(config).unwrap();

        let active_doc = json!({"status": "active", "name": "Active"});
        let inactive_doc = json!({"status": "inactive", "name": "Inactive"});

        field.add(1, &active_doc).unwrap();
        field.add(2, &inactive_doc).unwrap();

        assert!(field.index.contains(1));
        assert!(!field.index.contains(2));
    }

    #[test]
    fn test_fields_collection() {
        let mut fields = Fields::new();

        let title_field = Field::new(FieldConfig::new("title")).unwrap();
        let content_field = Field::new(FieldConfig::new("content")).unwrap();

        fields.add(title_field);
        fields.add(content_field);

        assert_eq!(fields.len(), 2);
        assert!(fields.get("title").is_some());
        assert!(fields.get("content").is_some());
        assert!(fields.get("missing").is_none());
    }
}
