use std::collections::{HashMap, BTreeSet};
use crate::core::{Value, Node, Direction};

#[derive(Debug)]
pub enum IndexError {
    IndexCreationError(String),
    IndexUpdateError(String),
}

/// Index for node labels
pub struct LabelIndex {
    indices: HashMap<String, BTreeSet<u64>>, // label -> node IDs
}

impl LabelIndex {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    pub fn insert(&mut self, label: String, node_id: u64) -> Result<(), IndexError> {
        self.indices
            .entry(label)
            .or_insert_with(BTreeSet::new)
            .insert(node_id);
        Ok(())
    }

    pub fn remove(&mut self, label: &str, node_id: u64) -> Result<(), IndexError> {
        if let Some(node_set) = self.indices.get_mut(label) {
            node_set.remove(&node_id);
            // Remove the label entry if it becomes empty
            if node_set.is_empty() {
                self.indices.remove(label);
            }
        }
        Ok(())
    }

    pub fn get_nodes_by_label(&self, label: &str) -> Option<&BTreeSet<u64>> {
        self.indices.get(label)
    }
}

/// Index for node properties
pub struct PropertyIndex {
    indices: HashMap<String, HashMap<Value, BTreeSet<u64>>>, // property_name -> (value -> node IDs)
}

impl PropertyIndex {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    pub fn insert(&mut self, property_name: String, value: Value, node_id: u64) -> Result<(), IndexError> {
        self.indices
            .entry(property_name)
            .or_insert_with(HashMap::new)
            .entry(value)
            .or_insert_with(BTreeSet::new)
            .insert(node_id);
        Ok(())
    }

    pub fn remove(&mut self, property_name: &str, value: &Value, node_id: u64) -> Result<(), IndexError> {
        if let Some(property_map) = self.indices.get_mut(property_name) {
            if let Some(node_set) = property_map.get_mut(value) {
                node_set.remove(&node_id);
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

    pub fn get_nodes_by_property(&self, property_name: &str, value: &Value) -> Option<&BTreeSet<u64>> {
        self.indices
            .get(property_name)
            .and_then(|property_map| property_map.get(value))
    }
}

/// Composite index for multiple properties
pub struct CompositeIndex {
    indices: HashMap<String, HashMap<String, BTreeSet<u64>>>, // property_name -> (value_str -> node IDs)
}

impl CompositeIndex {
    pub fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    /// This is a simplified approach that converts values to string representation
    pub fn insert(&mut self, property_name: String, value: Value, node_id: u64) -> Result<(), IndexError> {
        let value_str = value_to_string(&value);
        self.indices
            .entry(property_name)
            .or_insert_with(HashMap::new)
            .entry(value_str)
            .or_insert_with(BTreeSet::new)
            .insert(node_id);
        Ok(())
    }

    pub fn remove(&mut self, property_name: &str, value: &Value, node_id: u64) -> Result<(), IndexError> {
        let value_str = value_to_string(value);
        if let Some(property_map) = self.indices.get_mut(property_name) {
            if let Some(node_set) = property_map.get_mut(&value_str) {
                node_set.remove(&node_id);
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

    pub fn get_nodes_by_property(&self, property_name: &str, value: &Value) -> Option<&BTreeSet<u64>> {
        let value_str = value_to_string(value);
        self.indices
            .get(property_name)
            .and_then(|property_map| property_map.get(&value_str))
    }
}

/// Helper function to convert Value to string for indexing
fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::List(_) => "list".to_string(), // Simplified for indexing
        Value::Map(_) => "map".to_string(),   // Simplified for indexing
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

    pub fn update_indexes_for_node(&mut self, node: &Node) -> Result<(), IndexError> {
        // Update label index
        for label in &node.labels {
            self.label_index.insert(label.clone(), node.id)?;
        }

        // Update property index
        for (key, value) in &node.properties {
            self.property_index.insert(key.clone(), value.clone(), node.id)?;
            self.composite_index.insert(key.clone(), value.clone(), node.id)?;
        }

        Ok(())
    }

    pub fn remove_indexes_for_node(&mut self, node: &Node) -> Result<(), IndexError> {
        // Remove from label index
        for label in &node.labels {
            self.label_index.remove(label, node.id)?;
        }

        // Remove from property index
        for (key, value) in &node.properties {
            self.property_index.remove(key, value, node.id)?;
            self.composite_index.remove(key, value, node.id)?;
        }

        Ok(())
    }

    pub fn get_nodes_by_label(&self, label: &str) -> Option<&BTreeSet<u64>> {
        self.label_index.get_nodes_by_label(label)
    }

    pub fn get_nodes_by_property(&self, property_name: &str, value: &Value) -> Option<&BTreeSet<u64>> {
        self.property_index.get_nodes_by_property(property_name, value)
    }
}