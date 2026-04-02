//! 标签系统
//!
//! 为文档添加标签，支持基于标签的过滤和搜索
//!
//! # 示例
//!
//! ```rust
//! use inversearch::{DocId, TagSystem, TagConfig};
//!
//! let mut tag_system = TagSystem::new();
//! tag_system.add_config("category".to_string(), None);
//!
//! // 添加标签
//! tag_system.add_tags(1, &[("category", &json!("tech"))]);
//!
//! // 按标签查询
//! let ids = tag_system.query("category", "tech");
//! ```

use serde_json::Value;
use crate::DocId;
use std::collections::HashMap;

/// 标签配置
pub struct TagConfig {
    pub field: String,
    pub filter: Option<Box<dyn Fn(&Value) -> bool + Send + Sync>>,
}

impl TagConfig {
    /// 创建新的标签配置
    pub fn new(field: &str) -> Self {
        TagConfig {
            field: field.to_string(),
            filter: None,
        }
    }

    /// 添加过滤器
    pub fn with_filter<F>(mut self, filter: F) -> Self 
    where 
        F: Fn(&Value) -> bool + 'static + Send + Sync,
    {
        self.filter = Some(Box::new(filter));
        self
    }
}

/// 标签系统
pub struct TagSystem {
    configs: Vec<TagConfig>,
    indexes: Vec<HashMap<String, Vec<DocId>>>,
    doc_tags: HashMap<DocId, Vec<(usize, String)>>,
}

impl TagSystem {
    /// 创建新的标签系统
    pub fn new() -> Self {
        TagSystem {
            configs: Vec::new(),
            indexes: Vec::new(),
            doc_tags: HashMap::new(),
        }
    }

    /// 添加标签配置
    pub fn add_config(&mut self, field: &str, config: Option<TagConfig>) {
        let config = config.unwrap_or_else(|| TagConfig::new(field));
        self.configs.push(config);
        self.indexes.push(HashMap::new());
    }

    /// 添加标签配置（简化版）
    pub fn add_config_str(&mut self, field: String, filter: Option<Box<dyn Fn(&Value) -> bool + Send + Sync>>) {
        let config = TagConfig {
            field,
            filter,
        };
        self.configs.push(config);
        self.indexes.push(HashMap::new());
    }

    /// 获取所有配置字段名
    pub fn config_fields(&self) -> Vec<&str> {
        self.configs.iter().map(|c| c.field.as_str()).collect()
    }

    /// 为文档添加标签
    pub fn add_tags(&mut self, doc_id: DocId, tags: &[(&str, &Value)]) {
        let mut doc_tag_list = Vec::new();
        
        for (field, value) in tags {
            if let Some(idx) = self.configs.iter().position(|c| c.field == *field) {
                let config = &self.configs[idx];
                if let Some(ref filter) = config.filter {
                    if !filter(value) {
                        continue;
                    }
                }
                
                let tag_str = value.as_str().unwrap_or_default();
                if let Some(index) = self.indexes.get_mut(idx) {
                    let ids = index.entry(tag_str.to_string()).or_default();
                    if !ids.contains(&doc_id) {
                        ids.push(doc_id);
                    }
                }
                doc_tag_list.push((idx, tag_str.to_string()));
            }
        }
        
        if !doc_tag_list.is_empty() {
            self.doc_tags.insert(doc_id, doc_tag_list);
        }
    }

    /// 移除文档的标签
    pub fn remove_tags(&mut self, doc_id: DocId) {
        if let Some(tags) = self.doc_tags.remove(&doc_id) {
            for (idx, tag) in tags {
                if let Some(index) = self.indexes.get_mut(idx) {
                    if let Some(ids) = index.get_mut(&tag) {
                        if let Some(pos) = ids.iter().position(|&id| id == doc_id) {
                            ids.swap_remove(pos);
                        }
                    }
                }
            }
        }
    }

