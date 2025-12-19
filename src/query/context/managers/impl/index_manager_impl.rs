//! 索引管理器实现 - 内存中的索引管理

use super::super::{Index, IndexManager};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 内存中的索引管理器实现
#[derive(Debug, Clone)]
pub struct MemoryIndexManager {
    indexes: Arc<RwLock<HashMap<String, Index>>>,
}

impl MemoryIndexManager {
    /// 创建新的内存索引管理器
    pub fn new() -> Self {
        Self {
            indexes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加索引
    pub fn add_index(&self, index: Index) -> Result<(), String> {
        let mut indexes = self.indexes.write().map_err(|e| e.to_string())?;
        indexes.insert(index.name.clone(), index);
        Ok(())
    }

    /// 删除索引
    pub fn remove_index(&self, name: &str) -> Result<(), String> {
        let mut indexes = self.indexes.write().map_err(|e| e.to_string())?;
        indexes.remove(name);
        Ok(())
    }

    /// 更新索引
    pub fn update_index(&self, name: &str, index: Index) -> Result<(), String> {
        let mut indexes = self.indexes.write().map_err(|e| e.to_string())?;
        indexes.insert(name.to_string(), index);
        Ok(())
    }

    /// 根据Schema名称获取索引
    pub fn get_indexes_by_schema(&self, schema_name: &str) -> Vec<Index> {
        match self.indexes.read() {
            Ok(indexes) => indexes
                .values()
                .filter(|index| index.schema_name == schema_name)
                .cloned()
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 检查字段是否被索引
    pub fn is_field_indexed(&self, schema_name: &str, field_name: &str) -> bool {
        match self.indexes.read() {
            Ok(indexes) => indexes
                .values()
                .any(|index| index.schema_name == schema_name && index.fields.contains(&field_name.to_string())),
            Err(_) => false,
        }
    }
}

impl Default for MemoryIndexManager {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexManager for MemoryIndexManager {
    fn get_index(&self, name: &str) -> Option<Index> {
        let indexes = self.indexes.read().ok()?;
        indexes.get(name).cloned()
    }

    fn list_indexes(&self) -> Vec<String> {
        match self.indexes.read() {
            Ok(indexes) => indexes.keys().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    fn has_index(&self, name: &str) -> bool {
        match self.indexes.read() {
            Ok(indexes) => indexes.contains_key(name),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_index_manager_creation() {
        let manager = MemoryIndexManager::new();
        assert!(manager.list_indexes().is_empty());
    }

    #[test]
    fn test_memory_index_manager_add_index() {
        let manager = MemoryIndexManager::new();
        
        let index = Index {
            name: "idx_users_id".to_string(),
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            is_unique: true,
        };
        
        assert!(manager.add_index(index).is_ok());
        assert!(manager.has_index("idx_users_id"));
        assert_eq!(manager.list_indexes(), vec!["idx_users_id".to_string()]);
    }

    #[test]
    fn test_memory_index_manager_get_index() {
        let manager = MemoryIndexManager::new();
        
        let index = Index {
            name: "idx_users_id".to_string(),
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            is_unique: true,
        };
        
        manager.add_index(index.clone()).unwrap();
        
        let retrieved = manager.get_index("idx_users_id");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "idx_users_id");
    }

    #[test]
    fn test_memory_index_manager_remove_index() {
        let manager = MemoryIndexManager::new();
        
        let index = Index {
            name: "idx_users_id".to_string(),
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            is_unique: true,
        };
        
        manager.add_index(index).unwrap();
        assert!(manager.has_index("idx_users_id"));
        
        manager.remove_index("idx_users_id").unwrap();
        assert!(!manager.has_index("idx_users_id"));
    }

    #[test]
    fn test_memory_index_manager_get_indexes_by_schema() {
        let manager = MemoryIndexManager::new();
        
        let index1 = Index {
            name: "idx_users_id".to_string(),
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            is_unique: true,
        };
        
        let index2 = Index {
            name: "idx_users_name".to_string(),
            schema_name: "users".to_string(),
            fields: vec!["name".to_string()],
            is_unique: false,
        };
        
        let index3 = Index {
            name: "idx_orders_id".to_string(),
            schema_name: "orders".to_string(),
            fields: vec!["id".to_string()],
            is_unique: true,
        };
        
        manager.add_index(index1).unwrap();
        manager.add_index(index2).unwrap();
        manager.add_index(index3).unwrap();
        
        let user_indexes = manager.get_indexes_by_schema("users");
        assert_eq!(user_indexes.len(), 2);
        
        let order_indexes = manager.get_indexes_by_schema("orders");
        assert_eq!(order_indexes.len(), 1);
    }

    #[test]
    fn test_memory_index_manager_is_field_indexed() {
        let manager = MemoryIndexManager::new();
        
        let index = Index {
            name: "idx_users_id".to_string(),
            schema_name: "users".to_string(),
            fields: vec!["id".to_string()],
            is_unique: true,
        };
        
        manager.add_index(index).unwrap();
        
        assert!(manager.is_field_indexed("users", "id"));
        assert!(!manager.is_field_indexed("users", "name"));
        assert!(!manager.is_field_indexed("orders", "id"));
    }
}