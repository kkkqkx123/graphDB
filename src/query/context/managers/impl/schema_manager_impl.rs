//! Schema管理器实现 - 内存中的Schema管理

use super::super::{Schema, SchemaManager};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 内存中的Schema管理器实现
#[derive(Debug, Clone)]
pub struct MemorySchemaManager {
    schemas: Arc<RwLock<HashMap<String, Schema>>>,
}

impl MemorySchemaManager {
    /// 创建新的内存Schema管理器
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加Schema
    pub fn add_schema(&self, schema: Schema) -> Result<(), String> {
        let mut schemas = self.schemas.write().map_err(|e| e.to_string())?;
        schemas.insert(schema.name.clone(), schema);
        Ok(())
    }

    /// 删除Schema
    pub fn remove_schema(&self, name: &str) -> Result<(), String> {
        let mut schemas = self.schemas.write().map_err(|e| e.to_string())?;
        schemas.remove(name);
        Ok(())
    }

    /// 更新Schema
    pub fn update_schema(&self, name: &str, schema: Schema) -> Result<(), String> {
        let mut schemas = self.schemas.write().map_err(|e| e.to_string())?;
        schemas.insert(name.to_string(), schema);
        Ok(())
    }
}

impl Default for MemorySchemaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaManager for MemorySchemaManager {
    fn get_schema(&self, name: &str) -> Option<Schema> {
        let schemas = self.schemas.read().ok()?;
        schemas.get(name).cloned()
    }

    fn list_schemas(&self) -> Vec<String> {
        match self.schemas.read() {
            Ok(schemas) => schemas.keys().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn has_schema(&self, name: &str) -> bool {
        match self.schemas.read() {
            Ok(schemas) => schemas.contains_key(name),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_schema_manager_creation() {
        let manager = MemorySchemaManager::new();
        assert!(manager.list_schemas().is_empty());
    }

    #[test]
    fn test_memory_schema_manager_add_schema() {
        let manager = MemorySchemaManager::new();
        
        let schema = Schema {
            name: "users".to_string(),
            fields: HashMap::from([
                ("id".to_string(), "int".to_string()),
                ("name".to_string(), "string".to_string()),
            ]),
            is_vertex: true,
        };
        
        assert!(manager.add_schema(schema).is_ok());
        assert!(manager.has_schema("users"));
        assert_eq!(manager.list_schemas(), vec!["users".to_string()]);
    }

    #[test]
    fn test_memory_schema_manager_get_schema() {
        let manager = MemorySchemaManager::new();
        
        let schema = Schema {
            name: "users".to_string(),
            fields: HashMap::from([
                ("id".to_string(), "int".to_string()),
                ("name".to_string(), "string".to_string()),
            ]),
            is_vertex: true,
        };
        
        manager.add_schema(schema.clone()).expect("Expected successful addition of schema");

        let retrieved = manager.get_schema("users");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.expect("Expected schema 'users' to exist").name, "users");
    }

    #[test]
    fn test_memory_schema_manager_remove_schema() {
        let manager = MemorySchemaManager::new();
        
        let schema = Schema {
            name: "users".to_string(),
            fields: HashMap::new(),
            is_vertex: true,
        };

        manager.add_schema(schema).expect("Expected successful addition of schema for removal test");
        assert!(manager.has_schema("users"));

        manager.remove_schema("users").expect("Expected successful removal of schema");
        assert!(!manager.has_schema("users"));
    }

    #[test]
    fn test_memory_schema_manager_update_schema() {
        let manager = MemorySchemaManager::new();
        
        let schema1 = Schema {
            name: "users".to_string(),
            fields: HashMap::from([("id".to_string(), "int".to_string())]),
            is_vertex: true,
        };
        
        let schema2 = Schema {
            name: "users".to_string(),
            fields: HashMap::from([
                ("id".to_string(), "int".to_string()),
                ("name".to_string(), "string".to_string()),
            ]),
            is_vertex: true,
        };
        
        manager.add_schema(schema1).expect("Expected successful addition of first schema");
        assert_eq!(manager.get_schema("users").expect("Expected schema 'users' to exist").fields.len(), 1);

        manager.update_schema("users", schema2).expect("Expected successful update of schema");
        assert_eq!(manager.get_schema("users").expect("Expected schema 'users' to exist after update").fields.len(), 2);
    }
}