    /// 按标签查询文档
    pub fn query(&self, field: &str, tag: &str) -> Option<&Vec<DocId>> {
        let idx = self.configs.iter()
            .position(|c| c.field == field)?;
        self.indexes[idx].get(tag)
    }

    /// 按多个标签查询（交集）
    pub fn query_multi(&self, field: &str, tags: &[&str]) -> Vec<DocId> {
        let idx = match self.configs.iter().position(|c| c.field == field) {
            Some(i) => i,
            None => return Vec::new(),
        };
        
        let mut result: Option<Vec<DocId>> = None;
        for tag in tags {
            if let Some(ids) = self.indexes[idx].get(*tag) {
                if let Some(ref mut combined) = result {
                    let set: std::collections::HashSet<&DocId> = combined.iter().collect();
                    *combined = ids.iter()
                        .filter(|id| set.contains(id))
                        .copied()
                        .collect();
                } else {
                    result = Some(ids.clone());
                }
            }
        }
        
        result.unwrap_or_default()
    }

    /// 按多个标签查询（并集）
    pub fn query_any(&self, field: &str, tags: &[&str]) -> Vec<DocId> {
        let idx = match self.configs.iter().position(|c| c.field == field) {
            Some(i) => i,
            None => return Vec::new(),
        };
        
        let mut result = std::collections::HashSet::new();
        for tag in tags {
            if let Some(ids) = self.indexes[idx].get(*tag) {
                result.extend(ids);
            }
        }
        
        result.into_iter().collect()
    }

    /// 获取文档的所有标签
    pub fn get_doc_tags(&self, doc_id: DocId) -> Option<&Vec<(usize, String)>> {
        self.doc_tags.get(&doc_id)
    }

    /// 清空所有标签
    pub fn clear(&mut self) {
        for index in &mut self.indexes {
            index.clear();
        }
        self.doc_tags.clear();
    }

    /// 获取配置数量
    pub fn len(&self) -> usize {
        self.configs.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.configs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_add_and_query() {
        let mut tag_system = TagSystem::new();
        tag_system.add_config_str("category".to_string(), None);
        
        tag_system.add_tags(1, &[("category", &json!("tech"))]);
        tag_system.add_tags(2, &[("category", &json!("science"))]);
        
        let tech_docs = tag_system.query("category", "tech");
        assert_eq!(tech_docs, Some(&vec![1]));
        
        let science_docs = tag_system.query("category", "science");
        assert_eq!(science_docs, Some(&vec![2]));
    }

    #[test]
    fn test_query_multi() {
        let mut tag_system = TagSystem::new();
        tag_system.add_config_str("status".to_string(), None);
        
        tag_system.add_tags(1, &[("status", &json!("active"))]);
        tag_system.add_tags(2, &[("status", &json!("active"))]);
        tag_system.add_tags(3, &[("status", &json!("inactive"))]);
        
        let result = tag_system.query_multi("status", &["active"]);
        assert!(result.contains(&1));
        assert!(result.contains(&2));
        assert!(!result.contains(&3));
    }

    #[test]
    fn test_remove_tags() {
        let mut tag_system = TagSystem::new();
        tag_system.add_config_str("category".to_string(), None);
        
        tag_system.add_tags(1, &[("category", &json!("tech"))]);
        assert!(tag_system.query("category", "tech").is_some());
        
        tag_system.remove_tags(1);
        assert!(tag_system.query("category", "tech").is_none() || tag_system.query("category", "tech").map(|v| v.is_empty()).unwrap_or(true));
    }

    #[test]
    fn test_with_filter() {
        let mut tag_system = TagSystem::new();
        tag_system.add_config_str("category".to_string(), Some(Box::new(|v| v != &json!("banned"))));
        
        tag_system.add_tags(1, &[("category", &json!("tech"))]);
        tag_system.add_tags(2, &[("category", &json!("banned"))]);
        
        assert!(tag_system.query("category", "tech").is_some());
        assert!(tag_system.query("category", "banned").map(|v| v.is_empty()).unwrap_or(true));
    }
}
