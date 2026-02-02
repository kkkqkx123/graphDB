use crate::core::StorageError;
use crate::core::types::{
    EdgeTypeInfo, SpaceInfo, TagInfo,
};
use crate::storage::{FieldDef, Schema};
use crate::storage::utils::{tag_info_to_schema, edge_type_info_to_schema};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub trait SchemaManager: Send + Sync + std::fmt::Debug {
    fn create_space(&self, space: &SpaceInfo) -> Result<bool, StorageError>;
    fn drop_space(&self, space_name: &str) -> Result<bool, StorageError>;
    fn get_space(&self, space_name: &str) -> Result<Option<SpaceInfo>, StorageError>;
    fn list_spaces(&self) -> Result<Vec<SpaceInfo>, StorageError>;

    fn create_tag(&self, space: &str, tag: &TagInfo) -> Result<bool, StorageError>;
    fn get_tag(&self, space: &str, tag_name: &str) -> Result<Option<TagInfo>, StorageError>;
    fn list_tags(&self, space: &str) -> Result<Vec<TagInfo>, StorageError>;
    fn drop_tag(&self, space: &str, tag_name: &str) -> Result<bool, StorageError>;

    fn create_edge_type(&self, space: &str, edge: &EdgeTypeInfo) -> Result<bool, StorageError>;
    fn get_edge_type(&self, space: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError>;
    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError>;
    fn drop_edge_type(&self, space: &str, edge_type_name: &str) -> Result<bool, StorageError>;

    fn get_tag_schema(&self, space: &str, tag: &str) -> Result<Schema, StorageError>;
    fn get_edge_type_schema(&self, space: &str, edge: &str) -> Result<Schema, StorageError>;
}

pub struct MemorySchemaManager {
    spaces: Arc<Mutex<HashMap<String, SpaceInfo>>>,
    tags: Arc<Mutex<HashMap<String, HashMap<String, TagInfo>>>>,
    edge_types: Arc<Mutex<HashMap<String, HashMap<String, EdgeTypeInfo>>>>,
}

impl std::fmt::Debug for MemorySchemaManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemorySchemaManager")
            .field("spaces_count", &self.spaces.lock().map(|s| s.len()).unwrap_or(0))
            .field("tags_count", &self.tags.lock().map(|t| t.len()).unwrap_or(0))
            .field("edge_types_count", &self.edge_types.lock().map(|e| e.len()).unwrap_or(0))
            .finish()
    }
}

impl MemorySchemaManager {
    pub fn new() -> Self {
        Self {
            spaces: Arc::new(Mutex::new(HashMap::new())),
            tags: Arc::new(Mutex::new(HashMap::new())),
            edge_types: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn tag_info_to_schema(tag_name: &str, tag_info: &TagInfo) -> Schema {
        tag_info_to_schema(tag_name, tag_info)
    }

    fn edge_type_info_to_schema(edge_type_name: &str, edge_info: &EdgeTypeInfo) -> Schema {
        edge_type_info_to_schema(edge_type_name, edge_info)
    }
}

impl SchemaManager for MemorySchemaManager {
    fn create_space(&self, space: &SpaceInfo) -> Result<bool, StorageError> {
        let mut spaces = self.spaces.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if spaces.contains_key(&space.space_name) {
            return Ok(false);
        }
        spaces.insert(space.space_name.clone(), space.clone());

        let mut tags = self.tags.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        tags.insert(space.space_name.clone(), HashMap::new());

        let mut edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        edge_types.insert(space.space_name.clone(), HashMap::new());

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
            if space_tags.contains_key(&tag.tag_name) {
                return Ok(false);
            }
            space_tags.insert(tag.tag_name.clone(), tag.clone());
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

    fn create_edge_type(&self, space: &str, edge: &EdgeTypeInfo) -> Result<bool, StorageError> {
        let mut edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edges) = edge_types.get_mut(space) {
            if space_edges.contains_key(&edge.edge_type_name) {
                return Ok(false);
            }
            space_edges.insert(edge.edge_type_name.clone(), edge.clone());
            Ok(true)
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn get_edge_type(&self, space: &str, edge_type_name: &str) -> Result<Option<EdgeTypeInfo>, StorageError> {
        let edge_types = self.edge_types.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        if let Some(space_edges) = edge_types.get(space) {
            Ok(space_edges.get(edge_type_name).cloned())
        } else {
            Err(StorageError::DbError(format!("Space '{}' not found", space)))
        }
    }

    fn list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>, StorageError> {
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
            if let Some(edge_info) = space_edges.get(edge) {
                Ok(Self::edge_type_info_to_schema(edge, edge_info))
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
