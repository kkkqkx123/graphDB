use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct TagSchema {
    pub name: String,
    pub space_id: i64,
    pub tag_id: i64,
    pub properties: Vec<PropertySchema>,
}

#[derive(Debug, Clone)]
pub struct EdgeTypeSchema {
    pub name: String,
    pub space_id: i64,
    pub edge_type_id: i64,
    pub properties: Vec<PropertySchema>,
}

#[derive(Debug, Clone)]
pub struct PropertySchema {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Float,
    Double,
    String,
    Date,
    Time,
    DateTime,
    Vertex,
    Edge,
    Path,
    List,
    Set,
    Map,
}

pub struct SchemaManager {
    tags: Arc<RwLock<HashMap<i64, HashMap<i64, TagSchema>>>>,
    edge_types: Arc<RwLock<HashMap<i64, HashMap<i64, EdgeTypeSchema>>>>,
    tag_names: Arc<RwLock<HashMap<String, TagSchema>>>,
    edge_type_names: Arc<RwLock<HashMap<String, EdgeTypeSchema>>>,
}

impl SchemaManager {
    pub fn new() -> Self {
        Self {
            tags: Arc::new(RwLock::new(HashMap::new())),
            edge_types: Arc::new(RwLock::new(HashMap::new())),
            tag_names: Arc::new(RwLock::new(HashMap::new())),
            edge_type_names: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_tag(&self, tag_schema: TagSchema) -> Result<()> {
        let mut tags = self.tags.write().map_err(|e| anyhow!("获取标签写锁失败: {}", e))?;
        let space_tags = tags.entry(tag_schema.space_id).or_insert_with(HashMap::new);
        space_tags.insert(tag_schema.tag_id, tag_schema.clone());

        let mut tag_names = self
            .tag_names
            .write()
            .map_err(|e| anyhow!("获取标签名写锁失败: {}", e))?;
        tag_names.insert(tag_schema.name.clone(), tag_schema);

        Ok(())
    }

    pub fn add_edge_type(&self, edge_type_schema: EdgeTypeSchema) -> Result<()> {
        let mut edge_types = self
            .edge_types
            .write()
            .map_err(|e| anyhow!("获取边类型写锁失败: {}", e))?;
        let space_edge_types = edge_types
            .entry(edge_type_schema.space_id)
            .or_insert_with(HashMap::new);
        space_edge_types.insert(edge_type_schema.edge_type_id, edge_type_schema.clone());

        let mut edge_type_names = self
            .edge_type_names
            .write()
            .map_err(|e| anyhow!("获取边类型名写锁失败: {}", e))?;
        edge_type_names.insert(edge_type_schema.name.clone(), edge_type_schema);

        Ok(())
    }

    pub fn get_tag(&self, space_id: i64, tag_id: i64) -> Option<TagSchema> {
        let tags = self.tags.read().ok()?;
        tags.get(&space_id)?.get(&tag_id).cloned()
    }

    pub fn get_tag_by_name(&self, tag_name: &str) -> Option<TagSchema> {
        let tag_names = self.tag_names.read().ok()?;
        tag_names.get(tag_name).cloned()
    }

    pub fn get_edge_type(&self, space_id: i64, edge_type_id: i64) -> Option<EdgeTypeSchema> {
        let edge_types = self.edge_types.read().ok()?;
        edge_types.get(&space_id)?.get(&edge_type_id).cloned()
    }

    pub fn get_edge_type_by_name(&self, edge_type_name: &str) -> Option<EdgeTypeSchema> {
        let edge_type_names = self.edge_type_names.read().ok()?;
        edge_type_names.get(edge_type_name).cloned()
    }

    pub fn get_all_tags(&self, space_id: i64) -> Vec<TagSchema> {
        let tags = self.tags.read().expect("获取标签读锁失败");
        tags.get(&space_id)
            .map(|space_tags| space_tags.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_all_edge_types(&self, space_id: i64) -> Vec<EdgeTypeSchema> {
        let edge_types = self.edge_types.read().expect("获取边类型读锁失败");
        edge_types
            .get(&space_id)
            .map(|space_edge_types| space_edge_types.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn remove_tag(&self, space_id: i64, tag_id: i64) -> Result<()> {
        let mut tags = self.tags.write().map_err(|e| anyhow!("获取标签写锁失败: {}", e))?;
        if let Some(space_tags) = tags.get_mut(&space_id) {
            if let Some(tag_schema) = space_tags.remove(&tag_id) {
                let mut tag_names = self
                    .tag_names
                    .write()
                    .map_err(|e| anyhow!("获取标签名写锁失败: {}", e))?;
                tag_names.remove(&tag_schema.name);
            }
        }
        Ok(())
    }

    pub fn remove_edge_type(&self, space_id: i64, edge_type_id: i64) -> Result<()> {
        let mut edge_types = self
            .edge_types
            .write()
            .map_err(|e| anyhow!("获取边类型写锁失败: {}", e))?;
        if let Some(space_edge_types) = edge_types.get_mut(&space_id) {
            if let Some(edge_type_schema) = space_edge_types.remove(&edge_type_id) {
                let mut edge_type_names = self
                    .edge_type_names
                    .write()
                    .map_err(|e| anyhow!("获取边类型名写锁失败: {}", e))?;
                edge_type_names.remove(&edge_type_schema.name);
            }
        }
        Ok(())
    }

    pub fn tag_exists(&self, space_id: i64, tag_name: &str) -> bool {
        let tags = self.tags.read().expect("获取标签读锁失败");
        if let Some(space_tags) = tags.get(&space_id) {
            space_tags.values().any(|tag| tag.name == tag_name)
        } else {
            false
        }
    }

    pub fn edge_type_exists(&self, space_id: i64, edge_type_name: &str) -> bool {
        let edge_types = self.edge_types.read().expect("获取边类型读锁失败");
        if let Some(space_edge_types) = edge_types.get(&space_id) {
            space_edge_types
                .values()
                .any(|edge_type| edge_type.name == edge_type_name)
        } else {
            false
        }
    }
}

impl Default for SchemaManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_manager_creation() {
        let schema_manager = SchemaManager::new();
        assert_eq!(schema_manager.get_all_tags(1).len(), 0);
        assert_eq!(schema_manager.get_all_edge_types(1).len(), 0);
    }

    #[test]
    fn test_add_tag() {
        let schema_manager = SchemaManager::new();

        let tag_schema = TagSchema {
            name: "user".to_string(),
            space_id: 1,
            tag_id: 1,
            properties: vec![],
        };

        let result = schema_manager.add_tag(tag_schema.clone());
        assert!(result.is_ok());

        let retrieved_tag = schema_manager.get_tag(1, 1);
        assert_eq!(retrieved_tag, Some(tag_schema.clone()));

        let retrieved_tag_by_name = schema_manager.get_tag_by_name("user");
        assert_eq!(retrieved_tag_by_name, Some(tag_schema));
    }

    #[test]
    fn test_add_edge_type() {
        let schema_manager = SchemaManager::new();

        let edge_type_schema = EdgeTypeSchema {
            name: "follows".to_string(),
            space_id: 1,
            edge_type_id: 1,
            properties: vec![],
        };

        let result = schema_manager.add_edge_type(edge_type_schema.clone());
        assert!(result.is_ok());

        let retrieved_edge_type = schema_manager.get_edge_type(1, 1);
        assert_eq!(retrieved_edge_type, Some(edge_type_schema.clone()));

        let retrieved_edge_type_by_name = schema_manager.get_edge_type_by_name("follows");
        assert_eq!(retrieved_edge_type_by_name, Some(edge_type_schema));
    }

    #[test]
    fn test_get_all_tags() {
        let schema_manager = SchemaManager::new();

        schema_manager
            .add_tag(TagSchema {
                name: "user".to_string(),
                space_id: 1,
                tag_id: 1,
                properties: vec![],
            })
            .unwrap();

        schema_manager
            .add_tag(TagSchema {
                name: "post".to_string(),
                space_id: 1,
                tag_id: 2,
                properties: vec![],
            })
            .unwrap();

        let tags = schema_manager.get_all_tags(1);
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_remove_tag() {
        let schema_manager = SchemaManager::new();

        let tag_schema = TagSchema {
            name: "user".to_string(),
            space_id: 1,
            tag_id: 1,
            properties: vec![],
        };

        schema_manager.add_tag(tag_schema.clone()).unwrap();
        assert!(schema_manager.get_tag(1, 1).is_some());

        let result = schema_manager.remove_tag(1, 1);
        assert!(result.is_ok());
        assert!(schema_manager.get_tag(1, 1).is_none());
        assert!(schema_manager.get_tag_by_name("user").is_none());
    }

    #[test]
    fn test_tag_exists() {
        let schema_manager = SchemaManager::new();

        assert!(!schema_manager.tag_exists(1, "user"));

        schema_manager
            .add_tag(TagSchema {
                name: "user".to_string(),
                space_id: 1,
                tag_id: 1,
                properties: vec![],
            })
            .unwrap();

        assert!(schema_manager.tag_exists(1, "user"));
        assert!(!schema_manager.tag_exists(2, "user"));
    }

    #[test]
    fn test_edge_type_exists() {
        let schema_manager = SchemaManager::new();

        assert!(!schema_manager.edge_type_exists(1, "follows"));

        schema_manager
            .add_edge_type(EdgeTypeSchema {
                name: "follows".to_string(),
                space_id: 1,
                edge_type_id: 1,
                properties: vec![],
            })
            .unwrap();

        assert!(schema_manager.edge_type_exists(1, "follows"));
        assert!(!schema_manager.edge_type_exists(2, "follows"));
    }
}
