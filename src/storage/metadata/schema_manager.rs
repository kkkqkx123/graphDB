use crate::core::StorageError;
use crate::expression::storage::{FieldDef, FieldType, Schema};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::types::{EdgeTypeSchema, IndexInfo, SpaceInfo, TagInfo};

pub trait SchemaManager: Send + Sync {
    fn create_space(&self, space: &SpaceInfo) -> Result<bool, StorageError>;
    fn drop_space(&self, space_name: &str) -> Result<bool, StorageError>;
    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError>;
    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError>;

    fn create_tag(&self, space: &str, tag: &TagInfo) -> Result<bool, StorageError>;
    fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError>;
    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError>;
    fn drop_tag(&self, space: &str, tag_name: &str) -> Result<bool, StorageError>;

    fn create_edge_type(&self, space: &str, edge: &EdgeTypeSchema) -> Result<bool, StorageError>;
    fn get_edge_type(&self, space: &str, edge_type_name: &str) -> Result<Option<EdgeTypeSchema>, StorageError>;
    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeSchema>, StorageError>;
    fn drop_edge_type(&self, space: &str, edge_type_name: &str) -> Result<bool, StorageError>;

    fn get_tag_schema(&self, space: &str, tag: &str) -> Result<Schema, StorageError>;
    fn get_edge_type_schema(&self, space: &str, edge: &str) -> Result<Schema, StorageError>;
}

pub struct MemorySchemaManager {
    spaces: Arc<Mutex<HashMap<String, SpaceInfo>>>,
    tags: Arc<Mutex<HashMap<String, HashMap<String, TagInfo>>>>,
    edge_types: Arc<Mutex<HashMap<String, HashMap<String, EdgeTypeSchema>>>>,
}

impl MemorySchemaManager {
    pub fn new() -> Self {
        Self {
            spaces: Arc::new(Mutex::new(HashMap::new())),
            tags: Arc::new(Mutex::new(HashMap::new())),
            edge_types: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn data_type_to_field_type(data_type: &crate::core::DataType) -> FieldType {
        match data_type {
            crate::core::DataType::Bool => FieldType::Bool,
            crate::core::DataType::Int8 => FieldType::Int8,
            crate::core::DataType::Int16 => FieldType::Int16,
            crate::core::DataType::Int32 => FieldType::Int32,
            crate::core::DataType::Int64 => FieldType::Int64,
            crate::core::DataType::Float => FieldType::Float,
            crate::core::DataType::Double => FieldType::Double,
            crate::core::DataType::String => FieldType::String,
            crate::core::DataType::Date => FieldType::Date,
            crate::core::DataType::Time => FieldType::Time,
            crate::core::DataType::DateTime => FieldType::DateTime,
            crate::core::DataType::List => FieldType::List,
            crate::core::DataType::Map => FieldType::Map,
            crate::core::DataType::Set => FieldType::Set,
            crate::core::DataType::Geography => FieldType::Geography,
            crate::core::DataType::Duration => FieldType::Duration,
            _ => FieldType::String,
        }
    }

    fn tag_info_to_schema(tag_name: &str, tag_info: &TagInfo) -> Schema {
        let fields: Vec<FieldDef> = tag_info.properties.iter().map(|prop| {
            let field_type = Self::data_type_to_field_type(&prop.type_def);
            FieldDef {
                name: prop.name.clone(),
                field_type,
                nullable: prop.is_nullable,
                default_value: None,
                fixed_length: None,
                offset: 0,
                null_flag_pos: None,
                geo_shape: None,
            }
        }).collect();

        Schema {
            name: tag_name.to_string(),
            version: 1,
            fields: fields.into_iter().map(|f| (f.name.clone(), f)).collect(),
        }
    }

    fn edge_type_schema_to_schema(edge_type_name: &str, edge_schema: &EdgeTypeSchema) -> Schema {
        let fields: Vec<FieldDef> = edge_schema.properties.iter().map(|prop| {
            let field_type = Self::data_type_to_field_type(&prop.type_def);
            FieldDef {
                name: prop.name.clone(),
                field_type,
                nullable: prop.is_nullable,
                default_value: None,
                fixed_length: None,
                offset: 0,
                null_flag_pos: None,
                geo_shape: None,
            }
        }).collect();

        Schema {
            name: edge_type_name.to_string(),
            version: 1,
            fields: fields.into_iter().map(|f| (f.name.clone(), f)).collect(),
        }
    }
}

impl SchemaManager for MemorySchemaManager {
    fn create_space(&self, space: &SpaceInfo) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if spaces.contains_key(&space.name) {
            return Ok(false);
        }
        spaces.insert(space.name.clone(), space.clone());

        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tags.insert(space.name.clone(), HashMap::new());

        let mut edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_types.insert(space.name.clone(), HashMap::new());

        Ok(true)
    }

    fn drop_space(&self, space_name: &str) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if !spaces.contains_key(space_name) {
            return Ok(false);
        }
        spaces.remove(space_name);

        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tags.remove(space_name);

        let mut edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_types.remove(space_name);

        Ok(true)
    }

    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError> {
        let spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(spaces.get(space_name).cloned())
    }

    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError> {
        let spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        Ok(spaces.values().cloned().collect())
    }

    fn create_tag(&self, space: &str, tag: &TagInfo) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(space) {
            if space_tags.contains_key(&tag.name) {
                return Ok(false);
            }
            space_tags.insert(tag.name.clone(), tag.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError> {
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get(space) {
            Ok(space_tags.get(tag_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError> {
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get(space) {
            Ok(space_tags.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn drop_tag(&self, space: &str, tag_name: &str) -> Result<bool, StorageError> {
        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get_mut(space) {
            Ok(space_tags.remove(tag_name).is_some())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn create_edge_type(&self, space: &str, edge: &EdgeTypeSchema) -> Result<bool, StorageError> {
        let mut edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edges) = edge_types.get_mut(space) {
            if space_edges.contains_key(&edge.name) {
                return Ok(false);
            }
            space_edges.insert(edge.name.clone(), edge.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn get_edge_type(&self, space: &str, edge_type_name: &str) -> Result<Option<EdgeTypeSchema>, StorageError> {
        let edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edges) = edge_types.get(space) {
            Ok(space_edges.get(edge_type_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeSchema>, StorageError> {
        let edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edges) = edge_types.get(space) {
            Ok(space_edges.values().cloned().collect())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn drop_edge_type(&self, space: &str, edge_type_name: &str) -> Result<bool, StorageError> {
        let mut edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edges) = edge_types.get_mut(space) {
            Ok(space_edges.remove(edge_type_name).is_some())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn get_tag_schema(&self, space: &str, tag: &str) -> Result<Schema, StorageError> {
        let tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_tags) = tags.get(space) {
            if let Some(tag_info) = space_tags.get(tag) {
                Ok(Self::tag_info_to_schema(tag, tag_info))
            } else {
                Err(StorageError::DbError(format!("Tag '{}' not found in space '{}'", tag, space)))
            }
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn get_edge_type_schema(&self, space: &str, edge: &str) -> Result<Schema, StorageError> {
        let edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edges) = edge_types.get(space) {
            if let Some(edge_schema) = space_edges.get(edge) {
                Ok(Self::edge_type_schema_to_schema(edge, edge_schema))
            } else {
                Err(StorageError::DbError(format!("Edge type '{}' not found in space '{}'", edge, space)))
            }
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }
}

impl Default for MemorySchemaManager {
    fn default() -> Self {
        Self::new()
    }
}
