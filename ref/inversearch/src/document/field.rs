//! 字段定义
//!
//! 定义文档字段的配置和操作

use crate::{Encoder, EncoderOptions, DocId};
use crate::index::IndexOptions;
use crate::Index;
use crate::document::tree::{parse_tree, TreePath, extract_value};
use serde_json::Value;
use std::collections::HashMap;

/// 字段类型
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Bool,
    Array,
    Object,
}

/// 字段配置
pub struct FieldConfig {
    pub name: String,
    pub field_type: FieldType,
    pub extract: Vec<TreePath>,
    pub encoder: Option<EncoderOptions>,
    pub filter: Option<Box<dyn Fn(&Value) -> bool + Send + Sync>>,
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
    /// 创建新的字段配置
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

    /// 设置字段类型
    pub fn with_type(mut self, field_type: FieldType) -> Self {
        self.field_type = field_type;
        self
    }

    /// 设置编码器选项
    pub fn with_encoder(mut self, encoder: EncoderOptions) -> Self {
        self.encoder = Some(encoder);
        self
    }

    /// 设置过滤器
    pub fn with_filter<F>(mut self, filter: F) -> Self 
    where 
        F: Fn(&Value) -> bool + 'static + Send + Sync,
    {
        self.filter = Some(Box::new(filter));
        self
    }

    /// 设置权重
    pub fn with_boost(mut self, boost: i32) -> Self {
        self.boost = Some(boost);
        self
    }

    /// 从文档中提取字段值
    pub fn extract_value(&self, document: &Value) -> Option<String> {
        extract_value(document, &self.extract)
    }
}

/// 字段实例
pub struct Field {
    config: FieldConfig,
    index: Index,
}

impl Field {
    /// 创建新的字段实例
    pub fn new(mut config: FieldConfig) -> Result<Self, crate::error::InversearchError> {
        let index_options = IndexOptions {
            encoder: config.encoder.clone(),
            fastupdate: Some(false),
            ..Default::default()
        };
        
        let index = Index::new(index_options)?;
        
        Ok(Field {
            config,
            index,
        })
    }

    /// 获取字段名
    pub fn name(&self) -> &str {
        &self.config.name
    }

    /// 获取字段权重
    pub fn boost(&self) -> Option<i32> {
        self.config.boost
    }

    /// 添加文档到字段索引
    pub fn add(&mut self, id: DocId, document: &Value) -> Result<(), crate::error::InversearchError> {
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

    /// 从字段索引移除文档
    pub fn remove(&mut self, id: DocId) -> Result<(), crate::error::InversearchError> {
        self.index.remove(id, false)?;
        Ok(())
    }

    /// 清空字段索引
    pub fn clear(&mut self) {
        self.index.clear();
    }

    /// 获取内部索引引用（用于搜索协调器）
    pub fn index(&self) -> &Index {
        &self.index
    }

    /// 获取可变内部索引引用
    pub fn index_mut(&mut self) -> &mut Index {
        &mut self.index
    }
}

/// 字段集合
pub struct Fields {
    fields: Vec<Field>,
    name_to_index: HashMap<String, usize>,
}

impl Fields {
    /// 创建新的字段集合
    pub fn new() -> Self {
        Fields {
            fields: Vec::new(),
            name_to_index: HashMap::new(),
        }
    }

    /// 添加字段
    pub fn add(&mut self, field: Field) {
        let name = field.name().to_string();
        self.name_to_index.insert(name.clone(), self.fields.len());
        self.fields.push(field);
    }

    /// 按名称获取字段
    pub fn get(&self, name: &str) -> Option<&Field> {
        self.name_to_index.get(name).map(|&idx| &self.fields[idx])
    }

    /// 按名称获取可变字段
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Field> {
        self.name_to_index.get(name).map(|&idx| &mut self.fields[idx])
    }

    /// 获取所有字段
    pub fn all(&self) -> &[Field] {
        &self.fields
    }

    /// 获取字段数量
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// 清空所有字段
    pub fn clear(&mut self) {
        self.fields.clear();
        self.name_to_index.clear();
    }

    /// 迭代所有字段
    pub fn iter(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter()
    }

    /// 可变迭代
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
        let config = FieldConfig::new("status")
            .with_filter(|v| v.get("status") == Some(&json!("active")));
        
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
