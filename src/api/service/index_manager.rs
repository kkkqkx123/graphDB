use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IndexType {
    Tag,
    Edge,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TagIndex {
    pub index_id: i64,
    pub space_id: i64,
    pub tag_name: String,
    pub index_name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeIndex {
    pub index_id: i64,
    pub space_id: i64,
    pub edge_type_name: String,
    pub index_name: String,
    pub fields: Vec<String>,
}

pub struct IndexManager {
    tag_indexes: Arc<RwLock<HashMap<i64, HashMap<i64, TagIndex>>>>,
    edge_indexes: Arc<RwLock<HashMap<i64, HashMap<i64, EdgeIndex>>>>,
    tag_index_names: Arc<RwLock<HashMap<String, TagIndex>>>,
    edge_index_names: Arc<RwLock<HashMap<String, EdgeIndex>>>,
}

impl IndexManager {
    pub fn new() -> Self {
        Self {
            tag_indexes: Arc::new(RwLock::new(HashMap::new())),
            edge_indexes: Arc::new(RwLock::new(HashMap::new())),
            tag_index_names: Arc::new(RwLock::new(HashMap::new())),
            edge_index_names: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_tag_index(&self, index: TagIndex) -> Result<()> {
        let mut tag_indexes = self
            .tag_indexes
            .write()
            .map_err(|e| anyhow!("获取标签索引写锁失败: {}", e))?;
        let space_indexes = tag_indexes.entry(index.space_id).or_insert_with(HashMap::new);
        space_indexes.insert(index.index_id, index.clone());

        let mut tag_index_names = self
            .tag_index_names
            .write()
            .map_err(|e| anyhow!("获取标签索引名写锁失败: {}", e))?;
        tag_index_names.insert(index.index_name.clone(), index);

        Ok(())
    }

    pub fn add_edge_index(&self, index: EdgeIndex) -> Result<()> {
        let mut edge_indexes = self
            .edge_indexes
            .write()
            .map_err(|e| anyhow!("获取边索引写锁失败: {}", e))?;
        let space_indexes = edge_indexes.entry(index.space_id).or_insert_with(HashMap::new);
        space_indexes.insert(index.index_id, index.clone());

        let mut edge_index_names = self
            .edge_index_names
            .write()
            .map_err(|e| anyhow!("获取边索引名写锁失败: {}", e))?;
        edge_index_names.insert(index.index_name.clone(), index);

        Ok(())
    }

    pub fn get_tag_index(&self, space_id: i64, index_id: i64) -> Option<TagIndex> {
        let tag_indexes = self.tag_indexes.read().ok()?;
        tag_indexes.get(&space_id)?.get(&index_id).cloned()
    }

    pub fn get_tag_index_by_name(&self, index_name: &str) -> Option<TagIndex> {
        let tag_index_names = self.tag_index_names.read().ok()?;
        tag_index_names.get(index_name).cloned()
    }

    pub fn get_edge_index(&self, space_id: i64, index_id: i64) -> Option<EdgeIndex> {
        let edge_indexes = self.edge_indexes.read().ok()?;
        edge_indexes.get(&space_id)?.get(&index_id).cloned()
    }

    pub fn get_edge_index_by_name(&self, index_name: &str) -> Option<EdgeIndex> {
        let edge_index_names = self.edge_index_names.read().ok()?;
        edge_index_names.get(index_name).cloned()
    }

    pub fn get_all_tag_indexes(&self, space_id: i64) -> Vec<TagIndex> {
        let tag_indexes = self.tag_indexes.read().expect("获取标签索引读锁失败");
        let mut indexes: Vec<TagIndex> = tag_indexes
            .get(&space_id)
            .map(|indexes| indexes.values().cloned().collect())
            .unwrap_or_default();
        indexes.sort_by(|a, b| a.index_id.cmp(&b.index_id));
        indexes
    }

    pub fn get_all_edge_indexes(&self, space_id: i64) -> Vec<EdgeIndex> {
        let edge_indexes = self.edge_indexes.read().expect("获取边索引读锁失败");
        edge_indexes
            .get(&space_id)
            .map(|indexes| indexes.values().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_tag_indexes_for_tag(&self, space_id: i64, tag_name: &str) -> Vec<TagIndex> {
        let tag_indexes = self.tag_indexes.read().expect("获取标签索引读锁失败");
        tag_indexes
            .get(&space_id)
            .map(|indexes| {
                indexes
                    .values()
                    .filter(|index| index.tag_name == tag_name)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_edge_indexes_for_edge_type(
        &self,
        space_id: i64,
        edge_type_name: &str,
    ) -> Vec<EdgeIndex> {
        let edge_indexes = self.edge_indexes.read().expect("获取边索引读锁失败");
        edge_indexes
            .get(&space_id)
            .map(|indexes| {
                indexes
                    .values()
                    .filter(|index| index.edge_type_name == edge_type_name)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn remove_tag_index(&self, space_id: i64, index_id: i64) -> Result<()> {
        let mut tag_indexes = self
            .tag_indexes
            .write()
            .map_err(|e| anyhow!("获取标签索引写锁失败: {}", e))?;
        if let Some(space_indexes) = tag_indexes.get_mut(&space_id) {
            if let Some(index) = space_indexes.remove(&index_id) {
                let mut tag_index_names = self
                    .tag_index_names
                    .write()
                    .map_err(|e| anyhow!("获取标签索引名写锁失败: {}", e))?;
                tag_index_names.remove(&index.index_name);
            }
        }
        Ok(())
    }

    pub fn remove_edge_index(&self, space_id: i64, index_id: i64) -> Result<()> {
        let mut edge_indexes = self
            .edge_indexes
            .write()
            .map_err(|e| anyhow!("获取边索引写锁失败: {}", e))?;
        if let Some(space_indexes) = edge_indexes.get_mut(&space_id) {
            if let Some(index) = space_indexes.remove(&index_id) {
                let mut edge_index_names = self
                    .edge_index_names
                    .write()
                    .map_err(|e| anyhow!("获取边索引名写锁失败: {}", e))?;
                edge_index_names.remove(&index.index_name);
            }
        }
        Ok(())
    }

    pub fn tag_index_exists(&self, space_id: i64, index_name: &str) -> bool {
        let tag_indexes = self.tag_indexes.read().expect("获取标签索引读锁失败");
        tag_indexes
            .get(&space_id)
            .map(|indexes| indexes.values().any(|index| index.index_name == index_name))
            .unwrap_or(false)
    }

    pub fn edge_index_exists(&self, space_id: i64, index_name: &str) -> bool {
        let edge_indexes = self.edge_indexes.read().expect("获取边索引读锁失败");
        edge_indexes
            .get(&space_id)
            .map(|indexes| indexes.values().any(|index| index.index_name == index_name))
            .unwrap_or(false)
    }

    pub fn get_index_count(&self, space_id: i64) -> (usize, usize) {
        let tag_indexes = self.tag_indexes.read().expect("获取标签索引读锁失败");
        let edge_indexes = self.edge_indexes.read().expect("获取边索引读锁失败");

        let tag_count = tag_indexes
            .get(&space_id)
            .map(|indexes| indexes.len())
            .unwrap_or(0);
        let edge_count = edge_indexes
            .get(&space_id)
            .map(|indexes| indexes.len())
            .unwrap_or(0);

        (tag_count, edge_count)
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_manager_creation() {
        let index_manager = IndexManager::new();
        let (tag_count, edge_count) = index_manager.get_index_count(1);
        assert_eq!(tag_count, 0);
        assert_eq!(edge_count, 0);
    }

    #[test]
    fn test_add_tag_index() {
        let index_manager = IndexManager::new();

        let index = TagIndex {
            index_id: 1,
            space_id: 1,
            tag_name: "user".to_string(),
            index_name: "user_name_index".to_string(),
            fields: vec!["name".to_string()],
        };

        let result = index_manager.add_tag_index(index.clone());
        assert!(result.is_ok());

        let retrieved_index = index_manager.get_tag_index(1, 1);
        assert_eq!(retrieved_index, Some(index.clone()));

        let retrieved_index_by_name = index_manager.get_tag_index_by_name("user_name_index");
        assert_eq!(retrieved_index_by_name, Some(index));
    }

    #[test]
    fn test_add_edge_index() {
        let index_manager = IndexManager::new();

        let index = EdgeIndex {
            index_id: 1,
            space_id: 1,
            edge_type_name: "follows".to_string(),
            index_name: "follows_rank_index".to_string(),
            fields: vec!["rank".to_string()],
        };

        let result = index_manager.add_edge_index(index.clone());
        assert!(result.is_ok());

        let retrieved_index = index_manager.get_edge_index(1, 1);
        assert_eq!(retrieved_index, Some(index.clone()));

        let retrieved_index_by_name = index_manager.get_edge_index_by_name("follows_rank_index");
        assert_eq!(retrieved_index_by_name, Some(index));
    }

    #[test]
    fn test_get_all_tag_indexes() {
        let index_manager = IndexManager::new();

        index_manager
            .add_tag_index(TagIndex {
                index_id: 1,
                space_id: 1,
                tag_name: "user".to_string(),
                index_name: "user_name_index".to_string(),
                fields: vec!["name".to_string()],
            })
            .expect("add_tag_index should succeed");

        index_manager
            .add_tag_index(TagIndex {
                index_id: 2,
                space_id: 1,
                tag_name: "user".to_string(),
                index_name: "user_email_index".to_string(),
                fields: vec!["email".to_string()],
            })
            .expect("add_tag_index should succeed");

        let indexes = index_manager.get_all_tag_indexes(1);
        assert_eq!(indexes.len(), 2);
        
        // Verify the content of retrieved indexes
        let expected_indexes = vec![
            TagIndex {
                index_id: 1,
                space_id: 1,
                tag_name: "user".to_string(),
                index_name: "user_name_index".to_string(),
                fields: vec!["name".to_string()],
            },
            TagIndex {
                index_id: 2,
                space_id: 1,
                tag_name: "user".to_string(),
                index_name: "user_email_index".to_string(),
                fields: vec!["email".to_string()],
            },
        ];
        assert_eq!(indexes, expected_indexes);
        assert_eq!(indexes.len(), 2);
    }

    #[test]
    fn test_get_tag_indexes_for_tag() {
        let index_manager = IndexManager::new();

        index_manager
            .add_tag_index(TagIndex {
                index_id: 1,
                space_id: 1,
                tag_name: "user".to_string(),
                index_name: "user_name_index".to_string(),
                fields: vec!["name".to_string()],
            })
            .expect("add_tag_index should succeed");

        index_manager
            .add_tag_index(TagIndex {
                index_id: 2,
                space_id: 1,
                tag_name: "post".to_string(),
                index_name: "post_title_index".to_string(),
                fields: vec!["title".to_string()],
            })
            .expect("add_tag_index should succeed");

        let user_indexes = index_manager.get_tag_indexes_for_tag(1, "user");
        assert_eq!(user_indexes.len(), 1);
        assert_eq!(user_indexes[0].tag_name, "user");

        let post_indexes = index_manager.get_tag_indexes_for_tag(1, "post");
        assert_eq!(post_indexes.len(), 1);
        assert_eq!(post_indexes[0].tag_name, "post");
    }

    #[test]
    fn test_remove_tag_index() {
        let index_manager = IndexManager::new();

        let index = TagIndex {
            index_id: 1,
            space_id: 1,
            tag_name: "user".to_string(),
            index_name: "user_name_index".to_string(),
            fields: vec!["name".to_string()],
        };

        index_manager.add_tag_index(index.clone()).expect("add_tag_index should succeed");
        assert!(index_manager.get_tag_index(1, 1).is_some());

        let result = index_manager.remove_tag_index(1, 1);
        assert!(result.is_ok());
        assert!(index_manager.get_tag_index(1, 1).is_none());
        assert!(index_manager.get_tag_index_by_name("user_name_index").is_none());
    }

    #[test]
    fn test_tag_index_exists() {
        let index_manager = IndexManager::new();

        assert!(!index_manager.tag_index_exists(1, "user_name_index"));

        index_manager
            .add_tag_index(TagIndex {
                index_id: 1,
                space_id: 1,
                tag_name: "user".to_string(),
                index_name: "user_name_index".to_string(),
                fields: vec!["name".to_string()],
            })
            .expect("add_tag_index should succeed");

        assert!(index_manager.tag_index_exists(1, "user_name_index"));
        assert!(!index_manager.tag_index_exists(2, "user_name_index"));
    }

    #[test]
    fn test_get_index_count() {
        let index_manager = IndexManager::new();

        index_manager
            .add_tag_index(TagIndex {
                index_id: 1,
                space_id: 1,
                tag_name: "user".to_string(),
                index_name: "user_name_index".to_string(),
                fields: vec!["name".to_string()],
            })
            .expect("add_tag_index should succeed");

        index_manager
            .add_tag_index(TagIndex {
                index_id: 2,
                space_id: 1,
                tag_name: "post".to_string(),
                index_name: "post_title_index".to_string(),
                fields: vec!["title".to_string()],
            })
            .expect("add_tag_index should succeed");

        index_manager
            .add_edge_index(EdgeIndex {
                index_id: 1,
                space_id: 1,
                edge_type_name: "follows".to_string(),
                index_name: "follows_rank_index".to_string(),
                fields: vec!["rank".to_string()],
            })
            .expect("add_edge_index should succeed");

        let (tag_count, edge_count) = index_manager.get_index_count(1);
        assert_eq!(tag_count, 2);
        assert_eq!(edge_count, 1);
    }
}
