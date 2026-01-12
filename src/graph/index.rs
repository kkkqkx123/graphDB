use crate::core::{Value, Vertex};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// 索引错误类型
///
/// 涵盖索引创建和更新过程中的错误
#[derive(Error, Debug)]
pub enum IndexError {
    #[error("索引创建错误: {0}")]
    IndexCreationError(String),
    #[error("索引更新错误: {0}")]
    IndexUpdateError(String),
}

/// Index for node labels
pub struct LabelIndex {
    indices: HashMap<String, HashSet<Value>>, // tag name -> node IDs
}

impl LabelIndex {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    pub fn insert(&mut self, tag_name: String, node_id: Value) -> Result<(), IndexError> {
        self.indices
            .entry(tag_name)
            .or_insert_with(HashSet::new)
            .insert(node_id);
        Ok(())
    }

    pub fn remove(&mut self, tag_name: &str, node_id: &Value) -> Result<(), IndexError> {
        if let Some(node_set) = self.indices.get_mut(tag_name) {
            node_set.remove(node_id);
            // Remove the tag entry if it becomes empty
            if node_set.is_empty() {
                self.indices.remove(tag_name);
            }
        }
        Ok(())
    }

    pub fn get_nodes_by_label(&self, label: &str) -> Option<&HashSet<Value>> {
        self.indices.get(label)
    }
}

/// Index for node properties
pub struct PropertyIndex {
    indices: HashMap<String, HashMap<Value, HashSet<Value>>>, // property_name -> (value -> node IDs)
}

impl PropertyIndex {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        property_name: String,
        value: Value,
        node_id: Value,
    ) -> Result<(), IndexError> {
        self.indices
            .entry(property_name)
            .or_insert_with(HashMap::new)
            .entry(value)
            .or_insert_with(HashSet::new)
            .insert(node_id);
        Ok(())
    }

    pub fn remove(
        &mut self,
        property_name: &str,
        value: &Value,
        node_id: &Value,
    ) -> Result<(), IndexError> {
        if let Some(property_map) = self.indices.get_mut(property_name) {
            if let Some(node_set) = property_map.get_mut(value) {
                node_set.remove(node_id);
                // Remove the value entry if it becomes empty
                if node_set.is_empty() {
                    property_map.remove(value);
                }
                // Remove the property entry if it becomes empty
                if property_map.is_empty() {
                    self.indices.remove(property_name);
                }
            }
        }
        Ok(())
    }

    pub fn get_nodes_by_property(
        &self,
        property_name: &str,
        value: &Value,
    ) -> Option<&HashSet<Value>> {
        self.indices
            .get(property_name)
            .and_then(|property_map| property_map.get(value))
    }
}

/// Composite index for multiple properties
pub struct CompositeIndex {
    indices: HashMap<String, HashMap<String, HashSet<Value>>>, // property_name -> (value_str -> node IDs)
}

impl CompositeIndex {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    /// This is a simplified approach that converts values to string representation
    pub fn insert(
        &mut self,
        property_name: String,
        value: Value,
        node_id: Value,
    ) -> Result<(), IndexError> {
        let value_str = value_to_string(&value);
        self.indices
            .entry(property_name)
            .or_insert_with(HashMap::new)
            .entry(value_str)
            .or_insert_with(HashSet::new)
            .insert(node_id);
        Ok(())
    }

    pub fn remove(
        &mut self,
        property_name: &str,
        value: &Value,
        node_id: &Value,
    ) -> Result<(), IndexError> {
        let value_str = value_to_string(value);
        if let Some(property_map) = self.indices.get_mut(property_name) {
            if let Some(node_set) = property_map.get_mut(&value_str) {
                node_set.remove(node_id);
                // Remove the value entry if it becomes empty
                if node_set.is_empty() {
                    property_map.remove(&value_str);
                }
                // Remove the property entry if it becomes empty
                if property_map.is_empty() {
                    self.indices.remove(property_name);
                }
            }
        }
        Ok(())
    }

    pub fn get_nodes_by_property(
        &self,
        property_name: &str,
        value: &Value,
    ) -> Option<&HashSet<Value>> {
        let value_str = value_to_string(value);
        self.indices
            .get(property_name)
            .and_then(|property_map| property_map.get(&value_str))
    }
}

/// Helper function to convert Value to string for indexing
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Empty => "empty".to_string(),
        Value::Null(_) => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Date(d) => format!("{}-{}-{}", d.year, d.month, d.day),
        Value::Time(t) => format!("{}:{}:{}", t.hour, t.minute, t.sec),
        Value::DateTime(dt) => format!(
            "{}-{}-{} {}:{}:{}",
            dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.sec
        ),
        Value::Vertex(_) => "vertex".to_string(), // Simplified for indexing
        Value::Edge(_) => "edge".to_string(),     // Simplified for indexing
        Value::Path(_) => "path".to_string(),     // Simplified for indexing
        Value::List(_) => "list".to_string(),     // Simplified for indexing
        Value::Map(_) => "map".to_string(),       // Simplified for indexing
        Value::Set(_) => "set".to_string(),       // Simplified for indexing
        Value::Geography(_) => "geography".to_string(), // Simplified for indexing
        Value::Duration(_) => "duration".to_string(), // Simplified for indexing
        Value::DataSet(_) => "dataset".to_string(), // Simplified for indexing
    }
}

pub struct IndexManager {
    label_index: LabelIndex,
    property_index: PropertyIndex,
    composite_index: CompositeIndex,
}

impl IndexManager {
    pub fn new() -> Self {
        Self {
            label_index: LabelIndex::new(),
            property_index: PropertyIndex::new(),
            composite_index: CompositeIndex::new(),
        }
    }

    pub fn update_indexes_for_node(&mut self, vertex: &Vertex) -> Result<(), IndexError> {
        // Update tag index
        for tag in &vertex.tags {
            self.label_index
                .insert(tag.name.clone(), (*vertex.vid).clone())?;

            // Update property index for each tag's properties
            for (key, value) in &tag.properties {
                self.property_index
                    .insert(key.clone(), value.clone(), (*vertex.vid).clone())?;
                self.composite_index
                    .insert(key.clone(), value.clone(), (*vertex.vid).clone())?;
            }
        }

        Ok(())
    }

    pub fn remove_indexes_for_node(&mut self, vertex: &Vertex) -> Result<(), IndexError> {
        // Remove from tag index
        for tag in &vertex.tags {
            self.label_index.remove(&tag.name, &(*vertex.vid))?;

            // Remove from property index for each tag's properties
            for (key, value) in &tag.properties {
                self.property_index.remove(key, value, &(*vertex.vid))?;
                self.composite_index.remove(key, value, &(*vertex.vid))?;
            }
        }

        Ok(())
    }

    pub fn get_nodes_by_label(&self, label: &str) -> Option<&HashSet<Value>> {
        self.label_index.get_nodes_by_label(label)
    }

    pub fn get_nodes_by_property(
        &self,
        property_name: &str,
        value: &Value,
    ) -> Option<&HashSet<Value>> {
        self.property_index
            .get_nodes_by_property(property_name, value)
    }
}
