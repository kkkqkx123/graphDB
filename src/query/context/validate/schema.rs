//! Schema管理模块
//! 提供Schema相关的数据结构和管理功能

use std::collections::HashMap;

/// Schema提供者trait（简化版）
pub trait SchemaProvider: Send + Sync {
    fn get_schema(&self, name: &str) -> Option<SchemaInfo>;
    fn list_schemas(&self) -> Vec<String>;
}

/// Schema信息
#[derive(Debug, Clone)]
pub struct SchemaInfo {
    pub name: String,
    pub fields: HashMap<String, String>, // 字段名 -> 类型
    pub is_vertex: bool,
}

impl SchemaInfo {
    /// 创建新的Schema信息
    pub fn new(name: String, is_vertex: bool) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            is_vertex,
        }
    }

    /// 添加字段
    pub fn add_field(&mut self, name: String, type_: String) {
        self.fields.insert(name, type_);
    }

    /// 获取字段类型
    pub fn get_field_type(&self, name: &str) -> Option<&String> {
        self.fields.get(name)
    }

    /// 检查字段是否存在
    pub fn has_field(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    /// 获取所有字段名
    pub fn get_field_names(&self) -> Vec<String> {
        self.fields.keys().cloned().collect()
    }

    /// 验证字段类型是否匹配
    pub fn validate_field_type(&self, name: &str, expected_type: &str) -> bool {
        self.fields.get(name).map_or(false, |t| t == expected_type)
    }
}

/// Schema管理器
#[derive(Debug, Clone)]
pub struct SchemaManager {
    schemas: HashMap<String, SchemaInfo>,
}

impl SchemaManager {
    /// 创建新的Schema管理器
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    /// 添加Schema
    pub fn add_schema(&mut self, schema: SchemaInfo) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// 获取Schema
    pub fn get_schema(&self, name: &str) -> Option<&SchemaInfo> {
        self.schemas.get(name)
    }

    /// 列出所有Schema名称
    pub fn list_schemas(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }

    /// 检查Schema是否存在
    pub fn has_schema(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }

    /// 移除Schema
    pub fn remove_schema(&mut self, name: &str) -> Option<SchemaInfo> {
        self.schemas.remove(name)
    }

    /// 获取所有顶点Schema
    pub fn get_vertex_schemas(&self) -> Vec<&SchemaInfo> {
        self.schemas
            .values()
            .filter(|s| s.is_vertex)
            .collect()
    }

    /// 获取所有边Schema
    pub fn get_edge_schemas(&self) -> Vec<&SchemaInfo> {
        self.schemas
            .values()
            .filter(|s| !s.is_vertex)
            .collect()
    }
}

impl Default for SchemaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaProvider for SchemaManager {
    fn get_schema(&self, name: &str) -> Option<SchemaInfo> {
        self.schemas.get(name).cloned()
    }

    fn list_schemas(&self) -> Vec<String> {
        self.list_schemas()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_info_creation() {
        let mut schema = SchemaInfo::new("person".to_string(), true);
        
        schema.add_field("id".to_string(), "INT".to_string());
        schema.add_field("name".to_string(), "STRING".to_string());
        
        assert_eq!(schema.name, "person");
        assert!(schema.is_vertex);
        assert_eq!(schema.fields.len(), 2);
        assert!(schema.has_field("id"));
        assert!(schema.has_field("name"));
        assert!(!schema.has_field("age"));
        assert_eq!(schema.get_field_type("id"), Some(&"INT".to_string()));
    }

    #[test]
    fn test_schema_info_field_validation() {
        let mut schema = SchemaInfo::new("person".to_string(), true);
        
        schema.add_field("id".to_string(), "INT".to_string());
        schema.add_field("name".to_string(), "STRING".to_string());
        
        assert!(schema.validate_field_type("id", "INT"));
        assert!(!schema.validate_field_type("id", "STRING"));
        assert!(!schema.validate_field_type("age", "INT"));
    }

    #[test]
    fn test_schema_manager() {
        let mut manager = SchemaManager::new();
        
        // 添加顶点Schema
        let mut person_schema = SchemaInfo::new("person".to_string(), true);
        person_schema.add_field("id".to_string(), "INT".to_string());
        person_schema.add_field("name".to_string(), "STRING".to_string());
        manager.add_schema(person_schema);
        
        // 添加边Schema
        let mut knows_schema = SchemaInfo::new("knows".to_string(), false);
        knows_schema.add_field("since".to_string(), "DATETIME".to_string());
        knows_schema.add_field("weight".to_string(), "DOUBLE".to_string());
        manager.add_schema(knows_schema);
        
        // 测试基本功能
        assert!(manager.has_schema("person"));
        assert!(manager.has_schema("knows"));
        assert!(!manager.has_schema("company"));
        
        let schemas = manager.list_schemas();
        assert_eq!(schemas.len(), 2);
        assert!(schemas.contains(&"person".to_string()));
        assert!(schemas.contains(&"knows".to_string()));
        
        // 测试顶点和边Schema分离
        let vertex_schemas = manager.get_vertex_schemas();
        assert_eq!(vertex_schemas.len(), 1);
        assert_eq!(vertex_schemas[0].name, "person");
        
        let edge_schemas = manager.get_edge_schemas();
        assert_eq!(edge_schemas.len(), 1);
        assert_eq!(edge_schemas[0].name, "knows");
    }

    #[test]
    fn test_schema_provider_trait() {
        let mut manager = SchemaManager::new();
        
        let mut schema = SchemaInfo::new("test".to_string(), true);
        schema.add_field("id".to_string(), "INT".to_string());
        manager.add_schema(schema);
        
        // 测试trait方法
        let provider: &dyn SchemaProvider = &manager;
        assert!(provider.get_schema("test").is_some());
        assert!(provider.get_schema("nonexistent").is_none());
        
        let schemas = provider.list_schemas();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0], "test");
    }

    #[test]
    fn test_schema_manager_remove() {
        let mut manager = SchemaManager::new();
        
        let schema = SchemaInfo::new("test".to_string(), true);
        manager.add_schema(schema);
        
        assert!(manager.has_schema("test"));
        
        let removed = manager.remove_schema("test");
        assert!(removed.is_some());
        assert!(!manager.has_schema("test"));
        
        let removed_again = manager.remove_schema("test");
        assert!(removed_again.is_none());
    }
}